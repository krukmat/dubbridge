# Tasks: S1 Asset Ingestion with Rights Ledger

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

---

## Task 1 — Domain types

**Effort:** M
**Depends on:** nothing

### Scope
Implement all domain types needed for S1 inside `crates/domain/src/`:

- `asset.rs`: `AssetId`, `Asset`, `IngestionStatus`
- `rights.rs`: `RightsRecord`, `RightsBasis`, `LicenseType`, `AllowedTerritory`, `AllowedLanguage`
- `artifact.rs`: `ArtifactRecord`, `ArtifactKind`
- `audit.rs`: `AuditEvent`, `AuditEventKind`
- `ingestion.rs`: `FinalizeIngestionCommand`, `IngestionError`
- `lib.rs`: re-export all modules

### Acceptance criteria
- All types compile with `cargo check`
- `IngestionStatus` has no processing-ready variant
- `FinalizeIngestionCommand::validate()` returns `IngestionError::MissingRightsBasis` when rights_basis is None
- `FinalizeIngestionCommand::validate()` returns `IngestionError::MissingUploaderContext` when uploader_id is None

### Files affected
- `crates/domain/src/lib.rs`
- `crates/domain/src/asset.rs` (new)
- `crates/domain/src/rights.rs` (new)
- `crates/domain/src/artifact.rs` (new)
- `crates/domain/src/audit.rs` (new)
- `crates/domain/src/ingestion.rs` (new)
- `crates/domain/Cargo.toml` (add thiserror)

### Status: [x] DONE — 5/5 unit tests pass, cargo check clean.

---

## Task 2 — SQL migrations

**Effort:** S
**Depends on:** Task 1 (schema matches domain types)

### Scope
Create four migration files under `infra/migrations/`:

- `0001_create_assets.sql`
- `0002_create_rights_records.sql`
- `0003_create_artifact_records.sql`
- `0004_create_audit_events.sql`

### Acceptance criteria
- All migrations apply cleanly via `sqlx migrate run` against a fresh Postgres instance
- `artifact_records.ingest_token` has a UNIQUE constraint
- `assets.status` has a CHECK constraint limiting to defined values
- All tables have `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`

### Files affected
- `infra/migrations/0001_create_assets.sql` (new)
- `infra/migrations/0002_create_rights_records.sql` (new)
- `infra/migrations/0003_create_artifact_records.sql` (new)
- `infra/migrations/0004_create_audit_events.sql` (new)

### Status: [x] DONE — 4 migrations created, cargo check clean.

---

## Task 3 — SQLx repositories in crates/db

**Effort:** M
**Depends on:** Task 1, Task 2

### Scope
Implement repository functions in `crates/db/src/`:

- `lib.rs`: `create_pool(database_url) -> PgPool`, re-export repo modules
- `asset_repo.rs`: `insert_asset`, `find_asset_by_id`
- `rights_repo.rs`: `insert_rights_record`
- `artifact_repo.rs`: `insert_artifact_record`, `find_original_by_ingest_token`
- `audit_repo.rs`: `insert_audit_event`

### Acceptance criteria
- All functions use SQLx query APIs and typed row mappings where rows are decoded.
- `insert_artifact_record` returns `Err` on unique constraint violation
- `find_asset_by_id` returns `Option<Asset>`
- `cargo check` passes

### Files affected
- `crates/db/src/lib.rs`
- `crates/db/src/asset_repo.rs` (new)
- `crates/db/src/rights_repo.rs` (new)
- `crates/db/src/artifact_repo.rs` (new)
- `crates/db/src/audit_repo.rs` (new)
- `crates/db/Cargo.toml` (add dubbridge-domain dep)

### Status: [x] DONE — cargo check clean, 5 repo functions implemented.

---

## Task 4 — Storage adapter boundary in crates/storage

**Effort:** M
**Depends on:** Task 1

### Scope
Implement storage abstraction in `crates/storage/src/`:

- `adapter.rs`: `StorageAdapter` trait with `put`, `get`, `delete`, `object_url`
- `local.rs`: `LocalFsAdapter` writing to a configurable local directory
- `config.rs`: `StorageConfig` (bucket, base_path, endpoint_url)
- `lib.rs`: re-export, `build_adapter(config) -> Box<dyn StorageAdapter>`

### Acceptance criteria
- `StorageAdapter` is object-safe
- `LocalFsAdapter::put` writes bytes to `{base_path}/{key}` and returns the stored key
- `LocalFsAdapter::object_url` returns `file://{base_path}/{key}`
- `cargo check` passes

### Files affected
- `crates/storage/src/lib.rs`
- `crates/storage/src/adapter.rs` (new)
- `crates/storage/src/local.rs` (new)
- `crates/storage/src/config.rs` (new)
- `crates/storage/src/error.rs` (new)
- `crates/storage/Cargo.toml` (added tokio/fs, thiserror, async-trait, uuid, serde; tempfile dev-dep)

### Status: [x] DONE — 5/5 unit tests pass, cargo check --workspace clean. 2026-05-31.
Also added `recording_prefix(session_id)` path helper (needed by S3-T4).

---

## Task 5 — Axum ingestion endpoints in apps/api

**Effort:** L
**Depends on:** Task 1, Task 3, Task 4, S0 API Client Authentication Task 2

### Scope
Implement ingestion routes in `apps/api/src/`:

- `state.rs`: `AppState` with `PgPool`, `Box<dyn StorageAdapter>`, auth verifier, `AppConfig`
- `dto/ingestion.rs`: request/response DTOs
- `routes/ingestion.rs`: route handlers
- `main.rs`: wire AppState, add ingestion router

All ingestion mutation endpoints require `assets:ingest`; asset reads require
`assets:read`. `uploader_id` is derived from the S0
`AuthenticatedPrincipal.subject_id`, never accepted from a request body.

### Endpoints
| Method | Path | Description |
|--------|------|-------------|
| POST | `/ingest` | Create ingestion session, receive multipart file |
| POST | `/ingest/{token}/rights` | Submit rights basis |
| POST | `/ingest/{token}/finalize` | Finalize — validates rights, writes asset+rights+artifact+audit |
| GET | `/assets/{id}` | Read asset summary |

### Acceptance criteria
- Finalize without rights returns HTTP 422
- Missing or invalid bearer token returns HTTP 401
- Authenticated caller without the required scope returns HTTP 403
- Finalize derives uploader_id from the authenticated principal
- Ingestion DTOs do not accept a caller-supplied uploader_id
- Duplicate finalize for same token returns HTTP 409
- Successful finalize returns HTTP 201 with asset summary
- GET /assets/:id returns 404 if not found
- `cargo check` passes

### Files affected
- `apps/api/src/main.rs`
- `apps/api/src/state.rs` (new)
- `apps/api/src/dto/ingestion.rs` (new)
- `apps/api/src/routes/ingestion.rs` (new)
- `apps/api/Cargo.toml` (add dubbridge-domain, dubbridge-db, dubbridge-storage, dubbridge-auth, dubbridge-audit, uuid, axum multipart)

### Status: [x] DONE — apps/api ingestion routes wired, cargo check --workspace clean. 2026-05-31.

### Evidence
- Added `AppState` plus in-memory pending-ingestion session tracking in
  `apps/api/src/state.rs`.
- Added ingestion DTOs in `apps/api/src/dto/ingestion.rs`.
- Added protected Axum routes in `apps/api/src/routes/ingestion.rs` for:
  - `POST /ingest`
  - `POST /ingest/{token}/rights`
  - `POST /ingest/{token}/finalize`
  - `GET /assets/{id}`
- Mutation routes require `assets:ingest`; asset reads require `assets:read`.
- Finalize derives `uploader_id` from `AuthenticatedPrincipal.subject_id` and does
  not accept caller-supplied uploader identity.
- Finalize maps rights validation failures to `422`, duplicate finalization to
  `409`, missing asset/session lookups to `404`, and successful finalization to
  `201`.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo check -p dubbridge-api`
  - `~/.cargo/bin/cargo check --workspace`

---

## Task 6 — Tests for S1 acceptance cases

**Effort:** M
**Depends on:** Task 1, Task 2, Task 3, Task 4, Task 5

### Scope
Create deterministic tests in `crates/domain/src/` (unit) and `tests/integration/` (integration):

**Unit tests (no DB)**
- `domain::ingestion::tests::validate_rejects_missing_rights`
- `domain::ingestion::tests::validate_rejects_missing_uploader_context`
- `domain::ingestion::tests::validate_accepts_valid_command`

**Integration tests (require live Postgres)**
- `ingestion_test::successful_ingestion_creates_asset_rights_artifact_and_audit`
- `ingestion_test::missing_rights_is_rejected`
- `ingestion_test::duplicate_finalization_does_not_create_duplicate_artifact`
- `ingestion_test::missing_bearer_token_is_rejected`
- `ingestion_test::uploader_id_is_derived_from_authenticated_principal`

### Acceptance criteria
- Unit tests pass with `cargo test -p dubbridge-domain`
- Integration tests are gated by `#[cfg(feature = "integration")]` or env var `DATABASE_URL`
- All test assertions are meaningful (not just `assert!(true)`)

### Files affected
- `crates/domain/src/ingestion.rs` (add #[cfg(test)] module)
- `tests/integration/ingestion_test.rs` (new)
- `tests/integration/mod.rs` or `tests/integration/Cargo.toml` if needed

### Status: [x] DONE — domain + API acceptance tests passing, cargo check --workspace clean. 2026-05-31.

### Evidence
- Existing domain unit coverage in `crates/domain/src/ingestion.rs` continues to
  prove fail-closed validation for valid input, missing rights, missing uploader
  context, empty owner, and empty proof reference.
- Added package-owned API integration tests in
  `apps/api/tests/ingestion_test.rs` covering:
  - successful ingestion creates asset, rights, artifact, and audit
  - missing rights is rejected
  - duplicate finalization does not create duplicate artifact
  - missing bearer token is rejected
  - uploader identity is derived from the authenticated principal
- Integration tests are gated by `DATABASE_URL`; when the variable is absent they
  return early instead of trying to run DB-backed setup.
- Documented the workspace test-location constraint in `tests/integration/README.md`:
  the workspace root is not a package, so runnable integration suites live under
  the owning app/crate.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo test -p dubbridge-domain`
  - `~/.cargo/bin/cargo test -p dubbridge-api --test ingestion_test`
  - `~/.cargo/bin/cargo check --workspace`

---

## Agent handoff prompt (for delegation)

```
You are implementing S1 of the DubBridge MVP — asset ingestion with rights ledger.

The public repo is at /Users/matiasleandrokruk/Documents/dubbridge.
The plan is at docs/plan/s1-asset-ingestion-rights-ledger.md.
The task list is at docs/tasks/s1-asset-ingestion-rights-ledger.md.

S0 API Client Authentication (docs/tasks/s0-api-client-authentication.md) must be
complete before resuming S1 T5. Work one task at a time in order. After each task:
1. Run `cargo check` in the workspace root.
2. Mark the task [x] in the tasks document.
3. Report the result before moving to the next task.

Key invariants:
- Rights validation must fail closed — no asset reaches a processing state without valid rights.
- Never trust uploader_id from request JSON; derive it from the S0 verified principal.
- The ingest_token column has a UNIQUE constraint — duplicate finalization must return 409.
- Do not copy code from private repos. Clean-room implementation only.
- All communication to the user must be in Spanish.
```
