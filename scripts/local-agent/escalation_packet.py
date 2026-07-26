#!/usr/bin/env python3
"""Builds the ADR-036 §7 escalation packet markdown from runner artifacts."""

import argparse
import json
import os
import sys

MISSING = "MISSING"


def load_json(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def load_card(card_path):
    data = load_json(card_path)
    return {
        "task_id": data["task_id"],
        "spec": data["spec"],
        "plan": data.get("plan"),
        "allowed_paths": data.get("allowed_paths", []),
        "acceptance_tests": data.get("acceptance_tests", []),
    }


def validate_json_object_shape(value):
    """Shared by both packet builders (plan D9): a successfully-parsed
    runner/transcript artifact can still be a JSON list or scalar, which
    crashes downstream .get()/subscript calls. Returns (value, None) if value
    is a dict, else (None, failure_reason) -- never raises, so a caller can
    always render something fail-visible instead of an AttributeError.
    Scoped to shape only, never to decode/read failures, which each builder
    handles directly at its own read site."""
    if isinstance(value, dict):
        return value, None
    return None, f"expected a JSON object, got {type(value).__name__}"


def read_text_file(path):
    if not path:
        return None
    with open(path, encoding="utf-8") as f:
        return f.read()


def read_optional_text_file(path, *, label):
    """Read an optional text file, never letting a read failure destroy the
    caller's bundle. Returns a (text, missing_text) pair: on success, text is
    the file's content and missing_text is None; otherwise text is None and
    missing_text is the exact literal to render, distinguishing "no path
    given" from "path not found" from "path exists but is unreadable"
    (plan D2/D3). os.path.isfile cannot stand in for a read-success check --
    it returns True for a file a caller cannot actually read, and checking
    then opening would leave a race -- so failure is caught around the read
    itself, not predicted beforehand."""
    if not path:
        return None, MISSING
    if not os.path.isfile(path):
        return None, f"{MISSING} ({label} not found: {path})"
    try:
        with open(path, encoding="utf-8") as f:
            return f.read(), None
    except (OSError, UnicodeDecodeError) as exc:
        return None, f"{MISSING} ({label} unreadable: {path}: {exc})"


def render_task_spec_section(card, rri_table_text):
    rri_block = rri_table_text if rri_table_text else MISSING
    return (
        f"Task ID: `{card['task_id']}`\n\n"
        f"Spec:\n\n{card['spec']}\n\n"
        f"RRI table:\n\n{rri_block}"
    )


def render_plan_section(card):
    return card["plan"] if card.get("plan") else MISSING


def render_allowed_paths_section(card):
    paths = card.get("allowed_paths") or []
    if not paths:
        return MISSING
    return "\n".join(f"- `{p}`" for p in paths)


def render_diff_section(diff_text, diff_missing=None):
    if diff_missing is not None:
        return diff_missing
    if not diff_text:
        return MISSING
    return f"```diff\n{diff_text}\n```"


def extract_command_events(transcript):
    return [
        e["result"]
        for e in transcript
        if e.get("event") == "tool_result" and e.get("result", {}).get("tool") == "run_command"
    ]


def render_commands_section(transcript):
    commands = extract_command_events(transcript)
    if not commands:
        return MISSING
    parts = []
    for i, cmd in enumerate(commands, start=1):
        parts.append(
            f"### Command {i}\n\n"
            f"argv: `{cmd.get('argv')}`\n\n"
            f"returncode: `{cmd.get('returncode')}`\n\n"
            f"stdout:\n```\n{cmd.get('stdout', '')}\n```\n\n"
            f"stderr:\n```\n{cmd.get('stderr', '')}\n```"
        )
    return "\n\n".join(parts)


def extract_test_events(transcript):
    return [e for e in transcript if e.get("event") == "test_result"]


def render_test_results_section(transcript):
    test_events = extract_test_events(transcript)
    if not test_events:
        return MISSING
    parts = []
    for i, event in enumerate(test_events, start=1):
        result = event.get("result", {})
        status = "PASSED" if result.get("passed") else "FAILED"
        parts.append(
            f"### Attempt {i}: {status}\n\n"
            f"output:\n```\n{result.get('output', '')}\n```"
        )
    return "\n\n".join(parts)


def describe_event(event):
    kind = event.get("event")
    if kind is None:
        return None
    if kind == "tool_result":
        result = event.get("result", {})
        tool = result.get("tool")
        if tool == "run_command":
            return f"ran command `{result.get('argv')}` (returncode {result.get('returncode')})"
        if tool == "write_file":
            return f"wrote file `{result.get('path')}`"
        if tool == "finish":
            return "issued finish"
        return f"tool result: {tool}"
    if kind == "malformed_tool_call":
        return f"malformed tool call: {event.get('error')}"
    if kind == "boundary_violation":
        return f"boundary violation: {event.get('error')}"
    return f"event: {kind}"


def render_per_attempt_summaries_section(result):
    transcript = result.get("transcript", [])
    test_events_idx = [
        i for i, e in enumerate(transcript) if e.get("event") == "test_result"
    ]

    summaries = []
    if test_events_idx:
        start = 0
        for attempt_num, idx in enumerate(test_events_idx, start=1):
            preceding = transcript[start:idx]
            described = [d for d in (describe_event(e) for e in preceding) if d is not None]
            actions = "; ".join(described) or "no prior actions"
            test_result = transcript[idx].get("result", {})
            status = "passed" if test_result.get("passed") else "failed"
            summaries.append(
                f"- Attempt {attempt_num}: {actions}; tests {status}."
            )
            start = idx + 1

    status = result.get("status")
    terminal_note = ""
    if status in ("aborted", "boundary_violation", "transport_error", "transcript_shape_invalid"):
        terminal_events = [
            e for e in transcript
            if e.get("event") in ("boundary_violation", "transport_error", "malformed_tool_call")
        ]
        error_msg = None
        if terminal_events:
            error_msg = terminal_events[-1].get("error")
        elif result.get("reason"):
            error_msg = result.get("reason")
        if error_msg:
            terminal_note = f" ({error_msg})"

    summaries.append(f"- Final status: `{status}`{terminal_note}.")
    return "\n".join(summaries) if summaries else MISSING


def build_packet(card, transcript_data, diff_text, rri_table_text, diff_missing=None):
    sections = [
        ("1. Task spec + RRI table", render_task_spec_section(card, rri_table_text)),
        ("2. Plan", render_plan_section(card)),
        ("3. Allowed paths", render_allowed_paths_section(card)),
        ("4. Full diff", render_diff_section(diff_text, diff_missing)),
        ("5. Commands executed with output", render_commands_section(transcript_data.get("transcript", []))),
        ("6. Test results", render_test_results_section(transcript_data.get("transcript", []))),
        ("7. Per-attempt summaries", render_per_attempt_summaries_section(transcript_data)),
    ]

    parts = [f"# Escalation packet: `{card['task_id']}`\n"]
    for title, body in sections:
        parts.append(f"## {title}\n\n{body}\n")
    return "\n".join(parts)


def parse_args(argv=None):
    parser = argparse.ArgumentParser(
        description="Build the ADR-036 §7 escalation packet from runner artifacts.",
    )
    parser.add_argument("--transcript", required=True, help="Path to run_local_task.py's --out JSON artifact.")
    parser.add_argument("--card", required=True, help="Path to the original task card JSON.")
    parser.add_argument("--out", required=True, help="Path to write the markdown packet.")
    parser.add_argument("--diff-file", default=None, help="Path to a precomputed unified diff text file.")
    parser.add_argument(
        "--rri-table",
        default=None,
        help="Path to a markdown file containing the RRI table, or the table text itself.",
    )
    return parser.parse_args(argv)


def resolve_rri_table(rri_table_arg):
    """Resolve --rri-table, which is documented as either a path to a markdown
    file or the table text itself. The path-vs-text ambiguity (a nonexistent
    path, or a directory, silently becoming literal text) is unchanged and
    intentionally out of scope -- filed separately in
    docs/tasks/rri-table-path-text-ambiguity.md -- because os.path.isfile is
    False for both and neither can raise. Only the read-failure case on a
    path os.path.isfile confirms is a regular file is guarded here."""
    if not rri_table_arg:
        return None
    if not os.path.isfile(rri_table_arg):
        return rri_table_arg
    text, missing_text = read_optional_text_file(rri_table_arg, label="RRI table file")
    return text if missing_text is None else missing_text


def main(argv=None):
    args = parse_args(argv)
    card = load_card(args.card)
    raw_transcript_data = load_json(args.transcript)
    transcript_data, shape_failure_reason = validate_json_object_shape(raw_transcript_data)
    if shape_failure_reason is not None:
        transcript_data = {"status": "transcript_shape_invalid", "reason": shape_failure_reason}
    diff_text, diff_missing = read_optional_text_file(args.diff_file, label="diff file")
    rri_table_text = resolve_rri_table(args.rri_table)

    packet = build_packet(card, transcript_data, diff_text, rri_table_text, diff_missing=diff_missing)

    with open(args.out, "w", encoding="utf-8") as f:
        f.write(packet)

    return 0


if __name__ == "__main__":
    sys.exit(main())
