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
> the repository owner on 2026-08-27. A maintainability review materially
> replanned P1 on 2026-08-27; the repository owner then approved revised P1 and
> accepted ADR-043. The earlier P1/P1.A approval route remains superseded. P1.F1
> was separately approved, implemented, and closed PASS after owner verification
> on 2026-08-27. P1.F2 may now be prepared/presented but its source execution is
> not authorized.
> iPhone/iOS support is explicitly deferred by the repository owner.

## Objective

Prove whether the current Expo/React Native client can host a maintainable Bare
runtime boundary and a bounded P2P replication path. P0 established native
compatibility; P1 must now establish explicit mobile ownership, reproducible
worklet packaging, a versioned RPC contract, lifecycle/error handling, and an
isolated two-runtime proof without freezing spike scaffolding as product
architecture. Only after that foundation passes may separately approved tasks
add encrypted publication, invitation access, verified package sync, a local HLS
gateway, and product UI.

The target product flow is owner upload → existing rights/finalize/S-120 →
encrypted P2P publication → invitation, and viewer claim → verified local
package → loopback HLS → the existing `VideoPlayer`.

## Guardrails and design decisions

1. P0 is a stop/go feasibility spike, not a product P2P implementation. It does
   not add Hyperdrive, HyperSwarm, invitations, keys, package persistence, or a
   local media gateway.
2. P0's `AndroidBareRuntimeProbe`, inline worklet source, and custom JSON
   request/reply protocol are compatibility evidence, not permanent P1/product
   architecture. P1 first preserves their proven behavior as characterization,
   then deletes those implementations and their obsolete test/config/script
   wiring after parity exists through the accepted ADR-043 boundary. Historical
   P0 audit evidence is retained. Native Bare dependencies/build settings remain
   only when the replacement runtime or a native A/B proof demonstrates a live
   requirement.
3. Cross-cutting mobile ownership belongs in the app composition root:
   `SafeAreaProvider → AuthProvider → P2PProvider → RootNavigator`.
   `RootNavigator` remains navigation-only. `P2PProvider` owns a stable
   framework-independent `P2PService`, which owns one product
   `BareRuntimeClient`; mounting it never starts network activity.
4. Worklet source is packaged reproducibly with `bare-pack`; host/worklet
   multiplexing uses `bare-rpc` under a typed, versioned DubBridge protocol.
   Fatal exceptions/rejections and Bare suspend/resume events are explicit
   lifecycle inputs, not incidental logging.
5. Product mobile targets one Bare worklet. P1's seed/client pair exists only in
   a development proof runner backed by a runtime factory and is not exposed by
   the product service API.
6. Synthetic fixture bytes may be generated in memory, but Hyperdrive/Corestore
   metadata and blocks use a run-scoped cache directory with close-before-delete,
   verified removal, and a bounded abandoned-run janitor. P1 creates no durable
   product cache or device identity.
7. Reuse existing authentication, rights/finalize, `StorageAdapter`, S-120 HLS,
   `VideoPlayer`, and mobile navigation seams. New code must not replace them.
8. `PreparationStatus::Ready` remains the S-120 HLS-readiness signal. A later
   P2P publication workflow must introduce its own durable status and outbox;
   it must not delay S-120 readiness or its downstream transcription enqueue.
9. ADR-032 remains authoritative for present review playback. Before P2P invite
   delivery can be implemented, an ADR must define the new audience boundary:
   backend authorization and audit stay authoritative while P2P transports only
   ciphertext.
10. P2P publication is ciphertext-only. Raw invitation tokens, plaintext content
   keys, JWT signing material, and device private keys must never be persisted or
   logged. Key/envelope design is a prerequisite for P3, not an assumption.
11. The Availability Node, when planned, may seed ciphertext only; it must not
   receive PostgreSQL, JWT, signing-key, invitation, or plaintext content-key
   authority.
12. The repository owner waived phase-1 and phase-2 peer review only for this
   MVP0-P2P slice on 2026-08-27. The bounded exception and controls that remain
   mandatory are recorded in `docs/audit/mvp0-p2p-review-exception.md`; it does
   not waive HITL approval, RRI, tests, Reflection, coverage, or owner
   verification.

## Execution sequence

```text
P0 Bare/RN compatibility (stop/go)
 → P1 maintainable mobile runtime foundation + isolated replication proof
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

| Boundary | P0/P1 role | Later P2P role |
|---|---|---|
| Mobile composition root | P0 host today; P1 moves auth/P2P providers above navigation | Own cross-cutting mobile service composition without transport logic in navigation |
| `P2PProvider` / `P2PService` | P1 establishes an inert, stable product runtime owner | Expose sync state/actions and coordinate authenticated lifecycle |
| `BareRuntimeClient` / one product worklet | P1 establishes reproducible bundle, versioned RPC, fatal and suspend/resume handling | Own mobile P2P engine mechanics behind the service boundary |
| P1 proof runner | Create isolated seed/client sessions only for explicit Android evidence | Absent from product API and normal startup |
| `apps/api` / `apps/gateway` | Untouched | Control-plane authorization, descriptors, audit |
| `apps/worker-runner` / `StorageAdapter` | Untouched | Read prepared HLS, build encrypted package, publish through outbox |
| PostgreSQL / `crates/db` | Untouched | Publication, invitation, and envelope metadata |
| Availability Node | Absent | Ciphertext-only seed runtime |

## Verification strategy

P0's repeatable Android native-development proof passed with the selected
versions and was accepted by the repository owner. P1 adds deterministic bundle
drift checks, protocol/lifecycle unit coverage, provider/service ownership tests,
transient-storage deletion evidence, and an Android seed/client replication
witness. A digest match is necessary but insufficient unless both runtime
sessions close and their run-scoped cache paths are verified absent.

## Status artifacts

- `docs/tasks/mvp0-p2p-first.md`
- this plan
- `docs/plan/roadmap.md` — synchronized with P1's architecture reapproval gate
- `docs/architecture.md`, `docs/adr/README.md`, and accepted ADR-043
- `p2p-mvp/RUN_STATE.json` and the P0 handoff required by the external package

## Deferred decisions

- P2P audience-delivery ADR and ADR-032 relationship.
- Persistent product cache, device identity, sign-out wipe, and background
  execution requirements beyond P1's transient foreground proof.
- Publication/outbox schema and recovery semantics.
- Content-key algorithm, envelope format, device-key generation/storage, and
  revocation semantics.
- Availability Node deployment, authentication, observability, and operational
  ownership.
- P2P certification profile that disables legacy HTTP media routes without
  disabling control-plane APIs.
