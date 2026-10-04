#!/usr/bin/env python3
"""Ground-truth extraction, capture, and evaluation for review-decision shadow runs."""
from __future__ import annotations

import argparse
import json
import math
import os
from pathlib import Path
from typing import Any

SEVERITY_ORDER = {"none": 0, "nit": 1, "minor": 2, "major": 3, "blocking": 4}
PASS_SENTINELS = {"", "none", "n/a", "pass"}


def _nested_payload(data: dict[str, Any]) -> dict[str, Any] | None:
    message = data.get("message")
    content = message.get("content") if isinstance(message, dict) else None
    if not isinstance(content, str):
        return None
    try:
        parsed = json.loads(content)
    except json.JSONDecodeError:
        return None
    return parsed if isinstance(parsed, dict) else None


def _verdict(data: dict[str, Any]) -> str | None:
    for key in ("verdict", "status"):
        value = data.get(key)
        if isinstance(value, str) and value.strip().lower() in {"pass", "findings", "blocked"}:
            return value.strip().lower()
    return None


def _max_severity(findings: Any) -> str:
    best = "none"
    if not isinstance(findings, list):
        return best
    for finding in findings:
        severity = None
        if isinstance(finding, dict) and isinstance(finding.get("severity"), str):
            severity = finding["severity"].strip().lower()
        elif isinstance(finding, str):
            if finding.strip().lower() in PASS_SENTINELS:
                continue
            severity = finding.split("|", 1)[0].strip().lower()
        if severity in SEVERITY_ORDER and SEVERITY_ORDER[severity] > SEVERITY_ORDER[best]:
            best = severity
    return best


def extract_ground_truth(data: dict[str, Any], source_path: str) -> dict[str, Any] | None:
    payload = data
    verdict = _verdict(payload)
    if verdict is None:
        nested = _nested_payload(data)
        if nested is not None:
            payload, verdict = nested, _verdict(nested)
    if verdict is None:
        return None
    severity = _max_severity(payload.get("findings"))
    if verdict == "findings" and severity == "none":
        severity = "major"  # conservative when findings are untyped
    if verdict == "blocked":
        severity = "blocking"
    return {
        "case_id": source_path.replace(os.sep, "/"),
        "source_path": source_path.replace(os.sep, "/"),
        "ground_truth": {
            "verdict": verdict,
            "max_severity": severity,
            "critical": severity in {"blocking", "major"},
            "phase": data.get("phase") if isinstance(data.get("phase"), str) else None,
            "reviewer": data.get("reviewer") if isinstance(data.get("reviewer"), str) else None,
            "model": data.get("model") if isinstance(data.get("model"), str) else None,
        },
    }


def collect_ground_truth(root: str, limit: int | None = None) -> list[dict[str, Any]]:
    rows = []
    for path in sorted(Path(root).rglob("*.json")):
        if "review" not in path.name.lower():
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(data, dict):
            continue
        row = extract_ground_truth(data, str(path))
        if row:
            rows.append(row)
            if limit is not None and len(rows) >= limit:
                break
    return rows


def build_capture_row(request: dict[str, Any], normalized: dict[str, Any], review: dict[str, Any], review_path: str, expected_review: str | None = None) -> dict[str, Any]:
    state = request.get("state")
    if not isinstance(state, dict) or not isinstance(state.get("case_id"), str):
        raise ValueError("request.state.case_id missing")
    case_id = state["case_id"]
    if normalized.get("case_id") not in {None, case_id}:
        raise ValueError("normalized decision case_id mismatch")
    truth = extract_ground_truth(review, review_path)
    if truth is None:
        raise ValueError("review artifact has no supported verdict/status")
    ground = dict(truth["ground_truth"])
    if expected_review:
        if expected_review not in {"none", "local", "advanced", "human"}:
            raise ValueError("invalid expected_review")
        ground["expected_review"] = expected_review
    if not isinstance(normalized.get("decision"), dict) or not isinstance(normalized.get("fast_path"), dict):
        raise ValueError("normalized artifact missing decision/fast_path")
    row = {
        "case_id": case_id,
        "request": {k: state.get(k) for k in ("task_id", "phase", "rri", "content_sha256", "packet_truncated")},
        "ground_truth": ground,
        "decision": normalized["decision"],
        "fast_path": normalized["fast_path"],
    }
    if isinstance(normalized.get("latency_ms"), (int, float)):
        row["latency_ms"] = float(normalized["latency_ms"])
    return row


def _choice(row: dict[str, Any], name: str) -> str | None:
    try:
        value = row["decision"]["answers"][name]["choice"]
        return value if isinstance(value, str) else None
    except (KeyError, TypeError):
        return None


def _percentile(values: list[float], p: float) -> float | None:
    if not values:
        return None
    values = sorted(values)
    if len(values) == 1:
        return values[0]
    pos = (len(values) - 1) * p
    lo, hi = math.floor(pos), math.ceil(pos)
    return values[lo] if lo == hi else values[lo] * (hi - pos) + values[hi] * (pos - lo)


def evaluate(rows: list[dict[str, Any]]) -> dict[str, Any]:
    usable = [r for r in rows if isinstance(r.get("decision"), dict)]
    critical = [r for r in usable if r.get("ground_truth", {}).get("critical") is True]
    unsafe = [r for r in critical if r.get("fast_path", {}).get("eligible") is True]
    detected = [r for r in critical if _choice(r, "risk") in {"high", "critical"} or _choice(r, "suggested_review") in {"advanced", "human"}]
    labelled = [r for r in usable if isinstance(r.get("ground_truth", {}).get("expected_review"), str)]
    agreement = [r for r in labelled if _choice(r, "suggested_review") == r["ground_truth"]["expected_review"]]
    fast = [r for r in usable if r.get("fast_path", {}).get("eligible") is True]
    latencies = [float(r["latency_ms"]) for r in usable if isinstance(r.get("latency_ms"), (int, float)) and not isinstance(r.get("latency_ms"), bool)]
    recall = len(detected) / len(critical) if critical else None
    result = {
        "cases": len(rows), "usable_cases": len(usable), "critical_cases": len(critical),
        "critical_false_negatives": len(unsafe),
        "unsafe_fast_path_case_ids": [r.get("case_id") for r in unsafe],
        "critical_escalation_recall": recall,
        "review_agreement_rate": (len(agreement) / len(labelled)) if labelled else None,
        "agreement_cases": len(labelled), "fast_path_candidates": len(fast),
        "fast_path_rate": (len(fast) / len(usable)) if usable else None,
        "latency_ms_p50": _percentile(latencies, .50), "latency_ms_p95": _percentile(latencies, .95),
    }
    result["promotion_gate"] = {
        "critical_false_negatives_zero": len(unsafe) == 0,
        "critical_escalation_recall_ge_0_95": recall is not None and recall >= .95,
        "ready_for_policy_review": bool(critical) and len(unsafe) == 0 and recall is not None and recall >= .95,
    }
    return result


def _read(path: str) -> dict[str, Any]:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"{path}: expected object")
    return data


def _read_jsonl(path: str) -> list[dict[str, Any]]:
    rows = []
    for n, line in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
        if line.strip():
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError(f"line {n}: expected object")
            rows.append(value)
    return rows


def _write_json(data: dict[str, Any], path: str) -> None:
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _write_jsonl(rows: list[dict[str, Any]], path: str, append: bool = False) -> None:
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    mode = "a" if append else "w"
    with open(path, mode, encoding="utf-8") as stream:
        for row in rows:
            stream.write(json.dumps(row, sort_keys=True) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="DubBridge review-decision shadow tooling")
    sub = parser.add_subparsers(dest="command", required=True)
    gt = sub.add_parser("ground-truth"); gt.add_argument("--root", default="docs/audit"); gt.add_argument("--output", required=True); gt.add_argument("--limit", type=int)
    cap = sub.add_parser("capture"); cap.add_argument("--request", required=True); cap.add_argument("--normalized", required=True); cap.add_argument("--review-artifact", required=True); cap.add_argument("--output", required=True); cap.add_argument("--expected-review", choices=["none", "local", "advanced", "human"])
    ev = sub.add_parser("evaluate"); ev.add_argument("input"); ev.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.command == "ground-truth":
        rows = collect_ground_truth(args.root, args.limit)
        if not rows: raise SystemExit("no usable review artifacts found")
        _write_jsonl(rows, args.output); print(f"REVIEW_DECISION_CORPUS={len(rows)} output={args.output}"); return 0
    if args.command == "capture":
        row = build_capture_row(_read(args.request), _read(args.normalized), _read(args.review_artifact), args.review_artifact, args.expected_review)
        _write_jsonl([row], args.output, append=True); print(f"REVIEW_DECISION_CAPTURE={row['case_id']} output={args.output}"); return 0
    _write_json(evaluate(_read_jsonl(args.input)), args.output); return 0


if __name__ == "__main__":
    raise SystemExit(main())
