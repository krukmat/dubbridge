#!/usr/bin/env python3
"""Pure contract utilities for DubBridge decision-model review triage.

This module deliberately has no Ollama or model dependency. It builds the
System One request contract, validates saved responses, and applies the
fail-closed eligibility rules used by shadow evaluation. Runtime invocation is
kept in systemone_local_adapter.py so the contract and evaluation layer remain
fully testable in cloud/CI environments.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from typing import Any

SCHEMA_VERSION = "dubbridge-review-decision-v1"
DEFAULT_MODEL = "nimble:9b-q4_K_M"
DEFAULT_MAX_STATE_CHARS = 24000
DEFAULT_FAST_PATH_THRESHOLD = 0.95

QUESTION_SCHEMA: dict[str, dict[str, Any]] = {
    "risk": {
        "type": "choice",
        "instructions": (
            "Classify the review risk. Use the supplied task/review packet only; "
            "do not assume unseen code or evidence."
        ),
        "criteria": {
            "low": "No material review risk is visible in the supplied state.",
            "moderate": "A bounded issue or uncertainty warrants normal review.",
            "high": "A material correctness, safety, scope, or evidence risk is visible.",
            "critical": "A potentially blocking or high-impact risk requires escalation.",
        },
    },
    "evidence_complete": {
        "type": "noul",
        "instructions": (
            "Is the supplied evidence sufficient to make a review-routing decision "
            "without relying on missing or unseen evidence?"
        ),
        "criteria": {
            "true": "The supplied evidence is sufficient for routing.",
            "false": "Evidence is missing, contradictory, truncated, or insufficient.",
        },
    },
    "scope": {
        "type": "choice",
        "instructions": "Does the observed change stay within the declared task scope?",
        "criteria": {
            "expected": "The supplied change/evidence is consistent with declared scope.",
            "out_of_scope": "The supplied change includes material work outside declared scope.",
            "uncertain": "The supplied state is insufficient to determine scope safely.",
        },
    },
    "failure_domain": {
        "type": "choice",
        "instructions": "What is the primary visible failure or risk domain?",
        "criteria": {
            "none": "No visible failure domain in the supplied state.",
            "code": "Implementation or logic issue.",
            "test": "Test design, coverage, or assertion issue.",
            "environment": "Local runtime/tooling/environment issue.",
            "infrastructure": "Deployment, network, host, CI, or infrastructure issue.",
            "security": "Security, containment, authorization, integrity, or privacy issue.",
            "unknown": "A problem is visible but cannot be safely classified.",
        },
    },
    "suggested_review": {
        "type": "choice",
        "instructions": (
            "What review depth is appropriate? This is advisory only; choose none only "
            "when the supplied state is clearly low-risk and complete."
        ),
        "criteria": {
            "none": "No generative reviewer appears necessary after deterministic gates.",
            "local": "Use the normal local reviewer chain.",
            "advanced": "Use a stronger/deeper reviewer or fallback path.",
            "human": "Human/owner review is appropriate before proceeding.",
        },
    },
}

_EXPECTED_CHOICES = {
    name: tuple(question["criteria"].keys())
    for name, question in QUESTION_SCHEMA.items()
    if question["type"] == "choice"
}


def resolve_band(rri: int) -> str:
    if rri >= 56:
        return "Complex"
    if rri >= 41:
        return "Med-high"
    if rri >= 26:
        return "Moderate"
    return "Low"


def _excerpt(content: str, max_chars: int) -> tuple[str, bool]:
    if max_chars <= 0:
        raise ValueError("max_chars must be positive")
    if len(content) <= max_chars:
        return content, False
    # Keep both the beginning (task/criteria) and end (often tests/evidence).
    marker = "\n...<DUBBRIDGE_DECISION_PACKET_TRUNCATED>...\n"
    available = max_chars - len(marker)
    if available <= 1:
        return content[:max_chars], True
    head = available // 2
    tail = available - head
    return content[:head] + marker + content[-tail:], True


def build_state(
    *,
    content: str,
    phase: str,
    rri: int,
    task_id: str | None,
    metadata: dict[str, Any] | None = None,
    max_chars: int = DEFAULT_MAX_STATE_CHARS,
) -> dict[str, Any]:
    if phase not in {"task", "code"}:
        raise ValueError("phase must be 'task' or 'code'")
    if rri < 0:
        raise ValueError("rri must be non-negative")
    if not content.strip():
        raise ValueError("content must not be empty")

    excerpt, truncated = _excerpt(content, max_chars)
    metadata = dict(metadata or {})
    deterministic = dict(metadata.pop("deterministic", {}))
    checks = dict(metadata.pop("checks", {}))

    content_sha = hashlib.sha256(content.encode("utf-8")).hexdigest()
    normalized_task_id = task_id or "unknown"
    case_id = f"{normalized_task_id}:{phase}:{content_sha[:16]}"

    return {
        "schema_version": SCHEMA_VERSION,
        "case_id": case_id,
        "task_id": normalized_task_id,
        "phase": phase,
        "rri": rri,
        "band": resolve_band(rri),
        "content_sha256": content_sha,
        "content_chars": len(content),
        "packet_truncated": truncated,
        "content": excerpt,
        "checks": {
            "tests": checks.get("tests", "unknown"),
            "contracts": checks.get("contracts", "unknown"),
        },
        "deterministic": {
            "security_sensitive": deterministic.get("security_sensitive"),
            "migration_change": deterministic.get("migration_change"),
            "architecture_change": deterministic.get("architecture_change"),
            "dependency_sensitive": deterministic.get("dependency_sensitive"),
        },
        "metadata": metadata,
    }


def build_systemone_request(
    *,
    state: dict[str, Any],
    model: str = DEFAULT_MODEL,
    keep_alive: str | int = 0,
    images: list[str] | None = None,
) -> dict[str, Any]:
    request: dict[str, Any] = {
        "model": model,
        "state": state,
        "questions": QUESTION_SCHEMA,
        "keep_alive": keep_alive,
    }
    if images:
        request["images"] = images
    return request


def _number(value: Any, field: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field} must be numeric")
    number = float(value)
    if not 0.0 <= number <= 1.0:
        raise ValueError(f"{field} must be between 0 and 1")
    return number


def _normalize_choice(name: str, answer: dict[str, Any]) -> dict[str, Any]:
    expected = _EXPECTED_CHOICES[name]
    choice = answer.get("choice")
    if choice not in expected:
        raise ValueError(f"answers.{name}.choice must be one of {expected}")
    raw_probabilities = answer.get("probabilities")
    if not isinstance(raw_probabilities, dict):
        raise ValueError(f"answers.{name}.probabilities must be an object")
    probabilities = {
        option: _number(raw_probabilities.get(option), f"answers.{name}.probabilities.{option}")
        for option in expected
    }
    return {
        "type": "choice",
        "choice": choice,
        "probabilities": probabilities,
        "selected_probability": probabilities[choice],
        # System One confidence is concentration, not probability of correctness.
        "confidence": _number(answer.get("confidence", 0.0), f"answers.{name}.confidence"),
    }


def _normalize_noul(name: str, answer: dict[str, Any]) -> dict[str, Any]:
    probability_true = _number(answer.get("noul"), f"answers.{name}.noul")
    return {
        "type": "noul",
        "value": probability_true >= 0.5,
        "probability_true": probability_true,
        "probability_false": 1.0 - probability_true,
    }


def normalize_systemone_response(
    response: dict[str, Any], *, request: dict[str, Any] | None = None
) -> dict[str, Any]:
    if not isinstance(response, dict):
        raise ValueError("response must be an object")
    raw_answers = response.get("answers")
    if not isinstance(raw_answers, dict):
        raise ValueError("response.answers must be an object")

    answers: dict[str, Any] = {}
    for name, question in QUESTION_SCHEMA.items():
        raw = raw_answers.get(name)
        if not isinstance(raw, dict):
            raise ValueError(f"response missing answers.{name}")
        if question["type"] == "choice":
            answers[name] = _normalize_choice(name, raw)
        elif question["type"] == "noul":
            answers[name] = _normalize_noul(name, raw)
        else:  # pragma: no cover - current schema has only choice/noul
            raise ValueError(f"unsupported question type: {question['type']}")

    usage = response.get("usage") if isinstance(response.get("usage"), dict) else {}
    normalized: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "model": response.get("model") or (request or {}).get("model") or "unknown",
        "answers": answers,
        "usage": usage,
    }
    if request and isinstance(request.get("state"), dict):
        state = request["state"]
        normalized["case_id"] = state.get("case_id")
        normalized["task_id"] = state.get("task_id", "unknown")
        normalized["phase"] = state.get("phase")
        normalized["rri"] = state.get("rri")
        normalized["content_sha256"] = state.get("content_sha256")
    return normalized


def _selected(decision: dict[str, Any], name: str) -> tuple[str, float]:
    answer = decision["answers"][name]
    return str(answer["choice"]), float(answer["selected_probability"])


def fast_path_eligibility(
    *,
    decision: dict[str, Any],
    state: dict[str, Any],
    threshold: float = DEFAULT_FAST_PATH_THRESHOLD,
) -> dict[str, Any]:
    """Evaluate a prospective future fast-path, always fail-closed.

    This function does *not* grant authority. During the shadow phase its output
    is measurement only. A future policy change would still be required before
    this result could affect the reviewer chain.
    """
    if not 0.5 < threshold <= 1.0:
        raise ValueError("threshold must be > 0.5 and <= 1.0")

    reasons: list[str] = []
    if state.get("rri", 10**9) > 25:
        reasons.append("rri_not_low")
    if state.get("packet_truncated") is not False:
        reasons.append("packet_incomplete_or_truncated")

    checks = state.get("checks") if isinstance(state.get("checks"), dict) else {}
    if checks.get("tests") != "pass":
        reasons.append("tests_not_confirmed_pass")
    if checks.get("contracts") != "pass":
        reasons.append("contracts_not_confirmed_pass")

    deterministic = (
        state.get("deterministic") if isinstance(state.get("deterministic"), dict) else {}
    )
    for flag in (
        "security_sensitive",
        "migration_change",
        "architecture_change",
        "dependency_sensitive",
    ):
        if deterministic.get(flag) is not False:
            reasons.append(f"{flag}_not_confirmed_false")

    required_choices = {
        "risk": "low",
        "scope": "expected",
        "failure_domain": "none",
        "suggested_review": "none",
    }
    for name, expected in required_choices.items():
        choice, probability = _selected(decision, name)
        if choice != expected:
            reasons.append(f"{name}_is_{choice}")
        elif probability < threshold:
            reasons.append(f"{name}_probability_below_threshold")

    evidence = decision["answers"]["evidence_complete"]
    if evidence.get("value") is not True:
        reasons.append("evidence_incomplete")
    elif float(evidence.get("probability_true", 0.0)) < threshold:
        reasons.append("evidence_probability_below_threshold")

    return {
        "eligible": not reasons,
        "threshold": threshold,
        "reasons": reasons,
    }


def read_json(path: str) -> dict[str, Any]:
    with open(path, encoding="utf-8") as stream:
        data = json.load(stream)
    if not isinstance(data, dict):
        raise ValueError(f"{path}: expected JSON object")
    return data


def write_json(data: dict[str, Any], path: str | None) -> None:
    payload = json.dumps(data, indent=2, sort_keys=True) + "\n"
    if not path or path == "-":
        print(payload, end="")
        return
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as stream:
        stream.write(payload)
    os.replace(tmp, path)


def _read_text(path: str | None) -> str:
    if not path or path == "-":
        import sys
        return sys.stdin.read()
    with open(path, encoding="utf-8") as stream:
        return stream.read()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="DubBridge review decision contract tooling")
    sub = parser.add_subparsers(dest="command", required=True)

    packet = sub.add_parser("packet", help="Build a System One request without invoking a model")
    packet.add_argument("--content", default="-", help="Task/review packet path; '-' reads stdin")
    packet.add_argument("--phase", choices=["task", "code"], required=True)
    packet.add_argument("--rri", type=int, required=True)
    packet.add_argument("--task-id", default="unknown")
    packet.add_argument("--metadata", help="Optional JSON metadata/checks file")
    packet.add_argument("--model", default=DEFAULT_MODEL)
    packet.add_argument("--max-state-chars", type=int, default=DEFAULT_MAX_STATE_CHARS)
    packet.add_argument("--output", default="-")

    normalize = sub.add_parser("normalize", help="Normalize a saved System One response")
    normalize.add_argument("--request", required=True, help="Request JSON emitted by packet")
    normalize.add_argument("--response", required=True, help="Raw System One response JSON")
    normalize.add_argument("--threshold", type=float, default=DEFAULT_FAST_PATH_THRESHOLD)
    normalize.add_argument("--output", default="-")

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "packet":
        metadata = read_json(args.metadata) if args.metadata else None
        state = build_state(
            content=_read_text(args.content),
            phase=args.phase,
            rri=args.rri,
            task_id=args.task_id,
            metadata=metadata,
            max_chars=args.max_state_chars,
        )
        request = build_systemone_request(state=state, model=args.model)
        write_json(request, args.output)
        return 0

    request = read_json(args.request)
    response_envelope = read_json(args.response)
    latency_ms = response_envelope.get("latency_ms")
    response = response_envelope.get("response") if isinstance(response_envelope.get("response"), dict) else response_envelope
    decision = normalize_systemone_response(response, request=request)
    state = request.get("state")
    if not isinstance(state, dict):
        raise ValueError("request.state must be an object")
    result = {
        "case_id": state.get("case_id"),
        "decision": decision,
        "fast_path": fast_path_eligibility(
            decision=decision,
            state=state,
            threshold=args.threshold,
        ),
    }
    if isinstance(latency_ms, (int, float)) and not isinstance(latency_ms, bool):
        result["latency_ms"] = float(latency_ms)
    write_json(result, args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
