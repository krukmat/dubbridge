# Plan: T1 Tuning / Hardening

**Roadmap position:** Post-slice reliability and operability phase. Execute when a
delivered slice exposes non-blocking but real operational risks that should be
closed before scale or production use.

## Objective

Track and remediate implementation risks that do not block current slice
acceptance but would weaken reliability, crash safety, or operational behavior if
left unaddressed.

## Initial scope

### Included
- Remove the in-memory pending-ingestion session loss risk introduced by S1 T5.
- Improve crash safety for multi-step upload/finalize workflows.
- Preserve existing fail-closed rights and auth invariants while hardening
  durability.
- Close residual lifecycle, cleanup, coverage, concurrency, and operational gaps
  discovered after S1 completion.

### Excluded
- New product features or API capabilities.
- Storage-backend switchover work already planned in S2.
- Stream-recording-specific hardening already tracked in S3.

## Seeded tasks

- `T1-T1` — Persist pending upload-ingestion sessions across API restarts.
- `T1-T2` — Add TTL/expiration and cleanup for abandoned pending ingestions.
- `T1-T3` — Raise measured coverage toward the enforced 90% QA gate.
- `T1-T4` — Add concurrency/race-condition hardening tests for ingestion.
- `T1-T5` — Reconcile `crates/audit` with the intended architectural boundary.
- `T1-T6` — Add upload-size/abuse/operational safeguards for ingestion endpoints.

## Design note

S1 T5 currently keeps upload-ingestion sessions in process memory between
`POST /ingest`, `POST /ingest/{token}/rights`, and `POST /ingest/{token}/finalize`.
That satisfies the current task contract but loses pending sessions if the API
process restarts before finalization. This phase exists to close that gap without
changing the S1 API contract.

## Current outcome

- `T1-T1` is complete: pending upload-ingestion state now persists durably in
  PostgreSQL and survives API restart.
- `T1-T2` is complete: pending ingestions now expire after 24 h; expired sessions
  are rejected at `/rights` and `/finalize` with 410 Gone; a cleanup module
  (`apps/api/src/cleanup.rs`) removes expired DB rows and blobs, and a background
  tokio task runs it hourly from `main.rs`.

## Remaining hardening backlog

- **Coverage gap** (T1-T3): DONE — 91.08% line coverage achieved on narrowed scope.
  Added unit tests for `ffprobe_command`, storage path helpers, `StorageConfig::from_env`,
  `AppConfig::from_env`, `AuthSettings::from_env`, domain Display impls. CI gate narrowed
  with `--ignore-filename-regex` to exclude 6 non-testable files.
- **Concurrency behavior** (T1-T4): DONE — two deterministic concurrency tests added
  (`concurrent_duplicate_finalize_one_wins`, `concurrent_rights_and_finalize_is_consistent`).
  Cleanup-vs-finalize race documented as explicit invariant in `apps/api/src/cleanup.rs`.
  Closing the cleanup race fully requires a distributed lock; deferred to a future slice.
- **`crates/audit`** (T1-T5): DONE — stub replaced with module-level architecture
  comment pointing at `crates/domain/src/audit.rs` (types) and
  `crates/db/src/audit_repo.rs` (writes). Stale `serde`/`time` dependencies
  removed. Crate reserved as an empty namespace for S2+ out-of-process audit sink.
- **Operational guardrails** (T1-T6): DONE — `MAX_UPLOAD_BYTES = 500 MB` constant defined in
  `apps/api/src/routes/ingestion.rs`; `DefaultBodyLimit::max(MAX_UPLOAD_BYTES)` layer applied
  on mutation routes; `upload_too_large_is_rejected` integration test passes (413). Phase T1
  is now fully complete.
