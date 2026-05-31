# Tasks: Asset Intake — Platform Ingest (primary) + Stream Recording (S3b)

> **2026-05-31 REPLAN (S3-REPLAN).** Primary S3 intake is now owner-authorized
> **platform download** (ADR-025), not RTMP/SRT live capture. Shared foundation
> **S3-T0/T0b/T0c/T1/T2 stay DONE and are reused**. **S3-T3 … S3-T8 are marked
> `[~] REPLANNED → S3b`** (deferred FFmpeg live recorder). New primary-path tasks
> **S3-P1 … S3-P5** are added below. Build order for S3: P1 → P5. S3b is built only
> when a real live-broadcast client need exists.

Governing plan: `docs/plan/stream-recording-ingest.md`
Governing ADRs: ADR-006, ADR-008, ADR-018, ADR-021, ADR-023, **ADR-025 (primary)**;
ADR-019/020/022 (S3b live recorder, deferred)
Blocking foundation gate: `docs/tasks/h1-governance-atomicity-hardening.md`
Consistency review integrated: `docs/audit/2026-05-31-project-consistency-review.md`
(findings F1 audit reuse, F2 audit schema, F3 artifact-kind parsing)

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

## Default model recommendation (per AGENTS.md)
- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4` (escalate to `Claude Opus 4.1` for T3/T7 only if
  subprocess supervision logic stalls under Sonnet).

---

## Task 0 — Make S1 finalize path reusable (prerequisite)

**Effort:** M · **Complexity:** Medium · **Depends on:** S1 T4–T6
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Extract the S1 ingestion finalize logic (asset + rights_record + artifact_record +
audit_event, with fail-closed validation and idempotency) into a
transport-agnostic function callable by both the HTTP upload handler and the
recording bridge (ADR-021).

### Acceptance criteria
- A single function (e.g. `finalize_ingestion(cmd, deps) -> Result<AssetSummary, IngestionError>`)
  exists and is used by the S1 upload handler.
- It accepts an injectable `ArtifactKind` so the bridge can pass `RecordedStreamMedia`.
- Existing S1 tests still pass; no behavior change for uploads.
- `cargo check` + `cargo test` clean.

### Files affected
- `apps/api/src/routes/ingestion.rs` (refactor to call the extracted fn)
- `crates/db` / new ingestion service module (location decided during T0)

### Status: [x] DONE

Files affected:
- `apps/api/src/ingestion_service.rs` (new, S3-T0) — `finalize_ingestion_core` + `IngestionServiceError`
- `apps/api/src/routes/ingestion.rs` lines 197–260 (S3-T0) — handler thinned to load pending record, check expiry, call core
- `apps/api/src/lib.rs` (S3-T0) — `pub mod ingestion_service` registered
- `apps/api/Cargo.toml` (S3-T0) — `thiserror = "2.0"` added

Follow-up: H1 moves this first extraction into an app-neutral shared boundary before
the worker-runner recording bridge consumes it.

---

## Task 0b — Reconcile `crates/audit` (F1/F8)

**Effort:** S–M · **Complexity:** Medium · **Depends on:** nothing (S1 audit code present)
**Priority:** Medium — do **before Task 1**. Recommended-early, not a hard blocker.
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Remove the duplicate/conflicting `AuditEvent` so the workspace has a single audit
type and path. Before T1-T5, `crates/domain::audit::AuditEvent` was canonical while
`crates/audit` held an unrelated placeholder (F1), and the S1 plan's promised
`AuditLogger` was never built (F8).

### Resolution
- T1 Task 5 removed the unrelated placeholder `AuditEvent`.
- `crates/domain::audit::AuditEvent` remains the only audit type.
- `crates/audit` remains an empty reserved namespace. H1 Task 3 now owns the shared
  durable-emission wrapper decision required by ADR-018.

### Acceptance criteria
- Exactly one `AuditEvent` type exists in the workspace.
- The reserved `crates/audit` namespace documents the canonical domain + DB path and
  contains no conflicting type or logic.
- `cargo check --workspace` clean; existing S1 tests still pass.

### Files affected
- `crates/audit/src/lib.rs`
- `crates/audit/Cargo.toml`

### Status: [x] DONE via T1 Task 5 — duplicate audit type removed. 2026-05-31.

---

## Task 0c — Resolve and spike the v1 recording output contract

**Effort:** M · **Complexity:** High · **Depends on:** H1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4` with thinking On

### Objective
Remove the contradiction between segmented capture and one whole-session asset
bridge before recorder implementation.

### Scope
- Run an internal local FFmpeg spike with synthetic `lavfi` input.
- Validate the actual command shape for fMP4 segmentation, graceful stop, crash
  recovery behavior, upload timing, and retention timing.
- Select and document one v1 artifact model:
  - per-segment assets
  - whole-session assembled multimedia artifact
  - manifest-backed session artifact with segment lineage
- Define artifact cardinality, `ingest_token` cardinality, checksum scope, and any
  required `recording_segments` schema.
- Update ADR-020, ADR-021, this plan, and downstream task scopes with the decision.

### Acceptance criteria
- ADR-020 and ADR-021 describe one coherent validated v1 output model.
- The selected model produces an explicit downstream asset contract.
- Required migration files and idempotency rules are known before Task 1 starts.
- The spike contains no third-party source and is not published.

### Files affected
- `spikes/recorder-sandbox/` (temporary, internal)
- `docs/adr/ADR-020-recording-session-lifecycle-and-segment-model.md`
- `docs/adr/ADR-021-recording-to-asset-ingestion-bridge-fail-closed.md`
- `docs/plan/stream-recording-ingest.md`
- `docs/tasks/stream-recording-ingest.md`

### Status: [x] DONE — local FFmpeg spike validated HLS fMP4 staging (`init.mp4` + `session.m3u8` + `.m4s`), graceful `q` stop writes `ENDLIST`, clean remux to one assembled MP4 works, hard-kill leaves completed segments but an open manifest. V1 fixed as local segmented staging plus one assembled MP4 bridged once per session; `recording_segments` deferred; one `ingest_token` and one checksum per session artifact. ADR-020/021 accepted; S3 plan updated. 2026-05-31.

---

## Task 1 — Domain types for recording sessions

**Effort:** M · **Complexity:** Medium · **Depends on:** H1, Task 0c
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Model the recording session aggregate, its fail-closed state machine, and the
source model in `crates/domain`.

### Scope
- `recording.rs` (new): `RecordingSessionId`, `RecordingSession`, `RecordingStatus`
  (`Requested`, `RightsValidated`, `Capturing`, `Stopping`, `Recorded`, `Failed`,
  `RejectedMissingRights`), `SourceProtocol {Rtmp, Srt}`, `RecordingSource`
  (protocol + validated URL + credential reference), `StartRecordingCommand`
  (reuses `RightsBasis`), `RecordingError`.
- `artifact.rs`: add `ArtifactKind::RecordedStreamMedia` (+ `Display`) and a
  `parse_artifact_kind(&str) -> ArtifactKind` (used by the repo fix in T5 — F3).
- `audit.rs` (F1/F2): add recording `AuditEventKind` variants
  (`RecordingSessionCreated`, `RecordingRejectedMissingRights`,
  `RecordingCaptureStarted`, `RecordingRecorded`, `RecordingFailed`,
  `RecordingBridgedToAsset`); change `AuditEvent.ingest_token` to `Option<Uuid>`
  and add `recording_session_id: Option<Uuid>`; add a constructor for recording
  events. Update S1 call sites to pass `Some(token)` (mechanical).
- `lib.rs`: re-export `recording`.

### Acceptance criteria
- `StartRecordingCommand::validate()` returns `RejectedMissingRights`-equivalent
  error when `rights_basis` is absent/incomplete (reuses ADR-008 semantics).
- `RecordingStatus` has no path from `Requested`/`RejectedMissingRights` to
  `Capturing` without passing `RightsValidated`.
- Only `Rtmp` and `Srt` exist in `SourceProtocol`.
- `parse_artifact_kind("recorded_stream_media") == RecordedStreamMedia` and
  `parse_artifact_kind("original_media") == OriginalMedia`.
- `AuditEvent` round-trips with `ingest_token = None` + a `recording_session_id`.
- Existing S1 domain tests still pass; `cargo test -p dubbridge-domain` passes.

### Files affected
- `crates/domain/src/recording.rs` (new)
- `crates/domain/src/artifact.rs`
- `crates/domain/src/audit.rs`
- `crates/domain/src/lib.rs`
- `crates/domain/Cargo.toml` (thiserror already used)

### Status: [x] DONE — 2026-05-31

Files affected:
- `crates/domain/src/recording.rs` (new, S3-T1) — `RecordingSessionId`, `RecordingSession`, `RecordingStatus` (7 variants), `SourceProtocol {Rtmp, Srt}`, `RecordingSource`, `StartRecordingCommand::validate()` (fail-closed), `RecordingError`, state-machine transition methods, 11 unit tests
- `crates/domain/src/artifact.rs` lines 1–39 (S3-T1) — `ArtifactKind::RecordedStreamMedia` added; `parse_artifact_kind(&str) -> ArtifactKind` added (F3); 4 unit tests
- `crates/domain/src/audit.rs` (S3-T1) — `ingest_token: Uuid` → `Option<Uuid>`; `recording_session_id: Option<Uuid>` added; 6 recording `AuditEventKind` variants added; `new_recording()` constructor added; 3 unit tests
- `crates/domain/src/lib.rs` line 5 (S3-T1) — `pub mod recording` added
- `crates/audit/src/lib.rs` lines 37–40 (S3-T1) — tracing log updated for `Option<Uuid>` ingest_token

`cargo test -p dubbridge-domain`: 29/29 passed. `cargo check --workspace`: clean.

---

## Task 2 — Migrations: recording_sessions + audit generalization

**Effort:** S · **Complexity:** Low · **Depends on:** Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
- `infra/migrations/<next>_create_recording_sessions.sql`: id (UUID PK),
  source_protocol, source_url, owner/uploader, rights fields (or FK to a rights row
  created at validation), status with CHECK constraint matching `RecordingStatus`,
  asset_id (nullable FK, set on bridge), created_at/updated_at TIMESTAMPTZ
  DEFAULT now().
- `infra/migrations/<next+1>_alter_audit_events_for_recording.sql` (F2): relax
  `ingest_token` to nullable; add `recording_session_id UUID NULL REFERENCES
  recording_sessions(id)`. Ordering matters — `recording_sessions` must exist first.

### Acceptance criteria
- Both migrations apply cleanly on a fresh Postgres (`sqlx migrate run`).
- `recording_sessions.status` CHECK restricts to the seven defined values.
- Credentials are **not** stored in plaintext (reference/secret only).
- After the audit generalization migration, an `audit_events` row can be inserted
  with `ingest_token` NULL and a
  non-null `recording_session_id`; existing ingestion inserts still succeed.

### Files affected
- `infra/migrations/<next>_create_recording_sessions.sql` (new; allocate after H1)
- `infra/migrations/<next+1>_alter_audit_events_for_recording.sql` (new)
- No `recording_segments` migration in v1; deferred until a future segment-native contract exists

### Status: [x] DONE — 2026-05-31

Files affected:
- `infra/migrations/0008_create_recording_sessions.sql` (new, S3-T2)
- `infra/migrations/0009_alter_audit_events_for_recording.sql` (new, S3-T2)

---

## Task 3 — `crates/recorder`: FFmpeg engine boundary

**Effort:** L · **Complexity:** High · **Depends on:** Task 0c, Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`
(escalate to `Opus 4.1` only if supervision logic repeatedly stalls)

### Scope
New crate mirroring `crates/media`'s command-builder pattern (ADR-019, ADR-020):
- `ffmpeg.rs`: `ffmpeg_record_command(source, output_template, opts) -> Vec<String>`
  — pure, deterministic, no process spawn. Encodes the T0c-validated segment/container
  options and per-protocol input options (RTMP key, SRT passphrase) with
  redaction-safe handling.
- `session.rs`: `RecordingSupervisor` over `tokio::process` — spawn, read
  stdout/stderr lines, graceful stop via `q\n` to stdin with bounded timeout,
  bounded restart-with-backoff policy, surfaces lifecycle transitions and triggers
  post-stop assembly.
- `segments.rs`: staging path templater (`%session/%Y-%m-%d/%H-%M-%S`) for
  `init.mp4` + `session.m3u8` + `.m4s`; retention (`delete_after`) helper.
- `assemble.rs` (or equivalent): remux finalized manifest to one assembled MP4.

### Acceptance criteria
- `ffmpeg_record_command` is covered by unit tests asserting exact argument
  vectors for an RTMP and an SRT source (credentials redacted in any log).
- Path templater unit-tested for stable, collision-free keys.
- Supervisor graceful-stop path is unit/integration-tested against a short local
  `lavfi`/`testsrc` capture (no network).
- `cargo check` + `cargo test -p dubbridge-recorder` pass.

### Files affected
- `crates/recorder/Cargo.toml` (new) + workspace member in root `Cargo.toml`
- `crates/recorder/src/{lib,ffmpeg,session,segments,assemble}.rs` (new)

### Status: [~] REPLANNED → S3b — FFmpeg live capture is no longer the primary S3 path (ADR-025); deferred to the S3b live-recording sub-case.

---

## Task 4 — Jobs + storage wiring

**Effort:** M · **Complexity:** Medium · **Depends on:** Task 1, Task 3
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
- `crates/jobs`: `StreamRecordingJob` envelope (session id, source, output config).
- `crates/storage`: final assembled-object put helper to MinIO/S3. Local segment
  files remain worker-local staging in v1. `recording_prefix(session_id)` already
  exists from S1 T4.

### Acceptance criteria
- `StreamRecordingJob` serializes/deserializes (serde round-trip test).
- `recording_prefix` returns `recordings/{session_id}/`.
- `cargo check` passes.

### Files affected
- `crates/jobs/src/lib.rs`
- `crates/storage/src/lib.rs`

### Status: [~] REPLANNED → S3b — `StreamRecordingJob`/recording storage are live-capture specific; the platform path uses `PlatformIngestJob` (S3-P4). Deferred to S3b.

---

## Task 5 — Recording→asset bridge

**Effort:** M · **Complexity:** Medium · **Depends on:** H1, Task 0, Task 0c, Task 1, Task 3, Task 4
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
Apply the T0c-selected output contract: compute checksum(s), durably upload the
assembled MP4, and call the reused finalize path (T0) with
`ArtifactKind::RecordedStreamMedia` and the session's captured `RightsBasis`
(ADR-021). Emit audit events (`RecordingRecorded`, `RecordingBridgedToAsset`) via
the H1 shared durable-audit boundary. Apply the `parse_artifact_kind` repo fix from
F3 so recorded artifacts read back as `RecordedStreamMedia`. Preserve the T0c-defined
idempotency-token cardinality.

### Acceptance criteria
- A recorded session produces exactly one asset and one `recorded_stream_media`
  artifact row per session, with
  `recorded_stream_media` lineage and durable audit events.
- `find_original_by_ingest_token` returns the artifact with kind
  `RecordedStreamMedia` (not `OriginalMedia`) — regression guard for F3.
- Re-running the bridge for the same recording creates no duplicate artifacts.
- A session that reached `RejectedMissingRights` never produces an asset.

### Files affected
- `apps/worker-runner/src/handlers/recording.rs` (new) — or a service module
- `crates/db/src/artifact_repo.rs` (F3 parse fix), `crates/db/src/audit_repo.rs`
- `crates/domain/src/audit.rs` (recording audit constructor — shared with T1)

### Status: [~] REPLANNED → S3b — the recording→asset bridge is generalized by ADR-021 and implemented for downloads in S3-P4; this live-capture bridge is deferred to S3b.

---

## Task 6 — API endpoints

**Effort:** L · **Complexity:** High · **Depends on:** S0 T2, Task 1, Task 2, Task 4
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Endpoints
| Method | Path | Description |
|--------|------|-------------|
| POST | `/recordings` | Create session with source + rights; fail-closed (422 on missing rights) |
| POST | `/recordings/{id}/start` | Validate + enqueue `StreamRecordingJob` |
| POST | `/recordings/{id}/stop` | Request graceful stop |
| GET | `/recordings/{id}` | Read session status |

### Acceptance criteria
- Missing or invalid API bearer token returns `401`; insufficient scope returns `403`.
- Mutation endpoints require `recordings:write`; reads require `recordings:read`.
- Create without valid rights returns `422`.
- Create with unsupported scheme (not rtmp/srt) returns `422`.
- Start transitions `RightsValidated → Capturing` and enqueues exactly one job.
- Stop on a non-capturing session returns a clear `409`.
- `cargo check` passes.

### Files affected
- `apps/api/src/routes/recording.rs` (new)
- `apps/api/src/dto/recording.rs` (new)
- `apps/api/src/state.rs`, `apps/api/src/main.rs`
- `apps/api/Cargo.toml`

### Status: [~] REPLANNED → S3b — start/stop capture endpoints are live-recording specific; the platform path exposes create/start/get ingest endpoints in S3-P5. Deferred to S3b.

---

## Task 7 — Worker-runner integration

**Effort:** L · **Complexity:** High · **Depends on:** Task 3, Task 4, Task 5
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`
(escalate to `Opus 4.1` only if integration repeatedly stalls)

### Scope
Register the `StreamRecordingJob` handler in `apps/worker-runner`: instantiate
`RecordingSupervisor`, drive the capture, persist status transitions, on `Recorded`
invoke the bridge (T5), on `Failed` audit and surface the error.

### Acceptance criteria
- A queued `StreamRecordingJob` is consumed and a capture is supervised end to end.
- Status transitions are persisted and auditable.
- Graceful stop produces a non-corrupt file.
- `cargo check` passes.

### Files affected
- `apps/worker-runner/src/main.rs`
- `apps/worker-runner/src/handlers/recording.rs`
- `apps/worker-runner/Cargo.toml`

### Status: [~] REPLANNED → S3b — supervises FFmpeg capture, which is the deferred sub-case; the platform worker handler is S3-P4. Deferred to S3b.

---

## Task 8 — Tests (unit + integration)

**Effort:** L · **Complexity:** High · **Depends on:** Task 1–Task 7
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
- Unit: command builder argument vectors; source URL validation; state-machine
  transitions; path templater. (Already partly in T1/T3 — consolidate here.)
- Integration (package-owned recording test suite): use FFmpeg `lavfi`
  `testsrc`/`sine` as a synthetic local source to record a short clip, verify the
  assembled MP4 selected by T0c, then verify the bridge created the asset with
  fail-closed rights.
  Gated by `DATABASE_URL` + an `ffmpeg`-present check, consistent with S1.

### Acceptance criteria
- `record_with_valid_rights_creates_recorded_asset` passes against local Postgres.
- `record_without_rights_is_rejected_before_capture` passes.
- `duplicate_bridge_does_not_create_duplicate_outputs` passes.
- All assertions are meaningful (no `assert!(true)`); no mocking of the backend —
  real Postgres + real local FFmpeg per the project's testing policy.

### Files affected
- Package-owned integration test file selected during implementation (the workspace
  root is not a Cargo package)
- Unit test modules co-located in the crates above.

### Status: [~] REPLANNED → S3b — these tests exercise the FFmpeg `lavfi` capture path; the primary platform-ingest test suite lives in S3-P3/S3-P4. Deferred to S3b.

---

# Platform Ingester tasks (PRIMARY path — REPLAN 2026-05-31, ADR-025)

Build order for S3: **P1 → P5**. These reuse the DONE foundation
(T0/T0b/T0c/T1/T2) and the generalized ADR-021 bridge. The FFmpeg recorder
(T3–T8) is deferred to S3b.

## Task P1 — Connector trait boundary + domain generalization

**Effort:** L · **Complexity:** High · **Depends on:** T1, T2 (DONE)
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4` (thinking On —
new crate boundary + domain aggregate design)

### Objective
Create the `crates/connectors` boundary and the platform-ingest domain model,
reusing the S3-T1 fail-closed validation pattern.

### Scope
- New crate `crates/connectors` (workspace member): `PlatformConnector` trait,
  `Platform`, `SourceRef`, `ConnectorCredential`, `RemoteMediaMetadata`,
  `DownloadedMedia`, `ConnectorError`. Pure request-builder / IO-executor split.
  Depends only on `crates/domain` + `crates/config`; no DB.
- `crates/domain/src/platform_ingest.rs` (new): `PlatformIngestSession`,
  `PlatformIngestStatus` (`Requested`, `RightsValidated`, `Resolving`,
  `Downloading`, `Downloaded`, `Failed`, `RejectedMissingRights`),
  `StartPlatformIngestCommand::validate()` (fail-closed: rights + credential ref +
  source ref), state-machine transition methods.
- `crates/domain/src/artifact.rs`: add `ArtifactKind::DownloadedPlatformMedia`
  (+ `Display` + `parse_artifact_kind` arm).
- `crates/domain/src/audit.rs`: add platform-ingest `AuditEventKind` variants
  (`PlatformIngestSessionCreated`, `PlatformIngestRejectedMissingRights`,
  `PlatformIngestDownloadStarted`, `PlatformIngestDownloaded`,
  `PlatformIngestFailed`, `PlatformIngestBridgedToAsset`).
- `crates/domain/src/lib.rs`: re-export `platform_ingest`.

### Acceptance criteria
- `StartPlatformIngestCommand::validate()` returns a rejection error when rights or
  the owner credential reference is absent (ADR-008/ADR-025 fail-closed).
- No state path reaches `Downloading` without passing `RightsValidated`.
- `parse_artifact_kind("downloaded_platform_media") == DownloadedPlatformMedia`.
- `cargo check --workspace` clean; `cargo test -p dubbridge-domain` passes;
  existing recording/S1 tests still pass.

### Files affected
- `crates/connectors/{Cargo.toml,src/lib.rs}` (new); root `Cargo.toml` member
- `crates/domain/src/{platform_ingest.rs,artifact.rs,audit.rs,lib.rs}`

### Status: [ ]

---

## Task P2 — YouTube retrieval-mechanism spike (gate)

**Effort:** M · **Complexity:** High (external API dependency) · **Depends on:** P1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4` with thinking On

### Objective
Before implementing the YouTube connector, validate the legitimate owner-download
mechanism end to end — the T0c-style gate for the platform path. YouTube exposes no
single Data-API endpoint returning original bytes for arbitrary videos, so the
retrieval mechanism must be proven, not assumed.

### Scope
- Throwaway internal spike under `spikes/youtube-connector-sandbox/` (not published,
  no third-party source vendored).
- Determine and document, for v1: required OAuth scope(s); how ownership is verified
  (`resolve`); the concrete owner-authorized retrieval mechanism for the media bytes;
  the downloaded container/quality contract; credential-redaction approach.
- Update ADR-025 with the fixed mechanism if the spike narrows it.

### Acceptance criteria
- ADR-025 (or an appendix) records one coherent, validated v1 retrieval mechanism.
- The credential scope and redaction approach are documented before P3 starts.
- The spike contains no vendored third-party source and is not published.

### Files affected
- `spikes/youtube-connector-sandbox/` (temporary, internal)
- `docs/adr/ADR-025-platform-connector-ingest-and-owner-authorized-credentials.md`
- `docs/plan/stream-recording-ingest.md` (decision note)

### Status: [ ]

---

## Task P3 — YouTube connector v1

**Effort:** L · **Complexity:** High · **Depends on:** P1, P2
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Implement the YouTube `PlatformConnector` using the P2-validated mechanism:
`resolve()` (ownership + metadata) and `download()` (owner-authorized bytes to local
staging), with credentials resolved by reference and redacted in logs.

### Scope
- `crates/connectors/src/youtube.rs`: pure request builder (unit-testable, no
  network) + executor performing authenticated IO.
- Connector config in `crates/config` (timeouts, staging path, secrets-store ref).

### Acceptance criteria
- Request-builder unit tests assert exact request shape; no secret appears in any
  log line (redaction test).
- `resolve()` rejects items not owned/accessible by the credential.
- Integration test runs against a recorded/sandbox fixture (no live secret in CI).
- `cargo check` + `cargo test -p dubbridge-connectors` pass.

### Files affected
- `crates/connectors/src/youtube.rs`, `crates/connectors/Cargo.toml`
- `crates/config/src/lib.rs`

### Status: [ ]

---

## Task P4 — `PlatformIngestJob` + download→bridge wiring

**Effort:** L · **Complexity:** High · **Depends on:** T0 (DONE), P1, P3
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Wire the worker path: consume a `PlatformIngestJob`, drive
`resolve → download → checksum → upload → bridge`, and persist + audit each
transition. The bridge calls `finalize_ingestion_core` with
`ArtifactKind::DownloadedPlatformMedia` and the session's `RightsBasis` (ADR-021
generalized). One asset, one `ingest_token` per session.

### Scope
- `crates/jobs/src/lib.rs`: `PlatformIngestJob` envelope (serde round-trip).
- `crates/storage/src/lib.rs`: `ingests/{session_id}/` prefix + downloaded-object
  put helper.
- `infra/migrations/<next>_create_platform_ingest_sessions.sql` (status CHECK
  matching `PlatformIngestStatus`; credential stored by reference only; asset_id
  nullable FK set on bridge).
- `crates/db/src/platform_ingest_repo.rs` (new) — persist + transition sessions.
- `apps/worker-runner/src/handlers/platform_ingest.rs` (new) — orchestrate the flow.

### Acceptance criteria
- A successful ingest produces exactly one asset + one `downloaded_platform_media`
  artifact row; `find_original_by_ingest_token` reads it back with the correct kind.
- Re-running the job for the same session creates no duplicate artifacts.
- A session rejected for missing rights/credential never downloads or produces an
  asset.
- Migration applies cleanly on fresh Postgres; `cargo check` passes.

### Files affected
- `crates/jobs/src/lib.rs`, `crates/storage/src/lib.rs`
- `crates/db/src/platform_ingest_repo.rs` (new)
- `infra/migrations/<next>_create_platform_ingest_sessions.sql` (new)
- `apps/worker-runner/src/{main.rs,handlers/platform_ingest.rs}`

### Status: [ ]

---

## Task P5 — API endpoints (platform ingest)

**Effort:** M · **Complexity:** Medium · **Depends on:** S0 T2, P1, P4
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Endpoints
| Method | Path | Description |
|--------|------|-------------|
| POST | `/ingests/platform` | Create session with source ref + owner credential ref + rights; fail-closed (422 on missing rights/credential) |
| POST | `/ingests/platform/{id}/start` | Validate + enqueue `PlatformIngestJob` |
| GET | `/ingests/platform/{id}` | Read session status |

### Acceptance criteria
- Missing/invalid API bearer → `401`; insufficient scope → `403`.
- Mutations require `ingests:write`; reads require `ingests:read`.
- Create without valid rights or without a credential reference → `422`.
- Create with unsupported platform → `422`.
- Start transitions `RightsValidated → Resolving/Downloading` and enqueues exactly
  one job.
- API bearer identity (S0/ADR-023) stays separate from owner platform credentials.
- `cargo check` passes.

### Files affected
- `apps/api/src/routes/platform_ingest.rs` (new)
- `apps/api/src/dto/platform_ingest.rs` (new)
- `apps/api/src/{state.rs,main.rs}`, `apps/api/Cargo.toml`

### Status: [ ]

---

## Task 9 — Housekeeping: align docker-compose Rust pin (F7)

**Effort:** S · **Complexity:** Low · **Depends on:** nothing
**Priority:** Low — independent of the recording work; can be done at any time.
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
`infra/docker-compose.yml` pins `image: rust:1.88` for the `api` / `worker-runner`
services, while `rust-toolchain.toml` and CI use `stable`. Align the pin so local
compose builds match the toolchain policy and cannot silently lag.

### Decision
Either pin a documented minimum (the MSRV required for edition 2024 + resolver 3) or
track `stable` to match CI. Record the chosen Rust version policy in the README
`## Validation` section.

### Acceptance criteria
- The compose Rust image and the toolchain policy agree, or the divergence is
  intentional and documented.
- `docker compose -f infra/docker-compose.yml config` is valid.

### Files affected
- `infra/docker-compose.yml`
- `README.md` (Rust version policy note, if changed)

### Status: [ ]

---

## Agent handoff prompt (for delegation) — PRIMARY path (S3 platform ingest)

```
You are implementing the PRIMARY S3 Platform Ingest path of DubBridge
(owner-authorized platform download — YouTube first). RTMP/SRT live recording is
the DEFERRED S3b sub-case; do not build it unless explicitly asked.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/stream-recording-ingest.md
Tasks: docs/tasks/stream-recording-ingest.md
ADRs: ADR-025 (primary: connector ingest + owner credentials),
      ADR-021 (generalized intake->asset bridge), foundations ADR-006/008/018,
      ADR-023 (API identity). ADR-019/020/022 govern the deferred S3b recorder.

Foundation T0/T0b/T0c/H1/T1/T2 are complete and reused. Work one approved task at a
time in order: P1 -> P5. After each task:
1. Run `cargo check` (and `cargo test` for the touched crate) at the workspace root.
2. Mark the task [x] in the tasks document and record the files/lines affected.
3. Report a summary and WAIT for approval before starting the next task.

Hard invariants (do not violate):
- Fail-closed rights (ADR-008): a session lacking a valid RightsBasis OR a valid
  owner credential is rejected BEFORE any bytes are downloaded. Downloaded media
  reuses the S1 finalize gate (finalize_ingestion_core); no second ingestion path.
- Connector boundary (ADR-025): the URL/request builder is a pure function
  (media::ffprobe_command discipline); only the executor does network IO.
  crates/connectors has no DB dependency.
- Lineage (ADR-006): compute SHA-256 and write the artifact row with
  ArtifactKind::DownloadedPlatformMedia before the asset is considered ingested.
- Credentials (ADR-025): owner platform credentials are stored by reference, never
  in plaintext, and redacted in all logs/traces.
- API identity (ADR-023): platform-ingest HTTP endpoints reuse the S0 verified
  principal; keep API bearer tokens separate from owner platform credentials.
- YouTube retrieval mechanism is fixed by the P2 spike before P3 implementation.
- Do NOT commit if any test is broken. Run all tests before commit/push.
- All user-facing communication is in Spanish; code/docs/commits in English.
```

## Agent handoff prompt (for delegation) — DEFERRED path (S3b live recorder)

```
S3b (RTMP/SRT live recording, ex-T3..T8) is deferred. The prior recorder handoff
applied here: FFmpeg supervised subprocess (ADR-019), HLS fMP4 staging + assembled
MP4 (ADR-020/T0c), bridge via ADR-021, RTMP/SRT auth (ADR-022). Build only when a
real live-broadcast client need exists and the task is re-approved.
```
