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
    }


def read_text_file(path):
    if not path:
        return None
    with open(path, encoding="utf-8") as f:
        return f.read()


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


def render_diff_section(diff_text):
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
    if status in ("aborted", "boundary_violation", "transport_error"):
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


def build_packet(card, transcript_data, diff_text, rri_table_text):
    sections = [
        ("1. Task spec + RRI table", render_task_spec_section(card, rri_table_text)),
        ("2. Plan", render_plan_section(card)),
        ("3. Allowed paths", render_allowed_paths_section(card)),
        ("4. Full diff", render_diff_section(diff_text)),
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
    if not rri_table_arg:
        return None
    if os.path.isfile(rri_table_arg):
        return read_text_file(rri_table_arg)
    return rri_table_arg


def main(argv=None):
    args = parse_args(argv)
    card = load_card(args.card)
    transcript_data = load_json(args.transcript)
    diff_text = read_text_file(args.diff_file)
    rri_table_text = resolve_rri_table(args.rri_table)

    packet = build_packet(card, transcript_data, diff_text, rri_table_text)

    with open(args.out, "w", encoding="utf-8") as f:
        f.write(packet)

    return 0


if __name__ == "__main__":
    sys.exit(main())
