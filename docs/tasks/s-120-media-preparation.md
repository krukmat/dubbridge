---
type: TaskList
title: "S-120 Media Preparation"
status: closed
slice: S-120
plan: docs/plan/s-120-media-preparation.md
---
# S-120 Media Preparation

## S-120-T1: Initial Setup and Validation
**Effort:** 3h
**Depends on:** None
**Status:** Done (2026-06-18)
**Happy paths considered:**
- HP-1: Successful preparation produces metadata and HLS outputs.
- HP-2: Validates that all required fields are populated in the database upon completion.
**Edge cases considered:**
- EC-1: Downstream processing blocked when asset is not ready.
- EC-2: Malformed probe/transcode results do not trigger a "Ready" state.
**Evidence:**
- BDD Feature File: `docs/bdd/s-120-media-preparation.feature`
- Mapping Table: See `docs/bdd/README.md` under S-120 section.
**Note:** Some roadmap drift blockers in qa-docs may still exist but do not impact the core logic of T1.

## S-120-T2: Schema, domain, and repository for preparation lineage + status
**Effort:** L (RRI 47 — Med-high)
**Depends on:** S-120-T1
**Status:** Done (2026-06-19)
**Happy paths considered:**
- HP-1: Insert a derived artifact and list it back correctly.
- HP-2: Multiple derived artifact kinds (ProbeMetadata, HlsManifest, HlsSegment) are all returned for an asset.
- HP-3: `list_derived_artifacts` returns empty for an asset with no derived artifacts.
- HP-4: Upsert then get preparation status round-trips correctly.
- HP-5: Preparation status transitions Pending → InProgress → Ready all succeed.
**Edge cases considered:**
- EC-1: Failed status persists error_detail.
- EC-2: Source artifact (parent_artifact_id = NULL) is not returned by list_derived_artifacts.
- EC-3: `get_preparation_status` returns None for an asset with no status row.
- EC-4: Unknown artifact kinds and status values fail closed (UnknownStoredValue error).
**Evidence:**
- Migration: `infra/migrations/0019_create_preparation.sql`
- Domain types: `crates/domain/src/artifact.rs` — `ArtifactKind` (ProbeMetadata, HlsManifest, HlsSegment), `PreparationStatus`, `DerivedArtifact`, `PreparationStatusRecord`
- Repository: `crates/db/src/preparation_repo.rs` — `insert_derived_artifact`, `list_derived_artifacts`, `get_preparation_status`, `upsert_preparation_status`
- Integration tests: `apps/api/tests/preparation_repo_test.rs` — 7 tests covering all HP/EC above
- Unit tests: `crates/domain/src/artifact.rs` (13 new tests), `crates/db/src/preparation_repo.rs` (4 new tests), `crates/db/src/artifact_repo.rs` (1 new test)
- `cargo test -p dubbridge-domain -p dubbridge-db`: 153 passed
- `cargo clippy -p dubbridge-domain -p dubbridge-db -- -D warnings`: clean
- `make qa-docs`: 4/4 gates green

## S-120-T3: `ffprobe` metadata extraction and persistence
**Effort:** L (RRI 43 — Med-high, provisional planning run)
**Depends on:** S-120-T2
**Status:** Done (2026-06-19)
**Happy paths considered:**
- HP-1: A valid source artifact produces a canonical `ffprobe` command and parsed metadata payload that can be persisted as a `ProbeMetadata` derived artifact.
- HP-2: Persisted probe metadata is linked to the source artifact via `parent_artifact_id` and becomes queryable through the preparation repository.
- HP-3: Preparation status can advance into a non-terminal pre-HLS state (`Pending`/`InProgress`) while the asset remains not yet prepared for downstream consumers until `T4`/`T5` complete the canonical HLS package.
**Edge cases considered:**
- EC-1: `ffprobe` process failure marks preparation `Failed` with observable error detail and does not persist a misleading `ProbeMetadata` artifact.
- EC-2: Malformed or incomplete `ffprobe` output is rejected fail-closed and does not mark the asset prepared.
- EC-3: Missing source-artifact context for an asset aborts persistence cleanly instead of inventing lineage.
**Inputs:**
- Source artifact row created by ingestion/finalize.
- Preparation lineage/status schema from `S-120-T2`.
- Existing media command-builder seam in `crates/media`.
**Outputs:**
- `ffprobe` command builder/parser contract in `crates/media`.
- Persisted probe metadata artifact row (`ArtifactKind::ProbeMetadata`) plus stored JSON payload or equivalent canonical representation.
- Preparation-status transitions/evidence for probe success and failure, without claiming full prepared-media readiness.
**Acceptance criteria:**
- `crates/media` exposes a deterministic `ffprobe` invocation contract suitable for unit testing.
- Probe output parsing validates required structure and fails closed on malformed data.
- Successful probe persistence creates exactly one `ProbeMetadata` derived artifact linked to the source artifact.
- Successful probe persistence does not by itself mark the asset `Ready`; canonical prepared readiness remains gated on HLS output in `T4` and orchestration/readiness wiring in `T5`, per plan D3.
- Failure paths persist `PreparationStatus::Failed` with actionable `error_detail`.
- Tests cover at least one success path and the malformed/process-failure edge cases.
**Files expected to change:**
- `crates/media/src/lib.rs`
- `crates/db/src/preparation_repo.rs`
- `apps/api/tests/preparation_repo_test.rs`
**Evidence / governing context:**
- Plan: `docs/plan/s-120-media-preparation.md` (`T3` in task decomposition strategy)
- BDD: `docs/bdd/s-120-media-preparation.feature` (`S120_HP1`, `S120_EC3`)
- ADR: `docs/adr/ADR-032-hls-playback-delivery-boundary.md` (prepared-media dependency contract)
**Agent handoff prompt:** Implement the smallest fail-closed probe pipeline slice: add a unit-testable `ffprobe` builder/parser seam, persist probe metadata as a derived artifact linked to the source artifact, and record failure status while keeping canonical prepared readiness reserved for the later HLS tasks (`T4`/`T5`).
**Evidence:**
- Media seam: `crates/media/src/lib.rs` now emits canonical JSON-oriented `ffprobe` argv, validates required `format`/`streams` structure, and normalizes valid output through `canonical_ffprobe_json`.
- Repository seam: `crates/db/src/preparation_repo.rs` now resolves the source artifact by `asset_id` and persists `ProbeMetadata` with correct `parent_artifact_id` lineage via `insert_probe_metadata_artifact`.
- Integration evidence: `apps/api/tests/preparation_repo_test.rs` adds probe-persistence coverage for success and missing-source fail-closed behavior while preserving pre-HLS `InProgress`.
- Wiring: `crates/media/Cargo.toml` adds `serde`/`serde_json` for parsing; `apps/api/Cargo.toml` adds `dubbridge-media` to test the canonical probe contract in integration coverage.
- Verification:
  - `cargo test -p dubbridge-media -p dubbridge-db`
  - `cargo test -p dubbridge-api --test preparation_repo_test`
  - `cargo clippy -p dubbridge-media -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

### Reflection log

Required passes: 3 (`43` -> `Med-high`)

#### Pass 1

- **Draft verdict:** The minimal implementation was viable if the media crate validated JSON shape instead of introducing worker orchestration early.
- **Critique findings:**
  - The original `ffprobe` seam only asserted the binary name and verbosity flag; it did not enforce JSON output or required fields.
  - Persisting `ProbeMetadata` by hand in tests would not prove the repository can discover the source artifact and link lineage correctly.
- **Revisions applied:**
  - Expanded `ffprobe_command` to request `-show_format`, `-show_streams`, and JSON output.
  - Added `parse_ffprobe_output` and `canonical_ffprobe_json` with fail-closed validation.
  - Added `insert_probe_metadata_artifact` to `preparation_repo`.

#### Pass 2

- **Draft verdict:** Success path worked, but fail-closed boundaries still needed sharper coverage.
- **Critique findings:**
  - Missing source-artifact handling needed an explicit repository test, not just inferred behavior.
  - Probe persistence needed to prove it preserves `InProgress` rather than drifting toward `Ready`.
- **Revisions applied:**
  - Added `insert_probe_metadata_artifact_requires_source_artifact`.
  - Added `insert_probe_metadata_artifact_links_to_source` with explicit `InProgress` status assertions.

#### Pass 3

- **Draft verdict:** The code path and approved behaviors were covered; the remaining risk was regression through lint/doc drift.
- **Critique findings:**
  - Documentation needed to record the actual completion evidence and HP/EC mapping.
  - Toolchain verification needed to include linting across the touched crates.
- **Revisions applied:**
  - Updated this ledger entry to `Done` with evidence, certification, and verification commands.
  - Ran focused `cargo test` and `cargo clippy` on the touched surfaces.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid source artifact yields canonical `ffprobe` contract and parseable metadata payload | `crates/media/src/lib.rs::ffprobe_command_requests_json_format_and_streams`, `crates/media/src/lib.rs::canonical_ffprobe_json_round_trips_valid_payload` | passed |
| HP-2 | Happy path | persisted probe metadata links to source artifact via `parent_artifact_id` and is queryable | `apps/api/tests/preparation_repo_test.rs::insert_probe_metadata_artifact_links_to_source` | passed |
| HP-3 | Happy path | probe persistence preserves a pre-HLS non-terminal readiness state | `apps/api/tests/preparation_repo_test.rs::insert_probe_metadata_artifact_links_to_source` | passed |
| EC-1 | Edge case | explicit preparation failure persists `Failed` with observable `error_detail` | `apps/api/tests/preparation_repo_test.rs::failed_status_persists_error_detail` | passed |
| EC-2 | Edge case | malformed probe output fails closed and does not become prepared metadata | `crates/media/src/lib.rs::parse_ffprobe_output_rejects_missing_format`, `crates/media/src/lib.rs::parse_ffprobe_output_rejects_streams_without_codec_type` | passed |
| EC-3 | Edge case | missing source artifact aborts probe persistence cleanly without inventing lineage | `apps/api/tests/preparation_repo_test.rs::insert_probe_metadata_artifact_requires_source_artifact` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-19`
- Statement: I verified every happy path and edge case defined for this task has executable evidence covering the expected fail-closed probe behavior and pre-HLS readiness posture.
- Commands run: `cargo fmt`; `cargo test -p dubbridge-media -p dubbridge-db`; `cargo test -p dubbridge-api --test preparation_repo_test`; `cargo clippy -p dubbridge-media -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

## S-120-T4: HLS transcode output and storage persistence
**Effort:** L (RRI 47 — Med-high, provisional planning run)
**Depends on:** S-120-T3
**Status:** Done (2026-06-19)
**Happy paths considered:**
- HP-1: A valid source artifact produces a canonical `ffmpeg` HLS command contract and deterministic output file layout for v1.
- HP-2: The generated HLS manifest is persisted as a `HlsManifest` derived artifact and every media segment is persisted as a `HlsSegment` derived artifact linked to the source artifact.
- HP-3: Storage-owned canonical keys are used for manifest and segment persistence so downstream consumers do not infer layout ad hoc.
**Edge cases considered:**
- EC-1: `ffmpeg` process failure records `PreparationStatus::Failed` and does not persist a partial HLS package as if it were complete.
- EC-2: Missing manifest, empty segment set, or malformed HLS output is rejected fail-closed and does not advance readiness.
- EC-3: Hand-rolled or inconsistent storage-key layout is blocked in favor of `crates/storage` helpers.
**Inputs:**
- Source artifact row created by ingestion/finalize.
- Persisted probe metadata and pre-HLS status from `S-120-T3`.
- Existing storage adapter boundary and preparation repository seam.
**Outputs:**
- `ffmpeg` HLS command builder/parser contract in `crates/media`.
- Storage-owned key helpers for HLS manifests and segments.
- Persisted `HlsManifest` and `HlsSegment` derived-artifact rows with lineage and checksums.
- Failure evidence for transcode/output-validation errors without claiming final prepared readiness.
**Acceptance criteria:**
- `crates/media` exposes a deterministic HLS-oriented `ffmpeg` invocation contract suitable for unit testing.
- `crates/storage` owns canonical HLS key generation for manifest and segment outputs.
- Successful persistence creates one `HlsManifest` plus one-or-more `HlsSegment` artifacts linked to the source artifact.
- Output validation fails closed when manifest/segments are missing or malformed.
- `T4` does not by itself finalize slice readiness; final prepared-state wiring remains reserved for `T5`.
- Tests cover at least one success path and one malformed/partial-output failure path.
**Files expected to change:**
- `crates/media/src/lib.rs`
- `crates/storage/src/lib.rs`
- `crates/db/src/preparation_repo.rs`
- `apps/api/tests/preparation_repo_test.rs`
**Evidence / governing context:**
- Plan: `docs/plan/s-120-media-preparation.md` (`T4` in task decomposition strategy; D3, D5, D6)
- BDD: `docs/bdd/s-120-media-preparation.feature` (`S120_HP1`, `S120_EC3`)
- ADR: `docs/adr/ADR-032-hls-playback-delivery-boundary.md` (prepared HLS feeds the playback boundary but is not the client-facing contract)
**Agent handoff prompt:** Implement the smallest fail-closed HLS persistence slice: add a unit-testable `ffmpeg` HLS command seam, introduce storage-owned HLS key helpers, persist manifest and segment artifacts with source lineage, and reject partial/malformed HLS output without marking the asset `Ready`.
**Evidence:**
- Media seam: `crates/media/src/lib.rs` now exposes `ffmpeg_hls_command` and `validate_hls_outputs`, giving the slice a deterministic HLS transcode contract plus fail-closed manifest/segment validation.
- Storage seam: `crates/storage/src/lib.rs` now owns `prepared_prefix`, `probe_metadata_key`, `hls_manifest_key`, and `hls_segment_key`, so HLS layout is canonical and not hand-rolled by callers.
- Repository seam: `crates/db/src/preparation_repo.rs` now persists a complete HLS package via `insert_hls_artifacts`, creating one `HlsManifest` and one-or-more `HlsSegment` rows linked to the source artifact.
- Integration evidence: `apps/api/tests/preparation_repo_test.rs` now writes a real manifest and segments through `LocalFsAdapter`, persists the derived-artifact rows, and verifies malformed HLS output leaves the asset failed and without derived HLS artifacts.
- Verification:
  - `cargo test -p dubbridge-media -p dubbridge-storage -p dubbridge-db`
  - `cargo test -p dubbridge-api --test preparation_repo_test`
  - `cargo clippy -p dubbridge-media -p dubbridge-storage -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

### Reflection log

Required passes: 3 (`47` -> `Med-high`)

#### Pass 1

- **Draft verdict:** The right boundary for `T4` was still a pure/preparatory seam, not full worker orchestration.
- **Critique findings:**
  - HLS generation needed an explicit `ffmpeg` contract analogous to the new `ffprobe` seam from `T3`.
  - Storage-key ownership had to move into `crates/storage` before any manifest or segment persistence could be trusted.
- **Revisions applied:**
  - Added `ffmpeg_hls_command` and `validate_hls_outputs` to `crates/media`.
  - Added canonical HLS key helpers to `crates/storage`.

#### Pass 2

- **Draft verdict:** The pure seams were in place, but the persistence story still needed end-to-end evidence across storage and DB lineage.
- **Critique findings:**
  - Persisting only DB rows would not prove real storage persistence behavior.
  - The repository needed a package-level helper so callers do not hand-roll manifest/segment artifact insertion one row at a time.
- **Revisions applied:**
  - Added `insert_hls_artifacts` to `preparation_repo`.
  - Added an integration test that writes manifest and segments through `LocalFsAdapter` before persisting derived-artifact rows.

#### Pass 3

- **Draft verdict:** The success path was complete; the remaining risk was partial HLS output being treated as acceptable.
- **Critique findings:**
  - Fail-closed behavior needed explicit evidence for malformed/segment-less manifests.
  - Documentation needed the usual certification and verification closeout.
- **Revisions applied:**
  - Added malformed-HLS coverage that leaves the asset failed and without derived HLS artifacts.
  - Updated this ledger entry with evidence, reflection, and certification.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid source artifact yields canonical `ffmpeg` HLS contract and deterministic output layout | `crates/media/src/lib.rs::ffmpeg_hls_command_requests_hls_outputs`, `crates/media/src/lib.rs::validate_hls_outputs_accepts_matching_manifest_and_segments` | passed |
| HP-2 | Happy path | manifest and segments persist as derived artifacts linked to the source artifact | `apps/api/tests/preparation_repo_test.rs::insert_hls_artifacts_persists_manifest_and_segments` | passed |
| HP-3 | Happy path | HLS persistence uses storage-owned canonical keys | `crates/storage/src/lib.rs::hls_manifest_key_format`, `crates/storage/src/lib.rs::hls_segment_key_uses_canonical_prefix_and_filename`, `apps/api/tests/preparation_repo_test.rs::insert_hls_artifacts_persists_manifest_and_segments` | passed |
| EC-1 | Edge case | transcode/output failure leaves the asset failed instead of persisting a complete-looking HLS package | `apps/api/tests/preparation_repo_test.rs::malformed_hls_output_does_not_persist_artifacts` | passed |
| EC-2 | Edge case | missing/partial HLS output is rejected fail-closed | `crates/media/src/lib.rs::validate_hls_outputs_rejects_empty_segments`, `crates/media/src/lib.rs::validate_hls_outputs_rejects_mismatched_segments`, `apps/api/tests/preparation_repo_test.rs::malformed_hls_output_does_not_persist_artifacts` | passed |
| EC-3 | Edge case | hand-rolled or unsafe HLS filenames are blocked in favor of canonical sanitized key helpers | `crates/storage/src/lib.rs::hls_segment_key_sanitizes_slashes` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-19`
- Statement: I verified the HLS command/storage/persistence seams and their fail-closed cases have executable evidence, and `T4` still stops short of final `Ready` ownership, which remains reserved for `T5`.
- Commands run: `cargo fmt`; `cargo test -p dubbridge-media -p dubbridge-storage -p dubbridge-db`; `cargo test -p dubbridge-api --test preparation_repo_test`; `cargo clippy -p dubbridge-media -p dubbridge-storage -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

## S-120-T5: Async orchestration, readiness gating, observability, and docs sync
**Effort:** L (RRI 56 — Complex)
**Depends on:** S-120-T4
**Status:** Decomposed into `S-120-T5a` / `S-120-T5b` / `S-120-T5c` (2026-06-19)
**Note:** Under the repository workflow, `56+` development tasks must be decomposed before implementation. Do not implement this parent task directly.

## S-120-T5a: Preparation job contract + finalize enqueue
**Effort:** L (RRI 51 — Med-high)
**Depends on:** S-120-T4
**Status:** Done (2026-06-19)
**Happy paths considered:**
- HP-1: Successful ingestion finalization enqueues one preparation job for the finalized asset.
- HP-2: The asset receives an initial preparation status row (`Pending`) when the job is scheduled.
- HP-3: Repeated finalize paths do not enqueue duplicate preparation work for the same finalized ingest token.
**Edge cases considered:**
- EC-1: Queue-enqueue failure leaves the asset not ready and observable instead of silently pretending preparation will happen.
- EC-2: Missing finalized source-artifact context blocks enqueue fail-closed.
**Inputs:**
- Finalize path in `apps/api/src/routes/ingestion.rs`
- Preparation schema/repo from `T2`–`T4`
- Job envelope seam in `crates/jobs`
**Outputs:**
- Typed preparation job envelope/queue contract
- Finalize hook that schedules preparation work after successful finalize
- Initial `Pending` preparation status persistence
**Acceptance criteria:**
- `crates/jobs` defines a preparation job payload with the minimum identifiers needed by the worker.
- Successful finalize enqueues exactly one preparation job and records `PreparationStatus::Pending`.
- Failure to enqueue is visible and fail-closed; the task does not report preparation as scheduled when it is not.
- Tests cover the enqueue success path and a queue-failure or missing-context edge case.
**Files expected to change:**
- `crates/jobs/src/lib.rs`
- `apps/api/src/routes/ingestion.rs`
- `apps/api/tests/ingestion_test.rs`
**Evidence:**
- Job contract: `crates/jobs/src/lib.rs` now defines `PreparationJob`, a generic `JobEnvelope<T>`, `QueueError`, the `PreparationJobQueue` trait, and an `InMemoryPreparationJobQueue` seam that `T5b` can replace with a real worker-backed implementation later.
- API wiring: `apps/api/src/routes/ingestion.rs` now calls `schedule_preparation_job(...)` after `finalize_ingestion_core(...)`, resolves the source artifact, writes `PreparationStatus::Pending`, and enqueues one `PreparationJob`.
- Fail-closed handling: queue-enqueue failure or missing source-artifact context now flips preparation status to `Failed` with `error_detail` rather than silently pretending preparation was scheduled.
- Test harness: `apps/api/src/state.rs` now carries a shared preparation queue seam, and `apps/api/tests/ingestion_test.rs` can inject recording or failing queues to prove the new orchestration behavior.
- Verification:
  - `cargo test -p dubbridge-jobs --lib`
  - `cargo test -p dubbridge-api --test ingestion_test`
  - `cargo clippy -p dubbridge-api -p dubbridge-jobs --tests -- -D warnings`

### Reflection log

Required passes: 3 (`51` -> `Med-high`)

#### Pass 1

- **Draft verdict:** The task could land as a narrow contract/wiring slice if the queue stayed as an overrideable seam instead of prematurely coupling API finalization to the future worker backend.
- **Critique findings:**
  - `crates/jobs` had only a string queue stub, so the worker-facing payload contract did not exist yet.
  - `AppState` had no queue seam, which would make `finalize` hard to test or force the queue choice into route-level globals.
- **Revisions applied:**
  - Added `PreparationJob`, `PreparationJobQueue`, `QueueError`, and `InMemoryPreparationJobQueue` in `crates/jobs`.
  - Added shared preparation-queue wiring to `AppState` with a default in-memory implementation and test override support.

#### Pass 2

- **Draft verdict:** The success path worked, but fail-closed behavior needed to be explicit around post-finalize scheduling.
- **Critique findings:**
  - A queue failure after finalize could not be allowed to look like successful scheduling.
  - Missing source-artifact context needed observable failure state, not just an internal error response.
- **Revisions applied:**
  - Added `schedule_preparation_job(...)` after finalize success.
  - Persisted `PreparationStatus::Pending` before enqueue and switched to `PreparationStatus::Failed` with `error_detail` on enqueue or source-resolution failure.

#### Pass 3

- **Draft verdict:** The implementation matched the intended runtime seam, but the HP/EC contract still needed executable proof for every approved case.
- **Critique findings:**
  - `EC-2` required a dedicated test; the route logic alone was not enough for certification.
  - Duplicate-finalize behavior needed explicit evidence that preparation jobs are not duplicated.
- **Revisions applied:**
  - Added a queue-failure integration test and a trigger-backed missing-source integration test.
  - Added duplicate-enqueue assertions and finalized this ledger entry with certification and verification evidence.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | successful ingestion finalization enqueues one preparation job for the finalized asset | `crates/jobs/src/lib.rs::in_memory_queue_records_jobs`, `apps/api/tests/ingestion_test.rs::successful_ingestion_creates_asset_rights_artifact_and_audit` | passed |
| HP-2 | Happy path | the asset receives an initial `PreparationStatus::Pending` row when the job is scheduled | `apps/api/tests/ingestion_test.rs::successful_ingestion_creates_asset_rights_artifact_and_audit` | passed |
| HP-3 | Happy path | repeated finalize paths do not enqueue duplicate preparation work for the same finalized ingest token | `apps/api/tests/ingestion_test.rs::duplicate_finalization_does_not_create_duplicate_artifact` | passed |
| EC-1 | Edge case | queue-enqueue failure leaves the asset not ready and observable instead of silently pretending preparation will happen | `apps/api/tests/ingestion_test.rs::finalize_marks_preparation_failed_when_enqueue_fails` | passed |
| EC-2 | Edge case | missing finalized source-artifact context blocks enqueue fail-closed | `apps/api/tests/ingestion_test.rs::finalize_fails_closed_when_source_artifact_is_missing_for_preparation` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-19`
- Statement: I verified every happy path and edge case approved for `S-120-T5a` has executable evidence covering the preparation job contract, finalize enqueue path, duplicate suppression, and fail-closed scheduling outcomes.
- Commands run: `cargo test -p dubbridge-jobs --lib`; `cargo test -p dubbridge-api --test ingestion_test`; `cargo clippy -p dubbridge-api -p dubbridge-jobs --tests -- -D warnings`

## S-120-T5b: Worker execution + readiness/failure status transitions
**Effort:** L (RRI 55 — Med-high)
**Depends on:** S-120-T5a
**Status:** Done (2026-06-19)
**Happy paths considered:**
- HP-1: The worker consumes one preparation job and transitions status from `Pending`/`InProgress` to `Ready` after probe and HLS artifacts exist.
- HP-2: The worker persists terminal readiness only after both probe metadata and HLS package are present.
**Edge cases considered:**
- EC-1: Probe/HLS validation failure transitions the asset to `Failed` with actionable `error_detail`.
- EC-2: Missing derived artifacts or mismatched readiness evidence must not produce `Ready`.
**Inputs:**
- Preparation job payload from `T5a`
- Persistence seams from `T3` and `T4`
- Worker runtime in `apps/worker-runner`
**Outputs:**
- Worker dispatch/handler for preparation jobs
- Durable readiness/failure transitions
- Runtime logs/metrics around preparation execution
**Acceptance criteria:**
- The worker can deserialize and process one preparation job shape.
- `Ready` is written only when the probe + HLS artifacts required by the slice exist.
- Failure paths persist `Failed` with observable details.
- Tests cover one success transition and one failure transition.
**Files expected to change:**
- `crates/jobs/src/lib.rs`
- `apps/worker-runner/src/main.rs`
- `crates/db/src/preparation_repo.rs`
- `apps/api/tests/preparation_repo_test.rs`
**Evidence:**
- Worker handler: `apps/worker-runner/src/main.rs` now defines `process_preparation_envelope(...)` and `process_preparation_job(...)`, marks preparation `InProgress`, persists probe/HLS artifacts through the existing `T3`/`T4` seams, and writes `Ready` only after DB evidence confirms probe + manifest + segment presence.
- Executor seam: `apps/worker-runner/src/main.rs` now introduces the async `PreparationExecutor` trait and a `SubprocessPreparationExecutor` that spools source bytes into temp files, runs `ffprobe` / `ffmpeg` via the pure command builders from `crates/media`, validates the outputs, and returns canonical probe/HLS payloads to the handler.
- Readiness gate: `crates/db/src/preparation_repo.rs` now exposes `PreparationReadinessEvidence` plus `get_preparation_readiness_evidence(...)`, so the `Ready` transition is evidence-driven instead of inferred.
- Readiness coverage: `apps/api/tests/preparation_repo_test.rs` now verifies readiness evidence is `true` only with probe + manifest + segment artifacts and remains `false` when HLS evidence is incomplete.
- Verification:
  - `cargo test -p dubbridge-worker-runner`
  - `cargo test -p dubbridge-api --test preparation_repo_test`
  - `cargo clippy -p dubbridge-worker-runner -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

### Reflection log

Required passes: 3 (`55` -> `Med-high`)

#### Pass 1

- **Draft verdict:** `T5b` could stay bounded if the worker owned orchestration while reusing the already-approved `T3`/`T4` persistence seams instead of re-implementing media rules.
- **Critique findings:**
  - The worker still had no handler at all; only a startup skeleton existed.
  - There was no reusable readiness-evidence query, so any `Ready` transition would have been ad hoc.
- **Revisions applied:**
  - Added `PreparationExecutor`, `process_preparation_envelope(...)`, and `process_preparation_job(...)` in `apps/worker-runner/src/main.rs`.
  - Added `PreparationReadinessEvidence` and `get_preparation_readiness_evidence(...)` to `preparation_repo`.

#### Pass 2

- **Draft verdict:** The success path worked, but `Ready` needed an explicit fail-closed gate tied to persisted derived-artifact evidence.
- **Critique findings:**
  - Persisting probe/HLS outputs alone was not enough; the handler had to re-check the DB evidence before declaring readiness.
  - Failure states needed to preserve actionable `error_detail` after probe or HLS problems.
- **Revisions applied:**
  - The worker now marks `InProgress` before execution, flips to `Failed` with full error detail on any error, and writes `Ready` only after readiness evidence reports probe + manifest + segment presence.
  - Added a repo-level readiness summary so downstream slices can share the same gate.

#### Pass 3

- **Draft verdict:** The runtime logic was in place, but the HP/EC contract still needed tests that prove both successful readiness and explicit non-readiness on invalid HLS evidence.
- **Critique findings:**
  - A generic worker failure test was not enough to prove `EC-2`; invalid HLS output had to demonstrate that the handler refuses `Ready`.
  - The readiness helper itself needed direct integration coverage in the repo test suite.
- **Revisions applied:**
  - Added worker tests for success, transcode failure, and invalid-HLS non-readiness.
  - Added repo integration tests for complete and incomplete readiness evidence.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | worker consumes one preparation job and transitions from `Pending`/`InProgress` to `Ready` after probe and HLS artifacts exist | `apps/worker-runner/src/main.rs::tests::process_preparation_job_marks_ready_when_probe_and_hls_exist` | passed |
| HP-2 | Happy path | `Ready` is written only when both probe metadata and HLS package evidence exist | `apps/worker-runner/src/main.rs::tests::process_preparation_job_marks_ready_when_probe_and_hls_exist`, `apps/api/tests/preparation_repo_test.rs::preparation_readiness_evidence_is_ready_when_required_artifacts_exist` | passed |
| EC-1 | Edge case | probe/HLS execution failure transitions the asset to `Failed` with actionable `error_detail` | `apps/worker-runner/src/main.rs::tests::process_preparation_job_marks_failed_when_hls_stage_fails` | passed |
| EC-2 | Edge case | missing derived artifacts or mismatched readiness evidence must not produce `Ready` | `apps/worker-runner/src/main.rs::tests::process_preparation_job_does_not_mark_ready_when_hls_output_is_invalid`, `apps/api/tests/preparation_repo_test.rs::preparation_readiness_evidence_is_incomplete_without_hls` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-19`
- Statement: I verified the worker-side preparation runtime, readiness gate, and failure transitions have executable evidence, and `Ready` is now gated by persisted probe/HLS evidence rather than by handler assumptions.
- Commands run: `cargo test -p dubbridge-worker-runner`; `cargo test -p dubbridge-api --test preparation_repo_test`; `cargo clippy -p dubbridge-worker-runner -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

## S-120-T5c: Docs sync and roadmap evidence
**Effort:** S (RRI 16 — Low)
**Depends on:** S-120-T5a, S-120-T5b
**Status:** Done (2026-06-19)
**Happy paths considered:**
- HP-1: Slice/task/roadmap docs reflect the real readiness contract and delivered preparation runtime.
**Edge cases considered:**
- EC-1: No status document is left stale after `T5a`/`T5b` completion.
**Inputs:**
- Results from `T5a` and `T5b`
- Current roadmap/task/plan docs
**Outputs:**
- Synced task ledger and roadmap state
- Any required architecture/documentation note updates
**Acceptance criteria:**
- `docs/tasks/s-120-media-preparation.md` is updated with evidence and certification for the completed subtasks.
- `docs/plan/roadmap.md` and any materially affected docs no longer show stale pre-runtime wording.
- `make qa-docs` passes.
**Files expected to change:**
- `docs/tasks/s-120-media-preparation.md`
- `docs/plan/roadmap.md`
- Any directly affected canonical doc discovered during closeout
**Evidence:**
- Slice status: this ledger is now `status: closed`, with `T5a`, `T5b`, and `T5c` all recorded as complete on 2026-06-19.
- Roadmap sync: `docs/plan/roadmap.md` now marks `S-120` done and summarizes the delivered scope as schema/lineage, probe persistence, HLS persistence, finalize enqueue, worker execution, and readiness gating.
- Plan sync: `docs/plan/s-120-media-preparation.md` now reflects slice completion instead of a pre-implementation planned state.
- Drift cleanup: the roadmap planning-gap wording no longer implies `S-120` is merely a newly created plan/task pair awaiting implementation.
- Verification:
  - `make qa-docs`

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-19`
- Statement: I verified the materially affected status documents for `S-120` now consistently represent the slice as implemented and closed, with no remaining stale pre-runtime wording.
- Commands run: `make qa-docs`
