---
type: Audit
title: "RRI evidence: S-150-T2c versioned localization jobs and outbox-backed fan-out"
task: S-150-T2c
status: current
---

# S-150-T2c — Presentation-time RRI

Computed on 2026-08-13 against the current post-`T2b-ii-c` repository state.

```text
python3 scripts/rri.py --cc 13 --T 3 --A 2 --X 2 --D 4 --K 5 --P 5 \
  --touches crates/jobs/src/lib.rs \
  --touches crates/db/src/subtitle_repo.rs \
  --touches crates/db/src/translation_delivery_repo.rs \
  --touches apps/worker-runner/src/subtitle_runtime.rs \
  --touches apps/worker-runner/src/subtitle_enqueue.rs \
  --touches apps/worker-runner/src/translation_enqueue.rs \
  --touches apps/worker-runner/src/main.rs \
  --platform dubbridge
```

| Variable | Score | Basis |
|---|---:|---|
| C | 2 | Raw cyclomatic complexity 13 |
| F | 3 | Seven production paths |
| D | 4 | Durable job, runtime, and persistence boundary |
| T | 3 | Existing focused coverage, new cross-surface behavior required |
| A | 2 | Versioned payload compatibility and dispatch eligibility are explicit |
| K | 5 | Couples serialization, PostgreSQL outbox, Redis, and worker runtime |
| P | 5 | Can change fan-out, idempotency, and post-ready routing behavior |
| X | 2 | Existing S-140/S-150 plans and repository seams provide context |

- **Final RRI:** `65` — **Complex**
- **Effort:** `L`
- **Penalties:** none
- **Gate:** mandatory decomposition and explicit approval before any implementation.

The prior provisional score (`54`) no longer represents the actual current scope.
The expected seven-file route crosses the RRI 56 decomposition threshold; this parent
must be split into independently scoped children before an executable approval card
can be presented.
