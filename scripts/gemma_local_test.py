#!/usr/bin/env python3
"""Unit tests for shared local Gemma/Ollama helpers."""

import io
import json
import os
import socket
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gemma_local


class SharedConfig(unittest.TestCase):
    def test_defaults_are_role_neutral(self):
        self.assertEqual(gemma_local.DEFAULT_HOST, "http://localhost:11434")
        self.assertEqual(gemma_local.DEFAULT_MODEL, "gemma4:26b-a4b-it-qat")
        self.assertEqual(gemma_local.DEFAULT_FALLBACK_MODEL, "gemma4:26b-a4b-it-qat")
        self.assertEqual(gemma_local.DEFAULT_TEMPERATURE, 0.1)
        self.assertFalse(gemma_local.DEFAULT_THINK)

    def test_bool_from_env_defaults_false(self):
        with patch.dict(os.environ, {}, clear=True):
            self.assertFalse(gemma_local.bool_from_env("MISSING"))

    def test_bool_from_env_truthy(self):
        with patch.dict(os.environ, {"X": "yes"}, clear=True):
            self.assertTrue(gemma_local.bool_from_env("X"))

    def test_bool_from_env_falsey(self):
        with patch.dict(os.environ, {"X": "0"}, clear=True):
            self.assertFalse(gemma_local.bool_from_env("X", default=True))


class PacketAndResults(unittest.TestCase):
    def test_read_packet_from_file(self):
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as f:
            f.write("packet\n")
            path = f.name
        try:
            self.assertEqual(gemma_local.read_packet(path), "packet\n")
        finally:
            os.unlink(path)

    def test_write_result_is_json(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "result.json")
            gemma_local.write_result({"status": "pass"}, out)
            with open(out, encoding="utf-8") as f:
                self.assertEqual(json.load(f), {"status": "pass"})
            self.assertFalse(os.path.exists(out + ".tmp"))


class Payload(unittest.TestCase):
    def test_build_chat_payload_sets_generation_options(self):
        payload = gemma_local.build_chat_payload(
            model="m",
            system_prompt="system",
            packet="packet",
            num_ctx=8192,
            num_predict=2048,
            temperature=0.25,
            think=True,
        )

        self.assertEqual(payload["model"], "m")
        self.assertTrue(payload["stream"])
        self.assertTrue(payload["think"])
        self.assertEqual(payload["options"]["num_ctx"], 8192)
        self.assertEqual(payload["options"]["num_predict"], 2048)
        self.assertEqual(payload["options"]["temperature"], 0.25)
        self.assertEqual(payload["messages"][0]["content"], "system")
        self.assertEqual(payload["messages"][1]["content"], "packet")

    def test_estimate_payload_tokens_is_deterministic(self):
        payload = gemma_local.build_chat_payload(
            model="m",
            system_prompt="system",
            packet="packet",
            num_ctx=8192,
            num_predict=2048,
            temperature=0.25,
            think=True,
        )
        self.assertEqual(
            gemma_local.estimate_payload_tokens(payload),
            gemma_local.estimate_payload_tokens(payload),
        )

    def test_sum_measured_tokens_requires_complete_data(self):
        self.assertEqual(gemma_local.sum_measured_tokens([2, 3]), 5)
        self.assertIsNone(gemma_local.sum_measured_tokens([2, None]))


class ModelAvailability(unittest.TestCase):
    def test_ensure_model_available_accepts_installed_default(self):
        tags = {"models": [{"name": "gemma4:26b-a4b-it-qat"}]}
        with patch.object(gemma_local, "get_json", return_value=tags):
            gemma_local.ensure_model_available(
                "http://localhost:11434",
                gemma_local.DEFAULT_MODEL,
                1,
            )

    def test_ensure_model_available_reports_missing_default(self):
        tags = {"models": [{"name": "qwen3.6:35b-a3b"}]}
        with patch.object(gemma_local, "get_json", return_value=tags):
            with self.assertRaises(RuntimeError) as ctx:
                gemma_local.ensure_model_available(
                    "http://localhost:11434",
                    gemma_local.DEFAULT_MODEL,
                    1,
                )
        self.assertIn("gemma4:26b-a4b-it-qat", str(ctx.exception))
        self.assertIn("qwen3.6:35b-a3b", str(ctx.exception))

    def test_resolve_model_with_fallback_uses_primary_when_installed(self):
        tags = {"models": [{"name": "gemma4:26b-a4b-it-qat"}]}
        with patch.object(gemma_local, "get_json", return_value=tags):
            model = gemma_local.resolve_model_with_fallback(
                "http://localhost:11434",
                gemma_local.DEFAULT_MODEL,
                1,
                gemma_local.DEFAULT_FALLBACK_MODEL,
            )
        self.assertEqual(model, "gemma4:26b-a4b-it-qat")

    def test_resolve_model_with_fallback_uses_current_model_when_primary_absent(self):
        tags = {"models": [{"name": "gemma4:26b-a4b-it-qat"}]}
        with patch.object(gemma_local, "get_json", return_value=tags):
            model = gemma_local.resolve_model_with_fallback(
                "http://localhost:11434",
                "gemma4:some-missing-tag",
                1,
                gemma_local.DEFAULT_FALLBACK_MODEL,
            )
        self.assertEqual(model, "gemma4:26b-a4b-it-qat")

    def test_default_fallback_model_disabled_by_explicit_override(self):
        with patch.dict(os.environ, {"DUBBRIDGE_LOW_RRI_MODEL": "custom"}, clear=True):
            fallback = gemma_local.default_fallback_model_for("DUBBRIDGE_LOW_RRI_MODEL")
        self.assertIsNone(fallback)


class TaggedHelpers(unittest.TestCase):
    def test_normalize_tagged_content_strips_crlf(self):
        self.assertEqual(
            gemma_local.normalize_tagged_content("x\r\ny\r\n", "review"),
            "x\ny",
        )

    def test_normalize_tagged_content_rejects_empty(self):
        with self.assertRaises(RuntimeError) as ctx:
            gemma_local.normalize_tagged_content(" \n", "review")
        self.assertIn("empty", str(ctx.exception))

    def test_next_nonempty_line(self):
        line, idx = gemma_local.next_nonempty_line(
            ["", "PATH: x"], 0, "PATH", "invalid review response"
        )
        self.assertEqual(line, "PATH: x")
        self.assertEqual(idx, 2)

    def test_parse_header_value(self):
        self.assertEqual(
            gemma_local.parse_header_value(
                "STATUS: PASS", "STATUS", "invalid review response"
            ),
            "PASS",
        )

    def test_parse_header_value_rejects_wrong_label(self):
        with self.assertRaises(RuntimeError) as ctx:
            gemma_local.parse_header_value(
                "PATH: x", "STATUS", "invalid review response"
            )
        self.assertIn("STATUS", str(ctx.exception))


class AuditLog(unittest.TestCase):
    def _run(self, record, *, now=None, tmp=None):
        import datetime as dt
        ts = now or dt.datetime(2026, 6, 24, 12, 0, 0)
        log_dir = os.path.join(tmp, "logs", "gemma-audit")
        orig_cwd = os.getcwd()
        os.chdir(tmp)
        try:
            gemma_local.append_audit_log(record, now=ts)
        finally:
            os.chdir(orig_cwd)
        log_path = os.path.join(log_dir, "2026-06.jsonl")
        return log_path

    def test_hp1_automatic_fields_only(self):
        import datetime as dt
        with tempfile.TemporaryDirectory() as tmp:
            record = {
                "ts": "2026-06-24T12:00:00Z", "role": "developer",
                "outcome": "PATCH", "done_reason": "stop", "mode": "full-file",
                "elapsed_s": 1.2, "escalated": False,
                "system_prompt": "sys", "user_prompt": "user",
                "task_id": None, "rri": None, "band": None,
                "attempt": None, "disposition": None,
            }
            log_path = self._run(record, tmp=tmp)
            with open(log_path, encoding="utf-8") as f:
                lines = f.readlines()
            self.assertEqual(len(lines), 1)
            parsed = json.loads(lines[0])
            self.assertEqual(parsed["role"], "developer")
            self.assertIsNone(parsed["task_id"])
            self.assertEqual(parsed["system_prompt"], "sys")

    def test_hp2_orchestrator_fields_populated(self):
        import datetime as dt
        with tempfile.TemporaryDirectory() as tmp:
            record = {
                "ts": "2026-06-24T12:00:00Z", "role": "reviewer",
                "outcome": "FINDINGS", "done_reason": "stop", "mode": "n/a",
                "elapsed_s": 3.0, "escalated": False,
                "system_prompt": "sys", "user_prompt": "user",
                "task_id": "T5", "rri": 40, "band": "Moderate",
                "attempt": 1, "disposition": None,
            }
            log_path = self._run(record, tmp=tmp)
            with open(log_path, encoding="utf-8") as f:
                parsed = json.loads(f.read())
            self.assertEqual(parsed["task_id"], "T5")
            self.assertEqual(parsed["band"], "Moderate")

    def test_ec1_directory_auto_created(self):
        import datetime as dt
        with tempfile.TemporaryDirectory() as tmp:
            record = {"role": "developer", "system_prompt": "s", "user_prompt": "u"}
            log_path = self._run(record, tmp=tmp)
            self.assertTrue(os.path.exists(log_path))

    def test_ec2_secret_redacted_in_prompts(self):
        import datetime as dt
        with tempfile.TemporaryDirectory() as tmp:
            record = {
                "role": "developer",
                "system_prompt": "normal text",
                "user_prompt": "API_KEY=supersecret do something",
            }
            log_path = self._run(record, tmp=tmp)
            with open(log_path, encoding="utf-8") as f:
                line = f.read()
            self.assertNotIn("supersecret", line)
            self.assertIn("REDACTED", line)

    def test_ec2b_secret_redacted_inside_nested_list_field(self):
        # D14 finding (T6c): the original redaction only inspected top-level
        # string values. local-implementer audit records carry "commands" as
        # a list of argv lists — model-controlled content that could smuggle
        # a credential-shaped string past a shallow redaction pass.
        with tempfile.TemporaryDirectory() as tmp:
            record = {
                "role": "local-implementer",
                "commands": [["curl", "-H", "token=supersecrettoken", "https://example.com"]],
            }
            log_path = self._run(record, tmp=tmp)
            with open(log_path, encoding="utf-8") as f:
                line = f.read()
            self.assertNotIn("supersecrettoken", line)
            self.assertIn("REDACTED", line)

    def test_ec2c_secret_redacted_inside_nested_dict_field(self):
        with tempfile.TemporaryDirectory() as tmp:
            record = {
                "role": "local-implementer",
                "nested": {"inner": "password=hunter2 leaked here"},
            }
            log_path = self._run(record, tmp=tmp)
            with open(log_path, encoding="utf-8") as f:
                line = f.read()
            self.assertNotIn("hunter2", line)
            self.assertIn("REDACTED", line)

    def test_ec3_two_appends_produce_two_lines(self):
        import datetime as dt
        with tempfile.TemporaryDirectory() as tmp:
            ts = dt.datetime(2026, 6, 24, 12, 0, 0)
            record = {"role": "developer", "system_prompt": "s", "user_prompt": "u"}
            orig_cwd = os.getcwd()
            os.chdir(tmp)
            try:
                gemma_local.append_audit_log(record, now=ts)
                gemma_local.append_audit_log(record, now=ts)
            finally:
                os.chdir(orig_cwd)
            log_path = os.path.join(tmp, "logs", "gemma-audit", "2026-06.jsonl")
            with open(log_path, encoding="utf-8") as f:
                lines = [l for l in f.readlines() if l.strip()]
            self.assertEqual(len(lines), 2)

    def test_month_rollover(self):
        import datetime as dt
        with tempfile.TemporaryDirectory() as tmp:
            record = {"role": "reviewer", "system_prompt": "s", "user_prompt": "u"}
            orig_cwd = os.getcwd()
            os.chdir(tmp)
            try:
                gemma_local.append_audit_log(record, now=dt.datetime(2026, 5, 31))
                gemma_local.append_audit_log(record, now=dt.datetime(2026, 6, 1))
            finally:
                os.chdir(orig_cwd)
            log_dir = os.path.join(tmp, "logs", "gemma-audit")
            files = sorted(os.listdir(log_dir))
            self.assertIn("2026-05.jsonl", files)
            self.assertIn("2026-06.jsonl", files)


class _FakeResponse:
    def __init__(self, lines, block_after=None, delay_per_line=0):
        self._lines = list(lines)
        self._idx = 0
        self._block_after = block_after
        self._delay = delay_per_line

    def __enter__(self):
        return self

    def __exit__(self, *args):
        pass

    def readline(self):
        if self._block_after is not None and self._idx >= self._block_after:
            raise socket.timeout("simulated stall")
        if self._idx >= len(self._lines):
            return b""
        if self._delay:
            time.sleep(self._delay)
        line = self._lines[self._idx]
        self._idx += 1
        return line + b"\n"


def _chunk(text="", done=False, done_reason=None, eval_count=None, prompt_eval_count=None):
    data = {"message": {"content": text}, "done": done}
    if done_reason is not None:
        data["done_reason"] = done_reason
    if eval_count is not None:
        data["eval_count"] = eval_count
    if prompt_eval_count is not None:
        data["prompt_eval_count"] = prompt_eval_count
    return json.dumps(data).encode("utf-8")


class StreamChat(unittest.TestCase):
    def test_stream_chat_assembles_content(self):
        response = _FakeResponse([
            _chunk("a"),
            _chunk("b", done=True, eval_count=7, prompt_eval_count=11),
        ])
        with patch("urllib.request.urlopen", return_value=response):
            with patch("sys.stderr", new=io.StringIO()):
                result = gemma_local.stream_chat(
                    "http://host/api/chat",
                    {"stream": True},
                    idle_timeout=60,
                    max_wall=900,
                    progress_label="review",
                )
        self.assertEqual(result.content, "ab")
        self.assertEqual(result.usage.response_tokens, 7)
        self.assertEqual(result.usage.prompt_tokens, 11)

    def test_stream_chat_rejects_length_done_reason(self):
        response = _FakeResponse([_chunk(done=True, done_reason="length")])
        with patch("urllib.request.urlopen", return_value=response):
            with self.assertRaises(RuntimeError) as ctx:
                gemma_local.stream_chat("http://host/api/chat", {}, 60, 900)
        self.assertIn("token limit", str(ctx.exception))

    def test_stream_result_helpers_accept_legacy_string(self):
        self.assertEqual(gemma_local.stream_result_content("hello"), "hello")
        usage = gemma_local.stream_result_usage("hello")
        self.assertIsNone(usage.response_tokens)

    def test_stream_chat_idle_timeout(self):
        response = _FakeResponse([], block_after=0)
        with patch("urllib.request.urlopen", return_value=response):
            with self.assertRaises(gemma_local.GemmaIdleTimeout):
                gemma_local.stream_chat("http://host/api/chat", {}, 60, 900)

    def test_stream_chat_wall_timeout(self):
        response = _FakeResponse([_chunk("x")] * 3, delay_per_line=0.02)
        with patch("urllib.request.urlopen", return_value=response):
            with self.assertRaises(gemma_local.GemmaWallTimeout):
                gemma_local.stream_chat("http://host/api/chat", {}, 60, 0.01)


if __name__ == "__main__":
    unittest.main(verbosity=2)
