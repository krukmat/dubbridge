---
type: Plan
title: "Plan: ADR-036 Quality Metrics — Measurement Layer + Harness Remediation"
status: proposed
slice: adr036-quality-metrics
governed_by: [ADR-036]
---

# Plan: ADR-036 Quality Metrics — Measurement Layer + Harness Remediation

## Objective

Before any Stage 1 go/no-go report (`T8` in `docs/tasks/adr036-local-first-pilot.md`)
is attempted, make the ADR-036 pilot's own quality signal trustworthy: fix the
audit trail that is supposed to show whether the local-first approach is
succeeding over time, close the two diagnosed harness defects that corrupt
individual session outcomes, and resolve the 16-card benchmark corpus's
validity problem. This plan does not attempt T8 itself — it is the
precondition work T8 is currently blocked on.

## Context

`T7f` (2026-07-15) was the first genuinely real, non-corpus task run through
the pilot stack (`qwen3.6:35b-a3b` via `run_local_task.py`) and it succeeded —
useful evidence, but a single data point. While closing it out, checking
`logs/gemma-audit/*.jsonl` to see whether the audit layer could show a
success-rate trend surfaced a more basic problem: the measurement layer
itself cannot currently answer "is this working," independent of any one
session's result. `T7d`/`T7e` (the 16-card corpus runs) were already closed
as diagnostic-only, not valid promotion evidence, for related reasons
(corpus validity, scope-check false positives — the scope-check one fixed by
`T7f`). This plan generalizes that same "fix the instrument before trusting
its reading" discipline to the remaining known gaps.

Priority order below reflects dependency, not just severity: Priority 1 has
to be fixed first because it is what would let anyone tell, later, whether
fixing Priorities 2 and 3 actually helped.

## Priority 1 — Make the audit trail trustworthy

**Problem, with evidence:** `logs/gemma-audit/*.jsonl` already carries 1376
`role=local-implementer` records, which looks like a rich longitudinal
dataset — but it is not currently usable as one:

- The unit test suite writes real entries into the production audit log on
  every run. A single `pytest scripts/local-agent/` invocation during T7f's
  closeout added 42 new records with `task_id` values like `toy-1` /
  `integration-1` (test fixtures), indistinguishable from genuine sessions by
  `role` or `model`. Of the 1332 records logged on 2026-07-12 alone, most are
  plausibly the same kind of test-run noise accumulated during T6/T7 harness
  development, not real Ollama-backed sessions — there is no field that
  separates the two today.
- `scripts/gemma-audit-report.py --role` only accepts
  `{developer, reviewer, all}`. It has no way to select just
  `local-implementer` records, so there is no clean query for "how is the
  ADR-036 pilot doing" without ad hoc one-off scripting.
- `build_audit_record()` (`run_local_task.py`) always sets `rri` and `band`
  to `None`. Even once the two problems above are fixed, there is no way to
  ask "does success rate hold up as task complexity increases" from this
  data.

**Scope:**
- Isolate every `run_local_task_test.py` / `integration_test.py` test class
  that calls `main()` from the real `gemma_local.append_audit_log` /
  `logs/gemma-audit/` path — patch it (or redirect the log directory to a
  temp path) uniformly, not just in the tests that already target audit
  emission specifically.
- Extend `gemma-audit-report.py --role` to accept `local-implementer` (and
  reserve room for a future `local-reviewer`, for when Gemma plays T7e's
  comparator role), or relax the flag to accept any string.
- Thread `rri`/`band` into `build_audit_record()` when the calling task card
  carries them, instead of hardcoding `None`.
- Decide what to do with the historical contamination already in
  `logs/gemma-audit/2026-06.jsonl` / `2026-07.jsonl` (flag/filter vs. leave
  as known-noisy history) — this is dev-tooling log hygiene, not the
  ADR-018 durable production audit boundary, so it is not subject to that
  invariant, but it should not be rewritten silently either.

**Outputs:** test suite no longer writes to the real audit log; a report
command that can isolate ADR-036 pilot sessions cleanly; audit records that
carry task complexity so success rate can eventually be read against RRI
band, not just in aggregate.

## Priority 2 — Close two diagnosed harness defects

Both already root-caused (T7e disposition); neither has a dependency on
Priority 1 or 3, so either can be picked up independently once scheduled.

1. **`read_file` on a directory aborts the whole session.** Currently raises
   `BoundaryViolation` (via an `OSError` catch-all in
   `apply_tool_call`/`read_file` handling) and ends the run immediately.
   Should return a structured, recoverable tool-error result instead (the
   same shape as a missing-file `MalformedToolCall`), since an agent
   accidentally pointing `read_file` at a directory is ordinary model error,
   not a containment breach.
2. **`gemma_local.py:27`'s `DEFAULT_NUM_PREDICT = 4096` is too small for
   Gemma.** Implicated in all 4 `TRANSPORT_ERROR` (mid-JSON truncation)
   outcomes in T7e. Needs a larger budget for the Gemma-as-implementer/
   comparator path specifically (raising it globally affects Qwen's budget
   and cost too, so this should be model-aware, not a single constant bump).

**Outputs:** both fixes with dedicated regression tests reproducing the
original failure mode, following the same pattern `T7d-fix` and `T7f` used.

## Priority 3 — Decide the fate of the 16-card benchmark corpus

**Problem:** all 16 cards' `reference_commit`s are already merged into
`HEAD`; `setup_worktree` never reverts them, so several cards' described
bugs do not exist when a session starts (confirmed for `RC-01`, `CC-03`).
Any corpus-based rerun inherits this validity problem regardless of how many
harness bugs get fixed above it.

**Decision (resolved by T7j, 2026-07-15): retire the corpus as promotion /
comparison evidence.**

Why this won:

- Revalidation in T7j confirmed the core validity problem still holds now:
  all 16 `reference_commit`s remain ancestors of `HEAD`, so the corpus does
  not currently represent reproducible pre-fix states.
- Repair cost is not "rerun after harness fixes"; it is effectively corpus
  regeneration. Several cards point at small one-file fixes, but the missing
  input is not the patch itself — it is a fresh, currently unfixed issue plus
  a new trustworthy `reference_commit`, acceptance contract, and verification
  story.
- The highest-value signal is already drifting away from this corpus shape:
  `T7f` produced genuine real-task evidence, while `T7d`/`T7e` mostly measured
  harness defects and historical-card drift. More `T7f`-style real tasks are a
  cheaper and more decision-relevant path to `T8` than rebuilding a synthetic
  16-card benchmark around stale commits.

**Consequence:** the 16-card corpus is retained only as harness-regression /
forensics material. It is no longer a valid source for Stage 1 promotion data
or Qwen-vs-Gemma capability comparison.

**Options considered during T7j:**
1. **Repair the corpus** — regenerate cards against real, currently-unfixed
   issues (or re-derive `reference_commit`s so `setup_worktree` can revert to
   a genuine pre-fix state), so a 16-card rerun becomes valid comparison
   evidence again.
2. **Retire the corpus** — treat single, well-scoped real-task trials like
   `T7f` as the primary evidence going forward, and use the corpus only for
   harness regression testing (not promotion/comparison signal), formalizing
   what T7d/T7e's disposition already did in practice.

T7j resolved this in favor of option 2. A future corpus-rebuild effort would
be a new slice with new cards, not a silent resurrection of these 16 entries.

## Relationship to T8

`T8` (`docs/tasks/adr036-local-first-pilot.md`) stays `[ ] Blocked` until at
least Priority 1 is done (otherwise T8 would be written from the same kind of
untrustworthy signal T7d/T7e already were). Priority 3 is now resolved in
favor of retirement, so the remaining evidence path is: trustworthy audit
trail plus additional `T7f`-style real-task trials, not a repaired rerun of
this corpus. `T7k` (2026-07-15) exercised that path with three more real-task
trials and confirmed both sides of the current state: the lane can surface
useful live defects (`T7K-01`, `T7K-02`), but it still produces mostly
diagnostic evidence plus transport fragility (`T7K-03`), not enough clean wins
to unblock T8 by itself. Priority 2's two fixes strengthen any future
evidence but are not individually blocking in the way Priority 1 is.

## Execution

Each priority becomes its own RRI-scored task(s) in
`docs/tasks/adr036-local-first-pilot.md` when picked up, following the same
presentation-before-implementation discipline used for `T7f` — this document
fixes scope and order, not task IDs, effort, or model assignment.

## Related

- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/plan/adr036-local-first-pilot.md`
- `docs/tasks/adr036-local-first-pilot.md` (`T7d`, `T7e`, `T7f`, `T8`)
- `logs/gemma-audit/` (audit trail this plan repairs)
- `scripts/gemma-audit-report.py`, `scripts/local-agent/run_local_task.py`,
  `scripts/local-agent/scope_check.py`, `scripts/gemma_local.py`
