"""Tool-call parser for the Antares agentic loop (T2a).

Parses one raw JSON payload at a time into a `TerminalState`. This module
performs no command execution, no filesystem access, no network access, and
no subprocess invocation -- it only classifies structured data. Command
allowlisting, path containment, and sandboxed execution are the
responsibility of later, separately decomposed tasks (T2b, T2c).

Antares documents exactly three tool-call actions:
- `terminal`: run a read-only shell command, carried as argv (a JSON array
  of strings), never a shell string.
- `submit_vulnerable_files`: terminal submission naming candidate file
  paths (a JSON array of non-empty strings).
- `submit_no_vulnerability_found`: terminal submission with no candidates,
  an explicit negative result.

Wire-format note (T2a Reflection pass 2, 2026-07-29): T1's runtime preflight
never reached a live Antares inference call, so no observed Antares
transcript exists to pin an exact wire shape. `parse_tool_call` therefore
consumes a normalized internal schema --
`{"tool": <name>, "payload": {...}}` -- rather than any specific model or
API's native function-calling envelope (e.g. an OpenAI/Ollama-style
`{"function": {"name": ..., "arguments": {...}}}` frame). Whatever harness
invokes Antares for real (T2c) is responsible for translating the model's
actual `<tool_call>` output into this internal schema before calling
`parse_tool_call`; this module intentionally does not assume or hard-code
either wire format, per the model-card-only constraint in the T2a task
definition.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

# Loaded by absolute file path, not a bare `import terminal_state`, so this
# module resolves correctly regardless of the caller's working directory or
# sys.path (mirrors scripts/local-architect/run_analysis_test.py's pattern).
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

TOOL_TERMINAL = "terminal"
TOOL_SUBMIT_VULNERABLE_FILES = "submit_vulnerable_files"
TOOL_SUBMIT_NO_VULNERABILITY_FOUND = "submit_no_vulnerability_found"
SUPPORTED_TOOL_NAMES = frozenset(
    {TOOL_TERMINAL, TOOL_SUBMIT_VULNERABLE_FILES, TOOL_SUBMIT_NO_VULNERABILITY_FOUND}
)


def _malformed(detail: str) -> TerminalState:
    return TerminalState(kind=TerminalStateKind.MALFORMED_TOOL_CALL, detail=detail)


def _malformed_submit(detail: str) -> TerminalState:
    return TerminalState(kind=TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD, detail=detail)


def _is_string_list(value: object) -> bool:
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def _parse_terminal(payload: dict) -> TerminalState:
    argv = payload.get("argv")
    if argv is None:
        return _malformed("'terminal' tool call is missing the required 'argv' field.")
    if not isinstance(argv, list):
        return _malformed("'terminal' tool call 'argv' must be a JSON array.")
    if not _is_string_list(argv):
        # EC-4: a type-mismatched element (e.g. an integer) is rejected as
        # MALFORMED_TOOL_CALL, never coerced to a string.
        return _malformed("'terminal' tool call 'argv' must contain only strings.")
    return TerminalState(kind=TerminalStateKind.PARSED_TERMINAL_CALL, argv=tuple(argv))


def _parse_submit_vulnerable_files(payload: dict) -> TerminalState:
    candidates = payload.get("candidates")
    if candidates is None:
        return _malformed_submit(
            "'submit_vulnerable_files' payload is missing the required 'candidates' field."
        )
    if not isinstance(candidates, list):
        return _malformed_submit("'submit_vulnerable_files' 'candidates' must be a JSON array.")
    if not candidates:
        return _malformed_submit("'submit_vulnerable_files' 'candidates' must not be empty.")
    if not _is_string_list(candidates):
        # EC-4: a non-string candidate path is rejected, never coerced.
        return _malformed("'submit_vulnerable_files' 'candidates' must contain only strings.")
    if not all(item.strip() for item in candidates):
        return _malformed_submit("'submit_vulnerable_files' 'candidates' must not contain blank paths.")
    # Paths are preserved as untrusted strings only -- containment validation
    # against the filesystem is T2b's responsibility, not this parser's.
    return TerminalState(
        kind=TerminalStateKind.SUBMITTED_VULNERABLE_FILES, candidates=tuple(candidates)
    )


def parse_tool_call(raw_json: str) -> TerminalState:
    """Parse one raw tool-call JSON message into a `TerminalState`.

    Called once per tool-call message by the calling harness; this function
    holds no session state across calls (see `check_duplicate_submission`
    for cross-call duplicate detection).
    """
    try:
        message = json.loads(raw_json)
    except (json.JSONDecodeError, TypeError, ValueError) as exc:
        # EC-1: malformed JSON syntax records MALFORMED_TOOL_CALL with no
        # partial execution attempt.
        return _malformed(f"Tool-call payload is not valid JSON: {exc}")

    if not isinstance(message, dict):
        return _malformed("Tool-call payload must be a JSON object.")

    tool_name = message.get("tool")
    if not isinstance(tool_name, str):
        return _malformed("Tool-call payload is missing a string 'tool' field.")

    if tool_name not in SUPPORTED_TOOL_NAMES:
        # EC-2: an unsupported tool name is rejected into its own distinct
        # state before any policy evaluation.
        return TerminalState(
            kind=TerminalStateKind.UNSUPPORTED_TOOL_NAME,
            detail=f"Unsupported tool name: {tool_name!r}.",
        )

    payload = message.get("payload", {})
    if not isinstance(payload, dict):
        return _malformed_submit(f"{tool_name!r} payload must be a JSON object.")

    if tool_name == TOOL_TERMINAL:
        return _parse_terminal(payload)
    if tool_name == TOOL_SUBMIT_VULNERABLE_FILES:
        return _parse_submit_vulnerable_files(payload)
    # tool_name == TOOL_SUBMIT_NO_VULNERABILITY_FOUND
    # HP-2: an explicit negative terminal result, never an empty candidate
    # list and never implicit success.
    return TerminalState(kind=TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND)


def check_duplicate_submission(
    first: TerminalState, second: TerminalState
) -> TerminalState:
    """Check two already-parsed results for a duplicate terminal submission.

    EC-3: if both `first` and `second` are terminal submissions
    (submit_vulnerable_files or submit_no_vulnerability_found), the session
    must fail closed rather than silently preferring either payload. The
    parser itself is stateless (one call = one message); a caller that
    tracks a session's parsed results uses this function to detect the
    duplicate instead of the parser guessing at session state.
    """
    if first.is_terminal_submission and second.is_terminal_submission:
        return TerminalState(
            kind=TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION,
            detail="A terminal submission was already recorded in this session.",
        )
    return second
