# Architecture Overview

DubBridge is a Rust-first platform for processing authorized audiovisual media into
localized outputs. This overview describes stable boundaries and distinguishes
operational surfaces from planned ones. Delivery sequence lives in
`docs/plan/roadmap.md`.

## Core principles

- Rust owns API surfaces, orchestration, persistence boundaries, governance rules,
  and quality gates.
- Python is isolated to ML worker implementations where the ecosystem justifies an
  exception (`docs/python-exceptions.md`).
- PostgreSQL is authoritative for structured metadata. Binary artifacts are
  immutable object-store records referenced by storage key and SHA-256 checksum
  (ADR-006).
- No asset reaches processing without a valid rights basis (ADR-008).
- Publication remains blocked until rights, consent, processing, quality, and human
  review gates succeed.
- Governance-significant decisions require a durable audit row plus correlated
  structured tracing (ADR-018).
- Runtime configuration is fail-closed and environment-explicit: no environment-
  specific value is compiled into the binary, and a production-like process refuses to
  start on a missing required value or a local default (localhost datastore, local-fs
  storage, absent auth). Non-secret environment values live in committed per-environment
  profiles; secrets exist only in injected environment variables. Local Docker Compose
  is local infrastructure only, never the production deployment descriptor (ADR-026, P0).

## Delivery status

| Capability | Status | Source |
|------------|--------|--------|
| JWT API principal verification + scopes | Operational | S0, ADR-023 |
| Upload ingestion + rights ledger | Operational | S1, ADR-006/008/018 |
| Pending-upload durability, TTL, cleanup, coverage gate | Operational | T1 |
| Finalize atomicity + centralized durable audit emission | Planned blocking hardening | H1 |
| MinIO/S3 storage adapter | Planned | S2 |
| Platform ingest (owner-authorized download) | Planned (primary S3); foundation T0/T0c/T1/T2 done | S3, ADR-025/021/006/008/018 |
| RTMP/SRT live recording ingest | Deferred sub-case (S3b); shares the S3 foundation | S3b, ADR-019/020/022 |
| Media preparation through publication | Planned | S4..S9 |
| Environment separation + reproducible app-container runtime wiring | Planned supporting surface | P0, ADR-026 |
| First-party session gateway / BFF | Operational supporting surface | P1, ADR-024 |
| First-party mobile client (React Native + Expo) | Canonical authenticated product surface | P3, S-105, ADR-024/029 |

## Runtime surfaces

### Operational

- `apps/api` exposes HTTP endpoints and operational health checks.
- `apps/gateway` exposes first-party auth endpoints (`/auth/login`, `/auth/callback`,
  `/auth/logout`) plus the authenticated `/api/*` proxy. It owns first-party session
  validation, renewal, rotation, expiry, logout, and backend token refresh while
  keeping tokens server-side and preserving JWT verification at `apps/api` (P1,
  ADR-024/023).
- `mobile/` is the first-party React Native + Expo client surface. It authenticates
  only through the session gateway, persists only the gateway-owned opaque
  `session_ref` in secure storage, and uses the authenticated gateway `/api/*`
  proxy for product requests (P3, S-105, ADR-024/029). It is the only operational
  first-party authenticated UI. The former `web/` console was retired by S-105;
  any future public website or player is a separate product decision.
- `apps/worker-runner` is the Rust background-job execution surface; its real queue
  consumption remains to be implemented as slices require it.
- `apps/cli` hosts local operational commands for development and administration.
- `workers/*-py` define AI-workload contracts behind typed JSON schemas.

### Planned

- `crates/connectors` (primary S3, ADR-025) will own per-platform integrations
  behind a `PlatformConnector` trait. For the owner-authorized download use case
  (the content owner grants scoped access to their own YouTube/Vimeo account),
  it resolves ownership + metadata and downloads the owner's media to local
  staging, which is then bridged into the same fail-closed finalize path as an
  upload (ADR-021). The request builder is a pure function; only the executor
  performs network IO. No DB dependency.
- `crates/recorder` (deferred sub-case S3b, ADR-019) will supervise FFmpeg
  subprocess capture for RTMP/SRT **live** recording, driving a fail-closed
  recording-session lifecycle and segment model (ADR-020) with capture-edge source
  authentication (ADR-022). Its v1 output contract was fixed by S3 Task 0c (local
  HLS fMP4 staging + one assembled MP4). It is built only when a real
  live-broadcast client need exists; it is not on the primary S3 critical path.
## Shared crates

- `domain`: Core entities and invariants.
- `db`: SQLx persistence wiring and repositories.
- `storage`: Object-storage abstractions and path conventions.
- `jobs`: Background job types and scheduling adapters.
- `media`: Media probing and process-orchestration boundaries.
- `providers`: Worker and provider-facing contracts.
- `qc`: Deterministic quality checks.
- `auth`: Authentication and authorization policy boundaries.
- `audit`: Reserved shared namespace for the centralized audit-emission boundary;
  domain event types remain in `domain` and PostgreSQL writes remain in `db`.
- `ingestion` (H1 boundary): Transport-neutral finalize workflow reusable by API
  uploads, platform-download bridges, and (S3b) recording bridges.
- `connectors` (planned, primary S3): Per-platform `PlatformConnector` integrations
  for owner-authorized downloads (YouTube first). Pure request builder + IO executor;
  depends on `domain` + `config`, no DB (ADR-025).
- `recorder` (planned, deferred S3b): FFmpeg subprocess capture for RTMP/SRT live
  recording (ADR-019/020/022).
- `config`: Typed runtime configuration; layered fail-closed loader with an explicit
  `DUBBRIDGE_ENV` and production validation (ADR-026, P0).
- `observability`: Logging, tracing, and health-reporting helpers.

## Intake boundaries

```text
programmatic client -- JWT bearer token --> apps/api
first-party mobile -- opaque session ref --> session gateway / BFF -- JWT/internal credential --> apps/api

apps/api direct upload ---------------+
platform download (owner creds) ------+--> shared rights-gated finalize --> asset + lineage + audit
RTMP/SRT live recording (S3b) --------+
```

Direct upload and the first-party session gateway are operational. The gateway is
the only authenticated entrypoint for first-party mobile product API calls;
it renews or rotates first-party sessions when allowed and proxies `/api/*` to
`apps/api`. **Platform
download (primary S3, ADR-025)** is planned; **RTMP/SRT live recording is the
deferred S3b sub-case**.
Every intake mode — upload, platform download, and live recording — must use the same
fail-closed ingestion boundary (`finalize_ingestion_core`); none may create a weaker
parallel path (ADR-021, producer-agnostic).

## Persistence boundaries

- PostgreSQL stores assets, rights records, artifact references, audit events, and
  pending-ingestion lifecycle state.
- `StorageAdapter` owns binary access and key layout. `LocalFsAdapter` is currently
  operational; S2 adds MinIO/S3 behavior behind the same trait. The S1 upload route
  still builds its `ingests/{token}/...` key locally; S2 must move that convention
  behind `crates/storage`.
- The upload API currently buffers multipart file bytes in memory before
  `StorageAdapter::put`. S2 must choose a streaming or presigned object-store flow
  before production-scale uploads.
- Redis is reserved for job coordination.
- Cross-store writes are not atomic. Object-store orphan reconciliation is required
  in S2; relational writes inside finalize must become atomic in H1.

## Identity boundaries

`apps/api` is an OAuth 2.0 resource server. Protected routes consume a verified JWT
bearer principal through `crates/auth`; handlers never trust caller-supplied uploader
identity (ADR-023).

The session gateway / BFF changes first-party client ergonomics, not the core API
trust boundary (ADR-024). It owns browser/mobile session lifecycle behavior:
login, session validation, renewal, rotation, expiry, logout, backend token refresh,
and authenticated `/api/*` proxying. First-party clients never renew tokens or
sessions themselves; they carry the current opaque session transport and update it
only when the gateway returns a rotated reference. `apps/api` never receives a
browser/mobile session reference; it receives an authenticated backend request from
the gateway and continues to enforce protected-resource authorization.

Intake-source credentials are a separate concern from the API principal and from
each other: owner platform credentials for downloads are stored by reference and
redacted (primary S3, ADR-025); live RTMP/SRT source credentials are a capture-edge
concern (deferred S3b, ADR-022). Neither is ever conflated with the verified API
bearer principal.

## Audit boundary

Today, audit event types live in `crates/domain/src/audit.rs` and PostgreSQL writes
live in `crates/db/src/audit_repo.rs`. H1 must add one shared emission contract that
coordinates durable writes with tracing and removes fire-and-forget governance audit
paths. Recording lifecycle events must reuse that contract.

## Local development topology

Local development uses PostgreSQL for primary state, Redis for job coordination, and
MinIO for object storage. Until S2 lands, the API storage adapter still resolves to
`LocalFsAdapter`.

The infrastructure containers are usable today with
`docker compose -f infra/local/docker-compose.yml up -d postgres redis minio`. That Compose
file is **local infrastructure only**; it is never the production deployment
descriptor (ADR-026).

Environment separation is governed by ADR-026 and delivered in P0. `crates/config`
now uses a fail-closed layered model: an explicit `DUBBRIDGE_ENV`, committed
non-secret `config/<env>.toml` profiles, secrets only in injected environment
variables, and a `validate()` that rejects local defaults in production-like
environments. The opt-in `app` profile wires container service DNS URLs and
config-path resolution for `api` / `worker-runner`, and the local Rust container image
tracks the repo toolchain policy (`rust-toolchain.toml` = `stable`).
