---
type: Audit
title: "Audit: Antares T0 execution-time RRI"
date: 2026-07-29
status: closed
---

# Audit: Antares T0 execution-time RRI

Scope: execution-time RRI evidence for `T0 — Define role charter and authority
boundary` in the Antares security-specialist advisor slice.

Command:

```bash
python3 scripts/rri.py \
  --touches docs/plan/antares-security-specialist-advisor.md \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --touches docs/playbooks/AGENT_WORKFLOW_GUIDE.md \
  --touches docs/policies/RRI_POLICY.md \
  --touches docs/policies/HITL_AUTONOMY_POLICY.md \
  --cc 1 --D 1 --K 2 --P 1 --T 0 --A 1 --X 2 \
  --penalty arch_decision
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 2 | `--touches` -> 5 files | High |
| D domain | 1 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no rubric match) | High |
| P impact | 1 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 19
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 31 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered

Note: this execution-time artifact supersedes the earlier narrower provisional
calculation that was produced before the final T0 workflow scope was frozen.
