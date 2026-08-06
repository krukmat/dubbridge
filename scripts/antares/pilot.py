"""Observe-only workflow pilot runner (T4).

Invokes Antares (via `harness.dispatch_via_cli`) at the three non-blocking
touchpoints named in the task ledger -- refinement, post-implementation, and
post-CI -- and records every candidate into a `DispositionLedger` entry.
This module never gates, delays, or otherwise participates in approval, the
band-routed review chain, CI's actual pass/fail truth, or task closure: it
only observes and records (see docs/policies/HITL_AUTONOMY_POLICY.md and
docs/playbooks/AGENT_WORKFLOW_GUIDE.md's Antares authority-boundary
sections).

EC-1: any runtime failure (missing binary, execution failure, malformed
output, or an unhandled exception raised while invoking the harness) is
caught here and converted into a `PilotRunResult` carrying a degraded
`TerminalState`-derived status -- it is never allowed to propagate out of
`run_pilot_touchpoint` and affect the caller's own CI/workflow state.

EC-2: `run_pilot_touchpoint` requires an explicit `cwe_id` already present
on the T3a watchlist; there is no "run over every CWE" or "run without a
CWE" mode. A caller with no eligible CWE for the touchpoint at hand must
call `skip_pilot_touchpoint` instead, which records a typed skip and never
invokes Antares.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

from scripts.antares.cwe_watchlist import load_watchlist
from scripts.antares.disposition_ledger import (
    DispositionLedger,
    LedgerEntry,
    Touchpoint,
    compute_dedup_key,
    utc_now_iso,
)
from scripts.antares.harness import dispatch_via_cli


class PilotError(ValueError):
    """A fail-closed rejection with a stable machine-readable code, raised
    only for caller-input problems decided before any Antares invocation --
    runtime failures during invocation itself are EC-1 degraded results, not
    exceptions."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class PilotRunResult:
    """The outcome of one `run_pilot_touchpoint` call.

    `degraded` is True whenever the underlying Antares invocation did not
    complete cleanly (EC-1); `candidates` is always empty in that case and
    `detail` carries the reason. `ledger_entries` is empty for a degraded
    result -- a failed run produces no candidates to triage.
    """

    touchpoint: Touchpoint
    cwe_id: str
    snapshot_id: str
    degraded: bool
    detail: str
    candidates: tuple[str, ...]
    ledger_entries: tuple[LedgerEntry, ...]


@dataclass(frozen=True)
class PilotSkipResult:
    """EC-2: the typed record produced when a touchpoint has no eligible
    CWE. Never carries candidates or ledger entries -- a skip is not a
    degraded run, it is the absence of a run."""

    touchpoint: Touchpoint
    reason: str
    detail: str


_DEFAULT_SLA_HOURS = 72


def _sla_deadline_from(created_at: str, *, sla_hours: int) -> str:
    from datetime import datetime, timedelta, timezone

    created = datetime.strptime(created_at, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    deadline = created + timedelta(hours=sla_hours)
    return deadline.strftime("%Y-%m-%dT%H:%M:%SZ")


def skip_pilot_touchpoint(touchpoint: Touchpoint, *, reason: str) -> PilotSkipResult:
    """EC-2: record that `touchpoint` was skipped because no eligible CWE
    was available. `reason` must be non-empty -- a forced generic sweep to
    manufacture pilot volume is forbidden, and an unexplained skip is
    indistinguishable from that at review time."""
    if not reason.strip():
        raise PilotError(
            "empty_skip_reason",
            "skip_pilot_touchpoint requires a non-empty reason; skips must be "
            "explainable, not silent.",
        )
    return PilotSkipResult(
        touchpoint=touchpoint,
        reason=reason.strip(),
        detail=f"No eligible CWE for {touchpoint.value} touchpoint: {reason.strip()}",
    )


def run_pilot_touchpoint(
    *,
    touchpoint: Touchpoint,
    cwe_id: str,
    snapshot_root: Path,
    snapshot_id: str,
    triage_owner: str,
    sla_hours: int = _DEFAULT_SLA_HOURS,
    dispatch: Callable[..., Any] = dispatch_via_cli,
) -> PilotRunResult:
    """Invoke Antares once for `cwe_id` against `snapshot_root` and record
    every returned candidate as a `LedgerEntry`.

    `dispatch` defaults to `harness.dispatch_via_cli` but is injectable so
    callers (and tests) can supply a stub without touching the real
    `antares-cli` subprocess boundary -- consistent with how
    `harness_test.py` stubs `subprocess.Popen` rather than the CLI binary
    itself.

    HP-2: this function only observes; it does not consult, block on, or
    modify approval state, the review chain, or CI truth. EC-1: any
    exception raised by `dispatch`, or a `TerminalState` whose `kind` is not
    the CLI success kind, produces a degraded `PilotRunResult` with no
    ledger entries -- it never raises out of this function.
    """
    watchlist = load_watchlist()
    entry = watchlist.get(cwe_id)
    if entry is None:
        raise PilotError(
            "cwe_not_on_watchlist",
            f"{cwe_id!r} is not present in the T3a watchlist; run_pilot_touchpoint "
            "requires an explicit, already-justified CWE -- call "
            "skip_pilot_touchpoint instead if none is eligible.",
        )
    if not triage_owner.strip():
        raise PilotError(
            "empty_triage_owner", "run_pilot_touchpoint requires a non-empty triage_owner."
        )

    request = {"target": ".", "cwe_ids": [cwe_id]}
    try:
        state = dispatch(request, snapshot_root=snapshot_root)
    except Exception as exc:  # noqa: BLE001 - EC-1: any failure degrades, never propagates
        return PilotRunResult(
            touchpoint=touchpoint,
            cwe_id=cwe_id,
            snapshot_id=snapshot_id,
            degraded=True,
            detail=f"dispatch raised {type(exc).__name__}: {exc}",
            candidates=(),
            ledger_entries=(),
        )

    kind_name = getattr(getattr(state, "kind", None), "value", None)
    if kind_name != "cli_execution_complete":
        return PilotRunResult(
            touchpoint=touchpoint,
            cwe_id=cwe_id,
            snapshot_id=snapshot_id,
            degraded=True,
            detail=getattr(state, "detail", f"non-success terminal state: {kind_name!r}"),
            candidates=(),
            ledger_entries=(),
        )

    candidates: tuple[str, ...] = tuple(getattr(state, "candidates", ()) or ())
    created_at = utc_now_iso()
    sla_deadline = _sla_deadline_from(created_at, sla_hours=sla_hours)
    ledger_entries = tuple(
        LedgerEntry(
            entry_id=f"{touchpoint.value}:{cwe_id}:{snapshot_id}:{index}",
            touchpoint=touchpoint,
            cwe_id=cwe_id,
            candidate_file=candidate_file,
            dedup_key=compute_dedup_key(
                cwe_id=cwe_id, candidate_file=candidate_file, snapshot_id=snapshot_id
            ),
            created_at=created_at,
            triage_owner=triage_owner,
            sla_deadline=sla_deadline,
        )
        for index, candidate_file in enumerate(candidates)
    )
    return PilotRunResult(
        touchpoint=touchpoint,
        cwe_id=cwe_id,
        snapshot_id=snapshot_id,
        degraded=False,
        detail="",
        candidates=candidates,
        ledger_entries=ledger_entries,
    )


def record_pilot_run(ledger: DispositionLedger, result: PilotRunResult) -> None:
    """Add every ledger entry from a non-degraded `PilotRunResult` into
    `ledger`. A degraded result has no entries and this is a no-op for it --
    callers may call this unconditionally after `run_pilot_touchpoint`."""
    for entry in result.ledger_entries:
        ledger.add(entry)
