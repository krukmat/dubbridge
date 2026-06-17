#!/usr/bin/env python3
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_SOURCE = REPO_ROOT / "scripts" / "check-roadmap-drift.sh"


class RoadmapDriftGate(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "scripts").mkdir()
        shutil.copy(SCRIPT_SOURCE, self.root / "scripts" / "check-roadmap-drift.sh")
        os.chmod(self.root / "scripts" / "check-roadmap-drift.sh", 0o755)
        (self.root / "docs" / "plan").mkdir(parents=True)
        (self.root / "docs" / "tasks").mkdir(parents=True)
        self.run_cmd("git", "init")
        self.run_cmd("git", "config", "user.email", "test@example.com")
        self.run_cmd("git", "config", "user.name", "Test User")

    def tearDown(self):
        self.tmp.cleanup()

    def run_cmd(self, *args, check=True):
        result = subprocess.run(args, cwd=self.root, capture_output=True, text=True)
        if check and result.returncode != 0:
            self.fail(f"{args} failed\nstdout={result.stdout}\nstderr={result.stderr}")
        return result

    def write(self, path, contents):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(contents, encoding="utf-8")

    def commit_all(self):
        self.run_cmd("git", "add", ".")
        self.run_cmd("git", "commit", "-m", "fixture")

    def check_gate(self):
        return self.run_cmd("bash", "scripts/check-roadmap-drift.sh", check=False)

    def test_done_phase_with_committed_sid_evidence_passes(self):
        self.write(
            "docs/plan/roadmap.md",
            "| Phase | Title | Depends | Status | Evidence |\n"
            "|---|---|---|---|---|\n"
            "| **S-160** | Review | — | ✅ done | closed |\n",
        )
        self.write("docs/tasks/s-160-review.md", "# S-160\n")
        self.commit_all()

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("passed", result.stdout)

    def test_done_phase_without_sid_evidence_fails(self):
        self.write(
            "docs/plan/roadmap.md",
            "| Phase | Title | Depends | Status | Evidence |\n"
            "|---|---|---|---|---|\n"
            "| **S-777** | Missing | — | ✅ done | closed |\n",
        )
        self.commit_all()

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("S-777", result.stdout)

    def test_done_phase_with_uncommitted_evidence_fails(self):
        self.write(
            "docs/plan/roadmap.md",
            "| Phase | Title | Depends | Status | Evidence |\n"
            "|---|---|---|---|---|\n"
            "| **S-160** | Review | — | ✅ done | closed |\n",
        )
        self.write("docs/tasks/s-160-review.md", "# S-160\n")
        self.commit_all()
        self.write("docs/tasks/s-160-review.md", "# S-160\n\nUncommitted update.\n")

        result = self.check_gate()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("uncommitted", result.stdout)

    def test_non_done_phase_statuses_are_ignored(self):
        self.write(
            "docs/plan/roadmap.md",
            "| Phase | Title | Depends | Status | Evidence |\n"
            "|---|---|---|---|---|\n"
            "| **S-090** | Recording | — | 🟡 REPLANNED | plan |\n"
            "| **S-091** | Placeholder | — | ⬜ | plan |\n"
            "| **S-092** | Cancelled | — | cancelled | plan |\n"
            "| **S-093** | Superseded | — | superseded | plan |\n",
        )
        self.commit_all()

        result = self.check_gate()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
