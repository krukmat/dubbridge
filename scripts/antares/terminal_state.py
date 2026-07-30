"""Fail-closed terminal-state contract for Antares tool-call parsing (T2a).

Every outcome the parser can produce is one of these explicit, machine-
distinguishable states. No two failure modes collapse into a single generic
"error" bucket: downstream policy (T2b), sandbox (T2c), and artifact (T2d)
layers need to tell them apart.
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


# Kinds produced by a successful, well-formed tool call.
SUCCESS_KINDS = frozenset(
    {
        TerminalStateKind.PARSED_TERMINAL_CALL,
        TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
        TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
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
    """One parsed outcome.

    `argv` is populated only for PARSED_TERMINAL_CALL. `candidates` is
    populated only for SUBMITTED_VULNERABLE_FILES. `detail` carries a
    human-readable reason for any non-success kind; it is diagnostic only
    and must never be parsed by callers to distinguish states -- use `kind`.
    """

    kind: TerminalStateKind
    argv: tuple[str, ...] = field(default_factory=tuple)
    candidates: tuple[str, ...] = field(default_factory=tuple)
    detail: str = ""

    @property
    def is_success(self) -> bool:
        return self.kind in SUCCESS_KINDS

    @property
    def is_terminal_submission(self) -> bool:
        return self.kind in TERMINAL_SUBMISSION_KINDS
