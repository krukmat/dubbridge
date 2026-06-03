# Tasks: P0-T2 — Layered Loader (subdivided)

Parent task: `docs/tasks/p0-environment-separation.md` § Task 2
Plan: `docs/plan/p0-environment-separation.md`
ADR: ADR-026 — Decisions 2, 3, 4

## Why this file exists

Task 2 of P0 is an L-effort task that touches six distinct files across four crates.
Executing it as one block makes review and recovery hard. This document breaks it into
six focused sub-tasks that can each be approved, implemented, tested, and marked done
independently.

Each sub-task has its own acceptance criteria and evidence field. The parent task in
`p0-environment-separation.md` is marked [x] only after all six sub-tasks below are [x].

## Status legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

## Dependency graph

```
T2-A  (figment dep + qa-deny)
  └─> T2-B  (schema + TOML profiles)
        └─> T2-C  (AppConfig::load())
              ├─> T2-D  (StorageConfig consolidation)
              ├─> T2-E  (DATABASE_URL migration)
              └─> T2-F  (.env.example)
```

All sub-tasks must pass before closing parent Task 2.

---

## T2-A — Add `figment` dependency and verify `make qa-deny`

**Effort:** S  
**Complexity:** Low  
**Depends on:** Task 1 (done)

### Context
`figment` is the layered-config library chosen in ADR-026 § "Layered resolution".
Before writing any loader code, the dependency must be in `Cargo.toml` and must pass
the license/ban/advisory audit (`make qa-deny`). Adding a dependency that fails the
audit blocks every downstream sub-task, so this is isolated as the first step.

`figment` uses feature flags: `toml` (for TOML file merging) and `env` (for
`DUBBRIDGE_*` env var layer). Both must be listed explicitly — the crate does not
enable them by default.

### Files affected
- `crates/config/Cargo.toml`

### Acceptance criteria
- `figment` with features `["toml", "env"]` appears in `[dependencies]`.
- `make qa-deny` passes (no license, ban, or advisory violation).
- `cargo check -p dubbridge-config` passes.

### Status: [x] Done

**Evidence (2026-06-03):**
- `figment = { version = "0.10", features = ["toml", "env"] }` agregado a `crates/config/Cargo.toml`.
- `cargo check -p dubbridge-config`: green (15 nuevos paquetes resueltos y compilados).
- `make qa-deny`: `advisories ok, bans ok, licenses ok, sources ok`.

---

## T2-B — Typed schema + committed TOML profiles

**Effort:** M  
**Complexity:** Medium  
**Depends on:** T2-A

### Context
`AppConfig` currently holds a flat list of scalar fields (`api_port`, `database_url`,
etc.) read by `from_env()`. Task 2 replaces this with a schema that `figment` can
deserialize from layered TOML + env. The schema must:

- Group storage settings under `StorageSettings { backend: StorageBackend, base_path,
  bucket, endpoint_url }` so that the `StorageConfig` standalone reader can be
  replaced (T2-D).
- Group observability under `ObsSettings { log_format: LogFormat, filter }` so that
  Phase 0 consolidates the reader without changing behavior (behavior changes in
  Phase 2).
- Keep `auth: Option<AuthSettings>` unchanged (already typed).
- Add `env: AppEnv` as the first field so the loaded config carries its own identity.

Alongside the schema, four committed non-secret TOML profiles must be created under
`config/`. The former `localhost`/`/tmp`/`dubbridge-local` in-code defaults move
**entirely** into `config/local.toml` — they must not exist anywhere in `*.rs` after
this sub-task.

`StorageBackend` and `LogFormat` are new enums whose values map to TOML strings:
`"local_fs"` / `"s3"` and `"pretty"` / `"json"`. They need custom `Deserialize`
so figment can parse them from TOML strings.

`config/README.md` documents which variables live in TOML vs which arrive as env
overrides, and includes the `DATABASE_URL` alias rule (F2, ADR-026 §2).

### Files affected
- `crates/config/src/lib.rs` (schema structs + enums)
- `config/default.toml` (new)
- `config/local.toml` (new — receives all former in-code localhost defaults)
- `config/staging.toml` (new)
- `config/production.toml` (new)
- `config/README.md` (new)

### Acceptance criteria
- `AppConfig`, `StorageSettings`, `StorageBackend`, `ObsSettings`, `LogFormat` compile
  without errors.
- Each of the four TOML profiles deserializes into `AppConfig` via a unit test that
  supplies a fixture `DUBBRIDGE_CONFIG_DIR` pointing to `config/` (so the test works
  regardless of CWD).
- No `unwrap_or("localhost…")`, `unwrap_or("/tmp…")`, or `unwrap_or("dubbridge-local")`
  remains in any `*.rs` file.
- `cargo check -p dubbridge-config` passes.

### Status: [x] Done

**Evidence (2026-06-03):**
- `AppEnv`, `StorageBackend`, `LogFormat`, `StorageSettings`, `ObsSettings` agregados a `crates/config/src/lib.rs`.
- `AppConfig` expandido con `env`, `storage`, `observability`; campo plano `storage_bucket` eliminado.
- `from_env()` mantenido (legacy, se elimina en T4); actualizado para usar los nuevos sub-structs.
- `config/default.toml`, `config/local.toml`, `config/staging.toml`, `config/production.toml`, `config/README.md` creados.
- `apps/cli/src/main.rs` y `apps/api/src/main.rs` actualizados de `storage_bucket` → `storage.bucket`.
- 3 nuevos tests TDD de round-trip por perfil: `schema_local_profile_deserializes`, `schema_staging_profile_deserializes`, `schema_production_profile_deserializes` — todos pasan.
- `cargo test -p dubbridge-config`: **20/20 passed**.
- `make qa-local` (fmt + clippy + test + check): **green**, workspace completo.

---

## T2-C — `AppConfig::load()` layered implementation + TDD tests

**Effort:** M  
**Complexity:** Medium  
**Depends on:** T2-B

### Context
This is the core loader: `AppConfig::load() -> Result<Self, ConfigError>`.

Resolution order (ADR-026, Decision 2):
```
code defaults ← config/default.toml ← config/<env>.toml ← DUBBRIDGE_* env vars
```

The loader must:
1. Call `AppEnv::from_process()` first — fail-closed on missing/unknown `DUBBRIDGE_ENV`.
2. Resolve the config directory from `DUBBRIDGE_CONFIG_DIR` env var; default to
   a `config/` path relative to the workspace root (derived from
   `CARGO_MANIFEST_DIR` at compile time or from the binary's location at runtime).
   Tests supply `DUBBRIDGE_CONFIG_DIR` pointing to a fixture directory so they work
   from any CWD (ADR-026 §2, F3).
3. Merge the three layers with figment; map figment's error to `ConfigError::Load`.
4. Keep `AppConfig::from_env()` compiling but deprecated (it is removed in Task 4
   after the wire-up, not here — removing it here would break the API and worker
   before Task 4 is ready).

TDD: write the tests first, then implement.

Tests must cover:
- Missing `DUBBRIDGE_ENV` → `ConfigError::MissingEnv` (never falls back).
- Unknown `DUBBRIDGE_ENV` value → `ConfigError::UnknownEnv`.
- `DUBBRIDGE_ENV=local` → loads `local.toml` values (api_port, database_url, etc.).
- `DUBBRIDGE_ENV=staging` → loads `staging.toml` values.
- A `DUBBRIDGE_*` env var override (e.g. `DUBBRIDGE_API_PORT=9090`) wins over the TOML
  file value.
- Missing config file (bad `DUBBRIDGE_CONFIG_DIR`) → `ConfigError::Load`.

### Files affected
- `crates/config/src/lib.rs` (add `AppConfig::load()`, tests)

### Acceptance criteria
- All tests listed above pass under `cargo test -p dubbridge-config`.
- `from_env()` still compiles (not removed yet).
- No test reads env vars without `temp_env` isolation.
- `cargo check --workspace` passes.

### Status: [x] Done

**Evidence (2026-06-03):**
- `AppConfig::load()` agregado a `crates/config/src/lib.rs`; llama primero a `AppEnv::from_process()`, resuelve `DUBBRIDGE_CONFIG_DIR` o deriva `../../config` desde `CARGO_MANIFEST_DIR`, y mergea `default.toml` → `<env>.toml` → `Env::prefixed("DUBBRIDGE_").split("__")`.
- Errores de extracción de `figment` mapeados a `ConfigError::Load(e.to_string())`; `from_env()` se mantiene compilando sin cambios de contrato.
- 6 tests TDD nuevos para T2-C en `crates/config/src/lib.rs`: missing env, unknown env, `local`, `staging` con secretos inyectados, override `DUBBRIDGE_API_PORT`, y `DUBBRIDGE_CONFIG_DIR` inválido.
- `cargo test -p dubbridge-config`: **26/26 passed**.
- `cargo check --workspace`: **green**.
- `make qa-local` (fmt + clippy + test + check): **green**, workspace completo.

---

## T2-D — Consolidate `StorageConfig` into `AppConfig`

**Effort:** M  
**Complexity:** Medium  
**Depends on:** T2-C

### Context
Today `crates/storage/src/config.rs` has a standalone `StorageConfig::from_env()` that
reads `DUBBRIDGE_STORAGE_BASE_PATH` and `DUBBRIDGE_STORAGE_ENDPOINT` with an in-code
`/tmp` default. This violates ADR-026 §3 (one reader) and is a second source of
fail-open defaults.

After T2-B, `AppConfig` already has `storage: StorageSettings`. This sub-task:
1. Removes `StorageConfig::from_env()` from `crates/storage/src/config.rs`.
2. Makes `StorageConfig` constructible from `StorageSettings` (add a `From<&StorageSettings>`
   impl or adjust the fields to match directly).
3. Updates `apps/api/src/main.rs` to derive `StorageConfig` from `config.storage`
   instead of calling `StorageConfig::from_env(&config.storage_bucket)`.
4. Deletes the old `storage_bucket` field from `AppConfig` (it is now
   `config.storage.bucket`).
5. Updates or removes the tests in `crates/storage/src/config.rs` that tested the
   old `from_env()` method.

After this sub-task, no `.rs` file outside `crates/config` reads any `DUBBRIDGE_*`
variable — the grep `grep -r 'env::var.*DUBBRIDGE' crates apps --include='*.rs'`
returns only `crates/config`.

### Files affected
- `crates/storage/src/config.rs` (remove `from_env`; add conversion from `StorageSettings`)
- `apps/api/src/main.rs` (derive `StorageConfig` from typed settings)
- `crates/config/src/lib.rs` (remove `storage_bucket` flat field if still present)

### Acceptance criteria
- `grep -r 'env::var.*DUBBRIDGE' crates apps --include='*.rs'` returns only
  `crates/config` lines.
- No `unwrap_or("/tmp…")` remains in `crates/storage/src/config.rs`.
- `cargo test --workspace` passes (storage tests updated accordingly).
- `cargo check --workspace` passes.

### Status: [x] Done

**Evidence (2026-06-03):**
- `StorageConfig::from_env()` eliminado de `crates/storage/src/config.rs`; `StorageConfig` ahora implementa `From<&dubbridge_config::StorageSettings>`.
- `apps/api/src/main.rs` actualizado para derivar `StorageConfig` desde `&config.storage` en lugar de releer `DUBBRIDGE_*`.
- Tests de `crates/storage/src/config.rs` reemplazados por cobertura TDD de conversión tipada: copia de `base_path`, `bucket`, `endpoint_url`, y preservación de `None`.
- `rg -n "env::var\\(.*DUBBRIDGE" crates apps --glob '*.rs'` devuelve solo líneas en `crates/config/src/lib.rs`.
- `cargo test -p dubbridge-storage`: **13/13 passed**.
- `cargo check --workspace`: **green**.
- `make qa-local` (fmt + clippy + test + check): **green**, workspace completo.

---

## T2-E — Migrate integration tests from `DATABASE_URL` to `DUBBRIDGE_DATABASE_URL`

**Effort:** S  
**Complexity:** Low  
**Depends on:** T2-C

### Context
`apps/api/tests/ingestion_test.rs` reads bare `DATABASE_URL` (sqlx-cli convention).
ADR-026 §2 (F2) declares `DATABASE_URL` a **tooling alias only** — the application
and all its tests must use `DUBBRIDGE_DATABASE_URL` as the single authoritative name.

There are ~14 occurrences of `DATABASE_URL` in the test file (one read + many skip
guards). All must change to `DUBBRIDGE_DATABASE_URL`. The variable name in the local
`.env` / CI environment must also be set accordingly (document in `.env.example`,
which is T2-F).

This is intentionally isolated so the env-var rename does not get mixed with schema
or loader work.

### Files affected
- `apps/api/tests/ingestion_test.rs`

### Acceptance criteria
- `grep 'DATABASE_URL' apps/api/tests/ingestion_test.rs` returns zero matches for
  bare `DATABASE_URL` (only `DUBBRIDGE_DATABASE_URL`).
- Integration tests still pass when `DUBBRIDGE_DATABASE_URL` is set in the environment
  (CI or local `.env`).
- `cargo test --workspace` passes (tests skip gracefully when the var is absent, as
  before).

### Status: [x] Done

**Evidence (2026-06-03):**
- `apps/api/tests/ingestion_test.rs` migrado de `DATABASE_URL` a `DUBBRIDGE_DATABASE_URL` tanto en el `env::var(...)` principal como en todos los skip guards.
- `rg -n "\\bDATABASE_URL\\b" apps/api/tests/ingestion_test.rs` devuelve cero matches; el archivo solo contiene `DUBBRIDGE_DATABASE_URL`.
- `make qa-local` (fmt + clippy + test + check): **green**, incluyendo `apps/api/tests/ingestion_test.rs` con **14/14 passed**.

---

## T2-F — Create `.env.example`

**Effort:** S  
**Complexity:** Low  
**Depends on:** T2-D, T2-E

### Context
After T2-D and T2-E, the complete set of injected variables (secrets and per-deploy
values not in TOML) is known and stable. `.env.example` documents every variable a
developer or deploy pipeline must supply, with placeholder values and a comment
explaining each one.

Variables that already live in `config/*.toml` (non-secrets) are listed with a note
saying "override via env if needed" but are not required for local dev.
Secrets (`DUBBRIDGE_AUTH_*`, `DUBBRIDGE_DATABASE_URL`, `DUBBRIDGE_REDIS_URL` if
per-deploy) are listed as required with `<REPLACE_ME>` placeholders.

`DUBBRIDGE_CONFIG_DIR` is documented as optional (defaults to `config/` relative to
the workspace root).

### Files affected
- `.env.example` (new, at repository root)

### Acceptance criteria
- `.env.example` exists at repository root.
- Every `DUBBRIDGE_*` variable read anywhere in the codebase appears in the file.
- No actual secret value appears — only `<REPLACE_ME>` or `<path/to/key.pem>`.
- `make qa-docs` passes (doc-consistency script does not flag the new file).

### Status: [x] Done

**Evidence (2026-06-03):**
- `/.env.example` creado en la raíz del repo con todas las variables `DUBBRIDGE_*` leídas por el código: `DUBBRIDGE_ENV`, `DUBBRIDGE_DATABASE_URL`, `DUBBRIDGE_REDIS_URL`, `DUBBRIDGE_AUTH_*`, `DUBBRIDGE_CONFIG_DIR`, `DUBBRIDGE_API_PORT`, `DUBBRIDGE_WORKER_CONCURRENCY`, `DUBBRIDGE_STORAGE_BASE_PATH`, `DUBBRIDGE_STORAGE_BUCKET`, `DUBBRIDGE_STORAGE_ENDPOINT`.
- El archivo usa solo placeholders y ejemplos no sensibles (`<REPLACE_ME>`, `<path/to/key.pem>`) y omite `DATABASE_URL` por ser alias de tooling, no variable de aplicación.
- Verificación de cobertura: `rg -o "DUBBRIDGE_[A-Z0-9_]+" crates apps --glob '*.rs' | sed 's/.*://' | sort -u` queda cubierto por `rg -o "DUBBRIDGE_[A-Z0-9_]+" .env.example | sort -u` (ignorando el separador de comentario `DUBBRIDGE__`).
- `make qa-docs`: **green** (`Documentation consistency check passed`).

---

## Agent handoff prompt (for delegation)

```
You are implementing P0-T2 of DubBridge — layered fail-closed configuration loader.
Work one sub-task at a time in this order: T2-A → T2-B → T2-C → T2-D → T2-E → T2-F.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Sub-task document: docs/tasks/p0-t2-layered-loader.md
Parent tasks: docs/tasks/p0-environment-separation.md § Task 2
Plan: docs/plan/p0-environment-separation.md
ADR: docs/adr/ADR-026-layered-fail-closed-configuration-and-environment-separation.md

Key invariants:
- TDD: write tests first, then implement.
- DUBBRIDGE_ENV has NO compiled default — from_process() must fail-closed (already done in Task 1).
- No URL, host, path, bucket, or credential may remain as an in-code unwrap_or default.
- No .rs file outside crates/config reads any DUBBRIDGE_* variable after T2-D.
- Tests must use temp_env for env isolation; DUBBRIDGE_CONFIG_DIR for config-dir isolation.
- from_env() must keep compiling until Task 4 removes it.
- All communication to the user must be in Spanish.

After each sub-task: run cargo test --workspace (or make qa-local), mark it [x] in
docs/tasks/p0-t2-layered-loader.md with evidence, report in Spanish, and wait for approval.
```
