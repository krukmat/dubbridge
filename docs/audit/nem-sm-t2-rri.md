---
type: Audit
title: "NEM-SM-T2 RRI evidence"
status: Active
---

# RRI evidence — NEM-SM-T2

Generated with:

```text
python3 scripts/rri.py --auto-cc --touches scripts/delegate-low-rri.py --touches scripts/delegate_low_rri_test.py --touches docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md --touches docs/gemma-local-improve.md --D 3 --K 3 --P 2 --T 1 --A 0 --X 2 --penalty refactor_and_behavior
```

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 1 | auto-cc fallback: no local Rust files in scope | Low |
| F files | 2 | four touched paths | High |
| D domain | 3 | local delegation workflow | High |
| T coverage | 1 | targeted unit tests exist | High |
| A ambiguity | 0 | owner-approved explicit matrix and cases | High |
| K coupling | 3 | delegation transport and fallback protocol | High |
| P impact | 2 | internal workflow behavior | High |
| X context | 2 | four bounded files | High |

- Base value: 34
- Penalty: `refactor_and_behavior` (+8)
- Final RRI: **42 — Med-high / Effort L**
