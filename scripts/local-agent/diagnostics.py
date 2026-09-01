from __future__ import annotations

import re

DEFAULT_MAX_LINES = 80
DEFAULT_MAX_CHARS = 6000
CONTEXT_RADIUS = 1

_SIGNAL_PATTERNS = (
    re.compile(r"\berror(?:\[[A-Z0-9]+\])?:", re.IGNORECASE),
    re.compile(r"\bfailed\b", re.IGNORECASE),
    re.compile(r"\bfailures?:\b", re.IGNORECASE),
    re.compile(r"\bpanicked at\b", re.IGNORECASE),
    re.compile(r"\bassert(?:ion)?\b", re.IGNORECASE),
    re.compile(r"\bexpected\b", re.IGNORECASE),
    re.compile(r"\bactual\b", re.IGNORECASE),
    re.compile(r"\btest\s+\S+\s+\.\.\.\s+FAILED\b"),
    re.compile(r"-->\s+[^\s:]+:\d+(?::\d+)?"),
    re.compile(r"\b[^\s:]+\.(?:rs|py|ts|tsx|js|jsx):\d+(?::\d+)?\b"),
)


def _is_signal(line):
    return any(pattern.search(line) for pattern in _SIGNAL_PATTERNS)


def _bounded_signal_lines(text, *, max_lines=DEFAULT_MAX_LINES, max_chars=DEFAULT_MAX_CHARS):
    lines = (text or "").splitlines()
    selected = set()
    for index, line in enumerate(lines):
        if not _is_signal(line):
            continue
        start = max(0, index - CONTEXT_RADIUS)
        end = min(len(lines), index + CONTEXT_RADIUS + 1)
        selected.update(range(start, end))

    if selected:
        ordered = [lines[index] for index in sorted(selected)]
    else:
        # Preserve deterministic bounded evidence even for unusual tools whose
        # failure format does not match the known signal patterns.
        head = lines[: min(12, len(lines))]
        tail = lines[-8:] if len(lines) > len(head) else []
        ordered = head + (["..."] if tail else []) + tail

    output = []
    chars = 0
    for line in ordered:
        if len(output) >= max_lines:
            break
        projected = chars + len(line) + 1
        if projected > max_chars:
            break
        output.append(line)
        chars = projected
    if len(output) < len(ordered):
        output.append("...[diagnostic output bounded]")
    return "\n".join(output)


def summarize_failure(result, kind, *, max_lines=DEFAULT_MAX_LINES, max_chars=DEFAULT_MAX_CHARS):
    """Create deterministic model-visible repair diagnostics from a runner result."""
    if not isinstance(result, dict):
        return f"KIND: {kind}\nDETAIL: unavailable"

    lines = [f"KIND: {kind}", f"PASSED: {bool(result.get('passed'))}"]
    commands = result.get("commands")
    if isinstance(commands, list):
        failing = [item for item in commands if isinstance(item, dict) and not item.get("ok")]
        selected = failing[:1] or [item for item in commands[:1] if isinstance(item, dict)]
        for item in selected:
            argv = item.get("argv")
            if isinstance(argv, list):
                lines.append("COMMAND: " + " ".join(str(part) for part in argv))
            lines.append(f"RETURN_CODE: {item.get('returncode')}")

    compact = _bounded_signal_lines(
        result.get("output", ""), max_lines=max_lines, max_chars=max_chars
    )
    if compact:
        lines.append("DIAGNOSTIC:\n" + compact)
    return "\n".join(lines)


def repair_hints(file_tools, diagnostic_summary):
    edited_paths = list(getattr(file_tools, "edited_paths", ()) or ())
    return {
        "edited_paths": edited_paths,
        "diagnostic_summary": diagnostic_summary or "",
    }
