#!/usr/bin/env python3
"""Additive v3.2 execution-normalization facade.

The facade delegates execution to the existing `run_local_task` path unchanged.
It observes existing runtime/audit outputs and emits one non-authoritative
execution summary for economics/correlation.

No routing, context, retry, fallback, or reviewer authority is implemented here.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
import uuid

import run_local_task as rlt
import gemma_local

from execution_contract import normalize_resolved_execution
from execution_evidence import build_execution_summary, write_summary_atomic


def _split_args(argv):
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument(
        "--execution-summary",
        default=None,
        help="Optional normalized execution-summary output path; defaults to <runner --out>.execution-summary.json",
    )
    return parser.parse_known_args(argv)


def _resolve_inputs(runner_argv):
    """Reuse current runner parsers/resolvers; do not recreate policy."""

    args = rlt.parse_args(runner_argv)
    card = rlt.load_card(args.card)
    limits = rlt.resolve_effective_limits(card)

    # Mirror only the runner's operator-supplied tightening so the normalized
    # snapshot reports the effective bound that `rlt.main()` will enforce.
    # Invalid values are intentionally left for the authoritative runner to
    # reject using its existing error path.
    if (
        args.max_turns is not None
        and args.max_turns > 0
        and args.max_turns <= limits.max_total_turns
    ):
        limits.max_total_turns = args.max_turns

    model = args.model or limits.required_model or rlt.default_local_agent_model(card)
    return args, card, limits, model


def _authoritative_audit(records, task_id):
    candidates = [
        record
        for record in records
        if isinstance(record, dict)
        and record.get("role") == "local-implementer"
        and record.get("task_id") == task_id
        and "verification_results" in record
    ]
    return candidates[-1] if candidates else None


def _observe_stream_chat(original, usage_records):
    invocation_counter = {"n": 0}

    def observed(*args, **kwargs):
        invocation_counter["n"] += 1
        invocation_index = invocation_counter["n"]
        started = time.monotonic()
        try:
            result = original(*args, **kwargs)
        except Exception as exc:
            usage_records.append(
                {
                    "invocation_index": invocation_index,
                    "status": "error",
                    "prompt_tokens": None,
                    "response_tokens": None,
                    "done_reason": None,
                    "elapsed_ms": round((time.monotonic() - started) * 1000, 3),
                    "error_class": type(exc).__name__,
                }
            )
            raise

        usage = gemma_local.stream_result_usage(result)
        usage_records.append(
            {
                "invocation_index": invocation_index,
                "status": "completed",
                "prompt_tokens": usage.prompt_tokens,
                "response_tokens": usage.response_tokens,
                "done_reason": usage.done_reason,
                "elapsed_ms": round((time.monotonic() - started) * 1000, 3),
                "error_class": None,
            }
        )
        return result

    return observed


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    wrapper_args, runner_argv = _split_args(argv)
    runner_args, card, limits, model = _resolve_inputs(runner_argv)

    execution_session_id = uuid.uuid4().hex
    usage_records = []
    audit_records = []

    original_stream_chat = gemma_local.stream_chat
    original_append_audit_log = gemma_local.append_audit_log

    def observed_append_audit_log(record, **kwargs):
        audit_records.append(record)
        return original_append_audit_log(record, **kwargs)

    gemma_local.stream_chat = _observe_stream_chat(original_stream_chat, usage_records)
    gemma_local.append_audit_log = observed_append_audit_log
    try:
        exit_code = rlt.main(runner_argv)
    finally:
        gemma_local.stream_chat = original_stream_chat
        gemma_local.append_audit_log = original_append_audit_log

    with open(runner_args.out, encoding="utf-8") as handle:
        result = json.load(handle)

    authorization_ref = result.get("fallback_selection_artifact")
    resolved = normalize_resolved_execution(
        card=card,
        limits=limits,
        model=model,
        num_ctx=runner_args.num_ctx,
        num_predict=runner_args.num_predict,
        authorization_ref=authorization_ref,
    ).as_dict()

    summary = build_execution_summary(
        execution_session_id=execution_session_id,
        resolved_execution=resolved,
        result=result,
        usage_records=usage_records,
        authoritative_audit=_authoritative_audit(audit_records, card.task_id),
    )

    summary_path = (
        wrapper_args.execution_summary
        or f"{runner_args.out}.execution-summary.json"
    )
    write_summary_atomic(summary, summary_path)
    print(f"[execution-seam] summary: {summary_path}", file=sys.stderr)
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
