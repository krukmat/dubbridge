# Tasks: P0 Environment Separation & Fail-Closed Configuration

Plan: `docs/plan/p0-environment-separation.md` · ADR: ADR-026 · Roadmap: P0, X21, X18, X2

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

This slice delivers **Phase 0** (Tasks 1–4) and **Phase 1** (Tasks 5–6). Phases 2–4
are deferred and documented at the end as out-of-scope follow-ups.

---

## Phase 0 — Fail-closed layered configuration

## Task 1 — `AppEnv` + `ConfigError` + fail-closed `DUBBRIDGE_ENV` resolution

**Effort:** M
**Depends on:** nothing

### Scope
In `crates/config/src/lib.rs`, introduce:

- `enum AppEnv { Local, Staging, Production }` with `is_production_like()`.
- `AppEnv::from_process()` reading `DUBBRIDGE_ENV` with **no compiled default**.
- `enum ConfigError` with at least `MissingEnv`, `UnknownEnv(String)`,
  `Load(..)`, `Validation(String)` (use `thiserror`).

### Acceptance criteria (TDD — write tests first)
- Missing `DUBBRIDGE_ENV` returns `ConfigError::MissingEnv` (not a fallback to local).
- An unknown value (`"prod"`, `""`) returns `ConfigError::UnknownEnv`.
- `local` / `staging` / `production` parse to the matching variant.
- `is_production_like()` is true for `Staging` and `Production`, false for `Local`.
- Tests use `temp_env` (or `figment::Jail`) for env isolation; `cargo test -p dubbridge-config` passes.

### Files affected
- `crates/config/src/lib.rs`
- `crates/config/Cargo.toml` (add `thiserror`)

### Status: [ ] Not started

---

## Task 2 — Layered schema + loader + committed profiles + `.env.example`

**Effort:** L
**Depends on:** Task 1

### Scope
- Add the layered-config dependency (`figment` recommended; `config` crate accepted).
  Run `make qa-deny` after adding it.
- Define the typed schema: `AppConfig { env, api_port, database_url, redis_url,
  storage: StorageSettings, worker_concurrency, observability: ObsSettings,
  auth: Option<AuthSettings> }`, with `enum StorageBackend { LocalFs, S3 }` and
  `enum LogFormat { Pretty, Json }`.
- Implement `AppConfig::load() -> Result<Self, ConfigError>` merging
  `config/default.toml` ← `config/<env>.toml` ← `Env::prefixed("DUBBRIDGE_")`.
- **Consolidate the three env readers into `AppConfig` (F1 — ADR-026 §3):**
  - `crates/storage/src/config.rs` (`DUBBRIDGE_STORAGE_BASE_PATH`,
    `DUBBRIDGE_STORAGE_ENDPOINT`): fold into `AppConfig.storage`; remove the
    standalone `StorageConfig::from_env` and its `/tmp` fallback.
  - `crates/observability` (`EnvFilter` from `RUST_LOG`): fold the filter-source field
    into `AppConfig.observability`. Phase 0 consolidates the reader only; JSON/exporter
    behavior is Phase 2. Task 4 wires the actual call site.
  After this task no `*.rs` file outside `crates/config` reads any `DUBBRIDGE_*` variable.
- **Config-directory resolution via `DUBBRIDGE_CONFIG_DIR` (F3 — ADR-026 §2):**
  `load()` resolves the config directory from `std::env::var("DUBBRIDGE_CONFIG_DIR")`,
  defaulting to a `config/` subdirectory located relative to the workspace root (not
  relative to CWD). Tests supply a `DUBBRIDGE_CONFIG_DIR` override pointing to a
  fixture directory so that `cargo test -p dubbridge-config` (CWD = `crates/config/`)
  resolves profiles correctly.
- **Apply the `DATABASE_URL` coexistence rule (F2 — ADR-026 §2):**
  `DATABASE_URL` is a tooling alias only (sqlx-cli, migration scripts). The
  application schema and all tests use `DUBBRIDGE_DATABASE_URL` exclusively. Any test
  in `apps/api/tests/ingestion_test.rs` that currently reads bare `DATABASE_URL`
  migrates to `DUBBRIDGE_DATABASE_URL` in this task.
- Create committed, non-secret profiles: `config/default.toml`, `config/local.toml`,
  `config/staging.toml`, `config/production.toml`, plus `config/README.md` (variable ×
  environment parity table; `DATABASE_URL` alias documented there).
- Move the former in-code defaults (`localhost` Postgres/Redis, `/tmp/dubbridge-storage`,
  `dubbridge-local`) into `config/local.toml`.
- Create `/.env.example` documenting every secret / per-deploy variable.

### Acceptance criteria
- No URL, host, path, bucket, or credential remains as an `unwrap_or(...)` default in
  any `*.rs` file.
- No `*.rs` file outside `crates/config` calls `std::env::var` for a `DUBBRIDGE_*`
  variable (verified by `grep -r 'env::var.*DUBBRIDGE' crates apps --include='*.rs'`
  returning only `crates/config`).
- `DUBBRIDGE_ENV=local` loads `config/local.toml` and yields the historical local
  values; a `DUBBRIDGE_*` env var override (e.g. `DUBBRIDGE_API_PORT=9090`) wins over
  the file.
- Each `config/*.toml` deserializes into `AppConfig` (a test asserts this for all four
  profiles using a fixture-dir override of `DUBBRIDGE_CONFIG_DIR`).
- `cargo test -p dubbridge-config` loads profiles successfully despite the crate-local
  test CWD (verified by a test that confirms the fixture profiles round-trip without
  file-not-found errors).
- No test reads bare `DATABASE_URL`; all DB connection strings in tests come from
  `DUBBRIDGE_DATABASE_URL`.
- `cargo test -p dubbridge-config`, `cargo check --workspace`, and `make qa-deny` pass.

### Files affected
- `crates/config/src/lib.rs`
- `crates/config/Cargo.toml`
- `crates/storage/src/config.rs` (remove standalone `from_env`; fold into `AppConfig`)
- `apps/api/tests/ingestion_test.rs` (migrate `DATABASE_URL` → `DUBBRIDGE_DATABASE_URL`)
- `config/default.toml` `config/local.toml` `config/staging.toml`
  `config/production.toml` `config/README.md` (new)
- `.env.example` (new)

### Status: [ ] Not started

---

## Task 3 — Fail-closed `validate()` + tests

**Effort:** M
**Depends on:** Task 2

### Scope
Implement `AppConfig::validate(&self) -> Result<(), ConfigError>` called at the end of
`load()`. In production-like environments it rejects: `localhost`/`127.0.0.1` database
or Redis URL, `StorageBackend::LocalFs`, `auth.is_none()`, and `LogFormat::Pretty`.

### Acceptance criteria (TDD)
- `production` with a localhost database URL fails validation.
- `production` with `storage.backend = local_fs` fails validation.
- `production` with `auth = None` fails validation (ADR-023).
- `production` with `log_format = pretty` fails validation (ADR-018).
- `local` permits all of the above (local is not production-like).
- The committed `config/production.toml` + a representative secret env set passes
  validation *except* for the storage backend, which is expected to fail until S2
  provides the S3 backend (a test documents this intentional fail-closed gap).
- `cargo test -p dubbridge-config` passes.

### Files affected
- `crates/config/src/lib.rs`

### Status: [ ] Not started

---

## Task 4 — Wire `apps/api` + `apps/worker-runner` to fail-closed `load()`

**Effort:** M
**Depends on:** Task 2, Task 3

### Scope
- Replace `AppConfig::from_env()` with `AppConfig::load()?` in
  `apps/api/src/main.rs` and `apps/worker-runner/src/main.rs`.
- Derive `StorageConfig` from the typed `AppConfig.storage` instead of
  `StorageConfig::from_env`.
- **Consolidate the observability reader (F1 — ADR-026 §3, Phase 0 boundary):**
  Replace the standalone `EnvFilter::try_from_default_env()` call in
  `crates/observability/src/lib.rs` with a parameterized `init_tracing(obs: &ObsSettings)`
  that receives the filter source from `AppConfig.observability`. The call sites in
  `apps/api/src/main.rs` and `apps/worker-runner/src/main.rs` pass `&config.observability`
  after `load()`. This is the **Phase 0 boundary**: the reader is consolidated here;
  JSON format and OTLP exporter behavior remain Phase 2 work (ADR-018). `RUST_LOG`
  may still be supported as a `DUBBRIDGE_OBSERVABILITY_FILTER` env override resolved
  through the existing `DUBBRIDGE_*` layer — not as a direct `EnvFilter` read.
- Add a startup log line printing the resolved `DUBBRIDGE_ENV`, storage backend, and
  log format (extends the existing `tracing::info!` in `apps/api/src/main.rs`).
- Update affected tests (the existing `crates/config` tests for `from_env` are
  replaced by `load()` tests in Tasks 1–3).

### Acceptance criteria
- A missing/invalid `DUBBRIDGE_ENV` makes both binaries exit non-zero with a clear
  error (manually verified; documented in evidence).
- `DUBBRIDGE_ENV=local` boots the API against the local profile as before.
- `crates/observability/src/lib.rs` no longer calls `std::env::var` or
  `EnvFilter::try_from_default_env()` directly; it accepts `ObsSettings` as a
  parameter (verified by `grep -n 'env::var\|from_default_env' crates/observability/src/lib.rs`
  returning no matches).
- The startup log shows the resolved environment and selected backends.
- `cargo check --workspace` and `make qa-test` pass.

### Files affected
- `apps/api/src/main.rs`
- `apps/worker-runner/src/main.rs`
- `crates/observability/src/lib.rs`

### Status: [ ] Not started

---

## Phase 1 — Local infrastructure hygiene

## Task 5 — `infra/local/` reorg; Compose = local-infra-only

**Effort:** M
**Depends on:** Task 4 (so docs describe the new config flow consistently)

### Scope
- Move `infra/docker-compose.yml` → `infra/local/docker-compose.yml`.
- Keep `postgres` / `redis` / `minio` as the default services; move `api` /
  `worker-runner` under an opt-in `app` profile and keep python workers under their
  existing `workers` profile.
- Add a banner comment: this file is local infrastructure only and is never the
  production deployment descriptor (ADR-026).
- **Wire env for the `api` and `worker-runner` app services (F4 — ADR-026 §2):**
  After Phase 0 removes the compiled localhost defaults, the containerized app services
  need explicit env to start. Set for each `app`-profile service:
  - `DUBBRIDGE_ENV=local`
  - `DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@postgres:5432/dubbridge`
    (DNS name `postgres`, not `localhost`)
  - `DUBBRIDGE_REDIS_URL=redis://redis:6379`
    (DNS name `redis`, not `127.0.0.1`)
  - `DUBBRIDGE_STORAGE_BASE_PATH=/tmp/dubbridge-storage`
  - `DUBBRIDGE_STORAGE_BUCKET=dubbridge-local`
  - `DUBBRIDGE_CONFIG_DIR=/workspace/config`
    (workspace bind-mount root; resolves profiles correctly inside the container)
  Auth variables (`DUBBRIDGE_AUTH_ISSUER`, `DUBBRIDGE_AUTH_AUDIENCE`,
  `DUBBRIDGE_AUTH_RSA_PUBLIC_KEY_PATH`) are left as an `env_file: .env` reference —
  they are secrets and must not be hardcoded in the Compose file.
- Update every reference to the old path (`README.md`, `docs/architecture.md`,
  `docs/plan/p0-environment-separation.md`, and any script).

### Acceptance criteria
- `docker compose -f infra/local/docker-compose.yml up -d postgres redis minio`
  starts only infrastructure (no app services).
- `--profile app` is required to start `api` / `worker-runner`.
- With `--profile app`, the `api` service connects to the `postgres` container (DNS
  `postgres:5432`), not to `localhost` (manually verified; documented in evidence).
- No doc references the old `infra/docker-compose.yml` path.
- `make qa-docs` passes.

### Files affected
- `infra/local/docker-compose.yml` (moved; env wiring added)
- `README.md`, `docs/architecture.md`, `docs/plan/p0-environment-separation.md`

### Status: [ ] Not started

---

## Task 6 — Pin Rust image to toolchain policy + secret guard

**Effort:** S
**Depends on:** Task 5

### Scope
- Pin the Compose Rust image tag to the channel in `rust-toolchain.toml` (closes
  X2 / T9).
- Add a guard (Makefile target + CI step) that fails if `config/*.toml` contains
  secret-looking keys (e.g. `password`, `secret`, `token`, `key =` with a value).

### Acceptance criteria
- The Compose Rust image matches `rust-toolchain.toml`.
- The secret guard fails on a deliberately planted secret in a `config/*.toml` fixture
  and passes on the real profiles.
- `make qa-ci` passes locally.

### Files affected
- `infra/local/docker-compose.yml`
- `Makefile`
- `.github/workflows/ci.yml`

### Status: [ ] Not started

---

## Deferred (out of scope for this slice — documented follow-ups)

- **Phase 2 (couples with S2):** wire `build_adapter` (`crates/storage`) to consume
  `storage.backend`, and parameterize `init_tracing` (`crates/observability`) to emit
  JSON + an exporter in production from `observability` settings (ADR-018). Phase 0
  already lands the schema + validation for both fields.
- **Phase 3 (later):** a production deployment descriptor under `infra/deploy/`
  (one image per app + env injection, no orchestration assumed) and the
  secret-manager injection boundary; the owner-credential secret-store decision
  (roadmap X20) and its own ADR.
- **Phase 4 (deferred):** orchestration (Kubernetes/Helm or Nomad), a telemetry
  collector, or a config service — only if multiple live environments or teams
  justify it (ADR-026, Decision 6).

---

## Agent handoff prompt (for delegation)

```
You are implementing P0 of DubBridge — environment separation & fail-closed configuration.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/p0-environment-separation.md
Tasks: docs/tasks/p0-environment-separation.md
Governing ADR: docs/adr/ADR-026-layered-fail-closed-configuration-and-environment-separation.md

Work one task at a time, in order (Tasks 1–6). TDD: write tests first, then implement,
then run them. After each task:
1. Run `cargo check --workspace` and the task's tests.
2. Mark the task [x] in the tasks document and record evidence.
3. Report the result and wait before moving to the next task.

Key invariants:
- DUBBRIDGE_ENV has NO compiled default — missing/unknown must fail closed at startup.
- No URL, host, path, bucket, or credential may remain as an in-code unwrap_or default;
  local values live in config/local.toml.
- One AppConfig schema + validate() is the only configuration reader for api and worker.
- In production-like environments, validate() rejects localhost datastores, local-fs
  storage, absent auth (ADR-023), and pretty logs (ADR-018).
- No secret is ever committed; secrets exist only as injected env vars.
- Docker Compose is local infrastructure only — never the production deployment descriptor.
- Adding the config dependency must pass `make qa-deny`. Do not break `make qa-ci`.
- All communication to the user must be in Spanish.
```
