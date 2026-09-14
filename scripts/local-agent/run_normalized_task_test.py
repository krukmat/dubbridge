#!/usr/bin/env python3
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import gemma_local
import run_normalized_task as rnt


class NormalizedExecutionObserverTest(unittest.TestCase):
    def test_observer_preserves_result_and_records_usage(self):
        records = []
        expected = gemma_local.StreamChatResult(
            content='{"tool_calls": []}',
            usage=gemma_local.StreamUsage(
                response_tokens=5,
                prompt_tokens=11,
                done_reason="stop",
            ),
        )

        def original(*args, **kwargs):
            return expected

        observed = rnt._observe_stream_chat(original, records)
        actual = observed("url", {}, 1, 1)

        self.assertIs(actual, expected)
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["invocation_index"], 1)
        self.assertEqual(records[0]["prompt_tokens"], 11)
        self.assertEqual(records[0]["response_tokens"], 5)
        self.assertEqual(records[0]["done_reason"], "stop")

    def test_observer_reraises_runtime_errors_and_records_no_fake_usage(self):
        records = []

        def original(*args, **kwargs):
            raise RuntimeError("boom")

        observed = rnt._observe_stream_chat(original, records)
        with self.assertRaises(RuntimeError):
            observed("url", {}, 1, 1)

        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["status"], "error")
        self.assertIsNone(records[0]["prompt_tokens"])
        self.assertIsNone(records[0]["response_tokens"])
        self.assertEqual(records[0]["error_class"], "RuntimeError")


if __name__ == "__main__":
    unittest.main()
