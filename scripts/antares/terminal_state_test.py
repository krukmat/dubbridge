#!/usr/bin/env python3
"""Unit tests for scripts/antares/terminal_state.py."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).with_name("terminal_state.py")
_SPEC = importlib.util.spec_from_file_location("antares_terminal_state", _SCRIPT)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SCRIPT}")
_MOD = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MOD
_SPEC.loader.exec_module(_MOD)

SUCCESS_KINDS = _MOD.SUCCESS_KINDS
TERMINAL_SUBMISSION_KINDS = _MOD.TERMINAL_SUBMISSION_KINDS
TerminalState = _MOD.TerminalState
TerminalStateKind = _MOD.TerminalStateKind


class TerminalStateKindTest(unittest.TestCase):
    def test_all_kinds_are_distinct_values(self) -> None:
        values = [kind.value for kind in TerminalStateKind]
        self.assertEqual(len(values), len(set(values)))

    def test_success_kinds_are_a_subset_of_all_kinds(self) -> None:
        self.assertTrue(SUCCESS_KINDS.issubset(set(TerminalStateKind)))

    def test_failure_kinds_are_not_success_kinds(self) -> None:
        failure_kinds = {
            TerminalStateKind.MALFORMED_TOOL_CALL,
            TerminalStateKind.UNSUPPORTED_TOOL_NAME,
            TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD,
            TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION,
        }
        self.assertTrue(failure_kinds.isdisjoint(SUCCESS_KINDS))


class TerminalStateTest(unittest.TestCase):
    def test_hp1_parsed_terminal_call_is_success_and_not_a_submission(self) -> None:
        state = TerminalState(kind=TerminalStateKind.PARSED_TERMINAL_CALL, argv=("ls", "-la"))
        self.assertTrue(state.is_success)
        self.assertFalse(state.is_terminal_submission)
        self.assertEqual(state.argv, ("ls", "-la"))

    def test_hp2_no_vulnerability_found_is_success_and_is_a_submission(self) -> None:
        state = TerminalState(kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)
        self.assertTrue(state.is_success)
        self.assertTrue(state.is_terminal_submission)
        self.assertEqual(state.candidates, ())

    def test_submitted_vulnerable_files_is_a_submission(self) -> None:
        state = TerminalState(
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES, candidates=("a.rs", "b.rs")
        )
        self.assertTrue(state.is_success)
        self.assertTrue(state.is_terminal_submission)

    def test_ec1_malformed_tool_call_is_not_success(self) -> None:
        state = TerminalState(kind=TerminalStateKind.MALFORMED_TOOL_CALL, detail="bad json")
        self.assertFalse(state.is_success)
        self.assertFalse(state.is_terminal_submission)

    def test_terminal_submission_kinds_excludes_parsed_terminal_call(self) -> None:
        self.assertNotIn(TerminalStateKind.PARSED_TERMINAL_CALL, TERMINAL_SUBMISSION_KINDS)


if __name__ == "__main__":
    unittest.main()
