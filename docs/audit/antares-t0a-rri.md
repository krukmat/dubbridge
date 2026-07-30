---
type: Audit
title: "RRI evidence: Antares T0a design correction"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t0a---correct-charter-and-close-design-gaps
date: 2026-07-29
---

# RRI Evidence: Antares T0a Design Correction

## Command

```bash
python3 scripts/rri.py \
  --touches docs/playbooks/AGENT_WORKFLOW_GUIDE.md \
  --touches docs/policies/HITL_AUTONOMY_POLICY.md \
  --touches docs/plan/antares-security-specialist-advisor.md \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --touches docs/audit/antares-t0a-rri.md \
  --cc 1 --D 2 --K 3 --P 3 --T 2 --A 1 --X 2 \
  --penalty arch_decision
```

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 2 | `--touches` -> 5 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 35

**Penalties applied:** `arch_decision` (+12, manual flag)

**Final RRI:** 47 -> band Med-high (41-55) -> Effort L. Codex Balanced ->
Premium. Claude Balanced -> Premium. Thinking On.

**Gates for this band:** plan plus explicit acceptance criteria required before
approval.

**Decomposition:** not triggered.

## Scoring rationale

- `D=2`: the task corrects a security-specialist integration contract but does
  not alter runtime authorization behavior.
- `K=3`: the workflow guide, HITL policy, plan, ledger, and RRI evidence must stay
  semantically consistent.
- `P=3`: misleading capability or authority wording can affect security-process
  decisions, while all Antares output remains non-binding.
- `T=2`: verification combines deterministic documentation QA with targeted
  semantic searches and diff review; no code coverage applies.
- `A=1`: the source-backed gaps and expected corrections are explicit.
- `X=2`: the task depends on the external model card and benchmark methodology,
  but no live model execution is required.
- `arch_decision`: the task decides where and how an advisory security component
  participates in refinement, review, and post-CI processing.
