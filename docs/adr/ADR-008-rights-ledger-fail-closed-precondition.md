---
type: ADR
title: "ADR-008: Rights ledger is a mandatory, fail-closed precondition"
status: Accepted
---

# ADR-008: Rights ledger is a mandatory, fail-closed precondition

- **Status:** Accepted (backfilled from S1 implementation)
- **Date:** 2026-05-31 (reconstructed)
- **Deciders:** DubBridge platform team

## Context

DubBridge only processes **authorized** audiovisual content. Localization
(transcription, translation, dubbing, voice cloning) creates derivative works,
which makes rights provenance a legal precondition, not a nice-to-have. The
platform must be unable to reach a processing state for any asset whose rights
basis is missing or incomplete. The safe failure mode is to **refuse**, never to
proceed by default.

## Decision

- Every asset must carry a valid **rights basis** before it can leave the
  ingestion boundary. The minimum basis is:
  `owner`, `license_type`, `source_type`, `proof_reference`.
- Validation **fails closed**: a missing or empty mandatory field rejects the
  command. Rights are validated **before** uploader context, because rights are
  the primary legal precondition. See
  `crates/domain/src/ingestion.rs::FinalizeIngestionCommand::validate`.
- Upload finalization rejection creates no asset and maps to HTTP `422`. The durable
  pending-ingestion session remains amendable until TTL expiry so a caller can
  attach corrected rights and retry. Any source that persists an aggregate before
  capture or processing (for example S3 recording sessions) records an explicit
  terminal rejected state.
- A persisted **`rights_records`** row is written for every finalized asset; the
  ledger is auditable and immutable.
- `IngestionStatus` deliberately has **no processing-ready variant** until a slice
  introduces one explicitly. Downstream capabilities (probe, transcode, ASR, TTS)
  must add transitions on top of a valid rights basis — never bypass it.

## Consequences

**Positive**
- A hard, testable gate: no rights → no processing, enforced in the domain layer.
- Legal defensibility through an auditable, immutable rights ledger.
- The invariant is reusable: any new ingestion source (e.g. stream recording, see
  ADR-021) must converge on the same gate rather than inventing its own.

**Negative / trade-offs**
- Friction at ingestion: callers must supply complete rights metadata up front.
- New sources cannot "shortcut" ingestion; they must route through the same
  validated finalize path, which constrains their design (intentionally).

## Alternatives considered

- **Fail-open with later reconciliation** — rejected: derivative works could be
  produced from unauthorized content before review, an unacceptable legal risk.
- **Rights checked only at publication** — rejected: processing itself
  (transcription, dubbing) already creates derivatives; the gate must precede
  processing, not publication.

## Related

- ADR-006 (PostgreSQL metadata) — the ledger requires transactional storage.
- ADR-021 (recording-to-asset bridge) — recorded streams reuse this exact gate.
- Implemented by: `crates/domain/src/{ingestion,rights,asset}.rs`,
  `infra/migrations/0002_create_rights_records.sql`.

> Implementation note: H1 adds database-backed append-only enforcement for
> `rights_records`. Repository conventions alone are not sufficient to guarantee an
> immutable ledger against direct SQL mutation.
