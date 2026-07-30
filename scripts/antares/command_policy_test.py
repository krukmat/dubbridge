#!/usr/bin/env python3
"""Unit tests for scripts/antares/command_policy.py.

Covers the approved T2b happy path HP-1 and edge cases EC-1, EC-2, EC-3 from
docs/tasks/antares-security-specialist-advisor.md.
"""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

_MODULE_SCRIPT = Path(__file__).with_name("command_policy.py")
_MODULE_SPEC = importlib.util.spec_from_file_location(
    "antares_command_policy", _MODULE_SCRIPT
)
if _MODULE_SPEC is None or _MODULE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_MODULE_SCRIPT}")
_MODULE = importlib.util.module_from_spec(_MODULE_SPEC)
sys.modules[_MODULE_SPEC.name] = _MODULE
_MODULE_SPEC.loader.exec_module(_MODULE)

validate_command = _MODULE.validate_command
TerminalStateKind = _MODULE.TerminalStateKind


class CommandPolicyTestBase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.snapshot_root = Path(self._tmp.name)
        (self.snapshot_root / "src").mkdir()
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() {}\n")

    def tearDown(self) -> None:
        self._tmp.cleanup()


class ValidateCommandHappyPathTest(CommandPolicyTestBase):
    def test_hp1_allowlisted_command_with_approved_options_is_valid_plan(self) -> None:
        argv = ("grep", "-n", "TODO", "src/main.rs")
        result = validate_command(argv, self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_PLAN_VALID)
        self.assertEqual(result.argv, argv)
        self.assertTrue(result.is_success)

    def test_hp1_find_with_allowed_name_option_is_valid_plan(self) -> None:
        argv = ("find", "src", "-name", "*.rs")
        result = validate_command(argv, self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_PLAN_VALID)

    def test_hp1_ls_with_no_operand_is_valid_plan(self) -> None:
        result = validate_command(("ls", "-la"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_PLAN_VALID)


class ValidateCommandShellSyntaxTest(CommandPolicyTestBase):
    def test_ec1_pipe_character_is_rejected(self) -> None:
        result = validate_command(("grep", "TODO", "src/main.rs", "|", "wc"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)
        self.assertFalse(result.is_success)

    def test_ec1_semicolon_is_rejected(self) -> None:
        result = validate_command(("cat", "src/main.rs;", "rm"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)

    def test_ec1_command_substitution_is_rejected(self) -> None:
        result = validate_command(("cat", "$(whoami)"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)

    def test_ec1_environment_assignment_prefix_is_rejected(self) -> None:
        result = validate_command(("FOO=bar", "cat", "src/main.rs"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)

    def test_ec1_output_redirect_is_rejected(self) -> None:
        result = validate_command(("cat", "src/main.rs", ">", "out.txt"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)

    def test_empty_argv_is_rejected_as_shell_syntax(self) -> None:
        result = validate_command((), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX)


class ValidateCommandExecutableAndOptionTest(CommandPolicyTestBase):
    def test_ec2_disallowed_executable_is_rejected(self) -> None:
        result = validate_command(("rm", "-rf", "src"), self.snapshot_root)
        self.assertEqual(
            result.kind, TerminalStateKind.COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED
        )
        self.assertFalse(result.is_success)

    def test_ec2_find_exec_is_rejected_even_though_find_is_allowed(self) -> None:
        # The `;` terminator is intentionally omitted: it would itself be
        # rejected as shell syntax before the option check runs. This test
        # isolates the option-allowlist rejection path specifically.
        result = validate_command(("find", "src", "-exec", "rm", "{}"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED)

    def test_ec2_disallowed_option_on_allowlisted_executable_is_rejected(self) -> None:
        result = validate_command(("grep", "--include=*.rs", "TODO", "src"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED)

    def test_ec2_option_with_value_truncated_at_end_of_argv_is_rejected(self) -> None:
        # Pass 1 Reflection finding: "find src -name" (missing the pattern
        # value) must not silently pass as a valid plan.
        result = validate_command(("find", "src", "-name"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED)


class ValidateCommandPathContainmentTest(CommandPolicyTestBase):
    def test_ec3_absolute_path_operand_is_rejected(self) -> None:
        result = validate_command(("cat", "/etc/passwd"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE)
        self.assertFalse(result.is_success)

    def test_ec3_dotdot_traversal_operand_is_rejected(self) -> None:
        result = validate_command(("cat", "../outside.txt"), self.snapshot_root)
        self.assertEqual(result.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE)

    def test_ec3_symlink_escape_operand_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as outside:
            outside_file = Path(outside) / "secret.txt"
            outside_file.write_text("outside\n")
            link = self.snapshot_root / "escape_link"
            link.symlink_to(outside_file)
            result = validate_command(("cat", "escape_link"), self.snapshot_root)
            self.assertEqual(
                result.kind, TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE
            )


if __name__ == "__main__":
    unittest.main()
