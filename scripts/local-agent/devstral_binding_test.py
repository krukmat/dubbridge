#!/usr/bin/env python3
"""Regression coverage for the Devstral local-implementer binding."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import cli
import run_local_task as rlt


DEVSTRAL = "devstral-small-2:24b-instruct-2512-q4_K_M"
QWEN = "qwen3.8:27b-mlx"
CONTEXT_128K = 131072


def _card(rri):
    return rlt.TaskCard(
        task_id=f"binding-{rri}",
        spec="binding regression",
        acceptance_tests=[],
        allowed_paths=[],
        rri=rri,
    )


class DevstralBindingRegression(unittest.TestCase):
    def test_low_rri_keeps_qwen(self):
        self.assertEqual(rlt.default_local_agent_model(_card(20)), QWEN)

    def test_moderate_defaults_to_devstral(self):
        self.assertEqual(rlt.default_local_agent_model(_card(30)), DEVSTRAL)
        self.assertEqual(rlt.MED_HIGH_REQUIRED_MODEL, DEVSTRAL)

    def test_rri_41_45_requires_exact_devstral_binding(self):
        limits = rlt.resolve_effective_limits(_card(42))
        self.assertTrue(limits.local_execution_allowed)
        self.assertEqual(limits.required_model, DEVSTRAL)
        self.assertEqual(limits.max_total_turns, rlt.MAX_TOTAL_TURNS)
        self.assertEqual(limits.max_repair_attempts, rlt.MAX_REPAIR_ATTEMPTS)

    def test_rri_46_plus_keeps_cloud_only_behavior(self):
        limits = rlt.resolve_effective_limits(_card(46))
        self.assertFalse(limits.local_execution_allowed)
        self.assertEqual(limits.required_model, DEVSTRAL)

    def test_runner_and_cli_share_128k_context_default(self):
        self.assertEqual(rlt.MODEL_CONTEXT_TOKENS, CONTEXT_128K)
        self.assertEqual(cli._DEFAULT_MODEL_CONTEXT_TOKENS, CONTEXT_128K)


if __name__ == "__main__":
    unittest.main()
