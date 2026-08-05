"""Fail-closed terminal-state contract for Antares tool-call parsing (T2a)
and command/path policy validation (T2b).

Every outcome the parser or policy layer can produce is one of these
explicit, machine-distinguishable states. No two failure modes collapse into
a single generic "error" bucket: downstream policy (T2b), sandbox (T2c), and
artifact (T2d) layers need to tell them apart.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class TerminalStateKind(Enum):
    """The distinct outcomes `parse_tool_call` can return.

    PARSED_TERMINAL_CALL / SUBMITTED_VULNERABLE_FILES /
    SUBMITTED_NO_VULNERABILITY_FOUND are successful parses (HP-1, HP-2).
    The remaining values are fail-closed rejections (EC-1..EC-4); each is
    kept distinct rather than folded into one generic failure so a caller
    can react differently to, say, a malformed payload versus an
    unsupported tool name.
    """

    PARSED_TERMINAL_CALL = "parsed_terminal_call"
    SUBMITTED_VULNERABLE_FILES = "submitted_vulnerable_files"
    SUBMITTED_NO_VULNERABILITY_FOUND = "submitted_no_vulnerability_found"
    MALFORMED_TOOL_CALL = "malformed_tool_call"
    UNSUPPORTED_TOOL_NAME = "unsupported_tool_name"
    MALFORMED_SUBMIT_PAYLOAD = "malformed_submit_payload"
    DUPLICATE_TERMINAL_SUBMISSION = "duplicate_terminal_submission"

    # T2b: command-policy and path-containment outcomes. Kept distinct from
    # the T2a parser states above -- a caller must be able to tell "the
    # tool-call JSON was malformed" apart from "the JSON parsed fine but the
    # requested command/path violates the execution policy".
    COMMAND_PLAN_VALID = "command_plan_valid"
    PATH_CONTAINMENT_VALID = "path_containment_valid"
    COMMAND_REJECTED_SHELL_SYNTAX = "command_rejected_shell_syntax"
    COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED = "command_rejected_executable_not_allowed"
    COMMAND_REJECTED_OPTION_NOT_ALLOWED = "command_rejected_option_not_allowed"
    PATH_REJECTED_CONTAINMENT_ESCAPE = "path_rejected_containment_escape"

    # T2c-1: sandboxed subprocess execution outcomes. Kept distinct from the
    # T2b policy-validation states above -- COMMAND_PLAN_VALID means "this
    # argv is authorized to run"; the states below describe what actually
    # happened when it did.
    SANDBOX_EXECUTION_COMPLETE = "sandbox_execution_complete"
    SANDBOX_RUNTIME_UNAVAILABLE = "sandbox_runtime_unavailable"
    SANDBOX_COMMAND_TIMED_OUT = "sandbox_command_timed_out"

    # T2c-2: aggregate session-accounting outcomes layered on top of T2c-1's
    # single-process states above. Kept distinct from SANDBOX_COMMAND_TIMED_OUT
    # (a per-command result) and from each other -- a caller must be able to
    # tell "this one command wrote too much output" apart from "the whole
    # session ran out of wall-clock budget" apart from "the 15-command counter
    # was already exhausted before this command could start".
    SANDBOX_OUTPUT_CAP_EXCEEDED = "sandbox_output_cap_exceeded"
    SANDBOX_WALL_BUDGET_EXCEEDED = "sandbox_wall_budget_exceeded"
    SANDBOX_BUDGET_EXHAUSTED = "sandbox_budget_exhausted"
    # A kill was issued but active post-kill verification could not confirm
    # the process group actually exited within its bounded grace period.
    # Kept distinct from the other T2c-2 kinds above (all of which imply
    # "killed and confirmed gone") -- collapsing an unconfirmed kill into a
    # normal timeout/cap-exceeded result would silently discard the one
    # signal that teardown, not just the command, may have failed.
    SANDBOX_TEARDOWN_UNCONFIRMED = "sandbox_teardown_unconfirmed"

    # Element 3 Subtask B: outcomes of the direct antares-cli subprocess
    # dispatch path in harness.py (antares tool query/sweep --stdin),
    # distinct from every T2a-T2c2 kind above -- those describe outcomes of
    # the internal-schema tool-call pipeline, which this path does not use.
    # Kept as four separate kinds, mirroring the T2c-1/T2c-2 discipline of
    # never folding distinct failure modes into one generic bucket: a caller
    # must be able to tell "the CLI is not installed" apart from "the CLI
    # ran and failed" apart from "the CLI ran, exited 0, but its stdout
    # could not be parsed".
    CLI_EXECUTION_COMPLETE = "cli_execution_complete"
    CLI_BINARY_UNAVAILABLE = "cli_binary_unavailable"
    CLI_EXECUTION_FAILED = "cli_execution_failed"
    CLI_OUTPUT_MALFORMED = "cli_output_malformed"


# Kinds produced by a successful, well-formed tool call.
SUCCESS_KINDS = frozenset(
    {
        TerminalStateKind.PARSED_TERMINAL_CALL,
        TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
        TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
        TerminalStateKind.COMMAND_PLAN_VALID,
        TerminalStateKind.PATH_CONTAINMENT_VALID,
        TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
        TerminalStateKind.CLI_EXECUTION_COMPLETE,
    }
)

# Kinds that are themselves a terminal submission (as opposed to a `terminal`
# command request). Used by the duplicate-submission check.
TERMINAL_SUBMISSION_KINDS = frozenset(
    {
        TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
        TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
    }
)


@dataclass(frozen=True)
class TerminalState:
    """One parsed or policy-validated outcome.

    `argv` is populated for PARSED_TERMINAL_CALL (T2a, as submitted) and for
    COMMAND_PLAN_VALID (T2b, the validated/normalized command plan).
    `candidates` is populated for SUBMITTED_VULNERABLE_FILES (T2a, as
    submitted) and for PATH_CONTAINMENT_VALID (T2b, the subset of candidate
    paths that resolved inside the snapshot). `detail` carries a
    human-readable reason for any non-success kind; it is diagnostic only
    and must never be parsed by callers to distinguish states -- use `kind`.
    `stdout`/`stderr`/`elapsed_seconds`/`exit_code` are populated for T2c-1's
    SANDBOX_EXECUTION_COMPLETE and SANDBOX_COMMAND_TIMED_OUT -- captured as
    produced until process termination, with no size cap applied at this
    layer (output-size limits are a T2c-2 resource-budget concern).
    """

    kind: TerminalStateKind
    argv: tuple[str, ...] = field(default_factory=tuple)
    candidates: tuple[str, ...] = field(default_factory=tuple)
    detail: str = ""
    stdout: str = ""
    stderr: str = ""
    elapsed_seconds: float = 0.0
    exit_code: int | None = None

    @property
    def is_success(self) -> bool:
        return self.kind in SUCCESS_KINDS

    @property
    def is_terminal_submission(self) -> bool:
        return self.kind in TERMINAL_SUBMISSION_KINDS
