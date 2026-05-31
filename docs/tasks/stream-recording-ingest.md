# Tasks: Stream Recording Ingest

Governing plan: `docs/plan/stream-recording-ingest.md`
Governing ADRs: ADR-006, ADR-008, ADR-018, ADR-019, ADR-020, ADR-021, ADR-022, ADR-023
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

---

## Task 0b — Reconcile `crates/audit` (F1/F8)

**Effort:** S–M · **Complexity:** Medium · **Depends on:** nothing (S1 audit code present)
**Priority:** Medium — do **before Task 1**. Recommended-early, not a hard blocker.
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Remove the duplicate/conflicting `AuditEvent` so the workspace has a single audit
type and path. `crates/domain::audit::AuditEvent` (persisted by `db::audit_repo`) is
canonical; `crates/audit` currently holds an unrelated placeholder (F1), and the S1
plan's promised `AuditLogger` was never built (F8).

### Decision (pick one — confirm before executing)
- **Option A (preferred):** implement the intended `AuditLogger` in `crates/audit`
  that wraps `db::audit_repo` writes + tracing spans, and delete the placeholder
  `AuditEvent`. Add deps `dubbridge-domain`, `dubbridge-db`. Fulfills S1 intent.
- **Option B:** remove `crates/audit` from the workspace if no `AuditLogger`
  consumer is planned. **Deleting a crate requires explicit confirmation** (CLAUDE.md).

### Acceptance criteria
- Exactly one `AuditEvent` type exists in the workspace.
- Either `crates/audit` wraps `domain::audit` + `db::audit_repo`, or it is removed
  from `Cargo.toml` members with no dangling references.
- `cargo check --workspace` clean; existing S1 tests still pass.

### Files affected
- `crates/audit/src/lib.rs` (rewrite or remove)
- `crates/audit/Cargo.toml` (add domain+db deps, or remove)
- root `Cargo.toml` (members list — only if removing)

### Status: [ ]

---

## Task 1 — Domain types for recording sessions

**Effort:** M · **Complexity:** Medium · **Depends on:** nothing
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

### Status: [ ]

---

## Task 2 — Migrations: recording_sessions + audit generalization

**Effort:** S · **Complexity:** Low · **Depends on:** Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
- `infra/migrations/0005_create_recording_sessions.sql`: id (UUID PK),
  source_protocol, source_url, owner/uploader, rights fields (or FK to a rights row
  created at validation), status with CHECK constraint matching `RecordingStatus`,
  asset_id (nullable FK, set on bridge), created_at/updated_at TIMESTAMPTZ
  DEFAULT now().
- `infra/migrations/0006_alter_audit_events_for_recording.sql` (F2): relax
  `ingest_token` to nullable; add `recording_session_id UUID NULL REFERENCES
  recording_sessions(id)`. Ordering matters — `recording_sessions` must exist first.

### Acceptance criteria
- Both migrations apply cleanly on a fresh Postgres (`sqlx migrate run`).
- `recording_sessions.status` CHECK restricts to the seven defined values.
- Credentials are **not** stored in plaintext (reference/secret only).
- After `0006`, an `audit_events` row can be inserted with `ingest_token` NULL and a
  non-null `recording_session_id`; existing ingestion inserts still succeed.

### Files affected
- `infra/migrations/0005_create_recording_sessions.sql` (new)
- `infra/migrations/0006_alter_audit_events_for_recording.sql` (new)

### Status: [ ]

---

## Task 3 — `crates/recorder`: FFmpeg engine boundary

**Effort:** L · **Complexity:** High · **Depends on:** Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`
(escalate to `Opus 4.1` only if supervision logic repeatedly stalls)

### Scope
New crate mirroring `crates/media`'s command-builder pattern (ADR-019, ADR-020):
- `ffmpeg.rs`: `ffmpeg_record_command(source, output_template, opts) -> Vec<String>`
  — pure, deterministic, no process spawn. Encodes segment muxer, part/segment
  durations, container = fMP4, and per-protocol input options (RTMP key, SRT
  passphrase) with redaction-safe handling.
- `session.rs`: `RecordingSupervisor` over `tokio::process` — spawn, read
  stdout/stderr lines, graceful stop via `q\n` to stdin with bounded timeout,
  bounded restart-with-backoff policy, surfaces lifecycle transitions.
- `segments.rs`: `recordPath`-style templater (`%session/%Y-%m-%d/%H-%M-%S`) →
  `storage_key`; retention (`delete_after`) helper.

### Acceptance criteria
- `ffmpeg_record_command` is covered by unit tests asserting exact argument
  vectors for an RTMP and an SRT source (credentials redacted in any log).
- Path templater unit-tested for stable, collision-free keys.
- Supervisor graceful-stop path is unit/integration-tested against a short local
  `lavfi`/`testsrc` capture (no network).
- `cargo check` + `cargo test -p dubbridge-recorder` pass.

### Files affected
- `crates/recorder/Cargo.toml` (new) + workspace member in root `Cargo.toml`
- `crates/recorder/src/{lib,ffmpeg,session,segments}.rs` (new)

### Status: [ ]

---

## Task 4 — Jobs + storage wiring

**Effort:** M · **Complexity:** Medium · **Depends on:** Task 1, Task 3
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
- `crates/jobs`: `StreamRecordingJob` envelope (session id, source, output config).
- `crates/storage`: `recording_prefix(session_id)` + segment put helper to MinIO.

### Acceptance criteria
- `StreamRecordingJob` serializes/deserializes (serde round-trip test).
- `recording_prefix` returns `recordings/{session_id}/`.
- `cargo check` passes.

### Files affected
- `crates/jobs/src/lib.rs`
- `crates/storage/src/lib.rs`

### Status: [ ]

---

## Task 5 — Recording→asset bridge

**Effort:** M · **Complexity:** Medium · **Depends on:** Task 0, Task 1, Task 3, Task 4
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
On `Recorded`, compute SHA-256 over the finalized file, upload to object storage,
and call the reused finalize path (T0) with `ArtifactKind::RecordedStreamMedia` and
the session's captured `RightsBasis` (ADR-021). Emit audit events
(`RecordingRecorded`, `RecordingBridgedToAsset`) via `crates/db::audit_repo` +
`crates/domain::audit` (F1 — not `crates/audit`). Apply the `parse_artifact_kind`
repo fix from F3 so the recorded artifact reads back as `RecordedStreamMedia`.
Idempotent via `ingest_token` + the `artifact_records.ingest_token` UNIQUE constraint.

### Acceptance criteria
- A recorded session produces exactly one asset + rights_record + artifact_record
  (kind `recorded_stream_media`) + audit events.
- `find_original_by_ingest_token` returns the artifact with kind
  `RecordedStreamMedia` (not `OriginalMedia`) — regression guard for F3.
- Re-running the bridge for the same recording creates no duplicate artifact.
- A session that reached `RejectedMissingRights` never produces an asset.

### Files affected
- `apps/worker-runner/src/handlers/recording.rs` (new) — or a service module
- `crates/db/src/artifact_repo.rs` (F3 parse fix), `crates/db/src/audit_repo.rs`
- `crates/domain/src/audit.rs` (recording audit constructor — shared with T1)

### Status: [ ]

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

### Status: [ ]

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

### Status: [ ]

---

## Task 8 — Tests (unit + integration)

**Effort:** L · **Complexity:** High · **Depends on:** Task 1–Task 7
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Scope
- Unit: command builder argument vectors; source URL validation; state-machine
  transitions; path templater. (Already partly in T1/T3 — consolidate here.)
- Integration (`tests/integration/recording_test.rs`): use FFmpeg `lavfi`
  `testsrc`/`sine` as a synthetic local source to record a short clip, verify a
  finalized file, then verify the bridge created the asset with fail-closed rights.
  Gated by `DATABASE_URL` + an `ffmpeg`-present check, consistent with S1.

### Acceptance criteria
- `record_with_valid_rights_creates_recorded_asset` passes against local Postgres.
- `record_without_rights_is_rejected_before_capture` passes.
- `duplicate_bridge_does_not_create_duplicate_artifact` passes.
- All assertions are meaningful (no `assert!(true)`); no mocking of the backend —
  real Postgres + real local FFmpeg per the project's testing policy.

### Files affected
- `tests/integration/recording_test.rs` (new)
- Unit test modules co-located in the crates above.

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

## Agent handoff prompt (for delegation)

```
You are implementing the Stream Recording Ingest slice of DubBridge.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/stream-recording-ingest.md
Tasks: docs/tasks/stream-recording-ingest.md
ADRs: docs/adr/ADR-019..ADR-022 (engine, lifecycle, bridge, protocols),
      foundations ADR-006/008/018.

Work one task at a time in order: T0, then T0b (reconcile crates/audit), then
T1 → T8; T9 (docker-compose housekeeping) is low-priority and independent. T0 is a
hard prerequisite: the S1 finalize path must be reusable before the bridge (T5) can
exist. After each task:
1. Run `cargo check` (and `cargo test` for the touched crate) at the workspace root.
2. Mark the task [x] in the tasks document and record the files/lines affected.
3. Report a summary and WAIT for approval before starting the next task.

Hard invariants (do not violate):
- Fail-closed rights (ADR-008): a session with invalid rights is rejected BEFORE
  any ffmpeg process is spawned. Recorded media reuses the S1 finalize gate; no
  second ingestion path.
- Rust owns orchestration (ADR-019): ffmpeg is a supervised subprocess; the command
  builder is a pure function like media::ffprobe_command. Do NOT link GPL libs.
- Lineage (ADR-006): compute SHA-256 and write an artifact row with
  ArtifactKind::RecordedStreamMedia before the asset is considered recorded.
- Credentials (ADR-022): RTMP keys / SRT passphrases are never logged; redact them.
- API identity (ADR-023): recording HTTP endpoints reuse the S0 verified principal;
  keep API bearer tokens separate from RTMP/SRT source credentials.
- Clean-room / IP: the recorder is ORIGINAL Rust. Do NOT copy third-party source.
  No public, customer-facing, or marketing artifact may state or imply the design
  copies or derives from a third-party project. Competitive/reference material
  (proposal, internal reviews) stays internal and is not for publication.
- Do NOT commit if any test is broken. Run all tests before commit/push.
- All user-facing communication is in Spanish; code/docs/commits in English.
```
