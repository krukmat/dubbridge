---
type: Audit
title: "S-150-T2c-v Redis-topic decision — resolution record"
status: resolved
---

# S-150-T2c-v Redis-topic decision — resolution record

## Origin

The phrase "parked pending a Redis-topic decision" was introduced in commit
`0e4a634` ("Close S-150-T2c-iv-c and add post-repair-budget Low-band
decomposition policy", 2026-08-16) as an unelaborated parking note on
`S-150-T2c-v`. No ADR, RFC, plan section, audit doc, or commit message ever
defined the decision's actual content — every later reference (the plan
status header, the task-ledger entry, `S-230-T3b`'s blocking-precondition
note) repeated the same phrase without elaboration. `git log --all -S
"Redis-topic"` confirms `0e4a634` as the sole introduction point.

## Investigation

Direct code inspection (2026-08-21) established the actual remaining gap:

- `TranslationJob::JOB_TYPE = "translation_generation"` is **already
  committed** (`crates/jobs/src/subtitle_job.rs:50`, delivered by
  `T2c-iv-c`) — a single flat job-type string, not per-target-language.
- Three existing Redis-backed queues already exist for the other job types
  (`RedisPreparationJobQueue`, `RedisTranscriptionJobQueue`,
  `RedisSubtitleJobQueue`), all built via the same `define_redis_job_queue!`
  macro (`crates/jobs/src/lib.rs`), each simply calling
  `apalis_redis::Config::default().set_namespace(<$job_ty>::JOB_TYPE)` — one
  shared queue per job type, no partitioning, no pub/sub.
- No `RedisTranslationJobQueue` exists yet; this is exactly the gap
  `T2c-v` is scoped to close.

Since the job-type/namespace string is already fixed in code, the open
question was narrowed to Redis topology, not naming.

## Options considered

**A — Single flat queue (mirror the existing three exactly).** Add
`define_redis_job_queue!(RedisTranslationJobQueue, TranslationJob,
TranslationJobQueue)` using the already-committed `JOB_TYPE`. Mechanical,
consistent with every other job type in the codebase.

**B — Per-target-language partitioned topic**
(`translation_generation:{target_language_id}`). Would allow independent
consumer scaling or provider isolation per language. Requires renaming/
changing the already-shipped `JOB_TYPE` constant and the enqueue/consumer
wiring. No requirement anywhere in the S-150 plan or task ledger names a
per-language rate-limit or provider-isolation need.

**C — Pub/sub broadcast instead of an apalis queue.** Would let multiple
future consumer types (e.g. a later TTS pipeline) react to the same event.
Breaks the durable, idempotent per-target dispatch model already built in
`T2b-ii-c`/`T2c-iv-c` (`translation_delivery_repo`); `apalis-redis` does not
support pub/sub semantics, so this would require an entirely different
transport.

## Decision

**Option A — single flat namespace, mirroring the existing three
`define_redis_job_queue!` queues exactly.** Confirmed with the owner
2026-08-21.

**Rationale:** the job-type constant driving the Redis namespace is already
committed in code from `T2c-iv-c`; reversing it now would mean unwinding an
already-shipped constant with no documented requirement forcing partitioning
or broadcast semantics. Options B and C each require a justification that
does not exist anywhere in the repository's governing documents.

## Effect

- Resolves the blocking precondition on `S-150-T2c-v`
  (`docs/tasks/s-150-translation-dubbing.md`) and on `S-230-T3b`'s child 1
  (`docs/tasks/s-230-poc-v1-digitalocean.md`).
- `S-150-T2c-v`'s own RRI/approval/review/closure requirements are otherwise
  unchanged — this record only resolves the parking note, not the task's own
  gate.

## Related

- `docs/plan/s-150-translation-dubbing.md`
- `docs/tasks/s-150-translation-dubbing.md` § S-150-T2c-v
- `docs/tasks/s-230-poc-v1-digitalocean.md` § S-230-T3b
- `docs/audit/s-150-t2c-decomposition-rri.md`
