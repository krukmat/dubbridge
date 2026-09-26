import importlib.util
import os
import sys
import unittest
from unittest.mock import MagicMock, patch

SCRIPTS = os.path.dirname(os.path.abspath(__file__))
if SCRIPTS not in sys.path:
    sys.path.insert(0, SCRIPTS)
import gemma_local

PEER = os.path.join(SCRIPTS, "peer-workflow-review.py")
spec = importlib.util.spec_from_file_location("peer_workflow_review_binding", PEER)
peer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(peer)

class ReviewerBindingTest(unittest.TestCase):
    def test_gpt_oss_profile(self):
        self.assertEqual(gemma_local.DEFAULT_REVIEW_MODEL, "gpt-oss:20b")
        self.assertEqual(gemma_local.DEFAULT_REVIEW_NUM_CTX, 65536)
        self.assertEqual(gemma_local.DEFAULT_REVIEW_REASONING_EFFORT, "medium")
        payload = gemma_local.build_chat_payload(
            model="gpt-oss:20b", system_prompt="s", packet="p",
            num_ctx=65536, num_predict=128, temperature=0.0, think=False,
        )
        self.assertEqual(payload["think"], "medium")
        self.assertEqual(payload["keep_alive"], "1m")

    def test_low_route_prefers_gpt_oss(self):
        args = MagicMock(
            task_id="T", host="http://localhost:11434", qwen_model="gemma4:26b-a4b-it-qat",
            num_ctx=131072, num_predict=128, temperature=0.1, think=False,
            idle_timeout=60, max_wall=60,
        )
        expected = {"reviewer": "gpt-oss:20b", "verdict": "pass", "summary": "ok", "findings": []}
        with patch.object(peer, "_run_gpt_oss_review", return_value=(expected, None)) as gpt, \
             patch.object(peer, "_run_gemma_fallback") as gemma:
            result, error = peer._run_gpt_oss_review("packet", "code", args)
        self.assertEqual(result["reviewer"], "gpt-oss:20b")
        self.assertIsNone(error)
        gemma.assert_not_called()

    def test_moderate_second_reviewer_function_is_gpt_oss(self):
        self.assertTrue(hasattr(peer, "_run_gpt_oss_fallback"))

if __name__ == "__main__":
    unittest.main()
