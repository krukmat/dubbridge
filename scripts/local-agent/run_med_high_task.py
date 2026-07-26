#!/usr/bin/env python3
"""Med-high hard-timeout supervisor and cloud evidence bundle (ADR-038 T4).

Owns the two things ADR-038 section 4 assigns to the primary, not to the
local implementer itself: (1) the hard 300-second wall-clock cutoff on the
one bounded Qwen35 attempt, enforced by killing the *entire* process group
`run_local_task.py` spawns (not just its immediate PID) so a stuck subprocess
`run_local_task.py` itself launched (e.g. a hung `cargo test`) cannot survive
the cutoff; and (2) emitting the complete ADR-038 section 5 evidence bundle on
every non-success route, so cloud continuation never starts from a blank slate.

This module never edits code and never re-implements the runner's own turn
loop, model binding, or repair budget -- that enforcement lives in
`run_local_task.py` (ADR-038 T3) and is invoked here strictly as an external,
killable subprocess. Per the handoff hard rule, the not-yet-enforced local
runner is not used to implement its own enforcement: the 300-second cutoff
and process-group kill are supervisor-level OS controls, external to the
runner process, not cooperative behavior requested of it.
"""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import time
from dataclasses import dataclass
from typing import Any

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import escalation_packet
import med_high_gate

MED_HIGH_WALL_CLOCK_SECONDS = 300
MED_HIGH_RUNNER_MODEL = "qwen3.6:35b-a3b"
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


def _load_json(path: str) -> dict[str, Any]:
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def decide_route(
    *, refinement_artifact_path: str, primary_receipt_path: str, card_hash: str, rri: int
) -> med_high_gate.GateDecision:
    """Thin wrapper over the T2 gate: pure, offline, fail-closed. Any read or
    validation failure is surfaced as CLOUD_REQUIRED by the caller, never as
    an uncaught exception that could be mistaken for a crash."""
    refinement_artifact = _load_json(refinement_artifact_path)
    primary_receipt = _load_json(primary_receipt_path)
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
        process.wait()
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


def _read_runner_out(out_path: str) -> dict[str, Any] | None:
    if not os.path.isfile(out_path):
        return None
    try:
        return _load_json(out_path)
    except (OSError, json.JSONDecodeError):
        return None


def build_evidence_bundle(
    *,
    bundle_out_path: str,
    card_path: str,
    runner_out_path: str,
    stop_reason: str,
    refinement_artifact_path: str | None = None,
    primary_receipt_path: str | None = None,
    effective_limits: dict[str, Any] | None = None,
    diff_file: str | None = None,
    rri_table: str | None = None,
    card_hash: str | None = None,
) -> str:
    """Assemble the ADR-038 section 5 cloud-escalation bundle.

    Reuses escalation_packet.build_packet for the seven ADR-036 sections
    (task spec/RRI, plan, allowed paths, diff, commands, tests, per-attempt
    summaries) verbatim, then appends the four ADR-038-specific sections the
    section-5 evidence list requires beyond ADR-036's original scope: the
    refinement artifact, the primary receipt, the effective limits, and an
    explicit stop-reason/hash/model-identity/elapsed-time footer. Every
    missing optional input renders literal "MISSING" text, the same
    fail-visible convention escalation_packet.py already uses, rather than
    silently omitting a section cloud continuation might need.
    """
    card = escalation_packet.load_card(card_path)
    runner_result = _read_runner_out(runner_out_path) or {}
    diff_text = escalation_packet.read_text_file(diff_file)
    rri_table_text = escalation_packet.resolve_rri_table(rri_table)

    base_packet = escalation_packet.build_packet(card, runner_result, diff_text, rri_table_text)

    refinement_artifact = (
        _load_json(refinement_artifact_path)
        if refinement_artifact_path and os.path.isfile(refinement_artifact_path)
        else None
    )
    primary_receipt = (
        _load_json(primary_receipt_path)
        if primary_receipt_path and os.path.isfile(primary_receipt_path)
        else None
    )

    extra_sections = [
        (
            "8. Refinement artifact (Qwen27)",
            json.dumps(refinement_artifact, indent=2, sort_keys=True)
            if refinement_artifact is not None
            else escalation_packet.MISSING,
        ),
        (
            "9. Primary route receipt",
            json.dumps(primary_receipt, indent=2, sort_keys=True)
            if primary_receipt is not None
            else escalation_packet.MISSING,
        ),
        (
            "10. Effective limits",
            json.dumps(effective_limits, indent=2, sort_keys=True)
            if effective_limits is not None
            else escalation_packet.MISSING,
        ),
        (
            "11. Stop reason and hashes",
            (
                f"Stop reason: `{stop_reason}`\n\n"
                f"Card hash: `{card_hash or escalation_packet.MISSING}`\n\n"
                f"Refinement artifact SHA-256: `{med_high_gate.sha256_of(refinement_artifact) if refinement_artifact is not None else escalation_packet.MISSING}`\n\n"
                f"Runner model: `{runner_result.get('model', escalation_packet.MISSING)}`\n\n"
                f"Runner status: `{runner_result.get('status', escalation_packet.MISSING)}`"
            ),
        ),
    ]

    parts = [base_packet.rstrip("\n")]
    for title, body in extra_sections:
        parts.append(f"\n\n## {title}\n\n{body}\n")
    full_packet = "".join(parts)

    with open(bundle_out_path, "w", encoding="utf-8") as f:
        f.write(full_packet)
    return bundle_out_path


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
) -> SupervisorResult:
    """The single entry point: decide route (T2 gate), then either hand off
    to cloud immediately (HP-2) or supervise exactly one bounded Qwen35
    attempt (HP-1) and emit a complete evidence bundle on any non-success
    outcome (EC-1, EC-2)."""
    try:
        decision = decide_route(
            refinement_artifact_path=refinement_artifact_path,
            primary_receipt_path=primary_receipt_path,
            card_hash=card_hash,
            rri=rri,
        )
    except med_high_gate.GateError as exc:
        bundle_path = build_evidence_bundle(
            bundle_out_path=bundle_out_path,
            card_path=card_path,
            runner_out_path=out_path,
            stop_reason=f"gate_error:{exc.code}",
            refinement_artifact_path=refinement_artifact_path,
            primary_receipt_path=primary_receipt_path,
            card_hash=card_hash,
            diff_file=diff_file,
            rri_table=rri_table,
        )
        return SupervisorResult(
            status="gate_rejected", route=ROUTE_CLOUD_REQUIRED, reason=str(exc),
            runner_result=None, bundle_path=bundle_path, elapsed_s=0.0,
        )

    if decision.route == ROUTE_CLOUD_REQUIRED:
        # HP-2: routes directly to cloud without ever launching Qwen35.
        bundle_path = build_evidence_bundle(
            bundle_out_path=bundle_out_path,
            card_path=card_path,
            runner_out_path=out_path,
            stop_reason="cloud_required",
            refinement_artifact_path=refinement_artifact_path,
            primary_receipt_path=primary_receipt_path,
            card_hash=card_hash,
            diff_file=diff_file,
            rri_table=rri_table,
        )
        return SupervisorResult(
            status="cloud_required", route=ROUTE_CLOUD_REQUIRED, reason=decision.reason,
            runner_result=None, bundle_path=bundle_path, elapsed_s=0.0,
        )

    launch_outcome = run_supervised_runner(
        card_path=card_path, worktree=worktree, out_path=out_path,
        model=MED_HIGH_RUNNER_MODEL,
        wall_clock_seconds=wall_clock_seconds, popen_fn=popen_fn,
        python_executable=python_executable,
    )
    elapsed_s = launch_outcome.get("elapsed_s", 0.0)
    runner_result = _read_runner_out(out_path)

    if launch_outcome["status"] == "runner_exited" and runner_result is not None and runner_result.get("status") == STATUS_SUCCESS:
        # HP-1: exactly one exact-model runner launched, success recorded
        # without escalation -- no bundle is built on this path.
        return SupervisorResult(
            status=STATUS_SUCCESS, route=ROUTE_GO_LOCAL, reason="Med-high session succeeded within budget.",
            runner_result=runner_result, bundle_path=None, elapsed_s=elapsed_s,
        )

    stop_reason = launch_outcome["status"]
    if launch_outcome["status"] == "runner_exited" and runner_result is not None:
        stop_reason = runner_result.get("status", stop_reason)

    effective_limits = (
        (runner_result or {}).get("effective_limits")
        or (runner_result or {}).get("audit_effective_limits")
    )
    bundle_path = build_evidence_bundle(
        bundle_out_path=bundle_out_path,
        card_path=card_path,
        runner_out_path=out_path,
        stop_reason=stop_reason,
        refinement_artifact_path=refinement_artifact_path,
        primary_receipt_path=primary_receipt_path,
        effective_limits=effective_limits,
        card_hash=card_hash,
        diff_file=diff_file,
        rri_table=rri_table,
    )
    return SupervisorResult(
        status=stop_reason, route=ROUTE_GO_LOCAL, reason=launch_outcome.get("reason", stop_reason),
        runner_result=runner_result, bundle_path=bundle_path, elapsed_s=elapsed_s,
    )


def parse_args(argv=None):
    import argparse

    parser = argparse.ArgumentParser(
        description="Supervise the one bounded Med-high Qwen35 attempt and emit a cloud evidence bundle on any non-success route (ADR-038 T4).",
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
    )
    print(json.dumps({
        "status": result.status,
        "route": result.route,
        "reason": result.reason,
        "bundle_path": result.bundle_path,
        "elapsed_s": result.elapsed_s,
    }))
    return 0 if result.status == STATUS_SUCCESS else 1


if __name__ == "__main__":
    sys.exit(main())
