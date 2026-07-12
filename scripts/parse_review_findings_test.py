#!/usr/bin/env python3
"""Unit tests for parse-review-findings.py."""

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest

_SCRIPT = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "parse-review-findings.py"
)
_spec = importlib.util.spec_from_file_location("parse_review_findings", _SCRIPT)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)


def _result(findings=None, reconciliation=None, status="findings", passes=3):
    return {
        "status": status,
        "passes_run": passes,
        "passes_succeeded": passes,
        "summary": "test summary",
        "findings": findings or [],
        "reconciliation": reconciliation or {},
    }


def _finding(severity="minor", path="src/foo.py", line=10, detail="d", suggestion="s"):
    return {
        "severity": severity,
        "path": path,
        "line": line,
        "detail": detail,
        "suggestion": suggestion,
    }


class TestCollectFindings(unittest.TestCase):
    def test_empty_result_returns_no_findings(self):
        data = _result()
        collected = _mod.collect_findings(data)
        self.assertEqual(collected, [])

    def test_top_level_findings_collected(self):
        data = _result(findings=[_finding(severity="major")])
        collected = _mod.collect_findings(data)
        self.assertEqual(len(collected), 1)
        self.assertEqual(collected[0][0], "findings")
        self.assertEqual(collected[0][1]["severity"], "major")

    def test_reconciliation_buckets_all_collected(self):
        recon = {
            "consensus": [_finding(severity="blocking", path="src/a.py")],
            "pass_specific": [_finding(severity="minor", path="src/b.py")],
            "location_inconsistent": [
                _finding(severity="minor", path="src/c.py"),
                _finding(severity="nit", path="src/d.py"),
            ],
            "severity_inconsistent": [_finding(severity="major", path="src/e.py")],
            "likely_false_positive": [_finding(severity="nit", path="src/f.py")],
        }
        data = _result(reconciliation=recon)
        collected = _mod.collect_findings(data)
        self.assertEqual(len(collected), 6)
        buckets = [b for b, _ in collected]
        self.assertIn("consensus", buckets)
        self.assertIn("pass_specific", buckets)
        self.assertIn("location_inconsistent", buckets)
        self.assertIn("severity_inconsistent", buckets)
        self.assertIn("likely_false_positive", buckets)

    def test_combined_top_level_and_reconciliation(self):
        data = _result(
            findings=[_finding(severity="minor")],
            reconciliation={"consensus": [_finding(severity="blocking")]},
        )
        collected = _mod.collect_findings(data)
        self.assertEqual(len(collected), 2)

    def test_duplicate_findings_consolidated_with_source_buckets(self):
        duplicate = _finding(severity="major", path="src/dup.py", line=7)
        data = _result(
            findings=[duplicate],
            reconciliation={
                "consensus": [duplicate],
                "pass_specific": [duplicate],
            },
        )
        collected = _mod.collect_findings(data)
        self.assertEqual(len(collected), 1)
        bucket, finding = collected[0]
        self.assertEqual(bucket, "findings, consensus, pass_specific")
        self.assertEqual(
            finding["source_buckets"],
            ["findings", "consensus", "pass_specific"],
        )

    def test_sorted_blocking_first(self):
        data = _result(
            findings=[_finding(severity="nit"), _finding(severity="blocking")],
        )
        collected = _mod.collect_findings(data)
        self.assertEqual(collected[0][1]["severity"], "blocking")
        self.assertEqual(collected[1][1]["severity"], "nit")

    def test_null_buckets_tolerated(self):
        data = _result(reconciliation={"consensus": None, "pass_specific": []})
        collected = _mod.collect_findings(data)
        self.assertEqual(collected, [])


class TestFormatText(unittest.TestCase):
    def test_no_findings_says_none(self):
        data = _result()
        output = _mod.format_text(data, [])
        self.assertIn("findings: none", output)

    def test_header_uses_compiled_passes_without_degraded(self):
        output = _mod.format_text(_result(), [])
        self.assertIn("passes compiled: 3/3", output)
        self.assertNotIn("degraded", output)

    def test_finding_appears_in_output(self):
        finding = _finding(severity="major", path="src/bar.py", line=42, detail="bad thing")
        collected = [("consensus", finding)]
        output = _mod.format_text(_result(), collected)
        self.assertIn("consensus", output)
        self.assertIn("major", output)
        self.assertIn("src/bar.py:42", output)
        self.assertIn("bad thing", output)

    def test_finding_without_line_number(self):
        finding = {"severity": "minor", "path": "src/x.py", "detail": "no line"}
        collected = [("pass_specific", finding)]
        output = _mod.format_text(_result(), collected)
        self.assertIn("src/x.py", output)
        self.assertNotIn("src/x.py:", output)


class TestMainExitCodes(unittest.TestCase):
    def _run(self, data, extra_args=None):
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as fh:
            json.dump(data, fh)
            path = fh.name
        try:
            cmd = [sys.executable, _SCRIPT, path] + (extra_args or [])
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode, result.stdout
        finally:
            os.unlink(path)

    def test_exit_0_when_no_findings(self):
        code, _ = self._run(_result())
        self.assertEqual(code, 0)

    def test_exit_1_when_top_level_findings(self):
        # 6841b37: only blocking/major findings fail the gate.
        data = _result(findings=[_finding(severity="major")])
        code, _ = self._run(data)
        self.assertEqual(code, 1)

    def test_exit_1_when_reconciliation_findings(self):
        data = _result(reconciliation={"location_inconsistent": [_finding(severity="major")]})
        code, _ = self._run(data)
        self.assertEqual(code, 1)

    def test_no_fail_flag_exits_0_despite_findings(self):
        data = _result(findings=[_finding(severity="blocking")])
        code, _ = self._run(data, extra_args=["--no-fail"])
        self.assertEqual(code, 0)

    def test_exit_2_when_file_missing(self):
        cmd = [sys.executable, _SCRIPT, "/tmp/does-not-exist-dubbridge.json"]
        result = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)

    def test_json_format_output(self):
        data = _result(findings=[_finding(severity="major")])
        _, stdout = self._run(data, extra_args=["--format", "json", "--no-fail"])
        parsed = json.loads(stdout)
        self.assertEqual(parsed["total_findings"], 1)
        self.assertEqual(parsed["findings"][0]["bucket"], "findings")
        self.assertEqual(parsed["findings"][0]["severity"], "major")
        self.assertNotIn("degraded", parsed)


if __name__ == "__main__":
    unittest.main()
