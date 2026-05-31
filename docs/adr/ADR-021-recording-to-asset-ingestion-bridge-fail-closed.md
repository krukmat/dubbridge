# ADR-021: Recording-to-asset ingestion bridge with fail-closed rights

- **Status:** Proposed
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

## Context

A recorded stream produces one or more media files (ADR-019, ADR-020). For the
platform to *incorporate* a recording, that output must become a first-class
**asset** with the same guarantees as an uploaded file: a valid rights basis
(ADR-008), explicit lineage and checksum (ADR-006), and audit events (ADR-018).

The risk to avoid is a **second, weaker ingestion path**. If recordings bypassed
the S1 rights gate, the platform could derive localized works from an
unauthorized live source — exactly what ADR-008 forbids.

## Decision

- A completed recording is **bridged into the existing S1 ingestion finalize
  path**, not given a parallel one. The bridge constructs the same domain command
  that an upload would (`FinalizeIngestionCommand` semantics) and writes the same
  four records: `asset`, `rights_record`, `artifact_record`, `audit_event`.
- **Rights are captured up front, at session creation**, and validated before
  capture starts (ADR-020 `Requested → RightsValidated`). A session whose rights
  basis is invalid is **rejected before any bytes are recorded** — the strongest
  possible fail-closed posture. The captured `RightsBasis` is carried through to
  the bridge; the recorded asset cannot exist without it.
- The recorded artifact is tagged with a **new `ArtifactKind::RecordedStreamMedia`**
  variant so lineage explicitly distinguishes recorded captures from direct
  uploads (`OriginalMedia`). `artifact_records.kind` is free-form `TEXT` today, so
  this is additive and non-breaking at the schema level.
- The recording's `source_type` maps to an existing rights `SourceType`
  (`InternalFeed` for live contribution, or `LicensedSource` when licensed),
  reusing the S1 rights vocabulary rather than extending it.
- **Idempotency** reuses the S1 mechanism: each bridged segment/recording carries a
  stable `ingest_token`; the `artifact_records.ingest_token` UNIQUE constraint makes
  re-bridging a completed segment safe (no duplicate artifacts).
- A **SHA-256 checksum** is computed over the finalized file before the artifact
  row is written (ADR-006), preserving the tamper-evident lineage guarantee.

## Consequences

**Positive**
- One ingestion gate, one rights invariant, one audit story — no bypass.
- Recorded assets are indistinguishable from uploads downstream (probe, ASR, TTS),
  so the rest of the pipeline needs no special-casing.
- Capturing rights before recording means unauthorized sources never hit disk.

**Negative / trade-offs**
- The S1 finalize logic must be refactored into a reusable, transport-agnostic
  function so both the HTTP upload handler and the recording bridge call it. This
  is a prerequisite (it depends on S1 tasks T4–T6 being completed or that finalize
  path being extracted).
- Per-segment bridging vs. whole-session bridging is a design choice (see plan):
  per-segment enables incremental availability but creates multiple artifacts per
  session; whole-session is simpler but delays availability.
- Introducing `ArtifactKind::RecordedStreamMedia` requires generalizing
  `db::artifact_repo::find_original_by_ingest_token`, which currently hardcodes
  `OriginalMedia` (a dead `if/else`). Without the fix, recorded artifacts read back
  as uploads, corrupting lineage. Tracked as F3 in the consistency review and in
  plan tasks T1/T5.

## Alternatives considered

- **A separate recording ingestion path** — rejected: duplicates governance and
  risks divergence from the fail-closed invariant (ADR-008).
- **Capture first, attach rights later** — rejected: allows unauthorized content
  to be recorded and stored before any rights check; violates fail-closed.
- **New `SourceType` for streams** — rejected for v1: `InternalFeed` /
  `LicensedSource` already express the relevant provenance; avoid vocabulary
  sprawl until a real gap appears.

## Related

- ADR-008 (rights ledger, fail-closed) — the gate reused here.
- ADR-006 (metadata + object storage) — artifact rows and checksums.
- ADR-020 (segment model) — the segment-complete event triggers the bridge.
- Implemented against: `crates/domain/src/ingestion.rs`, S1 finalize path.
