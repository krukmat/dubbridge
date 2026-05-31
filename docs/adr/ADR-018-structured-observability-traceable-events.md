# ADR-018: Structured observability; every governance event must be traceable

- **Status:** Accepted (backfilled from S1 implementation)
- **Date:** 2026-05-31 (reconstructed)
- **Deciders:** DubBridge platform team

## Context

DubBridge makes governance decisions (accepting media, rejecting for missing
rights, deduplicating ingestion). Because these decisions carry legal weight,
they must be **traceable after the fact**: who, what, when, and why. Ephemeral
logs alone are insufficient for audit; conversely, persisting every log line is
wasteful. The platform needs both a durable audit trail for governance events and
structured, correlated tracing for operational diagnosis.

## Decision

- **Governance-significant events** are persisted as rows in `audit_events`
  (event kind, optional `asset_id`, `ingest_token`, detail, timestamp). The
  `asset_id` is nullable because rejection events may occur before an asset is
  persisted.
- The same events are emitted through **structured tracing** (`tracing` /
  `tracing-subscriber`), initialized centrally by `crates/observability`.
- Audit writes and trace spans share correlation identifiers (e.g. `ingest_token`,
  later `recording_session_id`) so a durable audit row can be tied back to its
  operational trace.
- Audit emission is part of the operation's success path: finalize success, rights
  rejection, and duplicate-token handling all emit an audit event.

## Consequences

**Positive**
- Durable, queryable audit trail for legal/governance review, decoupled from log
  retention.
- Operational traces correlate to audit rows via shared identifiers.
- A single place (`crates/observability`) owns tracing setup, keeping services
  consistent.

**Negative / trade-offs**
- Two emission paths (DB row + trace) for the same event; they must not drift.
  Mitigation: emit both from one audit-logging helper.
- Defining "governance-significant" is a judgment call; over-auditing adds write
  load, under-auditing creates gaps. New subsystems must declare their auditable
  events explicitly (e.g. recording lifecycle events in ADR-020).

## Alternatives considered

- **Logs only** — rejected: not durable or queryable enough for legal audit.
- **Audit rows only, no tracing** — rejected: poor operational diagnosis and no
  correlation across services.

## Related

- ADR-006 (PostgreSQL metadata) — audit rows are transactional metadata.
- ADR-008 (rights ledger) — rights rejections are auditable events.
- ADR-020 (recording lifecycle) — defines recording-specific audit events.
- Implemented by: domain type `crates/domain/src/audit.rs`, persistence
  `crates/db/src/audit_repo.rs`, tracing setup `crates/observability`, and
  `infra/migrations/0004_create_audit_events.sql`.

> Note: `crates/audit` is reserved for an `AuditLogger` that would wrap
> `db::audit_repo` writes and tracing spans, but it is currently a stub holding an
> unrelated `AuditEvent` placeholder. Reconciling it is a tracked follow-up
> (see `docs/audit/2026-05-31-project-consistency-review.md`, F1/F8).
