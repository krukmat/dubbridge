---
type: Plan
title: "Plan: H1 Governance Atomicity + Durable Audit Hardening"
status: closed
---
# Plan: H1 Governance Atomicity + Durable Audit Hardening

**Roadmap position:** Blocking foundation gate after S1 and S3 Task 0, before
expanding the reusable ingestion finalize path through S3 recording ingest.

## Objective

Make the shared ingestion finalization boundary safe to reuse. Metadata writes must
commit atomically, pending-ingestion cleanup must not race finalization into deleting
a referenced blob, and governance audit events must be durably emitted through one
explicit contract rather than best-effort detached tasks.

## Why this gate exists

S3 Task 0 extracted `apps/api/src/ingestion_service.rs::finalize_ingestion_core` so
uploads and future recordings can share one rights-gated finalize path. The extracted
path exposed pre-existing S1 hardening gaps:

- `asset`, `rights_record`, `artifact_record`, finalized status, audit event, and
  pending-session deletion are separate SQL operations without one transaction.
- The extracted finalize core still lives under `apps/api`, but the S3 recording
  bridge runs from `apps/worker-runner`. The reusable service boundary must move to
  a shared app-neutral module.
- Concurrent finalization can lose the `artifact_records.ingest_token` race only
  after earlier rows have already been inserted.
- Cleanup can delete a blob after finalize checks expiration but before finalize
  commits its artifact reference.
- Rights-rejection audit writes are spawned with `tokio::spawn`, so the durable
  audit row is not guaranteed before the request completes or the process exits.
- Duplicate-token handling traces a rejection but does not currently persist the
  durable audit event required by ADR-018.
- ADR-018 calls for one audit-logging boundary to keep durable rows and tracing from
  drifting; the current path emits them manually at call sites.

S3 would multiply these risks because live recording adds lifecycle events and a
second caller of the finalize core. H1 closes the shared boundary first.

## Scope

### Included

- Transaction-aware repository operations for finalization.
- A transport-neutral finalize service that both `apps/api` and
  `apps/worker-runner` can consume without one application depending on the other.
- One atomic relational finalization unit: asset, rights, artifact, status, durable
  audit, and pending-session consumption.
- A locking or claim strategy that serializes finalize with cleanup for the same
  pending ingestion.
- A centralized governance-audit emission contract that persists and traces one
  correlated event without fire-and-forget semantics.
- Persistence-level rights-ledger immutability and strict decoding for
  governance-critical stored enums/statuses.
- Deterministic Postgres-backed tests for rollback and concurrency invariants.

### Excluded

- Cross-store atomic transactions between PostgreSQL and object storage. They are
  impossible across the current boundaries; S2 owns reconciliation for orphaned
  objects after partial cross-store failures (ADR-006).
- MinIO/S3 adapter implementation (S2).
- Recording-specific domain types and lifecycle events (S3).
- General event-stream fan-out or external audit shipping.

## Governing ADRs

- ADR-006: PostgreSQL metadata + object storage binaries; reconcile cross-store
  orphans out of band.
- ADR-008: rights validation fails closed.
- ADR-018: governance events require durable audit rows plus correlated tracing.
- ADR-021: recording must reuse the same ingestion finalize boundary.

## Design constraints

### Atomic relational finalization

The database transaction must include all relational effects of successful finalize.
If any metadata or audit write fails, no partial asset, rights, artifact, status, or
pending-session consumption may remain.

### Cleanup coordination

Finalize and cleanup must coordinate on the pending-ingestion row. The implementation
may use row locking, a claim state, or another PostgreSQL-backed mechanism, but it
must prevent cleanup from deleting a blob that a successful finalize references.

### Persistence-level governance invariants

ADR-008 describes the rights ledger as immutable. Domain validation and repository
conventions are necessary but insufficient if direct SQL can mutate ledger rows.
H1 must add database-backed append-only protection and fail explicitly on unknown
stored governance states rather than silently coercing them to a fallback state.

### Durable audit contract

Governance audit is not a detached side effect. The helper boundary must define:

- durable PostgreSQL persistence behavior
- correlated structured tracing behavior
- transaction participation for success events
- fail-closed behavior when a rejection event itself cannot be persisted

`crates/audit` is the reserved namespace for this boundary if that produces the
cleanest dependency graph. It must not duplicate domain types or repository logic.

## Module dependencies

```text
apps/api ingestion routes -----+
                               +-> shared finalize service
apps/worker-runner bridge -----+
     -> transaction-aware crates/db repositories
     -> governance audit boundary
        -> crates/domain::audit types
        -> crates/db::audit_repo persistence
        -> structured tracing

apps/api cleanup
  -> same pending-ingestion coordination strategy
```

## Execution order

```text
H1 T1 atomic finalize + cleanup coordination  [x] DONE 2026-05-31
  -> H1 T2 persistence-level governance invariants
  -> H1 T3 centralized durable audit boundary
  -> H1 T4 rollback + race regression tests
  -> resume S3 preparatory design task T0c
```

## Related documents

- `docs/tasks/h1-governance-atomicity-hardening.md`
- `docs/plan/roadmap.md`
- `docs/adr/ADR-006-postgres-metadata-object-storage-binaries.md`
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md`
- `docs/adr/ADR-018-structured-observability-traceable-events.md`
- `docs/adr/ADR-021-recording-to-asset-ingestion-bridge-fail-closed.md`
