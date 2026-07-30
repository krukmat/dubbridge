---
type: Audit
title: "RRI evidence: S-140-T3c-iii Redis queue integration tests"
status: proposed
slice: s-140-subtitle-generation
---

## S-140-T3c-iii — Presentation-time RRI

```bash
python3 scripts/rri.py \
  --auto-cc \
  --T 2 --A 1 --X 3 \
  --D 3 --K 3 --P 3 \
  --touches crates/jobs/src/lib.rs \
  --touches Makefile \
  --touches .github/workflows/ci.yml \
  --platform dubbridge
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | cargo clippy over crate graph -> no cognitive-complexity warnings in 1 touched file(s) -> CC 1 -> score 0 (policy CC table) | High |
| F files | 2 | --touches -> 3 files | High |
| D domain | 3 | anchor rubric: crates/jobs (ADR-006, ADR-018) -> floor 3 (agent 3 kept) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 3 | anchor rubric: crates/jobs (ADR-006, ADR-018) -> floor 3 (agent 3 kept) | High |
| P impact | 3 | anchor rubric: crates/jobs (ADR-006, ADR-018) -> floor 3 (agent 3 kept) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 39
**Penalties applied:** none
**Final RRI:** 39 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered
**Advisory:** Makefile: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** .github/workflows/ci.yml: no anchor-rubric match — agent judgment governs D/P/K
