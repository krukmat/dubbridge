---
type: Audit
title: "RRI evidence: Antares T1 runtime and model-access preflight"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t1---runtime-and-model-access-preflight
date: 2026-07-29
---

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | agent-supplied score | High |
| F files | 2 | --touches -> 4 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 4 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 49
**Penalties applied:** none
**Final RRI:** 49 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered

Command run:

```bash
python3 scripts/rri.py \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --touches docs/plan/antares-security-specialist-advisor.md \
  --touches docs/evaluations/antares-runtime-preflight.md \
  --touches docs/audit/antares-t1-rri.md \
  --C 1 --D 4 --K 4 --P 4 --T 2 --A 1 --X 2
```
