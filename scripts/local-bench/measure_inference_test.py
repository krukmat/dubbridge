#!/usr/bin/env python3
"""Unit tests for measure_inference.py (mocked Ollama HTTP API only)."""

import json
import os
import sys
import tempfile
import unittest
import urllib.error
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import measure_inference as mi


def _tags(models):
    return {"models": [{"name": name} for name in models]}


def _generate_response(prompt_eval_count, prompt_eval_duration_ns, eval_count, eval_duration_ns):
    return {
        "prompt_eval_count": prompt_eval_count,
        "prompt_eval_duration": prompt_eval_duration_ns,
        "eval_count": eval_count,
        "eval_duration": eval_duration_ns,
    }


class MetricComputation(unittest.TestCase):
    def test_measure_size_computes_tok_s_and_ttft(self):
        response = _generate_response(8150, 1_234_567_890, 64, 987_654_321)
        with patch.object(mi, "http_post_json", return_value=response):
            with patch.object(mi, "sample_peak_memory", return_value=23_938_333_577):
                result = mi.measure_size(
                    "http://localhost:11434", "m", "8k", 8192, timeout=5
                )

        self.assertEqual(result["prompt_eval_count"], 8150)
        self.assertEqual(result["eval_count"], 64)
        self.assertAlmostEqual(result["prefill_tok_s"], 8150 / (1_234_567_890 / 1e9), places=1)
        self.assertAlmostEqual(result["decode_tok_s"], 64 / (987_654_321 / 1e9), places=1)
        self.assertAlmostEqual(result["ttft_ms"], 1_234_567_890 / 1e6, places=1)
        self.assertEqual(result["peak_memory_bytes"], 23_938_333_577)

    def test_measure_size_handles_zero_durations_without_crash(self):
        response = _generate_response(0, 0, 0, 0)
        with patch.object(mi, "http_post_json", return_value=response):
            with patch.object(mi, "sample_peak_memory", return_value=None):
                result = mi.measure_size(
                    "http://localhost:11434", "m", "8k", 8192, timeout=5
                )
        self.assertEqual(result["prefill_tok_s"], 0.0)
        self.assertEqual(result["decode_tok_s"], 0.0)
        self.assertIsNone(result["peak_memory_bytes"])


class OllamaProcessRssBytes(unittest.TestCase):
    def _fake_process(self, name, rss):
        proc = type("P", (), {})()
        proc.info = {"name": name, "memory_info": type("M", (), {"rss": rss})()}
        return proc

    def test_includes_llama_server_child_process_not_just_ollama_named(self):
        # regression: llama-server is the actual inference engine and its RSS
        # dominates the parent "Ollama"/"ollama serve" processes; excluding it
        # undercounted peak memory by orders of magnitude.
        processes = [
            self._fake_process("Ollama", 4_571_136),
            self._fake_process("ollama", 6_733_824),
            self._fake_process("llama-server", 23_426_016_000),
            self._fake_process("unrelated-app", 999_999_999),
        ]
        with patch.object(mi.psutil, "process_iter", return_value=processes):
            total = mi.ollama_process_rss_bytes()
        self.assertEqual(total, 4_571_136 + 6_733_824 + 23_426_016_000)

    def test_returns_none_without_psutil(self):
        with patch.object(mi, "psutil", None):
            self.assertIsNone(mi.ollama_process_rss_bytes())


class PromptSizing(unittest.TestCase):
    def test_build_prompt_within_tolerance_of_target(self):
        for label, target in mi.SIZE_TARGET_TOKENS.items():
            prompt = mi.build_prompt(target)
            approx_tokens = len(prompt) / mi.CHARS_PER_TOKEN
            self.assertGreater(approx_tokens, target * 0.9, label)
            self.assertLess(approx_tokens, target * 1.1, label)

    def test_parse_sizes_accepts_known_labels(self):
        self.assertEqual(mi.parse_sizes("8k,16k,32k"), ["8k", "16k", "32k"])

    def test_parse_sizes_rejects_unknown_label(self):
        with self.assertRaises(RuntimeError):
            mi.parse_sizes("8k,99k")


class SuccessfulRun(unittest.TestCase):
    def test_hp1_and_hp2_multi_size_run_produces_full_schema(self):
        responses = [
            _generate_response(8150, 1_000_000_000, 64, 500_000_000),
            _generate_response(16300, 2_000_000_000, 64, 500_000_000),
            _generate_response(32600, 4_000_000_000, 64, 500_000_000),
        ]

        with patch.object(mi, "installed_model_names", return_value={"qwen3.6:35b-a3b"}):
            with patch.object(mi, "http_post_json", side_effect=responses):
                with patch.object(mi, "sample_peak_memory", return_value=1_000_000):
                    payload = mi.run(
                        "qwen3.6:35b-a3b",
                        "http://localhost:11434",
                        ["8k", "16k", "32k"],
                        timeout=5,
                    )

        self.assertEqual(payload["model"], "qwen3.6:35b-a3b")
        self.assertEqual(payload["host"], "http://localhost:11434")
        self.assertEqual(payload["errors"], [])
        self.assertEqual(len(payload["results"]), 3)
        for entry, label in zip(payload["results"], ["8k", "16k", "32k"]):
            self.assertEqual(entry["size_label"], label)
            self.assertGreater(entry["prefill_tok_s"], 0)
            self.assertGreater(entry["decode_tok_s"], 0)
            self.assertGreater(entry["ttft_ms"], 0)
            self.assertEqual(entry["peak_memory_bytes"], 1_000_000)

    def test_main_writes_json_artifact(self):
        responses = [_generate_response(8150, 1_000_000_000, 64, 500_000_000)]
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "result.json")
            with patch.object(mi, "installed_model_names", return_value={"m"}):
                with patch.object(mi, "http_post_json", side_effect=responses):
                    with patch.object(mi, "sample_peak_memory", return_value=42):
                        exit_code = mi.main(
                            ["--model", "m", "--sizes", "8k", "--out", out_path]
                        )
            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                payload = json.load(f)
            self.assertEqual(payload["results"][0]["size_label"], "8k")
            self.assertFalse(os.path.exists(out_path + ".tmp"))


class ModelNotFound(unittest.TestCase):
    def test_ec1_missing_model_exits_nonzero_without_artifact(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "result.json")
            with patch.object(mi, "installed_model_names", return_value={"other-model"}):
                exit_code = mi.main(
                    ["--model", "missing-model", "--sizes", "8k", "--out", out_path]
                )
            self.assertNotEqual(exit_code, 0)
            self.assertFalse(os.path.exists(out_path))


class UnreachableHost(unittest.TestCase):
    def test_ec2_connection_error_exits_nonzero_without_artifact(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "result.json")
            with patch.object(
                mi,
                "installed_model_names",
                side_effect=urllib.error.URLError("connection refused"),
            ):
                exit_code = mi.main(
                    ["--model", "m", "--sizes", "8k", "--out", out_path, "--timeout", "1"]
                )
            self.assertNotEqual(exit_code, 0)
            self.assertFalse(os.path.exists(out_path))

    def test_ec2_timeout_exits_nonzero_without_artifact(self):
        with tempfile.TemporaryDirectory() as tmp:
            out_path = os.path.join(tmp, "result.json")
            with patch.object(
                mi, "installed_model_names", side_effect=TimeoutError("timed out")
            ):
                exit_code = mi.main(
                    ["--model", "m", "--sizes", "8k", "--out", out_path, "--timeout", "1"]
                )
            self.assertNotEqual(exit_code, 0)
            self.assertFalse(os.path.exists(out_path))


if __name__ == "__main__":
    unittest.main(verbosity=2)
