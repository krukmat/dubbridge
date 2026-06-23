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


def _chunk(text="", done=False, done_reason=None):
    data = {"message": {"content": text}, "done": done}
    if done_reason is not None:
        data["done_reason"] = done_reason
    return json.dumps(data).encode("utf-8")


class StreamChat(unittest.TestCase):
    def test_stream_chat_assembles_content(self):
        response = _FakeResponse([_chunk("a"), _chunk("b", done=True)])
        with patch("urllib.request.urlopen", return_value=response):
            with patch("sys.stderr", new=io.StringIO()):
                self.assertEqual(
                    gemma_local.stream_chat(
                        "http://host/api/chat",
                        {"stream": True},
                        idle_timeout=60,
                        max_wall=900,
                        progress_label="review",
                    ),
                    "ab",
                )

    def test_stream_chat_rejects_length_done_reason(self):
        response = _FakeResponse([_chunk(done=True, done_reason="length")])
        with patch("urllib.request.urlopen", return_value=response):
            with self.assertRaises(RuntimeError) as ctx:
                gemma_local.stream_chat("http://host/api/chat", {}, 60, 900)
        self.assertIn("token limit", str(ctx.exception))

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
