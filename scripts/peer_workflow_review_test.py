#!/usr/bin/env python3
"""Unit tests for peer-workflow-review.py."""

import importlib.util
import json
import os
import sys
import tempfile
import unittest
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
