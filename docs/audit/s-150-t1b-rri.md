---
type: Audit
title: "RRI evidence: S-150-T1b per-target status and artifact-kind migration"
status: active
task: S-150-T1b
date: 2026-08-02
---

# S-150-T1b - Presentation-time RRI

## Scoped presentation surface

- `infra/migrations/00xx_s150_t1b_localization_status_and_artifact_kind.sql`

This presentation is intentionally scoped to the migration-only surface named by
the task ledger. It excludes repository wiring, queue fan-out, worker runtime,
and TTS/provider work, which begin in later tasks (`S-150-T1c` onward).

## Command

```bash
python3 scripts/rri.py \
  --cc 1 \
  --T 2 \
  --A 1 \
  --X 2 \
  --D 4 \
  --K 4 \
  --P 5 \
  --touches infra/migrations/00xx_s150_t1b_localization_status_and_artifact_kind.sql \
  --platform dubbridge
```

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 0 | `--touches` -> 1 files | High |
| D domain | 4 | anchor rubric: `infra/migrations` (ADR-008, ADR-018) -> floor 4 (agent 4 kept) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 4 | anchor rubric: `infra/migrations` (ADR-008, ADR-018) -> floor 4 (agent 4 kept) | High |
| P impact | 5 | anchor rubric: `infra/migrations` (ADR-008, ADR-018) -> floor 5 (agent 5 kept) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 42
**Penalties applied:** auth_security (+10, anchor-rubric P floor >= 4 (auth/audit/rights/secrets))
**Final RRI:** 52 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered

## Scoring notes

- `T=2`: the task has an explicit live-PostgreSQL verification path and schema
  inspection requirement, but no existing unit-style coverage surface in
  `infra/migrations/`.
- `A=1`: the ledger provides concrete acceptance criteria plus one happy path
  and two edge cases, so ambiguity is low but not zero because exact table and
  constraint names remain implementation details.
- `X=2`: the task needs the S-150 ledger/plan plus migration precedents
  (`0020`, `0022`, `0023`, `0024`) to avoid regressing the existing
  `artifact_kind_check` contract.
