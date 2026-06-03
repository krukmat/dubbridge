# Adjustment Plan: P0 Configuration Review — Pre-Implementation Findings

## Purpose

This document records a **read-only review of already-executed work** (slices S0, S1,
T1, H1, and the in-progress S3 foundation) against ADR-026 and the P0 plan, performed
**before P0 implementation starts**. Its goal is to catch anything in delivered code,
tests, or infrastructure that must be adjusted so the P0 environment-separation slice
leaves no loose ends.

This document changes no production code. It defines a small set of **adjustment
tasks (AJ1–AJ5)** that refine the existing P0 plan and ledger
(`docs/plan/p0-environment-separation.md`, `docs/tasks/p0-environment-separation.md`)
so the findings below become explicit scope and acceptance criteria there.

- Governing ADR: ADR-026 (layered fail-closed configuration & environment separation).
- Related: roadmap P0, X18, X21, X2; ADR-008, ADR-018, ADR-023, ADR-025.

## Scope of the review

Areas inspected (read-only):

- `crates/config`, `crates/storage`, `crates/observability` — configuration readers.
- `apps/api`, `apps/worker-runner` — startup wiring.
- `apps/api/tests/ingestion_test.rs` — test environment variables.
- `crates/connectors`, `crates/domain/src/platform_ingest.rs` — in-progress S3 work.
- `infra/local/docker-compose.yml`, `Makefile`, `.githooks/pre-push`, `.github/workflows/ci.yml`.

Verified facts that bound the findings:

- No `sqlx::query!` / `query_as!` compile-time macros are used → there is **no
  build-time `DATABASE_URL` dependency**; `DATABASE_URL` appears only at test runtime.
- `crates/observability` currently depends on `tracing` + `tracing-subscriber` only
  (no JSON/OTLP features yet) → JSON + exporter is net-new Phase 2 work.

## Findings

| ID | Finding | Evidence | Verdict |
|----|---------|----------|---------|
| **F1** | **Three scattered environment readers, not one.** Configuration is read independently in `crates/config` (`AppConfig` + `AuthSettings`), `crates/storage/src/config.rs` (`DUBBRIDGE_STORAGE_BASE_PATH` / `DUBBRIDGE_STORAGE_ENDPOINT`), and `crates/observability` (`EnvFilter` from `RUST_LOG`). | `crates/config/src/lib.rs:24-54`, `crates/storage/src/config.rs:18-20`, `crates/observability/src/lib.rs` | Contradicts ADR-026 §3 ("one schema, one reader"). The P0 plan consolidates storage but does not yet name the observability reader. Adjust (AJ1, AJ2, AJ3). |
| **F2** | **Env-var naming inconsistency.** Integration tests read `DATABASE_URL`; the application reads `DUBBRIDGE_DATABASE_URL`. | `apps/api/tests/ingestion_test.rs:57`; app readers in `crates/config/src/lib.rs:28` | No build impact (no `query!` macros). P0's `DUBBRIDGE_` grammar must decide how the sqlx-cli convention `DATABASE_URL` coexists. Adjust (AJ1, AJ2). |
| **F3** | **`config/` directory resolution breaks under `cargo test`.** A layered loader using `figment::Toml::file("config/<env>.toml")` resolves relative to the current working directory. `cargo test -p dubbridge-config` runs with CWD = `crates/config/` (no `config/` there), and the runtime binary has a different CWD again. | Design risk for `docs/tasks/p0-environment-separation.md` Task 2 | Real implementation risk. Resolution must be explicit (e.g. `DUBBRIDGE_CONFIG_DIR`, or workspace-root detection). Adjust (AJ1, AJ2). |
| **F4** | **Compose `api` / `worker-runner` services have no env wiring.** They run `cargo run` with no `DUBBRIDGE_ENV` and no URLs pointing at the `postgres` service DNS; today they "work" only via the compiled localhost defaults — which inside a container point at the container itself, not at `postgres`, so they are already broken for DB. | local Compose `api` / `worker-runner` blocks | After P0 removes defaults and requires `DUBBRIDGE_ENV`, these services fail to start. Task 5 must **wire** their env, not only move + profile + banner. Adjust (AJ4). |
| **F5** | **The 90% coverage gate covers the new config code.** `COVERAGE_IGNORE_REGEX` ignores `crates/(db|jobs|observability)/src/lib.rs` and the app `main.rs` files, but **not** `crates/config`. | `Makefile` (`COVERAGE_IGNORE_REGEX`) | Desirable, not a conflict. `load()` / `validate()` / `AppEnv` must reach ≥90% line coverage; the Task 4 `main.rs` wiring is already ignored. Record as a constraint (AJ1). |
| **F6** | **In-progress S3 work reads no env/secrets.** `crates/connectors` depends only on `dubbridge-domain`; `platform_ingest.rs` holds the domain aggregate with `RightsBasis`, no env or secret access. | `crates/connectors/Cargo.toml`, `crates/domain/src/platform_ingest.rs` | Clean today. When S3-P1 adds owner-credential handling (X20), it must read secrets through the injected-env layer (ADR-026 §4), never from committed profiles. Record the constraint (AJ5). |

## Adjustment tasks

### Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

All adjustment tasks edit **documentation only** (the P0 plan and ledger). No
production code, test, or infra file is modified by these tasks; their findings become
scope/acceptance in the P0 implementation tasks they target.

---

#### AJ1 — P0 plan: add the four design decisions

**Effort:** S
**Target:** `docs/plan/p0-environment-separation.md` (Design Decisions)
**Depends on:** nothing
**Covers:** F1, F2, F3, F5
**Status: [x] DONE — 2026-06-03. Four design decisions added to the P0 plan Design Decisions section; Module Dependencies note updated to reflect three-reader-today vs one-reader-after state.**

##### Scope
Add design-decision entries:
- **Single reader = consolidate three readers today** (F1): name `crates/config`,
  `crates/storage/src/config.rs`, and the `crates/observability` `EnvFilter` as the
  readers that must fold into `AppConfig`.
- **Config-directory resolution** (F3): the loader resolves its config directory via an
  explicit `DUBBRIDGE_CONFIG_DIR` (default to a workspace-root-relative `config/`),
  because `cargo test` CWD ≠ runtime CWD.
- **`DATABASE_URL` coexistence** (F2): state the chosen rule — keep `DATABASE_URL` as a
  sqlx-cli/tooling alias for migrations, while the application and its tests read
  `DUBBRIDGE_DATABASE_URL` (or the reverse, if decided otherwise).
- **Coverage constraint** (F5): note that `crates/config` is not in the coverage ignore
  list, so the new loader/validator must reach the 90% gate.

##### Acceptance criteria
- The four decisions appear in the P0 plan's Design Decisions section.
- Each references its finding ID and the relevant ADR-026 section.

##### Status: [x] Done — 2026-06-03

---

#### AJ2 — P0 ledger Task 2: enumerate readers + config-dir + `DATABASE_URL`

**Effort:** S
**Target:** `docs/tasks/p0-environment-separation.md` (Task 2)
**Depends on:** AJ1
**Covers:** F1, F2, F3

##### Scope
Extend Task 2 scope and acceptance criteria to:
- Explicitly consolidate all three readers (config, storage, observability filter)
  into `AppConfig`.
- Resolve the config directory via `DUBBRIDGE_CONFIG_DIR` and prove it works from both
  the workspace root and a crate-local `cargo test` CWD.
- Implement and document the `DATABASE_URL` ↔ `DUBBRIDGE_DATABASE_URL` rule from AJ1.

##### Acceptance criteria
- Task 2 lists the three readers by path.
- Task 2 has an acceptance criterion that `cargo test -p dubbridge-config` loads
  `config/*.toml` successfully despite the crate-local test CWD.
- Task 2 records the `DATABASE_URL` decision as an acceptance criterion.

##### Status: [x] Done — 2026-06-03. Task 2 scope extended with three-reader consolidation (paths explicit), DUBBRIDGE_CONFIG_DIR resolution + CWD acceptance criterion, and DATABASE_URL coexistence rule. Acceptance criteria updated accordingly.

---

#### AJ3 — P0 ledger Task 4: consolidate the observability filter

**Effort:** S
**Target:** `docs/tasks/p0-environment-separation.md` (Task 4)
**Depends on:** AJ1
**Covers:** F1

##### Scope
Extend Task 4 to route the `crates/observability` log configuration through
`AppConfig` (replacing the standalone `EnvFilter`-from-`RUST_LOG` read), keeping the
JSON/exporter behavior itself deferred to Phase 2 but the **reader** consolidated now.

##### Acceptance criteria
- Task 4 scope names the observability reader consolidation.
- Task 4 clarifies that Phase 0 consolidates the reader; Phase 2 adds JSON/exporter
  behavior (ADR-018).

##### Status: [x] Done — 2026-06-03. Task 4 scope extended with observability reader consolidation (Phase 0 boundary explicit: reader now, JSON/exporter Phase 2). Acceptance criterion added: grep for direct env reads in crates/observability must return no matches. Files affected updated.

---

#### AJ4 — P0 ledger Task 5: wire compose app-service env

**Effort:** S
**Target:** `docs/tasks/p0-environment-separation.md` (Task 5)
**Depends on:** AJ1
**Covers:** F4

##### Scope
Extend Task 5 beyond move + profile + banner to **wire the environment** of the
`api` / `worker-runner` compose services: set `DUBBRIDGE_ENV=local` and the
service-DNS URLs (`postgres`, `redis`, `minio`) so the containerized app actually
starts after the compiled defaults are removed.

##### Acceptance criteria
- Task 5 scope includes setting `DUBBRIDGE_ENV` and service-DNS connection URLs for the
  app services.
- Task 5 has an acceptance criterion that `--profile app` brings up `api` against the
  `postgres` service (not `localhost`).

##### Status: [x] Done — 2026-06-03. Task 5 scope extended with explicit env wiring for api/worker-runner compose services (DUBBRIDGE_ENV, service-DNS URLs, DUBBRIDGE_CONFIG_DIR). Auth vars delegated to env_file to keep secrets out of the compose file. Acceptance criterion added: --profile app connects to postgres DNS, not localhost.

---

#### AJ5 — Record the S3 owner-credential secret-layer constraint

**Effort:** S
**Target:** `docs/plan/p0-environment-separation.md` (Design Decisions — secret/config split)
**Depends on:** nothing
**Covers:** F6

##### Scope
Reinforce the secret/config-split decision: future S3-P1 owner-credential handling
(roadmap X20, ADR-025) must read credentials through the **injected-secret layer**
(ADR-026 §4), never from committed `config/*.toml` profiles.

##### Acceptance criteria
- The P0 plan's secret/config-split decision explicitly states the X20 constraint and
  references ADR-026 §4 and ADR-025.

##### Status: [x] Done — 2026-06-03. Secret/config-split decision in the P0 plan extended with explicit X20/ADR-025 constraint: owner credentials must arrive via the injected-secret layer (ADR-026 §4), never from committed profiles; actual token redacted per ADR-018/ADR-025; P0 establishes the layer, S3-P1 decides the store mechanism.

---

## Relationship to P0 implementation

These adjustments do not add new P0 implementation tasks; they sharpen existing ones.
After AJ1–AJ5 are applied, the findings are enforced as P0 acceptance criteria:

| Finding | Enforced in P0 task |
|---------|---------------------|
| F1 (three readers) | Task 2 (config + storage), Task 4 (observability) |
| F2 (`DATABASE_URL`) | Task 2 |
| F3 (config-dir CWD) | Task 2 |
| F4 (compose env) | Task 5 |
| F5 (coverage) | Tasks 1–3 (config crate tests) |
| F6 (S3 secret layer) | P0 plan decision; enforced later in S3-P1 |

## Agent handoff prompt (for delegation)

```
You are applying the P0 configuration-review adjustments for DubBridge.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
This doc: docs/plan/p0-config-review-adjustments.md
Targets: docs/plan/p0-environment-separation.md, docs/tasks/p0-environment-separation.md
Governing ADR: docs/adr/ADR-026-layered-fail-closed-configuration-and-environment-separation.md

Apply AJ1–AJ5 in order. Each edits documentation only (the P0 plan/ledger) — do not
touch production code, tests, or infra in these tasks. After each:
1. Mark the task [x] in this document.
2. Run `make qa-docs` (must stay green; cite only existing ADRs).
3. Report the result before the next task.

Key invariants the adjustments encode:
- One AppConfig reader; consolidate crates/config + crates/storage + crates/observability.
- Config dir resolved via DUBBRIDGE_CONFIG_DIR (cargo test CWD != runtime CWD).
- DATABASE_URL is a tooling alias; app/tests use DUBBRIDGE_DATABASE_URL (per AJ1 decision).
- Compose app services must set DUBBRIDGE_ENV + service-DNS URLs, not rely on defaults.
- S3 owner credentials use the injected-secret layer (ADR-026 §4), never committed profiles.
- All communication to the user must be in Spanish.
```
