---
type: Plan
title: "Plan: S-120 — Media Preparation"
status: planned
slice: S-120
---
# Plan: S-120 — Media Preparation

> **Status:** Planned. Authored 2026-06-18 after roadmap review and S-080 unblock analysis.
> **Roadmap phase:** `S-120` — media preparation (`ffprobe` metadata + HLS transcode).
> **Tasks ledger:** `docs/tasks/s-120-media-preparation.md`.

## Purpose

DubBridge can already ingest authorized media, persist immutable binaries through
`StorageAdapter`, and expose those source assets to the product surface. What it
still lacks is the first downstream preparation layer that turns an uploaded or
downloaded source file into processing-ready artifacts. `S-120` closes that gap:
probe the source, persist durable media metadata, produce a normalized HLS package,
and record the lineage needed for later ASR, subtitle, dubbing, review, and
publication slices.

Without this slice, every downstream phase would need to rediscover basic media
facts or depend directly on the original file layout. `S-120` establishes the
shared prepared-media contract that `S-130`–`S-180` consume.

## Objective

Deliver a fail-closed media-preparation stage that:

- probes the original media artifact with `ffprobe` and persists stable metadata;
- transcodes the source into a canonical HLS package suitable for downstream
  playback-oriented and processing-oriented consumers;
- stores every derived file behind `StorageAdapter` with canonical storage-owned keys;
- preserves artifact lineage, checksums, and readiness gates for downstream work; and
- keeps probing/transcoding orchestration in Rust while isolating FFmpeg/ffprobe
  process execution behind explicit boundaries.

## Scope

### Included

- A preparation contract rooted in the existing `artifact_records` / storage
  model, extended for derived media artifacts and their lineage.
- `ffprobe` metadata extraction for the source artifact.
- HLS transcode output (playlist(s) + media segments) as the canonical prepared
  representation for v1.
- Background-job orchestration and status transitions needed to run preparation
  asynchronously after ingestion/finalize.
- Observability, audit, and fail-closed status/reporting for preparation success
  and failure.
- API/product-facing readiness signals that let downstream slices distinguish
  "source ingested" from "prepared and ready".

### Excluded

- ASR/transcription (`S-130`), subtitles (`S-140`), translation/dubbing (`S-150`),
  and human review/publication runtime (`S-170`/`S-180`).
- Client-facing streaming delivery, CDN publication, signed playback URLs, or
  consumer playback UX. Those are owned by `S-125` HLS playback delivery and later
  publication/player decisions (ADR-032).
- Thumbnail generation, waveform extraction, loudness normalization variants, or
  alternate rendition ladders beyond the v1 HLS package.
- ML worker changes; this slice prepares the contract they will later consume.

## Governing constraints

- ADR-006: PostgreSQL remains authoritative for metadata; binaries remain immutable
  object-store artifacts referenced by key and checksum.
- ADR-018: preparation failures and transitions must produce durable traceable
  observability, not best-effort logs only.
- ADR-021: every intake path converges on one finalize boundary; preparation starts
  after that shared ingestion contract, not as a parallel ingest path.
- ADR-026: runtime configuration stays environment-explicit and fail-closed.
- Roadmap X12: derived artifacts must preserve lineage and quality-gate transitions.

## Affected components

| Layer | Path | Change |
|---|---|---|
| Domain | `crates/domain/src/artifact.rs` | add preparation artifact kinds + lineage contract |
| Domain | `crates/domain/src/asset.rs` or new preparation module | represent preparation status/readiness |
| Persistence | `infra/migrations/*` | extend artifact/preparation schema for derived outputs and lineage |
| Persistence | `crates/db/src/artifact_repo.rs` | query/write preparation artifacts and readiness state |
| Media boundary | `crates/media/src/lib.rs` | keep pure command builders / metadata parsers |
| Orchestration | `crates/jobs/src/lib.rs` | define preparation job envelope(s) |
| Worker runtime | `apps/worker-runner/src/main.rs` | execute preparation jobs |
| API integration | `apps/api/src/ingestion_service.rs`, related finalize hooks | enqueue preparation after successful finalize |
| Storage | `crates/storage/src/lib.rs`, adapters | canonical derived-artifact key helpers |
| Docs / product contract | `docs/architecture.md`, roadmap/task docs | record prepared-media boundary and readiness contract |

## Design decisions

### D1 — Preparation is rooted in the original artifact, not a replacement upload

The original artifact remains immutable and authoritative as the source object.
Preparation creates additional artifact rows and readiness state derived from that
source; it never overwrites or mutates the original upload/download artifact.

### D2 — `ffprobe` and `ffmpeg` stay behind pure-builder / IO-executor seams

`crates/media` continues the pattern already used elsewhere in the repo: pure Rust
builders/parsers define the command and output contract, while the runtime surface
that actually spawns subprocesses lives in an executor/orchestrator layer. This
keeps media command construction unit-testable and avoids leaking process-management
details into API handlers or domain code.

### D3 — HLS is the canonical prepared representation for v1

V1 preparation emits a deterministic HLS package. Later slices may add other derived
representations, but `S-120` needs one stable, storage-backed output that downstream
phases can reference without re-probing the original binary or assuming local files.

### D4 — Derived-artifact lineage must be queryable and fail-closed

The current `artifact_records` contract is source-oriented and keyed to one ingest
token. `S-120` therefore needs an explicit way to represent multiple derived
artifacts per asset plus their parentage. Unknown artifact kinds or incomplete
lineage must fail closed rather than silently collapsing into `original_media`.

### D5 — Preparation is asynchronous and readiness-gated

Finalize should remain focused on authorizing and committing source ingestion.
Preparation runs asynchronously after finalize succeeds and sets a durable readiness
signal. Downstream slices must be able to reject or defer work cleanly while an asset
is still unprepared or preparation failed.

### D6 — Storage keys for prepared media are owned by `crates/storage`

The same key-ownership rule adopted in `S-080` applies here: API/worker code must
not hand-roll transcode keys. `crates/storage` owns the canonical key layout for
probe outputs, HLS manifests, and HLS segments so every producer/consumer sees one
authoritative scheme.

## Task decomposition strategy

This slice is decomposed before implementation so media probing, transcode output,
schema/lineage changes, and orchestration can be reviewed independently:

1. `T0` — create the slice plan/task ledger and sync roadmap state.
2. `T1` — author BDD coverage and the preparation artifact/readiness contract.
3. `T2` — implement schema/domain/repo changes for preparation lineage + status.
4. `T3` — implement `ffprobe` metadata extraction and persistence.
5. `T4` — implement HLS transcode output and storage persistence.
6. `T5` — wire async orchestration, readiness gating, observability, and docs sync.

No implementation should start outside those bounded tasks.

## Relationship to adjacent slices

- **Built on:** `S-010` ingestion/finalize, `S-020` finalize hardening, `S-080`
  production-like storage, and the current product-layer asset surfaces.
- **Unblocks directly:** `S-130` ASR, which should consume preparation metadata and
  prepared media instead of rediscovering source state ad hoc.
- **Feeds playback:** `S-125` HLS playback delivery, which exposes prepared
  manifests and segments through backend-owned authorization grants instead of raw
  object-store keys (ADR-032).
- **Feeds later:** `S-140`, `S-150`, `S-170`, and `S-180` through stable derived
  artifacts and readiness state.
- **Does not solve:** owner-credential secret storage (`X20`), which remains in `S-090`.

## Open follow-ups

- Decide whether the preparation readiness signal lives on `assets`, a new
  preparation-run table, or both; the implementation tasks must choose one durable
  contract and keep downstream reads simple.
- Decide whether v1 HLS emits one rendition or a small ladder; the plan defaults to
  one canonical package unless task-level analysis justifies more.
- Decide whether source metadata is stored as structured columns, JSON payload, or a
  hybrid model; task `T2` must make that persistence contract explicit.
