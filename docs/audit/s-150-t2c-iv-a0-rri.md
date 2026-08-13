---
type: Audit
title: "RRI evidence: S-150-T2c-iv-a0"
task: S-150-T2c-iv-a0
status: current
---

# S-150-T2c-iv-a0 — RRI evidence

Computed on 2026-08-13 before approval against the current repository state.

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 0 | raw CC 2 -> score 0 (policy CC table) | High |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 3 | anchor rubric: `crates/jobs` (ADR-006, ADR-018) -> floor 3; raised from 2 | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | anchor rubric: `crates/jobs` (ADR-006, ADR-018) -> floor 3; raised from 2 | High |
| P impact | 3 | anchor rubric: `crates/jobs` (ADR-006, ADR-018) -> floor 3; raised from 2 | High |
| X context | 1 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 34
**Penalties applied:** none
**Final RRI:** 34 -> band Moderate (26-40) -> Effort M. Codex Balanced. Claude Balanced. thinking Off.

**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered.

Command:

```bash
python3 scripts/rri.py --platform dubbridge \
  --touches crates/jobs/src/lib.rs \
  --touches crates/jobs/src/subtitle_job.rs \
  --cc 2 --D 2 --K 2 --P 2 --T 2 --A 1 --X 1
```
