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
> **Planning gap:** `P2`–`P7` have no plan file yet — see
> `docs/plan/roadmap.md` § Known planning gaps for the tracked gap. Their
> design inputs are transcribed in `docs/plan/mvp0-p2p-design-inputs.md` and
> their scope/HP/EC summaries in `docs/tasks/mvp0-p2p-first.md` § Deferred
> task acceptance summaries, so they can be analyzed without the external
> package; the audience-delivery ADR required by design decision 9 below is
> drafted as `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`
> (**Proposed** — D1 grant composition closed on 2026-09-05 with
> `O3 parallel`; D2 key/device envelope closed with `K1`; D3 publication/outbox
> semantics closed with `O4`; `ADR044-D4` acceptance is the next gate and `P2`
> remains unpresentable until the ADR is accepted).

## Objective

Prove whether the current Expo/React Native client can host a maintainable Bare
runtime boundary and a bounded P2P replication path. P0 established native
compatibility; P1 establishes explicit mobile ownership, reproducible worklet
packaging, a versioned RPC contract, lifecycle/error handling, and an isolated
two-runtime proof without freezing spike scaffolding as product architecture.
Only after that foundation and the audience-delivery ADR gates pass may
separately approved tasks add encrypted publication, invitation access,
verified package sync, a local HLS gateway, and product UI.

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
8. `PreparationStatus::Ready` remains the S-120 HLS-readiness signal. D3
   selected a separate durable P2P readiness boundary: PostgreSQL authoritative
   publication state + transactional outbox, optional queue acceleration, and a
   PostgreSQL reconciler safety net. P2P publication must not delay S-120 Ready
   or downstream transcription enqueue.
9. ADR-032 remains authoritative for present review playback. Before P2P invite
   delivery can be implemented, ADR-044 must be accepted. Its first three
   architecture questions are resolved: D1 `O3 parallel` authorization, D2 `K1`
   key/device envelope, and D3 `O4` publication/recovery. D4 acceptance remains
   the explicit gate before `P2` may be presented.
10. P2P publication is ciphertext-only. Raw invitation tokens, plaintext content
   keys, JWT signing material, and device private keys must never be persisted or
   logged. The selected K1 contract is recorded in
   `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md`.
11. The Availability Node, when planned, may seed ciphertext only; it must not
   receive PostgreSQL, JWT, signing-key, invitation, business-authorization, or
   plaintext content-key authority. Under D3/O4 it also never owns product
   `P2P_READY` state.
12. The repository owner waived phase-1 and phase-2 peer review only for this
   MVP0-P2P slice on 2026-08-27. The bounded exception and controls that remain
   mandatory are recorded in `docs/audit/mvp0-p2p-review-exception.md`; it does
   not waive HITL approval, RRI, tests, Reflection, coverage, or owner
   verification.

## Execution sequence

```text
P0 Bare/RN compatibility (stop/go)
 → P1 maintainable mobile runtime foundation + isolated replication proof
 → ADR-044 D1 authorization / D2 key envelope / D3 publication contract
 → ADR044-D4 explicit ADR acceptance
 → P2 encrypted publication state/outbox
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
| `apps/worker-runner` / `StorageAdapter` | Untouched | Read prepared HLS, build encrypted package, publish from durable O4 intent |
| PostgreSQL / `crates/db` | Untouched | Publication state/outbox, invitation, and envelope metadata |
| Queue/job seam | Existing coordination seam | Optional O4 acceleration only; never publication authority |
| Availability Node | Absent | Ciphertext-only seed runtime / publication evidence source |

## Verification strategy

P0's repeatable Android native-development proof passed with the selected
versions and was accepted by the repository owner. P1 adds deterministic bundle
drift checks, protocol/lifecycle unit coverage, provider/service ownership tests,
transient-storage deletion evidence, and an Android seed/client replication
witness. A digest match is necessary but insufficient unless both runtime
sessions close and their run-scoped cache paths are verified absent.

Future P2 must additionally prove the D3/O4 crash and idempotency contract:
lost dispatch, duplicate delivery, unknown remote result, remote-success/ACK-loss,
and duplicate-after-Ready must converge on one logical package/K1 lineage without
false readiness.

## Status artifacts

- `docs/tasks/mvp0-p2p-first.md`
- this plan
- `docs/plan/mvp0-p2p-design-inputs.md`
- `docs/plan/roadmap.md`
- `docs/architecture.md`, `docs/adr/README.md`, accepted ADR-043, and proposed ADR-044
- D1/D2/D3 audit records under `docs/audit/`
- `p2p-mvp/RUN_STATE.json` and the P0 handoff required by the external package

## Deferred decisions

Every item below is carried as a numbered decision question in ADR-044
(`Proposed`), with the phase each one blocks. Resolved items remain listed to
preserve the decision trail.

- `[x]` P2P audience-delivery ADR and ADR-032 relationship — ADR-044 question 1
  resolved 2026-09-05 as `O3 parallel`; ADR-032 remains unchanged.
- `[x]` Content-key algorithm, envelope format, device-key generation/storage,
  and bounded revocation semantics — question 2 resolved as `K1`; full contract
  in `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md`.
- `[x]` Publication/outbox and recovery semantics — question 3 resolved as `O4`:
  PostgreSQL + transactional outbox authority, optional queue accelerator,
  reconciler safety net, same-lineage idempotency, and separate fail-closed
  `P2P_READY`; full contract in
  `docs/audit/mvp0-p2p-adr044-d3-publication.md`.
- ADR-044 acceptance — next closure task `ADR044-D4`; P2 remains blocked until
  acceptance.
- Persistent product cache, device identity, sign-out wipe, and background
  execution beyond P1's transient foreground proof.
- Availability Node deployment, authentication, observability, and operational
  ownership.
- P2P certification profile that disables legacy HTTP media routes without
  disabling control-plane APIs.
