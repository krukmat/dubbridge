---
type: TaskList
title: "RRI v2 Low-band correction tasks"
status: closed
plan: docs/plan/rri-v2-low-band-correction.md
---
# RRI v2 Low-band correction tasks

Behavioral coverage contract: behavior-v2

Parent RRI100 / Very high / XL. User approved direct implementation of the
previously proposed bounded patch on 2026-09-08. No fresh permission needed.
Phase-1/phase-2 reviewer: Claude (cross-vendor); author: primary Codex.
Evidence: [audit](../audit/rri-v2-low-band-correction.md) and review receipts.
Status synchronization: linked plan, this ledger, audit, RRI policy, ADR-045.

## Live phase checklist

- [x] Analyze and scope — Codex (`completed`).
- [x] Phase 1 review — Claude (`completed`, PASS).
- [x] Approval — owner, "cambialo directamente" (`completed`).
- [x] Implement — Codex (`completed`).
- [x] Phase 2 review — Claude (`completed`, PASS).
- [x] Reflect and verify — Codex, four passes (`completed`).
- [x] Close — Codex (`completed`).

## T1 — Freeze the amendment and risk limits

- **Status:** [x] Done — 2026-09-08.
- **Type:** planning
- **Effort:** XL (parent envelope).
- **Depends on:** owner instruction.
- **Acceptance:** exact table-only correction and unchanged higher levels;
  explicitly retain known risk-only limitations; no new scoring construct.
- **Handoff:** use the plan as the bounded implementation contract.
- **Evidence to emit:** plan and audit with pre-change calculator output.

## T2 — Correct bridge and verify behaviors

- **Status:** [x] Done — 2026-09-08.
- **Type:** development
- **RRI:** 100
- **Effort:** XL (parent envelope; one coherent policy/code invariant).
- **Depends on:** T1 and phase-1 review.
- **Acceptance:** all cases below pass; active ADR/policy match executable mapping.
- **Handoff:** change only the ICI25 bridge, add meaningful regressions,
  preserve risk aggregation and all higher mappings.
- **Evidence to emit:** unit/CLI regressions, independent review receipts,
  audit with commands and limitations.

### Happy paths considered

- HP-1: isolated ordinary constant C0/D1/K1/P1/T1 yields 25/Low; JSON and markdown agree.
- HP-2: mechanical all-zero profile remains 25; every technical level >=2 preserves its score.

### Edge cases considered

- EC-1: risk >25 with ICI25 still determines final RRI (security/architecture penalties).
- EC-2: auth path floors still produce 100; isolated P-only sensitive cases are explicitly not asserted safe.
- EC-3: raising any technical axis never decreases RRI across all 625 technical profiles.

### Reflection log

Required passes: 4 (RRI100 parent envelope).

Four Draft/Critique/Revise passes completed in the linked audit: contract and
upper levels; risk boundaries and review-vector correction; executable CLI
and exhaustive evidence; canonical documentation propagation.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | Local constant reaches Low in both CLI formats | unit | `scripts/rri_test.py::TechnicalProfileV2.test_local_constant_reaches_low_through_cli` | passed |
| HP-2 | Happy path | Mechanical floor and higher levels unchanged | unit | `scripts/rri_test.py::TechnicalProfileV2.test_all_technical_profiles_preserve_upper_levels_and_monotonicity` | passed |
| EC-1 | Edge case | Risk37 dominates technical25 | unit | `scripts/rri_test.py::TechnicalProfileV2.test_risk_above_low_still_wins_for_local_profile` | passed |
| EC-2 | Edge case | Sensitive path floors retain100 | unit | `scripts/rri_test.py::TechnicalProfileV2.test_sensitive_path_floors_are_unchanged` | passed |
| EC-3 | Edge case | All technical profiles and one-axis increases remain monotone | unit | `scripts/rri_test.py::TechnicalProfileV2.test_all_technical_profiles_preserve_upper_levels_and_monotonicity` | passed |

### Owner final verification

- Owner: Codex (primary task owner; not a claim of human manual verification).
- Date: 2026-09-08
- Statement: I verified every happy path and edge case defined for this task
  has executable evidence at an appropriate layer that replicates the expected behavior.
- Commands run: `make qa-rri`; `git diff --check`;
  `python3 scripts/rri.py --cc 1 --F 0 --D 1 --K 1 --P 1 --T 1 --A 0 --X 1`.
- Code-solution review: claude docs/audit/rri-v2-low-band-correction-phase2.json - PASS

## T3 — Integrated review and closure

- **Status:** [x] Done — 2026-09-08.
- **Type:** verification
- **Effort:** XL (parent review envelope).
- **Depends on:** T2.
- **Acceptance:** independent code review PASS before certification; four
  Reflection loops; relevant tests and documentation QA reported accurately.
- **Handoff:** synchronize only affected status artifacts and preserve user edits.

Task-analysis review: claude docs/audit/rri-v2-low-band-correction-phase1.json - PASS

Integrated closure: `make qa-rri`, `make qa-docs`, and `git diff --check` passed.
Both independent review phases PASS; no commit/push performed.
