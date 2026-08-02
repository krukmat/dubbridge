---
type: Audit
title: "RRI evidence: T3c - Deterministic context-closure algorithm"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3c---deterministic-context-closure-algorithm
date: 2026-08-02
---

# RRI evidence: T3c — Deterministic context-closure algorithm

Task: `docs/tasks/antares-security-specialist-advisor.md` § T3c
Depends on: T3b (`[x] Done (owner-verified, 2026-08-02)`)

## Presentation-time computation (2026-08-02, pre-decomposition)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/context_closure.py \
  --touches scripts/antares/context_closure_test.py \
  --cc 14 \
  --D 3 --K 3 --P 4 \
  --T 4 --A 3 --X 4 \
  --penalty arch_decision
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | raw CC 14 -> score 2 (policy CC table) | High |
| F files | 1 | --touches -> 2 files | High |
| D domain | 3 | agent-supplied (no rubric match) | High |
| T coverage | 4 | agent-supplied | High |
| A ambiguity | 3 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 4 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 58
**Penalties applied:** arch_decision (+12, manual flag); no_tests_high_impact (+10, T=4 >= 4 and P=4 >= 4)
**Final RRI:** 80 -> band High (71-85) -> Effort XL . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Characterization tests + explicit acceptance criteria + human reviews the diff.
**Decomposition:** triggered by RRI >= 56, T >= 4 and P >= 4 — split before implementing
**Advisory:** scripts/antares/context_closure.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/antares/context_closure_test.py: no anchor-rubric match — agent judgment governs D/P/K
