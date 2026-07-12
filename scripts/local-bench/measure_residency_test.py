#!/usr/bin/env python3
"""Unit tests for measure_residency.py (mocked Ollama HTTP API only)."""

import json
import os
import sys
import tempfile
import unittest
import urllib.error
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import measure_residency as mr


class CycleModel(unittest.TestCase):
    def test_hp1_cycle_produces_all_phase_timings(self):
        # cold-load generate, unload generate, then a resident-check poll
        # that immediately reports unloaded, then reload generate.
        with patch.object(mr, "generate", return_value={}):
            with patch.object(mr, "resident_model_names", return_value=set()):
                phases = mr.cycle_model("http://localhost:11434", "m", timeout=5)

        for key in ("cold_load_s", "unload_s", "reload_s", "total_cycle_s"):
            self.assertIn(key, phases)
            self.assertGreaterEqual(phases[key], 0)
        self.assertTrue(phases["unload_confirmed"])

    def test_wait_for_unload_returns_false_when_model_stays_resident(self):
        with patch.object(mr, "resident_model_names", return_value={"m"}):
            with patch.object(mr, "time") as mock_time:
                # first call sets the deadline reference point, subsequent
                # calls advance past it so the poll loop exits promptly.
                mock_time.monotonic.side_effect = [0, 0, 999]
                mock_time.sleep.return_value = None
                result = mr.wait_for_unload("http://localhost:11434", "m", timeout=5)
        self.assertFalse(result)


class MultiModelRun(unittest.TestCase):
    def test_hp1_two_models_both_succeed(self):
        with patch.object(mr, "generate", return_value={}):
            with patch.object(mr, "resident_model_names", return_value=set()):
                payload = mr.run(["a", "b"], "http://localhost:11434", timeout=5)

        self.assertEqual(payload["errors"], [])
        self.assertEqual(len(payload["results"]), 2)
        for entry, model in zip(payload["results"], ["a", "b"]):
            self.assertEqual(entry["model"], model)
            self.assertFalse(entry["failed"])

    def test_ec1_second_model_fails_records_structured_failure_and_unloads_first(self):
        call_count = {"n": 0}

        def flaky_generate(host, model, keep_alive, timeout):
            call_count["n"] += 1
            if model == "b" and keep_alive == mr.RESIDENT_KEEP_ALIVE:
                raise urllib.error.URLError("model failed to load")
            return {}

        with patch.object(mr, "generate", side_effect=flaky_generate):
            with patch.object(mr, "resident_model_names", return_value=set()):
                payload = mr.run(["a", "b"], "http://localhost:11434", timeout=5)

        self.assertEqual(len(payload["results"]), 2)
        self.assertFalse(payload["results"][0]["failed"])
        self.assertTrue(payload["results"][1]["failed"])
        self.assertIn("error", payload["results"][1])
        self.assertEqual(len(payload["errors"]), 1)
        # best-effort cleanup unload for the failed model must still be attempted
        self.assertGreaterEqual(call_count["n"], 4)

    def test_main_writes_artifact_even_on_failure_and_exits_nonzero(self):
        def flaky_generate(host, model, keep_alive, timeout):
            raise urllib.error.URLError("boom")

        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "result.json")
            with patch.object(mr, "generate", side_effect=flaky_generate):
                with patch.object(mr, "resident_model_names", return_value=set()):
                    exit_code = mr.main(
                        ["--models", "m", "--out", out_path, "--timeout", "1"]
                    )
            self.assertNotEqual(exit_code, 0)
            self.assertTrue(os.path.exists(out_path))
            with open(out_path, encoding="utf-8") as f:
                payload = json.load(f)
            self.assertTrue(payload["results"][0]["failed"])
            self.assertFalse(os.path.exists(out_path + ".tmp"))

    def test_main_success_exits_zero(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "result.json")
            with patch.object(mr, "generate", return_value={}):
                with patch.object(mr, "resident_model_names", return_value=set()):
                    exit_code = mr.main(["--models", "a,b", "--out", out_path])
            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                payload = json.load(f)
            self.assertEqual(len(payload["results"]), 2)


class ParseModels(unittest.TestCase):
    def test_parse_models_splits_and_trims(self):
        self.assertEqual(mr.parse_models(" a, b ,c"), ["a", "b", "c"])

    def test_parse_models_rejects_empty(self):
        with self.assertRaises(RuntimeError):
            mr.parse_models("  ,  ")


if __name__ == "__main__":
    unittest.main(verbosity=2)
