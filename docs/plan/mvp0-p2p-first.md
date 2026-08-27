---
type: Plan
title: "Plan: MVP-0 — P2P-first invited playback"
status: in_progress
slice: MVP0-P2P
---

# Plan: MVP-0 — P2P-first invited playback

> **Task ledger:** `docs/tasks/mvp0-p2p-first.md`.
> **External input:** `p2p-mvp/` (integrity verified against its package manifest).
> **Status:** P0 is closed with an Android-only compatibility PASS accepted by
> the repository owner on 2026-08-27. P1 is planned but has not started; iPhone/iOS
> support is explicitly deferred by the repository owner.

## Objective

Prove whether the current Expo/React Native client can host a bounded Bare
worklet and RPC bridge. Only if that proof passes may later, separately approved
tasks add encrypted P2P package publication, invitation access, replication, a
local HLS gateway, and a minimal mobile surface.

The target product flow is owner upload → existing rights/finalize/S-120 →
encrypted P2P publication → invitation, and viewer claim → verified local
package → loopback HLS → the existing `VideoPlayer`.

## Guardrails and design decisions

1. P0 is a stop/go feasibility spike, not a product P2P implementation. It does
   not add Hyperdrive, HyperSwarm, invitations, keys, package persistence, or a
   local media gateway.
2. Reuse existing authentication, rights/finalize, `StorageAdapter`, S-120 HLS,
   `VideoPlayer`, and mobile navigation seams. New code must not replace them.
3. `PreparationStatus::Ready` remains the S-120 HLS-readiness signal. A later
   P2P publication workflow must introduce its own durable status and outbox;
   it must not delay S-120 readiness or its downstream transcription enqueue.
4. ADR-032 remains authoritative for present review playback. Before P2P invite
   delivery can be implemented, an ADR must define the new audience boundary:
   backend authorization and audit stay authoritative while P2P transports only
   ciphertext.
5. P2P publication is ciphertext-only. Raw invitation tokens, plaintext content
   keys, JWT signing material, and device private keys must never be persisted or
   logged. Key/envelope design is a prerequisite for P3, not an assumption.
6. The Availability Node, when planned, may seed ciphertext only; it must not
   receive PostgreSQL, JWT, signing-key, invitation, or plaintext content-key
   authority.
7. The repository owner waived phase-1 and phase-2 peer review only for this
   MVP0-P2P slice on 2026-08-27. The bounded exception and controls that remain
   mandatory are recorded in `docs/audit/mvp0-p2p-review-exception.md`; it does
   not waive HITL approval, RRI, tests, Reflection, coverage, or owner
   verification.

## Execution sequence

```text
P0 Bare/RN compatibility (stop/go)
 → P1 isolated replication proof
 → ADR + P2 encrypted publication state/outbox
 → P3 invitation/claim and key envelope
 → P4 mobile verified ciphertext sync
 → P5 loopback HLS gateway
 → P6 dashboard
 → P7 no-HTTP-fallback certification
```

Each arrow is a hard dependency and each task is presented, scored, reviewed,
and approved independently under the repository workflow.

## P0 scope

P0 may touch only the mobile dependency/configuration surface, a deliberately
small `mobile/src/p2p/` bridge/proof module, its tests, and this task's evidence.
It must establish a bounded `initialize → ping → shutdown` RPC lifecycle on
Android using the selected Bare integration and document the Android native
build prerequisites. iPhone/iOS support is not part of this P0 result.

It must not create a persistent device identity, contact a DHT, open a local
HTTP server, replicate media, or introduce any backend/API/database behavior.

## Affected module boundaries

| Boundary | P0 role | Later P2P role |
|---|---|---|
| `mobile/` Expo app | Validate a contained Bare worklet/RPC bridge | Own sync state and player handoff |
| `apps/api` / `apps/gateway` | Untouched | Control-plane authorization, descriptors, audit |
| `apps/worker-runner` / `StorageAdapter` | Untouched | Read prepared HLS, build encrypted package, publish through outbox |
| PostgreSQL / `crates/db` | Untouched | Publication, invitation, and envelope metadata |
| Availability Node | Absent | Ciphertext-only seed runtime |

## Verification strategy

P0 requires a repeatable Android native-development proof, unit coverage for the
bridge state/error surface, a typecheck, and an explicit PASS or STOP conclusion.
The technical proof passed with the selected versions and was accepted by the
repository owner. The exact package versions and Android native prerequisites
were selected after checking the current Expo SDK 56 / React Native 0.85
compatibility evidence.

## Status artifacts

- `docs/tasks/mvp0-p2p-first.md`
- this plan
- `docs/plan/roadmap.md` — synchronized with P0 closure and P1's approval gate
- `p2p-mvp/RUN_STATE.json` and the P0 handoff required by the external package

## Deferred decisions

- P2P audience-delivery ADR and ADR-032 relationship.
- Publication/outbox schema and recovery semantics.
- Content-key algorithm, envelope format, device-key generation/storage, and
  revocation semantics.
- Availability Node deployment, authentication, observability, and operational
  ownership.
- P2P certification profile that disables legacy HTTP media routes without
  disabling control-plane APIs.
