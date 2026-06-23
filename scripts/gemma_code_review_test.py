#!/usr/bin/env python3
"""Unit tests for gemma-code-review.py."""

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest


_SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "gemma-code-review.py")
_spec = importlib.util.spec_from_file_location("gemma_code_review", _SCRIPT)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)


def _packet():
    return "\n".join([
        "# Review packet",
        "```diff",
        "diff --git a/scripts/a.py b/scripts/a.py",
        "--- a/scripts/a.py",
        "+++ b/scripts/a.py",
        "@@ -1 +1 @@",
        "-old",
        "+new",
        "```",
    ])


def _response(status="FINDINGS", finding_path="scripts/a.py", severity="major"):
    lines = [
        f"STATUS: {status}",
        "SUMMARY: reviewed",
    ]
    if status == "FINDINGS":
        lines.extend([
            "=== FINDING START ===",
            f"PATH: {finding_path}",
            "LINE: 12",
            f"SEVERITY: {severity}",
            "DETAIL: concrete issue",
            "SUGGESTION: fix it",
            "=== FINDING END ===",
        ])
    return "\n".join(lines)


class ChangedPaths(unittest.TestCase):
    def test_changed_paths_from_packet(self):
        self.assertEqual(_mod.changed_paths_from_packet(_packet()), ["scripts/a.py"])

    def test_changed_paths_handles_dev_null(self):
        packet = "diff --git a/scripts/a.py b/scripts/a.py\n--- /dev/null\n+++ b/scripts/a.py"
        self.assertEqual(_mod.changed_paths_from_packet(packet), ["scripts/a.py"])


class BuildReviewPayload(unittest.TestCase):
    def test_prompt_is_read_only(self):
        payload = _mod.build_review_payload("model", "packet", 16384, 4096, 0.1, False)
        system = payload["messages"][0]["content"]
        self.assertIn("read-only", system)
        self.assertIn("Do not approve", system)
        self.assertIn("output file bodies", system)
        self.assertIn("STATUS: PASS", system)

    def test_generation_options_are_shared_shape(self):
        payload = _mod.build_review_payload("model", "packet", 8192, 2048, 0.25, True)
        self.assertTrue(payload["stream"])
        self.assertTrue(payload["think"])
        self.assertEqual(payload["options"]["temperature"], 0.25)
        self.assertEqual(payload["options"]["num_ctx"], 8192)
        self.assertEqual(payload["options"]["num_predict"], 2048)


class ParseReviewResponse(unittest.TestCase):
    def test_pass_without_findings(self):
        result = _mod.parse_review_response(
            "STATUS: PASS\nSUMMARY: clean",
            ["scripts/a.py"],
        )
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["findings"], [])

    def test_status_without_prefix_accepted(self):
        result = _mod.parse_review_response(
            "PASS\nSUMMARY: clean",
            ["scripts/a.py"],
        )
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["findings"], [])

    def test_finding_in_scope(self):
        result = _mod.parse_review_response(_response(), ["scripts/a.py"])
        self.assertEqual(result["status"], "findings")
        self.assertEqual(result["findings"][0]["scope"], "in-scope")
        self.assertEqual(result["findings"][0]["line"], 12)

    def test_finding_out_of_scope_is_labeled_not_dropped(self):
        result = _mod.parse_review_response(
            _response(finding_path="scripts/other.py"),
            ["scripts/a.py"],
        )
        self.assertEqual(result["findings"][0]["scope"], "out-of-scope")
        self.assertEqual(result["findings"][0]["path"], "scripts/other.py")

    def test_blocked_allowed_without_findings(self):
        result = _mod.parse_review_response(
            "STATUS: BLOCKED\nSUMMARY: packet incomplete",
            ["scripts/a.py"],
        )
        self.assertEqual(result["status"], "blocked")

    def test_invalid_severity_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.parse_review_response(_response(severity="critical"), ["scripts/a.py"])
        self.assertIn("invalid severity", str(ctx.exception))

    def test_patch_like_output_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.parse_review_response(
                "STATUS: FINDINGS\nSUMMARY: x\n"
                "=== FINDING START ===\n"
                "PATH: scripts/a.py\n"
                "LINE: 1\n"
                "SEVERITY: major\n"
                "DETAIL: diff --git a/x b/x\n"
                "SUGGESTION: no\n"
                "=== FINDING END ===",
                ["scripts/a.py"],
            )
        self.assertIn("patch-like", str(ctx.exception))

    def test_pass_with_findings_rejected(self):
        content = "\n".join([
            "STATUS: PASS",
            "SUMMARY: x",
            "=== FINDING START ===",
            "PATH: scripts/a.py",
            "LINE: 1",
            "SEVERITY: minor",
            "DETAIL: issue",
            "SUGGESTION: fix",
            "=== FINDING END ===",
        ])
        with self.assertRaises(RuntimeError) as ctx:
            _mod.parse_review_response(content, ["scripts/a.py"])
        self.assertIn("PASS", str(ctx.exception))

    def test_findings_without_finding_block_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.parse_review_response("STATUS: FINDINGS\nSUMMARY: x", ["scripts/a.py"])
        self.assertIn("requires findings", str(ctx.exception))

    def test_missing_end_marker_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.parse_review_response(
                "\n".join([
                    "STATUS: FINDINGS",
                    "SUMMARY: x",
                    "=== FINDING START ===",
                    "PATH: scripts/a.py",
                    "LINE: 1",
                    "SEVERITY: minor",
                    "DETAIL: issue",
                    "SUGGESTION: fix",
                ]),
                ["scripts/a.py"],
            )
        self.assertIn("finding end", str(ctx.exception))


class CliBehavior(unittest.TestCase):
    def run_cli(self, *args, stdin=None, env=None):
        return subprocess.run(
            [sys.executable, _SCRIPT, *args],
            capture_output=True,
            text=True,
            input=stdin,
            env=env,
        )

    def test_dry_run_uses_review_model_env(self):
        env = os.environ.copy()
        env["DUBBRIDGE_REVIEW_MODEL"] = "review-model"
        r = self.run_cli("-", "--dry-run", stdin=_packet(), env=env)
        self.assertEqual(r.returncode, 0, r.stderr)
        payload = json.loads(r.stdout)
        self.assertEqual(payload["model"], "review-model")

    def test_dry_run_falls_back_to_low_rri_model_env(self):
        env = os.environ.copy()
        env.pop("DUBBRIDGE_REVIEW_MODEL", None)
        env["DUBBRIDGE_LOW_RRI_MODEL"] = "low-model"
        r = self.run_cli("-", "--dry-run", stdin=_packet(), env=env)
        self.assertEqual(r.returncode, 0, r.stderr)
        payload = json.loads(r.stdout)
        self.assertEqual(payload["model"], "low-model")

    def test_empty_packet_exits_1(self):
        r = self.run_cli("-", stdin=" \n")
        self.assertEqual(r.returncode, 1)
        self.assertIn("empty", r.stderr)

    def test_dry_run_think_false_by_default(self):
        r = self.run_cli("-", "--dry-run", stdin=_packet())
        self.assertEqual(r.returncode, 0, r.stderr)
        payload = json.loads(r.stdout)
        self.assertFalse(payload["think"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
