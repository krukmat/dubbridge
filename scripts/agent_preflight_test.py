#!/usr/bin/env python3
"""Tests for agent-preflight.py."""
from __future__ import annotations

import hashlib
import importlib.util
import tempfile
import unittest
from pathlib import Path


SPEC = importlib.util.spec_from_file_location(
    "agent_preflight",
    Path(__file__).parent / "agent-preflight.py",
)
agent_preflight = importlib.util.module_from_spec(SPEC)  # type: ignore[arg-type]
SPEC.loader.exec_module(agent_preflight)  # type: ignore[union-attr]


class AgentPreflightTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_hp1_mark_then_check_passes(self):
        path = agent_preflight.mark_preflight(self.root)

        self.assertTrue(path.exists())
        data = agent_preflight.check_preflight(self.root)

        self.assertEqual(data["repo_root"], str(self.root.resolve()))
        self.assertEqual(data["version"], agent_preflight.SCRIPT_VERSION)

    def test_hp2_summary_names_required_workflow_rules(self):
        summary = agent_preflight.preflight_summary()

        self.assertIn("AGENT_WORKFLOW_GUIDE.md", summary)
        self.assertIn("docs/architecture.md", summary)
        self.assertIn("ADRs", summary)
        self.assertIn("docs/plan/roadmap.md", summary)
        self.assertIn("BDD/product docs", summary)
        self.assertIn("scripts/rri.py", summary)
        self.assertIn("RRI 26+", summary)
        self.assertIn("DESIGN.md", summary)
        self.assertIn("Gemma Reviewer / D14", summary)

    def test_ec1_check_fails_when_sentinel_missing(self):
        with self.assertRaises(agent_preflight.PreflightError) as ctx:
            agent_preflight.check_preflight(self.root)

        self.assertIn("Missing", str(ctx.exception))
        self.assertIn("--mark", str(ctx.exception))

    def test_ec2_check_fails_for_different_repo_root(self):
        other_root = self.root / "other"
        other_root.mkdir()
        agent_preflight.mark_preflight(other_root)
        sentinel = agent_preflight.sentinel_path(other_root)
        local_sentinel = agent_preflight.sentinel_path(self.root)
        local_sentinel.parent.mkdir(parents=True, exist_ok=True)
        local_sentinel.write_text(sentinel.read_text(encoding="utf-8"), encoding="utf-8")

        with self.assertRaises(agent_preflight.PreflightError) as ctx:
            agent_preflight.check_preflight(self.root)

        self.assertIn("was marked for", str(ctx.exception))
        self.assertIn(str(self.root.resolve()), str(ctx.exception))

    def test_cli_check_returns_nonzero_without_sentinel(self):
        result = agent_preflight.main(["--repo-root", str(self.root), "--check"])

        self.assertEqual(result, 1)

    def test_cli_mark_and_check_returns_zero(self):
        mark_result = agent_preflight.main(["--repo-root", str(self.root), "--mark"])
        check_result = agent_preflight.main(["--repo-root", str(self.root), "--check"])

        self.assertEqual(mark_result, 0)
        self.assertEqual(check_result, 0)


class AgentPreflightV2ReceiptTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.instruction_path = self.root / "INSTRUCTIONS.md"
        self.instruction_path.write_text("do the task\n", encoding="utf-8")
        self.doc_path = self.root / "docs" / "notes.md"
        self.doc_path.parent.mkdir(parents=True, exist_ok=True)
        self.doc_path.write_text("some notes\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def _build_payload(self, **overrides):
        kwargs = dict(
            provider="claude",
            session_id="session-1",
            actor_id="actor-1",
            repo_root=self.root,
            hook_event_name="startup",
            source="hook",
            transcript_path="/tmp/transcript.json",
            native_instruction_mechanism="hook",
            native_instruction_path="INSTRUCTIONS.md",
            document_paths=["docs/notes.md"],
        )
        kwargs.update(overrides)
        return agent_preflight.build_v2_receipt_payload(**kwargs)

    def test_hp1_valid_claude_payload_has_correct_hashes_and_byte_counts(self):
        payload = self._build_payload()

        instruction_bytes = self.instruction_path.read_bytes()
        doc_bytes = self.doc_path.read_bytes()

        self.assertEqual(payload["schema_version"], 2)
        self.assertEqual(
            payload["native_instruction"]["sha256"],
            hashlib.sha256(instruction_bytes).hexdigest(),
        )
        self.assertEqual(payload["native_instruction"]["bytes"], len(instruction_bytes))
        self.assertEqual(len(payload["documents"]), 1)
        self.assertEqual(
            payload["documents"][0]["sha256"], hashlib.sha256(doc_bytes).hexdigest()
        )
        self.assertEqual(payload["documents"][0]["bytes"], len(doc_bytes))
        self.assertEqual(payload["lifecycle"]["hook_event_name"], "startup")

    def test_hp2_different_identity_tuples_yield_different_hashes(self):
        identity_a = agent_preflight.compute_receipt_identity("claude", "session-1", "actor-1")
        identity_b = agent_preflight.compute_receipt_identity("claude", "session-2", "actor-1")
        identity_c = agent_preflight.compute_receipt_identity("codex", "session-1", "actor-1")

        self.assertNotEqual(identity_a, identity_b)
        self.assertNotEqual(identity_a, identity_c)

    def test_ec1_unsupported_provider_rejected_before_payload_construction(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            self._build_payload(provider="other")

        self.assertFalse((self.root / "other-marker").exists())

    def test_ec1_unsupported_lifecycle_event_rejected(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            self._build_payload(hook_event_name="not_a_real_event")

    def test_ec2_malformed_session_id_rejected_without_path_derivation(self):
        for bad_value in ["", "has\x00nul", "has/slash", "has..dotdot"]:
            with self.assertRaises(agent_preflight.ReceiptValidationError):
                self._build_payload(session_id=bad_value)

    def test_ec2_malformed_actor_id_rejected(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            self._build_payload(actor_id="../escape")

    def test_ec3_legacy_v1_sentinel_rejected_with_distinct_error(self):
        legacy_payload = agent_preflight.sentinel_payload(self.root)

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(legacy_payload)

        self.assertIn("legacy v1 sentinel", str(ctx.exception))

    def test_ec3_valid_v2_payload_passes_validation(self):
        payload = self._build_payload()

        agent_preflight.validate_v2_receipt_payload(payload)

    def test_ec4_missing_native_instruction_file_fails_closed(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            self._build_payload(native_instruction_path="missing.md")

        self.assertIn("missing.md", str(ctx.exception))

    def test_ec4_missing_document_file_fails_closed(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            self._build_payload(document_paths=["docs/notes.md", "docs/missing.md"])

        self.assertIn("docs/missing.md", str(ctx.exception))

    def test_validate_v2_receipt_payload_rejects_non_dict(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            agent_preflight.validate_v2_receipt_payload("not-a-dict")

    def test_validate_v2_receipt_payload_rejects_wrong_schema_version(self):
        payload = self._build_payload()
        payload["schema_version"] = 1

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(payload)

        self.assertIn("schema_version", str(ctx.exception))

    def test_validate_v2_receipt_payload_rejects_non_dict_lifecycle(self):
        payload = self._build_payload()
        payload["lifecycle"] = "not-a-dict"

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(payload)

        self.assertIn("lifecycle", str(ctx.exception))

    def test_validate_v2_receipt_payload_rejects_missing_required_key(self):
        payload = self._build_payload()
        del payload["documents"]

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(payload)

        self.assertIn("documents", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
