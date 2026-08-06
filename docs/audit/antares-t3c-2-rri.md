---
type: Audit
title: "RRI evidence: T3c-2 - Governing security-boundary closure"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3c-2---governing-security-boundary-closure
date: 2026-08-06
---

# T3c-2 RRI evidence

Task: T3c-2 — Governing security-boundary closure  
Mode: pre-execution; no implementation diff  
Date: 2026-08-06

Command:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/governing_boundary_closure.py \
  --touches scripts/antares/governing_boundary_map.py \
  --touches scripts/antares/governing_boundary_closure_test.py \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --touches docs/plan/antares-security-specialist-advisor.md \
  --cc 8 \
  --D 3 --K 2 --P 3 --T 1 --A 1 --X 3 \
  --penalty arch_decision \
  --platform dubbridge
```

Output:

```text
**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 8 -> score 1 (policy CC table) | High |
| F files | 2 | --touches -> 5 files | High |
| D domain | 3 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 37
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 49 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered
**Advisory:** scripts/antares/governing_boundary_closure.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/antares/governing_boundary_map.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/antares/governing_boundary_closure_test.py: no anchor-rubric match — agent judgment governs D/P/K
```

Interpretation: the task lands in the Med-high band and therefore requires the
full RRI 26–55 workflow: phase-1 task-analysis review, explicit HITL approval,
ADR-038 route resolution for implementation, 3 Reflection passes, and the
band-routed independent phase-2 review before closure. The ambiguity score is
kept at 1 because the task definition fixes the boundary-map contract,
precedence rule, and fail-closed handling up front instead of leaving them to
implementation-time interpretation.
