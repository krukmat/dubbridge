# DubBridge

DubBridge is a platform for localizing audiovisual content — taking a video in one language and producing a dubbed, subtitle-ready version in another, with full traceability from intake to publication.

## What it does

Content arrives either as a direct upload or as a live stream recording. From there, DubBridge takes it through a governed pipeline:

1. **Rights verification** — nothing moves forward without a confirmed authorization basis. This is a hard gate, not a best-effort check.
2. **Media preparation** — the file is analyzed, normalized, and made ready for processing.
3. **Transcription** — speech is converted to text using AI-based recognition.
4. **Subtitle generation** — timed subtitles are produced from the transcript.
5. **Dubbing** — a translated audio track is synthesized and aligned to the original timing.
6. **Human review** — a person verifies the output before anything is published.
7. **Publication** — the localized asset is released once every gate has passed.

Every step is logged. Every artifact has a traceable origin. Nothing reaches an audience without clearing rights, quality, and review.

## Who it is for

- **Content owners** who want to expand reach into new languages without losing control of their rights or their brand.
- **Localization teams** who need a structured workflow instead of a patchwork of tools.
- **Platforms** that handle user-generated or licensed video and need to offer multilingual outputs at scale.

## How content gets in

DubBridge accepts content two ways:

- **Upload** — send a file through the API. Authenticated clients submit assets directly.
- **Live stream recording** — point an RTMP or SRT stream at DubBridge and it captures, segments, and ingests the recording automatically, feeding it into the same pipeline as an uploaded file.

Both paths converge at the same rights gate. There is no shortcut.

## Design philosophy

The platform is built so that authorization and auditability are structural, not optional. A job cannot proceed past a stage it has not cleared. Artifacts are immutable — once ingested, the original is never modified, only transformed into derived outputs with explicit lineage. Every governance event is recorded.

The processing core is written in Rust. AI workloads (transcription, translation, voice synthesis) run as isolated Python workers behind typed contracts, keeping the ML ecosystem contained and the orchestration layer stable.

## Development setup

Requires Rust (via `rustup`) and Docker.

```
docker compose -f infra/docker-compose.yml up -d
cargo run -p dubbridge-api
```

Infrastructure: PostgreSQL for state, Redis for job coordination, MinIO for object storage.

## Repository layout

```
apps/api            — HTTP API and health endpoints
apps/worker-runner  — background job execution
apps/cli            — operational utilities
crates/             — shared domain, persistence, storage, jobs, quality, auth, audit
workers/*-py        — Python AI worker contracts (ASR, translation, TTS)
infra/              — local infrastructure and database migrations
docs/               — architecture decisions, pipeline design, and development policy
```

---

*DubBridge is under active development. The rights gateway and ingestion pipeline are operational. Media preparation, transcription, dubbing, and publication are in progress.*
