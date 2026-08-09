#!/usr/bin/env python3
"""Unit tests for peer-workflow-review.py."""

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from contextlib import ExitStack
from unittest.mock import MagicMock, patch

_SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
if _SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, _SCRIPTS_DIR)

_SCRIPT = os.path.join(_SCRIPTS_DIR, "peer-workflow-review.py")
_spec = importlib.util.spec_from_file_location("peer_workflow_review", _SCRIPT)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)


# ---------------------------------------------------------------------------
# Band / reviewer resolution
# ---------------------------------------------------------------------------

class TestResolveBand(unittest.TestCase):
    def test_low(self):
        self.assertEqual(_mod.resolve_band(0), "Low")
        self.assertEqual(_mod.resolve_band(25), "Low")

    def test_moderate(self):
        self.assertEqual(_mod.resolve_band(26), "Moderate")
        self.assertEqual(_mod.resolve_band(40), "Moderate")

    def test_med_high(self):
        self.assertEqual(_mod.resolve_band(41), "Med-high")
        self.assertEqual(_mod.resolve_band(55), "Med-high")

    def test_complex(self):
        self.assertEqual(_mod.resolve_band(56), "Complex")
        self.assertEqual(_mod.resolve_band(100), "Complex")


class TestNeedsCrossVendor(unittest.TestCase):
    def test_gemma_band(self):
        self.assertFalse(_mod.needs_cross_vendor(40))
        self.assertFalse(_mod.needs_cross_vendor(0))

    def test_cross_vendor_band(self):
        self.assertFalse(_mod.needs_cross_vendor(55))
        self.assertTrue(_mod.needs_cross_vendor(56))
        self.assertTrue(_mod.needs_cross_vendor(70))


class TestNeedsLocalQwenReview(unittest.TestCase):
    def test_not_in_low_band(self):
        self.assertFalse(_mod.needs_local_qwen_review(25))

    def test_qwen_band(self):
        self.assertTrue(_mod.needs_local_qwen_review(26))
        self.assertTrue(_mod.needs_local_qwen_review(55))

    def test_not_in_cross_vendor_band(self):
        self.assertFalse(_mod.needs_local_qwen_review(56))


class TestD14Fallback(unittest.TestCase):
    def test_hyphenated_helper_is_loaded_and_packet_is_isolated(self):
        result = _mod.run_d14_fallback("task packet", "task", "qwen")

        self.assertEqual(result["reviewer"], "d14")
        self.assertEqual(result["verdict"], "d14_required")
        self.assertEqual(
            result["d14_packet"],
            {
                "diff": "",
                "criteria": "task packet",
                "reconciled_findings": [],
            },
        )


class TestReviewExitCode(unittest.TestCase):
    def test_pass_is_success(self):
        self.assertEqual(_mod.review_exit_code("pass"), 0)

    def test_findings_is_success(self):
        self.assertEqual(_mod.review_exit_code("findings"), 0)

    def test_blocked_is_failure(self):
        self.assertEqual(_mod.review_exit_code("blocked"), 1)


class TestResolvePeer(unittest.TestCase):
    def test_claude_code_to_codex(self):
        self.assertEqual(_mod.resolve_peer("claude-code"), "codex")

    def test_claude_to_codex(self):
        self.assertEqual(_mod.resolve_peer("claude"), "codex")

    def test_codex_to_claude(self):
        self.assertEqual(_mod.resolve_peer("codex"), "claude")

    def test_local_provider_to_claude(self):
        self.assertEqual(_mod.resolve_peer("local-provider"), "claude")

    def test_remote_provider_to_claude(self):
        self.assertEqual(_mod.resolve_peer("remote-provider"), "claude")

    def test_unknown_to_claude(self):
        self.assertEqual(_mod.resolve_peer("unknown"), "claude")

    def test_unrecognized_defaults_to_claude(self):
        self.assertEqual(_mod.resolve_peer("some-other-tool"), "claude")

    def test_case_insensitive(self):
        self.assertEqual(_mod.resolve_peer("Claude-Code"), "codex")
        self.assertEqual(_mod.resolve_peer("CODEX"), "claude")


# ---------------------------------------------------------------------------
# CLI availability probe
# ---------------------------------------------------------------------------

class TestPeerCliAvailable(unittest.TestCase):
    def test_unavailable(self):
        with patch("shutil.which", return_value=None):
            self.assertFalse(_mod.peer_cli_available("codex"))

    def test_available(self):
        with patch("shutil.which", return_value="/usr/local/bin/codex"):
            self.assertTrue(_mod.peer_cli_available("codex"))


# ---------------------------------------------------------------------------
# Cross-vendor peer invocation
# ---------------------------------------------------------------------------

class TestInvokePeerCli(unittest.TestCase):
    def test_success(self):
        mock_result = MagicMock()
        mock_result.returncode = 0
        mock_result.stdout = "VERDICT: PASS\nSUMMARY: looks good"
        with patch("subprocess.run", return_value=mock_result):
            ok, output = _mod.invoke_peer_cli("codex", "packet")
        self.assertTrue(ok)
        self.assertIn("VERDICT: PASS", output)

    def test_nonzero_exit(self):
        mock_result = MagicMock()
        mock_result.returncode = 1
        mock_result.stdout = ""
        mock_result.stderr = "auth error"
        with patch("subprocess.run", return_value=mock_result):
            ok, output = _mod.invoke_peer_cli("codex", "packet")
        self.assertFalse(ok)
        self.assertIn("auth error", output)

    def test_file_not_found(self):
        with patch("subprocess.run", side_effect=FileNotFoundError("not found")):
            ok, output = _mod.invoke_peer_cli("codex", "packet")
        self.assertFalse(ok)
        self.assertIn("not found", output)

    def test_timeout(self):
        import subprocess
        with patch("subprocess.run", side_effect=subprocess.TimeoutExpired("codex", 120)):
            ok, output = _mod.invoke_peer_cli("codex", "packet")
        self.assertFalse(ok)


# ---------------------------------------------------------------------------
# Response parsing
# ---------------------------------------------------------------------------

class TestParsePeerResponse(unittest.TestCase):
    def test_pass(self):
        result = _mod._parse_peer_response("VERDICT: PASS\nSUMMARY: ok", "codex", "task")
        self.assertEqual(result["verdict"], "pass")
        self.assertEqual(result["reviewer"], "codex")
        self.assertEqual(result["summary"], "ok")

    def test_blocked(self):
        result = _mod._parse_peer_response("VERDICT: BLOCKED\nSUMMARY: not reviewable", "claude", "code")
        self.assertEqual(result["verdict"], "blocked")

    def test_findings(self):
        result = _mod._parse_peer_response(
            "VERDICT: FINDINGS\nSUMMARY: issues found\nFINDING: missing test",
            "codex", "code",
        )
        self.assertEqual(result["verdict"], "findings")
        self.assertEqual(len(result["findings"]), 1)

    def test_no_verdict_defaults_blocked(self):
        result = _mod._parse_peer_response("SUMMARY: nothing", "codex", "task")
        self.assertEqual(result["verdict"], "blocked")


# ---------------------------------------------------------------------------
# run_cross_vendor_review routing
# ---------------------------------------------------------------------------

_D14_STUB = {"reviewer": "d14", "verdict": "d14_required", "summary": "stub", "findings": [], "d14_packet": {}}


class TestRunCrossVendorReview(unittest.TestCase):
    def test_routes_to_d14_when_cli_unavailable(self):
        with patch.object(_mod, "peer_cli_available", return_value=False), \
             patch.object(_mod, "run_d14_fallback", return_value=_D14_STUB):
            result = _mod.run_cross_vendor_review("packet", "task", "codex")
        self.assertEqual(result["reviewer"], "d14")
        self.assertEqual(result["verdict"], "d14_required")
        self.assertIn("d14_packet", result)

    def test_routes_to_d14_when_invocation_fails(self):
        with patch.object(_mod, "peer_cli_available", return_value=True), \
             patch.object(_mod, "invoke_peer_cli", return_value=(False, "auth error")), \
             patch.object(_mod, "run_d14_fallback", return_value=_D14_STUB):
            result = _mod.run_cross_vendor_review("packet", "code", "codex")
        self.assertEqual(result["reviewer"], "d14")

    def test_returns_peer_result_on_success(self):
        with patch.object(_mod, "peer_cli_available", return_value=True), \
             patch.object(_mod, "invoke_peer_cli", return_value=(True, "VERDICT: PASS\nSUMMARY: ok")):
            result = _mod.run_cross_vendor_review("packet", "task", "codex")
        self.assertEqual(result["reviewer"], "codex")
        self.assertEqual(result["verdict"], "pass")


class TestRunQwenBandReview(unittest.TestCase):
    def _args(self):
        return MagicMock(
            task_id="S-140-T1c",
            host="http://localhost:11434",
            qwen_model="qwen3.6:27b-q4_K_M",
            model="gemma4:26b-a4b-it-qat",
            num_ctx=4096,
            num_predict=128,
            temperature=0.1,
            think=False,
            idle_timeout=60,
            max_wall=60,
        )

    def test_returns_qwen_result_when_primary_succeeds(self):
        qwen_result = {"reviewer": "qwen3.6:27b-q4_K_M", "verdict": "pass", "summary": "ok", "findings": []}
        with patch.object(_mod, "_run_qwen_with_retry", return_value=(qwen_result, None)), \
             patch.object(_mod, "_run_gemma_fallback") as gemma_fallback:
            result = _mod.run_qwen_band_review("packet", "task", self._args())
        self.assertEqual(result["reviewer"], "qwen3.6:27b-q4_K_M")
        gemma_fallback.assert_not_called()

    def test_falls_back_to_gemma_after_qwen_failure(self):
        gemma_result = {"reviewer": "gemma", "verdict": "pass", "summary": "ok", "findings": []}
        with patch.object(_mod, "_run_qwen_with_retry", return_value=(None, "length")), \
             patch.object(_mod, "_run_gemma_fallback", return_value=(gemma_result, None)):
            result = _mod.run_qwen_band_review("packet", "task", self._args())
        self.assertEqual(result["reviewer"], "gemma")

    def test_returns_d14_signal_when_qwen_and_gemma_fail(self):
        d14 = {"reviewer": "d14", "verdict": "d14_required", "summary": "stub", "findings": [], "d14_packet": {}}
        with patch.object(_mod, "_run_qwen_with_retry", return_value=(None, "qwen failed")), \
             patch.object(_mod, "_run_gemma_fallback", return_value=(None, "gemma failed")), \
             patch.object(_mod, "run_d14_fallback", return_value=d14):
            result = _mod.run_qwen_band_review("packet", "task", self._args())
        self.assertEqual(result["reviewer"], "d14")
        self.assertEqual(result["verdict"], "d14_required")


# ---------------------------------------------------------------------------
# Human-selected D14 fallback checkpoint
# ---------------------------------------------------------------------------

class TestD14FallbackSelection(unittest.TestCase):
    def _d14_result(self, phase):
        return {
            "reviewer": "d14",
            "phase": phase,
            "verdict": "d14_required",
            "summary": "reviewer chain unusable",
            "findings": [],
            "d14_packet": {
                "diff": "diff" if phase == "code" else "",
                "criteria": "criteria" if phase == "task" else "",
                "reconciled_findings": [],
            },
        }

    def _run_main(self, rri, phase, mode="human-select", d14_result=None):
        temporary_dir = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_dir.cleanup)
        artifact = os.path.join(temporary_dir.name, "review.json")
        selection_artifact = os.path.join(temporary_dir.name, "selection.json")
        argv = [
            "peer-workflow-review.py",
            "--phase", phase,
            "--rri", str(rri),
            "--caller", "codex",
            "--task-id", "FMC-2",
            "--artifact", artifact,
            "--fallback-selection-artifact", selection_artifact,
            "--fallback-mode", mode,
        ]
        if mode == "preauthorized":
            argv.extend([
                "--fallback-model", "gpt-5.6-terra",
                "--fallback-reasoning-effort", "medium",
                "--fallback-selected-by", "owner",
            ])

        result = d14_result or self._d14_result(phase)
        patches = [
            patch("sys.argv", argv),
            patch.object(_mod.gemma_local, "read_packet", return_value="content"),
        ]
        if rri <= 25:
            patches.append(
                patch.object(
                    _mod,
                    "_run_gemma_fallback",
                    return_value=(None, "gemma unavailable"),
                )
            )
            patches.append(patch.object(_mod, "run_d14_fallback", return_value=result))
        elif rri <= 55:
            patches.append(patch.object(_mod, "run_qwen_band_review", return_value=result))
        else:
            patches.append(patch.object(_mod, "run_cross_vendor_review", return_value=result))

        with ExitStack() as stack:
            for active_patch in patches:
                stack.enter_context(active_patch)
            exit_code = _mod.main()

        with open(artifact, encoding="utf-8") as stream:
            review = json.load(stream)
        selection = None
        if os.path.exists(selection_artifact):
            with open(selection_artifact, encoding="utf-8") as stream:
                selection = json.load(stream)
        return exit_code, review, selection

    def test_all_bands_and_phases_pause_before_d14_without_selection(self):
        for rri in (12, 46, 60):
            for phase in ("task", "code"):
                with self.subTest(rri=rri, phase=phase):
                    exit_code, review, selection = self._run_main(rri, phase)
                    self.assertEqual(exit_code, 3)
                    self.assertEqual(review["verdict"], "awaiting_fallback_selection")
                    self.assertEqual(
                        os.path.basename(review["fallback_selection_artifact"]),
                        "selection.json",
                    )
                    self.assertEqual(review["fallback_selection"], selection)
                    self.assertNotIn("authorization_receipt", selection)

    def test_all_bands_and_phases_relay_exact_preauthorized_selection(self):
        for rri in (12, 46, 60):
            for phase in ("task", "code"):
                with self.subTest(rri=rri, phase=phase):
                    exit_code, review, selection = self._run_main(
                        rri, phase, mode="preauthorized"
                    )
                    self.assertEqual(exit_code, 1)
                    self.assertEqual(review["verdict"], "d14_required")
                    self.assertEqual(review["fallback_selection"], selection)
                    self.assertEqual(selection["selected_model"], "gpt-5.6-terra")
                    self.assertEqual(selection["selected_reasoning_effort"], "medium")
                    self.assertEqual(selection["selected_by"], "owner")
                    self.assertIn("authorization_receipt", selection)
                    self.assertEqual(
                        selection["packet_sha256"],
                        _mod.fallback_selection.packet_sha256(review["d14_packet"]),
                    )

    def test_malformed_d14_packet_fails_closed_without_spawn_authorization(self):
        malformed = self._d14_result("task")
        malformed["d14_packet"]["criteria"] = 123
        exit_code, review, selection = self._run_main(46, "task", d14_result=malformed)

        self.assertEqual(exit_code, 2)
        self.assertEqual(review["verdict"], "blocked")
        self.assertTrue(review["blocked"])
        self.assertIn("integrity", review["summary"].lower())
        self.assertIsNone(selection)

    def test_checkpoint_hash_mismatch_fails_closed(self):
        checkpoint = _mod.fallback_selection.build_checkpoint(
            task_id="FMC-2",
            phase="task",
            trigger="reviewer chain unusable",
            role=_mod.fallback_selection.ROLE_D14,
            rri=46,
            packet={"diff": "", "criteria": "different", "reconciled_findings": []},
            trigger_kind=_mod.fallback_selection.TRIGGER_REVIEWER_UNUSABLE,
            selection_mode=_mod.fallback_selection.MODE_PREAUTHORIZED,
            selected_model="gpt-5.6-terra",
            selected_reasoning_effort="medium",
            selected_by="owner",
        )
        with patch.object(
            _mod.fallback_selection,
            "build_checkpoint_from_args",
            return_value=checkpoint,
        ):
            exit_code, review, selection = self._run_main(
                46, "task", mode="preauthorized"
            )

        self.assertEqual(exit_code, 2)
        self.assertEqual(review["verdict"], "blocked")
        self.assertTrue(review["blocked"])
        self.assertIsNone(selection)

    def test_non_fallback_result_has_no_selection_keys(self):
        qwen_result = {
            "reviewer": "qwen3.6:27b-q4_K_M",
            "phase": "task",
            "verdict": "pass",
            "summary": "ok",
            "findings": [],
        }
        temporary_dir = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_dir.cleanup)
        artifact = os.path.join(temporary_dir.name, "review.json")
        argv = [
            "peer-workflow-review.py", "--phase", "task", "--rri", "46",
            "--task-id", "FMC-2", "--artifact", artifact,
        ]
        with patch("sys.argv", argv), \
             patch.object(_mod.gemma_local, "read_packet", return_value="content"), \
             patch.object(_mod, "run_qwen_band_review", return_value=qwen_result):
            exit_code = _mod.main()

        self.assertEqual(exit_code, 0)
        with open(artifact, encoding="utf-8") as stream:
            review = json.load(stream)
        self.assertEqual(
            {key: value for key, value in review.items() if key != "ts"},
            qwen_result,
        )
        self.assertNotIn("fallback_selection", review)
        self.assertNotIn("fallback_selection_artifact", review)


# ---------------------------------------------------------------------------
# Artifact writing
# ---------------------------------------------------------------------------

class TestWriteArtifact(unittest.TestCase):
    def test_writes_json_with_ts(self):
        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
            path = f.name
        try:
            _mod.write_artifact({"reviewer": "gemma", "verdict": "pass"}, path)
            with open(path) as f:
                data = json.load(f)
            self.assertIn("ts", data)
            self.assertEqual(data["reviewer"], "gemma")
        finally:
            os.unlink(path)

    def test_blocked_artifact_sets_blocked_true(self):
        with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
            path = f.name
        try:
            _mod.write_blocked_artifact("peer and D14 unavailable", "code", path, "codex")
            with open(path) as f:
                data = json.load(f)
            self.assertTrue(data["blocked"])
            self.assertEqual(data["verdict"], "blocked")
        finally:
            os.unlink(path)


# ---------------------------------------------------------------------------
# Default artifact path
# ---------------------------------------------------------------------------

class TestDefaultArtifactPath(unittest.TestCase):
    def test_contains_phase_and_task(self):
        path = _mod.default_artifact_path("PPR-2", "code")
        self.assertIn("peer-code-review", path)
        self.assertIn("ppr-2", path)

    def test_none_task_id(self):
        path = _mod.default_artifact_path(None, "task")
        self.assertIn("unknown", path)


# ---------------------------------------------------------------------------
# Packet builder
# ---------------------------------------------------------------------------

class TestBuildPeerPacket(unittest.TestCase):
    def test_task_phase_mentions_task_card(self):
        packet = _mod._build_peer_packet("task", "some task card", "PPR-2")
        self.assertIn("task card", packet)
        self.assertIn("PPR-2", packet)

    def test_code_phase_mentions_diff(self):
        packet = _mod._build_peer_packet("code", "some diff", "PPR-2")
        self.assertIn("diff", packet)


if __name__ == "__main__":
    unittest.main()
