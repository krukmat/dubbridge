#!/usr/bin/env python3
"""Unit tests for scripts/antares/tool_call_parser.py.

Covers the approved T2a happy paths (HP-1, HP-2) and edge cases
(EC-1..EC-4) from docs/tasks/antares-security-specialist-advisor.md.
"""

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path

_PARSER_SCRIPT = Path(__file__).with_name("tool_call_parser.py")
_PARSER_SPEC = importlib.util.spec_from_file_location("antares_tool_call_parser", _PARSER_SCRIPT)
if _PARSER_SPEC is None or _PARSER_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_PARSER_SCRIPT}")
_PARSER_MOD = importlib.util.module_from_spec(_PARSER_SPEC)
sys.modules[_PARSER_SPEC.name] = _PARSER_MOD
_PARSER_SPEC.loader.exec_module(_PARSER_MOD)

check_duplicate_submission = _PARSER_MOD.check_duplicate_submission
parse_tool_call = _PARSER_MOD.parse_tool_call
TerminalState = _PARSER_MOD.TerminalState
TerminalStateKind = _PARSER_MOD.TerminalStateKind


def _call(tool: str, payload: dict | None = None) -> str:
    message: dict = {"tool": tool}
    if payload is not None:
        message["payload"] = payload
    return json.dumps(message)


class ParseTerminalCallTest(unittest.TestCase):
    def test_hp1_valid_terminal_call_preserves_argv_order(self) -> None:
        raw = _call("terminal", {"argv": ["grep", "-n", "TODO", "src/main.rs"]})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.PARSED_TERMINAL_CALL)
        self.assertEqual(result.argv, ("grep", "-n", "TODO", "src/main.rs"))

    def test_hp1_empty_argv_is_a_valid_parsed_call(self) -> None:
        raw = _call("terminal", {"argv": []})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.PARSED_TERMINAL_CALL)
        self.assertEqual(result.argv, ())

    def test_terminal_call_missing_argv_is_malformed(self) -> None:
        raw = _call("terminal", {})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)

    def test_ec4_terminal_call_with_integer_argv_element_is_malformed_not_coerced(self) -> None:
        raw = _call("terminal", {"argv": ["cat", 42]})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)
        self.assertEqual(result.argv, ())

    def test_ec4_terminal_call_with_non_list_argv_is_malformed(self) -> None:
        raw = _call("terminal", {"argv": "ls -la"})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)


class ParseSubmitVulnerableFilesTest(unittest.TestCase):
    def test_valid_submission_preserves_candidate_paths(self) -> None:
        raw = _call("submit_vulnerable_files", {"candidates": ["crates/db/src/lib.rs"]})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.SUBMITTED_VULNERABLE_FILES)
        self.assertEqual(result.candidates, ("crates/db/src/lib.rs",))

    def test_ec2_missing_candidates_is_malformed_submit_payload(self) -> None:
        raw = _call("submit_vulnerable_files", {})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD)

    def test_ec2_empty_candidates_list_is_malformed_submit_payload(self) -> None:
        raw = _call("submit_vulnerable_files", {"candidates": []})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD)

    def test_ec2_blank_candidate_path_is_malformed_submit_payload(self) -> None:
        raw = _call("submit_vulnerable_files", {"candidates": ["  "]})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD)

    def test_ec4_non_string_candidate_is_malformed_not_coerced(self) -> None:
        raw = _call("submit_vulnerable_files", {"candidates": ["a.rs", 7]})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)
        self.assertEqual(result.candidates, ())


class ParseSubmitNoVulnerabilityFoundTest(unittest.TestCase):
    def test_hp2_no_vulnerability_found_is_explicit_negative_result(self) -> None:
        raw = _call("submit_no_vulnerability_found")

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)
        self.assertEqual(result.candidates, ())

    def test_hp2_result_is_distinguishable_from_vulnerable_files_with_empty_list(self) -> None:
        no_vuln = parse_tool_call(_call("submit_no_vulnerability_found"))
        malformed_empty = parse_tool_call(_call("submit_vulnerable_files", {"candidates": []}))

        self.assertNotEqual(no_vuln.kind, malformed_empty.kind)
        self.assertEqual(no_vuln.kind, TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)


class ParseMalformedJsonAndUnsupportedToolTest(unittest.TestCase):
    def test_ec1_malformed_json_records_malformed_tool_call(self) -> None:
        result = parse_tool_call("{not valid json")

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)

    def test_ec1_non_object_json_records_malformed_tool_call(self) -> None:
        result = parse_tool_call(json.dumps(["terminal", "ls"]))

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)

    def test_ec1_empty_string_records_malformed_tool_call(self) -> None:
        result = parse_tool_call("")

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)

    def test_ec2_unsupported_tool_name_is_rejected_distinctly(self) -> None:
        raw = _call("delete_repository", {})

        result = parse_tool_call(raw)

        self.assertEqual(result.kind, TerminalStateKind.UNSUPPORTED_TOOL_NAME)

    def test_missing_tool_field_is_malformed_tool_call(self) -> None:
        result = parse_tool_call(json.dumps({"payload": {}}))

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)

    def test_non_string_tool_field_is_malformed_tool_call(self) -> None:
        result = parse_tool_call(json.dumps({"tool": 5}))

        self.assertEqual(result.kind, TerminalStateKind.MALFORMED_TOOL_CALL)


class CheckDuplicateSubmissionTest(unittest.TestCase):
    def test_ec3_two_terminal_submissions_fail_closed_as_duplicate(self) -> None:
        first = TerminalState(kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)
        second = TerminalState(
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES, candidates=("a.rs",)
        )

        result = check_duplicate_submission(first, second)

        self.assertEqual(result.kind, TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION)

    def test_ec3_duplicate_check_does_not_silently_prefer_either_payload(self) -> None:
        first = TerminalState(
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES, candidates=("a.rs",)
        )
        second = TerminalState(
            kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES, candidates=("b.rs",)
        )

        result = check_duplicate_submission(first, second)

        self.assertEqual(result.kind, TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION)
        self.assertEqual(result.candidates, ())

    def test_non_submission_first_state_does_not_trigger_duplicate(self) -> None:
        first = TerminalState(kind=TerminalStateKind.PARSED_TERMINAL_CALL, argv=("ls",))
        second = TerminalState(kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)

        result = check_duplicate_submission(first, second)

        self.assertIs(result, second)
        self.assertEqual(result.kind, TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)


if __name__ == "__main__":
    unittest.main()
