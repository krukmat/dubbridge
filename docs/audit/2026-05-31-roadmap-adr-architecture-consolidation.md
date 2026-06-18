---
type: Audit
title: "Roadmap / ADR / Architecture Consolidation Review — 2026-05-31"
date: 2026-05-31
---

# Roadmap / ADR / Architecture Consolidation Review — 2026-05-31

## Scope

Review `docs/plan/roadmap.md` against `docs/architecture.md`, ADR-006, ADR-008,
ADR-018 through ADR-024, current plan/task ledgers, migrations, and the implemented
S1 finalize path before continuing S3.

## Findings

| ID | Severity | Finding | Consolidated action |
|----|----------|---------|---------------------|
| C1 | Medium | Roadmap status drift: S1 and T1 were complete, but the roadmap still said to resume S1 T5. | Roadmap corrected. |
| C2 | High | S3 migration plan reused `0005` and `0006`, already consumed by pending-ingestion hardening. | S3 must allocate the next free sequence after H1 migrations. |
| C3 | Medium | S3 Task 0b still treated `crates/audit` reconciliation as open after T1-T5 removed the duplicate type. | S3 T0b marked complete; central audit semantics moved to H1. |
| C4 | High | ADR-020/proposal described per-segment upload + bridge while the S3 plan selected one whole-session bridge without defining the materialized output contract. | Added blocking S3 T0c spike/decision gate; ADR-020 and ADR-021 now expose the unresolved boundary. |
| C5 | High | Shared finalize writes relational records sequentially without one SQL transaction. Concurrent or failed finalization can leave partial rows. | Added blocking H1 plan/tasks before S3 expansion. |
| C6 | High | Cleanup can delete a blob after finalize passes expiry validation but before artifact commit. | H1 owns finalize/cleanup coordination. |
| C7 | High | Missing-rights rejection audit uses detached `tokio::spawn`; ADR-018 requires durable audit emission in the success path of governance decisions. | H1 owns a centralized, awaited audit-emission boundary. |
| C8 | Medium | ADR-024 introduced a first-party session gateway/BFF but the roadmap and architecture map omitted it. | Added supporting slice P1 and architecture boundary. |
| C9 | Medium | ADR follow-ups were scattered: object-store orphan reconciliation, JWKS rotation, consent gates, derivative lineage, and deferred recording protocols had no canonical owners. | Roadmap cross-cutting obligations now assign owners. |
| C10 | Medium | `docs/architecture.md` mixed intended and implemented surfaces without delivery status. | Architecture overview expanded with explicit current/planned boundaries. |
| C11 | High | S3 T0 extracted finalize into `apps/api`, but the future recording bridge executes from `apps/worker-runner`; application-to-application reuse is the wrong boundary. | H1 moves finalize into an app-neutral shared service. |
| C12 | Medium | ADR-008 described terminal rejected assets, while upload rejection actually creates no asset and leaves a pending session amendable until TTL. | ADR-008 and S1 plan aligned with retryable pending-upload behavior. |
| C13 | Medium | ADR-006 assigns key layout to `crates/storage`, but S1 upload keys are built in the Axum route and file bytes are buffered in memory before `put`. | S2 must centralize keys and select streaming/presigned production uploads. |
| C14 | Low | Backfilled ADR-006 said SQLx queries were compile-time checked, but current repositories use runtime SQLx query APIs. | ADR-006 corrected to the implemented SQLx boundary. |
| C15 | Medium | Documented `docker compose up -d` included app containers without internal service URLs or API auth bootstrap, so protected API startup is not reproducible. | README now starts infrastructure only; added supporting slice P0 for full runtime wiring. |
| C16 | Medium | `.githooks/pre-push` enforced a different coverage scope than CI, despite the workflow requiring mirrored QA gates. | Pre-push hook aligned with CI `--ignore-filename-regex`. |

## Current implementation facts

- S0 API authentication is complete.
- S1 upload ingestion is complete.
- T1 initial hardening is complete.
- S3 Task 0 extracted `apps/api/src/ingestion_service.rs::finalize_ingestion_core`.
- S3 Task 0b's duplicate `AuditEvent` concern is closed by T1-T5.
- Migrations `0001` through `0006` are already allocated.
- Recording implementation beyond S3 Task 0 has not started.

## Required order before continuing S3

```text
H1 governance atomicity + durable audit hardening
  -> S3 T0c recording output-contract spike and ADR decision
  -> S3 T1+ implementation tasks
```

## Documents updated by this consolidation

- `docs/plan/roadmap.md`
- `docs/architecture.md`
- `docs/plan/h1-governance-atomicity-hardening.md`
- `docs/tasks/h1-governance-atomicity-hardening.md`
- `docs/plan/stream-recording-ingest.md`
- `docs/tasks/stream-recording-ingest.md`
- `docs/adr/ADR-018-structured-observability-traceable-events.md`
- `docs/adr/ADR-019-stream-recording-engine-ffmpeg-subprocess.md`
- `docs/adr/ADR-020-recording-session-lifecycle-and-segment-model.md`
- `docs/adr/ADR-021-recording-to-asset-ingestion-bridge-fail-closed.md`
- selected historical/status notes that referenced superseded state

## Follow-up: ADR traceability gaps (G1–G4) — same-day second pass

A second pass focused specifically on ADR ↔ roadmap ↔ architecture traceability and
execution-order dependencies surfaced four residual gaps not closed by C1–C16. All
four are documentation-traceability issues; none changes implemented behavior.

| ID | Severity | Gap | Correction |
|----|----------|-----|------------|
| G1 | Medium-low | `roadmap.md` never cited ADR-019 (FFmpeg engine) or ADR-022 (RTMP/SRT + source auth), though slice S3 governs both. | Added both ADRs to "Why recording remains S3" as explicit S3 hard dependencies. |
| G2 | Medium | The authorized-only posture was a governing principle only for upload rights (ADR-008); its capture-edge twin (fail-closed source authentication, ADR-022) lived only inside the S3 plan, invisible to the roadmap. | Added a governing principle for authorized-only live capture and a cross-cutting obligation X19 owned by S3. |
| G3 | Low | `architecture.md` referenced ADR-020 only inside the collective range `ADR-019..022`; the lifecycle/segment model was not individually traceable (a grep for `ADR-020` returned 0 hits). | Expanded the delivery-status range to `ADR-019/020/021/022` and individualized ADR-020 (lifecycle) and ADR-022 (source auth) in the Planned runtime surfaces. |
| G4 | Low | Neither map reflected that ADR-019/020/021/022 are `Proposed`, with the v1 output contract still open pending S3 Task 0c. | Added a `Proposed`/T0c provisionality note to the roadmap planning gaps and the architecture recorder surface. |

### Impact check on implemented work

The four corrections are confined to the recording domain (slice S3), which is not
implemented beyond Task 0 (finalize extraction) and Task 0b (audit reconciliation).
Verified against the working tree on 2026-05-31:

- `crates/recorder` does not exist; neither do `crates/domain/src/recording.rs`,
  `apps/api/src/routes/recording.rs`, or the worker recording handler.
- `ArtifactKind` still has only `OriginalMedia`; `RecordedStreamMedia` appears only
  in forward-looking comments in `apps/api/src/ingestion_service.rs`.
- No source-authentication code (stream key / SRT passphrase) exists yet.

Therefore no implemented code and no task already marked `[x]` (S0 T1–T2, S1 T1–T6,
T1 T1–T6, S3 T0/T0b) is affected. The S3 plan and task ledger already cite
ADR-019/020/022 in full, so no done-task content required correction. The gaps were
isolated to `roadmap.md` and `architecture.md`.

### Documents updated by this follow-up

- `docs/plan/roadmap.md` (governing principles, S3 dependencies, X19, planning gaps,
  consolidation pointer)
- `docs/architecture.md` (delivery-status ADR refs, Planned recorder surface)
- `docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md` (this section)
