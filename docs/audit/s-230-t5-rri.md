---
type: Audit
title: "S-230-T5 RRI at task-presentation time"
date: 2026-08-24
task: S-230-T5
---

# S-230-T5 RRI

Command:

```bash
python3 scripts/rri.py \
  --touches infra/production/docker-compose.yml \
  --touches infra/production/Caddyfile \
  --touches .env.example \
  --touches config/production.toml \
  --touches config/README.md \
  --touches docs/tasks/s-230-poc-v1-digitalocean.md \
  --touches docs/plan/s-230-poc-v1-digitalocean.md \
  --touches docs/plan/roadmap.md \
  --C 3 --D 3 --K 3 --P 3 --T 3 --A 3 --X 4 \
  --penalty auth_security
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 3 | agent-supplied score | High |
| F files | 3 | --touches -> 8 files | High |
| D domain | 3 | anchor rubric: config/README.md (ADR-026) -> floor 1 (agent 3 kept) | High |
| T coverage | 3 | agent-supplied | High |
| A ambiguity | 3 | agent-supplied | High |
| K coupling | 3 | anchor rubric: config/README.md (ADR-026) -> floor 1 (agent 3 kept) | High |
| P impact | 3 | anchor rubric: config/README.md (ADR-026) -> floor 1 (agent 3 kept) | High |
| X context | 4 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 61
**Penalties applied:** auth_security (+10, manual flag)
**Final RRI:** 71 -> band High (71-85) -> Effort XL . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Characterization tests + explicit acceptance criteria + human reviews the diff.
**Decomposition:** triggered by RRI >= 56 — split before implementing
**Advisory:** infra/production/docker-compose.yml: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** infra/production/Caddyfile: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** .env.example: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** config/production.toml: config floor is 0 (non-secret); if it wires env/secrets, raise D/P/K to >= 1 (ADR-026)

## Scoring notes

- `C=3` uses the non-development decision-weight heuristic: proxy/TLS choice,
  migration ordering, secret propagation, public hostname/DO values, upload
  ceiling, JWT lifetime coupling, and local dry-run design.
- `T=3` reflects that no task-specific production-descriptor tests exist yet;
  T5 defines `docker compose config`, a local-stack dry-run, and secret scanning
  as its verification strategy.
- `A=3` records unresolved deployment inputs (hostname, globally unique Spaces
  bucket, region, JWT lifetime) plus the ordering mismatch with T7c.
- The `auth_security` penalty applies because the task defines how JWT, storage,
  gateway, and translation credentials cross the production secret boundary,
  even though it must never commit their values.

