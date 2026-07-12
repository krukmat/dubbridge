#!/usr/bin/env python3
"""Unit tests for run_stage1_benchmark.py (mocked subprocess/runner, no live model)."""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "local-agent"))
import run_stage1_benchmark as bench


class MakeTestRunner(unittest.TestCase):
    def test_all_commands_pass_returns_passed_true(self):
        with patch("subprocess.run") as mock_run:
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = "ok"
            mock_run.return_value.stderr = ""

            runner = bench.make_test_runner(["cargo test -p ingestion", "cargo clippy -p ingestion"])
            result = runner("/some/worktree")

        self.assertTrue(result["passed"])
        self.assertEqual(mock_run.call_count, 2)

    def test_first_command_fails_short_circuits_remaining_commands(self):
        with patch("subprocess.run") as mock_run:
            mock_run.return_value.returncode = 1
            mock_run.return_value.stdout = ""
            mock_run.return_value.stderr = "error: test failed"

            runner = bench.make_test_runner(["cargo test -p ingestion", "cargo clippy -p ingestion"])
            result = runner("/some/worktree")

        self.assertFalse(result["passed"])
        self.assertEqual(mock_run.call_count, 1)
        self.assertIn("error: test failed", result["output"])

    def test_timeout_reports_failure_without_raising(self):
        import subprocess as sp

        with patch("subprocess.run", side_effect=sp.TimeoutExpired(cmd="make qa-mobile", timeout=600)):
            runner = bench.make_test_runner(["make qa-mobile"])
            result = runner("/some/worktree")

        self.assertFalse(result["passed"])
        self.assertIn("TIMEOUT", result["output"])


class WorktreeLifecycle(unittest.TestCase):
    def test_setup_creates_isolated_worktree_teardown_removes_it(self):
        with tempfile.TemporaryDirectory() as base:
            worktree_path, branch = bench.setup_worktree("TEST-01", base)
            self.assertTrue(os.path.isdir(worktree_path))

            bench.teardown_worktree(worktree_path, branch)
            self.assertFalse(os.path.isdir(worktree_path))


class RunCardOrchestration(unittest.TestCase):
    def test_run_card_invokes_runner_and_cleans_up_worktree(self):
        card = {
            "task_id": "TEST-02",
            "category": "docs",
            "spec": "toy spec",
            "acceptance_tests": ["HP-1: trivial"],
            "allowed_paths": ["README.md"],
            "verify_commands": ["true"],
        }

        with tempfile.TemporaryDirectory() as base, tempfile.TemporaryDirectory() as out_dir:
            def fake_main(argv, test_runner=None, **kwargs):
                out_path = argv[argv.index("--out") + 1]
                with open(out_path, "w", encoding="utf-8") as f:
                    json.dump({"status": "success", "transcript": []}, f)
                return 0

            with patch.object(bench.rlt, "main", side_effect=fake_main) as mock_main:
                result = bench.run_card(card, base, out_dir, "http://localhost:11434", "qwen3.6:35b-a3b")

        self.assertEqual(result["task_id"], "TEST-02")
        self.assertEqual(result["status"], "success")
        self.assertEqual(result["exit_code"], 0)
        mock_main.assert_called_once()
        # worktree must not survive run_card (teardown happens even though
        # rlt.main is mocked and never touches the worktree itself)
        worktree_path = os.path.join(base, "bench-test-02")
        self.assertFalse(os.path.isdir(worktree_path))


if __name__ == "__main__":
    unittest.main(verbosity=2)
