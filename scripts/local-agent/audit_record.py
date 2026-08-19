#!/usr/bin/env python3
"""Audit/evidence construction for the local agentic runner.

Extracted from run_local_task.py (LRPC-0b, Extract Module / Single
Responsibility): closure/evidence construction, no model interaction.
Behavior-preserving: no logic changed from the original module.
"""

from __future__ import annotations

import datetime

import fallback_selection


def build_audit_record(card, result, model, elapsed_s, effective_limits=None):
    # Derived entirely from the transcript run_loop already produced — no
    # new capture logic, only aggregation, so this stays in lockstep with
    # whatever event shapes T6a/T6b already emit instead of duplicating them.
    transcript = result.get("transcript", [])
    test_events = [e for e in transcript if e.get("event") == "test_result"]
    command_events = [
        e["result"] for e in transcript
        if e.get("event") == "tool_result" and e["result"].get("tool") == "run_command"
    ]
    command_events.extend(
        command
        for event in test_events
        for command in event["result"].get("commands", [])
    )
    edit_events = [
        e["result"] for e in transcript
        if e.get("event") == "tool_result" and e["result"].get("tool") in ("write_file", "apply_patch")
    ]
    boundary_violations = [e for e in transcript if e.get("event") == "boundary_violation"]
    scope_check_events = [e for e in transcript if e.get("event") == "scope_check"]
    scope_check_result = scope_check_events[-1] if scope_check_events else None
    acceptance_results = [e["result"]["passed"] for e in test_events]
    verification_results = {
        "acceptance_tests": acceptance_results,
        "final_acceptance_passed": acceptance_results[-1] if acceptance_results else None,
        "scope_in_scope": scope_check_result["in_scope"] if scope_check_result else None,
    }
    validation_errors = []
    if result["status"] == "success":
        if scope_check_result is None or not scope_check_result["in_scope"]:
            validation_errors.append("scope_gate_not_passed")
        if verification_results["final_acceptance_passed"] is not True:
            validation_errors.append("acceptance_tests_not_passed")
    signed = result["status"] == "success" and not validation_errors
    signature = {
        "status": "signed" if signed else "unsigned",
        "signer": "local-implementer" if signed else None,
        "reason": (
            "all_mandatory_gates_passed"
            if signed
            else (
                validation_errors[0]
                if validation_errors
                else result["status"]
            )
        ),
    }

    return {
        "ts": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
        "role": "local-implementer",
        "outcome": result["status"].upper(),
        "model": model,
        "task_id": card.task_id,
        "rri": card.rri,
        "band": card.band,
        "effective_limits": effective_limits.as_dict() if effective_limits else None,
        "attempts": len(test_events),
        "commands": [c["argv"] for c in command_events],
        "edit_metrics": [
            {
                "tool": e["tool"],
                "path": e["path"],
                "line_count": e.get("line_count"),
                "byte_count": e.get("byte_count"),
                "anchor_matches": e.get("anchor_matches"),
            }
            for e in edit_events
        ],
        "test_results": [e["result"]["passed"] for e in test_events],
        "boundary_violations": len(boundary_violations),
        "scope_check": {
            "in_scope": scope_check_result["in_scope"],
            "offending_paths": scope_check_result["offending_paths"],
        } if scope_check_result else None,
        "verification_results": verification_results,
        "audit_validation": {
            "valid": not validation_errors,
            "errors": validation_errors,
        },
        "signature": signature,
        "escalated": result["status"] != "success",
        "elapsed_s": round(elapsed_s, 3),
    }


def build_attempt_bundles(card, result, model, session_start, session_end):
    """T2 (docs/tasks/local-first-cloud-local-handoff.md): one T1 attempt
    bundle per repair attempt in this session, read-only over the same
    transcript build_audit_record already aggregates -- no new capture
    logic, no change to run_loop's control flow or build_audit_record's
    output.

    Segmentation: run_loop's transcript is one flat list across the whole
    session; each attempt ends at its `test_result` event (finish -> tests
    run -> pass or repair). Splitting on that event boundary turns the flat
    list into per-attempt slices without touching run_loop itself. A trailing
    slice with no closing test_result (e.g. the turn budget or a boundary
    violation cut the session off before finish) has no attempt to report,
    per EC-1 -- no partial/malformed bundle is emitted for it.

    Timestamps: run_loop's transcript events carry no per-event timestamps,
    so there is no ground truth for exactly when each attempt started/ended
    within the session. Qwen27 phase-1 review (T2) flagged an earlier version
    that called datetime.now() once per attempt *after* run_loop had already
    returned — every bundle got a near-identical wall-clock timestamp from
    bundle-generation time, not from when the attempt actually happened, which
    is worse than useless for audit purposes. Rather than fabricate false
    per-attempt precision, every bundle's start_ts/end_ts is bounded by the
    caller-supplied session_start/session_end (the same window build_audit_record's
    elapsed_s is computed from) -- honest about the granularity actually
    available instead of inventing timestamps the data doesn't support.

    Returns [] when the card carries no capsule_hash (session predates T1/T2
    adoption): a bundle without a real capsule hash cannot pass T1's
    known_capsule_hashes check, so emitting one would only be discarded
    downstream.
    """
    if not card.capsule_hash:
        return []

    transcript = result.get("transcript", [])
    test_event_count = sum(1 for e in transcript if e.get("event") == "test_result")
    bundles = []
    segment = []
    tests_seen = 0
    for event in transcript:
        segment.append(event)
        if event.get("event") != "test_result":
            continue
        tests_seen += 1
        test_result = event["result"]
        edit_events = [
            e["result"] for e in segment
            if e.get("event") == "tool_result" and e["result"].get("tool") in ("write_file", "apply_patch")
        ]
        is_last_test_event = tests_seen == test_event_count
        if test_result["passed"]:
            outcome = "success"
        elif is_last_test_event:
            # Qwen27 phase-1 review (T2): this used to check
            # `result["status"] == "budget_exhausted"` specifically, so a
            # failing last attempt under any other terminal status (e.g.
            # boundary_violation, transport_error, aborted) fell through to
            # "repair-needed" -- implying another attempt was coming, which
            # is false once the session has actually ended. Any failing final
            # attempt is escalated regardless of which terminal status ended
            # the session; only a non-final failing attempt (more repair
            # turns follow within the same session) is "repair-needed".
            outcome = "escalated"
        else:
            outcome = "repair-needed"
        bundles.append(
            {
                "capsule_hash": card.capsule_hash,
                "implementer_id": "nemotron",
                "model_tag": model,
                "start_ts": session_start.isoformat().replace("+00:00", "Z"),
                "end_ts": session_end.isoformat().replace("+00:00", "Z"),
                "diff_ref": [
                    {"tool": e["tool"], "path": e["path"]} for e in edit_events
                ],
                "test_results": test_result,
                "review_verdict": "pending",
                "outcome": outcome,
            }
        )
        segment = []
    return bundles


_MODERATE_TERMINAL_RESULTS = {
    ("budget_exhausted", "repair_attempts_exhausted"),
    ("budget_exhausted", "total_turns_exhausted"),
    ("transport_error", None),
    ("boundary_violation", None),
    ("out_of_scope", None),
}


def _is_moderate_card(card):
    rri = getattr(card, "rri", None)
    return isinstance(rri, int) and not isinstance(rri, bool) and 26 <= rri <= 40


def _terminal_result_is_eligible(card, result):
    if not _is_moderate_card(card):
        return False
    status = result.get("status")
    reason = result.get("reason")
    return (status, reason) in _MODERATE_TERMINAL_RESULTS or (status, None) in _MODERATE_TERMINAL_RESULTS


def build_terminal_attempt_packet(card, result, model, effective_limits):
    """Build the one canonical packet for an eligible Moderate terminal exit.

    This intentionally does not reuse the older per-test attempt bundle: a
    boundary, transport, scope, or turn-budget exit can have no test result at
    all, but still needs identical selection evidence before cloud handoff.
    """
    if not _terminal_result_is_eligible(card, result):
        raise ValueError("terminal attempt packet requires an eligible Moderate terminal result")

    transcript = result.get("transcript", [])
    test_results = [event["result"] for event in transcript if event.get("event") == "test_result"]
    turn_budget_events = [event for event in transcript if event.get("event") == "turn_budget_exhausted"]
    total_turns_used = (
        turn_budget_events[-1].get("total_turns")
        if turn_budget_events
        else sum(1 for event in transcript if event.get("role") == "assistant")
    )
    repair_attempts_used = result.get("attempts")
    if repair_attempts_used is None:
        repair_attempts_used = max(0, len(test_results) - 1)

    return {
        "task_id": card.task_id,
        "phase": "implementation",
        "terminal_status": result["status"],
        "terminal_reason": result.get("reason"),
        "trigger": result.get("reason") or result["status"],
        "trigger_kind": fallback_selection.TRIGGER_OPERATIONAL,
        "rri": card.rri,
        "implementer_model": model,
        "effective_limits": effective_limits.as_dict(),
        "counters_used": {
            "total_turns": total_turns_used,
            "repair_attempts": repair_attempts_used,
            "test_attempts": len(test_results),
        },
        "terminal_transcript": transcript,
        "final_test_result": test_results[-1] if test_results else None,
    }


def build_moderate_fallback_checkpoint(args, card, result, model, effective_limits):
    """Return the shared selection checkpoint only for a terminal Moderate exit."""
    if not _terminal_result_is_eligible(card, result):
        return None
    packet = build_terminal_attempt_packet(card, result, model, effective_limits)
    checkpoint = fallback_selection.build_checkpoint_from_args(
        args,
        task_id=card.task_id,
        phase="implementation",
        trigger=packet["trigger"],
        role=fallback_selection.ROLE_CLOUD_IMPLEMENTER,
        rri=card.rri,
        packet=packet,
        trigger_kind=fallback_selection.TRIGGER_OPERATIONAL,
    )
    if checkpoint["status"] == fallback_selection.AUTHORIZED_STATUS:
        fallback_selection.validate_authorized_checkpoint(checkpoint, packet)
    return checkpoint
