---
type: Plan
title: "Plan: S-030 - Environment Separation & Fail-Closed Configuration"
status: closed
slice: S-030
governed_by: [ADR-026]
---
# Plan: S-030 - Environment Separation & Fail-Closed Configuration

## Objective

Make the local ↔ production boundary explicit, maintainable, and hard to confuse.
Remove the central anti-pattern in which environment-specific defaults
(`localhost` datastores, `/tmp` storage, the `dubbridge-local` bucket) are compiled
into the binary as `unwrap_or(...)` fallbacks, so a misconfigured production process
boots silently against development resources.

Invert configuration to the same fail-closed posture as the rights gate (ADR-008):
a production-like process must **refuse to start** on a missing required value or a
local default, never degrade silently. This slice is governed by ADR-026.

## Scope

### Included (this slice — Phases 0 and 1)

- An explicit environment discriminator `DUBBRIDGE_ENV ∈ {local, staging, production}`
  with no compiled default (fail-closed).
- A layered, typed configuration loader in `crates/config`
  (`code defaults ← config/default.toml ← config/<env>.toml ← DUBBRIDGE_* env vars`).
- Committed, non-secret per-environment profiles under `config/` and a `.env.example`
  contract for secrets / per-deploy values.
- A fail-closed `validate()` that, in production-like environments, rejects localhost
  datastores, the local-fs storage backend, absent auth, and the human-pretty log
  format.
- Wiring `apps/api` and `apps/worker-runner` to the new fail-closed `load()`.
- Reorganizing `infra/` so Docker Compose is local infrastructure only (with the app
  under an opt-in profile and a non-production banner), and pinning the Compose Rust
  image to the toolchain policy (closes X2/T9).

### Excluded (deferred — documented as later phases)

- **Phase 2 (couples with S-080):** wiring `build_adapter` to consume the storage backend
  selector, and parameterizing `init_tracing` to emit JSON + an exporter in
  production (ADR-018). Phase 0 lands the *schema and validation* for these fields;
  Phase 2 lands the *behavior*.
- **Phase 3 (later):** a production deployment descriptor under `infra/deploy/` and the
  secret-manager injection boundary; the owner-credential secret-store decision
  (roadmap X20, its own ADR).
- **Phase 4 (deferred):** orchestration (Kubernetes/Helm or Nomad), a telemetry
  collector, or a config service — only if multiple live environments or teams
  justify it. Not assumed now (ADR-026, Decision 6).

## Governing ADRs

- ADR-026: Layered fail-closed configuration and environment separation — the
  governing decision for this slice.
- ADR-008: Rights ledger is a mandatory, fail-closed precondition — the posture
  mirrored by configuration validation.
- ADR-023: API client authentication — auth is required outside `local`.
- ADR-018: Structured observability — environment-driven log format / exporter
  (schema in Phase 0, behavior in Phase 2).
- ADR-006: PostgreSQL for metadata, object storage for binaries — the storage backend
  seam (selector in S-030, adapter in S-080).

## Affected Files

### crates/config/src/
- `lib.rs` — replace the `from_env` constellation with `AppEnv`, `ConfigError`, the
  layered `AppConfig::load()`, the typed sub-structs (`StorageSettings`,
  `ObsSettings`, `AuthSettings`), and `validate()`.
- `Cargo.toml` — add the layered-config dependency (`figment` recommended; the
  `config` crate is an accepted alternative — both MIT/Apache-2.0, must pass
  `make qa-deny`).

### config/ (new, committed, non-secret)
- `default.toml` — universal non-secret baseline.
- `local.toml` — local values (the former in-code `localhost`/`/tmp` defaults move
  here).
- `staging.toml` — staging non-secret values.
- `production.toml` — production non-secret values (no secrets).
- `README.md` — what lives in TOML vs env; variable × environment parity table.

### repository root
- `.env.example` (new) — documents every secret / per-deploy variable.

### apps/api/src/
- `main.rs` — call `AppConfig::load()?` (fail-closed) instead of
  `AppConfig::from_env()`; derive `StorageConfig` from typed settings; log the
  resolved `DUBBRIDGE_ENV` + selected backends at startup.

### apps/worker-runner/src/
- `main.rs` — call `AppConfig::load()?` (fail-closed).

### crates/storage/src/
- `config.rs` — fold `base_path` / `endpoint_url` / `backend` into the typed settings
  owned by `crates/config` (or accept them from it); remove the in-code `/tmp`
  default fallback.

### infra/ (Phase 1)
- `infra/local/docker-compose.yml` — infrastructure only by default; `api` /
  `worker-runner` / python workers under opt-in profiles; a banner stating the file
  is not the production deployment descriptor; Rust image pinned to the
  `rust-toolchain.toml` channel.

### docs / CI (Phase 1)
- `README.md`, `docs/architecture.md` — update the Compose path after the move.
- `Makefile` / `.github/workflows/ci.yml` — a guard rejecting secret-looking keys in
  `config/*.toml`.

## Design Decisions

### Fail-closed environment resolution
`DUBBRIDGE_ENV` has no compiled default. A missing or unknown value returns
`ConfigError::MissingEnv` / `ConfigError::UnknownEnv` and aborts startup. Rationale
(ADR-026, Decision 1): a production deploy that forgot to set the variable must not
silently run as `local`.

### Layered precedence; no environment value in the binary
Resolution is `code defaults (universal only) ← config/default.toml ←
config/<env>.toml ← DUBBRIDGE_* env vars`. Code defaults hold only values true in
every environment (e.g. a starting `worker_concurrency`); never a URL, host, path, or
credential. The former `unwrap_or("postgres://...localhost...")` and
`unwrap_or("/tmp/dubbridge-storage")` values move into `config/local.toml`.

### One schema, one validator, shared by api and worker
`AppConfig` is the single configuration reader for both `apps/api` and
`apps/worker-runner` (they already share it). `validate()` runs after load. In
`staging`/`production` it rejects: a `localhost`/`127.0.0.1` database or Redis URL,
the local-fs storage backend, absent auth (ADR-023), and the pretty log format
(production must emit JSON, ADR-018). This is the configuration analogue of the rights
gate.

### Schema now, behavior staged
Phase 0 introduces the `storage.backend` selector and the `observability.log_format`
fields and validates them, but `build_adapter` and `init_tracing` consume them only in
Phase 2 (with S-080 / the observability work). Consequence: a production-like
configuration can now pass `validate()` when it selects `storage.backend = s3`,
uses non-local datastore URLs, provides auth, and selects JSON logs; however,
runtime behavior still remains incomplete until `build_adapter` and
`init_tracing` consume those settings in later phases. This keeps configuration
fail-closed while preserving the staged runtime rollout.

### Secret / config split; Compose is local-only
Non-secret environment values live in committed `config/*.toml`; secrets and per-deploy
values exist only as injected env vars (`.env` locally, a secret manager in
production). No secret is committed; a CI guard rejects secret-looking keys in
`config/*.toml`. Docker Compose is local infrastructure only and carries an explicit
banner; the production deployment artifact is separate (Phase 3).

This split is the precise boundary into which the owner-credential secret-store
(roadmap X20, ADR-025) later plugs (F6 — ADR-026 §4). When S-090-C1 adds owner-platform
credential handling, those credentials must be resolved from the **injected-secret
layer** (env vars supplied by the secret store at deploy time) — never from committed
`config/*.toml` profiles. The `AppConfig` schema may expose a credential *reference*
or *handle* (an opaque identifier pointing into the store, not the credential itself),
but the actual token, refresh token, or API key must arrive only through the env layer
and must be redacted from all logs and traces (ADR-018, ADR-025). S-030 establishes the
layer; the store mechanism itself is decided during S-090-C1 and requires its own ADR.

### Single reader: consolidate three env readers today (F1 — ADR-026 §3)
Today there are three independent env readers: `crates/config` (`AppConfig` +
`AuthSettings`), `crates/storage/src/config.rs` (`DUBBRIDGE_STORAGE_BASE_PATH` /
`DUBBRIDGE_STORAGE_ENDPOINT`), and `crates/observability` (`EnvFilter` from
`RUST_LOG`). ADR-026 §3 requires one schema and one reader. This slice collapses all
three:
- Storage settings fold into `AppConfig.storage` (Task 2 scope: `crates/storage/src/config.rs`).
- The observability filter source folds into `AppConfig.observability` (Task 4 scope:
  `crates/observability`). Phase 0 consolidates the reader; Phase 2 adds JSON/exporter
  behavior (ADR-018).
After this slice, no crate reads the environment directly except through `AppConfig::load()`.

### Config-directory resolution via `DUBBRIDGE_CONFIG_DIR` (F3 — ADR-026 §2)
A layered loader resolving `config/<env>.toml` relative to CWD breaks under
`cargo test -p dubbridge-config` (CWD = `crates/config/`, no `config/` there) and
differs again at runtime (CWD = workspace root or binary install dir). Resolution:
the loader reads an explicit `DUBBRIDGE_CONFIG_DIR` env var and defaults to a
workspace-root-relative `config/` via `std::env::var("DUBBRIDGE_CONFIG_DIR")`.
Tests supply a `figment::Jail` or `temp_env` override pointing to a fixture
directory. This guarantees that `cargo test`, local dev, and production all resolve
the same way. Acceptance criteria in Task 2 enforce this.

### `DATABASE_URL` coexistence with `DUBBRIDGE_DATABASE_URL` (F2 — ADR-026 §2)
Integration tests today read `DATABASE_URL` (sqlx-cli convention); the application
reads `DUBBRIDGE_DATABASE_URL`. There are no compile-time `query!` macros, so there
is no build-time `DATABASE_URL` dependency. Decision: `DATABASE_URL` is kept as a
**tooling alias only** (sqlx-cli migrations, local shell convenience). The application
and all its tests use `DUBBRIDGE_DATABASE_URL` as the single authoritative name.
Tests that currently read bare `DATABASE_URL` migrate to `DUBBRIDGE_DATABASE_URL` in
Task 2. This keeps the `DUBBRIDGE_` prefix grammar consistent (ADR-026 §2).

### Coverage constraint: `crates/config` is not in the ignore list (F5)
`COVERAGE_IGNORE_REGEX` in `Makefile` excludes `main.rs` files and a set of
infra-only crates, but **not** `crates/config`. The new `AppEnv`, `load()`, and
`validate()` code must reach the 90% line-coverage gate (`make qa-coverage`). Tasks
1–3 are TDD and must produce tests that cover all branches of `from_process()`,
`load()`, and `validate()` including the rejection paths.

## Module Dependencies

```
apps/api          → crates/config, crates/db, crates/storage, crates/auth, crates/observability, crates/domain
apps/worker-runner → crates/config, crates/jobs, crates/observability
crates/config     → figment (or config); serde   [no internal crate deps]
crates/storage    → crates/config (typed storage settings)
```

`crates/config` remains dependency-light and owns the schema, the loader, and
`validate()`. Today three crates read the environment independently; after this slice
no crate re-reads the environment directly — only through `AppConfig::load()`.

## Lines Affected After Implementation

Tracked per-task in `docs/tasks/s-030-environment-separation.md`. Updated after each
task completes.
