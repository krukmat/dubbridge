# DubBridge

DubBridge is a platform for localizing audiovisual content — taking a video in one language and producing a dubbed, subtitle-ready version in another, with full traceability from intake to publication.

## What it does

Content enters the target platform as a direct upload or, once the planned intake slice lands, as an owner-authorized download from a content platform (for example, importing content from an account the owner controls). From there, DubBridge takes it through a governed pipeline:

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

DubBridge accepts content these ways:

- **Upload (operational)** — send a file through the API. Authenticated clients submit assets directly.
- **Platform download (planned, primary intake)** — the content owner authorizes DubBridge to import content from an account they control; DubBridge downloads it on their behalf and ingests it through the same governed pipeline as an uploaded file. Access requires the owner's explicit, scoped authorization.
- **Live stream recording (planned, deferred sub-case)** — for clients producing live broadcasts, DubBridge can capture an authorized live stream and ingest the result through the same governed pipeline.

All paths converge, once delivered, at the same rights gate. There is no shortcut.

## Design philosophy

The platform is built so that authorization and auditability are structural, not optional. A job cannot proceed past a stage it has not cleared. Artifacts are immutable — once ingested, the original is never modified, only transformed into derived outputs with explicit lineage. Every governance event is recorded.

The processing core is written in Rust. AI workloads (transcription, translation, voice synthesis) run as isolated Python workers behind typed contracts, keeping the ML ecosystem contained and the orchestration layer stable.

## Development setup

Requires Rust (via `rustup`) and Docker.

```
# Local infrastructure only — this Compose file is never the production deployment
# descriptor (ADR-026).
docker compose -f infra/local/docker-compose.yml up -d postgres redis minio
# Start local app containers only when you explicitly opt into the app profile.
# Put auth env vars in .env before starting the protected API container.
docker compose -f infra/local/docker-compose.yml --profile app up api worker-runner
# Configure the auth env required by your local profile before starting the
# protected API outside Compose.
cargo run -p dubbridge-api
```

Enable the shared Git hook path so local `pre-push` uses the repository hook:

```bash
git config core.hooksPath .githooks
```

Local Rust QA commands:

```bash
make qa-local          # fmt + clippy + test + cargo check
make qa-deny           # dependency policy / advisories
make qa-coverage       # 90% coverage gate (existing llvm-cov scope)
make qa-build-release  # release build verification
make qa-ci             # full local mirror of the blocking CI baseline
```

When `Cargo.toml` or `Cargo.lock` changes, install `cargo-deny` locally:

```bash
cargo install cargo-deny --version 0.18.4 --locked
```

Infrastructure: PostgreSQL for state, Redis for job coordination, MinIO for object storage.
Local app-container wiring now lives in the opt-in `app` profile of
`infra/local/docker-compose.yml`; it targets container DNS (`postgres`, `redis`) and
keeps auth secrets in a local `.env`.

### Environments (local vs production)

Local and production are separated by a fail-closed layered configuration model
governed by ADR-026 and delivered in slice P0
(`docs/plan/p0-environment-separation.md`). `crates/config` now requires an explicit
`DUBBRIDGE_ENV`, loads committed non-secret `config/<env>.toml` profiles, accepts
secrets only through injected environment variables, and runs a production
`validate()` that rejects local defaults (localhost datastores, local-fs storage,
absent auth). The Docker Compose file above is local infrastructure only and is never
the production deployment descriptor, and the local Rust app containers track
`rust-toolchain.toml` via `rust:stable`.

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

*DubBridge is under active development. JWT-protected upload ingestion and the rights ledger are operational. Platform-download intake (primary), live stream recording (deferred sub-case), media preparation, transcription, dubbing, and publication remain planned work.*
