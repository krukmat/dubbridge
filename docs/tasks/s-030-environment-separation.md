---
type: TaskList
title: "Tasks: S-030 - Environment Separation & Fail-Closed Configuration"
status: closed
slice: S-030
plan: docs/plan/s-030-environment-separation.md
governed_by: [ADR-026]
---
# Tasks: S-030 - Environment Separation & Fail-Closed Configuration

Plan: `docs/plan/s-030-environment-separation.md` · ADR: ADR-026 · Roadmap: S-030, X21, X18, X2

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

### Status: [x] Done

**Evidence (2026-06-03):**
- `AppEnv { Local, Staging, Production }` + `ConfigError { MissingEnv, UnknownEnv, Load, Validation }` added to `crates/config/src/lib.rs`.
- `thiserror = "2"` added to `crates/config/Cargo.toml`.
- 9 new TDD tests cover: missing var → MissingEnv, unknown values ("prod", "") → UnknownEnv, all three valid variants, is_production_like() true/false.
- `cargo test -p dubbridge-config`: **17/17 passed**.
- `make qa-local` (fmt + clippy + test + check): **green**, zero failures across workspace.

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

### Status: [x] Done — completed via `docs/tasks/s-030-t2-layered-loader.md`

**Evidence (2026-06-03):**
- T2-A through T2-F marked `[x]` in `docs/tasks/s-030-t2-layered-loader.md`.
- `crates/config` now owns the layered typed loader (`AppEnv`, `ConfigError`, `AppConfig::load()`) and committed `config/*.toml` profiles.
- `crates/storage` no longer reads `DUBBRIDGE_*`; integration tests now use `DUBBRIDGE_DATABASE_URL`; `/.env.example` documents the injected env contract.
- Verification across sub-tasks: `make qa-deny` green, `cargo test -p dubbridge-config` green, `cargo check --workspace` green, `make qa-local` green, `make qa-docs` green.

Sub-tasks (in order): T2-A (figment dep) → T2-B (schema + TOML profiles) →
T2-C (AppConfig::load()) → T2-D (StorageConfig consolidation) →
T2-E (DATABASE_URL migration) → T2-F (.env.example).
This task is marked [x] only when all six sub-tasks are [x].

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
  validation with `storage.backend = s3`; runtime adapter behavior remains deferred
  to S-080 and later observability wiring.
- `cargo test -p dubbridge-config` passes.

### Files affected
- `crates/config/src/lib.rs`

### Status: [x] Done

**Evidence (2026-06-03):**
- `AppConfig::validate(&self) -> Result<(), ConfigError>` agregado a `crates/config/src/lib.rs` y llamado al final de `AppConfig::load()`.
- Rechazos fail-closed implementados para entornos production-like: `database_url` localhost/`127.0.0.1`, `redis_url` localhost/`127.0.0.1`, `storage.backend = local_fs`, `auth = None`, y `observability.log_format = pretty`.
- 8 tests TDD nuevos en `crates/config/src/lib.rs`: cinco rejection paths de `validate()`, un caso `local` permitido, un caso `load()` que prueba que la validación corre al final, y un caso `production` con secretos representativos que valida correctamente.
- `cargo test -p dubbridge-config`: **34/34 passed**.
- `cargo check --workspace`: **green**.
- `make qa-local` (fmt + clippy + test + check): **green**, workspace completo.

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

### Status: [x] Done

**Evidence (2026-06-03):**
- `apps/api/src/main.rs` y `apps/worker-runner/src/main.rs` migrados de `AppConfig::from_env()` a `AppConfig::load()?`.
- `crates/observability/src/lib.rs` consolidado al reader tipado: `init_tracing(obs: &ObsSettings)` ya no usa `std::env::var` ni `EnvFilter::try_from_default_env()`.
- Startup logs enriquecidos con configuración resuelta: `env`, `storage_backend` y `log_format` en API; `env` y `log_format` en worker runner.
- Verificación manual fail-closed:
  - `cargo run -q -p dubbridge-worker-runner` con `DUBBRIDGE_ENV` ausente → `DUBBRIDGE_ENV is not set...`
  - `cargo run -q -p dubbridge-worker-runner` con `DUBBRIDGE_ENV=qa` → `unrecognised value 'qa'...`
  - `cargo run -q -p dubbridge-api` con `DUBBRIDGE_ENV` ausente → `DUBBRIDGE_ENV is not set...`
  - `cargo run -q -p dubbridge-api` con `DUBBRIDGE_ENV=qa` → `unrecognised value 'qa'...`
- Verificación manual local:
  - `cargo run -q -p dubbridge-worker-runner` con `DUBBRIDGE_ENV=local` arranca y emite el startup log con `env=Local` y `log_format=Pretty`.
  - `cargo run -q -p dubbridge-api` con `DUBBRIDGE_ENV=local` y auth inyectado progresa más allá de `load()`/`init_tracing()` y falla después al inicializar el verifier por falta del archivo de clave, no por configuración.
- `rg -n 'env::var|from_default_env' crates/observability/src/lib.rs` devuelve cero matches.
- `cargo check --workspace`: **green**.
- `make qa-test`: **green**.

---

## Phase 1 — Local infrastructure hygiene

## Task 5 — `infra/local/` reorg; Compose = local-infra-only

**Effort:** M
**Depends on:** Task 4 (so docs describe the new config flow consistently)

### Scope
- Relocate the local Compose file to `infra/local/docker-compose.yml`.
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
  `docs/plan/s-030-environment-separation.md`, and any script).

### Acceptance criteria
- `docker compose -f infra/local/docker-compose.yml up -d postgres redis minio`
  starts only infrastructure (no app services).
- `--profile app` is required to start `api` / `worker-runner`.
- With `--profile app`, the `api` service connects to the `postgres` container (DNS
  `postgres:5432`), not to `localhost` (manually verified; documented in evidence).
- No doc references the pre-move root-level Compose path.
- `make qa-docs` passes.

### Files affected
- `infra/local/docker-compose.yml` (moved; env wiring added)
- `README.md`, `docs/architecture.md`, `docs/plan/s-030-environment-separation.md`

### Status: [x] Done

**Evidence (2026-06-03):**
- Local Compose moved to `infra/local/docker-compose.yml` with a top-of-file non-production banner.
- `api` and `worker-runner` now sit behind `profiles: ["app"]`; default `docker compose -f infra/local/docker-compose.yml config` renders only `postgres`, `redis`, and `minio`.
- Both app-profile services now receive explicit local env wiring: `DUBBRIDGE_ENV=local`, container-DNS datastore URLs (`postgres`, `redis`), local storage settings, and `DUBBRIDGE_CONFIG_DIR=/workspace/config`.
- Auth remains injected, not hardcoded: the Compose file reads local `.env` when present and maps auth values into the typed config keys consumed by `AppConfig::load()`.
- `docker compose -f infra/local/docker-compose.yml up -d postgres redis minio` was attempted directly, but this host already had port `5432` allocated; the failure was environmental, not due to Compose selection.
- Manual isolated verification without published host ports:
  - `docker compose -f infra/local/docker-compose.yml --profile app config --services` includes `api` and `worker-runner`, proving `--profile app` is required for app containers.
  - Inside the `api` service container, `DUBBRIDGE_DATABASE_URL` resolved to `postgres://dubbridge:dubbridge@postgres:5432/dubbridge`, `getent hosts postgres` resolved successfully, and a TCP open to `postgres:5432` returned `tcp-ok`.
- Documentation updated to the new path; repo-wide search for the pre-move root-level Compose path now returns zero matches.

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

### Status: [x] Done

**Evidence (2026-06-03):**
- `infra/local/docker-compose.yml` now pins both Rust app containers to `image: rust:stable`, matching `rust-toolchain.toml`.
- Added `scripts/check-config-secrets.sh` plus `make qa-config-secrets`; the target passes on committed `config/*.toml` profiles and fails on a deliberately planted fixture containing `client_secret = "super-secret"`.
- Added a dedicated `config-secrets` job to `.github/workflows/ci.yml`, and folded the guard into `make qa-ci`.
- `docker compose -f infra/local/docker-compose.yml config` is valid with the updated image tags.
- Local verification:
  - `make qa-config-secrets` → passes on real profiles.
  - `bash scripts/check-config-secrets.sh /tmp/dubbridge-task6-fixture/bad.toml` → fails with `Secret-looking config key found ... client_secret`.
  - `make qa-ci` → passes locally.

With Task 6 complete, the current S-030 task ledger (Tasks 1-6) is complete for the
Phase 0 / Phase 1 scope defined in this slice.

---

## Deferred (out of scope for this slice — documented follow-ups)

- **Phase 2 (couples with S-080):** wire `build_adapter` (`crates/storage`) to consume
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
You are implementing S-030 of DubBridge — environment separation & fail-closed configuration.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/s-030-environment-separation.md
Tasks: docs/tasks/s-030-environment-separation.md
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
