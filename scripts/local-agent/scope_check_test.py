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

    # --- T7f: build-artifact directory exclusion ---

    def _make_repo_with_gitignore_preseeded(self):
        """Create a repo like setUp but with extra entries pre-seeded in .gitignore."""
        tmp = tempfile.TemporaryDirectory()
        repo = tmp.name
        _git(repo, "init", "-q")
        _git(repo, "config", "user.email", "scope-check@example.test")
        _git(repo, "config", "user.name", "Scope Check")
        os.makedirs(os.path.join(repo, "scripts", "local-agent"))
        with open(os.path.join(repo, "scripts", "local-agent", "runner.py"), "w", encoding="utf-8") as f:
            f.write("before\n")
        with open(os.path.join(repo, "README.md"), "w", encoding="utf-8") as f:
            f.write("before\n")
        # Pre-seed .gitignore so that target/ is ignored from the start.
        with open(os.path.join(repo, ".gitignore"), "w", encoding="utf-8") as f:
            f.write("target/\nignored-outside.txt\n")
        _git(repo, "add", ".")
        _git(repo, "commit", "-qm", "initial")
        return repo, tmp

    def test_build_artifact_path_is_excluded_from_check(self):
        """HP-1: changes inside target/ (or other artifact dir) plus allowed paths -> in_scope=True."""
        repo, holder = self._make_repo_with_gitignore_preseeded()
        try:
            artifact_dir = os.path.join(repo, "target", "debug", "build", "foo")
            os.makedirs(artifact_dir)
            with open(os.path.join(artifact_dir, "output"), "w", encoding="utf-8") as f:
                f.write("artifact\n")

            with open(os.path.join(repo, "scripts", "local-agent", "runner.py"), "w", encoding="utf-8") as f:
                f.write("after\n")

            result = scope_check.check_scope(repo, ["scripts/local-agent"])

            self.assertTrue(result.in_scope)
            self.assertEqual(result.offending_paths, [])
            self.assertTrue(result.has_diff)
        finally:
            holder.cleanup()

    def test_gitignored_non_artifact_path_is_still_flagged(self):
        """EC-1 (regression): a gitignored path that is NOT an artifact directory is still flagged."""
        repo, holder = self._make_repo_with_gitignore_preseeded()
        try:
            with open(os.path.join(repo, ".gitignore"), "a", encoding="utf-8") as f:
                f.write(".env\n")
            with open(os.path.join(repo, ".env"), "w", encoding="utf-8") as f:
                f.write("SECRET=123\n")

            result = scope_check.check_scope(repo, ["scripts/local-agent"])

            self.assertFalse(result.in_scope)
            self.assertIn(".env", result.offending_paths)
            self.assertTrue(result.has_diff)
        finally:
            holder.cleanup()

    def test_untracked_non_ignored_path_outside_allowed_is_flagged(self):
        """EC-2: plain untracked (non-ignored) path outside allowed_paths is still flagged."""
        with open(os.path.join(self.repo, "outside.txt"), "w", encoding="utf-8") as f:
            f.write("new\n")

        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertFalse(result.in_scope)
        self.assertIn("outside.txt", result.offending_paths)
        self.assertTrue(result.has_diff)

    def test_tracked_path_named_like_artifact_dir_is_still_flagged(self):
        """Regression for the T7f tightening: the artifact exclusion applies only to
        the --ignored scan, so a *tracked* change under a path component that happens
        to be named like a build-artifact directory (e.g. build/) must still be
        checked against allowed_paths like any other tracked change."""
        os.makedirs(os.path.join(self.repo, "build"))
        with open(os.path.join(self.repo, "build", "config.rs"), "w", encoding="utf-8") as f:
            f.write("tracked\n")
        _git(self.repo, "add", "build/config.rs")
        _git(self.repo, "commit", "-qm", "add tracked build/ file")

        with open(os.path.join(self.repo, "build", "config.rs"), "w", encoding="utf-8") as f:
            f.write("changed\n")

        result = scope_check.check_scope(self.repo, ["scripts/local-agent"])

        self.assertFalse(result.in_scope)
        self.assertIn("build/config.rs", result.offending_paths)


if __name__ == "__main__":
    unittest.main()
