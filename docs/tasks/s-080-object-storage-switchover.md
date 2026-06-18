---
type: TaskList
title: "Tasks: S-080 — Object Storage Switchover"
status: closed
slice: S-080
plan: docs/plan/s-080-object-storage-switchover.md
governed_by: [ADR-006]
---
# Tasks: S-080 — Object Storage Switchover

Plan: `docs/plan/s-080-object-storage-switchover.md` · ADRs: ADR-006, ADR-018, ADR-021, ADR-025, ADR-026 · Roadmap: S-080, X9

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

Behavioral coverage contract: unit-v1

This ledger decomposes the approved `S-080` slice into bounded tasks. The slice-level
RRI was `77` (`High`), so work proceeds only through these bounded tasks.

| Task | Title | Depends on | Type | Effort | Status |
|---|---|---|---|---|---|
| S-080-T0 | Slice decomposition + docs creation | — | Planning | S | [x] Done 2026-06-17 |
| S-080-T1 | Storage contract + canonical key ownership | S-080-T0 | Development | M | [x] Done 2026-06-17 |
| S-080-T2a | S3Adapter: dependency, struct, and constructor | S-080-T1 | Development | S | [x] Done 2026-06-17 |
| S-080-T2b | S3Adapter: StorageAdapter impl + tests | S-080-T2a | Development | M | [x] Done 2026-06-17 |
| S-080-T3 | Runtime selector wiring + local MinIO parity | S-080-T1, S-080-T2b | Development | L | [x] Done 2026-06-17 |
| S-080-T4 | Bounded-memory upload path for API ingestion | S-080-T1, S-080-T2b | Development | L | [x] Done 2026-06-17 |
| S-080-T5a | Immediate cleanup regression coverage + orphan observability | S-080-T3, S-080-T4 | Development | M | [x] Done 2026-06-17 |
| S-080-T5b | Storage reconciliation candidate listing seam | S-080-T2b, S-080-T3 | Development | M | [x] Done 2026-06-17 |
| S-080-T5c | Reconciliation planner against relational references | S-080-T4, S-080-T5b | Development | L | [x] Done 2026-06-17 |
| S-080-T5d | Reconciliation executor + operator docs | S-080-T5a, S-080-T5c | Development | L | [x] Done 2026-06-17 |
| S-080-T6 | Verification, docs sync, and roadmap evidence | S-080-T3, S-080-T4, S-080-T5a, S-080-T5b, S-080-T5c, S-080-T5d | Development | S | [x] Done 2026-06-18 |

---

## S-080-T0 — Slice decomposition + docs creation

**Effort:** S  
**Depends on:** nothing

### Scope

Create the missing plan and task ledger for `S-080`, align the roadmap so the slice no
longer shows `no plan yet`, and define the next executable storage tasks.

### Acceptance criteria

- `docs/plan/s-080-object-storage-switchover.md` exists and documents scope,
  constraints, and decomposition.
- `docs/tasks/s-080-object-storage-switchover.md` exists and defines ordered tasks.
- `docs/plan/roadmap.md` no longer shows `S-080` as `no plan yet`.

### Files affected

- `docs/plan/s-080-object-storage-switchover.md`
- `docs/tasks/s-080-object-storage-switchover.md`
- `docs/plan/roadmap.md`

### Status: [x] Done — 2026-06-17

**Evidence:**
- Slice plan authored in `docs/plan/s-080-object-storage-switchover.md`.
- Tasks ledger authored in `docs/tasks/s-080-object-storage-switchover.md`.
- Roadmap status/source synced in `docs/plan/roadmap.md`.

---

## S-080-T1 — Storage contract + canonical key ownership

**Effort:** M  
**Depends on:** S-080-T0

### Scope

Refine `StorageAdapter` and the storage helper API so key construction and object
reference behavior live in `crates/storage`, not in `apps/api/src/routes/ingestion.rs`.
Define the object-write contract that later tasks will implement for both local and
S3-compatible backends.

### Acceptance criteria

- The storage crate owns canonical key builders for upload and future derived-artifact paths.
- API/worker callers no longer hand-roll upload keys directly.
- The trait/API shape is sufficient for local fs and S3-compatible implementations.
- Existing local behavior remains covered by unit tests after the contract change.

### Files affected

- `crates/storage/src/adapter.rs`
- `crates/storage/src/lib.rs`
- `crates/storage/src/local.rs`
- `apps/api/src/routes/ingestion.rs`

### Happy paths considered

- `HP-1`: a source asset upload resolves to a canonical storage-owned key under `assets/...`.
- `HP-2`: local-fs behavior still stores and reads an object correctly after the contract change.

### Edge cases considered

- `EC-1`: malformed or empty filename input still yields a deterministic valid key.
- `EC-2`: callers cannot bypass canonical key construction with ad hoc object paths.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `crates/storage::ingest_key(token, filename)` added to `crates/storage/src/lib.rs`.
- `sanitize_filename` moved into `crates/storage` as `pub(crate)`.
- `build_storage_key` and `sanitize_filename` deleted from `apps/api/src/routes/ingestion.rs`.
- Call site updated to `dubbridge_storage::ingest_key(ingest_token, filename.as_deref())`.
- `cargo test -p dubbridge-storage`: 18/18 pass (includes 5 new ingest_key tests).
- `cargo test -p dubbridge-api`: all unit + integration tests pass (ingestion suite: 20/20).

---

## S-080-T2a — S3Adapter: dependency, struct, and constructor

**Effort:** S  
**Depends on:** S-080-T1

### Scope

Add the `object_store` crate dependency and define the `S3Adapter` struct with its
constructor and error-mapping helper. No I/O methods are implemented here; this task
establishes the foundation that T2b builds on.

### Acceptance criteria

- `object_store` (workspace version, feature `aws`) added to `crates/storage/Cargo.toml`.
- `S3Adapter { store: Arc<dyn ObjectStore>, bucket: String }` defined in `crates/storage/src/s3.rs`.
- `S3Adapter::new(config: &StorageConfig) -> Result<Self, StorageError>` implemented — returns
  `StorageError::Backend` if the `object_store` builder fails.
- `fn map_os_error(key: &str, e: object_store::Error) -> StorageError` defined — maps
  `object_store::Error::NotFound` to `StorageError::NotFound`, all others to `StorageError::Backend`.
- `pub mod s3` and `pub use s3::S3Adapter` added to `crates/storage/src/lib.rs`.
- `cargo check -p dubbridge-storage` passes with no errors or warnings.
- Existing tests (`cargo test -p dubbridge-storage`) remain green.

### Files affected

- `crates/storage/Cargo.toml`
- `crates/storage/src/s3.rs` (new)
- `crates/storage/src/lib.rs`

### Edge cases considered

- `EC-2`: `S3Adapter::new` with an invalid endpoint or missing bucket returns
  `StorageError::Backend` — no panic, no unwrap.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `object_store.workspace = true` added to `crates/storage/Cargo.toml`.
- `S3Adapter` struct and `S3Adapter::new` implemented in `crates/storage/src/s3.rs`.
- `map_os_error` helper defined; `#[allow(dead_code)]` applied pending T2b usage.
- `pub mod s3` + `pub use s3::S3Adapter` wired in `lib.rs`.
- `cargo clippy -p dubbridge-storage -- -D warnings`: clean.
- `cargo test -p dubbridge-storage`: 18/18 green.

---

## S-080-T2b — S3Adapter: StorageAdapter impl + tests

**Effort:** M  
**Depends on:** S-080-T2a

### Scope

Implement the four `StorageAdapter` methods on `S3Adapter` and write focused unit
tests using `object_store::memory::InMemory` as the test backend (no MinIO in CI).

### Acceptance criteria

- `impl StorageAdapter for S3Adapter` complete: `put`, `get`, `delete`, `object_url`.
- `put` stores bytes via `object_store::put`; returns the key string on success.
- `get` retrieves bytes via `object_store::get`; collects result bytes.
- `delete` removes the object via `object_store::delete`.
- `object_url` returns `s3://{bucket}/{key}` — never a public HTTP URL.
- All `object_store` errors go through `map_os_error` — no raw `.unwrap()` or `.expect()`.
- Unit tests (using `InMemory` store) cover:
  - `HP-1`: put then get returns identical bytes.
  - `HP-2`: put, delete, then get returns `StorageError::NotFound`.
  - `EC-1`: get on a missing key returns `StorageError::NotFound`.
- `cargo test -p dubbridge-storage` green; all T1 tests remain intact.

### Files affected

- `crates/storage/src/s3.rs`

### Happy paths considered

- `HP-1`: a binary object is stored and retrieved round-trip through the S3-compatible adapter.
- `HP-2`: deleting an existing object succeeds and subsequent reads fail with `NotFound`.

### Edge cases considered

- `EC-1`: a missing object maps to `StorageError::NotFound`, not a backend-generic failure.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `impl StorageAdapter for S3Adapter` completo: `put`, `get`, `delete`, `object_url`.
- `#[allow(dead_code)]` de T2a eliminados.
- Tests en `s3::tests` usando `object_store::memory::InMemory`: HP-1, HP-2, EC-1, `object_url`.
- `cargo clippy -p dubbridge-storage -- -D warnings`: limpio.
- `cargo test -p dubbridge-storage`: 22/22 green (18 anteriores + 4 nuevos).

---

## S-080-T3 — Runtime selector wiring + local MinIO parity

**Effort:** L  
**Depends on:** S-080-T1, S-080-T2b

### Scope

Wire `build_adapter()` and runtime startup to honor `storage.backend`, and make the
local compose environment usable for the S3-compatible path with MinIO.

### Acceptance criteria

- `build_adapter()` chooses local fs vs S3-compatible backend from config.
- API and worker startup can construct the selected backend.
- Local compose documentation and env wiring support MinIO for `storage.backend = s3`.
- Startup or first use fails clearly when S3 settings are incomplete or invalid.

### Files affected

- `crates/storage/src/lib.rs`
- `crates/storage/src/config.rs`
- `apps/api/src/main.rs`
- `apps/worker-runner/src/main.rs`
- `infra/local/docker-compose.yml`
- `README.md`

### Happy paths considered

- `HP-1`: `storage.backend = local_fs` preserves the current development path.
- `HP-2`: `storage.backend = s3` targets MinIO locally and constructs the correct adapter.

### Edge cases considered

- `EC-1`: selecting `s3` without required endpoint/bucket settings fails closed.
- `EC-2`: local compose keeps working for the current local-fs path while MinIO parity is introduced.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `StorageConfig` now carries `StorageBackend`, and `build_adapter()` returns a
  fallible adapter selected from `storage.backend`.
- S3 selector startup validation fails closed for an empty bucket or malformed
  endpoint URL before first object I/O.
- `apps/api` and `apps/worker-runner` initialize the configured storage backend at
  startup and propagate construction failures with explicit context.
- `infra/local/docker-compose.yml` supports local MinIO parity with a `minio-init`
  bucket creation service, S3-compatible endpoint env wiring, and local-only MinIO
  credentials.
- `README.md` documents the local MinIO path through
  `DUBBRIDGE_STORAGE_BACKEND=s3`.
- T3 RRI was computed as `43` (`Med-high`), so Effort was corrected from `M` to `L`
  per `docs/policies/RRI_POLICY.md`.

### Reflection log

Required passes: 3 (`43` -> `Med-high`)

#### Pass 1

- **Draft verdict:** selector wiring, startup propagation, compose env, README docs,
  and selector tests were implemented.
- **Critique findings:** `unwrap_err()` in a selector test required `Debug` on the
  storage trait object.
- **Revisions applied:** replaced `unwrap_err()` with an explicit `match` on the
  expected `StorageError::Backend`.

#### Pass 2

- **Draft verdict:** local-fs and S3 adapter construction paths compiled and storage
  tests passed.
- **Critique findings:** MinIO was exposed in compose, but local S3 parity still
  needed bucket creation to be usable on first run.
- **Revisions applied:** added `minio-init` using `minio/mc` and made app-profile
  containers wait for bucket creation.

#### Pass 3

- **Draft verdict:** storage, API, worker, compose profile, and docs were aligned.
- **Critique findings:** status docs still contained stale `implementation pending`
  language and T3 Effort did not match the computed RRI band.
- **Revisions applied:** corrected T3 Effort to `L` and synced this ledger, the
  slice plan, and the roadmap status.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `storage.backend = local_fs` preserves the current development path | `crates/storage/src/lib.rs::tests::build_adapter_local_fs_returns_file_adapter` | passed |
| HP-2 | Happy path | `storage.backend = s3` constructs an S3-compatible adapter for the configured bucket/endpoint | `crates/storage/src/lib.rs::tests::build_adapter_s3_returns_s3_adapter` | passed |
| EC-1 | Edge case | selecting `s3` without required bucket or with an invalid endpoint fails closed | `crates/storage/src/lib.rs::tests::build_adapter_s3_requires_bucket`, `crates/storage/src/lib.rs::tests::build_adapter_s3_rejects_invalid_endpoint` | passed |
| EC-2 | Edge case | local-fs remains selectable while MinIO parity is introduced | `crates/storage/src/lib.rs::tests::build_adapter_local_fs_returns_file_adapter` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-17`
- Statement: I verified every happy path and edge case defined for this task has unit
  test evidence that replicates the expected behavior.
- Commands run: `cargo fmt`; `cargo check -p dubbridge-storage -p dubbridge-api -p dubbridge-worker-runner`; `cargo test -p dubbridge-storage`; `cargo clippy -p dubbridge-storage -p dubbridge-api -p dubbridge-worker-runner -- -D warnings`; `docker compose -f infra/local/docker-compose.yml config`; `docker compose -f infra/local/docker-compose.yml --profile app config`; `cargo test -p dubbridge-api`

---

## S-080-T4 — Bounded-memory upload path for API ingestion

**Effort:** L  
**Depends on:** S-080-T1, S-080-T2b

### Scope

Replace the current whole-file `Vec<u8>` multipart ingestion path with a bounded-memory
write path suitable for large uploads while preserving rights/finalize behavior.

### Acceptance criteria

- `create_ingestion()` no longer requires holding the entire upload body in one in-memory `Vec<u8>`.
- The stored object, checksum, size, and pending-ingestion metadata remain correct.
- Existing upload limits still reject oversized payloads cleanly.
- Focused tests prove the new path preserves the current successful upload contract.

### Files affected

- `apps/api/src/routes/ingestion.rs`
- `apps/api/tests/ingestion_test.rs`
- `crates/storage/src/adapter.rs`
- `crates/storage/src/local.rs`
- `crates/storage/src/s3.rs`

### Happy paths considered

- `HP-1`: a valid multipart upload stores the object, records pending ingestion metadata, and returns `201`.
- `HP-2`: checksum and stored size match the uploaded bytes after the bounded-memory write path.

### Edge cases considered

- `EC-1`: payload above `MAX_UPLOAD_BYTES` still returns `413`.
- `EC-2`: storage failure during upload returns an error without leaving a committed pending-ingestion row.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `apps/api/src/routes/ingestion.rs` now stages multipart file fields into a local
  spool file via `field.chunk()` while computing checksum and size incrementally,
  then delegates to `storage.put_file(...)`.
- `StorageAdapter` now exposes `put_file`, with bounded-memory implementations in
  `LocalFsAdapter` and `S3Adapter`.
- API integration tests cover metadata correctness after chunked upload, `413`
  limit behavior, and storage failure without a committed `pending_ingestions` row.

### Reflection log

Required passes: 3 (`44` -> `Med-high`)

#### Pass 1

- **Draft verdict:** replaced `field.bytes().await.to_vec()` with chunked spooling and
  added a file-based storage write contract.
- **Critique findings:** the storage boundary still only accepted `Vec<u8>`, so the
  bounded-memory route would have collapsed back into full buffering at the adapter.
- **Revisions applied:** added `StorageAdapter::put_file` and implemented it in local
  and S3 adapters.

#### Pass 2

- **Draft verdict:** the route wrote multipart chunks to a tempfile and persisted the
  resulting object through `put_file`.
- **Critique findings:** the new path needed explicit tests for metadata correctness
  and for the no-row-on-storage-failure fail-closed guarantee.
- **Revisions applied:** added API tests for checksum/size correctness and a failing
  storage adapter case that proves no `pending_ingestions` row is committed.

#### Pass 3

- **Draft verdict:** storage, API, and tests were aligned and the bounded-memory path
  worked end to end.
- **Critique findings:** stale task metadata still referenced `S-080-T2` generically
  and slice status docs still reflected only T0-T3 completion.
- **Revisions applied:** tightened the dependency reference to `S-080-T2b` and synced
  this ledger, the slice plan, and the roadmap status.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid multipart upload stores the object, records pending ingestion metadata, and returns `201` | `apps/api/tests/ingestion_test.rs::successful_ingestion_creates_asset_rights_artifact_and_audit` | passed |
| HP-2 | Happy path | checksum and stored size match uploaded bytes after the bounded-memory write path | `apps/api/tests/ingestion_test.rs::ingestion_records_checksum_and_size_for_chunked_upload` | passed |
| EC-1 | Edge case | payload above `MAX_UPLOAD_BYTES` still returns `413` | `apps/api/tests/ingestion_test.rs::upload_too_large_is_rejected` | passed |
| EC-2 | Edge case | storage failure during upload returns an error without leaving a committed pending-ingestion row | `apps/api/tests/ingestion_test.rs::storage_failure_does_not_persist_pending_ingestion_row` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-17`
- Statement: I verified every happy path and edge case defined for this task has unit
  test evidence that replicates the expected behavior.
- Commands run: `cargo fmt`; `cargo check -p dubbridge-storage -p dubbridge-api`; `cargo test -p dubbridge-storage`; `cargo test -p dubbridge-api ingestion -- --test-threads=1`; `cargo clippy -p dubbridge-storage -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api`; `cargo check -p dubbridge-worker-runner`

---

## Replan note — 2026-06-17

The original `S-080-T5` task-level presentation computed `RRI 61` (`Complex`), which
made the work too broad for a single safe implementation pass. This ledger therefore
supersedes `S-080-T5` with `S-080-T5a` through `S-080-T5d`, each with a narrower
scope and its own acceptance criteria and behavioral coverage.

At execution time, recompute RRI per subtask before starting it. The Effort values
below are planning-time targets for the narrowed scopes and must be corrected if the
measured RRI disagrees.

---

## S-080-T5a — Immediate cleanup regression coverage + orphan observability

**Effort:** M  
**Depends on:** S-080-T3, S-080-T4

### Scope

Lock down the already-intended immediate cleanup behavior on pending-ingestion
persistence failure, and make orphan-producing cleanup failures observable enough for
later repair.

### Acceptance criteria

- Pending-ingestion persistence failure attempts exactly one immediate object deletion.
- Cleanup failure logging includes enough context to correlate the orphaned object with
  the ingest token and storage key.
- Regression tests prove the cleanup path preserves the existing fail-closed posture.
- The cleanup path returns the original relational persistence failure to the caller;
  cleanup failure must not mask it.

### Files affected

- `apps/api/src/routes/ingestion.rs`
- `apps/api/tests/ingestion_test.rs`

### Happy paths considered

- `HP-1`: metadata persistence failure triggers immediate successful object cleanup.
- `HP-2`: the cleanup regression tests prove no pending-ingestion row remains after the
  failed relational write path.

### Edge cases considered

- `EC-1`: cleanup deletion itself fails and leaves an orphan; the system logs it explicitly.
- `EC-2`: cleanup failure does not replace or hide the original relational persistence error.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `apps/api/tests/ingestion_test.rs` now includes a tracking storage adapter that
  proves the pending-ingestion persistence failure path attempts exactly one cleanup
  delete against the same canonical ingest key that was written.
- The same test suite now forces a relational insert failure after successful object
  storage and proves no `pending_ingestions` row remains after the failed write.
- A second regression test captures the warning log emitted when cleanup deletion
  fails and proves the HTTP-visible error remains the original relational failure.
- No production-route code change was needed: the existing cleanup path in
  `apps/api/src/routes/ingestion.rs` already satisfied the intended fail-closed
  ordering once the regression coverage and observability assertions were added.

### Reflection log

Required passes: 2 (`27` -> `Moderate`)

#### Pass 1

- **Draft verdict:** added a tracking `StorageAdapter` plus a forced-DB-failure path in
  integration tests to exercise object-written / metadata-write-failed cleanup.
- **Critique findings:** exact-once cleanup and original-error preservation were both
  required, so a single happy-path cleanup test was insufficient.
- **Revisions applied:** split the coverage into two tests: one for successful cleanup
  and one for cleanup failure while preserving the original DB error.

#### Pass 2

- **Draft verdict:** the two new tests covered cleanup success, cleanup failure, and
  no-row-left-behind behavior.
- **Critique findings:** the task also required explicit orphan observability, which
  needed evidence that the warning log includes enough repair context.
- **Revisions applied:** added a scoped tracing subscriber in test, captured the
  warning output, and asserted the presence of the warning message, `ingest_token`,
  `storage_key`, and cleanup error text.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | metadata persistence failure triggers immediate successful object cleanup | `apps/api/tests/ingestion_test.rs::pending_ingestion_persistence_failure_deletes_stored_object_once` | passed |
| HP-2 | Happy path | regression coverage proves no pending-ingestion row remains after the failed relational write path | `apps/api/tests/ingestion_test.rs::pending_ingestion_persistence_failure_deletes_stored_object_once` | passed |
| EC-1 | Edge case | cleanup deletion itself fails and leaves an orphan; the system logs it explicitly | `apps/api/tests/ingestion_test.rs::pending_ingestion_persistence_failure_logs_orphan_context_and_preserves_db_error` | passed |
| EC-2 | Edge case | cleanup failure does not replace or hide the original relational persistence error | `apps/api/tests/ingestion_test.rs::pending_ingestion_persistence_failure_logs_orphan_context_and_preserves_db_error` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-17`
- Statement: I verified every happy path and edge case defined for this task has unit
  test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-api pending_ingestion_persistence_failure -- --test-threads=1`; `cargo test -p dubbridge-api ingestion -- --test-threads=1`; `cargo check -p dubbridge-api`

---

## S-080-T5b — Storage reconciliation candidate listing seam

**Effort:** M  
**Depends on:** S-080-T2b, S-080-T3

### Scope

Extend the storage boundary with a narrow, storage-owned way to enumerate candidate
object keys for orphan reconciliation without leaking provider-specific SDK details
into the API or workers.

### Acceptance criteria

- `StorageAdapter` exposes a reconciliation-friendly listing seam scoped to canonical
  storage prefixes.
- Local-fs and S3-compatible adapters both implement the seam.
- Focused unit tests cover empty listings and stable key enumeration.
- The seam returns canonical storage keys or references only; it must not introduce
  public URLs or provider-specific details into higher layers.

### Files affected

- `crates/storage/src/adapter.rs`
- `crates/storage/src/local.rs`
- `crates/storage/src/s3.rs`
- `crates/storage/src/lib.rs`

### Happy paths considered

- `HP-1`: listing under the ingest prefix returns candidate canonical object keys.
- `HP-2`: local-fs and S3-compatible adapters expose the same canonical keys for the
  same stored objects.

### Edge cases considered

- `EC-1`: listing an empty or non-existent prefix returns an empty set, not an error.
- `EC-2`: the listing seam does not expose provider-specific transport details to callers.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `StorageAdapter` now exposes `list_keys(prefix) -> Result<Vec<String>, StorageError>`
  in `crates/storage/src/adapter.rs` as the narrow reconciliation listing seam.
- `LocalFsAdapter` enumerates recursively under `base_path/prefix`, maps paths back to
  canonical storage keys, sorts them for stability, and returns `Ok(vec![])` for empty
  or missing prefixes.
- `S3Adapter` lists object-store entries under the requested prefix, returns canonical
  object keys only, sorts them for stability, and returns `Ok(vec![])` for empty prefixes.
- `crates/storage/src/lib.rs` now exports `INGESTS_PREFIX`, and a parity test proves
  local-fs and S3 return the same canonical ingest keys for the same stored objects.

### Reflection log

Required passes: 2 (`36` -> `Moderate`)

#### Pass 1

- **Draft verdict:** added `StorageAdapter::list_keys`, implemented recursive local-fs
  enumeration, and implemented S3 object-store listing with canonical key output.
- **Critique findings:** backend parity was only implied by duplicate expectations in
  backend-specific tests; the task required direct proof that both adapters enumerate
  the same canonical keys.
- **Revisions applied:** added a cross-adapter parity test in `crates/storage/src/lib.rs`
  and introduced a test-only `S3Adapter::new_for_tests` helper to keep production
  fields private.

#### Pass 2

- **Draft verdict:** all adapter tests, empty-prefix behavior, and explicit parity
  coverage were in place.
- **Critique findings:** status artifacts still reported `T5b` as pending, which would
  leave slice progress stale after reporting completion.
- **Revisions applied:** synced this ledger, the slice plan header, and the roadmap row
  to show `T5b` complete and `T5c-T5d`/`T6` pending.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | listing under the ingest prefix returns candidate canonical object keys | `crates/storage/src/local.rs::tests::list_keys_returns_sorted_canonical_ingest_keys`, `crates/storage/src/s3.rs::tests::list_keys_returns_sorted_canonical_ingest_keys` | passed |
| HP-2 | Happy path | local-fs and S3-compatible adapters expose the same canonical keys for the same stored objects | `crates/storage/src/lib.rs::tests::local_fs_and_s3_list_the_same_canonical_ingest_keys` | passed |
| EC-1 | Edge case | listing an empty or non-existent prefix returns an empty set, not an error | `crates/storage/src/local.rs::tests::list_keys_empty_or_missing_prefix_returns_empty`, `crates/storage/src/s3.rs::tests::list_keys_empty_or_missing_prefix_returns_empty` | passed |
| EC-2 | Edge case | the listing seam does not expose provider-specific transport details to callers | `crates/storage/src/local.rs::tests::list_keys_returns_sorted_canonical_ingest_keys`, `crates/storage/src/s3.rs::tests::list_keys_returns_sorted_canonical_ingest_keys`, `crates/storage/src/lib.rs::tests::local_fs_and_s3_list_the_same_canonical_ingest_keys` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-17`
- Statement: I verified every happy path and edge case defined for this task has unit
  test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-storage`; `cargo clippy -p dubbridge-storage -- -D warnings`; `cargo check -p dubbridge-storage`

---

## S-080-T5c — Reconciliation planner against relational references

**Effort:** L  
**Depends on:** S-080-T4, S-080-T5b

### Scope

Build the reconciliation planning logic that compares storage candidates against the
relational source of truth and identifies which keys are safe orphan-delete
candidates before any destructive action runs.

### Acceptance criteria

- The planner can load candidate object keys from storage and referenced keys from
  relational state.
- The planner outputs or logs a deterministic orphan-vs-retained decision set.
- Tests prove referenced objects are retained and unreferenced objects are flagged as
  orphan candidates.
- Unexpected or malformed candidate keys are skipped or logged fail-closed rather
  than deleted.

### Files affected

- `apps/api/src/cleanup.rs`
- `apps/api/tests/ingestion_test.rs`
- `crates/storage/src/*`

### Happy paths considered

- `HP-1`: an unreferenced ingest object is identified as an orphan candidate.
- `HP-2`: an object still referenced by relational state is retained.

### Edge cases considered

- `EC-1`: duplicate candidate keys do not produce duplicate orphan actions.
- `EC-2`: malformed or unexpected candidate keys are skipped and logged, not deleted.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `apps/api/src/cleanup.rs` now exposes `plan_ingest_reconciliation(...)`, which loads
  candidate ingest keys from `StorageAdapter::list_keys(INGESTS_PREFIX)` and compares
  them against relational references from `pending_ingestions.storage_key` and
  `artifact_records.storage_key`.
- The planner returns a deterministic `IngestReconciliationPlan` with `retained`,
  `orphan_candidates`, and `skipped` keys. It does not delete or mutate storage.
- Malformed candidates are skipped with structured reasons and warning logs; duplicate
  candidates are deduplicated before classification.
- Integration tests in `apps/api/tests/ingestion_test.rs` cover referenced objects,
  orphan candidates, and malformed storage candidates.

### Reflection log

Required passes: 3 (`44` -> `Med-high`)

#### Pass 1

- **Draft verdict:** added the planner API, relational key loading, deterministic
  classification, and integration coverage for retained-vs-orphan outcomes.
- **Critique findings:** relational references needed to include both pending rows and
  finalized artifact rows to preserve cross-store safety after finalization.
- **Revisions applied:** `load_referenced_storage_keys` now unions
  `pending_ingestions.storage_key` and `artifact_records.storage_key`.

#### Pass 2

- **Draft verdict:** the planner compared storage candidates against relational truth
  and returned stable vectors.
- **Critique findings:** malformed and unexpected keys needed an explicit fail-closed
  representation instead of merely disappearing from the plan.
- **Revisions applied:** added `SkippedReconciliationKey` and
  `ReconciliationSkipReason`, plus warning logs for skipped candidates.

#### Pass 3

- **Draft verdict:** tests covered retained, orphan, malformed, and duplicate candidate
  behavior, with no deletion path added.
- **Critique findings:** task status docs still described `T5c` as pending after code
  verification.
- **Revisions applied:** synced this ledger, the slice plan header, and the roadmap row
  to show `T5c` complete while leaving `T5d` and `T6` pending.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | an unreferenced ingest object is identified as an orphan candidate | `apps/api/tests/ingestion_test.rs::reconciliation_plan_retains_referenced_objects_and_flags_orphans` | passed |
| HP-2 | Happy path | an object still referenced by relational state is retained | `apps/api/tests/ingestion_test.rs::reconciliation_plan_retains_referenced_objects_and_flags_orphans` | passed |
| EC-1 | Edge case | duplicate candidate keys do not produce duplicate orphan actions | `apps/api/src/cleanup.rs::tests::plan_from_candidate_keys_deduplicates_and_skips_malformed_keys` | passed |
| EC-2 | Edge case | malformed or unexpected candidate keys are skipped and logged, not deleted | `apps/api/tests/ingestion_test.rs::reconciliation_plan_skips_malformed_storage_candidates`, `apps/api/src/cleanup.rs::tests::plan_from_candidate_keys_deduplicates_and_skips_malformed_keys` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-17`
- Statement: I verified every happy path and edge case defined for this task has unit
  test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-api reconciliation_plan -- --test-threads=1`; `cargo check -p dubbridge-api`; `cargo test -p dubbridge-api`; `cargo test -p dubbridge-storage`; `cargo clippy -p dubbridge-api -p dubbridge-storage -- -D warnings`; `cargo check -p dubbridge-api -p dubbridge-storage`

---

## S-080-T5d — Reconciliation executor + operator docs

**Effort:** L  
**Depends on:** S-080-T5a, S-080-T5c

### Scope

Wire the approved reconciliation plan into an executable manual or scheduled cleanup
path, and document how operators can run or reason about orphan repair safely.

### Acceptance criteria

- The runtime exposes a reconciliation execution path that deletes only planned
  orphan keys.
- Delete failures are logged with enough context for later retry or repair.
- Docs describe the immediate cleanup path, the reconciliation boundary, and the
  intended operator flow.
- Re-running reconciliation is idempotent for already-deleted orphan keys.

### Files affected

- `apps/api/src/cleanup.rs`
- `apps/api/src/main.rs`
- `apps/api/tests/ingestion_test.rs`
- `README.md` or `docs/architecture.md`

### Happy paths considered

- `HP-1`: a planned orphan key is deleted and recorded by the reconciliation run.
- `HP-2`: a second reconciliation run over the same repaired state is a no-op.

### Edge cases considered

- `EC-1`: delete failure leaves enough repair context for a later rerun.
- `EC-2`: the executor never deletes keys outside the planner-approved orphan set.

### Status: [x] Done — 2026-06-17

**Evidence:**
- `apps/api/src/cleanup.rs` now exposes `run_ingest_reconciliation(...)`, which executes
  only `IngestReconciliationPlan.orphan_candidates` through `StorageAdapter::delete`.
- The executor returns `IngestReconciliationRun` with deleted, already-absent, and failed
  delete outcomes, and logs delete failures with `storage_key` and error context for retry.
- `apps/api/src/main.rs` runs reconciliation from the existing hourly cleanup worker after
  expired pending-ingestion cleanup.
- `README.md` documents the immediate cleanup path, reconciliation boundary, retry behavior,
  and idempotent rerun behavior.
- T5d RRI was computed as `46` (`Med-high`), so Effort was corrected from `M` to `L`
  per `docs/policies/RRI_POLICY.md`.

### Reflection log

Required passes: 3 (`46` -> `Med-high`)

#### Pass 1

- **Draft verdict:** added the executor result type, delete loop, and runtime invocation
  from the existing cleanup worker.
- **Critique findings:** the executor needed to make its delete scope auditable by
  exposing outcomes, not just logging side effects.
- **Revisions applied:** added `IngestReconciliationRun` with `deleted`, `already_absent`,
  and `failed` vectors.

#### Pass 2

- **Draft verdict:** orphan deletion and idempotent rerun behavior were covered by an
  integration test.
- **Critique findings:** delete failure needed enough operator context in logs, and retained
  or skipped keys needed explicit negative coverage.
- **Revisions applied:** added tests for delete-failure context and for ensuring retained
  or skipped keys never reach `StorageAdapter::delete`.

#### Pass 3

- **Draft verdict:** runtime, tests, and docs matched the task boundary without adding an
  external API or broad slice verification.
- **Critique findings:** task metadata still had planning-time `Effort: M`, which disagreed
  with the recomputed RRI band.
- **Revisions applied:** corrected T5d Effort to `L` and synced this ledger, the slice plan,
  and the roadmap while leaving `S-080-T6` unstarted.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | a planned orphan key is deleted and recorded by the reconciliation run | `apps/api/tests/ingestion_test.rs::reconciliation_run_deletes_orphan_and_second_run_is_noop` | passed |
| HP-2 | Happy path | a second reconciliation run over the same repaired state is a no-op | `apps/api/tests/ingestion_test.rs::reconciliation_run_deletes_orphan_and_second_run_is_noop` | passed |
| EC-1 | Edge case | delete failure leaves enough repair context for a later rerun | `apps/api/tests/ingestion_test.rs::reconciliation_run_records_delete_failure_for_retry` | passed |
| EC-2 | Edge case | the executor never deletes keys outside the planner-approved orphan set | `apps/api/tests/ingestion_test.rs::reconciliation_run_never_deletes_retained_or_skipped_keys` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-17`
- Statement: I verified every happy path and edge case defined for this task has unit
  test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-api reconciliation -- --test-threads=1`; `cargo check -p dubbridge-api -p dubbridge-storage`; `cargo test -p dubbridge-api`; `cargo test -p dubbridge-storage`; `cargo clippy -p dubbridge-api -p dubbridge-storage -- -D warnings`

---

## S-080-T6 — Verification, docs sync, and roadmap evidence

**Effort:** S  
**Depends on:** S-080-T3, S-080-T4, S-080-T5a, S-080-T5b, S-080-T5c, S-080-T5d

### Scope

Run the relevant verification, update status artifacts, and record the evidence needed
to mark `S-080` complete without leaving the roadmap or architecture docs stale.

### Acceptance criteria

- Relevant unit/integration verification commands are recorded and pass.
- `docs/plan/roadmap.md` and any affected architecture/runtime docs reflect the delivered storage path.
- Completion records include reflection log, unit coverage certification, and owner verification per workflow.

### Files affected

- `docs/plan/roadmap.md`
- `README.md`
- `docs/architecture.md`
- `docs/tasks/s-080-object-storage-switchover.md`

### Happy paths considered

- `HP-1`: the completed slice is reflected consistently in roadmap, plan, tasks, and architecture docs.
- `HP-2`: verification covers local-fs compatibility and S3-compatible storage behavior.

### Edge cases considered

- `EC-1`: docs lag behind the implementation and misstate the active backend behavior.
- `EC-2`: test evidence exists for local paths only and misses S3-compatible regressions.

### Status: [x] Done — 2026-06-18

**Evidence:**
- `README.md` and `docs/architecture.md` now reflect the delivered S-080 storage path:
  S3-compatible adapter operational, bounded-memory upload path active, and orphan
  reconciliation behavior documented.
- Verification commands passed on `2026-06-18`:
  - `cargo test -p dubbridge-storage`
  - `cargo test -p dubbridge-api`
  - `cargo clippy -p dubbridge-storage -p dubbridge-api -- -D warnings`
  - `cargo check -p dubbridge-storage -p dubbridge-api`
- `python3 scripts/check_roadmap_drift_test.py`: 6/6 green.
- `make qa-docs` now passes through documentation consistency, task-unit coverage,
  and the repaired roadmap-drift logic.
- `docs/plan/roadmap.md` and `docs/plan/s-080-object-storage-switchover.md` now
  record `S-080` complete, consistent with the delivered storage path and QA evidence.

### Reflection log

Required passes: 1 (`10` -> `Low`)

#### Pass 1

- **Draft verdict:** final verification passed for the storage and API scopes, but
  `qa-docs` was still blocked by `check-roadmap-drift.sh` over-counting unrelated
  SID mentions in new plan/task files.
- **Critique findings:** the drift gate was using `grep -rl "$sid"` across
  `docs/plan` and `docs/tasks`, which treated any prose mention of a completed SID
  as canonical evidence and produced false positives for unrelated completed phases.
- **Revisions applied:** narrowed the drift gate to roadmap-declared canonical
  backtick paths, added a regression test for unrelated SID mentions, added a
  regression test for the adjacent `Plan:` evidence pattern used by the foundation
  gate table, reran the script tests, and reran `make qa-docs` to confirm the fix.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | the completed slice is reflected consistently in roadmap, plan, tasks, and architecture docs | `scripts/check_roadmap_drift_test.py::test_done_phase_with_committed_sid_evidence_passes`, `scripts/check_roadmap_drift_test.py::test_done_gate_can_use_adjacent_plan_line_evidence` | passed |
| HP-2 | Happy path | verification covers local-fs compatibility and S3-compatible storage behavior | `crates/storage/src/lib.rs::tests::build_adapter_local_fs_returns_file_adapter`, `crates/storage/src/lib.rs::tests::build_adapter_s3_returns_s3_adapter`, `apps/api/tests/ingestion_test.rs::reconciliation_run_deletes_orphan_and_second_run_is_noop` | passed |
| EC-1 | Edge case | docs lag behind the implementation and misstate the active backend behavior | `scripts/check_roadmap_drift_test.py::test_done_phase_with_uncommitted_evidence_fails`, `scripts/check_roadmap_drift_test.py::test_done_phase_without_sid_evidence_fails` | passed |
| EC-2 | Edge case | test evidence exists for local paths only and misses S3-compatible regressions | `crates/storage/src/s3.rs::tests::put_get_round_trip`, `crates/storage/src/s3.rs::tests::list_keys_returns_sorted_canonical_ingest_keys`, `crates/storage/src/lib.rs::tests::local_fs_and_s3_list_the_same_canonical_ingest_keys` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-18`
- Statement: I verified every happy path and edge case defined for this task has
  concrete automated evidence, the roadmap/plan/task status artifacts are now in
  sync, and the docs QA gate passes without the prior drift false positives.
- Commands run: `python3 scripts/check_roadmap_drift_test.py`; `cargo test -p dubbridge-storage`; `cargo test -p dubbridge-api`; `cargo clippy -p dubbridge-storage -p dubbridge-api -- -D warnings`; `cargo check -p dubbridge-storage -p dubbridge-api`; `make qa-docs`
