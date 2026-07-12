#!/usr/bin/env python3
"""Unit tests for soak_contention.py (mocked Ollama HTTP API, time fully mocked)."""

import json
import os
import sys
import tempfile
import unittest
import urllib.error
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import soak_contention as sc


def _generate_response(eval_count, eval_duration_ns):
    return {"eval_count": eval_count, "eval_duration": eval_duration_ns}


class TakeSample(unittest.TestCase):
    def test_successful_sample_has_all_fields(self):
        with patch.object(sc, "http_post_json", return_value=_generate_response(16, 400_000_000)):
            with patch.object(sc.measure_inference, "sample_peak_memory", return_value=1_000_000):
                with patch.object(sc, "swap_used_bytes", return_value=0):
                    sample = sc.take_sample("http://localhost:11434", "m", 0, timeout=5)

        self.assertTrue(sample["sample_ok"])
        self.assertEqual(sample["elapsed_s"], 0)
        self.assertAlmostEqual(sample["decode_tok_s"], 16 / (400_000_000 / 1e9), places=1)
        self.assertEqual(sample["peak_memory_bytes"], 1_000_000)
        self.assertEqual(sample["swap_used_bytes"], 0)
        self.assertNotIn("error", sample)

    def test_ec1_generate_failure_is_recorded_not_raised(self):
        with patch.object(
            sc, "http_post_json", side_effect=urllib.error.URLError("timed out")
        ):
            with patch.object(sc, "swap_used_bytes", return_value=0):
                sample = sc.take_sample("http://localhost:11434", "m", 30, timeout=5)

        self.assertFalse(sample["sample_ok"])
        self.assertIsNone(sample["decode_tok_s"])
        self.assertIsNone(sample["peak_memory_bytes"])
        self.assertIn("error", sample)
        self.assertIn("timed out", sample["error"])


class Summarize(unittest.TestCase):
    def test_aggregates_only_ok_samples(self):
        samples = [
            {"sample_ok": True, "decode_tok_s": 40.0, "peak_memory_bytes": 100, "swap_used_bytes": 0},
            {"sample_ok": True, "decode_tok_s": 38.0, "peak_memory_bytes": 200, "swap_used_bytes": 512},
            {"sample_ok": False, "decode_tok_s": None, "peak_memory_bytes": None, "swap_used_bytes": None, "error": "boom"},
        ]
        summary = sc.summarize(samples)

        self.assertEqual(summary["min_decode_tok_s"], 38.0)
        self.assertEqual(summary["median_decode_tok_s"], 39.0)
        self.assertEqual(summary["peak_swap_used_bytes"], 512)
        self.assertEqual(summary["peak_memory_bytes"], 200)
        self.assertEqual(summary["failed_sample_count"], 1)
        self.assertEqual(summary["total_samples"], 3)

    def test_summarize_handles_all_samples_missing_optional_fields(self):
        samples = [
            {"sample_ok": True, "decode_tok_s": 10.0, "peak_memory_bytes": None, "swap_used_bytes": None},
        ]
        summary = sc.summarize(samples)
        self.assertIsNone(summary["peak_memory_bytes"])
        self.assertIsNone(summary["peak_swap_used_bytes"])


class HP1FullRun(unittest.TestCase):
    def test_hp1_run_produces_populated_samples_and_summary(self):
        responses = [_generate_response(16, 400_000_000) for _ in range(4)]
        # start, 3 ticks with a duration check each, plus loop exit checks;
        # generous fixed list avoids brittle call-count coupling to the loop shape
        monotonic_values = [0, 0, 30, 30, 60, 60, 90, 90, 120, 120, 120, 120, 120, 120]

        with patch.object(sc, "http_post_json", side_effect=responses):
            with patch.object(sc.measure_inference, "sample_peak_memory", return_value=42):
                with patch.object(sc, "swap_used_bytes", return_value=0):
                    with patch.object(sc.time, "monotonic", side_effect=monotonic_values):
                        with patch.object(sc.time, "sleep", return_value=None):
                            payload = sc.run(
                                "m", "http://localhost:11434",
                                duration_seconds=90, interval_seconds=30, timeout=5,
                            )

        self.assertGreater(len(payload["samples"]), 0)
        for sample in payload["samples"]:
            self.assertTrue(sample["sample_ok"])
        self.assertIsNotNone(payload["summary"]["min_decode_tok_s"])
        self.assertIsNotNone(payload["summary"]["median_decode_tok_s"])
        self.assertEqual(payload["summary"]["total_samples"], len(payload["samples"]))
        self.assertIsNone(payload["throttle_detected"])

    def test_main_writes_json_artifact_with_populated_summary(self):
        responses = [_generate_response(16, 400_000_000) for _ in range(2)]
        monotonic_values = [0, 0, 30, 30, 30, 30]

        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "soak.json")
            with patch.object(sc, "http_post_json", side_effect=responses):
                with patch.object(sc.measure_inference, "sample_peak_memory", return_value=42):
                    with patch.object(sc, "swap_used_bytes", return_value=0):
                        with patch.object(sc.time, "monotonic", side_effect=monotonic_values):
                            with patch.object(sc.time, "sleep", return_value=None):
                                exit_code = sc.main([
                                    "--model", "m",
                                    "--duration-seconds", "30",
                                    "--interval-seconds", "30",
                                    "--out", out_path,
                                ])

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                payload = json.load(f)
            self.assertGreater(len(payload["samples"]), 0)
            self.assertIsNotNone(payload["summary"]["median_decode_tok_s"])
            self.assertFalse(os.path.exists(out_path + ".tmp"))


class EC1MidSoakFailure(unittest.TestCase):
    def test_ec1_failure_mid_run_does_not_crash_and_is_recorded(self):
        def flaky_post(url, payload, timeout):
            flaky_post.calls += 1
            if flaky_post.calls == 2:
                raise urllib.error.URLError("connection reset")
            return _generate_response(16, 400_000_000)

        flaky_post.calls = 0
        monotonic_values = [0, 0, 30, 30, 60, 60, 60, 60]

        with patch.object(sc, "http_post_json", side_effect=flaky_post):
            with patch.object(sc.measure_inference, "sample_peak_memory", return_value=42):
                with patch.object(sc, "swap_used_bytes", return_value=0):
                    with patch.object(sc.time, "monotonic", side_effect=monotonic_values):
                        with patch.object(sc.time, "sleep", return_value=None):
                            payload = sc.run(
                                "m", "http://localhost:11434",
                                duration_seconds=60, interval_seconds=30, timeout=5,
                            )

        self.assertEqual(payload["summary"]["failed_sample_count"], 1)
        failed = [s for s in payload["samples"] if not s["sample_ok"]]
        self.assertEqual(len(failed), 1)
        self.assertIn("error", failed[0])
        # the failed sample's None decode value must not corrupt min/median math
        self.assertIsNotNone(payload["summary"]["min_decode_tok_s"])
        self.assertIsNotNone(payload["summary"]["median_decode_tok_s"])

    def test_main_exits_nonzero_when_any_sample_failed_but_still_writes_artifact(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "soak.json")
            with patch.object(
                sc, "http_post_json", side_effect=urllib.error.URLError("boom")
            ):
                with patch.object(sc, "swap_used_bytes", return_value=0):
                    with patch.object(sc.time, "monotonic", side_effect=[0, 0, 30, 30, 30, 30]):
                        with patch.object(sc.time, "sleep", return_value=None):
                            exit_code = sc.main([
                                "--model", "m",
                                "--duration-seconds", "30",
                                "--interval-seconds", "30",
                                "--out", out_path,
                            ])

            self.assertNotEqual(exit_code, 0)
            self.assertTrue(os.path.exists(out_path))
            with open(out_path, encoding="utf-8") as f:
                payload = json.load(f)
            self.assertGreaterEqual(payload["summary"]["failed_sample_count"], 1)


class ParseArgsDefaults(unittest.TestCase):
    def test_defaults_match_contract(self):
        args = sc.parse_args(["--model", "m", "--out", "out.json"])
        self.assertEqual(args.duration_seconds, 3600)
        self.assertEqual(args.interval_seconds, 30)
        self.assertEqual(args.timeout, 30)


if __name__ == "__main__":
    unittest.main(verbosity=2)
