#!/usr/bin/env python3
"""Focused tests for the simple (Serena-free) local-runner file tools."""

import os
import tempfile
import unittest

from runner_file_tools import ALLOWED_TOOL_NAMES, RunnerFileTools


class _AllowAllBoundary:
    def check_write(self, path):
        return None

    def check_command(self, argv):
        return None

    def env_for_subprocess(self):
        return None


class _Malformed(ValueError):
    pass


class _Boundary(RuntimeError):
    pass


class _Call:
    def __init__(self, name, arguments):
        self.name = name
        self.arguments = arguments


class RunnerFileToolsTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.worktree = self._tmp.name
        self.tools = RunnerFileTools(
            self.worktree, _AllowAllBoundary(), _Malformed, _Boundary
        )

    def tearDown(self):
        self._tmp.cleanup()

    def _write(self, rel, content):
        with open(os.path.join(self.worktree, rel), "w", encoding="utf-8") as f:
            f.write(content)

    def _read(self, rel):
        with open(os.path.join(self.worktree, rel), encoding="utf-8") as f:
            return f.read()

    def test_contract_is_exactly_five_tools_and_has_no_symbol_tools(self):
        self.assertEqual(
            set(ALLOWED_TOOL_NAMES),
            {"read_file", "write_file", "apply_patch", "run_command", "finish"},
        )

    def test_handle_returns_none_for_non_file_tools(self):
        # run_command / finish are not file tools; the runner routes those.
        self.assertIsNone(self.tools.handle(_Call("run_command", {"argv": ["ls"]})))
        self.assertIsNone(self.tools.handle(_Call("finish", {})))

    def test_read_file_returns_whole_file_with_no_line_cap(self):
        big = "".join(f"line {i}\n" for i in range(2000))  # far past the old 400 cap
        self._write("big.rs", big)
        result = self.tools.handle(_Call("read_file", {"path": "big.rs"}))
        self.assertTrue(result["ok"])
        self.assertEqual(result["content"], big)
        self.assertEqual(result["line_count"], 2000)

    def test_read_file_missing_is_malformed_not_crash(self):
        with self.assertRaises(_Malformed):
            self.tools.handle(_Call("read_file", {"path": "nope.rs"}))

    def test_write_file_creates_new_file(self):
        result = self.tools.handle(
            _Call("write_file", {"path": "new.rs", "content": "fn a() {}\n"})
        )
        self.assertTrue(result["ok"])
        self.assertTrue(result["created"])
        self.assertEqual(self._read("new.rs"), "fn a() {}\n")

    def test_write_file_overwrites_existing_file(self):
        self._write("existing.rs", "old\n")
        result = self.tools.handle(
            _Call("write_file", {"path": "existing.rs", "content": "new\n"})
        )
        self.assertTrue(result["ok"])
        self.assertFalse(result["created"])
        self.assertEqual(self._read("existing.rs"), "new\n")

    def test_write_file_has_no_size_budget(self):
        big = "".join(f"// line {i}\n" for i in range(500))  # past the old 120 cap
        result = self.tools.handle(
            _Call("write_file", {"path": "big_new.rs", "content": big})
        )
        self.assertTrue(result["ok"])
        self.assertEqual(result["line_count"], 500)

    def test_apply_patch_replaces_unique_anchor(self):
        self._write("m.rs", "fn a() { old(); }\n")
        result = self.tools.handle(
            _Call(
                "apply_patch",
                {"path": "m.rs", "anchor": "old()", "replacement": "new()"},
            )
        )
        self.assertTrue(result["ok"])
        self.assertEqual(result["anchor_matches"], 1)
        self.assertEqual(self._read("m.rs"), "fn a() { new(); }\n")

    def test_apply_patch_rejects_ambiguous_anchor(self):
        self._write("m.rs", "x = 1\nx = 1\n")
        with self.assertRaises(_Malformed):
            self.tools.handle(
                _Call(
                    "apply_patch",
                    {"path": "m.rs", "anchor": "x = 1", "replacement": "x = 2"},
                )
            )
        # unchanged on rejection
        self.assertEqual(self._read("m.rs"), "x = 1\nx = 1\n")

    def test_apply_patch_rejects_absent_anchor(self):
        self._write("m.rs", "nothing here\n")
        with self.assertRaises(_Malformed):
            self.tools.handle(
                _Call(
                    "apply_patch",
                    {"path": "m.rs", "anchor": "missing", "replacement": "x"},
                )
            )

    def test_apply_patch_has_no_size_budget(self):
        self._write("m.rs", "ANCHOR\n")
        big = "".join(f"line {i}\n" for i in range(300))  # past old 80-line patch cap
        result = self.tools.handle(
            _Call(
                "apply_patch",
                {"path": "m.rs", "anchor": "ANCHOR", "replacement": big},
            )
        )
        self.assertTrue(result["ok"])
        self.assertEqual(result["line_count"], 300)

    def test_write_through_symlink_is_rejected_at_open(self):
        outside = os.path.join(self._tmp.name, "..", "outside.txt")
        link = os.path.join(self.worktree, "link")
        os.symlink(os.path.abspath(outside), link)
        with self.assertRaises(_Boundary):
            self.tools.handle(
                _Call("write_file", {"path": "link", "content": "PWNED"})
            )

    def test_missing_required_argument_is_malformed(self):
        with self.assertRaises(_Malformed):
            self.tools.handle(_Call("read_file", {}))
        with self.assertRaises(_Malformed):
            self.tools.handle(_Call("apply_patch", {"path": "m.rs", "anchor": "a"}))


if __name__ == "__main__":
    unittest.main()
