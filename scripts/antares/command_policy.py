"""Command allowlist policy for Antares `terminal` tool calls (T2b).

Validates an already-parsed argv (from `tool_call_parser.parse_tool_call`,
T2a) against a fixed read-only executable/option allowlist and canonical
path containment before any sandboxed execution (T2c) is authorized. This
module performs no shell evaluation, no command execution, and no filesystem
mutation -- it only inspects and validates structured argv.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_TERMINAL_STATE_SCRIPT = Path(__file__).with_name("terminal_state.py")
_TERMINAL_STATE_SPEC = importlib.util.spec_from_file_location(
    "antares_terminal_state", _TERMINAL_STATE_SCRIPT
)
if _TERMINAL_STATE_SPEC is None or _TERMINAL_STATE_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_TERMINAL_STATE_SCRIPT}")
_TERMINAL_STATE_MOD = importlib.util.module_from_spec(_TERMINAL_STATE_SPEC)
sys.modules[_TERMINAL_STATE_SPEC.name] = _TERMINAL_STATE_MOD
_TERMINAL_STATE_SPEC.loader.exec_module(_TERMINAL_STATE_MOD)

TerminalState = _TERMINAL_STATE_MOD.TerminalState
TerminalStateKind = _TERMINAL_STATE_MOD.TerminalStateKind

_PATH_CONTAINMENT_SCRIPT = Path(__file__).with_name("path_containment.py")
_PATH_CONTAINMENT_SPEC = importlib.util.spec_from_file_location(
    "antares_path_containment", _PATH_CONTAINMENT_SCRIPT
)
if _PATH_CONTAINMENT_SPEC is None or _PATH_CONTAINMENT_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_PATH_CONTAINMENT_SCRIPT}")
_PATH_CONTAINMENT_MOD = importlib.util.module_from_spec(_PATH_CONTAINMENT_SPEC)
sys.modules[_PATH_CONTAINMENT_SPEC.name] = _PATH_CONTAINMENT_MOD
_PATH_CONTAINMENT_SPEC.loader.exec_module(_PATH_CONTAINMENT_MOD)

resolve_within_snapshot = _PATH_CONTAINMENT_MOD.resolve_within_snapshot

# EC-1: any argv element containing one of these substrings is refused
# outright. Because argv arrives as a pre-split JSON array (never a shell
# string, per T2a), these can never be shell-interpreted -- but the model
# could still submit an argv element that pastes shell syntax as a literal
# string (e.g. it hallucinates a shell one-liner into a single argv slot).
# Rejecting the literal characters closes that path too, on top of the fact
# that this harness never invokes a shell.
_SHELL_METACHARACTERS = ("|", ";", "&", "$", "`", ">", "<", "\n")


def _is_shell_unsafe(token: str) -> bool:
    if any(char in token for char in _SHELL_METACHARACTERS):
        return True
    # Environment-assignment prefix, e.g. "FOO=bar".
    if "=" in token and not token.startswith("-"):
        head = token.split("=", 1)[0]
        if head.isidentifier():
            return True
    return False


# Per-executable option allowlist. `None` for the option-set means "no
# options accepted, operands only" is not the rule here -- every entry below
# lists its own accepted flags explicitly. An option not listed is refused
# even if the executable itself is allowlisted (EC-2), e.g. `find -exec`.
_ALLOWED_OPTIONS: dict[str, frozenset[str]] = {
    "grep": frozenset({"-r", "-n", "-i", "-l", "-c", "-w", "-E", "-F", "-v"}),
    "find": frozenset({"-name", "-type", "-maxdepth", "-iname"}),
    "cat": frozenset({"-n"}),
    "ls": frozenset({"-l", "-a", "-la", "-al", "-R", "-1"}),
    "head": frozenset({"-n"}),
    "tail": frozenset({"-n"}),
    "wc": frozenset({"-l", "-w", "-c"}),
}

# Options that take a following value argument (that value is not itself
# checked against the option allowlist, but is still checked for shell
# metacharacters and, where it looks like a path, for containment).
_OPTIONS_WITH_VALUE: frozenset[str] = frozenset(
    {"-n", "-name", "-iname", "-maxdepth", "-type"}
)

ALLOWED_EXECUTABLES: frozenset[str] = frozenset(_ALLOWED_OPTIONS)


def _rejected_executable(executable: str) -> TerminalState:
    return TerminalState(
        kind=TerminalStateKind.COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED,
        detail=f"Executable {executable!r} is not on the read-only allowlist.",
    )


def _rejected_option(executable: str, option: str) -> TerminalState:
    return TerminalState(
        kind=TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED,
        detail=f"Option {option!r} is not allowed for executable {executable!r}.",
    )


def _rejected_shell_syntax(token: str) -> TerminalState:
    return TerminalState(
        kind=TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX,
        detail=f"Argv element {token!r} contains disallowed shell syntax.",
    )


def validate_command(
    argv: tuple[str, ...], snapshot_root: Path
) -> TerminalState:
    """Validate one argv sequence into a command plan or a refusal.

    HP-1: an allowlisted executable with only approved options and
    in-snapshot path operands becomes COMMAND_PLAN_VALID, carrying the
    original argv unchanged (this layer validates; it does not rewrite).
    EC-1: shell metacharacters or an environment-assignment-shaped token in
    any argv element is refused before the executable/option check runs.
    EC-2: an executable outside the allowlist, or an option outside that
    executable's own allowed set (e.g. `find -exec`), is refused.
    EC-3: a path-shaped operand that escapes the snapshot after canonical
    resolution is refused, deferring to `path_containment.py` for the
    resolution logic itself.
    """
    if not argv:
        return _rejected_shell_syntax("")

    for token in argv:
        if _is_shell_unsafe(token):
            return _rejected_shell_syntax(token)

    executable = argv[0]
    if executable not in _ALLOWED_OPTIONS:
        return _rejected_executable(executable)

    allowed_options = _ALLOWED_OPTIONS[executable]
    args = list(argv[1:])
    i = 0
    while i < len(args):
        token = args[i]
        if token.startswith("-"):
            if token not in allowed_options:
                return _rejected_option(executable, token)
            if token in _OPTIONS_WITH_VALUE:
                if i + 1 >= len(args):
                    return TerminalState(
                        kind=TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED,
                        detail=f"Option {token!r} for {executable!r} requires a value.",
                    )
                i += 2
                continue
            i += 1
            continue
        # Non-flag operand: treated as a path operand and must resolve
        # inside the snapshot (EC-3). find's `-name`/`-iname` patterns are
        # consumed as option values above, never reaching this branch.
        if resolve_within_snapshot(token, snapshot_root) is None:
            return TerminalState(
                kind=TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE,
                detail=(
                    f"Operand {token!r} escapes the read-only snapshot root "
                    f"{snapshot_root!s} after canonical resolution."
                ),
            )
        i += 1

    return TerminalState(kind=TerminalStateKind.COMMAND_PLAN_VALID, argv=tuple(argv))
