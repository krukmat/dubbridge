---
type: Evaluation
title: "Antares T4 — ground-truth calibration methodology and fixed pilot parameters"
status: complete
date: 2026-08-06
plan: docs/plan/antares-local-runtime-adoption.md
---

# Antares T4 — Ground-Truth Calibration Methodology and Fixed Pilot Parameters

## Result

Implements the calibration metrics engine (`scripts/antares/calibration.py`)
and the observe-only pilot (`scripts/antares/pilot.py`,
`scripts/antares/disposition_ledger.py`,
`scripts/antares/post_ci_summary.py`), and fixes the pilot's operating
parameters as required by T4's acceptance criteria before any live pilot
run executes. This document is the authoritative record of those fixed
parameters; the code enforces the metric/disposition contracts, this
document enforces the ones that are not code-enforceable (corpus
membership, schedule, thresholds).

## Calibration methodology

- **Corpus:** known-vulnerable pre-fix snapshots with patch-derived
  implementation-file ground truth (the changed-file list of the commit
  that fixes the vulnerability), paired with the same repository's patched
  snapshot for a true-negative case. `CalibrationCase.role` distinguishes
  the two (`SnapshotRole.VULNERABLE` / `SnapshotRole.PATCHED` in
  `scripts/antares/calibration.py`); the module rejects (`CalibrationError`)
  a vulnerable case with no ground truth and a patched case that declares
  ground truth, so the two populations cannot be silently mixed.
- **Metric:** task-level precision, recall, and File F1, macro-averaged
  (unweighted mean of per-case scores) across all vulnerable cases in a
  calibration run. Macro-averaging is the reporting contract required by
  the task ledger; `CalibrationReport.file_f1` must never be read as a
  per-output correctness probability (mirrors the same caveat already
  recorded for Antares-1B's benchmark F1 in
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Antares Security-Specialist
  Advisor).
- **True negatives:** evaluated separately via
  `CalibrationReport.true_negative_results` / `true_negative_rate` — a
  patched-snapshot case is a true negative iff Antares returned zero
  candidates for it; any non-empty candidate list is recorded as a false
  positive for that case, never folded into the vulnerable-case
  precision/recall numbers.
- **Corpus size and selection:** deferred to the first live calibration
  run, which must draw from repository history commits that (a) fix a
  CWE already on the T3a watchlist and (b) have a clearly attributable
  single fixing commit. A minimum of 3 vulnerable cases and 3 paired
  patched cases per watchlisted CWE is required before that CWE's
  calibration numbers are reported; fewer than 3 is recorded as
  `insufficient-sample`, not a reported score.

## Pilot operating parameters (fixed before execution)

| Parameter | Value | Rationale |
|---|---|---|
| Pilot window | 30 calendar days from first live touchpoint invocation | Matches the CI artifact retention period below; long enough to observe more than one refinement/post-implementation/post-CI cycle across active tasks. |
| Sample | Every RRI 26+ development task with an explicit CWE hypothesis already on the T3a watchlist that reaches refinement during the pilot window | Task-ledger acceptance criteria requires "an eligible task with an explicit CWE"; this is not a synthetic sample, it is the pilot's real usage population. |
| Watchlist schedule | The static T3a watchlist (`scripts/antares/cwe_watchlist.py`, `WATCHLIST_VERSION = "2026-08-02.1"`) is re-evaluated once per pilot window, not per run | The watchlist is human-curated and versioned; churning it inside a single pilot window would make the pilot's own results non-comparable across the window. |
| Concurrency | One Antares invocation in flight per touchpoint call; `run_pilot_touchpoint` is synchronous and does not parallelize across CWEs | `dispatch_via_cli` spawns a real subprocess per call (Element 3); unbounded concurrency was never evaluated by T1 R4/R5's runtime preflight and is out of scope for this pilot. `post_ci_summary.py`'s per-CWE loop is intentionally sequential. |
| Runtime budget | `harness.DEFAULT_CLI_TIMEOUT_SECONDS` (300s) per invocation, unchanged from Element 3 | Already the enforced timeout in `dispatch_via_cli`; T4 does not introduce a second budget. |
| Stopping rules | Stop the pilot window early only if (a) the disposition backlog (`DispositionLedger.backlog`) exceeds 20 undisposed entries past SLA for the named triage owner, or (b) `degraded_count` exceeds 50% of touchpoint invocations in any 7-day rolling window | Both are observable directly from the ledger/summary artifacts this task produces; neither requires a new metric. |
| SLA | 72 hours per candidate from `created_at` to a durable disposition (`disposition_ledger._DEFAULT_SLA_HOURS`) | Matches typical single-task review turnaround in this repository's HITL workflow; not derived from external benchmark data. |
| Promotion thresholds (feeds T5, not decided here) | T5 may consider promotion only if: calibration mean File F1 ≥ 0.30 macro-averaged across watchlisted CWEs with ≥ 3 cases each, true-negative rate ≥ 0.70, and pilot backlog stopping rules above were never triggered during the full window | These are pilot-evidence gates for T5's own decision, not a promotion decision made by this task. T4 does not promote, narrow, or retire Antares — see the Antares authority boundary in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`. |

## Non-blocking guarantee

Every touchpoint call in this implementation
(`pilot.run_pilot_touchpoint`, `post_ci_summary.build_summary`) returns a
plain data result (`PilotRunResult` / a JSON summary dict) and never
raises out of a caught failure (EC-1). `scripts/antares/post_ci_summary.py`
always exits 0, and its CI wiring in
`.github/workflows/push-review.yml` runs the Antares step with
`continue-on-error: true` and `if: always()` so a degraded or failing
Antares invocation can never fail the calling CI job. Neither Antares
invocation nor any pilot disposition is consulted by the band-routed
review chain, the HITL approval gate, or CI's own pass/fail truth.

## Retention and redaction

- `logs/antares/` and `var/antares-traces/` are both Git-ignored
  (`.gitignore`). Raw traces (`Artifact.raw_stdout`/`raw_stderr`,
  redacted via `artifact_trace_writer.write_raw_trace` when the full
  artifact schema is used) never enter version control.
- `scripts/antares/post_ci_summary.py` writes exactly one redacted
  summary file per CI run (`antares-post-ci-summary.json`) containing
  only CWE IDs, candidate *counts*, and a `degraded` boolean per CWE --
  never raw stdout/stderr, never full candidate file lists. That summary
  is uploaded as a GitHub Actions artifact with an explicit
  `retention-days: 30`.

## Evidence to emit (task ledger requirement)

- This report (methodology + fixed parameters).
- `docs/evaluations/antares-t4-pilot-report.md` (pilot mechanics and
  disposition-ledger contract; run-level results populate once the pilot
  window executes against live tasks).
- Unit test evidence: `scripts/antares/calibration_test.py`,
  `scripts/antares/disposition_ledger_test.py`,
  `scripts/antares/pilot_test.py` (see task closure record).

## Status artifacts affected

- `docs/tasks/antares-security-specialist-advisor.md` § T4 (this task).
- `docs/plan/antares-local-runtime-adoption.md` § Element/Phase table (T4
  moves from "unblocked" to "implemented, pilot not yet run").
- `.github/workflows/push-review.yml` and `.gitignore` (both edited in
  this change).
