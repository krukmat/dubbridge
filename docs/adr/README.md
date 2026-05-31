# Architecture Decision Records (ADR)

This directory holds the Architecture Decision Records that govern the DubBridge
platform. Each ADR captures one significant, hard-to-reverse decision: its
context, the decision itself, the consequences, and the alternatives that were
rejected.

## Format

ADRs follow a lightweight MADR-style structure:

- **Status** — `Proposed`, `Accepted`, `Superseded by ADR-XXX`, or `Deprecated`.
- **Context** — the forces at play and why a decision is required.
- **Decision** — what we decided and the precise scope of that decision.
- **Consequences** — positive, negative, and neutral effects.
- **Alternatives considered** — options rejected and why.

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-006](ADR-006-postgres-metadata-object-storage-binaries.md) | PostgreSQL for metadata, object storage for binary artifacts | Accepted |
| [ADR-008](ADR-008-rights-ledger-fail-closed-precondition.md) | Rights ledger is a mandatory, fail-closed precondition | Accepted |
| [ADR-018](ADR-018-structured-observability-traceable-events.md) | Structured observability; every event must be traceable | Accepted |
| [ADR-019](ADR-019-stream-recording-engine-ffmpeg-subprocess.md) | Stream recording engine: FFmpeg subprocess orchestration | Proposed |
| [ADR-020](ADR-020-recording-session-lifecycle-and-segment-model.md) | Recording session lifecycle and segment model | Proposed |
| [ADR-021](ADR-021-recording-to-asset-ingestion-bridge-fail-closed.md) | Recording-to-asset ingestion bridge with fail-closed rights | Proposed |
| [ADR-022](ADR-022-source-protocol-support-and-ingest-authentication.md) | Source protocol support (RTMP + SRT) and ingest authentication | Proposed |
| [ADR-023](ADR-023-api-client-authentication-and-principal-propagation.md) | API client authentication and principal propagation | Accepted |
| [ADR-024](ADR-024-low-friction-first-party-api-access-via-session-gateway.md) | Low-friction first-party API access via session gateway | Proposed |

## Backfill note

ADR-006, ADR-008, and ADR-018 were referenced by `docs/plan/s1-asset-ingestion-rights-ledger.md`
and by the SQL migrations under `infra/migrations/` before any ADR file existed in
the repository. They have been reconstructed here from the implemented behavior of
the S1 slice so that the references resolve and the decisions are auditable. If the
original intent differs from the implemented behavior, update these files and record
the divergence.

The numbering gaps (ADR-001..005, 007, 009..017) are intentionally left open for
decisions that predate or are unrelated to the slices currently in the repository.
