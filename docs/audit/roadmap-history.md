---
type: Audit
title: "Roadmap Detail Archive"
status: reference
---

# Roadmap Detail Archive

Companion to `docs/plan/roadmap.md`. The roadmap keeps only current status,
dependencies, and links so it stays cheap to load into every agent session
(it is one of the five sources concatenated into `AGENTS.override.md` and
natively imported by `CLAUDE.md`). This file preserves the fuller narrative —
consolidation changelog, design rationale, and detailed per-slice "what
shipped and why" history — that used to live inline in the roadmap. Nothing
here is authoritative on its own: it explains decisions already reflected in
the roadmap table, the linked task ledgers, and the ADRs.

Trimmed from `docs/plan/roadmap.md` on 2026-08-20 to reduce the token cost of
loading the roadmap into every Claude Code / Codex session; no information
was deleted, only relocated here.

## Roadmap consolidation changelog

Last consolidated: 2026-05-31 after the roadmap/ADR/architecture review in
`docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md` (including the
same-day ADR-traceability follow-up G1–G4 in that file). Updated the same day
after `S-020`/H1 completion. Updated 2026-06-03: scoped `S-030` around environment
separation and fail-closed configuration (see "S-030 Strategy" below, principle
added, and X21), then synchronized after `S-030` Task 2 and Task 3 completion.
Updated 2026-06-03 again: added plan/task ledgers for `S-040` (first-party session
gateway / BFF) and introduced `S-050` (first-party mobile client, React Native + Expo)
as an `S-040`-gated consumer
(ADR-024). Updated 2026-06-03 once more after
`S-030` Task 5 moved local Compose under `infra/local/` and wired the opt-in `app`
profile to fail-closed local config. Updated again on 2026-06-03 after `S-030` Task 6
aligned the local Rust image with `rust-toolchain.toml` and added the committed-config
secret guard. Updated 2026-06-07 after `S-050` T0–T5 completion: the mobile app is now
implemented, tested, and reflected in the architecture/task status documents.
Updated 2026-06-18 to add `S-125` HLS playback delivery and ADR-032 so prepared
`.m3u8` packages are served through an explicit backend boundary instead of being
hidden inside later publication work. Updated 2026-08-20 to track the Tiger Style
adoption evaluation (`docs/proposals/tiger-style-adaptation-evaluation.md`) as a
cross-cutting item pending owner resolution of its three decision points before any
plan/task ledger is drafted. Updated 2026-08-20 to add cross-cutting item X27
tracking the proposed-but-unapproved Gemma Push Reviewer role
(`docs/plan/gemma-push-reviewer-role.md`, revision r2, 2026-08-19) as scheduled
next-up work with no fixed date.

## S-150 — Translation + dubbing: full status narrative

🟡 planned 2026-08-16 — T0, T1a, T1b, T1c-i, T1c-ii, the T2a seam extraction, T2b-i,
all three T2b-ii delivery-repository children, `T2c-i`, `T2c-ii`, `T2c-iii`,
`T2c-iv-a0`, `T2c-iv-b`, and `T2c-iv-c` are complete; `T2c-iv-a` has its contract
cutover implemented and reviewed but stays `[~]` pending the still-open `T2c-vi-a`
runtime cut. The slice now has product-code domain types, four artifact kinds,
per-localization statuses, exact current-generation pointer/claim storage,
fail-closed repositories, durable per-target dispatch identities, atomic
claim/outbox persistence, guarded pending-to-enqueue-failed and
pending-to-acknowledged transitions, versioned subtitle/translation job contracts
with deterministic initial UUIDv5 identity, exact persisted Subtitle replay
resolution, a local-sized 286-line job-contract module, a subtitle producer that
builds `SubtitleJob` with only the route-free asset/project constructor
(`T2c-iv-b`, RRI 32, zero-diff formal closure of the 2026-08-15 workspace-compile
compatibility patch, local-first `qwen3.6:35b-a3b` run + Gemma phase-1/phase-2
review), and a durable localization fan-out service that resolves the exact
persisted subtitle and independently persists/dispatches one `TranslationJob` per
eligible configured target without letting one target's persistence failure
corrupt another's (`T2c-iv-c`, RRI 39 Moderate, corrected from a provisional RRI
49/Med-high estimate; whole-task local-agent route exhausted its 2-attempt budget
— attempt 1 missed module registration/an import path, repair 1 degraded into a
non-functional stub, repair 2 hit `budget_exhausted` — after which, per an owner
directive to maximize local-model usage and keep the cloud role
orchestration-only, the remaining work was decomposed into three Low-band (RRI
0-25) subtasks delegated to local Nemotron via `scripts/delegate-low-rri.py`, with
a handful of individually-diagnosed one/two-line type fixes applied directly only
after the delegation tooling itself failed to produce a usable before-after diff
twice; Gemma phase-2 review PASS, 0 findings). On 2026-08-13 the owner rejected
the unused legacy-review compatibility path; the resulting T2c-iv surface scored
RRI 63 and is decomposed into `T2c-iv-a0/a/b/c`, followed by the narrowed `T2c-v`
Redis adapter and decomposed `T2c-vi` runtime/cleanup cutover. `T2c-v` (Redis
translation queue adapter) is next — still carries its own separate, unresolved
"Redis-topic decision" parking note, independent of the S-230 scope question below
— followed by the `T2c-vi-a/b` runtime cutover. Queue, worker, and TTS/runtime
work remain pending; T8 tracks non-blocking future voice-consent hardening.
**Partially reopened 2026-08-16 (second pass) for the S-230 POC window**,
reversing the same-day initial parking: `T2c-v`(50), `T2c-vi-a`(51),
`T2c-vi-b`(31), `T3a`(42), `T3b`(44), `T3c`(53) are back in scope, tracked and
sequenced as `S-230-T3b` (`docs/tasks/s-230-poc-v1-digitalocean.md`), after the
owner reviewed and explicitly overrode the S-230 plan's own recommendation
against reopening them (`docs/plan/s-230-poc-v1-digitalocean.md`
§"The market-audience gap, examined"). `T4`(26), `T5`(68–70, mandatory
decomposition), `T6`(71, mandatory decomposition), and `T7` — the TTS/dubbed-audio
surface — remain parked/out of scope, blocked on the ADR-028 consent seam.
`S-230-T3b` runs as a parallel track outside S-230's original ten-day critical
path, not a hard gate on `S-230-T6`.

## S-230 — POC v1 deployment: full status narrative

🟡 planned 2026-08-17 — T0, T1, T1b, T2, and T3 done. T4 was decomposed before
implementation from one RRI 47 Med-high cloud task into 17 independently-scored
Low/S children (`T4a`–`T4q`, RRI 10–25) by owner direction because Codex cloud
tokens are unavailable; eligible one-path development patches route to local
`qwen3.8:27b-mlx`, with Muse Glimmer independent review, while T4p/T4q remain
primary-orchestrator operational/docs work. T3b, the T4 child chain, T5–T7, T7b,
T7c, T8, and T9 remain pending. Deployment-enablement slice, not a product slice:
it takes the already-closed upload → rights → HLS → ASR → subtitles → review →
publication → playback path and makes it publicly runnable on a Digital Ocean
droplet. Owner-scoped 2026-08-16, amended same day (second pass): `S-150-T2c-v`
through `T3c` (text-only cross-language subtitle translation) is reopened and
tracked as `T3b`, after the owner reviewed and explicitly overrode this plan's own
recorded recommendation against doing so; `S-150-T4`–`T7` (TTS/dubbed audio)
remain parked/out of scope, blocked on ADR-028. Deployment target is droplet +
production Compose (not App Platform); `apps/gateway` ships as-is with its
request/response buffering accepted as recorded debt and bounded by a lowered POC
upload ceiling. Adds no new application technology beyond what `T3b`'s S-150
children already depend on (Redis, already in use for 3 other queues). A
2026-08-16 owner-requested coverage review confirmed that **nothing from `S-070`,
`S-090`, `S-095` or `S-150` needs to move into this slice** — none of the four
mounts a route or registers a worker on the POC path — but it added two blocking
gaps that the first pass missed: **G10**, `apps/api` formerly enqueued
preparation jobs into an in-process `Mutex<Vec<_>>` while the worker-runner
consumed from Redis (closed by `T1b`); and **G11**, `config/production.toml`
carries no `[auth]` block while `AppConfig::validate()` requires one in
production-like environments, so all three binaries fail closed at boot until
five `DUBBRIDGE_AUTH__*` variables are injected (assigned to `T5`). On
2026-08-16 the owner promoted two secondary findings into planned work: **G12**,
the mobile app has no registration surface although the backend route exists
(now `T7b`, droppable, with T6's direct API fallback); and **G13**, tokens
default to 24h with no refresh path and stored sessions need an expiry check
(now `T7c`).

## Product-layer phase notes and S-050/S-040 mobile handoff narrative

`S-040` must be planned before building a first-party browser, operator-console, or
mobile auth flow. It does not block S-080 or S-090.

**Product-layer phases.** `S-100`, `S-105`, `S-110`, and `S-160` turn the governed pipeline
into a team-usable product. `S-100` is the collaboration foundation: orgs, roles,
projects, and target languages. `S-105` establishes mobile as the only authenticated
product UI (ADR-029) and retires the historical web prototype. `S-110` is intentionally
placed before `S-150` because TTS/dubbing must fail closed without voice consent.
`S-160` can be built against fixtures before `S-140/S-150` land, but its canonical
runtime role is to supply the review/publication gate that `S-170/S-180` adopt.
`S-125` supplies the shared HLS playback-delivery boundary those runtime slices use
for review preview and publication playback; it is not a public web/player UI.
These product-layer phases introduced architecture decisions that are now captured
by ADR-027, ADR-028, ADR-029, ADR-030, and ADR-032.

`S-050` (mobile) is a first-party interactive client and therefore a hard consumer of
the `S-040` gateway (ADR-024): the device must terminate in the same session-gateway
trust boundary as the web app and must not hold long-lived tokens. `S-040` was
completed for the browser/cookie transport on 2026-06-04; `S-050-T0` verified the
delivered surface was browser-oriented only. `S-040-T7` is the unblock, decomposed in
`docs/tasks/s-040-t7-mobile-session-handoff.md`. T7.1 (contract definition) is
complete as of 2026-06-04: five gateway surfaces are specified (`GET
/auth/login?return_uri`, mobile callback redirect with one-time handoff code,
`POST /auth/mobile/session` redemption, `ANY /api/*` and `POST /auth/logout`
with `X-Dubbridge-Session` header), ADR-024 invariants (no access or refresh
token on device, no parallel auth path) are enumerated, and implementation notes
for T7.2–T7.4 are recorded. T7.2 is now complete: the gateway validates
registered mobile `return_uri` values, carries the mobile intent through pending
OAuth state, and branches callback completion between the browser cookie path and
the mobile `handoff_code` redirect with no cookies set. T7.3 is now complete:
the gateway exposes `POST /auth/mobile/session`, redeems handoff codes into
opaque `session_ref` values, accepts `X-Dubbridge-Session` on `/api/*`, and
rejects mismatched cookie/header transports fail-closed. T7.4 is now complete:
mobile refresh returns the rotated opaque session reference in
`X-Dubbridge-Session`, mobile logout accepts the same transport, and a
deterministic end-to-end mobile lifecycle is covered by tests. Session renewal and
rotation are gateway-owned: mobile carries only the current opaque reference and
persists a rotated replacement when the gateway returns one. Stack decision
(2026-06-03): React Native + Expo. The mobile
app is now implemented in `mobile/` with gateway-backed auth, navigation, asset
list/detail surfaces, and deterministic Jest coverage. A planned
mobile-hardening sub-slice, **S-055** (Maestro screenshot / visual-audit suite,
`docs/plan/s-055-maestro-screenshot-suite.md` + `docs/tasks/s-055-maestro-screenshot-suite.md`)**,
was gated on **S-050-T4** and approved with Option A (ADR-024 handoff-code bootstrap,
no JWT on device) + sequencing S-080 (defer until after T4). That gate is satisfied.
The sub-slice is complete: test IDs, screenshot env, mock OAuth fixture,
handoff-code seed, dev-gated E2E bootstrap, both Maestro flow files, the
`seed-and-run.sh` runner with report sanitization, and the `npm run screenshots`
alias are all delivered. Both phases capture their screenshots (`01_auth_login.png`,
`02_home.png`). S-055 is done as of 2026-06-12.

## S-030 Strategy: environment separation & fail-closed configuration

`S-030` makes the local ↔ production boundary explicit and hard to confuse. Today
`crates/config` compiles local defaults into the binary (`AppConfig::from_env` falls
back to `localhost` Postgres/Redis and `/tmp` storage), so a misconfigured production
process boots silently against development resources. `S-030` inverts this to the same
fail-closed posture as the rights gate (ADR-008): wrong configuration must abort
startup, not degrade silently.

Design (recommended: typed layered config; no Kubernetes assumed at this stage):

- One explicit discriminator `DUBBRIDGE_ENV ∈ {local, staging, production}` with no
  compiled default; an unknown or missing value fails closed at startup.
- Resolution layers: code defaults (universal only) ← `config/default.toml` ←
  `config/<env>.toml` (committed, non-secret) ← `DUBBRIDGE_*` env vars (secrets and
  per-deploy overrides). The former in-code `localhost`/`/tmp` fallbacks move into
  `config/local.toml`; they never live in the binary again.
- A single typed schema + `validate()` is read by `apps/api` and `apps/worker-runner`
  alike and, in production-like environments, rejects localhost datastores, the
  local-fs storage backend, absent auth (ADR-023), and human-pretty log format
  (must be JSON, ADR-018).
- Storage backend selection becomes env-driven (`build_adapter` switches on a backend
  selector). The selector boundary is `S-030`; the MinIO/S3 adapter itself is `S-080` (X9).
- Observability format/exporter become env-driven (`init_tracing` parameterized):
  local pretty, production JSON + exporter (ADR-018).
- `infra/` is split so Compose is local infrastructure only (a banner states it is
  not the production descriptor); the production deployment descriptor is a separate
  artifact added when a first deploy target exists.

Phasing (now vs later):

- Phase 0 (now): `DUBBRIDGE_ENV` + a typed `load()` + `validate()`; move local
  defaults to `config/local.toml`; add `config/default.toml` and `.env.example`;
  api/worker switch to fail-closed load. This portion is complete and closes the
  compiled-default leak (core of X18).
- Phase 1 (now): reorganize to `infra/local/`; Compose = infra + app under a profile
  with a non-production banner. The file move, app-profile env wiring, and Rust image
  alignment to `rust-toolchain.toml` are complete.
- Phase 2 (couples with `S-080`): env-driven storage backend selector (X9) and env-driven
  observability format/exporter (ADR-018).
- Phase 3 (later): production deployment descriptor + secret-manager injection
  boundary; owner-credential secret-store decision (X20).
- Phase 4 (deferred): orchestration (k8s/Helm or Nomad), telemetry collector, config
  service — only if multiple live environments or teams justify it. Not assumed now.

The layered fail-closed configuration & environment-separation decision is recorded
in ADR-026. The owner-credential secret-store mechanism (X20) remains an open decision
and warrants its own ADR when authored (X3).

## Why Platform Ingest Is S-090 (And Live Recording Is S-095)

**Replan 2026-05-31 (ADR-025).** The real `S-090` intake use case is owner-authorized
**platform download**: the content owner provides scoped credentials to their own
platform account and DubBridge downloads the owner's content on their behalf. This
is the primary `S-090` path. RTMP/SRT live capture
is demoted to a deferred sub-slice (**S-095**) for the minority of clients who produce
live broadcasts.

Intake (in either mode) widens the funnel and has no dependency on media preparation
or ML stages, so it belongs before `S-120`–`S-180`. Hard dependencies of the **primary
platform-download path**:

- `S-000` verified principals for Axum ingest endpoints (ADR-023).
- `S-010`'s reusable finalize path (`finalize_ingestion_core`) and `StorageAdapter`
  boundary (ADR-006, ADR-021) — reused producer-agnostically.
- A per-connector engine behind `crates/connectors` (`PlatformConnector` trait),
  mirroring the `crates/media` pure-builder / IO-executor boundary (ADR-025).
- Owner-credential handling stored by reference and redacted (ADR-025, ADR-018).
- H1 atomicity and durable-audit hardening before the reused finalize path expands.
- The completed YouTube spike (`S-090-C2`), which ruled out YouTube as the pinned
  backend-download v1 provider, and a new provider-capability spike (`S-090-C4`) before
  the first connector is built.

The **deferred `S-095` live-recording path** additionally needs the FFmpeg-subprocess
recorder (ADR-019), the segment/lifecycle model and T0c output contract (ADR-020),
and RTMP/SRT capture-edge authentication (ADR-022). Its domain + migration foundation
(T1/T2) is already built and shared with the primary path.

`S-080` remains a prudent predecessor because intake is the first sustained, high-volume
writer. The trait boundaries make `S-090` technically possible without `S-080`, but building
retention and upload against the production-like MinIO/S3 adapter avoids rework.

## S-090 Internal Task Map (REPLANNED 2026-05-31, ADR-025)

The `S-090` ledger is `docs/tasks/stream-recording-ingest.md`. The primary intake use
case is owner-authorized **platform download**, not RTMP/SRT live capture. The
FFmpeg recorder (ex-T3–T8) is deferred to **S-095**.

```text
Shared foundation (DONE, reused by both paths):
  T0  reusable S-010 finalize core
  T0b duplicate audit type removed (via T1-T5)
  H1  atomicity + durable-audit gate closed
  T0c (S-095 only) HLS fMP4 staging + assembled MP4 contract fixed
  T1  domain: recording aggregate, ArtifactKind, audit generalization
  T2  migrations: recording_sessions + audit generalization

PRIMARY S-090 — platform ingest (internal S-090-C1 -> S-090-C7):
  S-090-C1 connector trait boundary (crates/connectors) + PlatformIngestSession domain
  S-090-C2 YouTube retrieval-mechanism spike (gate) -> DONE 2026-06-03
  S-090-C3 provider-path replan after YouTube spike -> DONE 2026-06-03
  S-090-C4 first supported-provider capability spike (gate) -> DEFERRED for this phase
  S-090-C5 first supported-provider connector v1 -> DEFERRED for this phase
  S-090-C6 PlatformIngestJob + download->bridge wiring + platform_ingest_sessions migration -> DEFERRED for this phase
  S-090-C7 API endpoints (/ingests/platform) -> DEFERRED for this phase

S-095 — live recorder (DEFERRED): ex-T3 recorder crate, ex-T4 jobs/storage,
  ex-T5 bridge, ex-T6 API, ex-T7 worker, ex-T8 tests. Marked [~] REPLANNED.
```

`T9` (docker-compose Rust pin) is independent low-priority housekeeping.

## Known planning gaps (full historical log)

- **S-090 replanned 2026-05-31 (ADR-025).** Primary path is owner-authorized platform
  download. `S-090-C1`/`S-090-C2`/`S-090-C3` are complete; the remaining `S-090-C4`–`S-090-C7` work is intentionally
  deferred for this phase. RTMP/SRT live recording (ex-T3–T8) is the deferred `S-095`
  sub-case.
- The shared foundation (T0/T0b/T0c/H1/T1/T2) is complete and reused by both paths.
  T0c only governs `S-095` (it fixed the live-recording output contract).
- The YouTube retrieval mechanism for the platform path was spiked on 2026-06-03.
  Result: official docs validate `resolve()` but not an API-driven backend
  `download()` path. YouTube is therefore deferred for backend-download in this
  slice; `S-090-C4` is the next gate for selecting the first officially
  supported provider.
- The owner-credential secrets-store mechanism (X20) has no dedicated ADR yet and
  must be decided during `S-090-C1`–`S-090-C6`; `S-030` establishes the config/secret
  split it plugs into.
- `S-070` still needs plan/task ledgers before execution. `S-120` is now complete
  with `docs/plan/s-120-media-preparation.md` + `docs/tasks/s-120-media-preparation.md`
  synchronized through `T5c`.
  `S-125` is now complete: `docs/plan/s-125-hls-playback-delivery.md` +
  `docs/tasks/s-125-hls-playback-delivery.md` record the delivered grant contract,
  schema/repo, pure rewriter, issuance API, rewritten manifests, short-lived scoped
  segment references, and ADR/docs propagation. ADR-032 is `Accepted`.
  `S-030` now has `docs/plan/s-030-environment-separation.md` +
  `docs/tasks/s-030-environment-separation.md` with its current Phase 0 / Phase 1
  scope complete. `S-040` now has
  `docs/plan/s-040-session-gateway-bff.md` + `docs/tasks/s-040-session-gateway-bff.md`
  (complete). `S-080` must include the object-store adapter, storage-key
  ownership, orphan reconciliation, and upload memory-safety strategy.
- **Mobile is phase S-050, introduced 2026-06-03 and completed 2026-06-07.** The
  repository now contains the first-party React Native + Expo app in `mobile/`.
  `S-050` has `docs/plan/s-050-mobile-client.md` + `docs/tasks/s-050-mobile-client.md` and
  is a hard consumer of the `S-040` gateway (ADR-024): a first-party device must
  terminate in the session-gateway trust boundary and must not hold long-lived
  tokens. `S-070` (JWKS) remains recommended before production device login. Stack:
  React Native + Expo.
- ADR-021 is generalized to all non-upload intake; ADR-019/020/022 are scoped to the
  deferred `S-095` live-recording sub-case (their technical decisions are unchanged).
- **ADR candidates for product-layer phases (X22/X23/X24).** These are now all closed:
  - **X22 → X-S-100-1:** ✅ closed by ADR-027 (S-100-T0b). Org-membership guard + `workspaces:*` scopes delivered in S-100. Open follow-ups: X-S-100-3 (role extensions), X-S-100-4 (auth server scope config).
  - **X23 → X-S-160-1:** ✅ closed by ADR-030 (S-160-T0b). Review/publication gate model fixed before S-160 schema/runtime work.
  - **X-S-160-2:** ✅ closed 2026-06-13 (S-160-T8). E2E mock-gateway review/notification fixtures and Maestro review flow (`mobile/maestro/review.yaml`) authored and passing. BDD mapping rows (SC-REVIEW-1/2/3, SC-PUBLISH-1/2, SC-NOTIFY-1) closed with executable evidence.
  - **X-S-160-3:** open, now owned by `S-150-T6` — S-140 creates review tasks from real subtitle readiness and `S-140-T5b-a` added nullable `review_tasks.subtitle_artifact_id`, but it still enqueues `None` and cannot bind decisions to a regenerated translation/dub set. `S-150-T0` ratified a normalized exact-artifact/version binding; T6 must be decomposed (provisional RRI 71) and implemented before this item can close.
  - **X24 → X-S-110-1:** ✅ closed by ADR-028 (S-110-T0b). Voice-consent ledger + TTS fail-closed precondition fixed before S-110 implementation.
- **Tiger Style adoption evaluation (X26, 2026-08-20).** The requirements-defining
  report is complete and evidence-backed (`docs/proposals/tiger-style-adaptation-evaluation.md`),
  but it is explicitly a precursor artifact, not a plan. It must be **re-evaluated**
  before drafting `docs/plan/tiger-style-adaptation.md`: the owner needs to resolve
  D1–D3, and since the underlying Rust/Python surfaces (especially `S-150`'s parked
  `translation-worker-py`/`tts-worker-py`) keep changing, the evaluation's evidence
  base (file:line citations, function-length inventory, dependency-pinning state)
  should be spot-checked for drift rather than assumed still accurate at that point.
- **Gemma Push Reviewer role (X27, plan drafted 2026-08-19, revision r2).** A
  separate role from Gemma Reviewer code review: it audits the latest GitHub push
  after the pipeline completes, collects run metadata/job status/logs/annotations/
  artifacts, triages findings into RRI-scored candidate tasks, and dispatches pure
  Low eligible incidents to the existing Gemma Developer role. The plan
  (`docs/plan/gemma-push-reviewer-role.md`) is audit-review ready with its task
  ledger (`docs/tasks/gemma-push-reviewer-role.md`) already covering the model
  invocation contract, log-budget/redaction, quorum staging, and audit trail —
  but status is `proposed`, not approved for implementation. Scheduled to work on
  next, no fixed date; requires explicit owner approval before `T1` starts.
- **S-200 mobile auth re-architecture (planned 2026-06-17, ADR-031 Proposed).** A
  platform directive adapts mobile auth to the FenixCRM reference flow at full
  fidelity: `apps/api` issues its own HS256 JWT, the gateway becomes a transparent
  relay, and the device stores the token directly. This **inverts** ADR-023
  (resource-server-only, RS256) and ADR-024 (no token on device, opaque session) and
  amends ADR-029 (transport only). It is a deliberate, directive-driven security
  downgrade with the accepted regressions recorded in ADR-031 §Risk analysis. The
  initiative RRI is 109 (Excessive), so only the ADR + risk + decomposition package
  exists today; ADR-031 acceptance (S-200-T0) and every code task require explicit
  approval. Recommended hardening X-S-200-1 (RS256) and X-S-200-2 (revocation) remain
  open.

## Related

- `docs/plan/roadmap.md` — the live, token-lean roadmap this file archives detail for
- `docs/plan/agents-override-sync.md` — why `AGENTS.override.md` concatenates the
  roadmap (and other governance docs) into a full-text session bundle for Codex
