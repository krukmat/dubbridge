# DubBridge Roadmap (General Plan)

## Purpose

This is the canonical sequencing map for the platform. It records delivered
foundations, blocking hardening gates, product slices, and cross-cutting obligations
derived from `docs/architecture.md` and the ADR set. Individual execution plans live
in `docs/plan/<slice>.md`; this file explains how they fit together.

Last consolidated: 2026-05-31 after the roadmap/ADR/architecture review in
`docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md` (including the
same-day ADR-traceability follow-up G1–G4 in that file). Updated the same day
after H1 completion. Updated 2026-06-03: scoped P0 around environment separation
and fail-closed configuration (see "P0 strategy" below, principle added, and X21),
then synchronized after P0 Task 2 and Task 3 completion. Updated 2026-06-03 again:
added plan/task ledgers for P1 (first-party session gateway / BFF) and introduced
slice P3 (first-party mobile client, React Native + Expo) as a P1-gated consumer
(ADR-024). Updated 2026-06-03 once more after
P0 Task 5 moved local Compose under `infra/local/` and wired the opt-in `app`
profile to fail-closed local config. Updated again on 2026-06-03 after P0 Task 6
aligned the local Rust image with `rust-toolchain.toml` and added the committed-config
secret guard. Updated 2026-06-07 after P3 T0–T5 completion: the mobile app is now
implemented, tested, and reflected in the architecture/task status documents.

## Status legend
- ✅ Done · 🟡 In progress · ⬜ Not started · 📄 Planned (plan exists, not built)

## Governing principles

- Rust owns API, orchestration, persistence boundaries, governance, and quality
  gates; Python is isolated to ML workers (`docs/architecture.md`).
- PostgreSQL is the system of record for structured metadata; immutable binary
  artifacts live behind `StorageAdapter` with explicit lineage and checksums
  (ADR-006).
- Rights are a mandatory fail-closed precondition for every intake mode and every
  downstream derivative (ADR-008).
- Runtime configuration is fail-closed and environment-explicit: no environment-
  specific value is compiled into the binary; production refuses to boot on a missing
  required value or a local default (localhost datastore, local-fs storage, absent
  auth). Non-secret environment values live in committed per-environment profiles;
  secrets exist only in injected environment variables. Local Docker Compose is local
  infrastructure only and is never the production deployment descriptor (P0, ADR-026, X21).
- Governance-significant events require durable audit rows plus correlated
  structured tracing (ADR-018).
- API caller identity is verified at the Axum boundary; first-party browser access
  may add a session gateway without weakening the protected API (ADR-023, ADR-024).
- Every non-upload intake is authorized-only and fail-closed before any bytes move:
  - **Platform download (primary S3, ADR-025):** the content owner grants scoped
    access to their own platform account; credentials are stored by reference and
    redacted, and a session lacking valid rights or a valid owner credential is
    rejected before any download.
  - **Live capture (deferred S3b, ADR-022):** an RTMP/SRT source must pass a
    validated stream key or SRT passphrase, redacted from logs, before any bytes are
    captured.
  Both are intake-edge twins of the upload rights gate (ADR-008) and converge on the
  same producer-agnostic finalize boundary (ADR-021).

## Product pipeline

```text
intake -> ingestion + rights gate -> media preparation -> processing (ASR)
       -> subtitles -> translation + dubbing -> human review -> publication
```

Both intake modes converge on the same ingestion and rights boundary:

```text
API client -> S0 authenticated principal
                  |
        +-- direct upload ............... S1 (operational)
intake -+-- platform download ........... S3 (primary, planned: owner-authorized
        |                                  first supported provider -> download -> same gate, ADR-025)
        +-- live stream recording ....... S3b (deferred: RTMP/SRT -> recording -> same gate)
```

## Required foundation gates

These are not optional tuning. A downstream slice must not expand a reused path
while its governing invariant remains weaker than the ADR contract.

| Gate | Name | Depends on | Status | Why it blocks |
|------|------|------------|--------|---------------|
| **H1** | Governance atomicity + durable audit hardening | S1, S3 T0 | ✅ done | Closed on 2026-05-31. Finalize now commits relational writes atomically, cleanup coordination is locked against finalize, durable governance audit emission is centralized, and regression coverage locks rollback + concurrency invariants before S3 expands the path. |

Plan: `docs/plan/h1-governance-atomicity-hardening.md`

## Slice sequence

| Slice | Name | Depends on | Status | Plan |
|-------|------|------------|--------|------|
| **S0** | API client authentication + principal propagation | — | ✅ done | `docs/plan/s0-api-client-authentication.md` |
| **S1** | Asset ingestion + rights ledger (upload) | S0 T2 for HTTP endpoints | ✅ done | `docs/plan/s1-asset-ingestion-rights-ledger.md` |
| **T1** | Initial tuning / hardening backlog | S1 | ✅ done | `docs/plan/tuning-hardening.md` |
| **S2** | Object storage switchover (MinIO/S3 behind `StorageAdapter`) | S1 T4 | ⬜ no plan yet | — |
| **S3** | Platform ingest (owner-authorized download: first supported provider) | S0 T2, S1, H1; S2 prudent before heavy writes | 🟡 REPLANNED 2026-05-31 — foundation T0/T0c/T1/T2 done; P1/P2/P3 done; P4-P7 deferred for this phase | `docs/plan/stream-recording-ingest.md` |
| **S3b** | Stream recording ingest (RTMP/SRT live capture) — deferred sub-case | S3 foundation | ⬜ deferred — built only for live-broadcast clients (ex-T3–T8) | `docs/plan/stream-recording-ingest.md` |
| **S4** | Media preparation (ffprobe metadata + HLS transcode) | S1, S2 | ⬜ no plan yet | — |
| **S5** | Processing / ASR (transcription) | S4 | ⬜ worker contract only | `workers/asr-worker-py` |
| **S6** | Subtitle generation | S5 | ⬜ | — |
| **S7** | Translation + dubbing (TTS / voice cloning) | S6 | ⬜ worker contracts only | `workers/translation-worker-py`, `workers/tts-worker-py` |
| **S8** | Human review (HITL) | S6, S7 | ⬜ | — |
| **S9** | Publication | S8 | ⬜ | — |

## Supporting platform slices

These are real architecture work, but they do not sit on the linear media pipeline.

| Slice | Name | Depends on | Status | Source |
|-------|------|------------|--------|--------|
| **P0** | Environment separation & deployment runtime wiring (layered fail-closed config, compose = local-infra-only, auth bootstrap, service DNS, version policy) | S0, S1 | ✅ done — Phase 0 and Phase 1 task ledgers complete; later env-driven runtime behavior stays deferred to S2+/later phases | `docs/plan/p0-environment-separation.md`, `docs/tasks/p0-environment-separation.md` |
| **P1** | First-party session gateway / BFF | S0, external authorization-server contract | ✅ done — T0–T7 complete as of 2026-06-04; browser/cookie transport and full mobile-safe gateway transport delivered | `docs/plan/p1-session-gateway-bff.md`, `docs/tasks/p1-session-gateway-bff.md`, `docs/tasks/p1-t7-mobile-session-handoff.md` (ADR-024) |
| **P2** | Production identity hardening (JWKS discovery, automatic key rotation, subject mapping if needed) | S0 | ⬜ no plan yet | ADR-023 |
| **P3** | First-party mobile client (React Native + Expo) | P1 T7 (hard); P2 recommended for production device login | ✅ done — T0–T5 complete as of 2026-06-07 | `docs/plan/p3-mobile-client.md`, `docs/tasks/p3-mobile-client.md` (ADR-024) |
| **P3-V** | Maestro screenshot / visual-audit suite (mobile hardening backlog) | **P3 complete** (historical gate satisfied: P3 T4/T5 done 2026-06-07) | 📄 planned — authored 2026-06-07, not built; approved Option A (ADR-024-clean handoff-code, no JWT on device) + S2 (defer until after P3 T4); ready to start at V1 | `docs/plan/p3-maestro-screenshot-suite.md`, `docs/tasks/p3-maestro-screenshot-suite.md` |

`P1` must be planned before building a first-party browser, operator-console, or
mobile auth flow. It does not block S2 or S3.

`P3` (mobile) is a first-party interactive client and therefore a hard consumer of
the P1 gateway (ADR-024): the device must terminate in the same session-gateway
trust boundary as the web app and must not hold long-lived tokens. P1 was
completed for the browser/cookie transport on 2026-06-04; P3 T0 verified the
delivered surface was browser-oriented only. P1 T7 is the unblock, decomposed in
`docs/tasks/p1-t7-mobile-session-handoff.md`. T7.1 (contract definition) is
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
(2026-06-03): React Native + Expo,
coherent with the React line reserved in `web/README.md`. The mobile
app is now implemented in `mobile/` with gateway-backed auth, navigation, asset
list/detail surfaces, and deterministic Jest coverage. A planned
mobile-hardening sub-slice, **P3-V (Maestro screenshot / visual-audit suite,
`docs/plan/p3-maestro-screenshot-suite.md` + `docs/tasks/p3-maestro-screenshot-suite.md`)**,
is gated on **P3 T4** and approved with Option A (ADR-024 handoff-code bootstrap, no
JWT on device) + sequencing S2 (defer until after T4); it is documented, not built.

## P0 strategy: environment separation & fail-closed configuration

P0 makes the local ↔ production boundary explicit and hard to confuse. Today
`crates/config` compiles local defaults into the binary (`AppConfig::from_env` falls
back to `localhost` Postgres/Redis and `/tmp` storage), so a misconfigured production
process boots silently against development resources. P0 inverts this to the same
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
  selector). The selector seam is P0; the MinIO/S3 adapter itself is S2 (X9).
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
- Phase 2 (couples with S2): env-driven storage backend selector (X9) and env-driven
  observability format/exporter (ADR-018).
- Phase 3 (later): production deployment descriptor + secret-manager injection
  boundary; owner-credential secret-store decision (X20).
- Phase 4 (deferred): orchestration (k8s/Helm or Nomad), telemetry collector, config
  service — only if multiple live environments or teams justify it. Not assumed now.

The layered fail-closed configuration & environment-separation decision is recorded
in ADR-026. The owner-credential secret-store mechanism (X20) remains an open decision
and warrants its own ADR when authored (X3).

## Why platform ingest is S3 (and live recording is S3b)

**Replan 2026-05-31 (ADR-025).** The real S3 intake use case is owner-authorized
**platform download**: the content owner provides scoped credentials to their own
platform account and DubBridge downloads the owner's content on their behalf. This
is the primary S3 path. RTMP/SRT live capture
is demoted to a deferred sub-slice (**S3b**) for the minority of clients who produce
live broadcasts.

Intake (in either mode) widens the funnel and has no dependency on media preparation
or ML stages, so it belongs before S4-S9. Hard dependencies of the **primary
platform-download path**:

- S0 verified principals for Axum ingest endpoints (ADR-023).
- S1's reusable finalize path (`finalize_ingestion_core`) and `StorageAdapter`
  boundary (ADR-006, ADR-021) — reused producer-agnostically.
- A per-connector engine behind `crates/connectors` (`PlatformConnector` trait),
  mirroring the `crates/media` pure-builder / IO-executor boundary (ADR-025).
- Owner-credential handling stored by reference and redacted (ADR-025, ADR-018).
- H1 atomicity and durable-audit hardening before the reused finalize path expands.
- The completed YouTube spike (P2), which ruled out YouTube as the pinned
  backend-download v1 provider, and a new provider-capability spike (P4) before
  the first connector is built.

The **deferred S3b live-recording path** additionally needs the FFmpeg-subprocess
recorder (ADR-019), the segment/lifecycle model and T0c output contract (ADR-020),
and RTMP/SRT capture-edge authentication (ADR-022). Its domain + migration foundation
(T1/T2) is already built and shared with the primary path.

S2 remains a prudent predecessor because intake is the first sustained, high-volume
writer. The trait boundaries make S3 technically possible without S2, but building
retention and upload against the production-like MinIO/S3 adapter avoids rework.

## S3 internal task map (REPLANNED 2026-05-31, ADR-025)

The S3 ledger is `docs/tasks/stream-recording-ingest.md`. The primary intake use
case is owner-authorized **platform download**, not RTMP/SRT live capture. The
FFmpeg recorder (ex-T3–T8) is deferred to **S3b**.

```text
Shared foundation (DONE, reused by both paths):
  T0  reusable S1 finalize core
  T0b duplicate audit type removed (via T1-T5)
  H1  atomicity + durable-audit gate closed
  T0c (S3b only) HLS fMP4 staging + assembled MP4 contract fixed
  T1  domain: recording aggregate, ArtifactKind, audit generalization
  T2  migrations: recording_sessions + audit generalization

PRIMARY S3 — platform ingest (build P1 -> P7):
  P1 connector trait boundary (crates/connectors) + PlatformIngestSession domain
  P2 YouTube retrieval-mechanism spike (gate) -> DONE 2026-06-03
  P3 provider-path replan after YouTube spike -> DONE 2026-06-03
  P4 first supported-provider capability spike (gate) -> DEFERRED for this phase
  P5 first supported-provider connector v1 -> DEFERRED for this phase
  P6 PlatformIngestJob + download->bridge wiring + platform_ingest_sessions migration -> DEFERRED for this phase
  P7 API endpoints (/ingests/platform) -> DEFERRED for this phase

S3b — live recorder (DEFERRED): ex-T3 recorder crate, ex-T4 jobs/storage,
  ex-T5 bridge, ex-T6 API, ex-T7 worker, ex-T8 tests. Marked [~] REPLANNED.
```

`T9` (docker-compose Rust pin) is independent low-priority housekeeping.

## Cross-cutting obligations

| Item | Obligation | Owner / next action |
|------|------------|---------------------|
| **X1** | Reconcile `crates/audit` duplicate type | ✅ closed by T1 Task 5; H1 now owns central audit emission semantics |
| **X2** | Align docker-compose Rust pin with toolchain policy | ✅ closed by P0 Task 6 on 2026-06-03 (`infra/local/docker-compose.yml` now tracks `rust-toolchain.toml` = `stable`) |
| **X3** | Backfill remaining open ADR numbers only when real decisions are identified | layered fail-closed configuration & environment separation now recorded as ADR-026; owner-credential secret-store (X20) still open, ADR to be authored |
| **X4** | Persist pending upload sessions across API restarts | ✅ closed by T1 Task 1 |
| **X5** | Add TTL/cleanup for abandoned pending uploads | ✅ closed by T1 Task 2 |
| **X6** | Enforce the 90% coverage gate | ✅ closed by T1 Task 3 |
| **X7** | Prevent partial relational finalization and cleanup-vs-finalize blob loss | ✅ closed by H1 on 2026-05-31 |
| **X8** | Centralize durable audit + tracing emission; do not use fire-and-forget governance audit | ✅ closed by H1 on 2026-05-31 |
| **X9** | Add production object-store adapter, canonical storage-owned key construction, orphan reconciliation, and a streaming/presigned strategy that avoids buffering large uploads in API memory | S2 |
| **X10** | Resolve recording segment/upload/asset cardinality before recorder implementation | ✅ closed by S3 Task 0c on 2026-05-31 |
| **X11** | Enforce consent and voice-cloning permissions before TTS derivatives | S7, then S9 publication gate |
| **X12** | Preserve lineage and quality-gate transitions for every derived artifact | S4-S9 |
| **X13** | Plan first-party browser auth through a session gateway / BFF | P1 |
| **X14** | Plan JWKS rotation and production identity-provider integration | P2 |
| **X15** | Keep RTSP, HLS pull, WebRTC, and per-segment publication as explicit live-recording follow-ups | post-S3b backlog |
| **X16** | Move reusable finalize logic from `apps/api` into an app-neutral shared boundary | ✅ closed by H1 on 2026-05-31 |
| **X17** | Enforce append-only rights rows and strict decoding of stored governance states | ✅ closed by H1 on 2026-05-31 |
| **X18** | Wire container service DNS, database/Redis URLs, auth bootstrap, health checks, and version policy so documented local startup is reproducible | ✅ closed by P0 Tasks 2-6 on 2026-06-03 for the documented local startup path |
| **X19** | Enforce fail-closed source authentication (RTMP stream key / SRT passphrase, credential redaction, `rtmp`/`srt` scheme allow-list) before any capture begins | S3b (domain T1 done, migration T2 done, recorder ex-T3, API ex-T6); ADR-022 |
| **X20** | Decide the secrets-store mechanism for owner-provided platform credentials (storage by reference, scope minimization, redaction); no dedicated ADR yet | S3 P1–P6 + P0 (the config/secret split — committed non-secret profiles vs injected env secrets — is the boundary the store plugs into); ADR-025 |
| **X21** | Make runtime configuration fail-closed and environment-explicit: no compiled environment-specific defaults; `DUBBRIDGE_ENV` required; production rejects localhost datastores, local-fs storage, absent auth, and pretty logs; committed non-secret per-env profiles separated from injected secrets; Compose is local-infra-only (ADR-026) | ✅ closed by P0 Tasks 1-6 on 2026-06-03 |

## Known planning gaps

- **S3 replanned 2026-05-31 (ADR-025).** Primary path is owner-authorized platform
  download. P1/P2/P3 are complete; the remaining P4–P7 work is intentionally
  deferred for this phase. RTMP/SRT live recording (ex-T3–T8) is the deferred S3b
  sub-case.
- The shared foundation (T0/T0b/T0c/H1/T1/T2) is complete and reused by both paths.
  T0c only governs S3b (it fixed the live-recording output contract).
- The YouTube retrieval mechanism for the platform path was spiked on 2026-06-03.
  Result: official docs validate `resolve()` but not an API-driven backend
  `download()` path. YouTube is therefore deferred for backend-download in this
  slice; P4 is the next gate for selecting the first officially supported provider.
- The owner-credential secrets-store mechanism (X20) has no dedicated ADR yet and
  must be decided during P1–P6; P0 establishes the config/secret split it plugs into.
- S2, S4, and P2 need plan/task ledgers before execution. P0 now has
  `docs/plan/p0-environment-separation.md` + `docs/tasks/p0-environment-separation.md`
  with its current Phase 0 / Phase 1 scope complete. P1 now has
  `docs/plan/p1-session-gateway-bff.md` + `docs/tasks/p1-session-gateway-bff.md`
  (complete). S2 must include the object-store adapter, storage-key
  ownership, orphan reconciliation, and upload memory-safety strategy.
- **Mobile is slice P3, introduced 2026-06-03 and completed 2026-06-07.** The
  repository now contains the first-party React Native + Expo app in `mobile/`.
  P3 has `docs/plan/p3-mobile-client.md` + `docs/tasks/p3-mobile-client.md` and
  is a hard consumer of the P1 gateway (ADR-024): a first-party device must
  terminate in the session-gateway trust boundary and must not hold long-lived
  tokens. P2 (JWKS) remains recommended before production device login. Stack:
  React Native + Expo.
- Slice numbering is provisional. Update this map whenever a slice, dependency, or
  ADR materially changes.
- ADR-021 is generalized to all non-upload intake; ADR-019/020/022 are scoped to the
  deferred S3b live-recording sub-case (their technical decisions are unchanged).
