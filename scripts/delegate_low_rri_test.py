#!/usr/bin/env python3
"""Unit tests for delegate-low-rri.py.

Run: python3 scripts/delegate_low_rri_test.py
     python3 -m unittest scripts/delegate_low_rri_test.py
"""

import importlib.util
import io
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import MagicMock, patch

# ---------------------------------------------------------------------------
# Import the script as a module (hyphen in filename requires importlib).
# ---------------------------------------------------------------------------
_SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "delegate-low-rri.py")
_spec = importlib.util.spec_from_file_location("delegate_low_rri", _SCRIPT)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def _valid_payload(**overrides):
    base = {
        "status": "patch",
        "summary": "ok",
        "files": [
            {"path": "scripts/x.py", "action": "create", "contents": "x = 1\n"},
        ],
        "test_commands": ["echo ok"],
        "risk_notes": [],
    }
    base.update(overrides)
    return base


def _ndjson_lines(*chunks, done_last=True):
    """Return a list of NDJSON-encoded bytes representing a streaming response."""
    lines = []
    for text in chunks:
        lines.append(json.dumps({"message": {"content": text}, "done": False}).encode())
    if done_last:
        lines.append(json.dumps({"message": {"content": ""}, "done": True}).encode())
    return lines


# ---------------------------------------------------------------------------
# validate_delegation_payload
# ---------------------------------------------------------------------------
class ValidatePayload(unittest.TestCase):
    def test_valid_passes(self):
        _mod.validate_delegation_payload(_valid_payload())  # no exception

    def test_missing_key_raises(self):
        p = _valid_payload()
        del p["files"]
        with self.assertRaises(RuntimeError) as ctx:
            _mod.validate_delegation_payload(p)
        self.assertIn("files", str(ctx.exception))

    def test_invalid_status_raises(self):
        with self.assertRaises(RuntimeError):
            _mod.validate_delegation_payload(_valid_payload(status="unknown"))

    def test_valid_statuses(self):
        # 'patch' needs files; no_patch/blocked may have an empty list.
        _mod.validate_delegation_payload(_valid_payload(status="patch"))
        _mod.validate_delegation_payload(_valid_payload(status="no_patch", files=[]))
        _mod.validate_delegation_payload(_valid_payload(status="blocked", files=[]))

    def test_test_commands_must_be_list(self):
        with self.assertRaises(RuntimeError):
            _mod.validate_delegation_payload(_valid_payload(test_commands="echo ok"))

    def test_risk_notes_must_be_list(self):
        with self.assertRaises(RuntimeError):
            _mod.validate_delegation_payload(_valid_payload(risk_notes="none"))

    def test_files_must_be_list(self):
        with self.assertRaises(RuntimeError):
            _mod.validate_delegation_payload(_valid_payload(files={"path": "x"}))

    def test_file_entry_missing_field(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.validate_delegation_payload(
                _valid_payload(files=[{"path": "a.py", "action": "create"}]))
        self.assertIn("contents", str(ctx.exception))

    def test_file_invalid_action(self):
        with self.assertRaises(RuntimeError):
            _mod.validate_delegation_payload(_valid_payload(
                files=[{"path": "a.py", "action": "rename", "contents": ""}]))

    def test_patch_status_requires_files(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.validate_delegation_payload(_valid_payload(status="patch", files=[]))
        self.assertIn("empty", str(ctx.exception))

    def test_empty_path_rejected(self):
        with self.assertRaises(RuntimeError):
            _mod.validate_delegation_payload(_valid_payload(
                files=[{"path": "  ", "action": "create", "contents": "x"}]))


# ---------------------------------------------------------------------------
# parse_stream_content
# ---------------------------------------------------------------------------
class ParseStreamContent(unittest.TestCase):
    def test_clean_json(self):
        content = json.dumps(_valid_payload())
        result = _mod.parse_stream_content(content)
        self.assertEqual(result["status"], "patch")

    def test_strips_markdown_fences(self):
        inner = json.dumps(_valid_payload())
        fenced = f"```json\n{inner}\n```"
        result = _mod.parse_stream_content(fenced)
        self.assertEqual(result["status"], "patch")

    def test_strips_plain_fences(self):
        inner = json.dumps(_valid_payload())
        fenced = f"```\n{inner}\n```"
        result = _mod.parse_stream_content(fenced)
        self.assertEqual(result["status"], "patch")

    def test_non_json_raises(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.parse_stream_content("this is not json")
        self.assertIn("non-JSON", str(ctx.exception))

    def test_whitespace_stripped(self):
        content = "\n\n" + json.dumps(_valid_payload()) + "\n"
        result = _mod.parse_stream_content(content)
        self.assertEqual(result["status"], "patch")

    def test_invalid_payload_inside_json_raises(self):
        bad = _valid_payload(status="bogus")
        with self.assertRaises(RuntimeError):
            _mod.parse_stream_content(json.dumps(bad))


# ---------------------------------------------------------------------------
# build_payload
# ---------------------------------------------------------------------------
class BuildPayload(unittest.TestCase):
    def test_stream_true(self):
        p = _mod.build_payload("gemma4:12b-it-q4_K_M", "packet text", 16384)
        self.assertTrue(p["stream"])

    def test_think_false(self):
        p = _mod.build_payload("gemma4:12b-it-q4_K_M", "packet text", 16384)
        self.assertFalse(p["think"])

    def test_num_ctx_set(self):
        p = _mod.build_payload("gemma4:12b-it-q4_K_M", "packet text", 16384)
        self.assertEqual(p["options"]["num_ctx"], 16384)

    def test_custom_num_ctx(self):
        p = _mod.build_payload("model", "text", 8192)
        self.assertEqual(p["options"]["num_ctx"], 8192)

    def test_format_is_schema(self):
        p = _mod.build_payload("model", "text", 16384)
        self.assertEqual(p["format"]["type"], "object")
        self.assertIn("status", p["format"]["properties"])

    def test_packet_in_user_message(self):
        p = _mod.build_payload("model", "MY PACKET", 16384)
        user_msg = next(m for m in p["messages"] if m["role"] == "user")
        self.assertIn("MY PACKET", user_msg["content"])

    def test_system_prompt_contains_schema(self):
        p = _mod.build_payload("model", "text", 16384)
        sys_msg = next(m for m in p["messages"] if m["role"] == "system")
        self.assertIn("schema", sys_msg["content"])


# ---------------------------------------------------------------------------
# stream_chat — idle-timeout and wall-timeout paths
# ---------------------------------------------------------------------------
class _FakeResponse:
    """Minimal file-like object that yields lines then blocks or ends."""

    def __init__(self, lines, block_after=None, delay_per_line=0):
        self._lines = list(lines)
        self._idx = 0
        self._block_after = block_after  # raise socket.timeout after N lines
        self._delay = delay_per_line

    def __enter__(self):
        return self

    def __exit__(self, *a):
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


class StreamChatTimeouts(unittest.TestCase):
    def _call(self, lines, idle=60, wall=900, block_after=None, delay=0):
        resp = _FakeResponse(lines, block_after=block_after, delay_per_line=delay)
        with patch("urllib.request.urlopen", return_value=resp):
            return _mod.stream_chat(
                "http://localhost:11434/api/chat",
                {"model": "test", "stream": True},
                idle_timeout=idle,
                max_wall=wall,
            )

    def test_happy_path_assembles_content(self):
        lines = _ndjson_lines('{"status":"patch"', ',"summary":"ok"',
                              ',"unified_diff":"","test_commands":[],"risk_notes":[]}')
        content = self._call(lines)
        self.assertIn("status", content)

    def test_idle_timeout_on_stall(self):
        lines = _ndjson_lines("partial")
        with self.assertRaises(_mod.DelegationIdleTimeout):
            self._call(lines, idle=1, block_after=1)

    def test_wall_timeout_exceeded(self):
        # Each line sleeps 0.4s; with 3 lines and max_wall=0.5s the cap fires.
        lines = _ndjson_lines("a", "b", "c")
        with self.assertRaises(_mod.DelegationWallTimeout):
            self._call(lines, wall=0.5, delay=0.4)

    def test_empty_response_returns_empty_string(self):
        content = self._call([])
        self.assertEqual(content, "")

    def test_done_flag_stops_iteration(self):
        lines = [
            json.dumps({"message": {"content": "hello"}, "done": True}).encode(),
            json.dumps({"message": {"content": " SHOULD_NOT_APPEAR"}, "done": False}).encode(),
        ]
        content = self._call(lines)
        self.assertIn("hello", content)
        self.assertNotIn("SHOULD_NOT_APPEAR", content)

    def test_malformed_ndjson_line_skipped(self):
        lines = [b"not-json\n",
                 json.dumps({"message": {"content": "ok"}, "done": True}).encode()]
        content = self._call(lines)
        self.assertIn("ok", content)


# ---------------------------------------------------------------------------
# enforce_scope — the caller's guard against out-of-scope writes
# ---------------------------------------------------------------------------
class EnforceScope(unittest.TestCase):
    def _files(self, *paths):
        return [{"path": p, "action": "create", "contents": ""} for p in paths]

    def test_empty_allowed_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.enforce_scope(self._files("scripts/x.py"), [])
        self.assertIn("no --allow-path", str(ctx.exception))

    def test_in_scope_prefix_passes(self):
        _mod.enforce_scope(self._files("scripts/x.py"), ["scripts/"])

    def test_in_scope_exact_passes(self):
        _mod.enforce_scope(self._files("scripts/x.py"), ["scripts/x.py"])

    def test_in_scope_glob_passes(self):
        _mod.enforce_scope(self._files("scripts/x.py"), ["scripts/*.py"])

    def test_out_of_scope_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.enforce_scope(self._files("crates/auth/lib.rs"), ["scripts/"])
        self.assertIn("out-of-scope", str(ctx.exception))

    def test_parent_escape_rejected(self):
        with self.assertRaises(RuntimeError) as ctx:
            _mod.enforce_scope(self._files("../etc/passwd"), ["scripts/"])
        self.assertIn("escapes", str(ctx.exception))

    def test_absolute_path_rejected(self):
        with self.assertRaises(RuntimeError):
            _mod.enforce_scope(self._files("/etc/passwd"), ["scripts/"])

    def test_prefix_does_not_match_sibling(self):
        # "scripts" prefix must not allow "scripts_evil/x.py".
        with self.assertRaises(RuntimeError):
            _mod.enforce_scope(self._files("scripts_evil/x.py"), ["scripts/"])


# ---------------------------------------------------------------------------
# build_diff + apply_diff — deterministic, git-owned diff framing
# ---------------------------------------------------------------------------
class BuildAndApplyDiff(unittest.TestCase):
    def _git_repo(self):
        d = tempfile.mkdtemp()
        subprocess.run(["git", "init", "-q"], cwd=d, check=True)
        subprocess.run(["git", "config", "user.email", "t@t"], cwd=d, check=True)
        subprocess.run(["git", "config", "user.name", "t"], cwd=d, check=True)
        return d

    def test_create_then_apply_multifile(self):
        # The exact failure case from the live test: TWO new files in one delegation.
        d = self._git_repo()
        os.makedirs(os.path.join(d, "scripts"))
        files = [
            {"path": "scripts/math_utils.py", "action": "create",
             "contents": "def add(a, b):\n    return a + b\n"},
            {"path": "scripts/math_utils_test.py", "action": "create",
             "contents": "from math_utils import add\nassert add(2, 3) == 5\n"},
        ]
        diff = _mod.build_diff(files, d)
        self.assertIn("scripts/math_utils.py", diff)
        self.assertIn("scripts/math_utils_test.py", diff)
        # The whole point: git apply --check must pass on the multi-file diff.
        outcome = _mod.apply_diff(diff, d)
        self.assertEqual(outcome, "applied")
        self.assertTrue(os.path.exists(os.path.join(d, "scripts/math_utils.py")))
        self.assertTrue(os.path.exists(os.path.join(d, "scripts/math_utils_test.py")))

    def test_modify_existing_file(self):
        d = self._git_repo()
        target = os.path.join(d, "a.txt")
        with open(target, "w") as f:
            f.write("line one\nline two\n")
        subprocess.run(["git", "add", "-A"], cwd=d, check=True)
        subprocess.run(["git", "commit", "-qm", "init"], cwd=d, check=True)
        files = [{"path": "a.txt", "action": "modify",
                  "contents": "line one\nline two CHANGED\n"}]
        diff = _mod.build_diff(files, d)
        outcome = _mod.apply_diff(diff, d)
        self.assertEqual(outcome, "applied")
        with open(target) as f:
            self.assertIn("CHANGED", f.read())

    def test_empty_diff_when_no_change(self):
        d = self._git_repo()
        target = os.path.join(d, "a.txt")
        with open(target, "w") as f:
            f.write("same\n")
        files = [{"path": "a.txt", "action": "modify", "contents": "same\n"}]
        diff = _mod.build_diff(files, d)
        self.assertEqual(diff.strip(), "")
        self.assertIn("nothing to apply", _mod.apply_diff(diff, d))


# ---------------------------------------------------------------------------
# write_result — atomic write
# ---------------------------------------------------------------------------
class WriteResult(unittest.TestCase):
    def test_writes_valid_json(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "result.json")
            _mod.write_result(_valid_payload(), out)
            with open(out) as f:
                data = json.load(f)
            self.assertEqual(data["status"], "patch")

    def test_atomic_replace(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "result.json")
            # Write once, then overwrite — no partial file visible.
            _mod.write_result(_valid_payload(summary="first"), out)
            _mod.write_result(_valid_payload(summary="second"), out)
            with open(out) as f:
                data = json.load(f)
            self.assertEqual(data["summary"], "second")

    def test_no_tmp_file_left_on_success(self):
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "result.json")
            _mod.write_result(_valid_payload(), out)
            self.assertFalse(os.path.exists(out + ".tmp"))


# ---------------------------------------------------------------------------
# CLI: --dry-run, --out, exit codes
# ---------------------------------------------------------------------------
class CliBehavior(unittest.TestCase):
    def run_cli(self, *args, stdin=None):
        return subprocess.run(
            [sys.executable, _SCRIPT, *args],
            capture_output=True, text=True,
            input=stdin,
        )

    def test_dry_run_prints_payload_no_network(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
            f.write("# test packet\n")
            fname = f.name
        try:
            r = self.run_cli(fname, "--dry-run")
            self.assertEqual(r.returncode, 0, r.stderr)
            data = json.loads(r.stdout)
            self.assertTrue(data["stream"])
            self.assertIn("num_ctx", data["options"])
        finally:
            os.unlink(fname)

    def test_dry_run_stream_true(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
            f.write("# test packet\n")
            fname = f.name
        try:
            r = self.run_cli(fname, "--dry-run")
            data = json.loads(r.stdout)
            self.assertTrue(data["stream"])
        finally:
            os.unlink(fname)

    def test_dry_run_num_ctx_default(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
            f.write("# p\n")
            fname = f.name
        try:
            r = self.run_cli(fname, "--dry-run")
            data = json.loads(r.stdout)
            self.assertEqual(data["options"]["num_ctx"], _mod.DEFAULT_NUM_CTX)
        finally:
            os.unlink(fname)

    def test_dry_run_custom_num_ctx(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
            f.write("# p\n")
            fname = f.name
        try:
            r = self.run_cli(fname, "--dry-run", "--num-ctx", "8192")
            data = json.loads(r.stdout)
            self.assertEqual(data["options"]["num_ctx"], 8192)
        finally:
            os.unlink(fname)

    def test_empty_packet_exits_1(self):
        r = self.run_cli("-", stdin="   \n  ")
        self.assertEqual(r.returncode, 1)
        self.assertIn("empty", r.stderr)

    def test_missing_packet_file_exits_nonzero(self):
        r = self.run_cli("/tmp/does_not_exist_dubbridge.md")
        self.assertNotEqual(r.returncode, 0)

    def test_help_exits_0(self):
        r = self.run_cli("--help")
        self.assertEqual(r.returncode, 0)


# ---------------------------------------------------------------------------
# DelegationIdleTimeout / DelegationWallTimeout exit codes
# ---------------------------------------------------------------------------
class TimeoutExitCodes(unittest.TestCase):
    def test_idle_timeout_exit_code_124(self):
        exc = _mod.DelegationIdleTimeout(60)
        self.assertEqual(exc.exit_code, 124)
        self.assertIn("60", str(exc))

    def test_wall_timeout_exit_code_124(self):
        exc = _mod.DelegationWallTimeout(900)
        self.assertEqual(exc.exit_code, 124)
        self.assertIn("900", str(exc))


if __name__ == "__main__":
    unittest.main(verbosity=2)
