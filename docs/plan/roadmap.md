# DubBridge Roadmap (General Plan)

## Purpose

This is the sequencing plan for the whole platform. It places every slice in
dependency order, records status, and explains **why** each slice sits where it
does. Individual slice plans live in `docs/plan/<slice>.md`; this file is the map
that connects them. Created 2026-05-31 to close the "no general plan" gap surfaced
in `docs/audit/2026-05-31-project-consistency-review.md`.

## Status legend
- ✅ Done · 🟡 In progress · ⬜ Not started · 📄 Planned (plan doc exists, not built)

## Guiding principles (from `docs/architecture.md` + ADRs)
- Rust owns API, orchestration, governance, and quality gates; Python only for ML
  workers.
- Media artifacts are immutable with explicit lineage (ADR-006).
- Rights are a fail-closed precondition — nothing reaches processing without a valid
  basis (ADR-008).
- Every governance event is auditable (ADR-018).

## The product pipeline (capability target)

```
intake ─► ingestion + rights gate ─► media preparation ─► processing(ASR)
       ─► subtitles ─► dubbing ─► human review ─► publication
```

`intake` has **two modalities** that both converge on the same ingestion rights
gate:

```
API client ─► S0 authenticated principal
                  │
        ┌─ direct upload ............ S1
intake ─┤
        └─ live stream recording .... S3  (RTMP/SRT → file → same rights gate)
```

## Slice sequence

| Slice | Name | Depends on | Status | Plan |
|-------|------|-----------|--------|------|
| **S0** | API client authentication + principal propagation | — | ✅ done | `docs/plan/s0-api-client-authentication.md` |
| **S1** | Asset ingestion + rights ledger (upload) | S0 T2 for HTTP endpoints | ✅ done | `docs/plan/s1-asset-ingestion-rights-ledger.md` |
| **S2** | Object storage switchover (MinIO/S3 behind `StorageAdapter`) | S1 T4 | ⬜ no plan yet | — |
| **S3** | **Stream recording ingest (RTMP/SRT)** | S0 T2, S1 (finalize + `StorageAdapter`), S2 (prudent) | 📄 planned | `docs/plan/stream-recording-ingest.md` |
| **S4** | Media preparation (ffprobe metadata + HLS transcode) | S1 | ⬜ no plan yet | — |
| **S5** | Processing / ASR (transcription) | S4 | ⬜ worker contract only | `workers/asr-worker-py` |
| **S6** | Subtitle generation | S5 | ⬜ | — |
| **S7** | Dubbing (TTS + voice cloning) | S6 | ⬜ worker contracts only | `workers/tts-worker-py`, `workers/translation-worker-py` |
| **S8** | Human review (HITL) | S6/S7 | ⬜ | — |
| **S9** | Publication | S8 | ⬜ | — |
| **T1** | Tuning / hardening backlog | Active slices as needed | ✅ done | `docs/plan/tuning-hardening.md` |

## Where stream recording sits — and why (the evaluated decision)

**Stream recording is placed at S3: the third slice, immediately after the
foundations it depends on, and ahead of every downstream ML stage.**

Dependency analysis:

- **Hard dependencies (must precede S3):**
  - **S0 API client authentication** — Axum recording endpoints reuse the same
    verified-principal boundary as S1; RTMP/SRT source credentials remain a
    separate ADR-022 layer.
  - **S1 finalize path** — the recording→asset bridge reuses it (ADR-021); this is
    S3's Task T0. S1 must be complete (T4–T6) or its finalize logic extracted.
  - **`StorageAdapter` trait (S1 T4)** — recording writes segments through the same
    storage boundary (`recordings/` prefix).
  - **Rights gate (ADR-008)** — recording converges on it; rights are validated
    before capture starts.
- **Prudent dependency (should precede S3): S2 storage switchover.** Recording is
  the platform's **first heavy, continuous writer to object storage** (segmented
  fMP4 + retention/upload, ADR-020). Doing the small MinIO/S3 switchover first means
  recording's upload/retention logic targets the real backend instead of being
  built against the local adapter and reworked later.
- **No dependency on downstream stages.** Recording is independent of media
  preparation, ASR, subtitles, dubbing, review, and publication — it only widens
  intake. Therefore it can, and should, come **before** S4–S9. This is what makes
  "as early as possible" technically valid.

**Why not even earlier (S2)?** The only *hard* storage dependency is the
`StorageAdapter` trait (delivered in S1 T4), so recording could technically be S2.
We still put the storage switchover first because recording is the heaviest storage
consumer and we want to avoid building its retention/upload path twice. If the team
later prioritizes shipping recording fastest over that rework, S3 and S2 can swap —
the trait boundary makes this safe.

**Why not later (after S4+)?** There is a product argument to prove the linear
localization pipeline end-to-end (upload → … → publication) before adding a second
intake modality. That is a prioritization choice, not a technical constraint. The
user's directive here is "the sooner the better, driven by technical dependencies",
so recording is placed as early as the dependencies allow (S3). If product priority
shifts to proving end-to-end localization first, recording moves after S4/S5 without
any technical rework.

## S3 internal task map

The S3 slice is fully planned in `docs/tasks/stream-recording-ingest.md`:
`T0` (S1 finalize extraction, prerequisite) → `T0b` (reconcile `crates/audit`) →
`T1`–`T8` (domain, migrations, recorder engine, jobs/storage, bridge, API, worker,
tests). `T9` (docker-compose Rust pin) is low-priority housekeeping.

## S0 foundation task map

The S0 slice is planned in `docs/tasks/s0-api-client-authentication.md`:
`T1` (JWT verifier + typed principal) → `T2` (Axum bearer middleware + scope
authorization). S0 is complete; resume S1 Task 5 next.

## Cross-cutting / transversal items (not slices)

| Item | What | Tracked in |
|------|------|-----------|
| X1 | Reconcile/remove `crates/audit` (F1/F8) | S3 Task 0b |
| X2 | Align docker-compose Rust pin (F7) | S3 Task 9 |
| X3 | Backfill remaining open ADR numbers when decisions are identified | open |
| X4 | Persist pending upload-ingestion sessions across API restarts; remove in-memory loss risk from S1 T5 | closed by T1 Task 1 |
| X5 | Add TTL/cleanup for abandoned pending ingestions | T1 Task 2 |
| X6 | Close the gap to the enforced 90% coverage gate | T1 Task 3 |
| X7 | Add ingestion concurrency/race-condition hardening | T1 Task 4 |
| X8 | Reconcile `crates/audit` with intended architecture | T1 Task 5 and S3 Task 0b |
| X9 | Add upload operational safeguards (size/abuse/resource limits) | T1 Task 6 |

## Known gaps in this roadmap (to resolve as slices are picked up)

- **S2 (storage switchover)** and **S4 (media preparation)** have no plan/tasks docs
  yet; create them before starting those slices (per the workflow).
- Slice numbering beyond S3 is provisional; reorder if priorities change. Update this
  table whenever a slice's status or position changes — it is the canonical map.
