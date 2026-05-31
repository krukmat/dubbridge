# Plan: S1 Asset Ingestion with Rights Ledger

## Objective

Implement the first MVP vertical slice: accept authorized media uploads, persist the
asset aggregate and rights ledger record, store the original artifact reference, and
emit a structured audit event. Rights validation must fail closed — no processing
state may be reached without explicit, valid rights basis.

## Scope

### Included
- Domain types: Asset, RightsRecord, ArtifactRecord, AuditEvent, IngestionStatus
- SQL migrations for all four tables
- SQLx repositories in `crates/db`
- Storage adapter boundary in `crates/storage` (local/config-driven, MinIO-compatible interface)
- Axum ingestion endpoints in `apps/api`
- Deterministic unit + integration tests

### Excluded (deferred to later slices)
- ffprobe / media metadata extraction
- HLS transcoding
- ASR, subtitle generation, TTS/dubbing
- Signed upload URLs / presigned S3 flows
- Human review and public publishing

## Governing ADRs
- ADR-006: PostgreSQL for metadata, MinIO/S3 for binary artifacts
- ADR-008: Rights ledger is a mandatory precondition — no processing without valid rights
- ADR-018: Structured observability; every event must be traceable
- ADR-023: API callers are authenticated; uploader identity comes from the verified principal

## Affected Files

### crates/domain/src/
- `lib.rs` — re-export modules
- `asset.rs` — Asset, AssetId, IngestionStatus
- `rights.rs` — RightsRecord, RightsBasis, LicenseType, territory/language enums
- `artifact.rs` — ArtifactRecord, ArtifactKind
- `audit.rs` — AuditEvent, AuditEventKind
- `ingestion.rs` — FinalizeIngestionCommand, IngestionError

### crates/db/src/
- `lib.rs` — PgPool setup, re-export
- `asset_repo.rs` — insert_asset, find_asset_by_id
- `rights_repo.rs` — insert_rights_record
- `artifact_repo.rs` — insert_artifact_record, find_original_by_ingest_token
- `audit_repo.rs` — insert_audit_event

### crates/storage/src/
- `lib.rs` — re-export
- `adapter.rs` — StorageAdapter trait + LocalFsAdapter impl
- `config.rs` — StorageConfig (bucket, endpoint, prefix)

### crates/audit/src/
- `lib.rs` — AuditLogger struct wrapping audit_repo writes + tracing spans

### apps/api/src/
- `main.rs` — add ingestion router, inject AppState
- `state.rs` — AppState (pool, storage, config)
- `routes/ingestion.rs` — POST /ingest, POST /ingest/:token/rights, POST /ingest/:token/finalize, GET /assets/:id
- `dto/ingestion.rs` — request/response DTOs

### infra/migrations/
- `0001_create_assets.sql`
- `0002_create_rights_records.sql`
- `0003_create_artifact_records.sql`
- `0004_create_audit_events.sql`

### tests/
- `integration/ingestion_test.rs` — S1 acceptance tests

## Design Decisions

### Rights fail-closed invariant
The `FinalizeIngestionCommand` validator rejects the command if `rights_basis` is
`None` or if any required field inside it is absent. Required fields per ADR-008:
`owner`, `license_type`, `source_type`, `proof_reference`.

`IngestionError` variants for rights validation:
- `MissingRightsBasis` — rights_basis field is None
- `MissingRightsOwner` — owner is None or empty
- `MissingLicenseType` — license_type is None
- `MissingSourceType` — source_type is None
- `MissingProofReference` — proof_reference is None or empty
- `MissingUploaderContext` — uploader_id is None

The API layer maps all these to HTTP 422. Fields deferred to later slices:
allowed_territories, allowed_languages, expiration_date, dubbing_permission,
voice_cloning_permission.

### Authenticated uploader context (S0 prerequisite)
S1 Task 5 depends on S0 API client authentication. Mutable ingestion routes require
the `assets:ingest` scope, and asset reads require `assets:read`. The API derives
`FinalizeIngestionCommand.uploader_id` from
`AuthenticatedPrincipal.subject_id`; request DTOs must not accept a caller-supplied
`uploader_id`.

### IngestionStatus state machine
```
Pending -> Finalized (success path)
Pending -> RejectedMissingRights (validation failure)
```
Processing-ready states (ReadyForProbe, ReadyForTranscode, etc.) are intentionally
absent from this slice. Downstream slices add transitions explicitly.

### Storage adapter boundary
`StorageAdapter` is a trait with `put` / `get` / `delete` / `object_url`. The
`LocalFsAdapter` writes to a temp directory and returns `file://` URLs. The API
accepts binary multipart uploads and delegates to the adapter. This preserves the
MinIO/S3 switch-over boundary for S2.

### Duplicate finalization guard (idempotent)
`artifact_records.ingest_token` has a UNIQUE constraint. A second finalize call
for the same token is state-idempotent: the handler detects the existing artifact
via `find_original_by_ingest_token`, creates no duplicate records, emits the audit
event, and returns 409 Conflict.

### Audit events
Emitted on: ingestion finalization success, rights validation failure, duplicate
token rejection. Written to `audit_events` table and also logged via `tracing::info!`.

## Module Dependencies

```
apps/api → crates/domain, crates/db, crates/storage, crates/auth, crates/audit, crates/config, crates/observability
crates/db → crates/domain, crates/config
crates/storage → crates/config
crates/audit → crates/domain, crates/db
crates/domain → (no internal deps)
```

## Lines Affected After Implementation

Tracked per-task in the tasks document. Updated after each task completes.
