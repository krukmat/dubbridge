---
type: TaskList
title: "Tasks: Plan Risk Analysis Standard"
plan: docs/plan/plan-risk-analysis-standard.md
status: planned
rri: 28
band: Moderate
effort: M
---

# Tasks: Plan Risk Analysis Standard

> **Status:** Planning package complete; adoption pending explicit approval.
> No workflow-policy or template implementation has started.

## PRA-1 — Adopt Plan Risk Register v1 across the workflow contract

- **Status:** Pending — RRI 28 requires explicit human approval.
- **Type:** Documentation / policy / template task.
- **Effort:** M (derived from RRI 28, Moderate).
- **Objective:** Make a lightweight risk register mandatory in every new plan,
  define its ADR and RRI boundaries, and surface residual posture in Compact
  Approval Task Card v2.
- **Dependencies:** The final design in
  `docs/plan/plan-risk-analysis-standard.md`; no runtime or architecture dependency.
- **Affected files:**
  - `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
  - `docs/policies/RRI_POLICY.md`
  - `docs/templates/plan.md` (new)
  - `docs/templates/compact-approval-task-card.md`
  - `AGENTS.md`
  - `CLAUDE.md`
- **Out of scope:** Runtime code, RRI formula/bands, HITL thresholds, legacy-plan
  bulk migration, and automated semantic evaluation of risks.
- **Evidence to emit:** Passing output from `make qa-docs` and
  `git diff --check`; a final cross-file acceptance review.
- **Status artifacts affected:** This ledger and
  `docs/plan/plan-risk-analysis-standard.md`.

### Acceptance criteria

1. The mandatory Plan step requires a `## Risk analysis` section for every new
   plan and defines the explicit no-material-risk form.
2. The canonical register records cause-event-consequence, category, inherent
   `L×I`, response, verification evidence, trigger/contingency, owner, residual
   `L×I`, and status.
3. Likelihood, impact, and `Low / Moderate / High / Critical` score anchors match
   the plan exactly.
4. Overall plan posture is the highest residual row, not an average.
5. High residual risks require response, evidence, owner, and contingency.
6. Critical residual risks block implementation unless reduced or explicitly
   accepted under the human/ADR rule.
7. The workflow and RRI policy distinguish plan risk from RRI and preserve
   `scripts/rri.py` as the sole execution-routing score calculator.
8. The ADR rule requires an ADR only for the conditions stated in the plan and
   avoids duplicate canonical registers.
9. `docs/templates/plan.md` contains both material-risk and no-material-risk
   examples.
10. Compact Approval Task Card v2 remains six blocks and adds only a Decision
    header row for plan risk posture and up to three top residual IDs.
11. `AGENTS.md` and `CLAUDE.md` are synchronized with the authoritative guide.
12. `make qa-docs` and `git diff --check` pass.

### Review and approval route

`Task-analysis review: n/a - docs/policy/template-only exemption`

The task still requires the RRI 26+ human approval gate. The phase-1 exemption
removes the independent reviewer step; it does not waive HITL approval.

At closure, record:

`Code-solution review: n/a - docs/policy/template-only exemption`

### Agent handoff prompt

```text
PRA-1 — Adopt Plan Risk Register v1 across the workflow contract.

Governing docs:
- docs/tasks/plan-risk-analysis-standard.md
- docs/plan/plan-risk-analysis-standard.md

Change only the six affected files listed in PRA-1.

Acceptance:
- Satisfy PRA-1 criteria 1–12 without changing RRI or HITL thresholds.
- Run make qa-docs and git diff --check.
- Update the plan and task ledger last; do not start a follow-up validator or legacy backfill.
```

## Full RRI evidence

Command:

```bash
python3 scripts/rri.py \
  --touches docs/playbooks/AGENT_WORKFLOW_GUIDE.md \
  --touches docs/policies/RRI_POLICY.md \
  --touches AGENTS.md \
  --touches CLAUDE.md \
  --touches docs/templates/plan.md \
  --touches docs/templates/compact-approval-task-card.md \
  --cc 1 --D 0 --K 1 --P 0 --T 1 --A 0 --X 3 \
  --penalty arch_decision
```

Unmodified calculator output:

```text
**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 3 | --touches -> 6 files | High |
| D domain | 0 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 0 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 16
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 28 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered
**Advisory:** AGENTS.md: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** CLAUDE.md: no anchor-rubric match — agent judgment governs D/P/K
```

### RRI rationale

- `C=0`: documentation and template changes add no executable branching.
- `F=3`: six explicitly bounded files.
- `D=0`, `P=0`: documentation/process scope with no runtime, public API,
  security, permission, or data impact.
- `T=1`: deterministic documentation checks exist, but semantic consistency
  still requires review.
- `A=0`: the plan fixes the schema, thresholds, ADR rules, affected files, and
  acceptance criteria.
- `K=1`: the policy text is internally coupled across authoritative and summary
  documents, without runtime side effects.
- `X=3`: the implementer must reconcile the full workflow contract across six
  files.
- `arch_decision +12`: adoption changes a process/policy contract.

## Ordered execution checklist

1. [ ] Obtain explicit human approval for PRA-1.
2. [ ] Update the authoritative guide and RRI policy.
3. [ ] Add the plan template and update Compact Approval Task Card v2.
4. [ ] Synchronize `AGENTS.md` and `CLAUDE.md`.
5. [ ] Run `make qa-docs` and `git diff --check`.
6. [ ] Review all twelve acceptance criteria and the plan risk register.
7. [ ] Update this ledger and the linked plan; report the docs/policy exemption.

