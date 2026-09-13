#!/usr/bin/env python3
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from execution_evidence import build_execution_summary


class ExecutionEvidenceTest(unittest.TestCase):
    def test_explicit_counters_and_measured_usage_are_preserved(self):
        resolved = {
            "task_id": "task-1",
            "policy_family": "rri",
            "policy_version": "rri-v2",
            "rri": 30,
            "band": "Moderate",
            "execution_mode": "local",
            "logical_binding": {"binding_id": "local-implementer", "role": "local-implementer"},
            "runtime_preset": {"model": "model-a"},
        }
        result = {
            "status": "success",
            "transcript": [
                {"event": "repair_diagnostic"},
                {"event": "test_result", "result": {"passed": False}},
                {"event": "test_result", "result": {"passed": True}},
            ],
        }
        usage = [
            {"invocation_index": 1, "prompt_tokens": 10, "response_tokens": 4, "done_reason": "stop"},
            {"invocation_index": 2, "prompt_tokens": 12, "response_tokens": 6, "done_reason": "stop"},
        ]

        summary = build_execution_summary(
            execution_session_id="session-1",
            resolved_execution=resolved,
            result=result,
            usage_records=usage,
        )

        self.assertEqual(summary["counters"]["model_invocations"], 2)
        self.assertEqual(summary["counters"]["repair_attempts"], 1)
        self.assertEqual(summary["counters"]["test_attempts"], 2)
        self.assertEqual(summary["usage"]["prompt_tokens"], 22)
        self.assertEqual(summary["usage"]["output_tokens"], 10)

    def test_missing_usage_is_not_fabricated(self):
        summary = build_execution_summary(
            execution_session_id="session-2",
            resolved_execution={"task_id": "task-2"},
            result={"status": "success", "transcript": []},
            usage_records=[{"invocation_index": 1, "prompt_tokens": None, "response_tokens": None}],
        )
        self.assertIsNone(summary["usage"]["prompt_tokens"])
        self.assertIsNone(summary["usage"]["output_tokens"])

    def test_fallback_receipt_is_referenced_not_copied(self):
        result = {
            "status": "budget_exhausted",
            "transcript": [],
            "fallback_selection_artifact": "out.fallback-selection.json",
            "fallback_selection": {
                "status": "fallback_authorized",
                "authorization_receipt": {"receipt_sha256": "abc123", "selected_model": "external-model"},
            },
        }
        summary = build_execution_summary(
            execution_session_id="session-3",
            resolved_execution={"task_id": "task-3"},
            result=result,
            usage_records=[],
        )
        self.assertEqual(summary["counters"]["fallback_handoffs"], 1)
        self.assertEqual(summary["evidence_refs"]["fallback_selection_artifact"], "out.fallback-selection.json")
        self.assertEqual(summary["evidence_refs"]["authorization_receipt_sha256"], "abc123")
        self.assertNotIn("selected_model", summary["evidence_refs"])

    def test_initial_cloud_only_rejection_is_a_handoff_requirement(self):
        summary = build_execution_summary(
            execution_session_id="session-4",
            resolved_execution={"task_id": "task-4"},
            result={"status": "local_execution_rejected", "transcript": []},
            usage_records=[],
        )
        self.assertEqual(summary["fallback_handoff_status"], "cloud_handoff_required")
        self.assertEqual(summary["counters"]["fallback_handoffs"], 1)


if __name__ == "__main__":
    unittest.main()
