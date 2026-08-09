#!/usr/bin/env python3
"""Unit tests for the shared fallback-model selection checkpoint."""

import json
import os
import tempfile
import unittest
import argparse

import fallback_selection as selection


class RecommendationTest(unittest.TestCase):
    def test_d14_uses_balanced_default(self):
        self.assertEqual(
            selection.recommend_fallback(89, "d14", "reviewer-chain-unusable"),
            selection.Recommendation("gpt-5.6-terra", "medium"),
        )

    def test_cloud_implementer_matrix(self):
        cases = (
            (25, "operational-only", "gpt-5.6-luna", "low"),
            (40, "capability-risk", "gpt-5.6-terra", "medium"),
            (55, "operational-only", "gpt-5.6-terra", "high"),
            (55, "capability-risk", "gpt-5.6-sol", "high"),
            (70, "capability-risk", "gpt-5.6-sol", "high"),
            (85, "capability-risk", "gpt-5.6-sol", "xhigh"),
            (100, "capability-risk", "gpt-5.6-sol", "max"),
        )
        for rri, trigger_kind, model, effort in cases:
            with self.subTest(rri=rri, trigger_kind=trigger_kind):
                self.assertEqual(
                    selection.recommend_fallback(
                        rri, "cloud-implementer", trigger_kind
                    ),
                    selection.Recommendation(model, effort),
                )

    def test_invalid_trigger_and_rri_fail(self):
        with self.assertRaisesRegex(selection.FallbackSelectionError, "trigger_kind"):
            selection.recommend_fallback(44, "cloud-implementer", "timeout")
        with self.assertRaisesRegex(selection.FallbackSelectionError, "rri"):
            selection.recommend_fallback(-1, "cloud-implementer", "operational-only")


class PacketHashTest(unittest.TestCase):
    def test_mapping_order_does_not_change_hash(self):
        left = {"a": 1, "b": 2}
        right = {"b": 2, "a": 1}
        self.assertEqual(
            selection.packet_sha256(left), selection.packet_sha256(right)
        )

    def test_bytes_and_utf8_text_hash_exact_bytes(self):
        self.assertEqual(
            selection.packet_sha256(b"evidence"),
            selection.packet_sha256("evidence"),
        )

    def test_non_utf8_encodable_text_fails(self):
        with self.assertRaisesRegex(
            selection.FallbackSelectionError,
            "packet: text must be UTF-8 encodable",
        ):
            selection.packet_sha256("\ud800")

    def test_non_serializable_packet_fails(self):
        with self.assertRaisesRegex(selection.FallbackSelectionError, "packet"):
            selection.packet_sha256({"bad": object()})


class CheckpointTest(unittest.TestCase):
    def _build(self, **overrides):
        values = {
            "task_id": "FMC-1",
            "phase": "phase-2-review",
            "trigger": "qwen_and_gemma_unusable",
            "role": "d14",
            "rri": 44,
            "packet": {"diff": "abc", "criteria": "HP-1"},
            "trigger_kind": "reviewer-chain-unusable",
            "now": "2026-08-09T10:00:00Z",
        }
        values.update(overrides)
        return selection.build_checkpoint(**values)

    def test_human_select_without_selection_waits(self):
        checkpoint = self._build()
        self.assertEqual(
            checkpoint["status"], "awaiting_fallback_selection"
        )
        self.assertIsNone(checkpoint["selected_model"])
        self.assertNotIn("authorization_receipt", checkpoint)

    def test_complete_human_selection_is_authorized_and_validates(self):
        packet = {"diff": "abc", "criteria": "HP-1"}
        checkpoint = self._build(
            packet=packet,
            selected_model="gpt-5.6-sol",
            selected_reasoning_effort="high",
            selected_by="matias",
        )
        self.assertEqual(checkpoint["status"], "fallback_authorized")
        self.assertTrue(checkpoint["authorization_receipt"]["receipt_sha256"])
        selection.validate_authorized_checkpoint(checkpoint, packet)

    def test_complete_preauthorization_is_authorized(self):
        checkpoint = self._build(
            selection_mode="preauthorized",
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        self.assertEqual(checkpoint["status"], "fallback_authorized")

    def test_incomplete_preauthorization_fails_closed(self):
        with self.assertRaisesRegex(
            selection.FallbackSelectionError, "selected_reasoning_effort"
        ):
            self._build(
                selection_mode="preauthorized",
                selected_model="gpt-5.6-terra",
                selected_by="matias",
            )

    def test_partial_human_selection_fails_closed(self):
        with self.assertRaisesRegex(
            selection.FallbackSelectionError, "selected_reasoning_effort"
        ):
            self._build(selected_model="gpt-5.6-terra")

    def test_changed_packet_invalidates_receipt(self):
        checkpoint = self._build(
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        with self.assertRaisesRegex(
            selection.FallbackSelectionError, "packet_sha256"
        ):
            selection.validate_authorized_checkpoint(
                checkpoint, {"diff": "changed", "criteria": "HP-1"}
            )

    def test_tampered_receipt_invalidates_checkpoint(self):
        checkpoint = self._build(
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        checkpoint["authorization_receipt"]["selected_model"] = "gpt-5.6-sol"
        with self.assertRaisesRegex(
            selection.FallbackSelectionError, "receipt_sha256"
        ):
            selection.validate_authorized_checkpoint(
                checkpoint, {"diff": "abc", "criteria": "HP-1"}
            )

    def test_validator_rejects_non_authorized_or_incomplete_checkpoint(self):
        packet = {"diff": "abc", "criteria": "HP-1"}
        with self.assertRaisesRegex(selection.FallbackSelectionError, "checkpoint"):
            selection.validate_authorized_checkpoint([], packet)
        awaiting = self._build(packet=packet)
        with self.assertRaisesRegex(selection.FallbackSelectionError, "status"):
            selection.validate_authorized_checkpoint(awaiting, packet)
        authorized = self._build(
            packet=packet,
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        authorized.pop("authorization_receipt")
        with self.assertRaisesRegex(
            selection.FallbackSelectionError, "authorization_receipt"
        ):
            selection.validate_authorized_checkpoint(authorized, packet)

    def test_validator_rejects_receipt_packet_or_checkpoint_field_mismatch(self):
        packet = {"diff": "abc", "criteria": "HP-1"}
        checkpoint = self._build(
            packet=packet,
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        receipt = checkpoint["authorization_receipt"]
        receipt["packet_sha256"] = "0" * 64
        receipt["receipt_sha256"] = selection._canonical_sha256(
            {key: value for key, value in receipt.items() if key != "receipt_sha256"}
        )
        with self.assertRaisesRegex(selection.FallbackSelectionError, "packet_sha256"):
            selection.validate_authorized_checkpoint(checkpoint, packet)

        checkpoint = self._build(
            packet=packet,
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        checkpoint["selected_by"] = "someone-else"
        with self.assertRaisesRegex(selection.FallbackSelectionError, "selected_by"):
            selection.validate_authorized_checkpoint(checkpoint, packet)

    def test_unsupported_values_raise_stable_errors(self):
        for field, value in (
            ("role", "implementer"),
            ("selection_mode", "automatic"),
            ("selected_reasoning_effort", "ultra"),
        ):
            with self.subTest(field=field):
                with self.assertRaisesRegex(
                    selection.FallbackSelectionError, field
                ):
                    self._build(
                        **{
                            field: value,
                            "selected_model": "gpt-5.6-terra",
                            "selected_reasoning_effort": (
                                value
                                if field == "selected_reasoning_effort"
                                else "medium"
                            ),
                            "selected_by": "matias",
                        }
                    )


class ArtifactWriterTest(unittest.TestCase):
    def test_writer_creates_json_and_validator_accepts_it(self):
        checkpoint = selection.build_checkpoint(
            task_id="FMC-1",
            phase="implementation",
            trigger="ollama_unavailable",
            role="cloud-implementer",
            rri=40,
            packet=b"attempt evidence",
            trigger_kind="operational-only",
            selection_mode="preauthorized",
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
            now="2026-08-09T10:00:00Z",
        )
        with tempfile.TemporaryDirectory() as directory:
            path = os.path.join(directory, "nested", "selection.json")
            selection.write_checkpoint(checkpoint, path)
            with open(path, encoding="utf-8") as stream:
                persisted = json.load(stream)
        selection.validate_authorized_checkpoint(persisted, b"attempt evidence")

    def test_default_path_and_cli_argument_adapter(self):
        self.assertEqual(
            selection.default_checkpoint_path("review.json"),
            "review.fallback-selection.json",
        )
        self.assertEqual(
            selection.default_checkpoint_path("review"),
            "review.fallback-selection.json",
        )

        parser = argparse.ArgumentParser()
        selection.add_cli_arguments(parser)
        args = parser.parse_args(
            [
                "--fallback-mode",
                "preauthorized",
                "--fallback-model",
                "gpt-5.6-terra",
                "--fallback-reasoning-effort",
                "medium",
                "--fallback-selected-by",
                "matias",
            ]
        )
        checkpoint = selection.build_checkpoint_from_args(
            args,
            task_id="FMC-1",
            phase="implementation",
            trigger="ollama_unavailable",
            role="cloud-implementer",
            rri=40,
            packet="evidence",
            trigger_kind="operational-only",
        )
        self.assertEqual(checkpoint["status"], "fallback_authorized")

    def test_build_without_fixed_time_records_utc_timestamp(self):
        checkpoint = selection.build_checkpoint(
            task_id="FMC-1",
            phase="implementation",
            trigger="ollama_unavailable",
            role="cloud-implementer",
            rri=40,
            packet="evidence",
            trigger_kind="operational-only",
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="matias",
        )
        self.assertTrue(
            checkpoint["authorization_receipt"]["authorized_at"].endswith("Z")
        )


if __name__ == "__main__":
    unittest.main()
