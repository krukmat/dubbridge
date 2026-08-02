---
type: Audit
title: "RRI evidence: S-150-T1c-i generation-claim and exact-pointer schema migration"
status: active
task: S-150-T1c-i
date: 2026-08-02
---

# S-150-T1c-i - Presentation-time RRI

## Scoped presentation surface

- `infra/migrations/0028_add_localization_generation_claims_and_exact_pointers.sql`

This presentation is intentionally scoped to the migration-only surface named by
the task ledger. It excludes repository wiring, queue fan-out, worker runtime,
and TTS/provider work, which begin in later tasks (`S-150-T1c-ii` onward).

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | agent-supplied score | High |
| F files | 0 | --touches -> 1 files | High |
| D domain | 4 | anchor rubric: infra/migrations (ADR-008, ADR-018) -> floor 4; raised from 3 | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 4 | anchor rubric: infra/migrations (ADR-008, ADR-018) -> floor 4; raised from 3 | High |
| P impact | 5 | anchor rubric: infra/migrations (ADR-008, ADR-018) -> floor 5; raised from 4 | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 42
**Penalties applied:** auth_security (+10, anchor-rubric P floor >= 4 (auth/audit/rights/secrets))
**Final RRI:** 52 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered

## Command

```bash
python3 scripts/rri.py --C 0 --T 2 --A 1 --X 2 --D 3 --K 3 --P 4 \
  --touches infra/migrations/0028_add_localization_generation_claims_and_exact_pointers.sql \
  --platform dubbridge
```
