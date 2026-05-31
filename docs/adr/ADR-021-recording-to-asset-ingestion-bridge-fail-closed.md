# ADR-021: Recording-to-asset ingestion bridge with fail-closed rights

- **Status:** Accepted — **generalized to all non-upload intake
  (2026-05-31 replan)**
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

> **2026-05-31 scope note (S3 replan, see ADR-025).** This bridge is **producer-
> agnostic** and is now the shared asset boundary for *every* intake mode that is
> not a direct HTTP upload: owner-authorized **platform downloads** (primary, S3,
> ADR-025) and **live recordings** (S3b, ADR-019/020). In all cases the bridge
> constructs the same finalize command and writes the same governed records through
> `finalize_ingestion_core`; only the artifact kind and the file producer differ
> (`DownloadedPlatformMedia` for platform downloads, `RecordedStreamMedia` for live
> recordings). Read "recording" below as "the completed intake artifact". This ADR
> becomes more central under the replan, not deprecated.

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
  governed core records: `asset`, `rights_record`, artifact lineage, `audit_event`.
  T0c fixes the v1 asset boundary as **one artifact row for one assembled
  whole-session MP4 per recording session**.
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
- **Idempotency** reuses the S1 mechanism: each recording session carries one
  stable bridge `ingest_token`; the `artifact_records.ingest_token` UNIQUE
  constraint makes re-bridging safe.
- **SHA-256** is computed over the finalized assembled MP4 before the artifact row
  is written (ADR-006), preserving tamper-evident lineage.

### Validated v1 artifact-cardinality decision

ADR-020 now fixes segmented capture as an internal staging format, while the asset
boundary is one assembled multimedia artifact:

- local capture writes `init.mp4` + `session.m3u8` + `.m4s` segments
- graceful stop finalizes the manifest
- the worker remuxes the manifest into one assembled MP4
- the assembled MP4 is uploaded and bridged once into the S1 finalize path

The HLS package is therefore **not** the first-class asset in v1, and individual
segments do **not** become assets.

## Consequences

**Positive**
- One ingestion gate, one rights invariant, one audit story — no bypass.
- Recorded assets enter the standard downstream asset pipeline as conventional
  single-file multimedia assets.
- Capturing rights before recording means unauthorized sources never hit disk.

**Negative / trade-offs**
- S3 T0 extracted the S1 finalize logic into a reusable transport-agnostic function
  for the HTTP upload handler and future recording bridge. H1 already moved and
  hardened that path in an app-neutral shared boundary before S3 expansion.
- Assembly is a new step between capture and asset bridge; a crash before clean
  stop leaves only staging files, not a bridged asset.
- V1 intentionally avoids automatic crash-recovery bridge logic. Failed sessions
  remain failed until a future recovery path is designed.
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
- **Manifest-backed session artifact** — rejected for v1: it would push HLS
  segment resolution into downstream consumers before the media-preparation slice
  exists.
- **Per-segment assets** — rejected for v1: it multiplies asset cardinality,
  audits, and idempotency without a clear product need.
- **New `SourceType` for streams** — rejected for v1: `InternalFeed` /
  `LicensedSource` already express the relevant provenance; avoid vocabulary
  sprawl until a real gap appears.

## Related

- ADR-008 (rights ledger, fail-closed) — the gate reused here.
- ADR-006 (metadata + object storage) — artifact rows and checksums.
- ADR-020 (segment model) — defines local segmented staging and the clean-stop
  assembly prerequisite for the bridge.
- Implemented against: `crates/domain/src/ingestion.rs`, S1 finalize path.
