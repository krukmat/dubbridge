---
type: Plan
title: "Plan: MVP0-P2P P1 isolated replication proof"
status: proposed
slice: MVP0-P2P
parent_task: P1
---

# Plan: MVP0-P2P P1 — Isolated P2P core replication proof

> **Parent task ledger:** `docs/tasks/mvp0-p2p-p1-replication.md`.
> **Dependency:** P0 closed PASS (Android-only) on 2026-08-27.
> **Status:** Planned only. No P1 code or P2P network activity has started.

## Objective

Prove on Android that two isolated, ephemeral Bare runtimes can seed and
replicate a synthetic opaque fixture through Hyperdrive and Hyperswarm, then
verify its SHA-256 digest. The proof establishes transport feasibility only; it
does not publish a DubBridge asset or create a product data plane.

## Boundaries and decisions

1. Preserve P0's generic `BareBridge`, `BARE_WORKLET_*`, Android native
   configuration, and opt-in `AndroidBareRuntimeProbe`. P1 extends beside that
   compatibility layer; it does not rename or reimplement it.
2. The seed fixture is generated for the proof, is opaque synthetic bytes, and
   lives only in memory for the run. It is not an asset, HLS package, ciphertext
   implementation, content key, invitation, or durable device identity.
3. Hyperdrive and Hyperswarm are constrained to two isolated P1 runtimes. Their
   discovery key and transient connection state are not persisted or logged.
4. A verified result requires client read completion **and** a SHA-256 equality
   check. Discovery, connect, replication, hash, and teardown failures must end
   in a typed failed state; P1 may never report the fixture as verified on a
   partial transfer or timeout.
5. P1 may use the existing Android-only probe to emit redacted state/timing
   evidence. It must not add UI, an HTTP server, backend/API/database calls,
   storage metadata, availability-node behavior, encryption/envelopes, invites,
   or `VideoPlayer` integration. iPhone/iOS remains out of scope.
6. P1 does not change ADR-032. Its existing HTTP HLS delivery stays untouched;
   any later audience-delivery architecture decision belongs before P2.

## Module boundary

| Module | P1 responsibility | Explicitly excluded |
|---|---|---|
| `mobile/src/p2p/bare-bridge.ts` | Reused P0 lifecycle/RPC boundary | Product replication policy |
| `mobile/src/p2p/replication-bridge.ts` | Typed P1 proof command, state and redacted evidence | Asset/invite/key APIs |
| `mobile/src/p2p/replication-worklet.ts` | Ephemeral seed/client Hyperdrive + Hyperswarm proof | Persistent node/cache or production package format |
| `mobile/src/p2p/AndroidBareRuntimeProbe.tsx` | Opt-in Android physical-proof trigger | Visible UI or default runtime behavior |
| `mobile/package*.json` | P1-specific compatible runtime/bundling dependencies | Backend/mobile product dependencies |
| `mobile/__tests__/p2p/replication-bridge.test.ts` | Typed success/failure and cleanup tests | End-to-end product playback tests |

## Required decomposition

P1 scores RRI 57 (Complex), so it is a planning parent and may not be
implemented as one change. After approval, the following children must each
receive their own RRI, task card, and explicit approval before source edits:

1. **P1.A — Ephemeral seed fixture and bundle boundary.** Select compatible P1
   dependencies; package an in-memory synthetic fixture behind the existing Bare
   lifecycle; prove seed creation and deterministic teardown without discovery.
2. **P1.B — Client discovery, replication, and verification witness.** Join the
   transient discovery key, replicate the fixture, hash-check it, exercise one
   bounded reconnect path, and make discovery/reconnect failure fail closed.

P1.A must PASS before P1.B is presented. Neither child may start P2 or alter
the backend.

## Verification strategy

- Unit tests cover every approved HP/EC mapping of each child.
- Android development-build proof records only lifecycle states, byte count, and
  a fixture digest; it logs neither fixture content nor keys.
- A successful P1.B proof demonstrates a seed/client transport and hash
  equivalence, not encryption, authorization, publication, persistence, or HLS
  playback.
- The normal mobile typecheck, lint, and full Jest suite must remain green.

## Status artifacts

- `docs/tasks/mvp0-p2p-p1-replication.md`
- this plan
- `docs/audit/mvp0-p2p-p1-rri.md`
- `docs/audit/mvp0-p2p-p1-approval-card.md`
- `docs/plan/mvp0-p2p-first.md`, `docs/tasks/mvp0-p2p-first.md`, and
  `docs/plan/roadmap.md` when P1 changes state
- `p2p-mvp/RUN_STATE.json` and `p2p-mvp/handoffs/P1.md` only at P1 closure

## Stop conditions

- The selected Hyperdrive/Hyperswarm runtime cannot be bundled and run in the
  proven Android Bare boundary without changing product or native architecture.
- The proof requires persistent identity/storage, backend/API/database changes,
  plaintext user media, an HTTP server, an encryption/key design, or iOS work.
- Discovery/reconnect cannot be bounded and diagnosed without treating an
  unverified fixture as success.
