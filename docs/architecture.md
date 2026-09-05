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
  is local infrastructure only, never the production deployment descriptor (ADR-026).

## Delivery status

| Capability | Status | Source |
|------------|--------|--------|
| Mobile credential login, backend-issued HS256 JWT | Operational (FenixCRM parity) | S-200, ADR-031 |
| Upload ingestion + rights ledger | Operational | S-010, ADR-006/008/018 |
| Pending-upload durability, TTL, cleanup, coverage gate | Operational | T1 |
| MinIO/S3 storage adapter | Operational | S-080, ADR-006/018/026 |
| Platform ingest (owner-authorized download) | Planned (primary); foundation done | S-090, ADR-025/021/006/008/018 |
| RTMP/SRT live recording ingest | Deferred sub-case; shares the S-090 foundation | S-095, ADR-019/020/022 |
| Media preparation (ffprobe + HLS) | Operational | S-120 |
| HLS playback delivery | Operational | S-125, ADR-032 |
| Environment separation + reproducible container runtime wiring | Operational | S-030, ADR-026 |
| First-party session gateway (transparent JWT relay) | Operational | S-040, ADR-031 |
| First-party mobile client (React Native + Expo) | Canonical, sole authenticated product surface | S-050/S-105, ADR-029/031 |
| Mobile P2P runtime boundary | P1 closed `[x] Done` 2026-09-01 (7/8 children PASS: packaging/protocol, ownership/composition, storage, replication transport, verification/reconnect/teardown; P1.F3b itself stays `not PASS`, non-blocking, deferred into X28); no product P2P runtime or network activity is active outside bounded proof runners | MVP0-P2P P1, ADR-043 |
| P2P audience delivery (encrypted publication, invite/claim, verified sync, loopback playback) | Not started — ADR-044 is `Proposed`; D1 `O3 parallel`, D2 `K1`, and D3 `O4` are resolved; explicit D4 ADR acceptance still blocks P2 | MVP0-P2P P2–P7, ADR-044 |

Human review runtime (S-170) and publication runtime (S-180) have no plan/task
ledger yet.

## Runtime surfaces

### Operational

- `apps/api` exposes HTTP endpoints, operational health checks, and — since ADR-031
  (S-200) — its own credential issuer: it validates email/password and issues a
  backend-signed **HS256** JWT with algorithm pinning. Protected routes consume a
  verified JWT bearer principal through `crates/auth`; handlers never trust
  caller-supplied uploader identity (ADR-023, carried forward by ADR-031). Accepted
  security tradeoffs (long-lived device token, symmetric signing secret, no
  pre-expiry revocation) are recorded in ADR-031 §Risk analysis; RS256 hardening is
  the recommended follow-up X-S-200-1.
- `apps/gateway` is a **transparent relay** (ADR-031, S-200): it forwards `/auth/*`
  and `Bearer`-authenticated `/api/*` without holding a server-side session store.
  It no longer performs opaque-session issuance/rotation — that P1/ADR-024 model
  was superseded 2026-06-17.
- `mobile/` is the first-party React Native + Expo client and the **only**
  operational first-party authenticated UI (ADR-029): it logs in with
  email/password, holds the backend-issued bearer JWT in secure storage, and calls
  the gateway's `/api/*` relay directly. The former `web/` console was retired by
  S-105; any future public website or player is a separate product decision.
- `apps/worker-runner` is the Rust background-job execution surface; real queue
  consumption is implemented as slices require it.
- `apps/cli` hosts local operational commands for development and administration.
- `workers/*-py` define AI-workload contracts behind typed JSON schemas.

### Planned

- `mobile/src/p2p/` (MVP0-P2P P1, accepted ADR-043): the app composition root,
  above `RootNavigator`, owns `AuthProvider` and a `P2PProvider`. The provider
  owns a framework-independent `P2PService`, which owns one product
  `BareRuntimeClient`/worklet and remains network-inert until an explicit future
  command. Reproducibly packaged, versioned `bare-rpc` communication and explicit
  fatal/suspend/resume handling form the runtime seam. P1's two-worklet
  seed/client topology stays isolated in a development-only proof runner and
  uses verifiably deleted cache storage; it is not the product topology. The P0
  probe/custom bridge was retained as the unchanged oracle through F3a.1's
  `P2PDevelopmentHarness` (owner-verified), then retired in P1.F3a.2; P1.F3b
  separately audited each related config/dependency setting but stays
  `not PASS` itself (device-proof criteria deferred, non-blocking, into
  X28). ADR-043 and the revised P1 parent were approved on 2026-08-27, and
  **P1 closed `[x] Done` on 2026-09-01** — 7/8 children (F1 packaging/RPC
  seam, F2 ownership/composition, F3a scaffold retirement, A1/A2 storage
  lifecycle, B1 replication transport, B2 verification/reconnect/teardown)
  are PASS, with P1.F3b's residual status accepted as non-blocking.
  Composition, storage, and an isolated replication proof exist
  and are unit-tested, but no product-facing P2P command, invite, or
  network activity is wired to the app — this boundary remains
  non-operational for end users pending P2–P7.
- **P2P audience delivery** (MVP0-P2P P2–P7, ADR-044 `Proposed`): the
  control-plane/data-plane split in which `apps/api` stays the sole
  authorization authority while an encrypted P2P data plane — package builder,
  durable publication state, an Availability Node seeding ciphertext only,
  mobile Hyperdrive/Hyperswarm sync, and a loopback HLS gateway — replaces
  server media transport for the certified invited-playback path. ADR-032
  remains authoritative for review playback and is not replaced. ADR-044 D1
  selected `O3 parallel`; D2 selected `K1` (AES-256-GCM package encryption,
  server-wrapped CK, HPKE P-256 device envelope, non-exportable Android
  Keystore key, no external hardware/StrongBox requirement, fail-closed
  capability handling, no silent K2 fallback). D3 selected `O4`: PostgreSQL
  publication state + transactional outbox are durable authority; queue usage
  is an optional/replayable accelerator only; a PostgreSQL reconciler repairs
  lost/stuck/unknown work; delivery is at-least-once/idempotent under one stable
  logical publication/K1 lineage; and `P2P_READY` is separate from S-120 Ready
  and written only after durable confirmation of the external publication.
  ADR-044 is still **not accepted**; D4 acceptance blocks P2, so none of this
  D2P publication path exists in product code. Design inputs:
  `docs/plan/mvp0-p2p-design-inputs.md`.
- `crates/connectors` (primary S-090, ADR-025): per-platform integrations behind a
  `PlatformConnector` trait. For owner-authorized download (content owner grants
  scoped access to their own YouTube/Vimeo account), it resolves ownership/metadata
  and downloads to local staging, bridged into the same fail-closed finalize path as
  an upload (ADR-021). Request builder is a pure function; only the executor
  performs network IO; no DB dependency.
- `crates/recorder` (deferred S-095, ADR-019): FFmpeg subprocess capture for
  RTMP/SRT **live** recording, driving a fail-closed recording-session lifecycle and
  segment model (ADR-020) with capture-edge source authentication (ADR-022). v1
  output contract: local HLS fMP4 staging + one assembled MP4. Built only when a
  real live-broadcast client need exists.

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
- `ingestion`: Transport-neutral finalize workflow (`finalize_ingestion_core`)
  reusable by API uploads, platform-download bridges, and (S-095) recording
  bridges (ADR-021).
- `connectors` (planned, primary S-090): Per-platform `PlatformConnector`
  integrations for owner-authorized downloads (YouTube first); depends on
  `domain` + `config`, no DB (ADR-025).
- `recorder` (planned, deferred S-095): FFmpeg subprocess capture for RTMP/SRT
  live recording (ADR-019/020/022).
- `config`: Typed runtime configuration; layered fail-closed loader with an explicit
  `DUBBRIDGE_ENV` and production validation (ADR-026).
- `observability`: Logging, tracing, and health-reporting helpers.

## Intake boundaries

```text
mobile client -- bearer JWT --> gateway (transparent relay) --> apps/api
programmatic client -- bearer JWT --> apps/api

apps/api direct upload ---------------+
platform download (owner creds) ------+--> shared rights-gated finalize --> asset + lineage + audit
RTMP/SRT live recording (S-095) ------+
```

Direct upload and the gateway relay are operational. Platform download
(primary, ADR-025) is planned; RTMP/SRT live recording is the deferred S-095
sub-case. Every intake mode must use the same fail-closed
`finalize_ingestion_core` boundary — none may create a weaker parallel path
(ADR-021, producer-agnostic).

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
- For future P2P publication, D3/O4 extends that cross-store discipline: a
  transactional outbox captures durable publication intent in PostgreSQL before
  external publication; queue delivery is optional and non-authoritative; a
  reconciler uses authoritative PostgreSQL state to recover lost/unknown work.

## Prepared media and playback boundaries

S-120 turns a source artifact into prepared media: durable probe metadata plus a
canonical HLS package stored behind `StorageAdapter` — not itself a client
contract. S-125 owns the playback-delivery boundary for `.m3u8` manifests and
segments (ADR-032): callers receive a backend-issued grant, rewritten manifest, or
signed URL set that is scoped, expiring, and policy-checked. Clients never
construct MinIO/S3 keys. Review-time playback is gated by authenticated
workspace/reviewer policy; audience-facing playback is additionally gated by the
S-180 publication runtime and ADR-030's fail-closed approval rule.

## Identity boundaries

`apps/api` is its own credential issuer and JWT resource server (ADR-031, S-200):
it validates email/password, issues a backend-signed HS256 JWT with algorithm
pinning, and verifies that bearer principal on every protected route through
`crates/auth`. Handlers never trust caller-supplied uploader identity — the actor
is always the verified token subject, never request-body input (ADR-023, carried
forward by ADR-031).

`apps/gateway` is a transparent relay (ADR-031): it forwards `/auth/*` and
`Bearer`-authenticated `/api/*` to `apps/api` without an intervening session
transform. The mobile device holds the bearer JWT directly; `apps/api` never
receives a browser/mobile session reference, only the verified JWT.

This inverts the earlier ADR-023/ADR-024 design (RS256 external resource server +
opaque-session gateway), superseded 2026-06-17. Accepted regressions (long-lived
device token, symmetric signing secret, no pre-expiry revocation) are recorded in
ADR-031 §Risk analysis; RS256 hardening is the recommended follow-up X-S-200-1.

Intake-source credentials are a separate concern from the API principal and from
each other: owner platform credentials for downloads are stored by reference and
redacted (primary S-090, ADR-025); live RTMP/SRT source credentials are a
capture-edge concern (deferred S-095, ADR-022). Neither is ever conflated with the
verified API bearer principal.

## Audit boundary

Audit event types live in `crates/domain/src/audit.rs`; PostgreSQL writes live in
`crates/db/src/audit_repo.rs`, coordinated with tracing through the centralized
durable audit-emission boundary (ADR-018). Recording lifecycle events reuse the
same contract.

## Local development topology

Local development uses PostgreSQL for primary state, Redis for job coordination, and
MinIO for object storage. The default app profile still uses local-fs storage, and
`DUBBRIDGE_STORAGE_BACKEND=s3` exercises the S3-compatible adapter against MinIO.

The infrastructure containers are usable today with
`docker compose -f infra/local/docker-compose.yml up -d postgres redis minio`. That Compose
file is **local infrastructure only**; it is never the production deployment
descriptor (ADR-026).

`crates/config` uses a fail-closed layered model: an explicit `DUBBRIDGE_ENV`,
committed non-secret `config/<env>.toml` profiles, secrets only in injected
environment variables, and a `validate()` that rejects local defaults in
production-like environments (ADR-026). The opt-in `app` profile wires container
service DNS URLs and config-path resolution for `api`/`worker-runner`, and the
local Rust container image tracks the repo toolchain policy
(`rust-toolchain.toml` = `stable`).
