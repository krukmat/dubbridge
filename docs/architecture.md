---
type: Architecture
title: "Architecture Overview"
---

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
| JWT API principal verification + scopes | Operational (RS256 resource server; superseded-by-decision, ADR-031/S-200 replaces it with in-house HS256 issuance) | S0, ADR-023 → ADR-031 |
| Upload ingestion + rights ledger | Operational | S1, ADR-006/008/018 |
| Pending-upload durability, TTL, cleanup, coverage gate | Operational | T1 |
| Finalize atomicity + centralized durable audit emission | Planned blocking hardening | H1 |
| MinIO/S3 storage adapter | Operational | S-080, ADR-006/018/026 |
| Platform ingest (owner-authorized download) | Planned (primary S3); foundation T0/T0c/T1/T2 done | S3, ADR-025/021/006/008/018 |
| RTMP/SRT live recording ingest | Deferred sub-case (S3b); shares the S3 foundation | S3b, ADR-019/020/022 |
| Media preparation through publication | Planned | S-120..S-180 |
| HLS playback delivery | Operational | S-125, ADR-032 |
| Environment separation + reproducible app-container runtime wiring | Planned supporting surface | P0, ADR-026 |
| First-party session gateway / BFF | Operational supporting surface (opaque-session transport; superseded-by-decision, ADR-031/S-200 reduces it to a transparent relay) | P1, ADR-024 → ADR-031 |
| First-party mobile client (React Native + Expo) | Canonical authenticated product surface (opaque `session_ref` transport; ADR-031/S-200 moves it to a backend-issued bearer JWT) | P3, S-105, ADR-024/029 → ADR-031 |
| Mobile credential login with backend-issued JWT (FenixCRM parity) | Decision accepted (ADR-031); implementation planned (S-200) | S-200, ADR-031 |

## Runtime surfaces

### Operational

- `apps/api` exposes HTTP endpoints and operational health checks.
- `apps/gateway` exposes first-party auth endpoints (`/auth/login`, `/auth/callback`,
  `/auth/logout`) plus the authenticated `/api/*` proxy. It owns first-party session
  validation, renewal, rotation, expiry, logout, and backend token refresh while
  keeping tokens server-side and preserving JWT verification at `apps/api` (P1,
  ADR-024/023). **Superseded-by-decision (ADR-031, 2026-06-17):** S-200 reduces this
  surface to a transparent relay (forward `/auth/*` and `Bearer` `/api/*`); the
  session store, opaque-reference, and rotation behavior described here remain in the
  tree only until the S-200 implementation tasks land.
- `mobile/` is the first-party React Native + Expo client surface. It authenticates
  only through the session gateway, persists only the gateway-owned opaque
  `session_ref` in secure storage, and uses the authenticated gateway `/api/*`
  proxy for product requests (P3, S-105, ADR-024/029). It is the only operational
  first-party authenticated UI. The former `web/` console was retired by S-105;
  any future public website or player is a separate product decision.
  **Superseded-by-decision (ADR-031, 2026-06-17):** S-200 replaces the opaque
  `session_ref` with a backend-issued HS256 bearer JWT stored in secure storage and
  an email/password login form; mobile remains the sole authenticated surface
  (ADR-029 unchanged on that point).
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
- `S-125` HLS playback delivery (ADR-032) exposes prepared HLS manifests and
  segments through a backend-owned grant boundary. It consumes S-120 HLS artifacts,
  validates readiness and caller/publication policy, and returns rewritten manifests
  plus short-lived scoped segment references without exposing raw object-store keys.
  It is a delivery API boundary, not a public website or revived authenticated web
  console.

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
- `StorageAdapter` owns binary access and canonical key layout. Local-fs and
  S3-compatible backends are selected by config, keeping API routes and workers
  storage-agnostic.
- Uploads use a bounded-memory staging path through `StorageAdapter::put_file`
  before metadata is committed.
- Redis is reserved for job coordination.
- Cross-store writes are not atomic. Immediate cleanup attempts repair
  object-write/metadata-write divergence, and periodic reconciliation lists
  canonical `ingests/` keys, compares them against relational references, and deletes
  only planner-approved orphan candidates.

## Prepared media and playback boundaries

S-120 turns a source artifact into prepared media: durable probe metadata plus a
canonical HLS package stored behind `StorageAdapter`. That package is not itself a
client contract. S-125 owns the playback-delivery boundary for `.m3u8` manifests and
segments (ADR-032).

Playback callers receive a backend-issued grant, rewritten manifest, or signed URL
set that is scoped, expiring, and policy-checked. Clients never construct MinIO/S3
keys. Review-time playback is gated by authenticated workspace/reviewer policy;
audience-facing playback is additionally gated by the S-180 publication runtime and
ADR-030's fail-closed approval rule.

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

> **Superseded-by-decision (ADR-031, 2026-06-17, S-200-T0).** The two paragraphs
> above describe the ADR-023/ADR-024 model that is in the tree today but is now
> superseded by ADR-031. Under the accepted decision (implemented by slice S-200):
> `apps/api` becomes its own credential issuer — it validates email/password and
> issues a backend-signed **HS256** JWT — and the gateway is reduced to a transparent
> relay. The mobile device holds the bearer JWT directly (no opaque session). The
> uploader-identity invariant is preserved (the actor is still the verified token
> subject, never request-body input). The accepted security regressions (long-lived
> device token, symmetric signing secret, no pre-expiry revocation) are recorded in
> ADR-031 §Risk analysis; RS256 hardening is the recommended follow-up X-S-200-1.

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
MinIO for object storage. The default app profile still uses local-fs storage, and
`DUBBRIDGE_STORAGE_BACKEND=s3` exercises the S3-compatible adapter against MinIO.

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
