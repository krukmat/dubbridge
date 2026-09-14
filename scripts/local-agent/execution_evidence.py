#!/usr/bin/env python3
"""Evidence normalization for the software-factory execution seam.

This module consumes existing runner results, existing audit records, and runtime
usage observations. It does not decide routing, retries, fallback authorization,
or reviewer outcomes.
"""

from __future__ import annotations

import json
import os
from typing import Iterable, Optional


def _count_events(transcript, event_name):
    return sum(1 for event in transcript if event.get("event") == event_name)


def _sum_measured(values):
    values = list(values)
    if not values or any(value is None for value in values):
        return None
    return sum(values)


def _authorization_receipt_sha(result):
    selection = result.get("fallback_selection")
    if not isinstance(selection, dict):
        return None
    receipt = selection.get("authorization_receipt")
    if not isinstance(receipt, dict):
        return None
    return receipt.get("receipt_sha256")


def _fallback_handoff_status(result):
    selection = result.get("fallback_selection")
    if isinstance(selection, dict):
        return selection.get("status")
    if result.get("status") == "local_execution_rejected":
        return "cloud_handoff_required"
    return None


def build_execution_summary(
    *,
    execution_session_id: str,
    resolved_execution: dict,
    result: dict,
    usage_records: Iterable[dict],
    authoritative_audit: Optional[dict] = None,
):
    """Build one non-authoritative correlation/economics summary."""

    usage_records = list(usage_records)
    transcript = result.get("transcript", [])
    prompt_tokens = _sum_measured(
        record.get("prompt_tokens") for record in usage_records
    )
    output_tokens = _sum_measured(
        record.get("response_tokens") for record in usage_records
    )

    fallback_ref = result.get("fallback_selection_artifact")
    fallback_status = _fallback_handoff_status(result)
    selection = result.get("fallback_selection")
    fallback_count = 1 if isinstance(selection, dict) else 0

    verification_status = None
    elapsed_ms = None
    if isinstance(authoritative_audit, dict):
        verification = authoritative_audit.get("verification_results")
        if isinstance(verification, dict):
            verification_status = verification.get("final_acceptance_passed")
        elapsed_s = authoritative_audit.get("elapsed_s")
        if isinstance(elapsed_s, (int, float)) and not isinstance(elapsed_s, bool):
            elapsed_ms = round(elapsed_s * 1000, 3)

    return {
        "schema_version": "execution-summary-v1",
        "execution_session_id": execution_session_id,
        "task_id": resolved_execution.get("task_id"),
        "policy_family": resolved_execution.get("policy_family"),
        "policy_version": resolved_execution.get("policy_version"),
        "rri": resolved_execution.get("rri"),
        "band": resolved_execution.get("band"),
        "execution_mode": resolved_execution.get("execution_mode"),
        "logical_binding": resolved_execution.get("logical_binding"),
        "runtime_preset": resolved_execution.get("runtime_preset"),
        "execution_status": result.get("status"),
        "normalized_error_class": (
            None if result.get("status") == "success" else result.get("status")
        ),
        "elapsed_ms": elapsed_ms,
        "counters": {
            "model_invocations": len(usage_records),
            "repair_attempts": _count_events(transcript, "repair_diagnostic"),
            "test_attempts": _count_events(transcript, "test_result"),
            "fallback_handoffs": fallback_count,
        },
        "usage": {
            "prompt_tokens": prompt_tokens,
            "output_tokens": output_tokens,
            "invocations": usage_records,
        },
        "evidence_refs": {
            "fallback_selection_artifact": fallback_ref,
            "authorization_receipt_sha256": _authorization_receipt_sha(result),
        },
        "fallback_handoff_status": fallback_status,
        "verification_status": verification_status,
    }


def write_summary_atomic(summary, out_path):
    tmp = out_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as handle:
        json.dump(summary, handle, indent=2, sort_keys=True)
        handle.write("\n")
    os.replace(tmp, out_path)
