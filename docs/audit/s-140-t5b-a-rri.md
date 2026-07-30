---
type: Audit
title: "RRI evidence: S-140-T5b-a review-task artifact identity schema contract"
status: active
task: S-140-T5b-a
date: 2026-07-30
---

# S-140-T5b-a — Presentation-time RRI

## Scoped presentation surface

- `infra/migrations/00xx_add_review_tasks_subtitle_artifact_id.sql`
- `crates/domain/src/review.rs`

This subtask is intentionally limited to the schema/domain contract. It does
not include repository wiring, enqueue wiring, or API/mobile exposure.

## Command

```bash
python3 scripts/rri.py \
  --auto-cc \
  --T 2 \
  --A 1 \
  --X 2 \
  --D 3 \
  --K 3 \
  --P 3 \
  --touches infra/migrations/0014_create_review_tasks.sql \
  --touches crates/domain/src/review.rs \
  --platform dubbridge
```

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | cargo clippy over crate graph -> no cognitive-complexity warnings in 1 touched file(s) -> CC 1 -> score 0 | High |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 4 | anchor rubric: `infra/migrations` (ADR-008, ADR-018) -> floor 4 | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 4 | anchor rubric: `infra/migrations` (ADR-008, ADR-018) -> floor 4 | High |
| P impact | 5 | anchor rubric: `infra/migrations` (ADR-008, ADR-018) -> floor 5 | High |
| X context | 2 | agent-supplied | High |

- Base value: `45`
- Penalties applied: `auth_security (+10, anchor-rubric P floor >= 4)`
- Final RRI: `55`
- Band: `Med-high (41-55)`
- Effort: `L`
- Codex capability: `Balanced -> Premium`
- Claude capability: `Balanced -> Premium`
- Thinking mode: `On`

## Implication

`S-140-T5b-a` qualifies for approval presentation as a Med-high task. It still
requires:

- explicit human approval before implementation
- phase-1 and phase-2 review via `qwen3.6:27b-q4_K_M` (Gemma, then D14 fallback)
- 3 Reflection passes at closure
- ADR-038 implementation routing if approved
