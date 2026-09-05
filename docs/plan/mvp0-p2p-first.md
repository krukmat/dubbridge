---
type: Plan
title: "Plan: MVP-0 — P2P-first invited playback"
status: in_progress
slice: MVP0-P2P
---

# Plan: MVP-0 — P2P-first invited playback

> **Task ledger:** `docs/tasks/mvp0-p2p-first.md`.
> **External input:** `p2p-mvp/` (integrity verified against its package manifest).
> **Status:** P0 is closed PASS. P1 is closed Done (2026-09-01) with its accepted
> non-blocking device-proof residual tracked separately. ADR-043 is Accepted.
> ADR-044 completed D1 `O3 parallel`, D2 `K1`, D3 `O4`, and D4 acceptance on
> 2026-09-05 and is now **Accepted**. The P2 architecture prerequisite is therefore
> satisfied. P2 now has its own plan/task/RRI decomposition at
> `docs/plan/mvp0-p2p-p2-encrypted-publication.md` and
> `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`; `P2.T0` is the next explicit
> owner gate. No P2 source implementation is authorized yet.
> iPhone/iOS support remains deferred by the repository owner.

## Objective

Deliver invited audience playback over a ciphertext-only P2P data plane while
preserving the existing control-plane authorization, rights, review playback, and
S-120 preparation pipeline.

P0 proved Android Bare/React Native compatibility. P1 established maintainable
mobile/Bare ownership and isolated verified replication. ADR-044 now defines the
accepted audience-delivery boundary. P2-P7 implement the product path in separately
planned and approved phases.

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
2. P1 retired the temporary P0 scaffold behind the accepted ADR-043 boundary.
3. Mobile composition remains `SafeAreaProvider -> AuthProvider -> P2PProvider ->
   RootNavigator`; navigation does not own the P2P runtime.
4. Product mobile owns one reproducibly bundled/versioned Bare worklet through
   `P2PService -> BareRuntimeClient`; normal mounting never starts networking.
5. P1's seed/client proof runners are development-only and absent from product APIs.
6. P1 transient proof storage does not pre-decide P4 product cache/device lifecycle.
7. Existing authentication, rights/finalize, `StorageAdapter`, S-120 HLS,
   `VideoPlayer`, and mobile navigation seams are reused rather than replaced.
8. `PreparationStatus::Ready` remains S-120 HLS readiness. P2P publication is a
   separate durable predicate and must not delay S-120 Ready or downstream ASR.
9. **ADR-044 is Accepted.** Its accepted core is D1 `O3 parallel` authorization,
   D2 `K1` key/device envelope, and D3 `O4` publication/recovery. P2 may now be
   planned/presented, but its own RRI/HITL is still mandatory before source work.
10. P2P publication is ciphertext-only. Raw invite tokens, plaintext CKs, server
    KEKs, JWT-signing material, and device private keys are never persisted/logged
    outside their accepted boundaries.
11. Availability Node may seed ciphertext only; it never owns PostgreSQL/business
    authorization, plaintext CK/KEK, user/invite state, or backend signing authority.
    Its concrete service trust/operation contract is the P2.T0 owner decision.
12. The owner waiver of phase-1/phase-2 peer review applies only to MVP0-P2P P0-P7
    as recorded in `docs/audit/mvp0-p2p-review-exception.md`; it does not waive RRI,
    HITL, tests, Reflection, coverage, or owner verification.

## Execution sequence

```text
P0 Bare/RN compatibility                         ✅
 -> P1 mobile foundation + replication proof    ✅
 -> ADR-044 D1/D2/D3/D4                         ✅ Accepted
 -> P2.T0 Availability Node/O4 contract          ⏭ next owner gate
 -> P2.T1-T6 encrypted publication               blocked on their own approvals
 -> P3 invitation/claim + K1 envelope
 -> P4 verified mobile ciphertext sync
 -> P5 loopback HLS gateway
 -> P6 dashboard
 -> P7 no-HTTP-fallback certification
```

Each arrow is a hard dependency. Each executable parent is independently scored,
presented, approved, implemented, verified, and synchronized under the repository
workflow.

## P0 / P1 result

- **P0:** Android Bare/Expo compatibility PASS accepted 2026-08-27.
- **P1:** `[x] Done` 2026-09-01. Maintainable composition/runtime ownership,
  reproducible RPC worklet, transient storage cleanup, Hyperdrive/Hyperswarm
  replication, digest verification, bounded reconnect, and teardown evidence are
  closed. P1.F3b's physical device-proof residual remains separately deferred and
  non-blocking under the existing roadmap disposition.
- **ADR-043:** Accepted and unchanged.

## P2 activation

P2 is no longer blocked on ADR acceptance. The activated plan is:

- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/audit/mvp0-p2p-p2-rri.md`
- `docs/audit/mvp0-p2p-p2-t0-approval-card.md`

The unreduced P2 phase is RRI 131 Excessive and therefore cannot execute directly.
It is re-scoped into T0-T6. `P2.T0` is a docs/architecture/security contract gate;
it freezes Availability Node runtime/auth, idempotency/observability, state semantics,
and the minimum ADR-018 audit inventory. T1-T6 remain independently gated.

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
| Availability Node | absent before P2 | ciphertext-only publication executor/evidence source |

## Verification strategy

P2 must prove the accepted O4 failure semantics rather than only the happy path:
lost dispatch, duplicate delivery, unknown external result, remote-success/ACK-loss,
local Ready-commit loss, and duplicate-after-Ready must converge on one logical
package/K1 lineage without false readiness. It must additionally prove ciphertext-
only publication and non-regression of S-120 Ready/ASR.

P3-P7 each add their own behavioral and end-to-end evidence; P7 is the phase that
proves the certified media path works with legacy HTTP media fallback disabled.

## Status artifacts

- `docs/tasks/mvp0-p2p-first.md`
- this plan
- `docs/plan/mvp0-p2p-design-inputs.md`
- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/plan/roadmap.md`
- `docs/architecture.md`
- `docs/adr/README.md`
- accepted ADR-043 and ADR-044
- D1-D4 and P2 planning audit records under `docs/audit/`

## Remaining phase-specific decisions

ADR-044 acceptance intentionally does not pre-decide all later implementation
choices:

- **P2.T0:** Availability Node runtime/deployment authentication, observability, and
  operational ownership; concrete O4 state semantics; P2 minimum audit inventory.
- **P4:** persistent product cache/device lifecycle, sign-out wipe, and background
  execution beyond P1 transient proof.
- **P7:** certification profile that disables legacy HTTP media routes without
  disabling control-plane APIs.

These are scoped downstream decisions under the accepted ADR, not reasons to reopen
D1-D4.
