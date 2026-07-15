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

    def test_write_temp_card_preserves_rri_and_band_when_present(self):
        card = {
            "task_id": "TEST-03",
            "spec": "toy spec",
            "acceptance_tests": ["HP-1: trivial"],
            "allowed_paths": ["README.md"],
            "rri": 29,
            "band": "Moderate",
        }

        with tempfile.TemporaryDirectory() as base:
            path = bench._write_temp_card(card, base)
            with open(path, encoding="utf-8") as f:
                written = json.load(f)

        self.assertEqual(written["rri"], 29)
        self.assertEqual(written["band"], "Moderate")


class MainPerCardIsolation(unittest.TestCase):
    # T7d-fix: found live when MC-01's malformed argv crashed run_local_task.py
    # with an uncaught exception (see run_local_task_test.py's
    # EC1MalformedArgvType for the harness-level half of this incident). The
    # exception propagated straight through run_card() (whose own
    # worktree-teardown `finally` still ran — no orphaned worktree/branch)
    # and killed main()'s whole for-loop, silently discarding the results of
    # every card queued after the one that crashed. This test proves one
    # card's harness-level exception no longer prevents a later card in the
    # same --only invocation from running.
    def test_one_card_crash_does_not_prevent_next_card_from_running(self):
        with tempfile.TemporaryDirectory() as cards_dir, \
             tempfile.TemporaryDirectory() as out_dir, \
             tempfile.TemporaryDirectory() as work_dir:
            for task_id in ("CRASH-01", "OK-02"):
                card = {
                    "task_id": task_id,
                    "category": "docs",
                    "spec": "toy spec",
                    "acceptance_tests": ["HP-1: trivial"],
                    "allowed_paths": ["README.md"],
                    "verify_commands": ["true"],
                }
                with open(os.path.join(cards_dir, f"{task_id}.json"), "w", encoding="utf-8") as f:
                    json.dump(card, f)

            def fake_main(argv, test_runner=None, **kwargs):
                card_path = argv[argv.index("--card") + 1]
                with open(card_path, encoding="utf-8") as f:
                    task_id = json.load(f)["task_id"]
                if task_id == "CRASH-01":
                    raise RuntimeError("simulated harness-level crash")
                out_path = argv[argv.index("--out") + 1]
                with open(out_path, "w", encoding="utf-8") as f:
                    json.dump({"status": "success", "transcript": []}, f)
                return 0

            with patch.object(bench.rlt, "main", side_effect=fake_main):
                exit_code = bench.main([
                    "--cards-dir", cards_dir,
                    "--out-dir", out_dir,
                    "--work-dir", work_dir,
                ])

            self.assertEqual(exit_code, 0)
            with open(os.path.join(out_dir, "summary.json"), encoding="utf-8") as f:
                results = json.load(f)

            by_id = {r["task_id"]: r for r in results}
            self.assertEqual(set(by_id), {"CRASH-01", "OK-02"})

            crashed = by_id["CRASH-01"]
            self.assertEqual(crashed["status"], "harness_crash")
            self.assertIsNone(crashed["exit_code"])
            self.assertIn("simulated harness-level crash", crashed["error"])
            # same result shape as every other card outcome, plus `error`
            self.assertEqual(
                set(crashed) - {"error"},
                {"task_id", "category", "exit_code", "status", "elapsed_s"},
            )

            ok = by_id["OK-02"]
            self.assertEqual(ok["status"], "success")
            self.assertEqual(ok["exit_code"], 0)
            # HP-2 requires the later card's transcript artifact, not just its
            # summary.json entry, to be unaffected by the preceding crash.
            ok_transcript_path = os.path.join(out_dir, "OK-02.transcript.json")
            self.assertTrue(os.path.isfile(ok_transcript_path))
            with open(ok_transcript_path, encoding="utf-8") as f:
                ok_transcript = json.load(f)
            self.assertEqual(ok_transcript["status"], "success")

            # the crashed card must not have left a stray transcript file
            # (fake_main raises before ever writing one for CRASH-01)
            crash_transcript_path = os.path.join(out_dir, "CRASH-01.transcript.json")
            self.assertFalse(os.path.isfile(crash_transcript_path))


if __name__ == "__main__":
    unittest.main(verbosity=2)
