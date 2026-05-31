# Plan: Asset Intake — Platform Ingest (primary) + Stream Recording (S3b)

> **2026-05-31 REPLAN (S3-REPLAN).** The S3 primary intake use case changed. It is
> **not** "a client points their live encoder at DubBridge" (RTMP/SRT capture). It
> is: **the content owner provides credentials to their own platform account
> (YouTube, Vimeo, …) and DubBridge downloads the owner's content on their behalf**
> — legal and authorized because the client owns the content. RTMP/SRT live
> recording is demoted to a **deferred sub-slice (S3b)**, valid only for clients who
> produce live broadcasts. The new primary architecture is governed by **ADR-025**.
>
> This file is retained (filename unchanged for reference continuity) but now covers
> both paths. Sections that describe the FFmpeg recorder are explicitly marked
> **[S3b — DEFERRED]**. The shared foundation already built (S3-T1 domain, S3-T2
> migrations, the ADR-021 bridge, the S1 `finalize_ingestion_core` reuse) is
> **reused as-is** by the platform-ingest path.

**Roadmap position:** Slice **S3** — see `docs/plan/roadmap.md`. Placed after S0
(API client authentication), S1 (ingestion finalize + `StorageAdapter`), and the H1
governance-atomicity gate. S2 (storage switchover) is a prudent predecessor before
heavy writes. Intake remains ahead of downstream ML because it widens intake and
converges on the S1 rights gate.

## Reuse assessment (what S3-T1/S3-T2 already give the platform path)

| Built artifact | Reuse under platform-ingest replan |
|---|---|
| `crates/domain/src/recording.rs` (state machine, `StartRecordingCommand::validate` fail-closed) | **Reuse the pattern, extend the model.** The fail-closed rights validation and the audit posture are directly reusable. A new `PlatformIngestSession` aggregate with a download-shaped status set (ADR-025) sits alongside `RecordingSession`; the `Rtmp/Srt` `SourceProtocol` stays for S3b. |
| `ArtifactKind::RecordedStreamMedia` + `parse_artifact_kind()` | **Reuse + extend.** Add `DownloadedPlatformMedia`; `parse_artifact_kind` gains one arm. F3 fix already in place. |
| `AuditEvent` (`ingest_token: Option<Uuid>`, `recording_session_id`) | **Reuse + extend.** Already generalized away from ingestion-only. Add platform-ingest `AuditEventKind` variants and (optionally) an `ingest_session_id` correlation field. |
| `0008_create_recording_sessions.sql`, `0009_alter_audit_events_for_recording.sql` | **Reuse as-is for S3b.** Platform ingest adds a sibling `platform_ingest_sessions` migration (new task). No rework of 0008/0009. |
| ADR-021 bridge + S1 `finalize_ingestion_core` | **Reuse unchanged.** Producer-agnostic; platform download bridges through the same gate (ADR-021 generalized). |

## Objective (REPLAN 2026-05-31)

**Primary (S3).** Add an **owner-authorized platform ingester**: given a content
owner's credentials to their own platform account (YouTube first, Vimeo/others
later) and a reference to an item they own, DubBridge **resolves ownership +
metadata, downloads the media to local staging, uploads the file to storage, and
incorporates it as an asset through the existing fail-closed ingestion path**
(ADR-021 + S1 `finalize_ingestion_core`). No downloaded media becomes a processable
asset without a valid rights basis and a valid owner credential, both validated
**before any bytes transfer** (ADR-025).

**Sub-case (S3b — DEFERRED).** Retain the capability to capture an authorized live
stream (RTMP/SRT) via the FFmpeg subprocess recorder, for the minority of clients
who produce live broadcasts. Same asset boundary, same rights gate; built only when
a real live-broadcast need exists (ADR-019/020/022).

## Scope

### Included — Platform ingester (primary, S3)
- Domain extension for a `PlatformIngestSession` aggregate + download-shaped status
  set, plus `ArtifactKind::DownloadedPlatformMedia` and platform-ingest
  `AuditEventKind` variants (ADR-025). Reuses S3-T1 fail-closed validation pattern.
- `crates/connectors` control-plane crate: `PlatformConnector` trait + pure request
  builder / IO executor split (mirrors `crates/media`/`crates/recorder`).
- YouTube connector (v1), preceded by a retrieval-mechanism spike (the T0c-style
  gate for the platform path).
- Owner-credential handling by reference in the secrets store, redacted in logs
  (ADR-018, ADR-025).
- `PlatformIngestJob` in `crates/jobs`; worker-runner consumption that downloads,
  bridges, and audits.
- SQL migration for `platform_ingest_sessions`.
- API endpoints to create / start / read a platform-ingest session.
- Deterministic unit tests (request builder, validation, state machine) + an
  integration test against the connector behind a recorded/sandbox fixture.

### Included — Stream recording (S3b, DEFERRED)
- Everything previously specified for the FFmpeg recorder path (the `crates/recorder`
  crate, supervisor, segments/assembly, recording API, worker integration, and the
  `lavfi` integration tests). Marked **[S3b — DEFERRED]** throughout this plan and
  in the task ledger (S3-T3 … S3-T8 are REPLANNED into S3b).

### Excluded (deferred, both paths)
- Vimeo and other connectors beyond YouTube v1 (additive behind the trait).
- GStreamer in-process pipeline (future upgrade, ADR-019).
- MediaMTX/SRS sidecar deployment (fallback, ADR-019).
- RTSP, HLS pull, WebRTC sources (ADR-022 follow-ups).
- Playback HTTP server.
- Per-segment assets and manifest-backed assets (S3b uses whole-session assembly).
- Transcoding / probe / ASR / TTS of the resulting asset (downstream slices own
  these once the asset exists).

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

The bridge (ADR-021) must call the S1 ingestion finalize logic. S1 tasks T4-T6 are
complete, Task T0 extracted `finalize_ingestion_core`, and H1 completed the
app-neutral move plus atomicity/audit hardening required before recording expands
that path.

Task T0c resolved the v1 output contract. V1 captures to a local HLS fMP4 staging
package (`init.mp4` + `session.m3u8` + `.m4s` segments), requires a graceful stop
to finalize the manifest, remuxes that manifest into one assembled MP4, and bridges
exactly one asset per recording session. Recorder code should implement that
contract and must not silently drift from ADR-020/021.

Recording API endpoints also depend on S0 API client authentication. This is
separate from ADR-022 source authentication: S0 identifies the Axum caller;
ADR-022 validates RTMP stream keys or SRT passphrases before capture.

## Consistency review inputs (2026-05-31)

This plan was adjusted after a full-repository review
(`docs/audit/2026-05-31-project-consistency-review.md`). Three findings are
integrated directly into the tasks below:

- **F1 — audit reuse.** Audit events are persisted via `crates/domain::audit::AuditEvent`
  + `crates/db::audit_repo`, exactly like S1. T1-T5 removed the conflicting
  `crates/audit` placeholder type. H1 now owns the shared durable-emission boundary
  required by ADR-018.
- **F2 — audit schema.** `audit_events.ingest_token` is `NOT NULL` and
  `AuditEventKind` is ingestion-only, so recording lifecycle events (which occur
  before any ingest token) cannot be stored today. T1 + T2 generalize this.
- **F3 — artifact-kind parsing.** `db::artifact_repo::find_original_by_ingest_token`
  hardcodes `ArtifactKind::OriginalMedia` (dead `if/else`). Adding
  `RecordedStreamMedia` requires a real parser, or recorded artifacts are mislabeled.

## Preparatory and housekeeping tasks (consolidated follow-ups)

Project-general follow-ups and the later roadmap/ADR consolidation are reflected in
this plan's task list:

- **Task 0b — Reconcile `crates/audit` (F1/F8).** Complete via T1-T5. The duplicate
  type is gone.
- **H1 — Governance atomicity + durable audit hardening.** Blocking external gate
  before recording expands the shared finalize path.
- **Task 0c — Resolve the recording output contract.** Complete on 2026-05-31.
  V1 is local HLS fMP4 staging plus one assembled MP4 bridged once per session.
- **Task 9 — Align docker-compose Rust pin (F7).** Low priority, fully independent
  of recording; can be done at any time (warm-up or final cleanup).

Full findings: `docs/audit/2026-05-31-project-consistency-review.md`.

## Governing ADRs
- ADR-006: PostgreSQL metadata + object storage for binaries.
- ADR-008: Rights ledger is a mandatory, fail-closed precondition.
- ADR-018: Structured observability; governance events are traceable.
- ADR-019: Recording engine = FFmpeg subprocess orchestration.
- ADR-020: Recording session lifecycle and segment model. Accepted after T0c.
- ADR-021: Recording-to-asset ingestion bridge with fail-closed rights. Accepted
  after T0c.
- ADR-022: Source protocols (RTMP + SRT) and ingest authentication.
- ADR-023: API client authentication and principal propagation.

## Platform ingester — design decisions (primary, S3, ADR-025)

### Connector boundary as a Rust crate
`crates/connectors` exposes a `PlatformConnector` trait. Each platform is one impl.
The URL/request builder is a **pure function** (unit-testable, no network); a
separate executor performs authenticated IO — the same discipline as
`media::ffprobe_command` and `recorder::ffmpeg`. The crate depends only on
`crates/domain` + `crates/config`; **no DB dependency**.

### Domain generalization (not a rewrite of recording.rs)
A new `PlatformIngestSession` aggregate carries `owner_id`, a `SourceRef`
(`platform` + `external_id`), an owner `credential_ref`, the `RightsBasis`, and a
download-shaped status: `Requested → RightsValidated → Resolving → Downloading →
Downloaded`, plus `Failed` / `RejectedMissingRights`. It **reuses the fail-closed
validation pattern** proven in `StartRecordingCommand::validate`. `RecordingSession`
and its `Rtmp/Srt` `SourceProtocol` are left intact for S3b.

### Owner-authorized credentials (fail-closed, redacted)
Owner credentials are scoped to the owner's own account, stored **by reference** in
the secrets store, never in plaintext, and redacted in logs/traces (ADR-018; the
secrets-store mechanism itself is an open follow-up, no dedicated ADR yet). A
session lacking a valid `RightsBasis` **or** a valid credential is
rejected before any download — the platform twin of the capture-edge gate.

### Same asset boundary (ADR-021, generalized)
The downloaded file is bridged through `finalize_ingestion_core` with
`ArtifactKind::DownloadedPlatformMedia`, one SHA-256, and one `ingest_token` per
session. No second ingestion gate. Idempotency reuses the
`artifact_records.ingest_token` UNIQUE constraint.

### YouTube retrieval mechanism is spiked first
YouTube has no single Data-API endpoint returning original bytes for arbitrary
videos. The legitimate owner-download mechanism (OAuth scope, ownership check,
retrieval) is validated by a throwaway internal spike before the connector is
implemented — the T0c-style gate for this path. The trait keeps the mechanism
swappable.

## Platform ingester — affected files (primary, S3)

### crates/connectors/ (new crate)
- `Cargo.toml` + workspace member
- `src/lib.rs` — `PlatformConnector` trait, `Platform`, `SourceRef`,
  `ConnectorCredential`, `RemoteMediaMetadata`, `DownloadedMedia`, `ConnectorError`
- `src/youtube.rs` — YouTube connector v1 (pure request builder + executor)

### crates/domain/src/
- `platform_ingest.rs` (new) — `PlatformIngestSession`, status set,
  `StartPlatformIngestCommand::validate()` (fail-closed)
- `artifact.rs` — add `ArtifactKind::DownloadedPlatformMedia` (+ `parse` arm)
- `audit.rs` — add platform-ingest `AuditEventKind` variants
- `lib.rs` — re-export `platform_ingest`

### crates/db/src/
- `platform_ingest_repo.rs` (new) — persistence for ingest sessions
- `artifact_repo.rs` — `parse_artifact_kind` already covers new kinds (F3)
- `audit_repo.rs` — bind any new correlation column if added

### crates/jobs/src/, crates/storage/src/, crates/config/src/
- `jobs/lib.rs` — `PlatformIngestJob`
- `storage/lib.rs` — `ingests/{session_id}/` prefix + downloaded-object put helper
- `config/lib.rs` — connector config (timeouts, staging path, secrets-store ref)

### apps/api/src/, apps/worker-runner/src/
- `routes/platform_ingest.rs` (new), `dto/platform_ingest.rs` (new), router mount
- `handlers/platform_ingest.rs` (new) — resolve → download → bridge → audit

### infra/migrations/
- `<next>_create_platform_ingest_sessions.sql` (new)

## Affected Files [S3b — DEFERRED] (FFmpeg live recorder)

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
- `src/session.rs` — `RecordingSupervisor` (spawn, monitor, graceful stop, restart,
  trigger assembly)
- `src/segments.rs` — staging path templater + retention policy
- `src/assemble.rs` (new) — remux finalized manifest to one assembled MP4

### crates/jobs/src/
- `lib.rs` — `StreamRecordingJob`

### crates/storage/src/
- `lib.rs` — `recording_prefix(session_id)`, final assembled-object put helper

### crates/config/src/
- `lib.rs` — recorder config fields

### apps/api/src/
- `state.rs` — extend `AppState` if needed (job queue handle)
- `dto/recording.rs` (new)
- `routes/recording.rs` (new) — create / start / stop / get
- `main.rs` — mount recording router

### apps/worker-runner/src/
- `main.rs` — register `StreamRecordingJob` handler
- `handlers/recording.rs` (new) — supervise capture, assemble, upload, bridge, audit

### infra/migrations/
- `<next>_create_recording_sessions.sql` (new; allocate after H1 migrations)
- `<next+1>_alter_audit_events_for_recording.sql` (new) — relax `ingest_token` to
  nullable, add nullable `recording_session_id` FK (F2)


### tests/
- Package-owned recording integration suite (location fixed by T0c/T8; the workspace
  root is not a Cargo package)

## Design Decisions [S3b — DEFERRED] (FFmpeg live recorder)

### Fail-closed before capture (ADR-020/021)
Rights are validated at session creation. A session with an invalid `RightsBasis`
is rejected (`RejectedMissingRights`) **before any FFmpeg process is spawned**. No
unauthorized stream is ever written to disk.

### Engine boundary mirrors `crates/media` (ADR-019)
`recorder::ffmpeg::ffmpeg_record_command` is a pure function returning the argument
vector, unit-testable without spawning anything — exactly like
`media::ffprobe_command`. The supervisor is the only part that touches
`tokio::process`.

### Recording output contract (resolved by T0c, ADR-020/021)
V1 uses local HLS fMP4 segmented staging during capture and one assembled MP4 as the
asset boundary. Segment files are internal recorder staging only; they are not
uploaded as assets and do not require a `recording_segments` table in v1.

The validated sequence is:

1. capture locally to `init.mp4` + `session.m3u8` + `.m4s`
2. stop gracefully with `q\n`
3. remux `session.m3u8` to one assembled MP4
4. compute one SHA-256 over that MP4
5. upload one final object
6. call the reused finalize path once with one session-scoped `ingest_token`

Crash-open manifests are not auto-bridged in v1; failed sessions keep bounded local
staging for later cleanup or a future recovery workflow.

### Idempotency
The bridge reuses S1's `ingest_token` + `artifact_records.ingest_token` UNIQUE
constraint. T0c fixed the cardinality at **one ingest token per recording session**,
so a retried bridge for the same session does not create duplicates.

### Source validation (ADR-022)
Only `rtmp://` and `srt://` schemes are accepted; URLs are validated/normalized
before reaching the command builder to prevent argument injection. RTMP stream
keys and SRT passphrases are required and are redacted in all logs/traces.

### Audit events reuse the hardened shared path (F1/F2, ADR-018)
Recording emits governance audit events through the H1 shared audit-emission
boundary backed by `domain::audit::AuditEvent` + `db::audit_repo`. Because recording
events occur before any ingest token, `AuditEvent.ingest_token` becomes `Option<Uuid>` and a
`recording_session_id: Option<Uuid>` correlation field is added (ADR-018 already
names `recording_session_id` as a correlation id). The `audit_events` table is
altered to match (the audit generalization migration). This is a non-breaking
relaxation for S1, which
always supplies an `ingest_token`.

### Artifact-kind parsing fix (F3)
Adding `ArtifactKind::RecordedStreamMedia` requires
`db::artifact_repo::find_original_by_ingest_token` to parse the stored `kind`
correctly (today it hardcodes `OriginalMedia` via a dead `if/else`). A single
`parse_artifact_kind(&str) -> ArtifactKind` is introduced and reused. This protects
lineage integrity: a recorded artifact must never read back as an upload.

## Module Dependencies

### Platform ingester (primary, S3)
```
apps/api          → crates/domain, crates/jobs, crates/db, crates/auth, crates/config, crates/observability
apps/worker-runner→ crates/domain, crates/connectors, crates/jobs, crates/db,
                    crates/storage, crates/config, crates/observability
crates/connectors → crates/domain, crates/config   (connector boundary; no DB)
crates/jobs       → crates/domain
crates/storage    → crates/config
crates/domain     → (no internal deps)
```

### Live recorder [S3b — DEFERRED]
```
apps/worker-runner→ crates/recorder (added when S3b is built)
crates/recorder   → crates/domain, crates/config   (engine boundary; no DB)
```

> Audit persistence remains backed by `crates/db::audit_repo` +
> `crates/domain::audit`, with the H1 shared emission wrapper in place before S3
> adds lifecycle events.

## Lines Affected After Implementation

Tracked per-task in `docs/tasks/stream-recording-ingest.md`. Updated after each
completed task (used as the crash-safe progress ledger per the workflow).
