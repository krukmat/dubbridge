---
type: Roadmap
title: "DubBridge Roadmap (General Plan)"
---
# DubBridge Roadmap (General Plan)

## Purpose

This is the canonical sequencing map for the platform. It records delivered
foundations, blocking hardening gates, product phases, and cross-cutting obligations
derived from `docs/architecture.md` and the ADR set. Individual execution plans live
in `docs/plan/<slice>.md`; this file explains how they fit together.

Roadmap phases use a single canonical `S-xxx` identifier. Older `S0`/`P*`/`T*`
labels remain as legacy aliases in source plans and historical task ledgers until
those files are renamed, but new roadmap references should use `S-xxx`.

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
hidden inside later publication work.

## Status legend
- ✅ Done · 🟡 In progress · ⬜ Not started · 📄 Planned (plan exists, not built)

## Governing principles

- Rust owns API, orchestration, persistence boundaries, governance, and quality
  gates; Python is isolated to ML workers (`docs/architecture.md`).
- PostgreSQL is the system of record for structured metadata; immutable binary
  artifacts live behind `StorageAdapter` with explicit lineage and checksums
  (ADR-006).
- Prepared HLS packages are storage-backed artifacts, not direct client contracts.
  Playback of `.m3u8` manifests and segments must go through the `S-125` backend
  delivery boundary with readiness, authorization, expiry, and publication gates
  enforced fail-closed (ADR-032).
- Rights are a mandatory fail-closed precondition for every intake mode and every
  downstream derivative (ADR-008).
- Runtime configuration is fail-closed and environment-explicit: no environment-
  specific value is compiled into the binary; production refuses to boot on a missing
  required value or a local default (localhost datastore, local-fs storage, absent
  auth). Non-secret environment values live in committed per-environment profiles;
  secrets exist only in injected environment variables. Local Docker Compose is local
  infrastructure only and is never the production deployment descriptor (S-030, ADR-026, X21).
- Governance-significant events require durable audit rows plus correlated
  structured tracing (ADR-018).
- API caller identity is verified at the Axum boundary; first-party browser access
  may add a session gateway without weakening the protected API (ADR-023, ADR-024).
  **Superseded by ADR-031 (2026-06-17, S-200):** the directive adopts FenixCRM
  parity — `apps/api` issues its own HS256 JWT, the gateway becomes a transparent
  relay, and the mobile device holds the bearer token. ADR-023/ADR-024 are
  `Superseded by ADR-031`; the inversion is implemented by slice S-200.
- Every non-upload intake is authorized-only and fail-closed before any bytes move:
  - **Platform download (primary S-090, ADR-025):** the content owner grants scoped
    access to their own platform account; credentials are stored by reference and
    redacted, and a session lacking valid rights or a valid owner credential is
    rejected before any download.
  - **Live capture (deferred S-095, ADR-022):** an RTMP/SRT source must pass a
    validated stream key or SRT passphrase, redacted from logs, before any bytes are
    captured.
  Both are intake-edge twins of the upload rights gate (ADR-008) and converge on the
  same producer-agnostic finalize boundary (ADR-021).

## Product Pipeline

```text
S-000 auth -> S-010 ingestion + rights gate -> S-120 media preparation
       -> S-130 ASR -> S-140 subtitles -> S-150 translation + dubbing
       -> S-170 human review runtime -> S-180 publication

S-120 prepared HLS -> S-125 playback delivery -> S-170/S-180 playback consumers
```

Both intake modes converge on the same ingestion and rights boundary:

```text
API client -> S-000 authenticated principal
                  |
        +-- direct upload ............... S-010 (operational)
intake -+-- platform download ........... S-090 (primary, planned: owner-authorized
        |                                  first supported provider -> download -> same gate, ADR-025)
        +-- live stream recording ....... S-095 (deferred: RTMP/SRT -> recording -> same gate)
```

## Required foundation gates

These are not optional tuning. A downstream slice must not expand a reused path
while its governing invariant remains weaker than the ADR contract.

| Gate | Name | Depends on | Status | Why it blocks |
|------|------|------------|--------|---------------|
| **S-020 / H1** | Governance atomicity + durable audit hardening | S-010, S-090-T0 | ✅ done | Closed on 2026-05-31. Finalize now commits relational writes atomically, cleanup coordination is locked against finalize, durable governance audit emission is centralized, and regression coverage locks rollback + concurrency invariants before S-090 expands the path. |

Plan: `docs/plan/h1-governance-atomicity-hardening.md`

## Canonical Phase Sequence

| Phase | Name | Depends on | Status | Source |
|-------|------|------------|--------|--------|
| **S-000** | API client authentication + principal propagation | — | ✅ done — auth model superseded by ADR-031/S-200 (RS256 resource server → in-house HS256 issuer) | `docs/plan/s0-api-client-authentication.md` |
| **S-010** | Asset ingestion + rights ledger (upload) | S-000-T2 for HTTP endpoints | ✅ done | `docs/plan/s1-asset-ingestion-rights-ledger.md` |
| **S-020** | Ingestion hardening: pending-upload durability, cleanup, coverage, finalize atomicity, durable audit | S-010 | ✅ done | `docs/plan/tuning-hardening.md`, `docs/plan/h1-governance-atomicity-hardening.md` |
| **S-030** | Environment separation + deployment runtime wiring | S-000, S-010 | ✅ done — Phase 0 and Phase 1 complete; later env-driven runtime behavior stays deferred to S-080+ | `docs/plan/s-030-environment-separation.md`, `docs/tasks/s-030-environment-separation.md` |
| **S-040** | First-party session gateway / BFF | S-000, external authorization-server contract | ✅ done — browser/cookie + mobile-safe gateway transport delivered; transport superseded by ADR-031/S-200 (gateway → transparent relay) | `docs/plan/s-040-session-gateway-bff.md`, `docs/tasks/s-040-session-gateway-bff.md`, `docs/tasks/s-040-t7-mobile-session-handoff.md` (ADR-024 → ADR-031) |
| **S-050** | First-party mobile client (React Native + Expo) | S-040-T7; S-070 recommended for production device login | ✅ done — T0–T5 complete as of 2026-06-07; auth transport superseded by ADR-031/S-200 (opaque `session_ref` → backend-issued bearer JWT) | `docs/plan/s-050-mobile-client.md`, `docs/tasks/s-050-mobile-client.md` (ADR-024 → ADR-031) |
| **S-055** | Maestro screenshot / visual-audit suite | S-050 | ✅ done — V1–V8 complete as of 2026-06-12; two-phase Maestro suite captures `01_auth_login` + `02_home`; `npm run screenshots` wired | `docs/plan/s-055-maestro-screenshot-suite.md`, `docs/tasks/s-055-maestro-screenshot-suite.md` |
| **S-060** | First-party mobile asset lifecycle: `GET /assets`, mobile list, upload→rights→finalize, BDD/Maestro, mock `/api/*` | S-050, S-055 infra, S-010 | ✅ done — T0–T6 + X-P3F-1/X-P3F-2 complete as of 2026-06-12; `GET /assets/{id}` ownership-enforced; `postMultipart` uses `expo-file-system/legacy` uploadAsync; SC-INGEST-1/SC-INGEST-2 Maestro flows complete (6 phases in runner) | `docs/plan/s-060-mobile-asset-lifecycle.md`, `docs/tasks/s-060-mobile-asset-lifecycle.md` |
| **S-070** | Production identity hardening (JWKS discovery, automatic key rotation, subject mapping if needed) | S-000 | ⬜ no plan yet | ADR-023 |
| **S-080** | Object storage switchover (MinIO/S3 behind `StorageAdapter`) | S-010-T4 | ✅ done 2026-06-18 — T0–T6 complete; S3-compatible adapter, bounded-memory upload path, orphan reconciliation, and drift-gate false-positive fix delivered | `docs/plan/s-080-object-storage-switchover.md`, `docs/tasks/s-080-object-storage-switchover.md` |
| **S-090** | Platform ingest (owner-authorized download: first supported provider) | S-000-T2, S-010, S-020; S-080 prudent before heavy writes | 🟡 REPLANNED 2026-05-31 — foundation T0/T0c/T1/T2 done; S-040/S-070/S-050 done; later connector work deferred | `docs/plan/stream-recording-ingest.md` |
| **S-095** | Stream recording ingest (RTMP/SRT live capture) | S-090 foundation | ⬜ deferred — built only for live-broadcast clients | `docs/plan/stream-recording-ingest.md` |
| **S-100** | Collaborative localization workspace: orgs, roles, projects, target languages, org authz, historical web prototype, mobile project surfaces | S-000, S-010, S-040, S-050; coordinates with S-055/S-060 | ✅ done — T0–T7 complete as of 2026-06-12; workspace API, authz, mobile projects, and a historical web prototype delivered. The web artifacts were retired by S-105. | `docs/plan/s-100-collaborative-workspace.md`, `docs/tasks/s-100-collaborative-workspace.md` |
| **S-105** | Mobile workspace parity and authenticated web-console retirement | S-100, S-050, S-060 | ✅ done — T0–T3 complete 2026-06-13; organization selection, members, target languages, compliance navigation, mobile-only BDD evidence, and web removal delivered | `docs/plan/s-105-mobile-workspace-parity.md`, `docs/tasks/s-105-mobile-workspace-parity.md` |
| **S-110** | Mobile compliance & consent center: audit/rights viewer, voice-consent ledger, fail-closed TTS precondition | S-105, S-010 audit/rights data | ✅ done — T0–T3, T5, and T6 complete 2026-06-13; T4 web dashboard cancelled and superseded by the complete mobile center; X11 closed at contract level | `docs/plan/s-110-compliance-consent-center.md`, `docs/tasks/s-110-compliance-consent-center.md` |
| **S-115** | Mobile UX foundation & design-system adoption: theme tokens + primitives, single "ink + teal" palette (ADR-029 mobile surface), safe-area correctness, consistent state/touch/accessibility, behavior- and testID-preserving migration | S-105, S-110 | ✅ done — T0–T5 complete 2026-06-13; design-system (tokens + 7 primitives) + SafeAreaProvider + all 13 screens migrated; a11y pass + Maestro syntax valid; 10 suites / 117 tests green | `docs/plan/s-115-mobile-ux-foundation.md`, `docs/tasks/s-115-mobile-ux-foundation.md` |
| **S-120** | Media preparation (ffprobe metadata + HLS transcode) | S-010, S-080 | ✅ done 2026-06-19 — T1–T5c complete; preparation schema/lineage, probe persistence, HLS persistence, finalize enqueue, worker execution, and evidence-driven readiness gating delivered | `docs/plan/s-120-media-preparation.md`, `docs/tasks/s-120-media-preparation.md` |
| **S-125** | HLS playback delivery (authorized `.m3u8` + segment serving) | S-120, S-080, S-160 review/publication gate contract | ✅ done 2026-06-22 — T0–T5c complete; playback grants, rewritten manifests, short-lived scoped segment references, and ADR-032 acceptance are delivered and the canonical docs/status artifacts are synchronized | `docs/plan/s-125-hls-playback-delivery.md`, `docs/tasks/s-125-hls-playback-delivery.md` (ADR-032) |
| **S-127** | Mobile review player surface: playback API client, `<VideoPlayer>` primitive (expo-video), `ReviewDetailScreen` v2 with embedded HLS player, `AssetDetailScreen` Play entry | S-125, S-115, S-190 | ✅ done 2026-06-24 — T0/T0b/T1/T2b/T2a/T3/T4/T5 complete; 159 tests green; typecheck clean; Maestro playback.yaml authored; BDD mapping synchronized; Maestro runtime execution pending Java-capable environment | `docs/plan/s-127-mobile-review-player.md`, `docs/tasks/s-127-mobile-review-player.md` (ADR-032, ADR-029) |
| **S-130** | Processing / ASR (transcription) | S-100 target-language intent, S-120 | 🟡 plan + tasks authored 2026-06-25 — pending approval | `docs/plan/s-130-asr-transcription.md`, `docs/tasks/s-130-asr-transcription.md` |
| **S-140** | Subtitle generation | S-130 | ⬜ no plan yet | — |
| **S-150** | Translation + dubbing (TTS / voice cloning) | S-140, S-110 consent precondition | ⬜ worker contracts only | `workers/translation-worker-py`, `workers/tts-worker-py` |
| **S-160** | Human review & publication workspace: review tasks, decisions, publication gate, notifications, complete mobile surface | S-105, S-115; forward-integrates S-140/S-150 derived artifacts | ✅ done 2026-06-13 — T0–T8 complete; review state machine, publication gate, notifications, complete mobile reviewer surface, Maestro fixtures, and docs sync are delivered | `docs/plan/s-160-review-publication-workspace.md`, `docs/tasks/s-160-review-publication-workspace.md` |
| **S-170** | Human review runtime (HITL execution over generated artifacts) | S-125, S-140, S-150, S-160 | ⬜ no plan yet | — |
| **S-180** | Publication runtime | S-125, S-170, S-160 publication gate | ⬜ no plan yet | — |
| **S-200** | Mobile credential login with backend-issued JWT (FenixCRM parity) | S-000, S-040, S-050 (re-architects their auth) | ✅ done 2026-06-18 — T0–T7 complete; ADR-031 Accepted; HS256 issuer + alg pinning (T1), user_account migration + repo (T2), bcrypt + AuthService (T3), apps/api auth handlers (T4), gateway relay (T5), mobile bearer auth runtime (T6), BDD + Maestro + docs sync (T7) | `docs/plan/s-200-mobile-jwt-credential-auth.md`, `docs/tasks/s-200-mobile-jwt-credential-auth.md` (ADR-031) |
| **S-205** | Mobile DESIGN.md adoption: agent-readable mobile design-intent contract, lint command, workflow integration, and playback-surface audit | S-115, S-190, S-127 | ✅ done 2026-06-25 — root `DESIGN.md` authored; `make qa-design` added as an opt-in alpha gate; mobile UI workflow now reads `DESIGN.md`; playback audit completed with two narrow follow-up patches delivered as separate tasks | `docs/plan/mobile-design-md-adoption.md`, `docs/tasks/mobile-design-md-adoption.md` |
| **S-210** | Mobile product experience (dashboard, ergonomics, media-first) | S-115, S-190, S-160, S-127 | ✅ done 2026-06-28 — T0–T9 complete; Home became a live dashboard, bottom action bars landed, technical ids were demoted, and screenshot-backed polish closed the post-S-190 audit | `docs/plan/s-210-mobile-product-experience.md`, `docs/tasks/s-210-mobile-product-experience.md` |
| **S-215** | Mobile streaming-style organization & continuity pass | S-210, S-125, S-160 | ✅ done 2026-06-29 — T1–T8 complete; continuity-led Home, library IA, media-first detail/review context, playback/publication reliability, palette recalibration, screenshot evidence, and docs sync are delivered | `docs/plan/s-215-mobile-streaming-organization-pass.md`, `docs/tasks/s-215-mobile-streaming-organization-pass.md` |

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

## Cross-cutting obligations

| Item | Obligation | Owner / next action |
|------|------------|---------------------|
| **X1** | Reconcile `crates/audit` duplicate type | ✅ closed by T1 Task 5; H1 now owns central audit emission semantics |
| **X2** | Align docker-compose Rust pin with toolchain policy | ✅ closed by `S-030` Task 6 on 2026-06-03 (`infra/local/docker-compose.yml` now tracks `rust-toolchain.toml` = `stable`) |
| **X3** | Backfill remaining open ADR numbers only when real decisions are identified | layered fail-closed configuration & environment separation now recorded as ADR-026; owner-credential secret-store (X20) still open, ADR to be authored |
| **X4** | Persist pending upload sessions across API restarts | ✅ closed by T1 Task 1 |
| **X5** | Add TTL/cleanup for abandoned pending uploads | ✅ closed by T1 Task 2 |
| **X6** | Enforce the 90% coverage gate | ✅ closed by T1 Task 3 |
| **X7** | Prevent partial relational finalization and cleanup-vs-finalize blob loss | ✅ closed by H1 on 2026-05-31 |
| **X8** | Centralize durable audit + tracing emission; do not use fire-and-forget governance audit | ✅ closed by H1 on 2026-05-31 |
| **X9** | Add production object-store adapter, canonical storage-owned key construction, orphan reconciliation, and a streaming/presigned strategy that avoids buffering large uploads in API memory | `S-080` |
| **X10** | Resolve recording segment/upload/asset cardinality before recorder implementation | ✅ closed by `S-090` Task 0c on 2026-05-31 |
| **X11** | Enforce consent and voice-cloning permissions before TTS derivatives | `S-110` defines the gate, `S-150` enforces it, `S-180` observes it at publication |
| **X12** | Preserve lineage and quality-gate transitions for every derived artifact | `S-120`–`S-180` |
| **X13** | Plan first-party browser auth through a session gateway / BFF | ✅ closed by `S-040` |
| **X14** | Plan JWKS rotation and production identity-provider integration | `S-070` |
| **X15** | Keep RTSP, HLS pull, WebRTC, and per-segment publication as explicit live-recording follow-ups | post-`S-095` backlog |
| **X16** | Move reusable finalize logic from `apps/api` into an app-neutral shared boundary | ✅ closed by H1 on 2026-05-31 |
| **X17** | Enforce append-only rights rows and strict decoding of stored governance states | ✅ closed by H1 on 2026-05-31 |
| **X18** | Wire container service DNS, database/Redis URLs, auth bootstrap, health checks, and version policy so documented local startup is reproducible | ✅ closed by `S-030` Tasks 2-6 on 2026-06-03 for the documented local startup path |
| **X19** | Enforce fail-closed source authentication (RTMP stream key / SRT passphrase, credential redaction, `rtmp`/`srt` scheme allow-list) before any capture begins | `S-095` (domain T1 done, migration T2 done, recorder ex-T3, API ex-T6); ADR-022 |
| **X20** | Decide the secrets-store mechanism for owner-provided platform credentials (storage by reference, scope minimization, redaction); no dedicated ADR yet | `S-090-C1`–`S-090-C6` + `S-030` config/secret split; ADR-025 |
| **X21** | Make runtime configuration fail-closed and environment-explicit: no compiled environment-specific defaults; `DUBBRIDGE_ENV` required; production rejects localhost datastores, local-fs storage, absent auth, and pretty logs; committed non-secret per-env profiles separated from injected secrets; Compose is local-infra-only (ADR-026) | ✅ closed by `S-030` Tasks 1-6 on 2026-06-03 |
| **X22** | Define the org/membership/role authorization model: multi-tenant boundary, RBAC scopes layered over ADR-023 principal, org-scoped API enforcement | ✅ closed by ADR-027 (S-100-T0b, 2026-06-12); org-membership guard + `workspaces:*` scopes delivered in S-100-T2/T3 |
| **X-S-100-3** | Non-hierarchical role extensions: current linear role order (`Viewer < Reviewer < Editor < Admin < Owner`) may not fit all future governance patterns; flat RBAC or per-resource role overrides deferred | open — revisit before S-110 membership model adds consent-specific roles |
| **X-S-100-4** | Configure external authorization server to issue `workspaces:write` and `workspaces:read` scopes; tests currently stub the verifier | open — required before workspace endpoints are usable in production deployment |
| **X23** | Define the review/decision/publication gate model: append-only decision ledger, fail-closed publication precondition (ADR-008 spirit), S-140/S-150 artifact contract | ✅ closed by ADR-030 (S-160-T0b, 2026-06-13); S-160-T1a/T1b/T1c/T2 consume it |
| **X24** | Define the voice-consent ledger and TTS precondition: append-only consent rows, evidence stored by reference (ADR-025 spirit), fail-closed gate before any TTS derivative; closes **X11** at the contract level | ✅ closed by ADR-028 (S-110-T0b, 2026-06-12); S-110-T1/T2 implemented it; `S-150` will enforce it |
| **X25** | Define and implement HLS playback delivery for prepared `.m3u8` manifests and segments without exposing raw object-store keys | ADR-032 created; implement as `S-125` before S-170/S-180 runtime consumers |

## Known planning gaps

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
- `S-xxx` numbering is canonical. Update this map whenever a phase, dependency, or
  ADR materially changes; do not introduce new active `P*` or bare `S0`–`S9` phase IDs.
- ADR-021 is generalized to all non-upload intake; ADR-019/020/022 are scoped to the
  deferred `S-095` live-recording sub-case (their technical decisions are unchanged).
- **ADR candidates for product-layer phases (X22/X23/X24).** These are now all closed:
  - **X22 → X-S-100-1:** ✅ closed by ADR-027 (S-100-T0b). Org-membership guard + `workspaces:*` scopes delivered in S-100. Open follow-ups: X-S-100-3 (role extensions), X-S-100-4 (auth server scope config).
  - **X23 → X-S-160-1:** ✅ closed by ADR-030 (S-160-T0b). Review/publication gate model fixed before S-160 schema/runtime work.
  - **X-S-160-2:** ✅ closed 2026-06-13 (S-160-T8). E2E mock-gateway review/notification fixtures and Maestro review flow (`mobile/maestro/review.yaml`) authored and passing. BDD mapping rows (SC-REVIEW-1/2/3, SC-PUBLISH-1/2, SC-NOTIFY-1) closed with executable evidence.
  - **X-S-160-3:** open — S-160 operates on fixtures (`asset-seed-1`, `project-seed-1`). Once S-140/S-150 derived artifacts land, review tasks must be created against real derived-asset identities. Gate: S-140 or S-150 delivery.
  - **X24 → X-S-110-1:** ✅ closed by ADR-028 (S-110-T0b). Voice-consent ledger + TTS fail-closed precondition fixed before S-110 implementation.
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
