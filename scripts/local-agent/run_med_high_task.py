#!/usr/bin/env python3
"""Med-high cloud-handoff supervisor and evidence bundle (ADR-038 T4).

Owns the Med-high cloud-handoff evidence required by ADR-038. Owner directive
2026-08-12 limits the local developer to Low/S and Moderate/M, so a valid
Med-high `GO_LOCAL` advisory is recorded as policy-excluded and never launches
`run_local_task.py`.

This module never edits code and never re-implements the runner's own turn
loop, model binding, or repair budget -- that enforcement lives in
`run_local_task.py` (ADR-038 T3) and is invoked here strictly as an external,
killable subprocess. Per the handoff hard rule, the not-yet-enforced local
runner is not used to implement its own enforcement: the 300-second cutoff
and process-group kill are supervisor-level OS controls, external to the
runner process, not cooperative behavior requested of it.
"""

from __future__ import annotations

import hashlib
import json
import os
import signal
import subprocess
import sys
import time
from dataclasses import dataclass
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import fallback_selection

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import escalation_packet
import med_high_gate

MED_HIGH_WALL_CLOCK_SECONDS = 300
MED_HIGH_RUNNER_MODEL = "nemotron-3.5-lightning:30b-a3b-q4_K_M"
POST_KILL_WAIT_SECONDS = 5
ROUTE_GO_LOCAL = med_high_gate.ROUTE_GO_LOCAL
ROUTE_CLOUD_REQUIRED = med_high_gate.ROUTE_CLOUD_REQUIRED

STATUS_SUCCESS = "success"


@dataclass(frozen=True)
class SupervisorResult:
    status: str
    route: str
    reason: str
    runner_result: dict[str, Any] | None
    bundle_path: str | None
    elapsed_s: float
    bundle_write_ok: bool = True
    fallback_selection_artifact: str | None = None
    fallback_selection: dict[str, Any] | None = None
    cloud_instruction: dict[str, str] | None = None


@dataclass(frozen=True)
class BundleWriteResult:
    path: str
    write_ok: bool
    write_error: str | None


def _load_json(path: str) -> dict[str, Any]:
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def _load_optional_artifact_json(path: str | None) -> tuple[Any | None, str | None]:
    """Load an optional gate artifact (refinement artifact or primary
    receipt), never letting an existing-but-unreadable file crash bundle
    construction (plan Defect B). Existence (os.path.isfile) is not
    readability -- the correct idiom, mirroring _read_runner_out, is
    try/except around the read itself. Returns (value, error): error is None
    on success or on a genuinely absent path; value is None whenever error is
    not None."""
    if not path or not os.path.isfile(path):
        return None, None
    try:
        return _load_json(path), None
    except (OSError, json.JSONDecodeError, UnicodeDecodeError) as exc:
        return None, str(exc)


class GateInputError(Exception):
    """A gate artifact (refinement artifact or primary receipt) could not be
    read or parsed. Distinct from med_high_gate.GateError, which signals a
    successfully-read artifact that failed gate validation. Caught by
    supervise() alongside GateError so a corrupt or unreadable gate input
    surfaces as CLOUD_REQUIRED with a bundle, never an uncaught traceback
    (plan Defect C, D6)."""

    def __init__(self, artifact_label: str, path: str, error: str):
        self.artifact_label = artifact_label
        self.path = path
        self.error = error
        super().__init__(f"{artifact_label} unreadable: {path}: {error}")


def decide_route(
    *, refinement_artifact_path: str, primary_receipt_path: str, card_hash: str, rri: int
) -> med_high_gate.GateDecision:
    """Thin wrapper over the T2 gate: pure, offline, fail-closed. Any read or
    validation failure is surfaced as CLOUD_REQUIRED by the caller, never as
    an uncaught exception that could be mistaken for a crash."""
    try:
        refinement_artifact = _load_json(refinement_artifact_path)
    except (OSError, json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise GateInputError("refinement artifact", refinement_artifact_path, str(exc)) from exc
    try:
        primary_receipt = _load_json(primary_receipt_path)
    except (OSError, json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise GateInputError("primary receipt", primary_receipt_path, str(exc)) from exc
    return med_high_gate.evaluate_route(
        refinement_artifact=refinement_artifact,
        primary_receipt=primary_receipt,
        card_hash=card_hash,
        rri=rri,
    )


def _runner_argv(*, card_path, worktree, out_path, model, python_executable=None):
    runner_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "run_local_task.py")
    return [
        python_executable or sys.executable,
        runner_path,
        "--card", card_path,
        "--worktree", worktree,
        "--out", out_path,
        "--model", model,
    ]


def run_supervised_runner(
    *,
    card_path: str,
    worktree: str,
    out_path: str,
    model: str,
    wall_clock_seconds: int = MED_HIGH_WALL_CLOCK_SECONDS,
    popen_fn=subprocess.Popen,
    python_executable=None,
) -> dict[str, Any]:
    """Launch run_local_task.py as its own process group and enforce the
    ADR-038 section 4 hard wall-clock cutoff.

    Mirrors run_local_task.py's own _run_command_with_timeout: start_new_session=True
    plus killpg on timeout, so a multi-process hang inside the runner (e.g. a
    stuck `cargo test` it launched) cannot survive the cutoff by hiding in a
    grandchild. On timeout, --out is left exactly as the runner's own
    checkpoint_fn last wrote it (ADR-038 T3), which is the "preserve the last
    checkpoint and partial diff" requirement (EC-1) -- this function does not
    need to reconstruct that state itself.
    """
    argv = _runner_argv(
        card_path=card_path, worktree=worktree, out_path=out_path, model=model,
        python_executable=python_executable,
    )
    start = time.monotonic()
    try:
        process = popen_fn(argv, start_new_session=True)
    except (OSError, ValueError) as exc:
        return {
            "status": "transport_error",
            "reason": f"failed to start run_local_task.py: {exc}",
        }

    try:
        process.wait(timeout=wall_clock_seconds)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(os.getpgid(process.pid), signal.SIGKILL)
        except ProcessLookupError:
            pass  # already exited between the timeout firing and the kill
        except OSError as exc:
            # Phase-2 review finding: getpgid/killpg can also raise a
            # permission or platform-specific OSError beyond the common
            # already-exited race. Report it as a structured stop reason
            # rather than letting it escape uncaught -- the supervisor must
            # fail closed into an evidence bundle, never crash silently.
            return {
                "status": "wall_clock_exceeded",
                "reason": f"run_local_task.py exceeded the {wall_clock_seconds}s Med-high wall clock; process group kill failed: {exc}",
                "elapsed_s": time.monotonic() - start,
            }
        try:
            process.wait(timeout=POST_KILL_WAIT_SECONDS)
        except (subprocess.TimeoutExpired, OSError) as exc:
            # D10: the process group has already been sent SIGKILL, so this
            # wait should return promptly; a hung wait or a wait-time OSError
            # (e.g. ECHILD if something else already reaped it) must not
            # prevent supervise() from reaching bundle construction.
            return {
                "status": "wall_clock_exceeded",
                "reason": f"run_local_task.py exceeded the {wall_clock_seconds}s Med-high wall clock; process group killed; post-kill wait failed: {exc}",
                "elapsed_s": time.monotonic() - start,
            }
        elapsed_s = time.monotonic() - start
        return {
            "status": "wall_clock_exceeded",
            "reason": f"run_local_task.py exceeded the {wall_clock_seconds}s Med-high wall clock; process group killed.",
            "elapsed_s": elapsed_s,
        }

    elapsed_s = time.monotonic() - start
    if process.returncode != 0:
        return {
            "status": "runner_nonzero_exit",
            "reason": f"run_local_task.py exited {process.returncode}.",
            "elapsed_s": elapsed_s,
            "returncode": process.returncode,
        }
    return {"status": "runner_exited", "elapsed_s": elapsed_s, "returncode": 0}


def _read_runner_out(out_path: str) -> tuple[dict[str, Any] | None, str | None]:
    """Returns (value, failure_reason): value is the parsed runner output on
    success, or None on either a read failure or a wrong-shaped-but-parsed
    value. failure_reason is only set for the wrong-shape case, since that is
    the only one with a meaningful reason to report -- a missing/undecodable
    file has nothing to shape-check (plan D9: consumed directly by the
    caller, which renders "MISSING with reason", not routed through
    escalation_packet.py's ADR-036-specific result.get("reason") convention)."""
    if not os.path.isfile(out_path):
        return None, None
    try:
        parsed = _load_json(out_path)
    except (OSError, json.JSONDecodeError, UnicodeDecodeError):
        return None, None
    return escalation_packet.validate_json_object_shape(parsed)


def _render_gate_artifact_sections(
    *, refinement_artifact_path: str | None, primary_receipt_path: str | None
) -> tuple[str, str, str]:
    """Render the refinement-artifact section, primary-receipt section, and
    refinement-artifact SHA-256 field, each fail-visible on a read failure
    rather than raising (plan Defect B/T2). Split out of build_evidence_bundle
    to keep its own cyclomatic complexity under the radon ceiling."""
    refinement_artifact, refinement_error = _load_optional_artifact_json(refinement_artifact_path)
    primary_receipt, receipt_error = _load_optional_artifact_json(primary_receipt_path)

    if refinement_error is not None:
        refinement_section = f"{escalation_packet.MISSING} (refinement artifact unreadable: {refinement_artifact_path}: {refinement_error})"
        refinement_sha_field = f"{escalation_packet.MISSING} (not computed: source unreadable)"
    elif refinement_artifact is not None:
        refinement_section = json.dumps(refinement_artifact, indent=2, sort_keys=True)
        refinement_sha_field = med_high_gate.sha256_of(refinement_artifact)
    else:
        refinement_section = escalation_packet.MISSING
        refinement_sha_field = escalation_packet.MISSING

    if receipt_error is not None:
        receipt_section = f"{escalation_packet.MISSING} (primary receipt unreadable: {primary_receipt_path}: {receipt_error})"
    elif primary_receipt is not None:
        receipt_section = json.dumps(primary_receipt, indent=2, sort_keys=True)
    else:
        receipt_section = escalation_packet.MISSING

    return refinement_section, receipt_section, refinement_sha_field


def build_evidence_bundle(
    *,
    bundle_out_path: str,
    card_path: str,
    runner_out_path: str,
    stop_reason: str,
    elapsed_s: float = 0.0,
    refinement_artifact_path: str | None = None,
    primary_receipt_path: str | None = None,
    effective_limits: dict[str, Any] | None = None,
    diff_file: str | None = None,
    rri_table: str | None = None,
    card_hash: str | None = None,
) -> BundleWriteResult:
    """Assemble the ADR-038 section 5 cloud-escalation bundle.

    Reuses escalation_packet.build_packet for the seven ADR-036 sections
    (task spec/RRI, plan, allowed paths, diff, commands, tests, per-attempt
    summaries) verbatim, then appends the ADR-038-specific sections the
    section-5 evidence list requires beyond ADR-036's original scope:
    acceptance tests, the refinement artifact, the primary receipt, the
    effective limits, and an explicit stop-reason/hash/model-identity/
    elapsed-time footer. Every missing optional input renders literal
    "MISSING" text, the same fail-visible convention escalation_packet.py
    already uses, rather than silently omitting a section cloud continuation
    might need.

    Writes atomically (temp file + fsync + os.replace) and never raises on a
    write failure (plan Defect E/D8): the caller inspects the returned
    BundleWriteResult instead. This is the plan's one deliberate exception
    besides the task-card read -- a storage-layer failure is reported
    structurally, not converted into a successful write.
    """
    card = escalation_packet.load_card(card_path)
    runner_value, runner_shape_failure_reason = _read_runner_out(runner_out_path)
    if runner_shape_failure_reason is not None:
        runner_result = {"status": "transcript_shape_invalid", "reason": runner_shape_failure_reason}
    else:
        runner_result = runner_value or {}
    diff_text, diff_missing = escalation_packet.read_optional_text_file(diff_file, label="diff file")
    rri_table_text = escalation_packet.resolve_rri_table(rri_table)

    base_packet = escalation_packet.build_packet(
        card, runner_result, diff_text, rri_table_text, diff_missing=diff_missing
    )

    refinement_section, receipt_section, refinement_sha_field = _render_gate_artifact_sections(
        refinement_artifact_path=refinement_artifact_path,
        primary_receipt_path=primary_receipt_path,
    )

    acceptance_tests = card.get("acceptance_tests") or []
    acceptance_tests_section = (
        "\n".join(f"- `{t}`" for t in acceptance_tests)
        if acceptance_tests
        else escalation_packet.MISSING
    )

    extra_sections = [
        ("8. Acceptance tests", acceptance_tests_section),
        ("9. Refinement artifact (Qwen27)", refinement_section),
        ("10. Primary route receipt", receipt_section),
        (
            "11. Effective limits",
            json.dumps(effective_limits, indent=2, sort_keys=True)
            if effective_limits is not None
            else escalation_packet.MISSING,
        ),
        (
            "12. Stop reason and hashes",
            (
                f"Stop reason: `{stop_reason}`\n\n"
                f"Card hash: `{card_hash or escalation_packet.MISSING}`\n\n"
                f"Refinement artifact SHA-256: `{refinement_sha_field}`\n\n"
                f"Runner model: `{runner_result.get('model', escalation_packet.MISSING)}`\n\n"
                f"Runner status: `{runner_result.get('status', escalation_packet.MISSING)}`\n\n"
                f"Elapsed: `{elapsed_s}s`"
            ),
        ),
    ]

    parts = [base_packet.rstrip("\n")]
    for title, body in extra_sections:
        parts.append(f"\n\n## {title}\n\n{body}\n")
    full_packet = "".join(parts)

    return _write_bundle_atomically(bundle_out_path, full_packet)


def _write_bundle_atomically(bundle_out_path: str, content: str) -> BundleWriteResult:
    """Write via a temp file in the same directory, fsync, then os.replace,
    so a reader never observes a partially-written bundle (plan D8). Catches
    both OSError (disk/permission/path failures) and UnicodeEncodeError (an
    unencodable code point reaching the write stream, the mirror of the
    UnicodeDecodeError guards on the read side) and never re-raises -- the
    caller inspects write_ok/write_error instead."""
    tmp_path = f"{bundle_out_path}.tmp"
    try:
        with open(tmp_path, "w", encoding="utf-8") as f:
            f.write(content)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp_path, bundle_out_path)
    except (OSError, UnicodeEncodeError) as exc:
        try:
            os.remove(tmp_path)
        except OSError:
            pass
        return BundleWriteResult(path=bundle_out_path, write_ok=False, write_error=str(exc))
    return BundleWriteResult(path=bundle_out_path, write_ok=True, write_error=None)


def _amend_reason_for_write_failure(reason: str, write_result: BundleWriteResult) -> str:
    """Plan D8 item 4: a write failure is reported by amending the existing
    reason field, not by adding a new output channel."""
    if write_result.write_ok:
        return reason
    return f"{reason} (bundle write failed: {write_result.write_error})"


def _trigger_kind_for_result(result: SupervisorResult) -> str:
    """Classify a Med-high takeover without inferring from mutable policy.

    A failed runner launch is the sole operational-only supervisor outcome.
    Gate rejections, direct CLOUD_REQUIRED decisions, and every launched
    runner failure have already established a capability or risk boundary.
    """
    if result.status == "transport_error":
        return fallback_selection.TRIGGER_OPERATIONAL
    return fallback_selection.TRIGGER_CAPABILITY_RISK


def build_fallback_packet(
    *, card_path: str, rri: int, result: SupervisorResult
) -> dict[str, Any]:
    """Bind a selection receipt to the exact bytes of the emitted bundle."""
    if not result.bundle_write_ok or not result.bundle_path:
        raise fallback_selection.FallbackSelectionError("bundle: unavailable")
    try:
        with open(result.bundle_path, "rb") as stream:
            bundle_bytes = stream.read()
    except OSError as exc:
        raise fallback_selection.FallbackSelectionError(
            f"bundle: unreadable ({exc})"
        ) from exc

    card = escalation_packet.load_card(card_path)
    task_id = card.get("task_id")
    if not isinstance(task_id, str) or not task_id.strip():
        raise fallback_selection.FallbackSelectionError("task_id: missing from card")
    trigger_kind = _trigger_kind_for_result(result)
    return {
        "task_id": task_id,
        "phase": "implementation",
        "terminal_status": result.status,
        "terminal_route": result.route,
        "terminal_reason": result.reason,
        "trigger": result.status,
        "trigger_kind": trigger_kind,
        "rri": rri,
        "handoff_bundle": {
            "artifact_path": os.path.abspath(result.bundle_path),
            "sha256": hashlib.sha256(bundle_bytes).hexdigest(),
        },
    }


def _blocked_checkpoint(
    *, task_id: str, rri: int, result: SupervisorResult, summary: str
) -> dict[str, Any]:
    """Return a receipt-free, fail-closed artifact with a stable summary."""
    return {
        "schema_version": fallback_selection.SCHEMA_VERSION,
        "task_id": task_id,
        "phase": "implementation",
        "status": "blocked",
        "verdict": "blocked",
        "summary": summary,
        "trigger": result.status,
        "trigger_kind": _trigger_kind_for_result(result),
        "role": fallback_selection.ROLE_CLOUD_IMPLEMENTER,
        "rri": rri,
    }


def _attach_fallback_selection(
    *,
    card_path: str,
    rri: int,
    result: SupervisorResult,
    fallback_mode: str,
    fallback_model: str | None,
    fallback_reasoning_effort: str | None,
    fallback_selected_by: str | None,
    fallback_selection_artifact: str | None,
) -> SupervisorResult:
    """Gate one non-success handoff behind a hash-bound selection receipt.

    This never invokes a cloud model.  It writes an awaiting, authorized, or
    blocked checkpoint artifact and exposes a cloud instruction only after the
    receipt validates against a freshly-read copy of the bundle bytes.
    """
    card = escalation_packet.load_card(card_path)
    task_id = card.get("task_id") if isinstance(card.get("task_id"), str) else "unknown"
    artifact_path = (
        fallback_selection_artifact
        or fallback_selection.default_checkpoint_path(result.bundle_path or "fallback")
    )
    try:
        packet = build_fallback_packet(card_path=card_path, rri=rri, result=result)
        checkpoint = fallback_selection.build_checkpoint(
            task_id=packet["task_id"],
            phase="implementation",
            trigger=packet["trigger"],
            role=fallback_selection.ROLE_CLOUD_IMPLEMENTER,
            rri=rri,
            packet=packet,
            trigger_kind=packet["trigger_kind"],
            selection_mode=fallback_mode,
            selected_model=fallback_model,
            selected_reasoning_effort=fallback_reasoning_effort,
            selected_by=fallback_selected_by,
        )
        if checkpoint["status"] == fallback_selection.AUTHORIZED_STATUS:
            # Re-read the bundle after receipt creation. A concurrent mutation
            # cannot be authorized by the earlier byte digest.
            current_packet = build_fallback_packet(
                card_path=card_path, rri=rri, result=result
            )
            try:
                fallback_selection.validate_authorized_checkpoint(
                    checkpoint, current_packet
                )
            except fallback_selection.FallbackSelectionError:
                checkpoint = _blocked_checkpoint(
                    task_id=packet["task_id"], rri=rri, result=result,
                    summary="bundle_receipt_mismatch",
                )
        fallback_selection.write_checkpoint(checkpoint, artifact_path)
    except (fallback_selection.FallbackSelectionError, OSError) as exc:
        checkpoint = _blocked_checkpoint(
            task_id=task_id, rri=rri, result=result, summary=str(exc)
        )
        try:
            fallback_selection.write_checkpoint(checkpoint, artifact_path)
        except OSError:
            artifact_path = None

    cloud_instruction = None
    if checkpoint["status"] == fallback_selection.AUTHORIZED_STATUS:
        cloud_instruction = {
            "model": checkpoint["selected_model"],
            "reasoning_effort": checkpoint["selected_reasoning_effort"],
        }
    return SupervisorResult(
        status=result.status, route=result.route, reason=result.reason,
        runner_result=result.runner_result, bundle_path=result.bundle_path,
        elapsed_s=result.elapsed_s, bundle_write_ok=result.bundle_write_ok,
        fallback_selection_artifact=artifact_path,
        fallback_selection=checkpoint,
        cloud_instruction=cloud_instruction,
    )


def _pre_launch_bundle(
    *,
    bundle_out_path: str,
    card_path: str,
    out_path: str,
    refinement_artifact_path: str,
    primary_receipt_path: str,
    card_hash: str,
    diff_file: str | None,
    rri_table: str | None,
    stop_reason: str,
    status: str,
    reason: str,
) -> SupervisorResult:
    """Build and emit a bundle for a route decided before any runner launch:
    a gate rejection, an unreadable gate artifact, or a direct CLOUD_REQUIRED
    decision. Shared by supervise()'s three pre-launch outcomes so the
    call/wrap pattern isn't triplicated."""
    write_result = build_evidence_bundle(
        bundle_out_path=bundle_out_path,
        card_path=card_path,
        runner_out_path=out_path,
        stop_reason=stop_reason,
        elapsed_s=0.0,
        refinement_artifact_path=refinement_artifact_path,
        primary_receipt_path=primary_receipt_path,
        card_hash=card_hash,
        diff_file=diff_file,
        rri_table=rri_table,
    )
    return SupervisorResult(
        status=status, route=ROUTE_CLOUD_REQUIRED,
        reason=_amend_reason_for_write_failure(reason, write_result),
        runner_result=None, bundle_path=write_result.path, elapsed_s=0.0,
        bundle_write_ok=write_result.write_ok,
    )


def _post_launch_bundle(
    *,
    bundle_out_path: str,
    card_path: str,
    out_path: str,
    refinement_artifact_path: str,
    primary_receipt_path: str,
    card_hash: str,
    diff_file: str | None,
    rri_table: str | None,
    launch_outcome: dict[str, Any],
    runner_result: dict[str, Any] | None,
    elapsed_s: float,
) -> SupervisorResult:
    """Build and emit a bundle once the runner has launched and did not
    succeed (EC-1, EC-2): the stop reason is drawn from the runner's own
    reported status when it exited cleanly, or from the launch outcome
    itself (e.g. wall_clock_exceeded, transport_error) otherwise."""
    stop_reason = launch_outcome["status"]
    if launch_outcome["status"] == "runner_exited" and runner_result is not None:
        stop_reason = runner_result.get("status", stop_reason)

    effective_limits = (
        (runner_result or {}).get("effective_limits")
        or (runner_result or {}).get("audit_effective_limits")
    )
    write_result = build_evidence_bundle(
        bundle_out_path=bundle_out_path,
        card_path=card_path,
        runner_out_path=out_path,
        stop_reason=stop_reason,
        elapsed_s=elapsed_s,
        refinement_artifact_path=refinement_artifact_path,
        primary_receipt_path=primary_receipt_path,
        effective_limits=effective_limits,
        card_hash=card_hash,
        diff_file=diff_file,
        rri_table=rri_table,
    )
    reason = launch_outcome.get("reason", stop_reason)
    return SupervisorResult(
        status=stop_reason, route=ROUTE_GO_LOCAL,
        reason=_amend_reason_for_write_failure(reason, write_result),
        runner_result=runner_result, bundle_path=write_result.path, elapsed_s=elapsed_s,
        bundle_write_ok=write_result.write_ok,
    )


def supervise(
    *,
    card_path: str,
    worktree: str,
    out_path: str,
    bundle_out_path: str,
    refinement_artifact_path: str,
    primary_receipt_path: str,
    card_hash: str,
    rri: int,
    wall_clock_seconds: int = MED_HIGH_WALL_CLOCK_SECONDS,
    popen_fn=subprocess.Popen,
    python_executable=None,
    diff_file: str | None = None,
    rri_table: str | None = None,
    fallback_mode: str = fallback_selection.MODE_HUMAN_SELECT,
    fallback_model: str | None = None,
    fallback_reasoning_effort: str | None = None,
    fallback_selected_by: str | None = None,
    fallback_selection_artifact: str | None = None,
) -> SupervisorResult:
    """Validate the Med-high route and emit its cloud handoff bundle.

    The refinement/receipt gate remains authoritative evidence. Local execution
    is policy-excluded for this band, including an otherwise-valid GO_LOCAL
    result, so this entry point never launches the local runner.
    """
    bundle_kwargs = dict(
        bundle_out_path=bundle_out_path, card_path=card_path, out_path=out_path,
        refinement_artifact_path=refinement_artifact_path,
        primary_receipt_path=primary_receipt_path, card_hash=card_hash,
        diff_file=diff_file, rri_table=rri_table,
    )

    try:
        decision = decide_route(
            refinement_artifact_path=refinement_artifact_path,
            primary_receipt_path=primary_receipt_path,
            card_hash=card_hash,
            rri=rri,
        )
    except med_high_gate.GateError as exc:
        result = _pre_launch_bundle(
            **bundle_kwargs,
            stop_reason=f"gate_error:{exc.code}", status="gate_rejected", reason=str(exc),
        )
    except GateInputError as exc:
        result = _pre_launch_bundle(
            **bundle_kwargs,
            stop_reason=f"gate_input_unreadable:{exc.artifact_label}",
            status="cloud_required", reason=str(exc),
        )
    else:
        result = None

    if result is None:
        if decision.route == ROUTE_GO_LOCAL:
            stop_reason = "policy_excluded_local_execution"
            reason = (
                "Med-high local execution is disabled: the owner limits the "
                "local developer to Low/S and Moderate/M."
            )
        else:
            stop_reason = "cloud_required"
            reason = decision.reason
        result = _pre_launch_bundle(
            **bundle_kwargs,
            stop_reason=stop_reason, status="cloud_required", reason=reason,
        )

    return _attach_fallback_selection(
        card_path=card_path, rri=rri, result=result,
        fallback_mode=fallback_mode, fallback_model=fallback_model,
        fallback_reasoning_effort=fallback_reasoning_effort,
        fallback_selected_by=fallback_selected_by,
        fallback_selection_artifact=fallback_selection_artifact,
    )


def parse_args(argv=None):
    import argparse

    parser = argparse.ArgumentParser(
        description="Supervise the one bounded Med-high Qwen27 attempt and emit a cloud evidence bundle on any non-success route (ADR-038 T4).",
    )
    parser.add_argument("--card", required=True)
    parser.add_argument("--worktree", required=True)
    parser.add_argument("--out", required=True, help="Path the runner writes its transcript artifact to.")
    parser.add_argument("--bundle-out", required=True, help="Path to write the escalation bundle to, if the route is not a local success.")
    parser.add_argument("--refinement-artifact", required=True)
    parser.add_argument("--primary-receipt", required=True)
    parser.add_argument("--card-hash", required=True)
    parser.add_argument("--rri", type=int, required=True)
    parser.add_argument("--wall-clock-seconds", type=int, default=MED_HIGH_WALL_CLOCK_SECONDS)
    parser.add_argument("--diff-file", default=None)
    parser.add_argument("--rri-table", default=None)
    fallback_selection.add_cli_arguments(parser)
    return parser.parse_args(argv)


def main(argv=None) -> int:
    args = parse_args(argv)
    result = supervise(
        card_path=args.card,
        worktree=args.worktree,
        out_path=args.out,
        bundle_out_path=args.bundle_out,
        refinement_artifact_path=args.refinement_artifact,
        primary_receipt_path=args.primary_receipt,
        card_hash=args.card_hash,
        rri=args.rri,
        wall_clock_seconds=args.wall_clock_seconds,
        diff_file=args.diff_file,
        rri_table=args.rri_table,
        fallback_mode=args.fallback_mode,
        fallback_model=args.fallback_model,
        fallback_reasoning_effort=args.fallback_reasoning_effort,
        fallback_selected_by=args.fallback_selected_by,
        fallback_selection_artifact=args.fallback_selection_artifact,
    )
    print(json.dumps({
        "status": result.status,
        "route": result.route,
        "reason": result.reason,
        "bundle_path": result.bundle_path,
        "elapsed_s": result.elapsed_s,
        "fallback_selection_artifact": result.fallback_selection_artifact,
        "fallback_selection": result.fallback_selection,
        "cloud_instruction": result.cloud_instruction,
    }))
    if result.status == STATUS_SUCCESS:
        return 0
    if result.fallback_selection is None or result.fallback_selection["status"] == "blocked":
        return 2
    if result.fallback_selection["status"] == fallback_selection.AWAITING_STATUS:
        return fallback_selection.HUMAN_SELECTION_EXIT_CODE
    return 1


if __name__ == "__main__":
    sys.exit(main())
