---
type: ADR
title: "ADR-026: Layered fail-closed configuration and environment separation"
status: Proposed
---

# ADR-026: Layered fail-closed configuration and environment separation

- **Status:** Proposed (scope: P0 environment separation)
- **Date:** 2026-06-03
- **Deciders:** DubBridge platform team

## Context

The repository is Rust-first and uses `DUBBRIDGE_*` environment variables read by a
typed `crates/config`. The configuration story is, however, only half 12-factor and
contains the central anti-pattern this ADR removes: **environment-specific values are
compiled into the binary as fallbacks.**

Concretely, at the time of this ADR:

- `AppConfig::from_env` (`crates/config/src/lib.rs`) falls back to
  `postgres://dubbridge:dubbridge@localhost:5432/dubbridge`, `redis://127.0.0.1:6379`,
  and bucket `dubbridge-local`. `StorageConfig::from_env`
  (`crates/storage/src/config.rs`) falls back to `/tmp/dubbridge-storage`. A
  production process that is missing an env var therefore **boots silently against
  development resources** instead of failing.
- There is no explicit notion of "which environment am I": nothing in the code
  distinguishes local from staging from production.
- `build_adapter` (`crates/storage/src/lib.rs`) hardcodes `LocalFsAdapter`, and
  `init_tracing` (`crates/observability/src/lib.rs`) always emits a human-readable
  format — neither is environment-driven.
- the local Compose file mixes local infrastructure (Postgres/Redis/MinIO) with
  application startup (`cargo run` over a bind-mounted workspace), which invites
  treating Compose as the production deployment model.
- There is no `.env.example` and no committed per-environment configuration profile,
  so the boundary between "what is local" and "what is production" is implicit and
  easy to confuse.

The platform's defining property is **fail-closed governance**: rights are a
mandatory precondition (ADR-008) and every intake converges on one fail-closed
finalize gate (ADR-021). Configuration must adopt the same posture. A misconfigured
production process is a governance hazard — it can write authorized media to the
wrong datastore or run without an identity boundary — so it must **refuse to start**,
exactly as the rights gate refuses to process without authorization.

This ADR fixes the configuration architecture and the local ↔ production separation
model. It is the governing decision for slice P0 (`docs/plan/roadmap.md`,
`docs/plan/s-030-environment-separation.md`).

## Decision

### 1. One explicit environment discriminator, fail-closed

A single variable `DUBBRIDGE_ENV ∈ {local, staging, production}` selects the
environment. It has **no compiled default**. A missing or unknown value is a hard
startup error, not a silent fallback to `local`. The process must always know which
environment it is.

### 2. Layered resolution; no environment value in the binary

Configuration resolves in ascending precedence:

```text
code defaults (universal only) ← config/default.toml ← config/<env>.toml ← DUBBRIDGE_* env vars
```

- **Code defaults** hold only values that are true in *every* environment (e.g. a
  starting `worker_concurrency`). Never a URL, host, path, or credential.
- **`config/default.toml`** is the committed, non-secret baseline.
- **`config/<env>.toml`** (`local.toml`, `staging.toml`, `production.toml`) are
  committed and **non-secret**. The former in-code `localhost`/`/tmp` fallbacks move
  here, into `config/local.toml`; they never live in the binary again.
- **`DUBBRIDGE_*` env vars** carry secrets and per-deploy overrides and win over all
  files.

The implementing slice may use a typed layered loader (`figment` or the `config`
crate; both are MIT/Apache-2.0 and pass `deny.toml`). Kubernetes is **not** assumed
at this stage.

### 3. One typed schema with a fail-closed `validate()`

A single `AppConfig` schema is the only configuration reader, shared by `apps/api`
and `apps/worker-runner` (they already share `AppConfig`). After loading, a
`validate()` step runs. In production-like environments (`staging`, `production`) it
**rejects**:

- a `localhost`/`127.0.0.1` database or Redis URL,
- the local-filesystem storage backend,
- absent auth settings (ADR-023),
- a human-pretty log format (production must emit structured JSON, ADR-018).

Validation failure aborts startup with a precise error. This is the configuration
analogue of the rights gate (ADR-008).

### 4. Non-secret profiles vs injected secrets; Compose is local-only

- **Non-secret, environment-specific values** live in committed `config/*.toml` so
  they are reviewable and diffable (`production.toml` and `local.toml` can be read
  side by side — the separation is *visible*).
- **Secrets and truly per-deploy values** exist only as injected environment
  variables — `.env` locally (already git-ignored), a secret manager in production.
  No secret is ever committed; a CI guard rejects secret-looking keys in `config/*.toml`.
- **Docker Compose is local infrastructure only** and is never the production
  deployment descriptor. The Compose file carries an explicit banner saying so; the
  production deployment artifact is separate and added when a deploy target exists.

This split is also the boundary into which the owner-credential secret-store
(ADR-025, roadmap X20) plugs: the store backs the injected-secret layer, not the
committed profiles.

### 5. Environment-driven backend seams

Backend selection becomes configuration-driven rather than hardcoded:

- A storage backend selector drives `build_adapter`. The **selector seam is P0**; the
  MinIO/S3 adapter behind it is **S2** (roadmap X9).
- `init_tracing` is parameterized by configuration: local emits a pretty format;
  production emits structured JSON plus an exporter (ADR-018).

### 6. No orchestration assumption now; the seam is the upgrade path

The env-var injection boundary (Decision 4) is exactly where a secret manager,
container platform, or orchestrator plugs in later. Adopting layered config now does
**not** commit the project to Kubernetes; it leaves that as a deferred option for when
multiple live environments or teams justify it.

## Consequences

**Positive**

- A production process can no longer boot against `localhost`/`/tmp` because of a
  missing variable — the most dangerous misconfiguration class is eliminated.
- "What is local vs production" becomes a readable, diffable property of committed
  files, not tribal knowledge.
- One schema read by both services prevents configuration drift between API and
  worker.
- The fail-closed posture matches the platform's governance identity end to end.

**Negative / trade-offs**

- Removing the in-code fallbacks means a developer with no `.env`/profile must rely on
  `config/local.toml` + the `make`/tooling default; ergonomics move from "default in
  code" to "default in the local profile" (no loss, but a change).
- One new dependency (`figment` or `config`) enters the graph; it must pass
  `make qa-deny`.
- A production-like configuration can now pass `validate()` when it selects the
  non-local storage backend, uses non-local datastore URLs, provides auth, and
  emits JSON logs. The runtime behavior behind those settings still lands in later
  phases (S2 / observability wiring), so configuration acceptance and runtime
  production-readiness are distinct checkpoints.

## Alternatives considered

- **Keep hand-rolled `std::env::var().unwrap_or(local_default)` (status quo)** —
  rejected: it is the source of the leak. Local defaults reach production, there is no
  layering and no validation.
- **Config service / Kubernetes ConfigMaps + Secrets + Kustomize overlays** —
  rejected as premature. The project has one operational environment today; this adds
  an entire orchestration vocabulary to solve a problem a typed schema and three TOML
  files solve. It remains the deferred upgrade path (Decision 6).
- **"Everything in environment variables" (12-factor purist)** — rejected: at scale it
  produces large, unreviewable env blobs that drift silently. The hybrid (committed
  non-secret profiles + injected secrets) is what makes the separation obvious.
- **Default `DUBBRIDGE_ENV` to `local`** — rejected: a production deploy that forgot
  to set it would silently run as local. No compiled default is the fail-closed choice.

## Related

- ADR-008 (rights ledger, fail-closed) — the governance posture mirrored here.
- ADR-018 (structured observability) — environment-driven log format and exporter.
- ADR-023 (API client authentication) — auth is required outside local.
- ADR-006 (metadata + object storage) — the storage backend seam (selector P0,
  adapter S2).
- ADR-025 (owner-authorized credentials) — the secret-store (roadmap X20) plugs into
  the injected-secret layer defined here.
- Roadmap: `docs/plan/roadmap.md` (slice P0, X21, X18, X2). Plan:
  `docs/plan/s-030-environment-separation.md`; Tasks:
  `docs/tasks/s-030-environment-separation.md`.
