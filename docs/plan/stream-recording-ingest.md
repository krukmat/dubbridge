# Plan: Stream Recording Ingest

**Roadmap position:** Slice **S3** — see `docs/plan/roadmap.md`. Placed after S0
(API client authentication), S1 (ingestion finalize + `StorageAdapter`) and S2
(storage switchover), ahead of all downstream ML stages, because recording only
widens intake and converges on the S1 rights gate.

## Objective

Add a stream-recording capability that captures an authorized live stream
(RTMP or SRT), writes it to disk as segmented media via an FFmpeg subprocess
supervised by Rust, uploads the result to object storage, and **incorporates it
into the platform as an asset through the existing fail-closed ingestion path**.
No recorded media may become a processable asset without a valid rights basis.

## Scope

### Included
- Domain types for recording sessions and the recording state machine
  (`crates/domain::recording`), plus a new `ArtifactKind::RecordedStreamMedia`.
- `crates/recorder` control-plane crate: FFmpeg command builder, process
  supervisor (start / graceful stop / restart), segment path templating + retention.
- `StreamRecordingJob` in `crates/jobs`; worker-runner consumption.
- Object-storage `recordings/` prefix and segment upload in `crates/storage`.
- Recording→asset bridge that reuses the S1 finalize path (ADR-021).
- API endpoints to create / start / stop / read recording sessions.
- SQL migration for `recording_sessions`.
- Deterministic unit tests (command builder, validation, state machine) and an
  integration test using a synthetic local source (FFmpeg `testsrc`/`lavfi`).

### Excluded (deferred)
- GStreamer in-process pipeline (future upgrade, ADR-019).
- MediaMTX/SRS sidecar deployment (fallback, ADR-019).
- RTSP, HLS pull, WebRTC sources (ADR-022 follow-ups).
- Playback HTTP server (MediaMTX `internal/playback` equivalent).
- Per-segment incremental bridging (v1 bridges on session stop; per-segment is a
  follow-up — see Design Decisions).
- Transcoding / probe / ASR / TTS of the recorded asset (existing downstream
  slices already own these once the asset exists).

## Confidentiality and publication

The open-source/competitive evaluation and reference review behind this slice
(`docs/proposals/stream-recording-module.md`) are **INTERNAL and must not be
published** or used in customer-facing/marketing materials.

- Public artifacts **must not state or imply** that the design copies or derives from
  any third-party project. The recorder is an **original, clean-room Rust
  implementation** using standard segmented-recording techniques; **no third-party
  source code is used**.
- Third-party projects (MediaMTX, SRS, GStreamer) are named only in internal
  engineering docs as references. FFmpeg is a runtime dependency acknowledged per its
  LGPL terms — a tooling dependency, not an architecture derivation.
- The reference spike (`spikes/recorder-sandbox/`) is internal, contains no
  third-party code, and is never published.
- Gate (applies to T8 and any docs/publication task): no public artifact contains
  "copy / clone / donor"-style framing tying the architecture to a competitor.

## Prerequisite

The bridge (ADR-021) must call the S1 ingestion finalize logic. Therefore **S1
tasks T4–T6 must be completed, or the finalize logic must be extracted** into a
transport-agnostic function reusable by both the HTTP upload handler and the
recording bridge. Task T0 below makes this explicit.

Recording API endpoints also depend on S0 API client authentication. This is
separate from ADR-022 source authentication: S0 identifies the Axum caller;
ADR-022 validates RTMP stream keys or SRT passphrases before capture.

## Consistency review inputs (2026-05-31)

This plan was adjusted after a full-repository review
(`docs/audit/2026-05-31-project-consistency-review.md`). Three findings are
integrated directly into the tasks below:

- **F1 — audit reuse.** Audit events are persisted via `crates/domain::audit::AuditEvent`
  + `crates/db::audit_repo`, exactly like S1. The placeholder `crates/audit` crate is
  **not** used (it is an unrelated stub; reconciling it is a separate follow-up).
- **F2 — audit schema.** `audit_events.ingest_token` is `NOT NULL` and
  `AuditEventKind` is ingestion-only, so recording lifecycle events (which occur
  before any ingest token) cannot be stored today. T1 + T2 generalize this.
- **F3 — artifact-kind parsing.** `db::artifact_repo::find_original_by_ingest_token`
  hardcodes `ArtifactKind::OriginalMedia` (dead `if/else`). Adding
  `RecordedStreamMedia` requires a real parser, or recorded artifacts are mislabeled.

## Preparatory and housekeeping tasks (consolidated follow-ups)

Two project-general follow-ups from the consistency review are consolidated into
this plan's task list. Rationale: this is the active plan, and one of them touches
the audit foundation the recording slice builds on. Placement reflects priority:

- **Task 0b — Reconcile `crates/audit` (F1/F8).** Medium priority. Scheduled
  **before Task 1**, because the recording slice writes audit events and Task 1
  changes `domain::audit`; reconciling first avoids entrenching the divergent audit
  path. Recommended-early, not a hard blocker.
- **Task 9 — Align docker-compose Rust pin (F7).** Low priority, fully independent
  of recording; can be done at any time (warm-up or final cleanup).

Full findings: `docs/audit/2026-05-31-project-consistency-review.md`.

## Governing ADRs
- ADR-006: PostgreSQL metadata + object storage for binaries.
- ADR-008: Rights ledger is a mandatory, fail-closed precondition.
- ADR-018: Structured observability; governance events are traceable.
- ADR-019: Recording engine = FFmpeg subprocess orchestration.
- ADR-020: Recording session lifecycle and segment model.
- ADR-021: Recording-to-asset ingestion bridge with fail-closed rights.
- ADR-022: Source protocols (RTMP + SRT) and ingest authentication.
- ADR-023: API client authentication and principal propagation.

## Affected Files

### crates/domain/src/
- `lib.rs` — re-export `recording`
- `recording.rs` (new) — `RecordingSessionId`, `RecordingSession`,
  `RecordingStatus`, `SourceProtocol`, `RecordingSource`, `StartRecordingCommand`,
  `RecordingError`
- `artifact.rs` — add `ArtifactKind::RecordedStreamMedia` + `parse`/`FromStr`
- `audit.rs` — add recording `AuditEventKind` variants; make `ingest_token`
  optional and add `recording_session_id: Option<Uuid>` (F1/F2)

### crates/db/src/
- `artifact_repo.rs` — replace the dead `if/else` with a `parse_artifact_kind`
  helper so `RecordedStreamMedia` round-trips correctly (F3)
- `audit_repo.rs` — bind the new nullable `ingest_token` and `recording_session_id`
  columns (F2)

### crates/recorder/ (new crate)
- `Cargo.toml`
- `src/lib.rs` — re-exports, `RecorderConfig`
- `src/ffmpeg.rs` — `ffmpeg_record_command(source, output_template, opts) -> Vec<String>`
- `src/session.rs` — `RecordingSupervisor` (spawn, monitor, graceful stop, restart)
- `src/segments.rs` — path templater + retention policy

### crates/jobs/src/
- `lib.rs` — `StreamRecordingJob`

### crates/storage/src/
- `lib.rs` — `recording_prefix(session_id)`, segment put helper

### crates/config/src/
- `lib.rs` — recorder config fields

### apps/api/src/
- `state.rs` — extend `AppState` if needed (job queue handle)
- `dto/recording.rs` (new)
- `routes/recording.rs` (new) — create / start / stop / get
- `main.rs` — mount recording router

### apps/worker-runner/src/
- `main.rs` — register `StreamRecordingJob` handler
- `handlers/recording.rs` (new) — supervise capture, upload, bridge, audit

### infra/migrations/
- `0005_create_recording_sessions.sql` (new)
- `0006_alter_audit_events_for_recording.sql` (new) — relax `ingest_token` to
  nullable, add nullable `recording_session_id` FK (F2)
- `0007_create_recording_segments.sql` (deferred — only if per-segment tracking
  is adopted; v1 bridges whole-session)

### tests/
- `integration/recording_test.rs` (new)

## Design Decisions

### Fail-closed before capture (ADR-020/021)
Rights are validated at session creation. A session with an invalid `RightsBasis`
is rejected (`RejectedMissingRights`) **before any FFmpeg process is spawned**. No
unauthorized stream is ever written to disk.

### Engine boundary mirrors `crates/media` (ADR-019)
`recorder::ffmpeg::ffmpeg_record_command` is a pure function returning the argument
vector, unit-testable without spawning anything — exactly like
`media::ffprobe_command`. The supervisor is the only part that touches
`tokio::process`.

### Whole-session bridging for v1 (ADR-021)
On graceful stop (`Capturing → Stopping → Recorded`), the supervisor finalizes the
output, computes a SHA-256 checksum, uploads to `recordings/{session_id}/...`, and
calls the reused finalize path once to register a single
`ArtifactKind::RecordedStreamMedia` artifact + asset + rights + audit. Per-segment
bridging (multiple artifacts during a long capture) is a deliberate follow-up.

### Idempotency
The bridge reuses S1's `ingest_token` + `artifact_records.ingest_token` UNIQUE
constraint, so a retried bridge for the same recording does not create duplicates.

### Source validation (ADR-022)
Only `rtmp://` and `srt://` schemes are accepted; URLs are validated/normalized
before reaching the command builder to prevent argument injection. RTMP stream
keys and SRT passphrases are required and are redacted in all logs/traces.

### Audit events reuse the S1 path, generalized (F1/F2, ADR-018)
Recording emits governance audit events through the same `domain::audit::AuditEvent`
+ `db::audit_repo` path as S1. Because recording events occur before any ingest
token, `AuditEvent.ingest_token` becomes `Option<Uuid>` and a
`recording_session_id: Option<Uuid>` correlation field is added (ADR-018 already
names `recording_session_id` as a correlation id). The `audit_events` table is
altered to match (migration `0006`). This is a non-breaking relaxation for S1, which
always supplies an `ingest_token`.

### Artifact-kind parsing fix (F3)
Adding `ArtifactKind::RecordedStreamMedia` requires
`db::artifact_repo::find_original_by_ingest_token` to parse the stored `kind`
correctly (today it hardcodes `OriginalMedia` via a dead `if/else`). A single
`parse_artifact_kind(&str) -> ArtifactKind` is introduced and reused. This protects
lineage integrity: a recorded artifact must never read back as an upload.

## Module Dependencies

```
apps/api          → crates/domain, crates/jobs, crates/db, crates/auth, crates/config, crates/observability
apps/worker-runner→ crates/domain, crates/recorder, crates/jobs, crates/db,
                    crates/storage, crates/config, crates/observability
crates/recorder   → crates/domain, crates/config   (engine boundary; no DB)
crates/jobs       → crates/domain
crates/storage    → crates/config
crates/domain     → (no internal deps)
```

> Audit persistence reuses `crates/db::audit_repo` (+ `crates/domain::audit`), not
> the `crates/audit` stub (F1). `crates/audit` stays out of this slice's graph.

## Lines Affected After Implementation

Tracked per-task in `docs/tasks/stream-recording-ingest.md`. Updated after each
completed task (used as the crash-safe progress ledger per the workflow).
