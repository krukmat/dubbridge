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

Last consolidated 2026-08-24. This file intentionally keeps only current status,
dependencies, and links — full consolidation changelog, design rationale, and
detailed per-slice history live in `docs/audit/roadmap-history.md`.

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
| **S-050** | First-party mobile client (React Native + Expo) | S-040-T7; S-070 recommended for production device login | ✅ done — T0–T5 complete; auth transport superseded by ADR-031/S-200 (opaque `session_ref` → backend-issued bearer JWT) | `docs/plan/s-050-mobile-client.md`, `docs/tasks/s-050-mobile-client.md` (ADR-024 → ADR-031) |
| **S-055** | Maestro screenshot / visual-audit suite | S-050 | ✅ done — Maestro suite captures login + home screenshots; `npm run screenshots` wired | `docs/plan/s-055-maestro-screenshot-suite.md`, `docs/tasks/s-055-maestro-screenshot-suite.md` |
| **S-060** | First-party mobile asset lifecycle: `GET /assets`, mobile list, upload→rights→finalize, BDD/Maestro, mock `/api/*` | S-050, S-055 infra, S-010 | ✅ done — mobile asset lifecycle (list/upload/rights/finalize) delivered with Maestro ingest coverage | `docs/plan/s-060-mobile-asset-lifecycle.md`, `docs/tasks/s-060-mobile-asset-lifecycle.md` |
| **S-070** | Production identity hardening (JWKS discovery, automatic key rotation, subject mapping if needed) | S-000 | ⬜ no plan yet | ADR-023 |
| **S-080** | Object storage switchover (MinIO/S3 behind `StorageAdapter`) | S-010-T4 | ✅ done — S3-compatible adapter, bounded-memory upload path, orphan reconciliation delivered | `docs/plan/s-080-object-storage-switchover.md`, `docs/tasks/s-080-object-storage-switchover.md` |
| **S-090** | Platform ingest (owner-authorized download: first supported provider) | S-000-T2, S-010, S-020; S-080 prudent before heavy writes | 🟡 REPLANNED 2026-05-31 — foundation T0/T0c/T1/T2 done; S-040/S-070/S-050 done; connector work (`C4`–`C7`) deferred | `docs/plan/stream-recording-ingest.md` |
| **S-095** | Stream recording ingest (RTMP/SRT live capture) | S-090 foundation | ⬜ deferred — built only for live-broadcast clients | `docs/plan/stream-recording-ingest.md` |
| **S-100** | Collaborative localization workspace: orgs, roles, projects, target languages, org authz, historical web prototype, mobile project surfaces | S-000, S-010, S-040, S-050; coordinates with S-055/S-060 | ✅ done — workspace API, authz, and mobile projects delivered; the historical web prototype was retired by S-105 | `docs/plan/s-100-collaborative-workspace.md`, `docs/tasks/s-100-collaborative-workspace.md` |
| **S-105** | Mobile workspace parity and authenticated web-console retirement | S-100, S-050, S-060 | ✅ done — org selection, members, target languages, compliance navigation, and web removal delivered | `docs/plan/s-105-mobile-workspace-parity.md`, `docs/tasks/s-105-mobile-workspace-parity.md` |
| **S-110** | Mobile compliance & consent center: audit/rights viewer, voice-consent ledger, fail-closed TTS precondition | S-105, S-010 audit/rights data | ✅ done — compliance/consent center delivered; T4 web dashboard cancelled and superseded by the mobile center; X11 closed at contract level | `docs/plan/s-110-compliance-consent-center.md`, `docs/tasks/s-110-compliance-consent-center.md` |
| **S-115** | Mobile UX foundation & design-system adoption: theme tokens + primitives, single "ink + teal" palette (ADR-029 mobile surface), safe-area correctness, consistent state/touch/accessibility, behavior- and testID-preserving migration | S-105, S-110 | ✅ done — design-system (tokens + primitives) + safe-area + accessibility migration delivered across all mobile screens | `docs/plan/s-115-mobile-ux-foundation.md`, `docs/tasks/s-115-mobile-ux-foundation.md` |
| **S-120** | Media preparation (ffprobe metadata + HLS transcode) | S-010, S-080 | ✅ done — probe/HLS persistence, finalize enqueue, worker execution, and evidence-driven readiness gating delivered | `docs/plan/s-120-media-preparation.md`, `docs/tasks/s-120-media-preparation.md` |
| **S-125** | HLS playback delivery (authorized `.m3u8` + segment serving) | S-120, S-080, S-160 review/publication gate contract | ✅ done — playback grants, rewritten manifests, and short-lived scoped segment references delivered; ADR-032 accepted | `docs/plan/s-125-hls-playback-delivery.md`, `docs/tasks/s-125-hls-playback-delivery.md` (ADR-032) |
| **S-127** | Mobile review player surface: playback API client, `<VideoPlayer>` primitive (expo-video), `ReviewDetailScreen` v2 with embedded HLS player, `AssetDetailScreen` Play entry | S-125, S-115, S-190 | ✅ done — playback API client, `<VideoPlayer>` primitive, and review/detail player surfaces delivered; Maestro `playback.yaml` authored, runtime execution pending a Java-capable environment | `docs/plan/s-127-mobile-review-player.md`, `docs/tasks/s-127-mobile-review-player.md` (ADR-032, ADR-029) |
| **S-130** | Processing / ASR (transcription) | S-100 target-language intent, S-120 | ✅ done — ASR domain/repository, preparation-ready enqueue, worker dispatch + readiness gating, and the Python `faster-whisper` worker delivered | `docs/plan/s-130-asr-transcription.md`, `docs/tasks/s-130-asr-transcription.md` |
| **S-140** | Subtitle generation | S-130 | ✅ done — subtitle generation pipeline delivered; `T3c-iv` and the `X-S-160-3`/`T5b` wiring-version follow-up explicitly deferred beyond slice closeout | `docs/plan/s-140-subtitle-generation.md`, `docs/tasks/s-140-subtitle-generation.md` |
| **S-150** | Translation + dubbing (TTS / voice cloning) | S-140, S-110 consent precondition | 🟡 in progress — T0 through `T3c` (Rust translation runtime persistence + readiness transition) are done, closing 6/6 of the `S-230-T3b` reopened child chain (`T2c-v` through `T3c`). `T3c` registered the fourth worker-runner Monitor worker and defined `DUBBRIDGE_TRANSLATION_WORKER_PATH`/`DUBBRIDGE_TRANSLATION_WORKER_PYTHON` for `S-230-T4`/`T5`/`T6` to consume. `T4`–`T7` (TTS/dubbed audio) remain parked, blocked on the ADR-028 consent seam. Full history: `docs/audit/roadmap-history.md` § S-150. | `docs/plan/s-150-translation-dubbing.md`, `docs/tasks/s-150-translation-dubbing.md`, `docs/plan/s-230-poc-v1-digitalocean.md`, `docs/tasks/s-230-poc-v1-digitalocean.md`, `workers/translation-worker-py`, `workers/tts-worker-py` |
| **S-160** | Human review & publication workspace: review tasks, decisions, publication gate, notifications, complete mobile surface | S-105, S-115; forward-integrates S-140/S-150 derived artifacts | ✅ done — review state machine, publication gate, notifications, and the complete mobile reviewer surface delivered | `docs/plan/s-160-review-publication-workspace.md`, `docs/tasks/s-160-review-publication-workspace.md` |
| **S-170** | Human review runtime (HITL execution over generated artifacts) | S-125, S-140, S-150, S-160 | ⬜ no plan yet | — |
| **S-180** | Publication runtime | S-125, S-170, S-160 publication gate | ⬜ no plan yet | — |
| **S-200** | Mobile credential login with backend-issued JWT (FenixCRM parity) | S-000, S-040, S-050 (re-architects their auth) | ✅ done — ADR-031 accepted; HS256 issuer + alg pinning, `user_account` migration, gateway relay, and mobile bearer auth runtime delivered | `docs/plan/s-200-mobile-jwt-credential-auth.md`, `docs/tasks/s-200-mobile-jwt-credential-auth.md` (ADR-031) |
| **S-205** | Mobile DESIGN.md adoption: agent-readable mobile design-intent contract, lint command, workflow integration, and playback-surface audit | S-115, S-190, S-127 | ✅ done — root `DESIGN.md` authored; `make qa-design` added as an opt-in gate; mobile UI workflow now reads `DESIGN.md` | `docs/plan/mobile-design-md-adoption.md`, `docs/tasks/mobile-design-md-adoption.md` |
| **S-210** | Mobile product experience (dashboard, ergonomics, media-first) | S-115, S-190, S-160, S-127 | ✅ done — Home became a live dashboard, bottom action bars landed, and screenshot-backed polish closed the post-S-190 audit | `docs/plan/s-210-mobile-product-experience.md`, `docs/tasks/s-210-mobile-product-experience.md` |
| **S-215** | Mobile streaming-style organization & continuity pass | S-210, S-125, S-160 | ✅ done — continuity-led Home, library IA, media-first detail/review context, and palette recalibration delivered | `docs/plan/s-215-mobile-streaming-organization-pass.md`, `docs/tasks/s-215-mobile-streaming-organization-pass.md` |
| **S-220** | Mobile dark theme — Netflix-style dark canvas | S-215 | ✅ done — dark canvas `#141414` + Netflix-red `#E50914` accent shipped; WCAG AA certified | `docs/plan/s-220-mobile-dark-theme.md`, `docs/tasks/s-220-mobile-dark-theme.md` |
| **S-230** | POC v1 deployment (Digital Ocean): production images, migration runner, S3/Spaces credential wiring, real readiness probes, production deployment descriptor, first deploy + end-to-end smoke | S-010, S-030 Phase 3, S-080, S-120, S-125, S-130, S-140, S-160, S-200 | 🟡 in progress — T0–T3 done; `T3b` (reopened S-150 subtitle-translation chain, 6/6 children) is **done** as of 2026-08-24; `T4` decomposed into 17 Low/S children (`T4a`–`T4q`) routed to local `qwen3.8:27b-mlx` + Muse Glimmer review; `T4m`/`T4n` (translation-bundle image + contract tests, conditional on `T3b`) are **done** as of 2026-08-24 now that `T3b` closed; `T4o` (full local image-pipeline contract case) is **done** as of 2026-08-24; `T4p` (local image evidence with live Docker pipeline) is **done** as of 2026-08-24 — HP-1 passed (exit 0, `asset_transcription_status = ready`), EC-1 passed, all four closure conditions met; `T4q` and `T5`–`T9` remain pending. Deployment-enablement slice: makes the already-closed pipeline publicly runnable on a Digital Ocean droplet; adds no new technology beyond Redis (already in use). Full history incl. gap findings G10–G13: `docs/audit/roadmap-history.md` § S-230. | `docs/plan/s-230-poc-v1-digitalocean.md`, `docs/tasks/s-230-poc-v1-digitalocean.md` |

`S-040` must be planned before building a first-party browser, operator-console, or
mobile auth flow; it does not block S-080 or S-090.

**Product-layer phases** (`S-100`, `S-105`, `S-110`, `S-160`) turn the governed
pipeline into a team-usable product: `S-100` is the collaboration foundation
(orgs/roles/projects/languages); `S-105` establishes mobile as the only
authenticated product UI (ADR-029); `S-110` gates TTS/dubbing on voice consent
ahead of `S-150`; `S-160` supplies the review/publication gate `S-170`/`S-180`
adopt; `S-125` supplies the shared HLS playback-delivery boundary those runtime
slices use. Captured by ADR-027, ADR-028, ADR-029, ADR-030, ADR-032.

`S-050` (mobile) consumes the `S-040` gateway as the transparent relay delivered
by S-200 (ADR-031): the device stores the backend-issued HS256 bearer JWT in secure
storage, and the gateway forwards authenticated requests to `apps/api`. The former
ADR-024 opaque-session transport is superseded historical context. `S-040-T7` and
`S-055` are complete. Full narrative: `docs/audit/roadmap-history.md`.

## S-030, S-090/S-095 design rationale

`S-030`'s environment-separation strategy (phasing, config-layering design) and the
rationale for splitting intake into primary `S-090` platform-download vs. deferred
`S-095` live-recording (including the `S-090` internal task map) are archived in
`docs/audit/roadmap-history.md`. The operative rule from that work is already
captured above under Governing principles and ADR-025/ADR-026.

## Cross-cutting obligations

| Item | Obligation | Owner / next action |
|------|------------|---------------------|
| **X1** | Reconcile `crates/audit` duplicate type | ✅ closed by T1 Task 5; H1 now owns central audit emission semantics |
| **X2** | Align docker-compose Rust pin with toolchain policy | ✅ closed by `S-030` Task 6 (`infra/local/docker-compose.yml` tracks `rust-toolchain.toml` = `stable`) |
| **X3** | Backfill remaining open ADR numbers only when real decisions are identified | layered fail-closed configuration & environment separation now recorded as ADR-026; owner-credential secret-store (X20) still open, ADR to be authored |
| **X4** | Persist pending upload sessions across API restarts | ✅ closed by T1 Task 1 |
| **X5** | Add TTL/cleanup for abandoned pending uploads | ✅ closed by T1 Task 2 |
| **X6** | Enforce the 90% coverage gate | ✅ closed by T1 Task 3 |
| **X7** | Prevent partial relational finalization and cleanup-vs-finalize blob loss | ✅ closed by H1 |
| **X8** | Centralize durable audit + tracing emission; do not use fire-and-forget governance audit | ✅ closed by H1 |
| **X9** | Add production object-store adapter, canonical storage-owned key construction, orphan reconciliation, and a streaming/presigned strategy that avoids buffering large uploads in API memory | `S-080` |
| **X10** | Resolve recording segment/upload/asset cardinality before recorder implementation | ✅ closed by `S-090` Task 0c |
| **X11** | Enforce consent and voice-cloning permissions before TTS derivatives | `S-110` defines the gate; `S-150` T0 ratified enqueue-time + dispatch-time enforcement but runtime implementation remains pending; `S-180` observes it at publication |
| **X12** | Preserve lineage and quality-gate transitions for every derived artifact | `S-120`–`S-180` |
| **X13** | Plan first-party browser auth through a session gateway / BFF | ✅ closed by `S-040` |
| **X14** | Plan JWKS rotation and production identity-provider integration | `S-070` |
| **X15** | Keep RTSP, HLS pull, WebRTC, and per-segment publication as explicit live-recording follow-ups | post-`S-095` backlog |
| **X16** | Move reusable finalize logic from `apps/api` into an app-neutral shared boundary | ✅ closed by H1 |
| **X17** | Enforce append-only rights rows and strict decoding of stored governance states | ✅ closed by H1 |
| **X18** | Wire container service DNS, database/Redis URLs, auth bootstrap, health checks, and version policy so documented local startup is reproducible | ✅ closed by `S-030` Tasks 2-6 for the documented local startup path |
| **X19** | Enforce fail-closed source authentication (RTMP stream key / SRT passphrase, credential redaction, `rtmp`/`srt` scheme allow-list) before any capture begins | `S-095` (domain T1 done, migration T2 done, recorder ex-T3, API ex-T6); ADR-022 |
| **X20** | Decide the secrets-store mechanism for owner-provided platform credentials (storage by reference, scope minimization, redaction); no dedicated ADR yet | `S-090-C1`–`S-090-C6` + `S-030` config/secret split; ADR-025 |
| **X21** | Make runtime configuration fail-closed and environment-explicit: no compiled environment-specific defaults; `DUBBRIDGE_ENV` required; production rejects localhost datastores, local-fs storage, absent auth, and pretty logs; committed non-secret per-env profiles separated from injected secrets; Compose is local-infra-only (ADR-026) | ✅ closed by `S-030` Tasks 1-6 |
| **X22** | Define the org/membership/role authorization model: multi-tenant boundary, RBAC scopes layered over ADR-023 principal, org-scoped API enforcement | ✅ closed by ADR-027 (S-100-T0b); org-membership guard + `workspaces:*` scopes delivered in S-100-T2/T3 |
| **X-S-100-3** | Non-hierarchical role extensions: current linear role order (`Viewer < Reviewer < Editor < Admin < Owner`) may not fit all future governance patterns; flat RBAC or per-resource role overrides deferred | open — revisit before S-110 membership model adds consent-specific roles |
| **X-S-100-4** | Configure external authorization server to issue `workspaces:write` and `workspaces:read` scopes; tests currently stub the verifier | open — required before workspace endpoints are usable in production deployment |
| **X23** | Define the review/decision/publication gate model: append-only decision ledger, fail-closed publication precondition (ADR-008 spirit), S-140/S-150 artifact contract | ✅ closed by ADR-030 (S-160-T0b); S-160-T1a/T1b/T1c/T2 consume it |
| **X24** | Define the voice-consent ledger and TTS precondition: append-only consent rows, evidence stored by reference (ADR-025 spirit), fail-closed gate before any TTS derivative; closes **X11** at the contract level | ✅ closed by ADR-028 (S-110-T0b); S-110-T1/T2 implemented it; `S-150-T0` ratified two-check runtime enforcement, while implementation remains pending behind S-150-T4/T5 |
| **X-S-150-1** | Future voice-consent hardening: consent-proof evidence lifecycle, automated real-stack checks, speaker/voice-profile scope, expiry/revocation effects on existing derivatives, and provider-side material governance | `S-150-T8` future High-RRI parent; coordinate `X-S-110-2`, `X-S-110-3`, X20, and S-180. Non-blocking for S-150 T1-T7 and does not reopen X24/X11 unless an approved future ADR changes the contract. |
| **X25** | Define and implement HLS playback delivery for prepared `.m3u8` manifests and segments without exposing raw object-store keys | ADR-032 created; implemented as `S-125` |
| **X26** | Tiger Style adoption for the Rust/Python backend: evidence-based gap analysis complete (`docs/proposals/tiger-style-adaptation-evaluation.md`, R1–R13). Blocked on three owner decision points (D1 always-on Rust assertions, D2 clippy `too_many_lines` tightening, D3 mandatory Postgres/Redis/MinIO CI) before a plan/task ledger is drafted; R13 can fold into S-150 `T4`–`T7` independently of D1–D3. | Owner sign-off on D1 (at minimum) required before `docs/plan/tiger-style-adaptation.md` is drafted; re-evaluate if repo state drifts materially from the evaluation's evidence base. Full detail: `docs/audit/roadmap-history.md`. |
| **X27** | 🟡 Gemma Push Reviewer remediation: a baseline is deployed, but the 2026-08-24 audit reopened T1/T1B/T2/T3/T4/T5/T7 because real quorum, model-visible/redacted evidence, fail-closed RRI planning, durable work-item follow-through, bounded Low repair, frontier/HITL handoff, and trusted/idempotent publication are incomplete. r5 rescored the aggregate at RRI 96 and decomposed it into T12-T19. Proposed ADR-042 separates evaluator, deterministic controller, implementer, and acceptor; it keeps pure-Low fixes phase-1/phase-2 reviewed and binds D14/frontier routes to HITL + ADR-039 selection. | Owner decides ADR-042/T11 first. No runtime remediation is approved; after acceptance, present and approve each T12-T19 task separately. `docs/plan/gemma-push-reviewer-role.md` r5, `docs/tasks/gemma-push-reviewer-role.md`, ADR-034/039/042. |

## Known planning gaps

- `S-xxx` numbering is canonical. Update this map whenever a phase, dependency, or
  ADR materially changes; do not introduce new active `P*` or bare `S0`–`S9` phase IDs.
- `S-070` (JWKS / production identity hardening) and `S-170`/`S-180` (human review
  and publication runtime) still need plan/task ledgers before execution.
- Full historical detail behind every closed gap above (the S-090 replan, X22–X24
  ADR closures, the S-200/ADR-031 mobile-auth decision, etc.):
  `docs/audit/roadmap-history.md`.

## Related

- `docs/audit/roadmap-history.md` — archived consolidation changelog, design
  rationale, and detailed per-slice status narrative trimmed from this file
