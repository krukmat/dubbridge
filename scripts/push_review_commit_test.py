#!/usr/bin/env python3
"""Unit tests for scripts/push_review_commit.py."""

import importlib.util
import json
import os
import tempfile
import unittest


_SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "push_review_commit.py")
_SPEC = importlib.util.spec_from_file_location("push_review_commit", _SCRIPT)
_MOD = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MOD)


class ReadResult(unittest.TestCase):
    def test_prefers_aggregate_when_multiple_artifacts_exist(self):
        with tempfile.TemporaryDirectory() as tmp:
            with open(os.path.join(tmp, "audit_skipped.json"), "w", encoding="utf-8") as fh:
                json.dump({"sentinel": "audit_skipped", "reason": "docs_only"}, fh)
            with open(os.path.join(tmp, "aggregate.json"), "w", encoding="utf-8") as fh:
                json.dump({"status": "pass", "pipeline": {"conclusion": "success"}}, fh)

            result = _MOD.read_result(tmp)

        self.assertEqual(result["status"], "pass")


class SummarizeResult(unittest.TestCase):
    def test_docs_only_skip_is_not_reported_as_blocked(self):
        result = {
            "sentinel": "audit_skipped",
            "reason": "docs_only",
            "run_info": {"conclusion": "failure"},
        }

        summary = _MOD.summarize_result(result, "(no report generated)")

        self.assertEqual(summary["status"], "skipped")
        self.assertEqual(summary["passes"], "n/a")
        self.assertEqual(summary["quorum"], "n/a")
        self.assertEqual(summary["routing"], "docs_only")
        self.assertEqual(summary["ci_conclusion"], "failure")
        self.assertEqual(summary["action"], "(docs-only push; no report generated)")

    def test_missing_result_keeps_fail_closed_blocked_fallback(self):
        summary = _MOD.summarize_result(None, "(no report generated)")

        self.assertEqual(summary["status"], "blocked")
        self.assertEqual(summary["passes"], "?/?")
        self.assertEqual(summary["quorum"], "?")
        self.assertEqual(summary["routing"], "?")
        self.assertEqual(summary["ci_conclusion"], "?")

    def test_aggregate_result_keeps_existing_summary_shape(self):
        result = {
            "status": "findings",
            "pipeline": {"conclusion": "failure"},
            "audit": {"quorum": "met", "passes_succeeded": 1, "passes_run": 1},
            "candidates": [{"routing": "gemma-developer-dispatch"}],
        }

        summary = _MOD.summarize_result(result, "report-link")

        self.assertEqual(summary["status"], "findings")
        self.assertEqual(summary["passes"], "1/1")
        self.assertEqual(summary["quorum"], "met")
        self.assertEqual(summary["routing"], "gemma-developer-dispatch")
        self.assertEqual(summary["ci_conclusion"], "failure")
        self.assertEqual(summary["action"], "report-link")


if __name__ == "__main__":
    unittest.main()
