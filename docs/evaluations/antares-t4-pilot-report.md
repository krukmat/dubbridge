---
type: Evaluation
title: "Antares T4 — observe-only workflow pilot mechanics"
status: complete
date: 2026-08-06
plan: docs/plan/antares-local-runtime-adoption.md
---

# Antares T4 — Observe-Only Workflow Pilot Mechanics

## Result

Implements the three non-blocking touchpoints named in the task ledger
(refinement, post-implementation, post-CI) as one shared entrypoint,
`scripts/antares/pilot.py::run_pilot_touchpoint`, plus a durable
disposition ledger (`scripts/antares/disposition_ledger.py`) that tracks
every candidate to a human disposition without ever gating approval,
review, or CI truth. This document records the pilot's operational
contract; fixed parameters (window, sample, thresholds) are recorded in
`docs/evaluations/antares-t4-calibration-report.md` to avoid duplicating
them here.

## Touchpoints

| Touchpoint | Caller | Invocation surface |
|---|---|---|
| Refinement | Primary security advisor, during task-analysis review, before implementation starts | `run_pilot_touchpoint(touchpoint=Touchpoint.REFINEMENT, ...)` against the baseline snapshot |
| Post-implementation | Primary security advisor, after implementation, before closure | `run_pilot_touchpoint(touchpoint=Touchpoint.POST_IMPLEMENTATION, ...)` against the candidate snapshot |
| Post-CI | `.github/workflows/push-review.yml`'s `Antares post-CI observe-only pilot (T4)` step | `scripts/antares/post_ci_summary.py` against the exact completed revision (`github.event.workflow_run.head_sha`) |

Every touchpoint call requires an explicit `cwe_id` already present on the
T3a watchlist (`scripts/antares/cwe_watchlist.py`). There is no
"sweep every CWE" or "run without a CWE" mode; a caller with no eligible
CWE for a given touchpoint calls `skip_pilot_touchpoint` instead, which
records a typed, non-empty reason and never invokes Antares (EC-2).

## Disposition ledger contract

- Every non-degraded touchpoint call creates one `LedgerEntry` per
  candidate file, always starting in `DispositionState.NEEDS_HUMAN_REVIEW`
  (`disposition_ledger._validate_new_entry` rejects any attempt to create
  an entry pre-closed).
- `DispositionLedger.disposition()` is a one-way transition: an entry that
  already carries a disposition cannot be re-opened or overwritten
  (`already_dispositioned`), and `ACCEPTED_FOLLOW_UP` requires a non-empty
  `follow_up_ref` linking to a task/refinement record.
- `rejected` is recorded only via a human-supplied `reviewer` and
  `reviewed_at`; the ledger has no automatic path to `REJECTED` and never
  infers it from a metric. `calibration.py`'s ground-truth-backed metrics
  are the only source that may support a false-positive/precision claim
  (task ledger acceptance criterion).
- `DispositionLedger.backlog(now=...)` returns undisposed entries whose
  `sla_deadline` has passed; they remain in the ledger at
  `NEEDS_HUMAN_REVIEW` and are reported as backlog to the named
  `triage_owner`, never silently closed (EC-3).
- `compute_dedup_key(cwe_id, candidate_file, snapshot_id)` is deterministic
  and touchpoint-independent, so the same underlying candidate surfacing
  at both refinement and post-implementation for the same snapshot
  collapses to the same `dedup_key`; `DispositionLedger.duplicates_of`
  exposes the group for deduplication-rate reporting.

## Degraded-run handling (EC-1)

`run_pilot_touchpoint` catches every exception raised by its `dispatch`
callable (default `harness.dispatch_via_cli`) and any non-success
`TerminalState.kind` (e.g. `cli_binary_unavailable`,
`cli_execution_failed`, `cli_output_malformed`), converting both into a
`PilotRunResult(degraded=True, candidates=(), ledger_entries=())`. A
degraded result is never turned into a ledger entry
(`record_pilot_run` is a no-op for it) and never raises out of the
function — the caller's own CI/workflow state is unaffected by
construction, not by convention.

`scripts/antares/post_ci_summary.py::main` always returns 0 regardless of
how many of the watchlist's CWEs degraded during the run; the redacted
summary's `degraded_count` field is the only signal a reader needs to see
that some CWEs did not complete cleanly.

## Operational metrics available from the artifacts this task produces

| Metric | Source |
|---|---|
| Volume | `len(DispositionLedger.entries)` per window, or `post-ci summary.total_candidates` per CI run |
| Dispositions (by state) | Group `DispositionLedger.entries` by `.state` |
| SLA age | `DispositionLedger.backlog(now=...)` for past-SLA; `sla_deadline - created_at` for in-flight age |
| Deduplication rate | `len(duplicates_of(k)) - 1` summed over distinct `dedup_key` values, divided by total entries |
| Triage time | `reviewed_at - created_at` per dispositioned entry (both ISO-8601 UTC strings, directly diffable) |
| Accepted-follow-up conversion | Count of `ACCEPTED_FOLLOW_UP` entries with a resolved `follow_up_ref` |
| Runtime | `PilotRunResult` does not itself carry elapsed time; the underlying `TerminalState.elapsed_seconds` from `dispatch_via_cli` is available to a caller that also persists the raw `Artifact` via `cli_terminal_state_to_artifact` |
| Resource cost | Not computed by this task; deferred to a future refinement that wires `sandbox_budget.py`'s existing resource accounting into the pilot path if operational cost tracking proves necessary |

## Live pilot run results

Not yet populated -- this task implements the pilot mechanism and fixes
its operating parameters; it does not itself constitute the 30-day pilot
window defined in `docs/evaluations/antares-t4-calibration-report.md`.
The first live touchpoint invocation starts that window. T5 (promote,
narrow, or retire) consumes this section once populated.
