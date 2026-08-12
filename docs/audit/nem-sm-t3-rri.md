---
type: Audit
title: "NEM-SM-T3 RRI evidence"
status: Active
---

# RRI evidence — NEM-SM-T3

Generated with:

```text
python3 scripts/rri.py --auto-cc --touches scripts/local-agent/run_med_high_task.py --touches scripts/local-agent/run_med_high_task_test.py --D 4 --K 4 --P 3 --T 1 --A 0 --X 2 --penalty refactor_and_behavior
```

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 1 | auto-cc fallback: no local Rust files in scope | Low |
| F files | 1 | two touched paths | High |
| D domain | 4 | agent orchestration and escalation boundary | High |
| T coverage | 1 | targeted unit tests exist | High |
| A ambiguity | 0 | owner-approved explicit matrix and cases | High |
| K coupling | 4 | supervisor, evidence bundle, and fallback protocol | High |
| P impact | 3 | internal implementation-routing control | High |
| X context | 2 | supervisor and test module | High |

- Base value: 39
- Penalty: `refactor_and_behavior` (+8)
- Final RRI: **47 — Med-high / Effort L**
