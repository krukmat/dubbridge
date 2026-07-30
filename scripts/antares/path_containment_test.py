#!/usr/bin/env python3
"""Unit tests for scripts/antares/path_containment.py.

Covers the approved T2b happy path HP-2 and edge case EC-3 from
docs/tasks/antares-security-specialist-advisor.md.
"""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

_MODULE_SCRIPT = Path(__file__).with_name("path_containment.py")
_MODULE_SPEC = importlib.util.spec_from_file_location(
    "antares_path_containment", _MODULE_SCRIPT
)
if _MODULE_SPEC is None or _MODULE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_MODULE_SCRIPT}")
_MODULE = importlib.util.module_from_spec(_MODULE_SPEC)
sys.modules[_MODULE_SPEC.name] = _MODULE
_MODULE_SPEC.loader.exec_module(_MODULE)

check_path_containment = _MODULE.check_path_containment
resolve_within_snapshot = _MODULE.resolve_within_snapshot
TerminalStateKind = _MODULE.TerminalStateKind


class PathContainmentTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.snapshot_root = Path(self._tmp.name)
        (self.snapshot_root / "src").mkdir()
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() {}\n")

        self._outside_tmp = tempfile.TemporaryDirectory()
        self.outside_root = Path(self._outside_tmp.name)
        (self.outside_root / "secret.txt").write_text("outside\n")

    def tearDown(self) -> None:
        self._tmp.cleanup()
        self._outside_tmp.cleanup()


class ResolveWithinSnapshotTest(PathContainmentTestBase):
    def test_hp2_relative_in_snapshot_path_resolves(self) -> None:
        resolved = resolve_within_snapshot("src/main.rs", self.snapshot_root)
        self.assertEqual(resolved, (self.snapshot_root / "src" / "main.rs").resolve())

    def test_ec3_absolute_path_is_rejected(self) -> None:
        self.assertIsNone(resolve_within_snapshot("/etc/passwd", self.snapshot_root))

    def test_ec3_dotdot_traversal_is_rejected(self) -> None:
        self.assertIsNone(
            resolve_within_snapshot("src/../../outside/secret.txt", self.snapshot_root)
        )

    def test_ec3_symlink_escaping_snapshot_is_rejected(self) -> None:
        link = self.snapshot_root / "escape_link"
        link.symlink_to(self.outside_root / "secret.txt")
        self.assertIsNone(resolve_within_snapshot("escape_link", self.snapshot_root))

    def test_blank_path_is_rejected(self) -> None:
        self.assertIsNone(resolve_within_snapshot("   ", self.snapshot_root))


class CheckPathContainmentTest(PathContainmentTestBase):
    def test_hp2_all_valid_paths_produce_containment_valid(self) -> None:
        result = check_path_containment(("src/main.rs",), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.PATH_CONTAINMENT_VALID)
        self.assertEqual(result.candidates, ("src/main.rs",))
        self.assertTrue(result.is_success)

    def test_ec3_one_escaping_path_fails_the_whole_batch_closed(self) -> None:
        result = check_path_containment(
            ("src/main.rs", "../outside/secret.txt"), self.snapshot_root
        )
        self.assertEqual(result.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE)
        self.assertEqual(result.candidates, ())
        self.assertFalse(result.is_success)

    def test_empty_batch_is_rejected(self) -> None:
        result = check_path_containment((), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE)


if __name__ == "__main__":
    unittest.main()
