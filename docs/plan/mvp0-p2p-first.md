---
type: Plan
title: "Plan: MVP-0 — P2P-first invited playback"
status: in_progress
slice: MVP0-P2P
---

# Plan: MVP-0 — P2P-first invited playback

> **Task ledger:** `docs/tasks/mvp0-p2p-first.md`.
> **External input:** `p2p-mvp/` (integrity verified against its package manifest).
> **Status:** P0 is closed PASS. P1 is closed Done (2026-09-01) with its accepted non-blocking device-proof residual tracked separately. ADR-043 is Accepted. ADR-044 completed D1 `O3 parallel`, D2 `K1`, D3 `O4`, and D4 acceptance on 2026-09-05 and is **Accepted**. P2 planning/decomposition is active. `P2.T0` is **PASS** with `AN-R1 + AN-A1`; `P2.T1` is the next explicit owner gate. No P2 source implementation is authorized yet.
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
 -> P2.T1 durable publication + outbox           ⏭ next owner gate
 -> P2.T2-T6 encrypted publication               blocked on dependencies/approvals
 -> P3 invitation/claim + K1 envelope
 -> P4 verified mobile ciphertext sync
 -> P5 loopback HLS gateway
 -> P6 dashboard
 -> P7 no-HTTP-fallback certification
```

Each executable parent is independently scored, presented, approved, implemented, verified, and synchronized under repository workflow.

## P0 / P1 result

- **P0:** Android Bare/Expo compatibility PASS accepted 2026-08-27.
- **P1:** `[x] Done` 2026-09-01. Maintainable composition/runtime ownership, reproducible RPC worklet, transient storage cleanup, Hyperdrive/Hyperswarm replication, digest verification, bounded reconnect, and teardown evidence are closed. P1.F3b's physical device-proof residual remains separately deferred and non-blocking.
- **ADR-043:** Accepted and unchanged.

## P2 activation

P2 is no longer blocked on ADR acceptance. The activated artifacts are:

- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/audit/mvp0-p2p-p2-rri.md`
- `docs/audit/mvp0-p2p-p2-t0-approval-card.md`
- `docs/audit/mvp0-p2p-p2-t0-selection.md`

The unreduced P2 phase is RRI 131 Excessive and cannot execute directly. T0 is complete. `P2.T1` is the next implementation parent and remains subject to its own exact-path RRI and HITL before source edits.

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

- **P2.T1-T6:** implementation details are gated per parent; T1 is next.
- **P4:** persistent product cache/device lifecycle, sign-out wipe, and background execution beyond P1 transient proof.
- **P7:** certification profile that disables legacy HTTP media routes without disabling control-plane APIs.

These are scoped downstream decisions under accepted ADR-044, not reasons to reopen D1-D4 or T0.
