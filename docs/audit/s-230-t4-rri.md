---
type: Audit
title: "S-230-T4 RRI evidence"
status: proposed
task: S-230-T4
---
# S-230-T4 — RRI evidence

## Scoring command

```bash
python3 scripts/rri.py --cc 1 \
  --touches apps/api/Dockerfile \
  --touches apps/gateway/Dockerfile \
  --touches apps/worker-runner/Dockerfile \
  --touches apps/cli/Dockerfile \
  --touches workers/asr-worker-py/requirements.txt \
  --D 3 --K 4 --P 3 --T 4 --A 1 --X 3
```

## Result

**Final RRI: 47 — Med-high (41–55) — Effort L.** No penalties apply.

| Variable | Score | Evidence |
|---|---:|---|
| C | 0 | The planned Dockerfile-only surface has raw CC 1. |
| F | 2 | Five planned files: API, gateway, worker-runner, and CLI/migration Dockerfiles plus the ASR Python lock input. |
| D | 3 | Container runtime integration spans the API, gateway, and async worker topology. |
| T | 4 | No focused image-build/runtime test harness exists; acceptance requires new local build/run and readiness evidence. |
| A | 1 | The ledger defines concrete images, runtime contents, HP/EC cases, and stop condition; exact image layout remains an implementation decision. |
| K | 4 | Image layout must preserve executable paths, process startup, Redis/PostgreSQL/S3 configuration, and worker subprocess behavior across service boundaries. |
| P | 3 | The change does not alter routes or credentials, but it packages the public API/gateway and operational processing path. |
| X | 3 | The task requires the three application manifests, worker runtime, ASR worker dependencies, local Compose topology, migration CLI, and readiness contracts. |

## Routing implications

This is a development task despite its Dockerfile-heavy surface: it creates the
production runtime boundary for three binaries and a migration job. It therefore
is not treated as a config-only review exemption.

RRI 47 requires explicit human approval, a phase-1 Gemma review before the
card, the ADR-038 Med-high route, three Reflection passes, phase-2 Gemma review,
unit-coverage certification, and owner final verification. No architectural
decision is needed: the existing single-droplet/production-Compose target and
the dependency stack are already fixed by S-230, so `arch_decision` is not a
penalty.
