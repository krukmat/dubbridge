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

Last consolidated 2026-09-05. This file intentionally keeps only current status,
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
| **S-230** | POC v1 deployment (Digital Ocean): production images, migration runner, S3/Spaces credential wiring, real readiness probes, production deployment descriptor, first deploy + end-to-end smoke | S-010, S-030 Phase 3, S-080, S-120, S-125, S-130, S-140, S-160, S-200 | 🟡 in progress — T0–T4 are **done** as of 2026-08-24. `T3b` closed its 6/6 reopened S-150 subtitle-translation children; all 17 T4 children (`T4a`–`T4q`) are closed, including the T3b-triggered `T4m`/`T4n` translation bundle path. T4p's live Docker pipeline passed HP-1 (exit 0, `asset_transcription_status = ready`) and EC-1; migration evidence and cleanup were recorded, and T4q closed the parent/status sync. `T5`–`T9` remain pending. `T5a` (child contract freeze) closed **Done 2026-08-26** — owner
supplied the public hostname (`poc.iotforce.es`, DNS-verified) and all seven
frozen inputs. `T5b`'s first 2026-08-27 recompute (RRI 59 Complex, triggering
the unconditional RRI ≥ 56 decomposition gate into `T5b-i`/`T5b-ii`/`T5b-iii`)
was retracted the same day as miscalibrated — it manually forced the
`auth_security` penalty and raised D/K/P above the ADR-026 anchor-rubric
floor without the rubric's own file-anchored basis for doing so. The
corrected, floor-anchored recompute landed at **RRI 27 Moderate** as a single
task, no decomposition triggered (`docs/audit/s-230-t5b-rri.md`). Approved
and implemented 2026-08-27 (Claude Sonnet 5 direct, per owner routing
override citing production auth/secret-boundary criticality — no local
model): added `production.toml`'s missing `[auth]` block plus the T5a-frozen
values, authored `.env.example`, extended `config/README.md`'s parity table,
and fixed a `.gitignore` defect (`.env.*` was blanket-excluding
`.env.example` itself). Gemma Reviewer (`gemma4:26b-a4b-it-qat`, RRI 26-55
chain primary) passed 3/3 with 0 findings; owner verified and closed
**Done 2026-08-27**. `T5c` (production Compose + Caddy TLS descriptor)
closed **Done 2026-08-27** — Claude Sonnet 5 direct (owner routing
override), Gemma Reviewer PASS 0 findings both phases, owner-verified.
`T5d` (local descriptor evidence + aggregate status sync) recomputed at
**RRI 22 Low** (ledger's provisional estimate was 27 Moderate) and closed
**Done 2026-08-27** — structural `docker-compose config` render plus the
16/16-passing `AppConfig::validate()` fail-closed guard suite were accepted
as dry-run evidence in place of a literal production-environment local
boot, which is architecturally precluded by ADR-026's own localhost/
local-fs rejection; full image-boot readiness remains T6's scope against
real DO infrastructure. **T5 (parent) is now closed** — all four children
(T5a–T5d) done. `T6` (first deploy) is next, unstarted. Deployment-enablement slice: makes the already-closed pipeline publicly runnable on a Digital Ocean droplet; adds no new technology beyond Redis (already in use). Full history incl. gap findings G10–G13: `docs/audit/roadmap-history.md` § S-230. | `docs/plan/s-230-poc-v1-digitalocean.md`, `docs/tasks/s-230-poc-v1-digitalocean.md`, `docs/audit/s-230-t5b-rri.md` |
| **MVP0-P2P** | P2P-first invited playback: maintainable mobile/Bare runtime foundation, isolated replication proof, encrypted publication, invite control plane, verified local package playback, and no-HTTP-fallback certification | S-120, S-125, S-127, S-160; accepted ADR-043 + separately approved P1 children before source work; separate audience-delivery ADR before P2 | 🟡 P0 and P1 closed — Android-only Bare worklet build/runtime proof passed; P1 closed `[x] Done` 2026-09-01 (7/8 children PASS; P1.F3b itself stays `not PASS`, non-blocking, deferred into X29 — see below). Revised P1 (RRI 94) and ADR-043 were approved on 2026-08-27. P1.F1, P1.F2, and P1.F3a.1 are closed PASS after owner verification; P1.F3a.2 closed Done 2026-08-27, retiring the P0 scaffold (`AndroidBareRuntimeProbe`, custom bridge/protocol, inline worklet) with `P2PDevelopmentHarness` as the sole diagnostic path. P1.F3b (RRI 24 Low) was implemented and audited 2026-08-27 but is **not PASS**: its dependency/build audit retained every contested item on evidence — notably `react-native-b4a`, which has zero JS imports yet is a `peerOptional` of `b4a` selected by `b4a`'s `react-native` export condition and wired by autolinking, so removing it would have degraded the RPC data path silently — leaving nothing to remove and the lockfile unchanged; its Android build/ping and the executed `useLegacyPackaging` native A/B are folded into X29. Its child `P1.F3b-fix-1` (RRI 17 Low) closed **Done 2026-08-28** — fixed a real Metro-bundling blocker in `mobile/src/p2p/runtime/protocol.ts` (TypeScript import-equals syntax) found once emulator access arrived, regenerated the drifted `worklet.bundle.js`, and certified coverage (27/27 P2P tests, typecheck clean, no bundle drift). It did **not** unblock the on-device run, which is root-caused to an upstream `bare-module@6.3.2` defect and stays in X29. P1.A1 (Hyperdrive/Corestore Android bundle smoke proof) was decomposed 2026-08-28 at owner request into four Low-band (RRI 0-25) children — `P1.A1a`-`P1.A1d` — after its parent-level Gemma phase-1 review passed 3/3 with 2 minor consensus findings, both incorporated into the children's acceptance criteria. `P1.A1a` (dependency add) closed PASS/Done 2026-08-28; P1.A1b, P1.A1c, and P1.A1d all closed PASS/Done 2026-08-30 after explicit owner verification, closing the P1.A1 parent. **P1.A2** (transient seed lifecycle + residue cleanup, RRI 46 Med-high) closed PASS/Done 2026-08-31 after a D14 cross-provider phase-2 review found and required repair of 3 BLOCKING correctness gaps (traversal guard, swallowed close-failure, uninvoked janitor) before owner verification. **P1.B1** (isolated Hyperswarm replication transport) closed `[x] Done` 2026-08-31 via **retrospective closure**: its implementation had already landed on `feature/p2p-mvp-core` before the task ledger was updated from "Deferred"; this session reconstructed the RRI report, implementation record, Reflection log, and coverage certification against the delivered code, independently re-ran all verification, and disclosed a real governance gap — the post-implementation RRI recomputes to **59 Complex** (crossing the decomposition-before-implementation threshold, driven mostly by a `many_files` penalty from mechanical maintainability-gate splits) against the stale 55 Med-high prospective estimate the work was actually authored under. The owner reviewed and accepted this as a one-time retrospective disposition, not a precedent (`docs/audit/mvp0-p2p-p1-b1-rri.md`, `docs/audit/mvp0-p2p-p1-b1-implementation.md`). Two items were explicitly carried forward to **P1.B2**: the transport layer's `byte_count: 0` hardcode (byte/hash verification is P1.B2's own designed scope) and a direct unit-test coverage gap for the Hyperswarm connection/timeout logic, currently exercised only through higher-layer mocks. **P1.B2** (verification, reconnect, and fail-closed witness; prospective RRI 56 Complex, decomposed before implementation into 12 children `a-0` through `f`) closed all 12 children PASS by 2026-09-01, closing with the prospective RRI kept as-is (a post-implementation recompute scoped to the 10 files actually touched gives RRI 35 Moderate, recorded as context only, not adopted). **P1 itself closed `[x] Done` on 2026-09-01**: 7/8 children (F1, F2, F3a, A1, A2, B1, B2) PASS, with P1.F3b's residual `not PASS`/X29 status accepted by the owner as non-blocking; parent-level 5-pass Reflection log against the original parent reflection plan, full unit coverage certification, and owner final verification recorded in `docs/tasks/mvp0-p2p-p1-replication.md` § Owner final verification. P1 closing PASS is **not** authorization to start P2 source work. iPhone/iOS remains deferred and no product P2P runtime or network activity is active outside these bounded proof runners. **P2–P7 still have no plan file**, but their design inputs and per-phase HP/EC scope were transcribed into canonical docs on 2026-08-28 (`docs/plan/mvp0-p2p-design-inputs.md`, expanded `docs/tasks/mvp0-p2p-first.md` § Deferred task acceptance summaries) and the required audience-delivery ADR is drafted as **ADR-044 (`Proposed`)** — `P2` stays unpresentable until it is accepted (see § Known planning gaps). | `docs/plan/mvp0-p2p-first.md`, `docs/tasks/mvp0-p2p-first.md`, `docs/plan/mvp0-p2p-design-inputs.md`, `docs/plan/mvp0-p2p-p1-replication.md`, `docs/tasks/mvp0-p2p-p1-replication.md`, `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`, `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`, `docs/audit/mvp0-p2p-p1-f1-implementation.md`, `docs/audit/mvp0-p2p-p1-f2-implementation.md`, `docs/audit/mvp0-p2p-p1-f3a1-implementation.md`, `docs/audit/mvp0-p2p-p1-f3b-implementation.md`, `docs/audit/mvp0-p2p-p1-b1-rri.md`, `docs/audit/mvp0-p2p-p1-b1-implementation.md`, `p2p-mvp/` |

> **MVP0-P2P ADR-044 update (2026-09-05):** D1 grant composition closed with
> owner-selected `O3 parallel`; D2 key/device envelope closed with owner-selected
> `K1`; and D3 publication/outbox semantics closed with owner-selected `O4`
> (transactional outbox as durable authority, queue as an optional accelerator,
> and PostgreSQL reconciliation as the recovery safety net). ADR-032 remains
> unchanged. ADR-044 remains `Proposed`; D4 acceptance is the next and only ADR
> closure gate before P2 may be planned/presented. No P2 source work is authorized.
>
> **MVP0-P2P update (2026-08-30):** `P1.A1b.0` (RRI 10 Low,
> documentation/contract only) closed PASS, freezing the host-to-Bare
> storage/RPC boundary. `P1.A1b` closed PASS at RRI 50 Med-high with focused
> checks passing and an explicit owner waiver for its residual no-action
> phase-2 finding; see its forced-closure record. `P1.A1c` (RRI 28 Moderate)
> closed PASS after a Matias-selected cloud fallback, typed error coverage,
> Phase-2 review, and owner verification. `P1.A1d` re-ran and consolidated the
> P1.A1 focused Jest evidence; the owner verified P1.A1 PASS on 2026-08-30.
> P1.A2 (RRI 46 Med-high) was approved 2026-08-30 and implemented via
> ADR-038 Amendment 4 Low-band decomposition (2/4 candidate subtasks
> delegated Low, 2/4 routed cloud) plus a direct cloud-takeover tramo.
> Phase-1 review: Gemma PASS. Phase-2 review: D14 (cross-provider, Codex)
> found 3 BLOCKING + 2 MAJOR findings, all repaired or dispositioned as
> accepted-follow-up. 3-pass Reflection and full unit coverage
> certification are complete. Owner-verified and closed **Done 2026-08-31**
> (`docs/tasks/mvp0-p2p-p1-replication.md` § P1.A2 § Owner final
> verification). `P1.B1` (Isolated Hyperswarm replication transport,
> prospective RRI 55 Med-high) is now unblocked and requires its own
> current RRI/card/explicit owner approval before implementation starts —
> no P1.B1 source work has begun.
> This closure update supersedes the earlier baseline-row wording that P1.A1d
> was awaiting owner verification.
> The earlier decomposition summary's "Low-band children" wording is
> superseded by the final child scores: P1.A1b is RRI 50 Med-high and P1.A1c
> is RRI 28 Moderate.

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
| **X26** | 🟡 Tiger Style adoption for the Rust/Python backend: evidence-based gap analysis complete (`docs/proposals/tiger-style-adaptation-evaluation.md`, R1–R13). Owner resolved all three decision points 2026-08-30 — **D1** `assert!` always-on at rights/finalize/playback-grant/audit boundaries; **D2** lower `too_many_lines` to 70 now (survey-then-decompose-then-flip); **D3** Postgres/Redis/MinIO integration tests mandatory in CI now. `docs/plan/tiger-style-adaptation.md` and `docs/tasks/tiger-style-adaptation.md` (`X26-T0`–`X26-T12`) are drafted; re-verification during planning found Postgres/Redis are already mandatory in CI, narrowing D3's remaining gap to MinIO/S3 (`X26-T5`). `X26-T3c` is decomposed into six Low-band domain-contract units (`T3c-a`, `b1`, `b2`, `c1`, `c2`, and `c3` all done) plus a separately governed audit-boundary integration (`T3c-d`); the contract matrix must first resolve the detected platform-ingest persistence mismatch. `T3c-b1` has an owner-recorded phase-2 review waiver after prolonged local-model failures; `T3c-b2`'s waiver was superseded by a genuine phase-2 `muse-glimmer` PASS (0 findings) obtained after a fresh per-task Ollama restart confirmed the local stack was healthy. `T3c-c1`'s first direct-implementation closure was retracted by the owner mid-session and redone from scratch through the real local Qwen pipeline (`scripts/delegate-low-rri.py --mode before-after`); attempt 1 hallucinated nonexistent `AuditEventKind` variants because the packet described the BEFORE block in prose instead of embedding its literal text (the script never injects `--before-file` into the model's prompt), attempt 2 (repair, 1/1 budget) succeeded once the packet embedded the literal block. Ahead of delegation, `crates/domain/src/audit.rs` (683 lines) was split into `crates/domain/src/audit/{mod,kind,event,tests}.rs` to satisfy the 500-line delegation file-size gate. Phase 1 (`muse-glimmer`) passed on the actual packet sent to Qwen; phase 2 failed 4/4 attempts on host memory saturation (not a content defect) and closed via an owner-issued urgency waiver (`docs/audit/gemma-review-overrides.md` row `X26-T3c-c1`). `T3c-c2` (workspace/consent no-correlation predicates, RRI 23 Low) succeeded on its first real Qwen delegation attempt with no repair needed; the per-task Ollama precheck reproduced `T3c-c1`'s host-memory-saturation symptom for `muse-glimmer` at the default `num_ctx=65536`, so both phase-1 and phase-2 review ran at a reduced `num_ctx=16384` instead, and phase 2 passed 3/3 usable with 0 findings — a genuine PASS, not a waiver. `T3c-c3` (review/playback/auth no-correlation predicates, RRI 23 Low) also succeeded on its first real Qwen delegation attempt with no repair needed; unlike `T3c-c1`/`-c2`, the per-task Ollama precheck this time found `muse-glimmer` healthy at the default `num_ctx=65536`, so both phase-1 and phase-2 review ran at full production context and phase 2 passed 3/3 usable with 0 findings. All six domain-contract units (`T3c-b1`, `-b2`, `-c1`, `-c2`, `-c3`) are now closed. `T3c-d` (audit-boundary integration), `T4` (retry-cap bounds), `T6`–`T11` (Python complexity gate + ASR worker hardening), and `T12` (S-150 forward-pointer, closed via owner wait-state waiver) were implemented directly on `main` on 2026-08-31 under explicit owner instruction to bypass the normal per-task presentation/approval and band-routed review workflow for this batch; per-task implementation/incident notes are at `docs/audit/x26-t3c-correlation-contract.md` and `docs/audit/x26-t{4,5,6,7,7-implementation-incidents,8,9,10,11,12-forward-pointer-closure}.md`. Independent verification on 2026-09-01 found the Rust and Python changes correct against their docs (one minor/theoretical gap: `asset_id` isn't checked by three no-correlation audit predicates, unreachable today). X26 is otherwise closed; R13 folds into S-150 `T4`–`T7` per the T12 forward-pointer, independently of the rest. | X26 implementation is complete; no further X26 task remains open. The two CI gaps this row previously flagged are now resolved: `deny`'s RUSTSEC-2026-0258 (h2) + yanked `chacha20` findings were cleared by `31b25eb`, and `workspace_test.rs`'s migration-reset race was hardened by `fb0b92f`. `main`'s CI is red again as of 2026-09-01 for three unrelated, newly-diagnosed reasons tracked at `X28` — none reopen X26. X26-T4 also has an unresolved acceptance-criteria deviation (no ADR-018 audit row on retry exhaustion) requiring owner ratification. Full detail: `docs/audit/x26-verification-2026-09-01.md`, `docs/audit/roadmap-history.md`. |
| **X27** | 🟡 Gemma Push Reviewer remediation: a baseline is deployed, but the 2026-08-24 audit reopened T1/T1B/T2/T3/T4/T5/T7 because real quorum, model-visible/redacted evidence, fail-closed RRI planning, durable work-item follow-through, bounded Low repair, frontier/HITL handoff, and trusted/idempotent publication are incomplete. r5 rescored the aggregate at RRI 96 and decomposed it into T12-T19. Proposed ADR-042 separates evaluator, deterministic controller, implementer, and acceptor; it keeps pure-Low fixes phase-1/phase-2 reviewed and binds D14/frontier routes to HITL + ADR-039 selection. | Owner decides ADR-042/T11 first. No runtime remediation is approved; after acceptance, present and approve each T12-T19 task separately. `docs/plan/gemma-push-reviewer-role.md` r5, `docs/tasks/gemma-push-reviewer-role.md`, ADR-034/039/042. |
| **X28** | 🟡 `main` CI red (`test`, `coverage`, `qa-docs`) as of 2026-09-01, confirmed pre-existing and unrelated to PR#6 (`feat/ckg-context-provider`, merged `f3adf34`). Three independent root causes, each diagnosed in `docs/audit/ci-red-findings-2026-09-01.md` and fixed under `docs/tasks/ci-red-fixes-2026-09.md` (CIRF-T1/T2/T3, all RRI 0-25 Low, delegated to and implemented by local Qwen Developer with Muse Glimmer phase-1/phase-2 review, both PASS 0 findings each): (1) `test` — `apps/api/src/routes/auth.rs`'s `migrate_and_reset` truncates shared tables against the one test DB with no per-test isolation; **CIRF-T3** applied the fast-unblock fix (`-- --test-threads=1` on `qa-test`'s Makefile recipe, mirroring `qa-coverage`'s existing pattern) — verified via a full local `make qa-test` run (60/61 tests passing; the one failure, `apps/worker-runner`'s `translation_fanout_tests::ec1_partial_claim_leaves_other_target_working`, was confirmed pre-existing and unrelated by reproducing it identically against clean `main`, not introduced by CIRF-T3); the durable per-test DB isolation redesign was subsequently approved by the owner (unique-per-test seed data: delete the shared `TRUNCATE ... RESTART IDENTITY CASCADE` from every affected setup helper, since all tables use UUID PKs) and recalculated at RRI 67 Complex, decomposed into 14 file-level subtasks (13 Low-band via `scripts/delegate-low-rri.py`, 1 Moderate-band originally routed to `run_local_task.py`) — tracked as **CIRF-T4**, now **done** (all 15 files closed as of 2026-09-04: the 14 originally-scoped files, plus a 15th, `apps/api/tests/notifications_api_test.rs`, found mid-implementation with the identical pattern, predating this session, owner-acknowledged and delegated under the same effort). The 14th file, `crates/db/src/user_account.rs` (RRI 29, floored by the `crates/db` anchor rubric per ADR-006/018 — re-evaluated on request and confirmed correctly Moderate, not split to Low), hit an operational `run_local_task.py` implementer timeout and was completed via an ADR-039 human-selected cloud fallback (Codex CLI, `gpt-5.6-terra`/`medium`), with Gemma phase-1/phase-2 both PASS; (2) `coverage` — **CIRF-T1** updated `apps/cli/tests/migrate_test.rs`'s stale `assert_eq!(count, 29, ...)` to `31`, verified against a real migrated local database; (3) `qa-docs` — **CIRF-T2** added `fetch-depth: 0` to the `qa-docs` job's checkout step in `.github/workflows/ci.yml`, mirroring the `maintainability`/`peer-workflow-review` jobs; verified locally via `make qa-docs` and a YAML-parse check. Validating CIRF-T4 with a real `cargo test --workspace --all-features` run (no `--test-threads=1`) surfaced 5 further racy tests confined to `apps/api/src/routes/auth.rs` (shared literal emails across tests plus absolute audit-event-count assertions racing other tests emitting the same event kind) — tracked as **CIRF-T5** (RRI 30 Moderate, decomposed per-function into 6 Low-band subtasks, RRI 13 each, per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band decomposition`), **implemented and independently verified** (8/8 `auth.rs` tests passing, 5/5 consecutive parallel runs), **pending owner sign-off**. CIRF-T5 also found a second-order collision the first 5 subtasks missed (two of the fixed tests still raced against *each other*, since a shared-event-kind counter can't be scoped by email without a schema change) — resolved by a 6th subtask (T5-6) using an existing per-email-scoped helper instead. Full evidence: `docs/tasks/ci-red-fixes-2026-09.md` § CIRF-T5. Running the full workspace suite under CIRF-T5 verification surfaced one **further, separate, pre-existing** race outside CIRF-T4/T5's scope — `apps/api/src/routes/compliance_tests.rs::get_audit_timeline_handler_returns_owned_events` fails only under full-workspace parallel execution (passes in isolation and within its own module), indicating it races against a test in a different module sharing the same database. **CIRF-T5 is done — owner sign-off recorded 2026-09-05.** The `compliance_tests.rs` finding was root-caused the same day (**CIRF-T5-addendum**, no separate task card, owner-authorized direct fix): not a data race (the underlying query is correctly scoped by unique-per-test `asset_id`), but connection-pool exhaustion — `compliance_tests.rs::setup_pool` and `auth.rs`'s `TestContext::new`/`with_closed_audit_pool` each opened uncapped `PgPool::connect` pools (sqlx default `max_connections=10`), so 7+6 parallel tests could contend for up to ~130 connections. Fixed by capping each with `PgPoolOptions::max_connections` (2 or 1, sized to actual per-test usage). Verified: build/fmt/clippy clean, 4 additional full-workspace `cargo test --workspace --all-features` runs with zero failures. Full detail: `docs/tasks/ci-red-fixes-2026-09.md` § CIRF-T5-addendum. | CIRF-T1/T2/T3/T4/T5 are all implemented, reviewed where applicable, and locally verified; CIRF-T5 has owner sign-off. The CIRF-T5-addendum connection-pool fix is implemented and locally verified but not yet independently re-reviewed or owner-signed-off. Remaining: confirm a CI run on `main` goes fully green. Full evidence, exact line numbers, and per-task delegation/review records: `docs/audit/ci-red-findings-2026-09-01.md`, `docs/tasks/ci-red-fixes-2026-09.md`. |
| **X29** | Physical Android ping proof for `P2PDevelopmentHarness` after P0 scaffold retirement (MVP0-P2P P1.F3a.2-iv): `AndroidBareRuntimeProbe.tsx` and its two superseded tests are deleted and the Jest characterization suite (`__tests__/p2p/`, 27/27 passing) proves the harness's logic, but no on-device `initialize → ping → shutdown` run has been performed on real Android hardware — this agent session has no device/emulator access. Deferred by owner request to a future general verification pass rather than blocking this task's other closure gates. **Extended 2026-08-27 by P1.F3b**, which has no device access either and folds two more items into this same pass: (a) `npm run android:p2p-dev` must build and complete a bounded `initialize → ping → shutdown`, confirming the renamed script/env gate (`EXPO_PUBLIC_P2P_DEV_HARNESS`) actually starts the harness; (b) the executed `useLegacyPackaging` on/off native A/B — currently justified only by static mechanism proof (bare-kit ships Bare native addons as jniLibs, and the flag governs whether they are extracted to disk for dynamic loading). **Reopened with emulator access 2026-08-28**: a `fenix_t7` Android 34 emulator became available; `P1.F3b-fix-1` (RRI 17 Low) fixed a real Metro-bundling blocker in `protocol.ts` (TS import-equals syntax), verified working. But the `initialize → ping → shutdown` run still cannot complete — reproduced and root-caused as a confirmed **upstream** `bare-module@6.3.2` bundle-evaluation-order bug (fixed in `bare-module@6.4.0`, one week later, no `react-native-bare-kit` release since `0.15.0` — the version pinned here and still the npm-latest — has picked the fix up). A minimal, dependency-free test bundle reproduces the identical crash, ruling out `protocol.ts`/`worklet.ts` content. Full trace: `docs/audit/mvp0-p2p-p1-f3b-implementation.md` § 9. The device-proof criteria remain blocked, now for this upstream reason rather than lack of device access. **Resolution path decided 2026-08-28:** the self-built `libbare-kit.so` alternative was evaluated and **rejected** by the owner — technically feasible (upstream CI recipe is public, and `bare-kit`'s `^bare-module@6.0.1` range resolves to the fixed `6.4.0` today), but it would transfer BoringSSL CVE-patching responsibility onto this repository with no assigned owner, contradict ADR-043's accepted decision to consume the vendor's native artifact rather than build it, and require `patch-package`/a fork to survive `npm install` — all to unblock criteria the roadmap already scopes as non-blocking. Evaluation and decision: `docs/audit/mvp0-p2p-p1-f3b-implementation.md` § 9.3. | Repository owner: **decided 2026-08-28 — continue deferring to the general hardware pass**; no self-build task opened. Revisit only if a real product need comes to depend on the harness proof, or if `react-native-bare-kit` has still not published a release bundling `bare-module ≥ 6.4.0` when P1's device-dependent criteria become blocking rather than deferrable. Handoff steps recorded in `docs/audit/mvp0-p2p-p1-f3a2-decomposition.md` § F3a.2-iv and `docs/audit/mvp0-p2p-p1-f3b-implementation.md` §§ 6, 9. Blocks only EC-F3a.2/HP-F3a.2's and HP-F3b's device-proof criteria, not the rest of P1.F3a.2/F3b closure. |

## Known planning gaps

- `S-xxx` numbering is canonical. Update this map whenever a phase, dependency, or
  ADR materially changes; do not introduce new active `P*` or bare `S0`–`S9` phase IDs.
- `S-070` (JWKS / production identity hardening) and `S-170`/`S-180` (human review
  and publication runtime) still need plan/task ledgers before execution.
- **MVP0-P2P `P2`–`P7`** (encrypted publication, invite/claim + key envelope,
  mobile verified sync, loopback HLS gateway, dashboard, no-HTTP-fallback
  certification) still have **no plan file**. Partially closed 2026-08-28:
  their design inputs are now transcribed into
  `docs/plan/mvp0-p2p-design-inputs.md` (use cases CU-01–CU-04, MVP-0 scope,
  the twelve global invariants with per-invariant adoption status, acceptance
  gates G0–G7, the control/data-plane split, package model, and the
  explicitly non-binding invite/RPC surfaces), and
  `docs/tasks/mvp0-p2p-first.md` § Deferred task acceptance summaries now
  carries per-phase objective, scope boundaries, and full HP/EC sets instead
  of one-liners — so P2–P7 can be analyzed and presented without opening the
  untracked external `p2p-mvp/` package. **Still missing:** a `docs/plan/`
  file per phase and acceptance of the audience-delivery ADR. ADR-044 remains
  `Proposed`, but D1 (`O3 parallel`), D2 (`K1`), and D3 (`O4`) are resolved as
  of 2026-09-05. D4 acceptance is now the only ADR-044 closure gate blocking
  P2 planning/presentation. P1 (or any of its children) reaching PASS is
  **not** authorization to start P2 source work — each of P2–P7 still needs
  its own plan, RRI, Compact Approval Task Card, and explicit HITL approval
  per `docs/plan/mvp0-p2p-first.md` § Execution sequence. The phase-1/phase-2
  peer-review waiver in `docs/audit/mvp0-p2p-review-exception.md` does not
  cover this planning gate.
- Full historical detail behind every closed gap above (the S-090 replan, X22–X24
  ADR closures, the S-200/ADR-031 mobile-auth decision, etc.):
  `docs/audit/roadmap-history.md`.
- **Nemotron → Devstral local-implementer migration (not scheduled):** a
  prepared migration bundle exists at
  `pending/dubbridge_devstral_main_migration/` to replace
  `nemotron-3.5-lightning:30b-a3b-q4_K_M` with
  `devstral-small-2:24b-instruct-2512-q4_K_M` as the RRI 26–40 (and
  `GO_LOCAL`-authorized 41–45) local implementer, and raise its context
  baseline from 32K to 128K tokens. Locked decisions, scope, and the
  deterministic substitution helper are in the bundle's `00_README.md`,
  `01_AGENT_EXECUTION_INSTRUCTIONS.md`, and `02_CHANGE_MATRIX.md`. Explicitly
  deferred — do not apply now; the owner will schedule this separately.

## Related

- `docs/audit/roadmap-history.md` — archived consolidation changelog, design
  rationale, and detailed per-slice status narrative trimmed from this file