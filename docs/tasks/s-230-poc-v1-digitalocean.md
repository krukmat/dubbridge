---
type: TaskList
title: "S-230 POC v1 Deployment (Digital Ocean)"
status: planned
slice: S-230
plan: docs/plan/s-230-poc-v1-digitalocean.md
---
# S-230 — POC v1 Deployment (Digital Ocean)

Ordered task ledger for the 10-day POC deployment window. Rationale, verified gap
analysis, architecture, calendar, and risks live in
`docs/plan/s-230-poc-v1-digitalocean.md`; this file is the crash-safe progress
ledger.

## Slice execution contract

- Product scope is frozen: subtitles + governed review/publication. `S-150` is
  out of scope for this slice and is not reopened by any task here.
- No new application technology. PostgreSQL, Redis, S3-compatible storage,
  ffmpeg, and Python/faster-whisper are all pre-existing dependencies.
- Every RRI must be computed with `scripts/rri.py` before the task is presented;
  the provisional efforts below are planning estimates, not scores.
- T1, T1b, T2, T3, T7b, T7c and T8 are development tasks and carry the full
  closure checklist (band-routed review, Reflection where the band requires it,
  unit coverage certification, owner verification). T4, T5, T6 are
  config/ops-shaped; their exemption status is decided per task at presentation
  time, not assumed here.
- Tasks touching `mobile/` (T7, T7b, T7c) must read the root `DESIGN.md` before
  planning or implementation, per the workflow guide's Analyze step.
- Task order is a dependency order, not a suggestion. T4 must not start while any
  of T1–T3 is open.

## Task index

| ID | Title | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| T0 | Slice plan, ledger, and roadmap entry | docs-only | S | — | [x] Done |
| T1 | S3/Spaces credential and region wiring | development | M | T0 | [ ] Planned |
| T1b | API preparation queue bound to Redis | development | M | T0 | [ ] Planned |
| T2 | Migration runner in the production path | development | M | T0 | [ ] Planned |
| T3 | Real readiness probes for api and gateway | development | M | T0 | [ ] Planned |
| T4 | Production container images | config/dev | M | T1, T1b, T2, T3 | [ ] Planned |
| T5 | Production deployment descriptor and secret boundary | config-only | M | T4 | [ ] Planned |
| T6 | First deploy and end-to-end smoke on Digital Ocean | operational | L | T5 | [ ] Planned |
| T7 | Mobile POC build against the deployed backend | development/ops | M | T6 | [ ] Planned |
| T7b | Mobile registration screen | development | M | T7 | [ ] Planned — droppable (first) |
| T7c | Session lifetime and expiry behavior | development/config | S | T7 | [ ] Planned |
| T8 | Subtitle visible in the review surface (optional) | development | M | T6 | [ ] Planned — droppable (second) |
| T9 | Status, README, and debt-register closeout | docs-only | S | T7, plus each of T7b / T7c / T8 that was executed | [ ] Planned |

---

## S-230-T0: Slice plan, ledger, and roadmap entry

**Type:** docs-only
**Effort:** S
**Depends on:** —
**Status:** [x] Done 2026-08-16

**Acceptance criteria:**

- [x] `docs/plan/s-230-poc-v1-digitalocean.md` records objective, frozen scope,
      the verified gap analysis with file-level evidence, target architecture,
      ten-day sequence, risks, and explicit out-of-scope boundary.
- [x] This ledger exists with ordered tasks, dependencies, per-task acceptance
      criteria, and HP/EC cases for every development task.
- [x] `docs/plan/roadmap.md` carries an `S-230` row and records that `S-150` is
      parked for the POC window.

**Evidence to emit:** the two documents; `make qa-docs` (passed 2026-08-16:
doc consistency, task coverage, roadmap drift, OKF frontmatter all green).

**Status artifacts affected:** `docs/plan/roadmap.md`.

---

## S-230-T1: S3/Spaces credential and region wiring

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [ ] Planned — approval pending

**Problem (plan G1):** `crates/storage/src/s3.rs:17` uses
`AmazonS3Builder::new()`, which reads no environment. Credentials fall through to
AWS instance-metadata providers that do not exist on Digital Ocean, and the
region silently defaults to `us-east-1`.

**Happy paths considered:**

- **HP-1:** With `storage.backend = "s3"`, a Spaces endpoint, an explicit region,
  and injected credentials, the adapter is built with a static credential
  provider and a `put` followed by `get` against a real Spaces bucket round-trips
  identical bytes.
- **HP-2:** The existing local MinIO path keeps working unchanged with the same
  configuration shape.

**Edge cases considered:**

- **EC-1:** `backend = "s3"` in a production-like environment with missing
  credentials or missing region fails `AppConfig::validate()` at startup, before
  any request is served — not on the first write.
- **EC-2:** Credentials never appear in configuration files, logs, or traces;
  only injected `DUBBRIDGE_*` environment variables carry them (ADR-026
  Decision 4, ADR-018 redaction).

**Acceptance criteria:**

- `StorageSettings` carries region and credential references; secrets are
  env-only and absent from `config/*.toml`.
- `S3Adapter::new` sets bucket, endpoint, region, and static credentials
  explicitly.
- Production validation rejects an `s3` backend that is missing endpoint, region,
  or credentials.
- A real round-trip against an S3-compatible endpoint is executed and recorded —
  a unit test alone does not satisfy HP-1.

**Files expected to change:** `crates/config/src/lib.rs`,
`crates/storage/src/config.rs`, `crates/storage/src/s3.rs`,
`crates/storage/src/lib.rs`, `config/production.toml`, `config/staging.toml`.
Recompute the exact list before presentation.

**Evidence to emit:** RRI report, phase-1 and phase-2 review artifacts,
round-trip evidence against a real endpoint, Reflection log if the band requires
it, unit coverage certification, owner verification.

**Status artifacts affected:** this ledger; roadmap X9 wording if the storage
contract changes materially.

**Handoff prompt:** Wire explicit S3 credentials and region through config into
`S3Adapter`, fail closed in production when any are absent, and prove one real
round-trip. Do not touch the upload path or key layout.

**Stop condition:** Stop after the round-trip evidence. Do not start T2.

---

## S-230-T1b: API preparation queue bound to Redis

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [ ] Planned — approval pending

> The `T1b` suffix marks a task inserted on 2026-08-16 by the
> S-070/S-090/S-095/S-150 coverage review, after T0–T9 were already published in
> the roadmap row. The suffix keeps T2–T9 stable rather than renumbering them.

**Problem (plan G10):** `apps/api/src/main.rs:29` builds state with
`AppState::with_auth_service(..)`, which hardcodes
`preparation_queue: Arc::new(InMemoryPreparationJobQueue::default())`
(`apps/api/src/state.rs:31`, `:49`, `:67`). That queue is a
`Mutex<Vec<PreparationJob>>` (`crates/jobs/src/lib.rs:72`–`94`). The
worker-runner consumes from Redis (`apps/worker-runner/src/main.rs:76`–`88`), and
`apps/api` never opens a Redis connection at all.

The result is a **silent** no-op: finalize returns success, the job is pushed onto
an in-process vector, and no preparation, transcription, subtitle or review work
ever happens. Every probe stays green. Without this task the deployed POC accepts
uploads and produces nothing.

`AppState::with_preparation_queue` (`apps/api/src/state.rs:75`) already accepts an
injected queue but is unused and forces `auth_service: None`, so it cannot
currently serve an authenticated API.

**Happy paths considered:**

- **HP-1:** With a reachable `redis_url`, `POST /ingest/{token}/finalize` enqueues
  a `PreparationJob` onto the same Redis namespace the worker-runner's
  `RedisPreparationJobQueue` consumes, and the worker observes and executes it.
- **HP-2:** The API state carries both a real authenticated auth service and an
  injected queue at once — the current constructor split does not force choosing
  one.

**Edge cases considered:**

- **EC-1:** When Redis is unreachable, finalize fails closed exactly as the
  existing handler already specifies: `preparation_status` is written as `Failed`
  with the enqueue detail, the error is logged, and the response is 500
  (`apps/api/src/routes/ingestion.rs:331`–`347`). It must never report success on
  a dropped job.
- **EC-2:** Tests that rely on `InMemoryPreparationJobQueue` for assertions keep
  working — the in-memory implementation stays available as a test double and is
  only removed from the production startup path.
- **EC-3:** The Redis connection is established during startup, so a misconfigured
  `redis_url` surfaces at boot rather than on the first upload.

**Acceptance criteria:**

- `apps/api` constructs a `RedisPreparationJobQueue` from `config.redis_url` at
  startup and injects it into `AppState`.
- A single constructor carries both `auth_service` and an injected
  `preparation_queue`; the production path uses it.
- `InMemoryPreparationJobQueue` is no longer reachable from any binary's startup
  path, and a test proves the production constructor does not select it.
- An integration-level test proves a job enqueued through the API's configured
  queue is visible to a Redis-backed consumer, not only that `enqueue` returned
  `Ok`.

**Files expected to change:** `apps/api/src/state.rs`, `apps/api/src/main.rs`,
and possibly `crates/jobs/src/lib.rs` for a shared constructor seam. Recompute the
exact list before presentation.

**Evidence to emit:** RRI report, phase-1 and phase-2 review artifacts, evidence
that a real worker consumed an API-enqueued job, Reflection log if the band
requires it, unit coverage certification, owner verification.

**Status artifacts affected:** this ledger; the plan's G10 entry.

**Handoff prompt:** Bind the API's preparation queue to Redis at startup and
allow one `AppState` constructor to carry both the auth service and the injected
queue. Do not change the finalize handler's existing fail-closed enqueue-error
behavior, the job payload, or the queue namespace.

**Stop condition:** Stop once a Redis-backed consumer is shown to receive an
API-enqueued job. Do not start T2 and do not touch the worker-runner's worker
registration.

---

## S-230-T2: Migration runner in the production path

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [ ] Planned — approval pending

**Problem (plan G2):** `sqlx::migrate!` is used only in test modules. Nothing
applies the 29 files in `infra/migrations/` to a real database, and `apps/cli` is
a skeleton.

**Happy paths considered:**

- **HP-1:** Against an empty database, the runner applies all 29 migrations in
  order and populates `_sqlx_migrations`.
- **HP-2:** A second run against the same database is a no-op and exits zero
  (safe to run on every deploy).

**Edge cases considered:**

- **EC-1:** An unreachable database or a failing migration exits non-zero with
  the failing version identified; api and worker startup must not proceed.
- **EC-2:** A database whose applied-migration checksum diverges from the
  embedded set fails closed rather than applying a partial or reordered set.

**Acceptance criteria:**

- `apps/cli` exposes an explicit `migrate` entry point that embeds
  `infra/migrations` and is runnable as a one-shot container command.
- Configuration is loaded through the same fail-closed `AppConfig::load()` path
  as api and worker; no separate connection-string handling.
- Exit codes are meaningful enough for a Compose dependency condition to gate
  application startup on migration success.

**Files expected to change:** `apps/cli/src/main.rs`, `apps/cli/Cargo.toml`,
and focused tests. Recompute before presentation.

**Evidence to emit:** RRI report, phase reviews, real-PostgreSQL run evidence
(empty database and re-run), Reflection log if required, unit coverage
certification, owner verification.

**Status artifacts affected:** this ledger.

**Handoff prompt:** Add a `migrate` command to `apps/cli` that applies
`infra/migrations` through the existing config loader, idempotently, with
non-zero exit on failure. Do not change any migration file.

**Stop condition:** Stop after migration-run evidence. Do not wire Compose yet.

---

## S-230-T3: Real readiness probes for api and gateway

**Type:** development
**Effort:** M (provisional Moderate; recompute with `scripts/rri.py`)
**Depends on:** S-230-T0
**Status:** [ ] Planned — approval pending

**Problem (plan G3):** `apps/api/src/lib.rs:49` returns `"ready"`
unconditionally, never touching PostgreSQL, Redis, or storage. The gateway has
the same stub pair.

**Happy paths considered:**

- **HP-1:** With PostgreSQL, Redis, and storage reachable, `GET /health/ready`
  returns 200 and names each checked component.
- **HP-2:** `GET /health/live` stays a cheap process-liveness answer and does not
  depend on any external system.

**Edge cases considered:**

- **EC-1:** With PostgreSQL unreachable, readiness returns 503 and identifies the
  failing component; the process does not crash.
- **EC-2:** A hung dependency is bounded by a timeout so the probe cannot itself
  hang the health check.
- **EC-3:** Gateway readiness reflects its upstream API reachability rather than
  reporting ready while the API is down.

**Acceptance criteria:**

- Readiness performs a bounded check per dependency and aggregates to 200 or 503.
- Liveness and readiness stay semantically distinct.
- No credential or connection string appears in the response body.

**Files expected to change:** `apps/api/src/lib.rs`, `apps/gateway/src/lib.rs`,
supporting state/helpers, and focused tests. Recompute before presentation.

**Evidence to emit:** RRI report, phase reviews, tests covering healthy and
degraded dependency states, Reflection log if required, unit coverage
certification, owner verification.

**Status artifacts affected:** this ledger.

**Handoff prompt:** Make `/health/ready` actually probe PostgreSQL, Redis, and
storage with bounded timeouts in api, and upstream reachability in gateway.
Leave `/health/live` cheap. No new endpoints.

**Stop condition:** Stop after probe tests. Do not build images.

---

## S-230-T4: Production container images

**Type:** config/development
**Effort:** M
**Depends on:** S-230-T1, S-230-T1b, S-230-T2, S-230-T3
**Status:** [ ] Planned

**Problem (plan G4):** Dockerfiles exist only for the Python workers. The
worker-runner image is the hard one: it shells out to `ffprobe`/`ffmpeg` and
spawns `python3 workers/asr-worker-py/main.py` as a subprocess.

**Acceptance criteria:**

- Multi-stage images for `dubbridge-api` and `dubbridge-gateway` producing a slim
  runtime layer with no build toolchain.
- A worker-runner image containing the Rust binary, ffmpeg/ffprobe, Python, and
  faster-whisper, with `DUBBRIDGE_ASR_WORKER_PATH` and
  `DUBBRIDGE_ASR_WORKER_PYTHON` resolving inside the image.
- `ASR_MODEL_SIZE` is an explicit build/run parameter; the POC value is `small`,
  not the `large-v3` default (plan G7).
- A migration image or entry point derived from T2 that Compose can run as a
  one-shot job.
- Every image starts against local infrastructure and passes its own
  `/health/ready` where applicable.

**Evidence to emit:** Dockerfiles; local build and run transcripts; image sizes;
a successful local pipeline run using the built images rather than `cargo run`.

**Status artifacts affected:** this ledger; `DEVELOPMENT_REFERENCE.md` if the
documented local run path changes.

**Handoff prompt:** Author production Dockerfiles for api, gateway, and
worker-runner (with ffmpeg + Python + faster-whisper), plus a migration entry
point. Prove each starts and reports ready locally.

**Stop condition:** Stop after local image verification. Do not provision cloud
resources.

---

## S-230-T5: Production deployment descriptor and secret boundary

**Type:** config-only
**Effort:** M
**Depends on:** S-230-T4
**Status:** [ ] Planned

**Problem (plan G5, G6):** `S-030` Phase 3 is deferred, so no production
descriptor exists; `config/production.toml` holds `*.example` placeholders and
there is no `.env.example` at the repository root.

**Acceptance criteria:**

- `infra/production/docker-compose.yml` exists, is explicitly labelled the
  production descriptor (the counterpart to the local-only banner in
  `infra/local/`), and composes reverse proxy, gateway, api, worker-runner,
  Redis, and the one-shot migration job with a startup ordering that gates the
  applications on migration success.
- An environment template enumerates every required `DUBBRIDGE_*` variable with
  no real secret values committed.
- `config/production.toml` carries real Digital Ocean values for every non-secret
  field; secrets remain injected only (ADR-026 Decision 4).
- The POC upload ceiling is set explicitly and documented as the mitigation for
  the gateway buffering debt (plan G8).
- TLS termination and the public hostname are declared.
- **The auth configuration surface is complete (plan G11).** `config/production.toml`
  has no `[auth]` block and `AppConfig::validate()` rejects `auth.is_none()` in
  production-like environments (`crates/config/src/lib.rs:196`–`200`), so the
  template must supply all five double-underscore variables that
  `Env::prefixed("DUBBRIDGE_").split("__")` expects: `DUBBRIDGE_AUTH__ISSUER`,
  `DUBBRIDGE_AUTH__AUDIENCE`, `DUBBRIDGE_AUTH__RSA_PUBLIC_KEY_PATH`,
  `DUBBRIDGE_AUTH__JWT_SECRET`, `DUBBRIDGE_AUTH__CLOCK_SKEW_LEEWAY_SECONDS`.
- The template uses the double-underscore names **only**, with a comment
  recording that the single-underscore variants read by
  `AuthSettings::from_env()` (`crates/config/src/lib.rs:361`–`382`) belong to the
  legacy `AppConfig::from_env()` reader that no binary calls. Setting them
  produces no error and no effect.
- The same auth set is applied to **all three services**, since `apps/api`,
  `apps/gateway` and `apps/worker-runner` each call `AppConfig::load()` and each
  fails closed at boot on a partial set.
- `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS` is set explicitly rather than left to the
  24-hour serde default (`crates/config/src/lib.rs:146`), because there is no
  refresh path. The chosen value and its rationale are recorded by `S-230-T7c`,
  which owns the decision; T5 owns only carrying it in the template.
- `rsa_public_key_path` is supplied as an explicit placeholder with an inline
  comment recording that ADR-031 made the field dead; removing it is T9 debt, not
  T5 work.

**Evidence to emit:** the descriptor and environment template; a local
dry-run of the descriptor against local infrastructure; confirmation that no
secret value is committed.

**Status artifacts affected:** this ledger; `docs/plan/roadmap.md` S-030 Phase 3
and X21 wording; ADR-026 implementation references if the secret boundary moves.

**Handoff prompt:** Author the production Compose descriptor, environment
template, and real `config/production.toml` values. Commit no secrets.

**Stop condition:** Stop after a local dry-run of the descriptor. Do not deploy.

---

## S-230-T6: First deploy and end-to-end smoke on Digital Ocean

**Type:** operational
**Effort:** L (operational; RRI expected well below the effort impression)
**Depends on:** S-230-T5
**Status:** [ ] Planned

**Acceptance criteria:**

- Droplet, managed PostgreSQL, Spaces bucket, DNS, and TLS are provisioned and
  recorded.
- Migrations applied via the T2 runner; api, gateway, and worker report ready via
  the T3 probes.
- The first account is created against `POST /auth/register` directly — at T6
  time the mobile app still has no registration screen — and the runbook records
  the exact call. This step stays in the runbook even if `S-230-T7b` later adds
  the screen: it is the operator's recovery path when no UI is reachable.
- A real video completes the full path: login, upload, rights confirmation,
  finalize, HLS preparation, ASR, subtitle generation, review-task creation,
  approval, publication, and in-app playback — with evidence for each stage
  (audit rows, artifact records, storage keys, manifest fetch).
- **Every stage is asserted on observed downstream state, never on a 2xx alone.**
  A successful finalize response is not evidence that preparation ran; the
  corresponding artifact rows, preparation status transitions and the review task
  appearing in the inbox are. This is the direct lesson of plan G10, where a green
  API and a silently inert pipeline coexisted.
- Managed-PostgreSQL TLS behavior through `create_pool` is confirmed rather than
  assumed.
- A runbook records provisioning, deploy, migrate, rollback, log access, and
  observed timings for preparation and ASR.

**Evidence to emit:** provisioning record, deploy transcript, per-stage E2E
evidence, runbook, cost summary.

**Status artifacts affected:** this ledger; `docs/plan/roadmap.md`; README status
table.

**Handoff prompt:** Provision, deploy, migrate, and drive one real video through
the entire pipeline on Digital Ocean; record evidence per stage and write the
runbook.

**Stop condition:** Stop after the smoke run and runbook. Do not change product
code to make the smoke pass — a failure is a finding, not a patch target.

---

## S-230-T7: Mobile POC build against the deployed backend

**Type:** development/operational
**Effort:** M
**Depends on:** S-230-T6
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** A build configured with the deployed `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL`
  completes login, upload, review, publish, and playback against the DO backend.

**Edge cases considered:**

- **EC-1:** A missing or malformed gateway URL surfaces the existing
  `ConfigErrorScreen` rather than failing opaquely at first request.
- **EC-2:** An expired or rejected token drives the existing logout path, not a
  silent stall.

**Acceptance criteria:**

- The POC build targets the deployed hostname over HTTPS with no local fallback
  compiled in.
- `npm run typecheck && npm run lint && npm test` stay green.
- Install and run instructions for a POC tester are recorded.

**Files expected to change:** mobile environment/build configuration only.
Product screens are expected to need no change; if any does, record why.

**Evidence to emit:** build transcript, `make qa-mobile` output, on-device
walkthrough evidence, distribution instructions.

**Status artifacts affected:** this ledger; README mobile section.

**Handoff prompt:** Produce a distributable mobile build pointed at the deployed
Digital Ocean backend and verify the full flow on a device.

**Stop condition:** Stop after the device walkthrough. Do not start T8.

---

## S-230-T7b: Mobile registration screen

**Type:** development (mobile)
**Effort:** M (provisional; recompute with `scripts/rri.py`)
**Depends on:** S-230-T7
**Status:** [ ] Planned — droppable, first drop candidate

> Added 2026-08-16 at owner request, promoting a secondary finding of the
> S-070/S-090/S-095/S-150 coverage review into planned work (plan G12). The
> `T7b` suffix keeps T8 and T9 stable.

**Problem (plan G12):** the backend is complete and the mobile surface is
missing. `apps/api/src/routes/auth.rs:23` mounts `POST /auth/register` and its
handler returns `(StatusCode::CREATED, Json(AuthSuccessResponse))` — the same
payload shape login returns, so registration can authenticate directly.
`apps/gateway/src/auth/mod.rs:16` already relays the route. But
`mobile/src/screens/` has only `LoginScreen.tsx`, and `AuthContextValue`
(`mobile/src/auth/AuthProvider.tsx:21`–`28`) exposes no `register` method.

Every tester account therefore has to be created by the operator with a direct
API call.

**Read first:** root `DESIGN.md` (mobile visual intent and component usage) and
`mobile/src/screens/LoginScreen.tsx`, whose structure and error presentation this
screen should mirror rather than reinvent.

**Happy paths considered:**

- **HP-1:** A valid email, password and workspace name submitted from the
  registration screen create the account through the gateway relay and leave the
  app authenticated on the post-login screen, with no separate login step —
  `AuthSuccessResponse` already carries `token`, `userId` and `workspaceId`.
- **HP-2:** The session persisted by registration is indistinguishable from one
  persisted by login: `saveAuthSession` stores the same shape and a relaunch
  hydrates it normally.

**Edge cases considered:**

- **EC-1:** A duplicate email returns 409 (`from_register_error`,
  `apps/api/src/routes/auth.rs:162`; covered by
  `register_handler_maps_duplicate_email_to_conflict`, `:470`) and the screen
  shows a distinct "email already registered" message rather than a generic
  failure.
- **EC-2:** A validation failure returns 400
  (`register_handler_maps_validation_errors_to_bad_request`, `:501`) and the
  screen reports which constraint failed without clearing the entered email.
- **EC-3:** A network failure or unreachable gateway surfaces the same
  `network_error` treatment `submitLogin` already uses
  (`mobile/src/auth/AuthProvider.tsx`, `loginErrorKind`), and leaves no partial
  session behind.
- **EC-4:** A missing or malformed runtime config routes to the existing
  `ConfigErrorScreen`, exactly as the login path does.
- **EC-5:** The password field is never logged, never included in an error
  message, and not retained after a failed submission.

**Acceptance criteria:**

- `AuthProvider` exposes a `register` method that shares `submitLogin`'s
  response-validation and session-persistence path rather than duplicating it;
  `isAuthSuccessPayload` remains the single acceptance check for both.
- A `RegisterScreen` exists with reciprocal navigation to and from
  `LoginScreen`.
- Registration failures map to distinct, user-readable states for 409, 400, and
  network, and are covered by tests.
- `npm run typecheck && npm run lint && npm test` stay green.
- No new API route, no change to `apps/api` or `apps/gateway`.

**Files expected to change:** `mobile/src/auth/AuthProvider.tsx`,
`mobile/src/screens/RegisterScreen.tsx` (new), the navigator, and tests.
Recompute the exact list before presentation.

**Evidence to emit:** RRI report, phase-1 and phase-2 review artifacts,
`make qa-mobile` output, Reflection log if the band requires it, unit coverage
certification, owner verification, screenshot evidence of the success and the
409 states.

**Status artifacts affected:** this ledger; the plan's G12 entry; the T9 debt
register, which drops the "no registration screen" item if this task lands and
keeps it if the task is dropped.

**Handoff prompt:** Add a mobile registration screen over the existing
`POST /auth/register` relay, reusing the login session-persistence path. Do not
add API routes, password reset, email verification, or invites.

**Stop condition:** Stop after the screen registers an account and lands
authenticated. Do not touch the backend auth surface.

---

## S-230-T7c: Session lifetime and expiry behavior

**Type:** development/config (mobile + descriptor value)
**Effort:** S (provisional; recompute with `scripts/rri.py`)
**Depends on:** S-230-T7
**Status:** [ ] Planned — not a drop candidate

> Added 2026-08-16 at owner request, promoting the second secondary finding of
> the coverage review into planned work (plan G13).

**Problem (plan G13), stated precisely so it is not over-built:** the 401
handling **already exists**. `mobile/src/api/client.ts:57` maps 401 to
`session_expired` and about fifteen call sites act on it by calling
`auth.logout()` — `mobile/src/screens/useUploadFlow.ts:72`, `:82`, `:93`;
`mobile/src/screens/useReviewInboxLoader.ts:108`, `:123`;
`mobile/src/screens/useReviewDetailMutations.ts:42`, `:62`, among others. This
task must not re-implement that.

Two things are actually open:

1. `jwt_expiry_hours` has no production value. Its serde default is 24
   (`crates/config/src/lib.rs:146`) and `config/production.toml` has no `[auth]`
   block at all (plan G11), so today the deployed lifetime would be set by
   omission.
2. `hydrateStoredSession` → `acceptStoredSession`
   (`mobile/src/auth/AuthProvider.tsx`) accepts a persisted session and sets
   status `authed` without checking expiry, so an app launched after the token
   expired renders the authenticated UI and only falls back to login on the
   first 401.

**Happy paths considered:**

- **HP-1:** The POC token lifetime is an explicit value in the environment
  template with a recorded rationale, and a token issued by the deployed API
  carries exactly that `exp`.
- **HP-2:** Launching the app with a stored session that is still valid goes
  straight to the authenticated surface, unchanged from today.

**Edge cases considered:**

- **EC-1:** Launching with a stored session whose token has already expired
  routes to login immediately, without rendering the authenticated surface and
  without waiting for a request to fail.
- **EC-2:** A token that expires mid-session still produces the existing
  `session_expired` → `logout()` behavior; this task changes none of those call
  sites.
- **EC-3:** A stored session whose token cannot be parsed is treated as absent
  and cleared, matching the existing `hydrateStoredSession` catch path — a
  malformed token must never be treated as valid.
- **EC-4:** Expiry evaluation tolerates clock skew consistently with the server's
  `clock_skew_leeway_seconds`, so a client clock a few seconds fast does not
  eject a valid session.

**Acceptance criteria:**

- `DUBBRIDGE_AUTH__JWT_EXPIRY_HOURS` carries an explicit POC value; the chosen
  number and the reason are recorded here and consumed by the T5 template.
- Stored-session hydration rejects an expired or unparseable token before
  setting status `authed`.
- No refresh-token mechanism, no silent renewal, and no change to the ~15
  existing `session_expired` call sites.
- `npm run typecheck && npm run lint && npm test` stay green.
- The absence of a refresh path is written into the T9 debt register as a
  deliberate POC decision, not an oversight.

**Files expected to change:** `mobile/src/auth/AuthProvider.tsx`,
`mobile/src/auth/session.ts`, tests, and the T5 environment template value.
Recompute the exact list before presentation.

**Evidence to emit:** RRI report, phase-1 and phase-2 review artifacts, a test
proving an expired stored session never reaches `authed`, `make qa-mobile`
output, Reflection log if the band requires it, unit coverage certification,
owner verification.

**Status artifacts affected:** this ledger; the plan's G13 entry; `S-230-T5`
(the expiry value); the T9 debt register.

**Handoff prompt:** Set the POC token lifetime deliberately and stop an expired
stored session from rendering as authenticated. Do not add refresh tokens and do
not modify the existing `session_expired` handling.

**Stop condition:** Stop once an expired stored session routes to login on
launch. Do not touch the API's token issuance.

---

## S-230-T8: Subtitle visible in the review surface (optional)

**Type:** development
**Effort:** M
**Depends on:** S-230-T6
**Status:** [ ] Planned — droppable, second drop candidate after S-230-T7b

**Problem (plan G9):** `apps/worker-runner/src/review_enqueue.rs:35` creates
review tasks with `subtitle_artifact_id: None`, so a reviewer approves a
subtitle they cannot see.

**Explicit boundary:** this is **not** `S-150-T6`. It binds the existing nullable
column at creation time for the current subtitle only. It introduces no
generation-aware identity, no version uniqueness, no regeneration semantics, and
does not close `X-S-160-3`.

**Happy paths considered:**

- **HP-1:** Subtitle readiness creates a review task whose `subtitle_artifact_id`
  references the exact persisted subtitle artifact.
- **HP-2:** The mobile review surface renders that subtitle's content alongside
  the player before a decision is recorded.

**Edge cases considered:**

- **EC-1:** When the subtitle artifact cannot be resolved, the review task is
  still created with a null binding and the surface degrades visibly — subtitle
  readiness is never rolled back by a review-enqueue failure (current
  `prepare_review_post_ready` contract).
- **EC-2:** Pre-existing review rows with a null binding stay readable and
  decidable.
- **EC-3:** Subtitle content is only served to a caller already authorized for
  that review task's organization.

**Acceptance criteria:** artifact ID bound at creation; a read endpoint scoped by
the existing org guard; mobile rendering; no change to the ADR-030 publication
gate.

**Files expected to change:** `apps/worker-runner/src/review_enqueue.rs`,
a scoped API route, and `mobile/src/screens/ReviewDetailScreen.tsx`. Recompute
before presentation.

**Evidence to emit:** RRI report, phase reviews, Reflection log, unit coverage
certification, owner verification, screenshot evidence.

**Status artifacts affected:** this ledger; explicitly **not** `X-S-160-3`, which
stays open and owned by `S-150-T6`.

**Handoff prompt:** Bind the existing nullable subtitle artifact column at review
creation and render the subtitle in the review surface. Do not introduce
generation-aware identity.

**Stop condition:** Stop after the review surface renders the subtitle. Do not
touch S-150 artifacts.

---

## S-230-T9: Status, README, and debt-register closeout

**Type:** docs-only
**Effort:** S
**Depends on:** S-230-T7, and whichever of S-230-T7b / S-230-T7c / S-230-T8 were
executed rather than dropped
**Status:** [ ] Planned

**Acceptance criteria:**

- `docs/plan/roadmap.md` carries the `S-230` row, records `S-150` as parked for
  the POC window, and updates S-030 Phase 3 and X9/X21 wording to match what
  T1/T5 actually delivered.
- README status table reflects what the deployed POC genuinely does; nothing
  deferred is described as working.
- Debt carried by this slice is registered explicitly: gateway body buffering
  (G8), null-bound review tasks if T8 was dropped (G9), the vestigial required
  `auth.rsa_public_key_path` field that ADR-031 made dead (G11), the coexisting
  single-underscore `AuthSettings::from_env()` reader that no binary calls (G11),
  full-segment in-memory reads in `StorageAdapter::get`, the absent mobile
  registration screen **if T7b was dropped** (G12), the absence of any refresh or
  silent-renewal path (G13, a deliberate POC decision rather than an oversight),
  and any T6 finding not fixed in-window.
- The status of every droppable task is stated explicitly — executed or dropped,
  and why — so the debt register cannot silently omit a dropped task's gap.
- The stale roadmap note "`S-070` (JWKS) remains recommended before production
  device login" is corrected: ADR-031/S-200 moved token issuance in-house, so the
  open hardening items are `X-S-200-1` (RS256) and `X-S-200-2` (revocation), not
  JWKS discovery.
- `make qa-docs` passes.

**Evidence to emit:** documentation diff; `make qa-docs` output.

**Status artifacts affected:** `docs/plan/roadmap.md`, `README.md`, this ledger,
the S-230 plan.

**Stop condition:** Stop after documentation QA. Do not start S-150 or S-170.
