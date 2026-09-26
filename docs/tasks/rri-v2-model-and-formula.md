---
type: TaskList
title: "RRI v2: model and formula design tasks"
status: closed
plan: docs/plan/rri-v2-model-and-formula.md
---
# RRI v2: model and formula design tasks

> **Superseded scope note (2026-09-07, ADR-045):** "the candidate does not
> score or authorize its own adoption" described this ledger's own frozen
> scope at the time it closed. Later the same day, by explicit owner
> override, `docs/adr/ADR-045-rri-v2-authority-replacement.md` adopted a
> bridged version of this design as authoritative in `scripts/rri.py`,
> ahead of the staged validation sequence these design docs describe. This
> ledger's history, RRI, and review evidence remain accurate as a record of
> what was designed here; they no longer describe the current authority
> state of `scripts/rri.py`. See ADR-045 and
> `docs/audit/rri-v2-authority-replacement-2026-09-07.md`.

**Scope:** Documentation/planning only, authorized by the owner's 2026-09-07
instruction to work on the model and then the formula. Primary agent: Codex in
the current session; no delegated implementer or fallback is needed.
**Parent RRI:** 33, Moderate, Effort M. Existing RRI governs this task; the
candidate does not score or authorize its own adoption.

Task-analysis review: n/a — docs/planning-only exemption.
Code-solution review: n/a — docs/planning-only exemption.

## Live phase checklist

- [x] T1 — Model definition and evidence contract — Codex (`completed`).
- [x] T2 — Formula derived from the model — Codex (`completed`).
- [x] T3 — Verification, limits, and synchronized closure — Codex (`completed`).

- [x] T4 — Gap review, corrections, and renewed verification — Codex (`completed`).

- [x] T5 — Final consistency review and verification — Codex (`completed`).

## T1 — Define the measurement model

- **Status:** [x] Done — 2026-09-07 (model specification; empirical validation pending)
- **Type:** documentation/planning
- **Effort:** M
- **Complexity:** Moderate
- **RRI:** 33 — exact computation in the linked design audit, T1.
- **Depends on:** initial repository/source review
- **Objective:** Define the construct, observable dimensions, measurement units,
  anchors, missing-data semantics, and conditions for empirical validation.
- **Acceptance criteria:** Distinguish size/difficulty/risk/uncertainty; define
  each technical level with inspectable evidence; separate observations from
  estimates; address new code, mechanical edits, concurrency, unavailable tools,
  and retrospective evidence; cite primary technical sources with limits.
- **Evidence to emit:** `docs/proposals/rri-v2-model.md`; T1 score and design
  review notes in `docs/audit/rri-v2-design-validation.md`.
- **Status artifacts affected:** this ledger and linked plan.
- **Handoff:** Define and review the measurement contract before deriving any
  equation. Do not infer predictive accuracy from reproducibility or source names.
- **Verification:** Primary design review checked construct/units, four anchored
  axes, temporal provenance, source limits, unknowns, and validation separation.
  Cross-document QA subsequently passed under the T3 integration gate.

## T2 — Derive the candidate formula

- **Status:** [x] Done — 2026-09-07 (candidate formula specification)
- **Type:** documentation/planning
- **Effort:** M
- **Complexity:** Moderate
- **RRI:** 33 — exact computation in the linked design audit, T2.
- **Depends on:** T1
- **Objective:** Specify an immediately computable screening rule and a separate
  calibrated effort model, with examples and explicit abstention conditions.
- **Acceptance criteria:** Every symbol maps to T1; ordinal values are not
  treated as ratio quantities; unknowns cannot silently become low complexity;
  size and risk do not inflate the technical profile; no fabricated fitted
  coefficients; aggregation/decomposition and uncertainty are explicit.
- **Evidence to emit:** `docs/proposals/rri-v2-formula.md`; T2 score and numerical
  property checks in `docs/audit/rri-v2-design-validation.md`.
- **Status artifacts affected:** this ledger and linked plan.
- **Handoff:** Derive the rule after T1, document rejected alternatives and their
  counterexamples, and leave existing governance rules active.
- **Verification:** Primary formula review checked ordinal semantics, unknown
  bounds, all symbols against T1, decomposition, quantile interpretation, and
  abstention without fitted coefficients. Exhaustive arithmetic checks subsequently passed in T3.

## T3 — Verify the candidate design and close documentation

- **Status:** [x] Done — 2026-09-07 (design validation only)
- **Type:** documentation/planning
- **Effort:** S
- **Complexity:** Low
- **RRI:** 15 — exact computation in the linked design audit, T3; parent remains 33.
- **Depends on:** T2
- **Objective:** Verify the stated mathematical properties and examples, assess
  available evidence, and report exactly what remains unvalidated.
- **Acceptance criteria:** Reproducible checks pass; examples agree with the
  formula; empirical validation is explicitly pending; local links and docs QA
  are checked; no changes to the active calculator/policies or user-owned edits.
- **Evidence to emit:** `docs/audit/rri-v2-design-validation.md`, including exact
  commands, outcomes, limitations, and any unrelated documentation QA failures.
- **Status artifacts affected:** this ledger, linked plan, both proposals' status.
- **Handoff:** Verify internal consistency without labeling synthetic scenarios
  as real project outcomes; report blockers honestly and preserve existing work.
- **Verification:** The audit's executable mathematical block passed all 625
  profiles, 2,000 monotone steps, 50,625 interval boxes, and 1,500,625 admissible
  profile evaluations. Point/range examples and invalid-input checks passed.
  `make qa-docs`, local-link/JSON/whitespace checks, and `git diff --check` passed.

## Integrated closure — 2026-09-07

- Completed the model first, the formula second, and documentation verification
  third, under the owner's bounded instruction.
- [Model](../proposals/rri-v2-model.md) and
  [formula](../proposals/rri-v2-formula.md) remain `Proposed`, version
  `rri-v2-design-0.2` after T4 review, with empirical prediction status `not_calibrated`.
- [Audit](../audit/rri-v2-design-validation.md) records legacy scores, commands,
  evidence, design limitations, and successful checks; linked plan is synchronized.
- No runtime/policy/model-routing change, independent development review claim,
  commit, push, or modification of the pre-existing user-owned task edit.
- Product roadmap and ADR synchronization: n/a; no product status, architecture,
  dependency, or active governance decision changed.

## T4 — Review gaps and inconsistencies

- **Status:** [x] Done — 2026-09-07 (design review and corrections only)
- **Type:** documentation/planning
- **Effort:** M
- **Complexity:** Moderate
- **RRI:** 33, same five-file inputs and decision penalty as the parent; rerun
  before corrections. The owner's follow-up authorizes review and bounded fixes.
- **Depends on:** T3
- **Objective:** Challenge model/formula assumptions, repair inconsistencies,
  and distinguish demonstrated properties from future validation requirements.
- **Acceptance criteria:** Resolve identified specification contradictions;
  record residual empirical gaps; rerun mathematical and documentation checks.
- **Evidence to emit:** Follow-up review findings in the existing design audit.
- **Status artifacts affected:** Both proposals, this ledger, the linked plan,
  and the existing audit; no production artifacts.
- **Handoff:** Check measurement semantics, aggregation, evaluation splits,
  target eligibility, uncertainty, and the reach of existing validation claims.
- **Verification:** Exhaustive arithmetic checks and `make qa-docs` passed again;
  23 local links, example JSON, version synchronization, input-order contract,
  whitespace, and `git diff --check` passed. Ten findings and residual empirical
  requirements are recorded in the audit. Primary review only; no predictive
  calibration or independent certification claimed.

## T5 — Final consistency review

- **Status:** [x] Done — 2026-09-07 (final design review only)
- **Type:** documentation/planning
- **Effort:** S
- **Complexity:** Low
- **RRI:** 15, rerun with the exact T3 inputs and five-document scope; parent
  design envelope remains 33. Primary authoring under the docs-only route.
- **Depends on:** T4
- **Objective:** Perform the owner's requested "revision final" of version 0.2.
- **Acceptance criteria:** Check the final model/formula against the T4 findings,
  replay arithmetic and document checks, and record residual limitations honestly.
- **Evidence to emit:** T5 final review section in the existing design audit.
- **Status artifacts affected:** This ledger, linked plan, and design audit.
- **Handoff:** Review all five artifacts; do not equate design consistency with
  empirical validity or production readiness.
- **Verification:** Exhaustive arithmetic, 23 local links, JSON/version checks,
  and `make qa-docs` passed. No new blocking design inconsistency found. Proposal
  fingerprints and residual empirical requirements are recorded under T5 in the
  audit. Independent development reviews n/a under the documentation-only
  exemption; no delegated reviewer invoked.
