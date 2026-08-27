---
type: Plan
title: "Plan: MVP0-P2P P1 maintainable mobile foundation and replication proof"
status: approved_decomposed
slice: MVP0-P2P
parent_task: P1
---

# Plan: MVP0-P2P P1 — Maintainable mobile P2P foundation and replication proof

> **Parent task ledger:** `docs/tasks/mvp0-p2p-p1-replication.md`.
> **Dependency:** P0 closed PASS (Android-only) on 2026-08-27.
> **Status:** Materially replanned on 2026-08-27 after the owner requested a
> maintainable architecture rather than an extension of P0 scaffolding. The
> earlier P1 approval and P1.A card are superseded before source execution.
> Revised P1 and ADR-043 were approved by the repository owner on 2026-08-27.
> The parent approval authorizes child preparation/presentation only. P1.F1 was
> separately approved, implemented, and closed PASS after owner verification on
> 2026-08-27 within its frozen packaging/protocol scope. P1.F2 was separately
> approved, implemented, and closed PASS after owner verification on 2026-08-27.
> No P2P network activity or later-child source execution has started.

## Objective

Establish a maintainable Android mobile/Bare ownership boundary, then prove that
two isolated proof runtimes can seed and replicate a synthetic opaque fixture
through Hyperdrive and Hyperswarm, verify its SHA-256 digest, and remove all
run-scoped storage. P1 establishes reusable runtime architecture plus transport
evidence; it does not publish a DubBridge asset or create a product data plane.

## Boundaries and decisions

1. Accepted ADR-043 governs the P1 target. `App.tsx` is the composition root:
   `SafeAreaProvider → AuthProvider → P2PProvider → RootNavigator`.
   `RootNavigator` owns navigation, route selection, deep links, and
   navigation-coupled push behavior; it does not instantiate or manage Bare.
2. `P2PProvider` owns a stable, framework-independent `P2PService`, which owns
   one product `BareRuntimeClient`. Provider construction is network-inert.
   Runtime snapshots are externally subscribable so progress does not force
   whole-navigation rerenders.
3. P0's `AndroidBareRuntimeProbe`, inline worklet source, and custom JSON
   multiplexer are temporary compatibility evidence. P1 preserves the tested
   `initialize → ping → shutdown` behavior as characterization while replacing
   those architectural roles with a reproducibly `bare-pack`-built worklet,
   `bare-rpc`, and a typed, versioned DubBridge protocol. Once parity passes,
   P1 deletes the old probe/bridge/protocol/inline worklet and obsolete wiring;
   it retains P0 audit records and only still-consumed native foundations.
4. The worklet treats uncaught exceptions, unhandled rejections, suspend, and
   resume as explicit protocol/lifecycle inputs. Runtime lifecycle and
   per-operation progress remain separate state machines.
5. Normal product mobile targets one Bare worklet. P1's two-session seed/client
   topology belongs only to `P1ReplicationProofRunner`, created through a
   runtime factory and invoked by an explicit development-only harness. Proof
   commands never enter the `P2PService` product API.
6. Fixture bytes are synthetic and may be generated in memory. Hyperdrive/
   Corestore metadata and blocks use a validated run-scoped directory under
   Expo `Paths.cache`; shutdown closes handles before deletion, verifies
   nonexistence, and includes a bounded startup janitor for abandoned proof runs.
7. A verified result requires complete client read, SHA-256 equality, successful
   bounded teardown, and verified removal of both run directories. Discovery,
   connect, replication, hash, lifecycle, and cleanup failures end in typed,
   redacted failure states; partial transfer or cleanup failure is never PASS.
8. P1 adds no user media, UI, HTTP server, backend/API/database behavior,
   availability node, encryption/envelopes, invitations, durable identity, or
   iOS work. It does not change ADR-032 or auto-start a P2P network.

## Module boundary

| Module | P1 responsibility | Explicitly excluded |
|---|---|---|
| `mobile/App.tsx` + `RootNavigator` | Compose providers above a navigation-only root | P2P mechanics inside navigation |
| `mobile/src/p2p/P2PProvider.tsx` | Stable service ownership and selective status subscription | Auto-start or proof commands in product context |
| `mobile/src/p2p/P2PService.ts` | Framework-independent product-facing runtime facade | React, routes, Hyperdrive proof topology |
| `mobile/src/p2p/runtime/*` | One product runtime, bundled worklet, versioned RPC, fatal/suspend/resume lifecycle | Product authorization or persistence policy |
| `mobile/src/p2p/proof/*` | Two-session seed/client proof runner and transient-storage lifecycle | Product service API or normal startup |
| P0 spike source/config | Characterization oracle until F3a/F3b retire it after parity | Permanent duplicate runtime/protocol/harness |
| `mobile/scripts/build-bare-worklet.mjs` | Deterministic bundle generation/drift verification | Runtime business logic |
| `mobile/__tests__/p2p/*` | Protocol, lifecycle, composition, storage, replication, and cleanup evidence | End-to-end product playback |

## Required decomposition

P1 now scores RRI 94 (Very high): the architectural decision, 29-file potential
surface, and combined refactor/behavior change trigger the ADR, risk-analysis,
and decomposition gates. Accepted ADR-043 contains the decision and risks. P1
may not be implemented as one change.

Under the approved revised parent, execute only this sequence; every child receives
its own current `scripts/rri.py` report, Compact Approval Task Card v2, and
explicit approval before source edits:

1. **P1.F1 — Reproducible worklet bundle and versioned RPC contract.** Add the
   deterministic `bare-pack` pipeline, `bare-rpc`, handshake/validators, typed
   failures, global fatal handlers, and suspend/resume protocol. No Hyperdrive,
   app composition, or network.
   Closed PASS after owner verification on 2026-08-27. P1.F2's dependency is
   satisfied; F2 still needs its current RRI/card/approval before source work.
2. **P1.F2 — Mobile service ownership and composition.** Add
   `BareRuntimeClient`, `P2PService`, `P2PProvider`, external-store snapshots,
   and composition-root ownership; make `RootNavigator` navigation-only and
   run the P0 oracle against the new seam. No P2P network activity or P0 source
   deletion yet.
   Closed PASS after owner verification on 2026-08-27. P1.F3a remains deferred
   until its own current RRI, approval card, and explicit approval.
3. **P1.F3a — P0 runtime-scaffold migration and retirement.** Move the bounded
   diagnostic ping and characterization cases to the new development harness,
   then delete `AndroidBareRuntimeProbe`, the custom bridge/protocol, inline
   worklet, and obsolete test.
4. **P1.F3b — P0 config/dependency cleanup.** Remove the old probe flag/script;
   keep each P0-added direct dependency and Android build setting only when the
   new runtime imports it or an Android A/B proof requires it. Preserve P0 audit
   documents.
5. **P1.A1 — Hyperdrive/Corestore Android bundle smoke proof.** Add compatible
   storage dependencies and prove an empty transient drive can open/close in the
   packaged Android worklet without discovery.
6. **P1.A2 — Transient seed lifecycle and residue cleanup.** Generate the
   synthetic fixture, write/hash it using a run-scoped cache directory, close
   handles, delete/verify absence, and clean abandoned proof directories.
7. **P1.B1 — Isolated Hyperswarm replication transport.** Use a proof-only
   runtime factory for seed/client sessions; discover and replicate the complete
   fixture without exposing proof commands through `P2PService`.
8. **P1.B2 — Verification, reconnect, and fail-closed witness.** Verify SHA-256,
   exercise one bounded reconnect, distinguish runtime/operation state, redact
   evidence, and require complete resource/storage teardown before PASS.

The current F1 score is 54; remaining child scores are planning estimates (F2 55, F3a
43, F3b 30, A1 40, A2 47, B1 55, B2 55). They demonstrate a plausible ≤55
split but are not
authorization; each remaining child is recalculated from its frozen paths immediately before
presentation. None may start P2 or alter the backend.

## P0 retirement contract

| Classification | P0 artifact | Treatment |
|---|---|---|
| Retain as history | P0 audit report, native proof, closure/handoff evidence | Keep: they are the factual feasibility record. |
| Retain with direct consumer | `react-native-bare-kit`; Android `minSdkVersion: 31` | Keep while ADR-043's Bare runtime requires them. |
| Provisional, evidence required | `b4a`, `react-native-b4a`, `@types/b4a`, `useLegacyPackaging`, and their Expo config support | F3b retains each only with an import/consumer or Android A/B proof; otherwise removes it and updates the lockfile. |
| Retire after parity | `AndroidBareRuntimeProbe`, custom bridge/protocol, inline worklet, probe wiring, and P0 bridge test | F3a first migrates every ping/error characterization to the ADR-043 seam, then deletes the duplicate scaffold. |
| Disposable local output | ignored `mobile/android` prebuild output | Reproducible and not tracked; it may be removed separately with owner approval, never as proof/history deletion. |

## Verification strategy

- Unit tests cover every approved HP/EC mapping of each child, including bundle
  drift, protocol mismatch, fatal lifecycle, provider ownership, transient path
  validation/deletion, replication, reconnect, and teardown.
- Android development-build proof records only protocol/runtime versions,
  lifecycle/operation states, byte count, fixture digest, timings, and cleanup
  outcome; it logs neither fixture content nor discovery keys.
- A successful P1.B proof demonstrates a seed/client transport and hash
  equivalence, not encryption, authorization, publication, persistence, or HLS
  playback.
- The normal mobile typecheck, lint, and full Jest suite must remain green.

## Status artifacts

- `docs/tasks/mvp0-p2p-p1-replication.md`
- this plan
- `docs/audit/mvp0-p2p-p1-rri.md`
- `docs/audit/mvp0-p2p-p1-approval-card.md`
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`,
  `docs/adr/README.md`, and `docs/architecture.md`
- `docs/plan/mvp0-p2p-first.md`, `docs/tasks/mvp0-p2p-first.md`, and
  `docs/plan/roadmap.md` when P1 changes state
- `p2p-mvp/RUN_STATE.json` and `p2p-mvp/handoffs/P1.md` only at P1 closure

## Stop conditions

- ADR-043 ceases to be Accepted or an approved child would conflict with it.
- The selected `bare-pack`/`bare-rpc`/Hyperdrive/Hyperswarm stack cannot run in
  the proven Android Bare boundary without a custom native service or another
  unapproved runtime architecture.
- The proof requires storage outside its validated cache subtree, persistent
  identity/product storage, backend/API/database changes, plaintext user media,
  an HTTP server, an encryption/key design, or iOS work.
- Discovery/reconnect cannot be bounded and diagnosed without treating an
  unverified fixture as success.
