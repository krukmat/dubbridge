---
type: ADR
title: "ADR-006: PostgreSQL for metadata, object storage for binary artifacts"
status: Accepted
---

# ADR-006: PostgreSQL for metadata, object storage for binary artifacts

- **Status:** Accepted (backfilled from S1 implementation)
- **Date:** 2026-05-31 (reconstructed)
- **Deciders:** DubBridge platform team

## Context

DubBridge processes authorized audiovisual media. Two classes of state exist:

1. **Structured metadata** — assets, rights records, artifact references, audit
   events. These require transactional integrity, relational constraints
   (foreign keys, UNIQUE, CHECK), and queryability.
2. **Binary media artifacts** — original uploads and, later, renditions,
   transcripts, and recorded segments. These are large, immutable, and must be
   addressable by a stable key.

Storing large binaries inside the relational database would bloat it, complicate
backups, and couple binary lifecycle to row lifecycle. Conversely, storing
metadata in object storage would forfeit transactional guarantees and the
fail-closed invariants the platform depends on.

## Decision

- **PostgreSQL** is the system of record for all structured metadata. Repositories
  in `crates/db` use SQLx queries and typed domain mappings from `crates/domain`.
- **Object storage** (MinIO locally, S3-compatible in production) holds all binary
  artifacts. Access is mediated by the `StorageAdapter` boundary in
  `crates/storage`, never by direct client coupling.
- Binary artifacts are referenced from PostgreSQL by a **`storage_key`** plus a
  **SHA-256 `checksum`**. The database row is the authority for existence and
  lineage; the object store holds the bytes.
- Path conventions are owned by `crates/storage` (e.g. `assets/{asset_id}/...`),
  keeping key layout in one place.

## Consequences

**Positive**
- Transactional integrity and relational invariants for governance-critical state.
- Cheap, scalable, immutable binary storage decoupled from the database.
- Clean local/production parity via the MinIO/S3 switch behind `StorageAdapter`.
- Checksums make artifacts verifiable and tamper-evident, supporting lineage.

**Negative / trade-offs**
- Two stores to keep consistent: an orphaned object or a dangling row is possible
  if writes are not ordered carefully. Mitigation: write the object first, then
  the metadata row; reconcile orphans out of band.
- Cross-store transactions are not atomic; idempotency keys (e.g. `ingest_token`)
  are required to make retries safe.

## Alternatives considered

- **Binaries as `bytea`/large objects in PostgreSQL** — rejected: database bloat,
  backup cost, and lifecycle coupling.
- **Object storage only, metadata as JSON sidecars** — rejected: loses
  transactional integrity and the fail-closed rights invariant (see ADR-008).

## Related

- ADR-008 (rights ledger precondition) — depends on transactional metadata.
- ADR-018 (observability) — audit events live in PostgreSQL.
- Implemented by: `infra/migrations/0001..0004`, `crates/db`, `crates/storage`.

> Implementation note: S1 still constructs upload keys in
> `apps/api/src/routes/ingestion.rs` and passes buffered bytes to
> `StorageAdapter::put`. S2 must move upload-key construction behind
> `crates/storage`, implement MinIO/S3 behavior, add orphan reconciliation, and
> choose a streaming or presigned upload strategy for production-scale files.
