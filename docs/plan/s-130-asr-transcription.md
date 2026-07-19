---
type: Plan
title: "Plan: S-130 — ASR Transcription"
status: closed
slice: S-130
---
# Plan: S-130 — ASR Transcription

> **Status:** Done 2026-07-19. Authored 2026-06-25. T1–T5 are complete; T1/T2 owner
> sign-off was recorded 2026-07-19 against the already-recorded implementation
> evidence, and T5 synchronized the canonical BDD + roadmap/task/plan status
> artifacts under the repository's pre-commit roadmap drift guard.
> **Roadmap phase:** `S-130` — Processing / ASR (transcription).
> **Tasks ledger:** `docs/tasks/s-130-asr-transcription.md`.

## Purpose

S-120 delivers a prepared HLS package and a `Ready` readiness gate per asset. The
next processing stage — transcribing the audio track into a timed transcript and
word-level alignment — has a defined worker contract
(`workers/asr-worker-py/input.schema.json`, `output.schema.json`,
`error.schema.json`) but no Rust orchestration, no DB schema, no job wiring, and
no working Python implementation.

S-130 closes that gap end-to-end: domain types, migration, repository, job
enqueueing triggered by the preparation-ready transition, worker-runner dispatch
via a subprocess-based ASR client, artifact persistence, readiness gating, and a
minimal but functional Python ASR worker (Whisper v3 via `faster-whisper`).

Without this slice, `S-140` (subtitle generation) has no source transcript to
consume, and the remainder of the ML pipeline (`S-150`, `S-170`, `S-180`) cannot
proceed.

## Objective

Deliver a fail-closed ASR transcription stage that:

- extends the artifact and status model for transcript and word-alignment artifacts;
- enqueues a `TranscriptionJob` when an asset's preparation state transitions to `Ready`;
- dispatches the job to a Python ASR subprocess via a unit-testable `AsrWorkerClient`
  trait in `crates/providers`;
- persists `TranscriptText` and `WordAlignment` derived artifacts in object storage
  with canonical storage-owned keys and correct lineage;
- gates transcript readiness on the presence of both artifact types, fail-closed;
- produces durable observability on success and failure (ADR-018); and
- implements a functional Python ASR worker (`faster-whisper`) behind the existing
  typed JSON contract.

## Scope

### Included

- New `ArtifactKind` variants: `TranscriptText`, `WordAlignment`.
- New `TranscriptionStatus` domain type (Pending / InProgress / Ready / Failed)
  and `TranscriptionStatusRecord`.
- Migration: `asset_transcription_status` table + `artifact_kind_check` extension.
- `crates/db`: `transcription_repo.rs` (status CRUD, artifact insertion, readiness
  evidence query).
- `crates/storage`: canonical key helpers for `transcripts/<asset_id>/`.
- `crates/jobs`: `TranscriptionJob` payload + `TranscriptionJobQueue` trait +
  `InMemoryTranscriptionJobQueue`.
- `crates/providers`: `AsrWorkerClient` trait + `SubprocessAsrWorkerClient` +
  `StubAsrWorkerClient` for tests.
- `apps/worker-runner`: preparation-ready hook that enqueues `TranscriptionJob`;
  `process_transcription_job(...)` handler; readiness transition logic.
- `workers/asr-worker-py`: functional implementation (FastAPI not required; the
  worker is a subprocess that reads JSON from stdin and writes JSON to stdout;
  uses `faster-whisper` for transcription and outputs word-level alignment).
- BDD feature file and docs sync.

### Excluded

- Subtitle generation (`S-140`), translation (`S-150`), and downstream ML stages.
- Multi-language ASR (one transcript per asset, source language only).
- Speaker diarization, confidence scoring, or alternative-transcription variants.
- Streaming or long-form chunked ASR; v1 processes the full audio file.
- Production ASR model selection or benchmarking; `faster-whisper` (large-v3) is
  the v1 engine.
- CDN delivery of transcript artifacts; consumption is internal pipeline only.

## Governing constraints

- ADR-006: transcript and alignment artifacts are immutable object-store records
  referenced by key and SHA-256 checksum.
- ADR-018: transcription failures and transitions must produce durable traceable
  observability.
- ADR-021: preparation completion is the upstream gate; transcription is a
  downstream derived stage, not a parallel ingest path.
- ADR-026: no environment-specific defaults compiled in; `DUBBRIDGE_ENV` governs
  the storage backend used by the worker-runner.
- Roadmap X12: derived artifacts must preserve lineage (parent_artifact_id → source
  artifact) and quality-gate transitions.

## Design decisions

### D1 — Audio input to the ASR subprocess

The `audio_uri` field in the ASR worker input schema carries a URI. For v1 the
worker-runner downloads the source artifact from `StorageAdapter` into a temporary
file and passes a `file://` URI to the subprocess. This keeps the Python worker
storage-agnostic and consistent with the `ffprobe`/`ffmpeg` temp-file pattern from
S-120.

### D2 — Transcript and alignment output path

The Python worker writes `transcript.json` and `alignment.json` to a temp directory
and returns their paths as `transcript_uri: "file:///…"` and `alignment_uri:
"file:///…"`. The worker-runner reads these files, uploads them to `StorageAdapter`
under canonical keys (`transcripts/<asset_id>/transcript.json` and
`transcripts/<asset_id>/alignment.json`), and persists the storage keys as derived
artifact records with correct lineage (parent_artifact_id = source artifact).

### D3 — ASR client abstraction in `crates/providers`

`AsrWorkerClient` is a synchronous trait with one method:
`fn transcribe(input: AsrInput) -> Result<AsrOutput, AsrError>`.
`SubprocessAsrWorkerClient` spawns the Python process and communicates via
`stdin`/`stdout` JSON. `StubAsrWorkerClient` returns a deterministic transcript
without calling a subprocess, enabling unit tests without the Python runtime.

### D4 — language_hint resolution

`TranscriptionJob` carries `source_language: String` (BCP47). This value is
resolved at enqueue time by looking up `target_languages.source_lang` for the
asset's project. If no `target_languages` row exists for the project, enqueueing
fails closed with an observable error and records `TranscriptionStatus::Failed`.

### D5 — Enqueue trigger location

The enqueue is performed inside the `process_preparation_job` handler in
`apps/worker-runner`, immediately after it writes `PreparationStatus::Ready`. This
mirrors the pattern from S-120-T5b and keeps orchestration in the worker-runner
rather than in the API route.

### D6 — Python worker engine

`workers/asr-worker-py` will use `faster-whisper` (v1.1+) with the `large-v3`
model. The worker entry point reads a single JSON payload from stdin, transcribes
the audio file referenced by `audio_uri`, and writes the result to stdout. Word
timestamps are produced natively by `faster-whisper`; the alignment JSON follows
the schema contract in `output.schema.json`.

## Affected components

| Layer | Path | Change |
|---|---|---|
| Domain | `crates/domain/src/artifact.rs` | `+TranscriptText`, `+WordAlignment` kinds; `+TranscriptionStatus`, `+TranscriptionStatusRecord` |
| Migration | `infra/migrations/0022_create_transcription.sql` | `asset_transcription_status` table + `artifact_kind_check` extension |
| DB | `crates/db/src/transcription_repo.rs` (new) | status CRUD, artifact insertion, readiness evidence |
| Storage | `crates/storage/src/lib.rs` | `+transcript_key`, `+alignment_key` helpers |
| Jobs | `crates/jobs/src/lib.rs` | `+TranscriptionJob`, `+TranscriptionJobQueue` trait, `+InMemoryTranscriptionJobQueue` |
| Providers | `crates/providers/src/lib.rs` | `+AsrWorkerClient` trait, `+SubprocessAsrWorkerClient`, `+StubAsrWorkerClient` |
| Worker-runner | `apps/worker-runner/src/main.rs` | preparation-ready enqueue hook; `process_transcription_job` handler; readiness transition |
| Python worker | `workers/asr-worker-py/` | functional `main.py` (`faster-whisper`) + `requirements.txt` + updated `Dockerfile` |
| BDD | `docs/bdd/s-130-asr-transcription.feature` | new feature file |
| Docs | `docs/plan/roadmap.md`, this plan, task ledger | sync to closed on completion |

## Task decomposition

| Task | Title | Effort | Provisional RRI | Band |
|------|-------|--------|-----------------|------|
| T1 | Domain types + migration + repository | M | 35 | Moderate |
| T2 | Job contract + enqueue from preparation-ready | M | 37 | Moderate |
| T3 | ASR client trait + worker handler + readiness gating | L | 42 | Med-high |
| T4 | Python ASR worker implementation (`faster-whisper`) | M | 37 | Moderate |
| T5 | BDD feature file + docs sync | S | — | Done (docs) |

Tasks must run in order: T1 → T2 → T3 → T4 → T5.

## Pipeline context

```
S-120 Ready (HLS + probe) → [T2 enqueue] → TranscriptionJob
    → [T3 worker-runner] → AsrWorkerClient (subprocess)
    → [T4 Python worker] → transcript.json + alignment.json
    → [T3 persist] → TranscriptText + WordAlignment artifacts
    → TranscriptionStatus::Ready → S-140 gate
```

## Open questions / risks

| Risk | Disposition |
|------|-------------|
| `faster-whisper` large-v3 model weight download time in CI | Use `base` model in tests; `large-v3` configurable via env var `ASR_MODEL_SIZE` |
| Assets without a `target_languages` row | Fail-closed at enqueue time with observable error; do not silently skip |
| Audio files that are too large for single-pass Whisper | Out of scope for v1; document as known limitation |
| `source_lang` missing from `target_languages` row | `source_lang` is `NOT NULL` in the schema (migration 0012); not an issue |
