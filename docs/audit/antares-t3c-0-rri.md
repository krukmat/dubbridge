---
type: Audit
title: "RRI evidence: T3c-0 - Characterization corpus and omission-reason contract"
status: proposed
task: docs/tasks/antares-security-specialist-advisor.md#t3c-0---characterization-corpus-and-omission-reason-contract
date: 2026-08-02
---

# RRI evidence: T3c-0 — Characterization corpus and omission-reason contract

Task: `docs/tasks/antares-security-specialist-advisor.md` § T3c-0
Depends on: T3b (`[x] Done (owner-verified, 2026-08-02)`)

## Presentation-time computation (2026-08-02, pre-implementation)

```bash
python3 scripts/rri.py \
  --touches scripts/antares/packet_schema.py \
  --touches scripts/antares/packet_schema_test.py \
  --touches scripts/antares/context_closure_characterization_test.py \
  --cc 8 \
  --D 3 --K 3 --P 4 \
  --T 1 --A 0 --X 3
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 8 -> score 1 (policy CC table) | High |
| F files | 2 | --touches -> 3 files | High |
| D domain | 3 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 39
**Penalties applied:** none
**Final RRI:** 39 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered
**Advisory:** scripts/antares/packet_schema.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/antares/packet_schema_test.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/antares/context_closure_characterization_test.py: no anchor-rubric match — agent judgment governs D/P/K
