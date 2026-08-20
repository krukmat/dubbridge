#!/usr/bin/env python3
"""LRPC-6 golden-set behavioral-equivalence harness.

Proves that prompt_builder.build_system_prompt() output (LRPC-2) preserves
the model-observed authority-boundary semantics of the full canonical
governing-doc prose it was extracted from (plan Design decision 6,
docs/plan/local-role-prompt-canonicalization.md). This is a verification
tool, not a role: it never authors, reviews, or approves anything itself.

For each fixture in golden_fixtures.FIXTURES, this harness sends the same
transcript to the same live Ollama model twice -- once with the full
canonical prose as system-prompt context ("before"), once with
build_system_prompt() output for the same role ("after") -- and requires
both live-model verdicts to equal the fixture's expected_verdict. A fixture
where the two conditions disagree, or where either disagrees with the
expected verdict, is a FAIL: it means the builder's compressed clause either
lost or changed information the full prose conveyed.

This module performs live network IO against Ollama and is therefore
excluded from the deterministic `make qa-ci` gate (mirrors `make
qa-gemma-review`'s existing local-only status). See `make qa-golden-set`.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import dataclass, field
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from golden_fixtures import (  # noqa: E402
    FIXTURES,
    OUTPUT_FORMAT_TEXT,
    VALID_VERDICTS,
    Fixture,
)
from prompt_builder import build_system_prompt  # noqa: E402
from gemma_local import (  # noqa: E402
    DEFAULT_HOST,
    build_chat_payload,
    endpoint,
    ensure_model_available,
    stream_chat,
    write_result,
)

DEFAULT_MODEL = "gemma4:26b-a4b-it-qat"
DEFAULT_NUM_CTX = 8192
DEFAULT_NUM_PREDICT = 512
DEFAULT_TEMPERATURE = 0.0
DEFAULT_IDLE_TIMEOUT_SECONDS = 180
DEFAULT_MAX_WALL_SECONDS = 300

CONDITION_BEFORE = "canonical_prose"
CONDITION_AFTER = "builder_output"


class GoldenSetError(RuntimeError):
    """Raised for a fail-closed harness error (bad model output, unreachable
    Ollama, unknown role) -- distinct from a fixture verdict mismatch, which
    is a recorded FAIL result, not an exception."""


@dataclass
class ConditionResult:
    condition: str
    system_prompt: str
    verdict: str | None
    reason: str | None
    raw_content: str
    error: str | None = None


@dataclass
class FixtureResult:
    fixture_id: str
    role: str
    expected_verdict: str
    before: ConditionResult
    after: ConditionResult
    equivalent: bool = field(init=False)

    def __post_init__(self) -> None:
        self.equivalent = (
            self.before.error is None
            and self.after.error is None
            and self.before.verdict == self.expected_verdict
            and self.after.verdict == self.expected_verdict
        )


def parse_verdict_response(raw_content: str) -> tuple[str, str]:
    """Parse the model's JSON verdict response. Raises GoldenSetError on any
    structurally invalid response -- this harness never guesses a verdict."""
    trimmed = raw_content.strip()
    if trimmed.startswith("```"):
        lines = trimmed.splitlines()
        if len(lines) >= 3 and lines[-1].strip() == "```":
            trimmed = "\n".join(lines[1:-1]).strip()
    try:
        payload = json.loads(trimmed)
    except json.JSONDecodeError as exc:
        raise GoldenSetError(f"model response is not valid JSON: {exc}") from exc
    if not isinstance(payload, dict):
        raise GoldenSetError("model response root must be a JSON object")
    verdict = payload.get("verdict")
    if verdict not in VALID_VERDICTS:
        raise GoldenSetError(f"model verdict {verdict!r} is not one of {VALID_VERDICTS}")
    reason = payload.get("reason")
    if not isinstance(reason, str):
        reason = ""
    return verdict, reason


def run_condition(
    *,
    condition: str,
    system_prompt: str,
    transcript: str,
    model: str,
    host: str,
    num_ctx: int,
    num_predict: int,
    temperature: float,
    idle_timeout: float,
    max_wall: float,
) -> ConditionResult:
    payload = build_chat_payload(
        model=model,
        system_prompt=system_prompt,
        packet=transcript,
        num_ctx=num_ctx,
        num_predict=num_predict,
        temperature=temperature,
        think=False,
    )
    url = endpoint(host, "/api/chat")
    try:
        result = stream_chat(url, payload, idle_timeout, max_wall, progress_label="golden-set")
        verdict, reason = parse_verdict_response(result.content)
        return ConditionResult(
            condition=condition,
            system_prompt=system_prompt,
            verdict=verdict,
            reason=reason,
            raw_content=result.content,
        )
    except (GoldenSetError, RuntimeError) as exc:
        return ConditionResult(
            condition=condition,
            system_prompt=system_prompt,
            verdict=None,
            reason=None,
            raw_content="",
            error=str(exc),
        )


def run_fixture(
    fixture: Fixture,
    *,
    model: str,
    host: str,
    num_ctx: int,
    num_predict: int,
    temperature: float,
    idle_timeout: float,
    max_wall: float,
) -> FixtureResult:
    before_prompt = f"{fixture.canonical_prose}\n\n{OUTPUT_FORMAT_TEXT}"
    after_prompt = build_system_prompt(
        role=fixture.role,
        num_ctx=num_ctx,
        num_predict=num_predict,
        output_format_text=OUTPUT_FORMAT_TEXT,
    )

    before = run_condition(
        condition=CONDITION_BEFORE,
        system_prompt=before_prompt,
        transcript=fixture.transcript,
        model=model,
        host=host,
        num_ctx=num_ctx,
        num_predict=num_predict,
        temperature=temperature,
        idle_timeout=idle_timeout,
        max_wall=max_wall,
    )
    after = run_condition(
        condition=CONDITION_AFTER,
        system_prompt=after_prompt,
        transcript=fixture.transcript,
        model=model,
        host=host,
        num_ctx=num_ctx,
        num_predict=num_predict,
        temperature=temperature,
        idle_timeout=idle_timeout,
        max_wall=max_wall,
    )
    return FixtureResult(
        fixture_id=fixture.fixture_id,
        role=fixture.role,
        expected_verdict=fixture.expected_verdict,
        before=before,
        after=after,
    )


def run_all(
    fixtures: tuple[Fixture, ...],
    *,
    model: str,
    host: str,
    num_ctx: int,
    num_predict: int,
    temperature: float,
    idle_timeout: float,
    max_wall: float,
) -> list[FixtureResult]:
    return [
        run_fixture(
            fixture,
            model=model,
            host=host,
            num_ctx=num_ctx,
            num_predict=num_predict,
            temperature=temperature,
            idle_timeout=idle_timeout,
            max_wall=max_wall,
        )
        for fixture in fixtures
    ]


def condition_to_dict(condition: ConditionResult) -> dict[str, Any]:
    return {
        "condition": condition.condition,
        "verdict": condition.verdict,
        "reason": condition.reason,
        "error": condition.error,
    }


def result_to_dict(result: FixtureResult) -> dict[str, Any]:
    return {
        "fixture_id": result.fixture_id,
        "role": result.role,
        "expected_verdict": result.expected_verdict,
        "equivalent": result.equivalent,
        "before": condition_to_dict(result.before),
        "after": condition_to_dict(result.after),
    }


def summarize(results: list[FixtureResult]) -> dict[str, Any]:
    failures = [r.fixture_id for r in results if not r.equivalent]
    return {
        "status": "PASS" if not failures else "FAIL",
        "total": len(results),
        "passed": len(results) - len(failures),
        "failed": failures,
        "results": [result_to_dict(r) for r in results],
    }


def build_arg_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default=DEFAULT_MODEL, help="Ollama model tag to run all fixtures against.")
    parser.add_argument("--host", default=DEFAULT_HOST, help="Ollama base host/URL.")
    parser.add_argument("--num-ctx", type=int, default=DEFAULT_NUM_CTX)
    parser.add_argument("--num-predict", type=int, default=DEFAULT_NUM_PREDICT)
    parser.add_argument("--temperature", type=float, default=DEFAULT_TEMPERATURE)
    parser.add_argument("--idle-timeout", type=float, default=DEFAULT_IDLE_TIMEOUT_SECONDS)
    parser.add_argument("--max-wall", type=float, default=DEFAULT_MAX_WALL_SECONDS)
    parser.add_argument("--role", default=None, help="Restrict the run to fixtures for a single role.")
    parser.add_argument("--out", default=None, help="Write the full JSON result to this path.")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_arg_parser().parse_args(argv)

    fixtures = FIXTURES
    if args.role:
        fixtures = tuple(f for f in fixtures if f.role == args.role)
        if not fixtures:
            print(f"[golden-set] no fixtures for role {args.role!r}", file=sys.stderr)
            return 2

    try:
        ensure_model_available(args.host, args.model, args.idle_timeout)
    except RuntimeError as exc:
        print(f"[golden-set] {exc}", file=sys.stderr)
        return 2

    results = run_all(
        fixtures,
        model=args.model,
        host=args.host,
        num_ctx=args.num_ctx,
        num_predict=args.num_predict,
        temperature=args.temperature,
        idle_timeout=args.idle_timeout,
        max_wall=args.max_wall,
    )
    summary = summarize(results)

    if args.out:
        write_result(summary, args.out)

    print(json.dumps({"status": summary["status"], "total": summary["total"], "passed": summary["passed"], "failed": summary["failed"]}))

    return 0 if summary["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
