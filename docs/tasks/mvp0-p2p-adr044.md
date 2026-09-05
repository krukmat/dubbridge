---
type: TaskList
title: "Tasks: MVP0-P2P ADR-044 decision closure"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-first.md
---

# Tasks: MVP0-P2P ADR-044 decision closure

## Purpose

Resolve the architectural questions that keep ADR-044 `Proposed` and block P2.
The decision sequence separates evidence extraction, owner selection, mechanical
codification, and canonical synchronization. Detailed option evidence lives in
the per-decision audits; this parent ledger records the authoritative closure
state and next gate.

Governing sources:

- `docs/plan/mvp0-p2p-first.md`
- `docs/plan/mvp0-p2p-design-inputs.md`
- `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`

No task in this ledger authorizes P2 source work. P2 requires an accepted
ADR-044 plus its own plan, full task definition, RRI, Compact Approval Task Card,
and explicit HITL approval.

## Task map

| Task | Objective | Status | Depends on |
|---|---|---|---|
| `ADR044-D1` | Resolve ADR-032 grant composition | **Complete 2026-09-05 — owner selected `O3 parallel`** | P1 PASS |
| `ADR044-D2` | Resolve content-key / device-envelope contract | **Complete 2026-09-05 — owner selected `K1`** | D1 PASS |
| `ADR044-D3` | Resolve publication/outbox state and recovery semantics | **Complete 2026-09-05 — owner selected `O4`** | D2 PASS |
| `ADR044-D4` | Review consolidated ADR-044 and explicitly accept/reject it; propagate status | **Next gate — not yet presented/approved** | D1-D3 PASS |

## D1 — grant composition

- **Outcome:** `O3 parallel`.
- **Meaning:** after a valid invitation claim, a distinct backend-owned audience
  authorization gates key release. The invitation claim alone, ADR-032
  `PlaybackGrant`, Hyperdrive key, or ciphertext possession is insufficient.
- **ADR-032:** unchanged and authoritative for review-time HTTP HLS.
- **Parent RRI:** 55 → Med-high → Effort L.
- **Leaves:** S1-S3 RRI 24 Low S; S4 RRI 23 Low S; status sync completed.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d1-grant-composition.md`.

D1 did not accept ADR-044 or authorize P2/P3.

## D2 — content key / device envelope

- **Outcome:** `K1`.
- **Content:** one fresh 256-bit CK per package; AES-256-GCM media encryption.
- **Server custody:** CK persisted only server-wrapped under versioned
  AES-256-GCM KEK; plaintext CK never persisted/logged.
- **Device envelope:** HPKE Base using
  `DHKEM(P-256, HKDF-SHA256)` / `HKDF-SHA256` / `AES-256-GCM`.
- **Device key:** non-exportable P-256 ECDH key in Android Keystore. StrongBox
  is optional, no external hardware is required, and missing required
  Keystore/ECDH capability fails closed. No silent K2/software-key fallback.
- **Binding:** invitation + viewer + active device + asset/package + O3 audience
  authorization + expiry/revocation.
- **Bare:** may receive only a transient authorized playback CK, never device
  private key/backend secrets.
- **Parent RRI:** 70 → Complex → Effort L.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md`.
- **Ledger:** `docs/tasks/mvp0-p2p-adr044-d2.md`.

D2 did not accept ADR-044 or authorize P2/P3.

## D3 — publication / outbox / recovery

- **Status:** `[x] Complete 2026-09-05`.
- **Owner checkpoint:** owner explicitly selected `O4` with
  `apruebo o4. documentalo` after reviewing O1/O2/O3 and the long-term hybrid.
- **Selected contract:** **O4 — transactional outbox + optional queue
  accelerator + PostgreSQL reconciler**.
- **Parent RRI:** **60 → Complex → Effort L**.
- **Leaves:** S1-S4 RRI 14 Low S; S5 RRI 13 Low S; SYNC RRI 17 Low S.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d3-publication.md`.
- **Ledger:** `docs/tasks/mvp0-p2p-adr044-d3.md`.

### D3 frozen semantics

1. `PreparationStatus::Ready` remains S-120 HLS readiness and does not wait for
   P2P publication or delay ASR/transcription enqueue.
2. P2P has a separate durable readiness predicate (`P2P_READY` semantically).
3. PostgreSQL is authoritative for stable logical publication identity,
   publication state, durable outbox intent, and the Ready transition.
4. A transactional outbox records the publication obligation atomically with
   local publication state before any external side effect is required.
5. Delivery is at-least-once and idempotent; D3 makes no distributed
   exactly-once claim.
6. An existing/future queue may accelerate normal dispatch/backpressure, but it
   is replaceable and non-authoritative. Queue enqueue/ACK is never publication
   success evidence.
7. A PostgreSQL-driven reconciler is the recovery safety net for lost dispatch,
   stale/incomplete work, unknown remote result, and remote-success/ACK-loss.
8. Outbox retry, queue duplicates, reconciler re-drive, and Availability Node
   confirmation all use one stable logical package/publication identity and K1
   lineage. Retry cannot silently create a new package/CK lineage.
9. Availability Node remains ciphertext-only and cannot own PostgreSQL/business
   state, authorization, KEK, plaintext CK, invite/viewer data, or signing keys.
10. D2/K1 envelope release consumes the authoritative D3 Ready predicate
    fail-closed.

### O4 authority hierarchy

```text
PostgreSQL publication state + transactional outbox
        = durable authority

queue
        = optional delivery accelerator

PostgreSQL reconciler
        = recovery safety net

Availability Node
        = ciphertext publication executor/evidence source
```

No queue or Availability Node status independently establishes `P2P_READY`.

### Crash-window requirements

| Window | Required O4 result |
|---|---|
| before durable transaction | no publication obligation |
| committed, before dispatch | outbox/reconciler recovers same work |
| during external operation | non-ready/unknown; same identity reconciled/retried |
| remote success, ACK lost | no second logical package; same identity converges on existing result |
| ACK received, local Ready commit lost | remains non-ready until reconciliation reconfirms then commits Ready |
| duplicate after Ready | idempotent no-op; no lineage rotation/state regression |

### Fail-closed readiness

`P2P_READY` is true only when authoritative PostgreSQL state durably proves the
current lineage is valid, K1 package construction completed, external
publication of that same lineage is confirmed, persisted publication evidence
matches that lineage, and no unresolved unknown-outcome condition remains.

Anything missing, stale, conflicting, or unknown means **not P2P-ready**.

### D3 integrated Reflection

1. **Transactional/crash-window correctness — PASS.** Local durable intent is
   atomic in PostgreSQL; external publication remains explicitly non-atomic and
   recoverable.
2. **Idempotency/unknown outcome — PASS.** One stable identity spans outbox,
   queue, reconciler, and Availability Node; unknown outcome never yields Ready.
3. **S-120 independence / D2 interaction — PASS.** Existing S-120 Ready/ASR
   continue independently; D3 readiness gates K1 release.
4. **Scope/status — PASS.** D3 does not select SQL names, routes, queue product,
   retry constants, Availability Node credentials/deployment, certification
   profile, or complete audit-event inventory. ADR-044 remains `Proposed`.

## D4 — explicit ADR acceptance gate

D1-D3 are now complete. `ADR044-D4` is the next gate and must be independently
scored/presented under current workflow policy.

D4 may only:

1. review the consolidated ADR-044 proposal and D1-D3 evidence;
2. resolve any closure-blocking inconsistency discovered by that review;
3. obtain explicit owner acceptance/rejection;
4. if accepted, change ADR-044 status and synchronize canonical status docs.

D4 is **not** P2 approval. P2 planning/RRI/HITL starts only after ADR acceptance.

## Phase-specific open decisions after D3

ADR-044 questions 4–7 remain later phase gates rather than D1-D3 blockers:

- Availability Node deployment/authentication/observability/operational ownership
  — blocks P2 deployment completion;
- certification profile — blocks P7;
- complete ADR-018 P2P audit-event inventory — blocks P2/P3 closure evidence;
- persistent cache/device/sign-out/background lifecycle — blocks P4 product
  lifecycle closure.

## Verification / environment

- Local Ollama/models/devices/emulators: `n/a` for D1-D3 docs/ADR decision work;
  no local evidence simulated.
- Phase-1/phase-2 peer review: docs/ADR/task-ledger exemption where recorded in
  each child ledger/audit.
- D3 final repository verification is recorded against the synchronized branch
  head by CI and AGENTS parity checks.

## Current result

```text
D1  O3 parallel    ✅ complete
D2  K1             ✅ complete
D3  O4             ✅ complete
D4  ADR acceptance ⏭ next explicit gate

ADR-044 = Proposed
P2      = NOT AUTHORIZED
P3      = NOT AUTHORIZED
```
