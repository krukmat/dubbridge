#!/usr/bin/env python3
import os
import sys
import unittest
from types import SimpleNamespace

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from execution_contract import normalize_resolved_execution


class ExecutionContractTest(unittest.TestCase):
    def test_local_contract_has_runtime_preset_without_priority(self):
        card = SimpleNamespace(task_id="t1", rri=30, band="Moderate", policy_version="rri-v2")
        limits = SimpleNamespace(logical_binding="local-implementer", local_execution_allowed=True, max_total_turns=30, max_repair_attempts=2)
        resolved = normalize_resolved_execution(card=card, limits=limits, model="model-a", num_ctx=131072, num_predict=8192).as_dict()
        self.assertEqual(resolved["execution_mode"], "local")
        self.assertEqual(resolved["runtime_preset"]["model"], "model-a")
        self.assertNotIn("priority", resolved["logical_binding"])

    def test_cloud_contract_has_no_local_runtime_preset(self):
        card = SimpleNamespace(task_id="t2", rri=50, band="Med-high")
        limits = SimpleNamespace(local_execution_allowed=False, max_total_turns=30, max_repair_attempts=2)
        resolved = normalize_resolved_execution(card=card, limits=limits, model="unused", num_ctx=131072, num_predict=8192).as_dict()
        self.assertEqual(resolved["execution_mode"], "cloud_handoff_required")
        self.assertEqual(resolved["logical_binding"]["binding_id"], "cloud-handoff")
        self.assertIsNone(resolved["runtime_preset"])

    def test_policy_version_defaults_to_rri_v2(self):
        card = SimpleNamespace(task_id="t3", rri=20, band="Low", policy_version=None)
        limits = SimpleNamespace(local_execution_allowed=True, max_total_turns=30, max_repair_attempts=2)
        resolved = normalize_resolved_execution(card=card, limits=limits, model="model-b", num_ctx=65536, num_predict=8192).as_dict()
        self.assertEqual(resolved["policy_version"], "rri-v2")


if __name__ == "__main__":
    unittest.main()
