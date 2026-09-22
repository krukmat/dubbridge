"""Behavioral contracts for the human CLI adapter; never invoke a model."""
from __future__ import annotations

import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


SCRIPT = Path(__file__).with_name("human-orchestrator.py").resolve()
SPEC = importlib.util.spec_from_file_location("human_orchestrator", SCRIPT)
launcher = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(launcher)


class LauncherTest(unittest.TestCase):
    def setUp(self):
        self.output = io.StringIO()
        self.enterContext(contextlib.redirect_stdout(self.output))
        self.enterContext(contextlib.redirect_stderr(self.output))

    def test_hp1_ec2_dry_run_never_spawns_for_any_alias(self):
        with mock.patch.object(launcher.subprocess, "run") as child:
            for alias in launcher.TOOLS:
                self.assertEqual(launcher.main(["run", alias, "--", "$(touch nope)"]), 0)
            child.assert_not_called()
        self.assertIn("DRY RUN", self.output.getvalue())

    def test_hp3_ec2_literal_arguments_and_repo_cwd(self):
        arguments = ["--content", "a b.md", "; touch nope", "$(uname)", "`id`", "--"]
        with mock.patch.object(launcher.subprocess, "run") as child:
            child.return_value.returncode = 7
            code = launcher.main(["--execute", "run", "review", "--", *arguments])
        self.assertEqual(code, 7)
        child.assert_called_once_with(
            [sys.executable, str(launcher.REPO_ROOT / launcher.TOOLS["review"]), *arguments],
            cwd=launcher.REPO_ROOT, check=False,
        )

    def test_ec1_bad_aliases_and_missing_arguments_do_not_spawn(self):
        with mock.patch.object(launcher.subprocess, "run") as child:
            for args in ([], ["run"], ["run", "typo"], ["run", "--evil"],
                         ["run", "rri;id"], ["--unknown"], ["run", "--execute"]):
                with self.subTest(args=args), self.assertRaises(SystemExit) as exc:
                    launcher.main(args)
                self.assertEqual(exc.exception.code, 2)
            child.assert_not_called()

    def test_ec1_missing_tool_fails_before_launch(self):
        with tempfile.TemporaryDirectory(prefix="kt space ") as directory:
            with mock.patch.object(launcher, "REPO_ROOT", Path(directory)):
                with mock.patch.object(launcher.subprocess, "run") as child:
                    with self.assertRaises(SystemExit) as exc:
                        launcher.main(["--execute", "run", "rri"])
                    self.assertEqual(exc.exception.code, 2)
                    child.assert_not_called()

    def test_hp2_status_only_runs_local_git_inspections(self):
        with mock.patch.object(launcher.subprocess, "run") as child:
            child.return_value.returncode = 0
            self.assertEqual(launcher.main(["status"]), 0)
        self.assertEqual(child.call_args_list, [
            mock.call(list(command), cwd=launcher.REPO_ROOT, check=False)
            for command in (
                ("git", "status", "--short", "--branch"),
                ("git", "rev-parse", "--short", "HEAD"),
                ("git", "diff", "--stat"),
                ("git", "worktree", "list"),
            )
        ])
        self.assertEqual(len(child.call_args_list), 4)
        self.assertTrue(all(call.args[0][0] == "git" for call in child.call_args_list))

    def test_ec3_status_failure_stops_without_retry(self):
        with mock.patch.object(launcher.subprocess, "run") as child:
            child.return_value.returncode = 128
            self.assertEqual(launcher.main(["status"]), 128)
            child.assert_called_once()

    def test_ec3_launch_error_is_nonzero_without_retry(self):
        with mock.patch.object(launcher.subprocess, "run", side_effect=OSError("unavailable")) as child:
            self.assertEqual(launcher.main(["--execute", "run", "rri"]), 2)
            child.assert_called_once()
        self.assertIn("Unable to launch command", self.output.getvalue())

    def test_ec3_signal_and_keyboard_interrupt_exit_codes(self):
        with mock.patch.object(launcher.subprocess, "run") as child:
            child.return_value.returncode = -15
            self.assertEqual(launcher.invoke(["example"]), 143)
            child.side_effect = KeyboardInterrupt
            self.assertEqual(launcher.invoke(["example"]), 130)

    def test_hp1_hp3_real_preflight_from_another_working_directory(self):
        with tempfile.TemporaryDirectory(prefix="kt space ") as directory:
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--execute", "run", "preflight", "--", "--print-summary"],
                cwd=directory, capture_output=True, text=True,
            )
            self.assertEqual(list(Path(directory).iterdir()), [])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("DubBridge agent preflight", result.stdout)

    def test_ec3_real_child_usage_error_is_propagated(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--execute", "run", "preflight", "--", "--not-a-valid-option"],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("unrecognized arguments", result.stderr)


if __name__ == "__main__":
    unittest.main()
