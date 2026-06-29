#!/usr/bin/env python3
"""Consolidate and display all findings from a Gemma code-review result JSON.

Reads the aggregate result file produced by gemma-code-review.py and collects
findings from BOTH top-level `findings[]` and every bucket inside `reconciliation`
(consensus, pass_specific, location_inconsistent, severity_inconsistent,
likely_false_positive). Duplicate findings are merged into one developer-review
entry with all source buckets preserved. Prints a human-readable table and exits
non-zero when any finding is present, forcing the primary agent to read and
disposition each one before proceeding.

Exit codes:
  0  no findings, or all findings are minor/nit severity
  1  one or more blocking or major findings (agent must disposition)
  2  result file missing or unreadable
"""

import argparse
import json
import os
import sys


_RECONCILIATION_BUCKETS = [
    "consensus",
    "pass_specific",
    "location_inconsistent",
    "severity_inconsistent",
    "likely_false_positive",
]

_SEVERITY_ORDER = ["blocking", "major", "minor", "nit", "unknown"]


def parse_args():
    parser = argparse.ArgumentParser(
        description="Consolidate Gemma review findings and fail if any exist.",
    )
    parser.add_argument(
        "result",
        nargs="?",
        default=os.environ.get(
            "DUBBRIDGE_REVIEW_RESULT", "/tmp/dubbridge-gemma-review.json"
        ),
        help=(
            "Path to the aggregate review JSON "
            "(default: DUBBRIDGE_REVIEW_RESULT or /tmp/dubbridge-gemma-review.json)."
        ),
    )
    parser.add_argument(
        "--format",
        choices=["text", "json"],
        default="text",
        dest="fmt",
        help="Output format (default: text).",
    )
    parser.add_argument(
        "--no-fail",
        action="store_true",
        help="Print findings but always exit 0 (for reporting-only use).",
    )
    return parser.parse_args()


def _sev_rank(sev):
    try:
        return _SEVERITY_ORDER.index(sev)
    except ValueError:
        return len(_SEVERITY_ORDER)


def _finding_key(finding):
    return (
        finding.get("path") or finding.get("location") or "",
        finding.get("line"),
        finding.get("severity", "unknown"),
        finding.get("detail") or finding.get("summary") or "",
        finding.get("suggestion") or "",
    )


def collect_findings(data):
    """Return a list of (bucket, finding_dict) pairs, sorted by severity."""
    consolidated = {}

    # Top-level findings array (single-pass or pre-reconciliation overflow)
    for f in data.get("findings") or []:
        _add_finding(consolidated, "findings", f)

    # Reconciliation buckets
    reconciliation = data.get("reconciliation") or {}
    for bucket in _RECONCILIATION_BUCKETS:
        for f in reconciliation.get(bucket) or []:
            _add_finding(consolidated, bucket, f)

    # Stable sort: blocking first, nit last
    collected = []
    for item in consolidated.values():
        buckets = item["source_buckets"]
        finding = {**item["finding"], "source_buckets": buckets}
        collected.append((", ".join(buckets), finding))
    collected.sort(key=lambda pair: _sev_rank(pair[1].get("severity", "unknown")))
    return collected


def _add_finding(consolidated, bucket, finding):
    key = _finding_key(finding)
    if key not in consolidated:
        consolidated[key] = {
            "finding": dict(finding),
            "source_buckets": [],
        }
    buckets = consolidated[key]["source_buckets"]
    if bucket not in buckets:
        buckets.append(bucket)


def format_text(data, collected):
    lines = []
    passes_run = data.get("passes_run", "?")
    passes_ok = data.get("passes_succeeded", "?")
    status = data.get("status", "?")
    summary = data.get("summary", "")

    lines.append(f"passes compiled: {passes_ok}/{passes_run}  status: {status}")
    if summary:
        lines.append(f"summary: {summary}")
    lines.append("")

    if not collected:
        lines.append("findings: none")
        return "\n".join(lines)

    lines.append(f"findings: {len(collected)}")
    lines.append("")

    for bucket, f in collected:
        sev = f.get("severity", "unknown")
        path = f.get("path") or f.get("location") or "?"
        line_no = f.get("line")
        loc = f"{path}:{line_no}" if line_no else path
        detail = f.get("detail") or f.get("summary") or ""
        suggestion = f.get("suggestion") or ""

        lines.append(f"  [{bucket}] severity={sev}  loc={loc}")
        if detail:
            # Wrap long detail lines at 100 chars
            for chunk in _wrap(detail, 96):
                lines.append(f"    detail: {chunk}")
        if suggestion:
            for chunk in _wrap(suggestion, 96):
                lines.append(f"    suggestion: {chunk}")
        lines.append("")

    return "\n".join(lines)


def _wrap(text, width):
    """Very simple word-wrap that keeps URLs intact."""
    words = text.split()
    line = ""
    result = []
    for word in words:
        if line and len(line) + 1 + len(word) > width:
            result.append(line)
            line = word
        else:
            line = (line + " " + word).lstrip()
    if line:
        result.append(line)
    return result or [""]


def format_json_output(data, collected):
    output = {
        "passes_run": data.get("passes_run"),
        "passes_succeeded": data.get("passes_succeeded"),
        "status": data.get("status"),
        "total_findings": len(collected),
        "findings": [
            {"bucket": bucket, **finding} for bucket, finding in collected
        ],
    }
    return json.dumps(output, indent=2)


def main():
    args = parse_args()

    try:
        with open(args.result, encoding="utf-8") as fh:
            data = json.load(fh)
    except FileNotFoundError:
        print(f"error: result file not found: {args.result}", file=sys.stderr)
        return 2
    except json.JSONDecodeError as exc:
        print(f"error: invalid JSON in {args.result}: {exc}", file=sys.stderr)
        return 2

    collected = collect_findings(data)

    if args.fmt == "json":
        print(format_json_output(data, collected))
    else:
        print(format_text(data, collected))

    has_blocking = any(
        f.get("severity") in ("blocking", "major") for _, f in collected
    )
    if has_blocking and not args.no_fail:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
