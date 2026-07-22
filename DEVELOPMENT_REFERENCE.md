# Development Reference

The technical entry point for developers and agents working in the DubBridge codebase. For a product overview, start with [README.md](README.md).

This document orients you to the architecture, the decisions that constrain it, the plan, and the local workflow. It links out to the authoritative source for each topic rather than duplicating it.

---

## Contents

- [Start here](#start-here)
- [Architecture](#architecture)
- [Workspace map](#workspace-map)
- [Key invariants](#key-invariants)
- [Architecture Decision Records (ADR)](#architecture-decision-records-adr)
- [Roadmap and planning](#roadmap-and-planning)
- [Behavior specs (BDD)](#behavior-specs-bdd)
- [Developer setup](#developer-setup)
- [Infrastructure internals](#infrastructure-internals)
- [Environment configuration](#environment-configuration)
- [QA gates](#qa-gates)
- [Knowledge format (OKF)](#knowledge-format-okf)
- [Agent workflow and RRI](#agent-workflow-and-rri)
- [Directory reference](#directory-reference)

---

## Start here

Read these in order before changing anything. They are the authoritative rules for how work is planned and implemented here.

| # | Document | Why |
|---|----------|-----|
| 1 | [`README_AGENT_ORDER.md`](README_AGENT_ORDER.md) | Orientation and reading order. |
| 2 | [`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`](docs/playbooks/AGENT_WORKFLOW_GUIDE.md) | The mandatory `analyze → plan → tasks → approval → implement` workflow. **Highest authority** for any agent-facing decision. |
| 3 | [`docs/policies/HITL_AUTONOMY_POLICY.md`](docs/policies/HITL_AUTONOMY_POLICY.md) | When explicit human approval is required and what autonomy is permitted. |
| 4 | [`AGENTS.md`](AGENTS.md) | The shared task-presentation contract. |
| 5 | [`docs/architecture.md`](docs/architecture.md) | Stable boundaries; operational vs. planned surfaces. |
| 6 | [`docs/adr/`](docs/adr/) | The decisions that constrain implementation. |
| 7 | [`docs/plan/roadmap.md`](docs/plan/roadmap.md) | Slice sequence and dependencies — where any task sits. |

Mobile UI work has one additional required read: root [`DESIGN.md`](DESIGN.md), the agent-readable design intent that mirrors the shipped token system in [`mobile/src/theme/tokens.ts`](mobile/src/theme/tokens.ts).

---

## Architecture

Full detail: [`docs/architecture.md`](docs/architecture.md). The shape in brief:

DubBridge is a **Rust-first** platform for processing authorized audiovisual media into localized outputs. Rust owns the API, orchestration, persistence boundaries, governance rules, and quality gates. Python is isolated to ML worker implementations behind typed JSON contracts (`workers/*-py`), the one place the ecosystem justifies an exception ([`docs/python-exceptions.md`](docs/python-exceptions.md)).

Core principles:

- **PostgreSQL is authoritative** for structured metadata. Binary artifacts are immutable object-store records, referenced by storage key and SHA-256 checksum (ADR-006).
- **No asset reaches processing without a valid rights basis** (ADR-008). Publication stays blocked until rights, consent, processing, quality, and human-review gates all pass.
- **Governance-significant decisions require a durable audit row** plus correlated structured tracing (ADR-018) — no fire-and-forget.
- **Configuration is fail-closed and environment-explicit** (ADR-026): nothing environment-specific is compiled in, and a production-like process refuses to start on a missing value or a local default.

Intake boundaries — every mode converges on one fail-closed finalize path:

```text
programmatic client -- JWT bearer --> apps/api
first-party mobile -- session ref --> gateway/BFF -- JWT --> apps/api

apps/api direct upload ----------+
platform download (owner creds) -+--> shared rights-gated finalize --> asset + lineage + audit
RTMP/SRT live recording (S3b) ---+
```

Direct upload and the first-party session gateway are operational. Platform download (primary, ADR-025) and RTMP/SRT live recording (deferred sub-case) are planned. None may create a weaker parallel path to the one in `crates/ingestion` (ADR-021).

---

## Workspace map

Rust workspace defined in [`Cargo.toml`](Cargo.toml) (edition 2024). Dependency order flows `domain → db/storage → ingestion → apps`.

### Apps

| Crate | Role |
|-------|------|
| [`apps/api`](apps/api) | Axum HTTP API: asset ingestion, rights, finalize, HLS playback endpoints. |
| [`apps/gateway`](apps/gateway) | Session gateway / BFF; the only authenticated entrypoint for first-party mobile, relays JWT to the API (ADR-024, ADR-031). |
| [`apps/worker-runner`](apps/worker-runner) | Background job execution surface (apalis). |
| [`apps/cli`](apps/cli) | Local operational utilities. |

### Shared crates

| Crate | Role |
|-------|------|
| [`crates/domain`](crates/domain) | Core entities and invariants. No DB, no IO. |
| [`crates/db`](crates/db) | SQLx repositories; PostgreSQL is the system of record. |
| [`crates/storage`](crates/storage) | `StorageAdapter` abstraction; local-fs and S3-compatible backends; canonical key layout. |
| [`crates/ingestion`](crates/ingestion) | `finalize_ingestion_core` — the single fail-closed finalize boundary reused by all intake modes (ADR-021). |
| [`crates/auth`](crates/auth) | JWT verification, scope enforcement, principal propagation. |
| [`crates/audit`](crates/audit) | Centralized durable audit-emission boundary; domain event types stay in `domain`. |
| [`crates/jobs`](crates/jobs) | Job type definitions and apalis scheduling adapters. |
| [`crates/media`](crates/media) | ffprobe metadata extraction + HLS transcode orchestration. |
| [`crates/providers`](crates/providers) | Worker- and provider-facing contracts. |
| [`crates/playback`](crates/playback) | HLS grant issuance, manifest rewriting, short-lived scoped segment references (ADR-032). |
| [`crates/qc`](crates/qc) | Deterministic quality checks. |
| [`crates/config`](crates/config) | Typed fail-closed config loader; requires `DUBBRIDGE_ENV ∈ {local,staging,production}` (ADR-026). |
| [`crates/connectors`](crates/connectors) | `PlatformConnector` integrations for owner-authorized downloads (ADR-025). |
| [`crates/observability`](crates/observability) | Tracing/logging/health helpers; production requires JSON format (ADR-018). |

A `recorder` crate (FFmpeg subprocess capture for RTMP/SRT, ADR-019/020/022) is planned for the deferred recording sub-case and is not yet a workspace member.

---

## Key invariants

These are load-bearing. A change that weakens any of them needs an ADR first.

- **Rights gate is fail-closed (ADR-008).** No asset moves to any processing stage without a confirmed rights basis.
- **Single finalize path (ADR-021).** All intake modes (upload, platform download, live recording) must use `finalize_ingestion_core`; no parallel weaker path.
- **Immutable artifacts (ADR-006).** Ingested originals are never modified; derivatives carry explicit lineage and SHA-256 checksums.
- **Durable audit (ADR-018).** Every governance-significant event requires a PostgreSQL audit row plus correlated structured tracing.
- **Compiled defaults forbidden (ADR-026).** `crates/config` loads `config/<env>.toml` + env vars; nothing environment-specific is baked into the binary.
- **Consent precedes synthesis (ADR-028).** TTS/voice work is gated on a fail-closed voice-consent ledger.
- **Publication is gated (ADR-030).** A review-decision ledger blocks publication until a human approves.
- **Mobile is the only authenticated product UI (ADR-029).** `apps/gateway` is a relay; `mobile/` is the sole first-party surface.

---

## Architecture Decision Records (ADR)

The authoritative record of every constraining decision lives in [`docs/adr/`](docs/adr/). ADR `status:` frontmatter mirrors the prose `- **Status:**` line.

**Data, artifacts & persistence**

| ADR | Title |
|-----|-------|
| [006](docs/adr/ADR-006-postgres-metadata-object-storage-binaries.md) | PostgreSQL for metadata, object storage for binary artifacts |

**Governance & fail-closed gates**

| ADR | Title |
|-----|-------|
| [008](docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md) | Rights ledger is a mandatory, fail-closed precondition |
| [018](docs/adr/ADR-018-structured-observability-traceable-events.md) | Structured observability; every governance event must be traceable |
| [021](docs/adr/ADR-021-recording-to-asset-ingestion-bridge-fail-closed.md) | Recording-to-asset ingestion bridge with fail-closed rights |
| [028](docs/adr/ADR-028-voice-consent-ledger.md) | Voice-consent ledger and fail-closed TTS precondition |
| [030](docs/adr/ADR-030-review-decision-ledger-and-fail-closed-publication-gate.md) | Review-decision ledger and fail-closed publication gate |

**Intake & connectors**

| ADR | Title |
|-----|-------|
| [019](docs/adr/ADR-019-stream-recording-engine-ffmpeg-subprocess.md) | Stream recording engine — FFmpeg subprocess orchestration |
| [020](docs/adr/ADR-020-recording-session-lifecycle-and-segment-model.md) | Recording session lifecycle and segment model |
| [022](docs/adr/ADR-022-source-protocol-support-and-ingest-authentication.md) | Source protocol support (RTMP + SRT) and ingest authentication |
| [025](docs/adr/ADR-025-platform-connector-ingest-and-owner-authorized-credentials.md) | Platform connector ingest and owner-authorized credential model |

**Identity & access**

| ADR | Title |
|-----|-------|
| [023](docs/adr/ADR-023-api-client-authentication-and-principal-propagation.md) | API client authentication and principal propagation |
| [024](docs/adr/ADR-024-low-friction-first-party-api-access-via-session-gateway.md) | Low-friction first-party API access via session gateway |
| [027](docs/adr/ADR-027-org-membership-authorization.md) | Organization membership authorization |
| [029](docs/adr/ADR-029-mobile-as-sole-authenticated-product-surface.md) | Mobile as the sole authenticated product surface |
| [031](docs/adr/ADR-031-mobile-jwt-credential-auth-fenix-parity.md) | Mobile credential login with backend-issued JWT (FenixCRM parity) |

**Configuration & playback**

| ADR | Title |
|-----|-------|
| [026](docs/adr/ADR-026-layered-fail-closed-configuration-and-environment-separation.md) | Layered fail-closed configuration and environment separation |
| [032](docs/adr/ADR-032-hls-playback-delivery-boundary.md) | HLS playback delivery boundary |

**Knowledge, process & experience**

| ADR | Title |
|-----|-------|
| [033](docs/adr/ADR-033-open-knowledge-format-adoption.md) | Adopt the Open Knowledge Format (OKF) for repository knowledge |
| [034](docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md) | Gemma process audit log and reviewer multi-pass reconciliation |
| [035](docs/adr/ADR-035-mobile-dark-theme-netflix-style.md) | Mobile dark-theme visual identity — Netflix-style dark canvas |

---

## Roadmap and planning

The canonical sequencing map is [`docs/plan/roadmap.md`](docs/plan/roadmap.md): delivered foundations, blocking hardening gates, product phases, and cross-cutting obligations derived from the architecture and the ADR set.

- Phases use a single canonical `S-xxx` identifier (older `S0`/`P*`/`T*` labels remain as legacy aliases).
- Status legend: ✅ Done · 🟡 In progress · ⬜ Not started · 📄 Planned (plan exists, not built).
- Per-slice execution plans live in [`docs/plan/`](docs/plan/) as `s-nnn-*.md`; their task ledgers live in [`docs/tasks/`](docs/tasks/).

Before implementing, locate your task in the roadmap, open its slice plan, then its task ledger. Roadmap ↔ ledger consistency is enforced by `make qa-roadmap-drift`.

---

## Behavior specs (BDD)

Canonical `.feature` specs live in [`docs/bdd/`](docs/bdd/) ([convention](docs/bdd/README.md)). Scenario IDs are stable and behavioral. Executable evidence for mobile flows may live in `mobile/maestro/` or mobile tests even when the canonical spec is here.

| Feature | Scope |
|---------|-------|
| [`s-050-mobile-client.feature`](docs/bdd/s-050-mobile-client.feature) | First-party mobile client |
| [`s-055-maestro-suite.feature`](docs/bdd/s-055-maestro-suite.feature) | Maestro E2E visual suite |
| [`s-060-mobile-asset-lifecycle.feature`](docs/bdd/s-060-mobile-asset-lifecycle.feature) | Mobile asset lifecycle |
| [`s-120-media-preparation.feature`](docs/bdd/s-120-media-preparation.feature) | Media preparation |
| [`s-125-hls-playback-delivery.feature`](docs/bdd/s-125-hls-playback-delivery.feature) | HLS playback delivery |
| [`s-127-mobile-review-player.feature`](docs/bdd/s-127-mobile-review-player.feature) | Mobile review player |
| [`s-160-review.feature`](docs/bdd/s-160-review.feature) | Human review workflow |
| [`s-200-mobile-auth.feature`](docs/bdd/s-200-mobile-auth.feature) | Mobile authentication |
| [`s-210-mobile-product-experience.feature`](docs/bdd/s-210-mobile-product-experience.feature) | Mobile product experience |
| [`p4-workspace.feature`](docs/bdd/p4-workspace.feature) · [`p6-compliance.feature`](docs/bdd/p6-compliance.feature) | Workspace · compliance |

---

## Developer setup

Requires Rust (via `rustup`, tracked by [`rust-toolchain.toml`](rust-toolchain.toml)) and Docker.

```bash
# Start local infrastructure (never the production descriptor — ADR-026)
docker compose -f infra/local/docker-compose.yml up -d postgres redis minio

# Run the API against local config
DUBBRIDGE_ENV=local cargo run -p dubbridge-api
```

Install the repository Git hooks once per clone:

```bash
git config core.hooksPath .githooks   # or: make install-hooks
```

### Single crate / single test

```bash
cargo test -p <crate-name>
cargo test -p <crate-name> -- <test_name> --nocapture
```

### Database migrations

SQL migrations live in [`infra/migrations/`](infra/migrations/) (`0001_*.sql` onward) and are applied against the local Postgres container.

### Dependency policy

When `Cargo.toml` or `Cargo.lock` changes, install `cargo-deny` and run the policy gate:

```bash
cargo install cargo-deny --version 0.18.4 --locked
make qa-deny   # advisories + license policy
```

### Storage backend

The default local profile uses `storage.backend = local_fs`. To exercise the S3-compatible path against local MinIO:

```bash
DUBBRIDGE_STORAGE_BACKEND=s3 docker compose -f infra/local/docker-compose.yml --profile app up
```

The Compose file targets `http://minio:9000`, bucket `dubbridge-local`, credentials `dubbridge` / `dubbridge123`. Local-only — never use in staging or production.

### App profile (Compose)

To run the full local stack including the API and worker containers:

```bash
docker compose -f infra/local/docker-compose.yml --profile app up api worker-runner
```

Requires auth environment variables in a local `.env` before starting. Container DNS targets `postgres`, `redis`, and `minio` by service name.

### Mobile

```bash
cd mobile && npm install
npm run typecheck && npm run lint && npm test
npm run screenshots   # Maestro E2E visual suite (requires Java + device)
```

Mobile presentation work must read [`DESIGN.md`](DESIGN.md) first (ADR-035).

---

## Infrastructure internals

### Object cleanup and reconciliation

Uploads write the storage object first, then persist the relational pending-ingestion row. If the relational write fails, the API attempts immediate cleanup of the just-written object and logs any cleanup failure with the ingest token and storage key for later recovery.

The API also runs periodic storage reconciliation from the cleanup worker. The pass:

1. Lists canonical `ingests/` keys through `StorageAdapter`.
2. Compares them with `pending_ingestions.storage_key` and `artifact_records.storage_key`.
3. Deletes only planner-approved orphan candidates.

Referenced objects are retained. Malformed or unexpected keys are skipped and logged. Delete failures preserve enough context for a later run to retry. Rerunning after a successful deletion is a no-op for already-repaired state.

---

## Environment configuration

DubBridge uses a fail-closed layered configuration model (ADR-026, delivered in slice S-030):

- `DUBBRIDGE_ENV` is required; valid values are `local`, `staging`, `production`.
- `crates/config` loads the matching committed profile from `config/<env>.toml`.
- Secrets are injected through environment variables only — nothing environment-specific is baked into the binary.
- In `production`, the loader rejects any configuration with localhost datastores, local-fs storage, or absent auth. The check is fail-closed and cannot be bypassed.

The Compose file under `infra/local/` is local infrastructure only and is never the production deployment descriptor.

Full design and rationale: [`docs/plan/s-030-environment-separation.md`](docs/plan/s-030-environment-separation.md).

---

## QA gates

Run `make qa-local` before committing. `make qa-ci` is the blocking baseline CI enforces.

| Command | What it checks |
|---------|----------------|
| `make qa-local` | fmt + clippy + tests + cargo check |
| `make qa-fmt` | `cargo fmt --check` |
| `make qa-lint` | `cargo clippy -D warnings` |
| `make qa-test` | `cargo test --workspace --all-features` |
| `make qa-deny` | dependency advisories and license policy |
| `make qa-coverage` | 90% line coverage gate (llvm-cov, `--test-threads=1`) |
| `make qa-docs` | doc consistency + task coverage + roadmap drift + OKF frontmatter (deterministic, no LLM) |
| `make qa-docs-review` | `qa-docs` + Gemma Reviewer pass (task closure / CI, not pre-push) |
| `make qa-okf-frontmatter` | OKF frontmatter validator alone |
| `make qa-roadmap-drift` | ledger ↔ roadmap consistency |
| `make qa-maintainability` | `python3 scripts/check-maintainability.py` |
| `make qa-mobile` | React Native typecheck + lint + Jest |
| `make qa-build-release` | release build verification |
| `make qa-ci` | Full CI mirror: all of the above |

---

## Knowledge format (OKF)

Every file in [`docs/`](docs/) carries YAML frontmatter declaring a closed `type:` and a `status:` (ADR-033). Missing or invalid frontmatter blocks commits and CI.

```yaml
---
type: ADR
status: Accepted
---
```

The closed `type` vocabulary and its location rules (authoritative: [`docs/knowledge/README.md`](docs/knowledge/README.md)):

| `type` | Location |
|--------|----------|
| `ADR` | `docs/adr/ADR-*.md` |
| `Architecture` | `docs/architecture.md` (singleton) |
| `Roadmap` | `docs/plan/roadmap.md` (singleton) |
| `Plan` | `docs/plan/*.md` |
| `TaskList` | `docs/tasks/*.md` |
| `Playbook` | `docs/playbooks/*.md` |
| `Policy` | `docs/policies/*.md` |
| `Proposal` | `docs/proposals/*.md` |
| `Audit` | `docs/audit/*.md` |
| `Prompt` | `docs/prompts/*.md` |

```bash
make qa-okf-frontmatter   # validator alone
make qa-docs              # deterministic doc gate (includes OKF; no Gemma)
make qa-docs-review       # qa-docs + Gemma Reviewer (task closure / CI)
```

---

## Agent workflow and RRI

All development follows the mandatory `analyze → plan → tasks → approval → implement` workflow in [`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`](docs/playbooks/AGENT_WORKFLOW_GUIDE.md) — the highest-authority source for agent-facing decisions. Approval rules are in [`docs/policies/HITL_AUTONOMY_POLICY.md`](docs/policies/HITL_AUTONOMY_POLICY.md).

### Required Reasoning Index (RRI)

Before any implementation task, agents compute an RRI score — a measure of how much reasoning, caution, and verification the task requires. It combines cyclomatic complexity, files affected, domain risk, test-coverage risk, ambiguity, coupling, and security impact, with penalties for high-risk combinations.

| Band | Range | What it requires |
|------|-------|-----------------|
| Low | 0–25 | Auto-execute via local Gemma (Ollama); orchestrator reviews and reports |
| Moderate | 26–40 | Confirm area tests exist |
| Med-high | 41–55 | Plan + explicit acceptance criteria |
| Complex | 56–70 | Plan first; human reviews the plan |
| High | 71–85 | Characterization tests + human reviews the diff |
| Very high | 86–100 | ADR + risk analysis + decompose first |
| Excessive | > 100 | Architecture work required before any implementation |

```bash
python3 scripts/rri.py   # score a task before starting
```

Full policy: [`docs/policies/RRI_POLICY.md`](docs/policies/RRI_POLICY.md). Low-RRI local-model handoff: [`docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`](docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md).

---

## Directory reference

```
apps/api            — Axum HTTP API and health endpoints
apps/gateway        — session gateway / BFF (first-party mobile entrypoint)
apps/worker-runner  — background job execution
apps/cli            — operational utilities
crates/             — shared domain, persistence, storage, jobs, media, auth, audit, …
workers/*-py        — Python AI worker contracts (ASR, translation, TTS)
infra/local/        — local Docker Compose infrastructure
infra/migrations/   — SQL migrations (PostgreSQL system of record)
config/             — per-environment non-secret config profiles
docs/adr/           — architecture decision records
docs/architecture.md— architecture overview
docs/plan/          — roadmap + per-slice execution plans
docs/tasks/         — per-slice task ledgers
docs/bdd/           — behavior specs (.feature)
docs/playbooks/     — workflow guides
docs/policies/      — autonomy, RRI, and other policies
scripts/            — rri.py, OKF + roadmap-drift + maintainability checks
mobile/             — React Native + Expo mobile client
```
