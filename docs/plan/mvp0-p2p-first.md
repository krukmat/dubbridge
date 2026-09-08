---
type: Plan
title: "Plan: MVP-0 — P2P-first invited playback"
status: in_progress
slice: MVP0-P2P
---

# Plan: MVP-0 — P2P-first invited playback

> **Task ledger:** `docs/tasks/mvp0-p2p-first.md`.
> **External input:** `p2p-mvp/` (integrity verified against its package manifest).
> **Status:** P0 is closed PASS. P1 is closed Done (2026-09-01) with its device-proof residual tracked in X29. ADR-043 and ADR-044 are Accepted. `P2.T0` is PASS with `AN-R1 + AN-A1`; `P2.T1a`-`P2.T1f` are **Done and owner-approved as P2.T1** (2026-09-06); and `P2.C0` is **PASS** (2026-09-06). P2.T2 is Done, including T2c-r and T2g recertification (2026-09-08); T4a is Done (2026-09-07). Remaining P2 work is T3a-T3d, T4b-T4f, T5a-T5d, and T6a-T6e (18 planned leaves). The October target is a controlled Android P2P beta/POC deployed through S-230 by 2026-10-30, not GA.
> iPhone/iOS support remains deferred by the repository owner.

## Objective

Deliver invited audience playback over a ciphertext-only P2P data plane while preserving the existing control-plane authorization, rights, review playback, and S-120 preparation pipeline.

Target product flow:

```text
owner upload
 -> existing rights/finalize
 -> S-120 prepared HLS Ready
 -> P2 encrypted P2P publication / P2P_READY
 -> P3 invitation + claim + K1 device envelope
 -> P4 verified local ciphertext sync
 -> P5 loopback HLS gateway
 -> existing VideoPlayer
 -> P6 product dashboard
 -> P7 no-HTTP-fallback certification
```

## Guardrails and design decisions

1. P0 is feasibility evidence, not product architecture.
2. P1 retired the temporary P0 scaffold behind accepted ADR-043.
3. Mobile composition remains `SafeAreaProvider -> AuthProvider -> P2PProvider -> RootNavigator`; navigation does not own the P2P runtime.
4. Product mobile owns one reproducibly bundled/versioned Bare worklet through `P2PService -> BareRuntimeClient`; normal mounting never starts networking.
5. P1 seed/client proof runners are development-only and absent from product APIs.
6. Existing authentication, rights/finalize, `StorageAdapter`, S-120 HLS, `VideoPlayer`, and mobile navigation seams are reused rather than replaced.
7. `PreparationStatus::Ready` remains S-120 HLS readiness. P2P publication is a separate durable predicate and must not delay S-120 Ready or downstream ASR.
8. **ADR-044 is Accepted.** D1 `O3 parallel`, D2 `K1`, and D3 `O4` constrain P2-P7.
9. P2P publication is ciphertext-only. Raw invite tokens, plaintext CKs, server KEKs, JWT-signing material, and device private keys are never persisted/logged outside their accepted boundaries.
10. Availability Node is now frozen by P2.T0 as `AN-R1`: a dedicated Node.js/TypeScript service using Hyperdrive/Hyperswarm, operationally independent from mobile Bare.
11. Publication-control authentication is frozen by P2.T0 as `AN-A1`: mTLS service identity on a private endpoint. Availability Node never owns PostgreSQL credentials, business authorization, plaintext CK/KEK, invite/viewer state, or backend signing authority.
12. T0 also froze semantic publication state `building -> publish_pending -> publishing -> reconciling -> ready`, with `failed` terminal only, plus the minimum P2 ADR-018 audit-event set. PostgreSQL/outbox remain authority; queue and Availability Node remain subordinate.
13. The owner waiver of phase-1/phase-2 peer review applies only to MVP0-P2P P0-P7 as recorded in `docs/audit/mvp0-p2p-review-exception.md`; it does not waive RRI, HITL, tests, Reflection, coverage, or owner verification.

## Execution sequence

```text
P0 Bare/RN compatibility                         ✅
 -> P1 mobile foundation + replication proof    ✅
 -> ADR-044 D1/D2/D3/D4                         ✅ Accepted
 -> P2.T0 Availability Node/O4 contract          ✅ AN-R1 + AN-A1
 -> P2.T1 durable publication + outbox           ✅ Done / owner-approved
 -> P2.C0 shared publication contract freeze     ✅ PASS
 -> P2.T2 K1 construction + T4a recovery kernel   Done
 -> P2.T3, T4b-f, T5, T6                         remaining encrypted-publication work
 -> P3 invitation/claim + K1 envelope
 -> P4 verified mobile ciphertext sync
 -> P5 loopback HLS gateway
 -> P6 dashboard
 -> P7 no-HTTP-fallback certification
```

Each executable parent is independently scored, presented, approved, implemented, verified, and synchronized under repository workflow.

## P0 / P1 result

- **P0:** Android Bare/Expo compatibility PASS accepted 2026-08-27.
- **P1:** `[x] Done` 2026-09-01. Maintainable composition/runtime ownership, reproducible RPC worklet, transient storage cleanup, Hyperdrive/Hyperswarm replication, digest verification, bounded reconnect, and teardown evidence are closed. P1.F3b's physical device-proof residual remains non-blocking for historical P1 closure; X29 is blocking for the October T7p/T9g release.
- **ADR-043:** Accepted and unchanged.

## P2 activation

P2 is no longer blocked on ADR acceptance. The activated artifacts are:

- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/audit/mvp0-p2p-p2-rri.md`
- `docs/audit/mvp0-p2p-p2-t0-approval-card.md`
- `docs/audit/mvp0-p2p-p2-t0-selection.md`

The unreduced P2 phase is RRI 131 Excessive and cannot execute directly. T0 and T1 are complete, and `P2.C0` passed on 2026-09-06. C0 froze `p2p-manifest-v1`, `p2p-aad-v1`, K1 custody, `availability-publication-v1`, P2 audit correlation, `p2p-ready-descriptor-v1`, golden fixtures, and exact T2-T6 path ownership. The canonical contract and leaf map are in `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md` and `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`; P2.T2 is Done, including T2c-r and T2g recertification (2026-09-08); T4a is Done (2026-09-07). Remaining P2 work is T3a-T3d, T4b-T4f, T5a-T5d, and T6a-T6e (18 planned leaves). C0 and P2.T1 remain closed.

## October 2026 release profile and S-230 dependency

The October goal is a controlled Android beta/POC for one owner, one invited viewer, one device, and a short video. The must-have path is P2 encrypted publication, P3 invite/claim and K1 device envelope, P4 verified full-package sync, P5 loopback playback through the existing `VideoPlayer`, P6 minimal product states/actions, and P7 certification with legacy HTTP/S3 audience-media delivery disabled.

S-230 may deploy and validate its existing base platform independently. Its **P2P go-live decision**, however, depends on the deployed P2P publication plane, an Android P2P release candidate, P2-P6 PASS, and P7 certification against the exact release artifact. P2 alone proves backend publication; it does not prove invited playback.

The deployment lane is intentionally deferred until the implementation surfaces
exist. S-230's local-stack development path (`T7local -> T7c`, validated
against `infra/local/docker-compose.yml`, added 2026-09-06) and MVP0-P2P
development (`P2 -> P3 -> P4 -> P5 -> P6`) converge on `S-230-T6p-a -> T6p-b ->
T6p-c -> T6p-d -> T7p -> P7 -> T9g`. `P2.C0 PASS` remains the frozen
contractual input, but is not by itself the activation gate for T6p-a. The
independent S-230 `T6 -> T7` Digital Ocean deploy is not a T6p-a
prerequisite, but T7 PASS is required before T7p. The T6p-a gate uses
local-stack T7local/T7c evidence so deployment-input planning does not
depend on a completed Digital Ocean deployment. T6p-d proves
only backend ciphertext publication plus durable `P2P_READY`; invited
playback is not claimed until the physical RC and exact-artifact P7/T9g
gates pass.

The October release does not include iOS, multi-device, email delivery, offline/background operation, progressive streaming, performance certification, TTS/dubbed audio, or managed deployment automation. `S-230-T7b`, `T8`, and `T8b` remain optional unless explicitly selected for demo polish.

### Calendar target

| Window | Required outcome |
|---|---|
| Sep 6-18 | Record T1 Done and `P2.C0 PASS`, execute P2 leaf-by-leaf, advance S-230 `T7local -> T7c` against the local Docker Compose stack, and resolve X29 |
| Sep 19-Oct 15 | Complete P2 and the P3-P6 development sequence; do not activate T6p-a while either development gate remains open. The independent S-230 `T6 -> T7` Digital Ocean deploy may proceed in this window or later without affecting T6p-a |
| Oct 16-21 | Freeze deployment inputs in T6p-a, then execute T6p-b/T6p-c and deploy the backend publication plane through T6p-d; complete base T6/T7 before the T7p join |
| Oct 22-26 | Build and prove the physical Android P2P release candidate in T7p |
| Oct 27-30 | Run P7 on the exact deployed artifacts, complete soak/rollback evidence, and issue the S-230 P2P GO/NO-GO |

If physical Android proof is not available by September 18, or the required development gate (S-230 `T7local -> T7c` PASS against the local Docker Compose stack, and MVP0-P2P `P2 -> P6` PASS) is not complete by October 15, October may ship only the base S-230 HTTP/HLS POC plus a clearly labeled backend P2P preview. It must not claim P2P invited playback.

## Parallel execution policy

Planning, interface definition, test design, and evidence preparation within the
active MVP0-P2P phase may run concurrently. With `P2.C0 PASS`, the K1 builder,
Availability Node, recovery kernel/client, S-120 characterization, and audit/test
harness are designed as disjoint workstreams with explicit joins. S-230 T6p
deployment planning is the exception: it remains deferred until S-230
T7local, T7c, and MVP0-P2P P2-P6 are PASS, so it is based on implemented surfaces
rather than provisional ones — this intentionally does not require the
independent S-230 T6 -> T7 Digital Ocean deploy.

The current repository workflow still executes one approved task ID at a time. Multiple agents may write source within one approved RRI 26-55 task only when ADR-040's split-authorship conditions are met: one orchestrator, a common base SHA, frozen interfaces, disjoint path ownership, one writer per path, and whole-task integration/verification. Running separate executable task IDs concurrently requires a separately accepted workflow amendment; this plan does not assume one.

## Affected module boundaries

| Boundary | P0/P1 result | P2+ role |
|---|---|---|
| Mobile composition root | ADR-043 ownership established | unchanged by P2 |
| `P2PProvider` / `P2PService` | inert stable runtime owner | P4+ sync/lifecycle coordination |
| `BareRuntimeClient` / product worklet | versioned product runtime boundary | P4/P5 ciphertext sync/local playback mechanics |
| `apps/api` / gateway | existing control plane | P3+ invitation/audience authorization; not P2 media transport |
| `apps/worker-runner` | existing S-120/ASR orchestration | P2 package build/dispatch/reconcile downstream of S-120 Ready |
| `StorageAdapter` | existing storage authority | P2 reads prepared HLS through bounded existing seams |
| PostgreSQL / `crates/db` | existing metadata authority | P2 publication state/outbox; later invitation/device metadata |
| queue/jobs | existing coordination seam | O4 optional acceleration only, never publication authority |
| Availability Node | absent before P2 | `AN-R1` Node.js/TS ciphertext-only executor, mTLS-controlled by `AN-A1` |

## Verification strategy

P2 must prove accepted O4 failure semantics rather than only the happy path: lost dispatch, duplicate delivery, unknown external result, remote-success/ACK-loss, local Ready-commit loss, and duplicate-after-Ready must converge on one logical package/K1 lineage without false readiness. It must additionally prove ciphertext-only publication and non-regression of S-120 Ready/ASR.

## Remaining phase-specific decisions

- **P2 remaining:** P2.T2 is Done, including T2c-r and T2g recertification (2026-09-08); T4a is Done (2026-09-07). Remaining P2 work is T3a-T3d, T4b-T4f, T5a-T5d, and T6a-T6e (18 planned leaves). C0 contracts and ownership remain frozen.
- **P4:** persistent product cache/device lifecycle, sign-out wipe, and background execution beyond P1 transient proof.
- **P7:** certification profile that disables legacy HTTP media routes without disabling control-plane APIs.

These are scoped downstream decisions under accepted ADR-044, not reasons to reopen D1-D4 or T0.

## P3-P7 phase planning index — 2026-09-08

| Phase | Plan | Planning ledger | Disposition |
|---|---|---|---|
| P3 | `docs/plan/mvp0-p2p-p3-invitation-envelope.md` | `docs/tasks/mvp0-p2p-p3-invitation-envelope.md` | Planned; activation gate remains P2 PASS; Accepted ADR-044 |
| P4 | `docs/plan/mvp0-p2p-p4-mobile-sync.md` | `docs/tasks/mvp0-p2p-p4-mobile-sync.md` | Planned; activation gate remains P3 PASS |
| P5 | `docs/plan/mvp0-p2p-p5-local-playback.md` | `docs/tasks/mvp0-p2p-p5-local-playback.md` | Planned; activation gate remains P4 PASS |
| P6 | `docs/plan/mvp0-p2p-p6-dashboard.md` | `docs/tasks/mvp0-p2p-p6-dashboard.md` | Planned; activation gate remains P3-P5 PASS |
| P7 | `docs/plan/mvp0-p2p-p7-certification.md` | `docs/tasks/mvp0-p2p-p7-certification.md` | Planned; activation gate remains P2-P6 PASS; S-230-T7p PASS |

Detailed phase plans and work-package ledgers now exist. Exact-path executable
leaf decomposition, per-parent/leaf RRI, ownership and elapsed-time estimates
remain activation work; no phase is approved for source execution by this update.
P2 still has 18 planned leaves after T2/T4a closure. The October calendar is a
target with unvalidated capacity, not a delivery guarantee. X29 is required by
2026-09-18; P2-P6 and T7local/T7c by 2026-10-15; base T6/T7 and T6p-a-d by
2026-10-21; T7p by 2026-10-26; P7/T9g by 2026-10-30. X28 closure and exact-release
CI green, rollback/log/security/soak evidence remain T9g gates. A missed gate
permits only an explicitly labeled base POC/backend preview, not an invited-playback claim.
