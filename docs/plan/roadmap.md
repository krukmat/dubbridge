# DubBridge Roadmap (General Plan)

## Purpose

This is the canonical sequencing map for the platform. It records delivered
foundations, blocking hardening gates, product slices, and cross-cutting obligations
derived from `docs/architecture.md` and the ADR set. Individual execution plans live
in `docs/plan/<slice>.md`; this file explains how they fit together.

Last consolidated: 2026-05-31 after the roadmap/ADR/architecture review in
`docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md` (including the
same-day ADR-traceability follow-up G1–G4 in that file). Updated the same day
after H1 completion.

## Status legend
- ✅ Done · 🟡 In progress · ⬜ Not started · 📄 Planned (plan exists, not built)

## Governing principles

- Rust owns API, orchestration, persistence boundaries, governance, and quality
  gates; Python is isolated to ML workers (`docs/architecture.md`).
- PostgreSQL is the system of record for structured metadata; immutable binary
  artifacts live behind `StorageAdapter` with explicit lineage and checksums
  (ADR-006).
- Rights are a mandatory fail-closed precondition for every intake mode and every
  downstream derivative (ADR-008).
- Governance-significant events require durable audit rows plus correlated
  structured tracing (ADR-018).
- API caller identity is verified at the Axum boundary; first-party browser access
  may add a session gateway without weakening the protected API (ADR-023, ADR-024).
- Every non-upload intake is authorized-only and fail-closed before any bytes move:
  - **Platform download (primary S3, ADR-025):** the content owner grants scoped
    access to their own platform account; credentials are stored by reference and
    redacted, and a session lacking valid rights or a valid owner credential is
    rejected before any download.
  - **Live capture (deferred S3b, ADR-022):** an RTMP/SRT source must pass a
    validated stream key or SRT passphrase, redacted from logs, before any bytes are
    captured.
  Both are intake-edge twins of the upload rights gate (ADR-008) and converge on the
  same producer-agnostic finalize boundary (ADR-021).

## Product pipeline

```text
intake -> ingestion + rights gate -> media preparation -> processing (ASR)
       -> subtitles -> translation + dubbing -> human review -> publication
```

Both intake modes converge on the same ingestion and rights boundary:

```text
API client -> S0 authenticated principal
                  |
        +-- direct upload ............... S1 (operational)
intake -+-- platform download ........... S3 (primary, planned: owner-authorized
        |                                  YouTube/Vimeo -> download -> same gate, ADR-025)
        +-- live stream recording ....... S3b (deferred: RTMP/SRT -> recording -> same gate)
```

## Required foundation gates

These are not optional tuning. A downstream slice must not expand a reused path
while its governing invariant remains weaker than the ADR contract.

| Gate | Name | Depends on | Status | Why it blocks |
|------|------|------------|--------|---------------|
| **H1** | Governance atomicity + durable audit hardening | S1, S3 T0 | ✅ done | Closed on 2026-05-31. Finalize now commits relational writes atomically, cleanup coordination is locked against finalize, durable governance audit emission is centralized, and regression coverage locks rollback + concurrency invariants before S3 expands the path. |

Plan: `docs/plan/h1-governance-atomicity-hardening.md`

## Slice sequence

| Slice | Name | Depends on | Status | Plan |
|-------|------|------------|--------|------|
| **S0** | API client authentication + principal propagation | — | ✅ done | `docs/plan/s0-api-client-authentication.md` |
| **S1** | Asset ingestion + rights ledger (upload) | S0 T2 for HTTP endpoints | ✅ done | `docs/plan/s1-asset-ingestion-rights-ledger.md` |
| **T1** | Initial tuning / hardening backlog | S1 | ✅ done | `docs/plan/tuning-hardening.md` |
| **S2** | Object storage switchover (MinIO/S3 behind `StorageAdapter`) | S1 T4 | ⬜ no plan yet | — |
| **S3** | Platform ingest (owner-authorized download: YouTube/Vimeo) | S0 T2, S1, H1; S2 prudent before heavy writes | 🟡 REPLANNED 2026-05-31 — foundation T0/T0c/T1/T2 done; primary path P1–P5 pending | `docs/plan/stream-recording-ingest.md` |
| **S3b** | Stream recording ingest (RTMP/SRT live capture) — deferred sub-case | S3 foundation | ⬜ deferred — built only for live-broadcast clients (ex-T3–T8) | `docs/plan/stream-recording-ingest.md` |
| **S4** | Media preparation (ffprobe metadata + HLS transcode) | S1, S2 | ⬜ no plan yet | — |
| **S5** | Processing / ASR (transcription) | S4 | ⬜ worker contract only | `workers/asr-worker-py` |
| **S6** | Subtitle generation | S5 | ⬜ | — |
| **S7** | Translation + dubbing (TTS / voice cloning) | S6 | ⬜ worker contracts only | `workers/translation-worker-py`, `workers/tts-worker-py` |
| **S8** | Human review (HITL) | S6, S7 | ⬜ | — |
| **S9** | Publication | S8 | ⬜ | — |

## Supporting platform slices

These are real architecture work, but they do not sit on the linear media pipeline.

| Slice | Name | Depends on | Status | Source |
|-------|------|------------|--------|--------|
| **P0** | Local/deployment runtime wiring (compose env, auth bootstrap, service DNS, version policy) | S0, S1 | ⬜ no plan yet | `infra/docker-compose.yml`, `README.md` |
| **P1** | First-party session gateway / BFF | S0, external authorization-server contract | ⬜ no plan yet | ADR-024 |
| **P2** | Production identity hardening (JWKS discovery, automatic key rotation, subject mapping if needed) | S0 | ⬜ no plan yet | ADR-023 |

`P1` must be planned before building a first-party browser or operator-console auth
flow. It does not block S2 or S3.

## Why platform ingest is S3 (and live recording is S3b)

**Replan 2026-05-31 (ADR-025).** The real S3 intake use case is owner-authorized
**platform download**: the content owner provides scoped credentials to their own
platform account (YouTube first, Vimeo/others later) and DubBridge downloads the
owner's content on their behalf. This is the primary S3 path. RTMP/SRT live capture
is demoted to a deferred sub-slice (**S3b**) for the minority of clients who produce
live broadcasts.

Intake (in either mode) widens the funnel and has no dependency on media preparation
or ML stages, so it belongs before S4-S9. Hard dependencies of the **primary
platform-download path**:

- S0 verified principals for Axum ingest endpoints (ADR-023).
- S1's reusable finalize path (`finalize_ingestion_core`) and `StorageAdapter`
  boundary (ADR-006, ADR-021) — reused producer-agnostically.
- A per-connector engine behind `crates/connectors` (`PlatformConnector` trait),
  mirroring the `crates/media` pure-builder / IO-executor boundary (ADR-025).
- Owner-credential handling stored by reference and redacted (ADR-025, ADR-018).
- H1 atomicity and durable-audit hardening before the reused finalize path expands.
- A validated YouTube retrieval mechanism (P2 spike gate) before the connector is
  built — the T0c-style gate for this path.

The **deferred S3b live-recording path** additionally needs the FFmpeg-subprocess
recorder (ADR-019), the segment/lifecycle model and T0c output contract (ADR-020),
and RTMP/SRT capture-edge authentication (ADR-022). Its domain + migration foundation
(T1/T2) is already built and shared with the primary path.

S2 remains a prudent predecessor because intake is the first sustained, high-volume
writer. The trait boundaries make S3 technically possible without S2, but building
retention and upload against the production-like MinIO/S3 adapter avoids rework.

## S3 internal task map (REPLANNED 2026-05-31, ADR-025)

The S3 ledger is `docs/tasks/stream-recording-ingest.md`. The primary intake use
case is owner-authorized **platform download**, not RTMP/SRT live capture. The
FFmpeg recorder (ex-T3–T8) is deferred to **S3b**.

```text
Shared foundation (DONE, reused by both paths):
  T0  reusable S1 finalize core
  T0b duplicate audit type removed (via T1-T5)
  H1  atomicity + durable-audit gate closed
  T0c (S3b only) HLS fMP4 staging + assembled MP4 contract fixed
  T1  domain: recording aggregate, ArtifactKind, audit generalization
  T2  migrations: recording_sessions + audit generalization

PRIMARY S3 — platform ingest (build P1 -> P5):
  P1 connector trait boundary (crates/connectors) + PlatformIngestSession domain
  P2 YouTube retrieval-mechanism spike (gate)
  P3 YouTube connector v1
  P4 PlatformIngestJob + download->bridge wiring + platform_ingest_sessions migration
  P5 API endpoints (/ingests/platform)

S3b — live recorder (DEFERRED): ex-T3 recorder crate, ex-T4 jobs/storage,
  ex-T5 bridge, ex-T6 API, ex-T7 worker, ex-T8 tests. Marked [~] REPLANNED.
```

`T9` (docker-compose Rust pin) is independent low-priority housekeeping.

## Cross-cutting obligations

| Item | Obligation | Owner / next action |
|------|------------|---------------------|
| **X1** | Reconcile `crates/audit` duplicate type | ✅ closed by T1 Task 5; H1 now owns central audit emission semantics |
| **X2** | Align docker-compose Rust pin with toolchain policy | S3 Task 9; fold into P0 when planned |
| **X3** | Backfill remaining open ADR numbers only when real decisions are identified | open |
| **X4** | Persist pending upload sessions across API restarts | ✅ closed by T1 Task 1 |
| **X5** | Add TTL/cleanup for abandoned pending uploads | ✅ closed by T1 Task 2 |
| **X6** | Enforce the 90% coverage gate | ✅ closed by T1 Task 3 |
| **X7** | Prevent partial relational finalization and cleanup-vs-finalize blob loss | ✅ closed by H1 on 2026-05-31 |
| **X8** | Centralize durable audit + tracing emission; do not use fire-and-forget governance audit | ✅ closed by H1 on 2026-05-31 |
| **X9** | Add production object-store adapter, canonical storage-owned key construction, orphan reconciliation, and a streaming/presigned strategy that avoids buffering large uploads in API memory | S2 |
| **X10** | Resolve recording segment/upload/asset cardinality before recorder implementation | ✅ closed by S3 Task 0c on 2026-05-31 |
| **X11** | Enforce consent and voice-cloning permissions before TTS derivatives | S7, then S9 publication gate |
| **X12** | Preserve lineage and quality-gate transitions for every derived artifact | S4-S9 |
| **X13** | Plan first-party browser auth through a session gateway / BFF | P1 |
| **X14** | Plan JWKS rotation and production identity-provider integration | P2 |
| **X15** | Keep RTSP, HLS pull, WebRTC, and per-segment publication as explicit live-recording follow-ups | post-S3b backlog |
| **X16** | Move reusable finalize logic from `apps/api` into an app-neutral shared boundary | ✅ closed by H1 on 2026-05-31 |
| **X17** | Enforce append-only rights rows and strict decoding of stored governance states | ✅ closed by H1 on 2026-05-31 |
| **X18** | Wire container service DNS, database/Redis URLs, auth bootstrap, health checks, and version policy so documented local startup is reproducible | P0 |
| **X19** | Enforce fail-closed source authentication (RTMP stream key / SRT passphrase, credential redaction, `rtmp`/`srt` scheme allow-list) before any capture begins | S3b (domain T1 done, migration T2 done, recorder ex-T3, API ex-T6); ADR-022 |
| **X20** | Decide the secrets-store mechanism for owner-provided platform credentials (storage by reference, scope minimization, redaction); no dedicated ADR yet | S3 P1–P4; ADR-025 |

## Known planning gaps

- **S3 replanned 2026-05-31 (ADR-025).** Primary path is owner-authorized platform
  download; next S3 work is P1 (connector boundary + `PlatformIngestSession` domain).
  RTMP/SRT live recording (ex-T3–T8) is the deferred S3b sub-case.
- The shared foundation (T0/T0b/T0c/H1/T1/T2) is complete and reused by both paths.
  T0c only governs S3b (it fixed the live-recording output contract).
- The YouTube retrieval mechanism for the platform path is not yet fixed; P2 is a
  blocking spike gate before P3, analogous to T0c for recording.
- The owner-credential secrets-store mechanism (X20) has no dedicated ADR yet and
  must be decided during P1–P4.
- S2, S4, P0, P1, and P2 need plan/task ledgers before execution. S2 must include the
  object-store adapter, storage-key ownership, orphan reconciliation, and upload
  memory-safety strategy.
- Slice numbering is provisional. Update this map whenever a slice, dependency, or
  ADR materially changes.
- ADR-021 is generalized to all non-upload intake; ADR-019/020/022 are scoped to the
  deferred S3b live-recording sub-case (their technical decisions are unchanged).
