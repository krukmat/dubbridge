---
type: TaskList
title: "Tasks: Gemma Reviewer Triple-Pass Reconciliation"
plan: docs/plan/gemma-reviewer-triple-pass.md
status: superseded
superseded_by: docs/tasks/gemma-audit-and-triple-pass.md
rri: 48
band: Med-high
effort: L
---
# Tasks: Gemma Reviewer Triple-Pass Reconciliation

> **Status:** SUPERSEDED by `docs/tasks/gemma-audit-and-triple-pass.md` (plan
> `docs/plan/gemma-audit-and-triple-pass.md`), which folds this triple-pass work
> into the audit-and-triple-pass slice and resolves the gaps left open here. Kept
> for history; do not execute from this ledger.

## Objective

Add a three-pass local review mode for `Gemma Reviewer`, verify the three
reports, perform a reconciliation/reflection step that contrasts them, and
document the resulting evidence contract.

## Governing documents

- `docs/plan/gemma-reviewer-triple-pass.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/RRI_POLICY.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/gemma-local-improve.md`
- `docs/plan/low-medium-gemma-code-review-role.md`
- `docs/tasks/low-medium-gemma-code-review-role.md`

## RRI

**Score: 48 -> Med-high (41-55) -> Effort L -> Balanced -> Premium -> thinking On**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | wrapper control flow + aggregation logic | High |
| F files | 2 | wrapper, tests, docs, Makefile | High |
| D domain | 3 | workflow/Ollama integration | High |
| T coverage | 2 | existing test area, new aggregation behavior | High |
| A ambiguity | 0 | this task file defines scope and AC | High |
| K coupling | 3 | wrapper + workflow docs + local pipeline usage | High |
| P impact | 2 | internal tooling/reporting only | High |
| X context | 4 | wrapper, tests, local tooling, workflow docs | High |

## Task order and dependencies

```text
T1
```

---

## T1 - Triple-pass reviewer + reconciliation

- **Status:** ⬜ Not started
- **Type:** Development
- **Effort:** L
- **RRI:** 48 -> Med-high
- **Scope:** `scripts/gemma-code-review.py`, `scripts/gemma_code_review_test.py`,
  `docs/gemma-local-improve.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `Makefile` only if required by the artifact/output contract.

### Goal

Run `Gemma Reviewer` three times per review, persist three reports, compare the
reports in a wrapper-owned reflection step, and expose a structured aggregate
artifact that makes disagreements and likely inconsistencies explicit.

### Acceptance criteria

- The wrapper runs **three review passes** per invocation against the same packet.
- Each pass produces a separate persisted report artifact or report entry with
  stable pass numbering.
- The aggregate result includes:
  - per-pass status and summary;
  - merged finding list;
  - indication of consensus vs pass-specific findings;
  - inconsistencies across the three passes;
  - a reconciliation/reflection summary.
- The wrapper keeps the existing read-only review contract for each individual pass.
- Parser and validation logic still reject patch-like output and malformed finding blocks.
- Tests cover:
  - three-pass execution orchestration;
  - aggregation of identical findings;
  - disagreement on severity or line number;
  - one pass `PASS` while another returns `FINDINGS`;
  - reflection summary for likely false positives or inconsistent reports.
- Docs explain the new evidence shape and how the primary agent should report it.

### Happy paths considered

- **HP-1:** all three passes return `PASS` -> aggregate report is clean and reflection notes no inconsistencies.
- **HP-2:** the same finding appears in all three passes -> aggregate marks it as consensus.
- **HP-3:** two passes agree and one differs -> aggregate surfaces the disagreement instead of hiding it.

### Edge cases considered

- **EC-1:** one pass returns malformed output -> wrapper fails clearly and does not emit a misleading aggregate pass result.
- **EC-2:** two findings describe the same issue with different severities or line numbers -> reflection marks the inconsistency explicitly.
- **EC-3:** one pass hallucinates an issue outside the packet semantics -> reflection/reporting makes it identifiable as pass-specific rather than consensus.
- **EC-4:** one pass returns `BLOCKED` -> wrapper reports operational failure consistently with the current authority contract.

### Handoff prompt

T1 - triple-pass Gemma Reviewer with reconciliation. Governing docs:
`docs/tasks/gemma-reviewer-triple-pass.md` and `docs/plan/gemma-reviewer-triple-pass.md`.
Files: `scripts/gemma-code-review.py`, `scripts/gemma_code_review_test.py`,
`docs/gemma-local-improve.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, and
`Makefile` only if needed. Acceptance: three passes, three reports, aggregate
reconciliation/reflection output, parser safety unchanged, tests green, docs updated.
