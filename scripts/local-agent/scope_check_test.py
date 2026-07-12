#!/usr/bin/env python3
"""Unit tests for the pure local-agent diff-scope utility."""

import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scope_check


def _git(repo, *args):
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )


class ScopeCheck(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = self.tmp.name
        _git(self.repo, "init", "-q")
        _git(self.repo, "config", "user.email", "scope-check@example.test")
        _git(self.repo, "config", "user.name", "Scope Check")
        os.makedirs(os.path.join(self.repo, "scripts", "local-agent"))
        with open(os.path.join(self.repo, "scripts", "local-agent", "runner.py"), "w", encoding="utf-8") as f:
            f.write("before\n")
        with open(os.path.join(self.repo, "README.md"), "w", encoding="utf-8") as f:
            f.write("before\n")
        with open(os.path.join(self.repo, ".gitignore"), "w", encoding="utf-8") as f:
            f.write("ignored-outside.txt\n")
        _git(self.repo, "add", ".")
        _git(self.repo, "commit", "-qm", "initial")

    def tearDown(self):
        self.tmp.cleanup()

    def test_hp1_allowed_diff_is_in_scope(self):
        with open(os.path.join(self.repo, "scripts", "local-agent", "runner.py"), "w", encoding="utf-8") as f:
            f.write("after\n")

        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertTrue(result.in_scope)
        self.assertEqual(result.offending_paths, [])
        self.assertTrue(result.has_diff)

    def test_ec1_out_of_scope_and_untracked_paths_are_reported(self):
        with open(os.path.join(self.repo, "README.md"), "w", encoding="utf-8") as f:
            f.write("after\n")
        with open(os.path.join(self.repo, "outside.txt"), "w", encoding="utf-8") as f:
            f.write("new\n")

        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertFalse(result.in_scope)
        self.assertEqual(result.offending_paths, ["README.md", "outside.txt"])
        self.assertTrue(result.has_diff)

    def test_ec2_clean_worktree_is_distinct_from_an_in_scope_diff(self):
        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertTrue(result.in_scope)
        self.assertEqual(result.offending_paths, [])
        self.assertFalse(result.has_diff)

    def test_ec1_ignored_untracked_path_cannot_evade_scope_check(self):
        with open(os.path.join(self.repo, "ignored-outside.txt"), "w", encoding="utf-8") as f:
            f.write("new\n")

        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertFalse(result.in_scope)
        self.assertEqual(result.offending_paths, ["ignored-outside.txt"])
        self.assertTrue(result.has_diff)

    def test_directory_prefix_does_not_allow_similarly_named_directory(self):
        os.makedirs(os.path.join(self.repo, "scripts", "local-agent-extra"))
        with open(os.path.join(self.repo, "scripts", "local-agent-extra", "escape.py"), "w", encoding="utf-8") as f:
            f.write("new\n")

        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertFalse(result.in_scope)
        self.assertEqual(result.offending_paths, ["scripts/local-agent-extra/escape.py"])


if __name__ == "__main__":
    unittest.main()
