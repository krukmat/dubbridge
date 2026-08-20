#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

REPO_ROOT = Path(__file__).resolve().parent.parent


SCRIPT = Path(__file__).with_name("generate-agents-override.py")
SPEC = importlib.util.spec_from_file_location("generate_agents_override", SCRIPT)
assert SPEC is not None
generator = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["generate_agents_override"] = generator
SPEC.loader.exec_module(generator)


class GenerateAgentsOverrideTest(unittest.TestCase):
    def test_hp1_generate_matches_fixed_concatenation(self) -> None:
        """The fixed source set is now the single AGENTS.md bootstrap."""
        with patch.object(
            generator,
            "read_source",
            return_value="agents-bootstrap\n",
        ):
            result = generator.generate()
        self.assertEqual(result, "agents-bootstrap\n")

    def test_hp1_source_set_contains_only_agents_md(self) -> None:
        self.assertEqual(generator.SOURCE_RELATIVE_PATHS, ("AGENTS.md",))

    def test_hp2_bootstrap_routes_to_canonical_sources_on_demand(self) -> None:
        result = generator.generate()
        for relative_path in (
            "docs/playbooks/AGENT_WORKFLOW_GUIDE.md",
            "docs/policies/HITL_AUTONOMY_POLICY.md",
            "docs/policies/RRI_POLICY.md",
            "docs/plan/roadmap.md",
            "docs/architecture.md",
            "docs/adr/",
            "docs/plan/",
            "docs/tasks/",
        ):
            self.assertIn(relative_path, result)

    def test_hp2_generate_includes_hitl_source(self) -> None:
        """Compatibility ID: HITL is now a routed source, not inlined content."""
        self.assertIn("docs/policies/HITL_AUTONOMY_POLICY.md", generator.generate())

    def test_hp3_generate_is_idempotent(self) -> None:
        with patch.object(
            generator,
            "read_source",
            side_effect=lambda relative_path: f"content-of-{relative_path}",
        ):
            first = generator.generate()
            second = generator.generate()
        self.assertEqual(first, second)

    def test_hp3_no_separator_inserted_between_sources(self) -> None:
        """Compatibility ID: the one-source projection inserts no wrapper text."""
        with patch.object(
            generator,
            "read_source",
            side_effect=lambda relative_path: "X",
        ):
            result = generator.generate()
        self.assertEqual(result, "X")

    def test_ec2_missing_source_file_exits_nonzero(self) -> None:
        with patch.object(Path, "is_file", return_value=False):
            with self.assertRaises(SystemExit) as ctx:
                generator.read_source("AGENTS.md")
        self.assertIn("missing source file", str(ctx.exception))

    def test_ec2_empty_source_file_exits_nonzero(self) -> None:
        with patch.object(Path, "is_file", return_value=True), patch.object(
            Path, "read_text", return_value=""
        ):
            with self.assertRaises(SystemExit) as ctx:
                generator.read_source("AGENTS.md")
        self.assertIn("empty source file", str(ctx.exception))

    def test_ec2_oversized_bootstrap_exits_nonzero(self) -> None:
        oversized = "X" * (generator.MAX_OUTPUT_BYTES + 1)
        with patch.object(generator, "read_source", return_value=oversized):
            with self.assertRaises(SystemExit) as ctx:
                generator.generate()
        self.assertIn("exceeds", str(ctx.exception))

    def test_ec2_missing_source_does_not_call_generate_further(self) -> None:
        calls: list[str] = []

        def fake_read_source(relative_path: str) -> str:
            calls.append(relative_path)
            raise SystemExit("generate-agents-override: missing source file: " + relative_path)

        with patch.object(generator, "read_source", side_effect=fake_read_source):
            with self.assertRaises(SystemExit):
                generator.generate()
        self.assertEqual(
            calls,
            ["AGENTS.md"],
        )

    def test_main_default_mode_writes_to_stdout_not_file(self) -> None:
        with patch.object(generator, "generate", return_value="generated-content"), patch.object(
            Path, "write_text"
        ) as mock_write, patch("sys.stdout") as mock_stdout:
            exit_code = generator.main([])
        self.assertEqual(exit_code, 0)
        mock_write.assert_not_called()
        mock_stdout.write.assert_called_once_with("generated-content")

    def test_main_write_mode_writes_output_file(self) -> None:
        with patch.object(generator, "generate", return_value="generated-content"), patch.object(
            Path, "write_text"
        ) as mock_write:
            exit_code = generator.main(["--write"])
        self.assertEqual(exit_code, 0)
        mock_write.assert_called_once_with("generated-content", encoding="utf-8")


class GenerateAgentsOverrideRealFilesystemTest(unittest.TestCase):
    """EC-2 exercised against a real emptied source file and a real
    AGENTS.override.md on disk, in an isolated worktree, to close the
    mock-only coverage gap the phase-2 reviewer flagged for --write mode."""

    def setUp(self) -> None:
        self.worktree = Path("/tmp") / "aos1-ec2-write-mode-unittest-worktree"
        if self.worktree.exists():
            self._remove_worktree()
        subprocess.run(
            ["git", "worktree", "add", "--detach", str(self.worktree), "HEAD", "-q"],
            cwd=REPO_ROOT,
            check=True,
        )
        shutil.copy(
            REPO_ROOT / "scripts" / "generate-agents-override.py",
            self.worktree / "scripts" / "generate-agents-override.py",
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

    def test_ec2_write_mode_does_not_modify_output_on_empty_source(self) -> None:
        override_path = self.worktree / "AGENTS.override.md"
        before = override_path.read_bytes()

        (self.worktree / "AGENTS.md").write_text("", encoding="utf-8")
        result = subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            capture_output=True,
            text=True,
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("empty source file: AGENTS.md", result.stderr)
        after = override_path.read_bytes()
        self.assertEqual(before, after, "write mode must not modify AGENTS.override.md on EC-2 failure")

    def test_ec2_write_mode_does_not_create_output_on_missing_source(self) -> None:
        override_path = self.worktree / "AGENTS.override.md"
        override_path.unlink()
        (self.worktree / "AGENTS.md").unlink()

        result = subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            capture_output=True,
            text=True,
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("missing source file: AGENTS.md", result.stderr)
        self.assertFalse(override_path.exists(), "write mode must not create a partial AGENTS.override.md")

    def test_ec2_write_mode_does_not_modify_output_on_oversized_source(self) -> None:
        override_path = self.worktree / "AGENTS.override.md"
        before = override_path.read_bytes()
        (self.worktree / "AGENTS.md").write_text(
            "X" * (generator.MAX_OUTPUT_BYTES + 1), encoding="utf-8"
        )

        result = subprocess.run(
            ["python3", "scripts/generate-agents-override.py", "--write"],
            cwd=self.worktree,
            capture_output=True,
            text=True,
        )

        self.assertEqual(result.returncode, 1)
        self.assertIn("exceeds", result.stderr)
        self.assertEqual(before, override_path.read_bytes())


if __name__ == "__main__":
    unittest.main()
