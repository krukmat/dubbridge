---
type: Audit
title: "RRI evidence: S-150-T2b-i translation dispatch outbox migration"
status: active
task: S-150-T2b-i
date: 2026-08-09
---

# S-150-T2b-i — Presentation-time RRI

## Scoped presentation surface

- `infra/migrations/0029_create_translation_dispatch_outbox.sql`

This task is strictly the forward-only schema boundary for durable translation
dispatch. It excludes the repository transaction and asset/project/target lookup
(`S-150-T2b-ii`), serialized jobs, Redis enqueue, and worker wiring (`S-150-T2c`).

## Result

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 0 | raw CC 3 -> score 0 | High |
| F files | 0 | one numbered migration | High |
| D domain | 4 | `infra/migrations` anchor floor (ADR-008, ADR-018) | High |
| T coverage | 3 | fresh-PostgreSQL migration/constraint checks required | High |
| A ambiguity | 1 | named schema identity and constraints | High |
| K coupling | 4 | `infra/migrations` anchor floor | High |
| P impact | 5 | persisted ownership and audit-relevant state | High |
| X context | 2 | migration plus `0028` constraints and task contract | High |

**Base value:** 100 x (weighted / 5) = 45
**Penalties applied:** auth_security (+10, anchor-rubric P floor >= 4)
**Final RRI:** 55 -> band Med-high (41-55) -> Effort L.
**Gates:** explicit human approval; migration-only review/Reflection exemptions;
fresh PostgreSQL verification before closure.
**Decomposition:** not triggered.

## Command

```bash
python3 scripts/rri.py --cc 3 --T 3 --A 1 --X 2 --D 4 --K 4 --P 5 \
  --touches infra/migrations/0029_create_translation_dispatch_outbox.sql \
  --penalty auth_security --platform dubbridge
```
