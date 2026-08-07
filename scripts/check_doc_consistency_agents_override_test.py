#!/usr/bin/env python3
"""Unit tests for the AGENTS.override.md drift-check function in
check-doc-consistency.sh, exercised via an isolated disposable git worktree
so the tests never touch the real repository's AGENTS.override.md."""

from __future__ import annotations

import shutil
import subprocess
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


class AgentsOverrideDriftCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.worktree = Path("/tmp") / "aos1-drift-check-unittest-worktree"
        if self.worktree.exists():
            self._remove_worktree()
        subprocess.run(
            ["git", "worktree", "add", "--detach", str(self.worktree), "HEAD", "-q"],
            cwd=REPO_ROOT,
            check=True,
        )
        # The generator/drift-check function are new, uncommitted files in the
        # primary checkout; copy them into the worktree so it can exercise
        # today's code rather than the last commit's.
        shutil.copy(
            REPO_ROOT / "scripts" / "generate-agents-override.py",
            self.worktree / "scripts" / "generate-agents-override.py",
        )
        shutil.copy(
            REPO_ROOT / "scripts" / "check-doc-consistency.sh",
            self.worktree / "scripts" / "check-doc-consistency.sh",
        )

    def tearDown(self) -> None:
        self._remove_worktree()

    def _remove_worktree(self) -> None:
        subprocess.run(
            ["git", "worktree", "remove", str(self.worktree), "--force"],
            cwd=REPO_ROOT,
            check=False,
        )
        shutil.rmtree(self.worktree, ignore_errors=True)

    def _run_check(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["bash", "scripts/check-doc-consistency.sh"],
            cwd=self.worktree,
            capture_output=True,
            text=True,
        )

    def test_hp1_check_passes_after_regeneration(self) -> None:
        subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            check=True,
        )
        result = self._run_check()
        self.assertEqual(result.returncode, 0, msg=result.stdout + result.stderr)
        self.assertIn("Documentation consistency check passed.", result.stdout)

    def test_ec1_check_fails_closed_on_hand_edited_drift(self) -> None:
        subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            check=True,
        )
        override_path = self.worktree / "AGENTS.override.md"
        with override_path.open("a", encoding="utf-8") as f:
            f.write("\nstale hand-edited line\n")

        result = self._run_check()
        self.assertEqual(result.returncode, 1)
        self.assertIn("AGENTS.override.md", result.stdout)
        self.assertIn("generate-agents-override.py --write", result.stdout)

    def test_ec1_check_catches_trailing_newline_only_drift(self) -> None:
        """Byte-exact comparison (cmp) must catch drift a naive bash $(...)
        string comparison would hide, since command substitution strips
        trailing newlines from both sides symmetrically. AGENTS.override.md
        is a hashed native-instruction attestation source, so byte-identity
        is the actual requirement, not string-identity modulo trailing
        newlines."""
        subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            check=True,
        )
        override_path = self.worktree / "AGENTS.override.md"
        with override_path.open("a", encoding="utf-8") as f:
            f.write("\n\n\n")

        result = self._run_check()
        self.assertEqual(result.returncode, 1, msg=result.stdout + result.stderr)
        self.assertIn("AGENTS.override.md: content is stale", result.stdout)

    def test_ec3_check_fails_closed_when_file_missing(self) -> None:
        override_path = self.worktree / "AGENTS.override.md"
        override_path.unlink(missing_ok=True)

        result = self._run_check()
        self.assertEqual(result.returncode, 1)
        self.assertIn("AGENTS.override.md: file does not exist", result.stdout)

    def test_ec1_fix_then_pass_round_trip(self) -> None:
        subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            check=True,
        )
        override_path = self.worktree / "AGENTS.override.md"
        with override_path.open("a", encoding="utf-8") as f:
            f.write("\ndrift\n")
        self.assertEqual(self._run_check().returncode, 1)

        subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            check=True,
        )
        result = self._run_check()
        self.assertEqual(result.returncode, 0, msg=result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
