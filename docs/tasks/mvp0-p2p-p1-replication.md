---
type: TaskList
title: "Tasks: MVP0-P2P P1 isolated replication proof"
status: proposed
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p1-replication.md
---

# Tasks: MVP0-P2P P1 — Isolated P2P core replication proof

> **Parent plan:** `docs/plan/mvp0-p2p-p1-replication.md`.
> **External input:** `p2p-mvp/taskpacks/P1.zip`.
> **Dependency gate:** P0 closed PASS — satisfied on 2026-08-27.
> **Status:** Awaiting approval of the P1 parent plan. P1.A/P1.B are not
> presented or executable yet.

## Task map

| ID | Title | Status | Depends on |
|---|---|---|---|
| P1 | Isolated P2P core replication proof (planning parent) | Awaiting approval | P0 PASS |
| P1.A | Ephemeral seed fixture and bundle boundary | Deferred — needs RRI/card/approval | P1 approval |
| P1.B | Client discovery, replication, and verification witness | Deferred — needs RRI/card/approval | P1.A PASS |

## P1 — Isolated P2P core replication proof

- **Status:** Awaiting explicit approval. No implementation started.
- **Type:** Complex development-spike parent; decomposed before implementation.
- **Complexity / Effort / RRI:** Complex / L / 57. Full report:
  `docs/audit/mvp0-p2p-p1-rri.md`.
- **External taskpack declaration:** `gpt-5.6-terra` / high. This is retained as
  external input, but no direct implementation route is authorized because the
  repository RRI is Complex and requires decomposition.
- **Allowed future paths:** `mobile/package.json`, `mobile/package-lock.json`,
  `mobile/src/p2p/replication-worklet.ts`, `mobile/src/p2p/replication-bridge.ts`,
  `mobile/src/p2p/AndroidBareRuntimeProbe.tsx`,
  `mobile/__tests__/p2p/replication-bridge.test.ts`, and P1 evidence documents.
- **Objective:** Seed and replicate an in-memory synthetic opaque fixture between
  isolated Bare runtimes using Hyperdrive/Hyperswarm, then verify SHA-256 equality.
- **Out of scope:** All backend/API/database work; real DubBridge assets, HLS,
  `StorageAdapter`, product UI, durable cache/identity, encryption/key/envelope
  design, invitations, availability node, local HTTP, HTTP fallback, and iOS.

### Happy paths considered

- **HP-1:** An ephemeral seed writes a synthetic fixture; an isolated client
  discovers it, replicates it, and yields the expected SHA-256 digest.
- **HP-2:** After one bounded disconnect/rejoin, the client re-establishes the
  proof and only reports verification after a complete digest match.

### Edge cases considered

- **EC-1:** Discovery, connection, worklet, or replication failure yields a
  typed failed result and releases seed/client resources; no partial transfer is
  presented as verified.
- **EC-2:** A timeout, mismatched digest, malformed reply, or reconnect-budget
  exhaustion fails closed with redacted state evidence and no persistent residue.

### Acceptance criteria

- The Android development build proves Hyperdrive → Hyperswarm → Hyperdrive
  replication of a synthetic fixture and its SHA-256 equality.
- One bounded reconnect is observed or the exact limitation is recorded as STOP;
  failed discovery/reconnect is typed, redacted, and cleaned up.
- P0's generic Bare compatibility layer and ordinary mobile behavior remain
  intact; no UI or product P2P capability is introduced.
- Every child HP/EC has passing unit-test evidence and a physical Android proof
  before P1 can close.

### Evidence to emit

- P1 and child RRI reports, approval cards, and any required route artifacts.
- Dependency/bundling compatibility evidence and an Android native proof log.
- Unit-test, typecheck, lint, and full mobile Jest outputs; P1 handoff at close.

### Status artifacts affected

- This ledger, its plan, `docs/tasks/mvp0-p2p-first.md`,
  `docs/plan/mvp0-p2p-first.md`, `docs/plan/roadmap.md`,
  `p2p-mvp/RUN_STATE.json`, and `p2p-mvp/handoffs/P1.md`.

### Task-analysis review

Task-analysis review: REVIEW-OVERRIDE — explicit owner-directed MVP0-P2P
exception; `docs/audit/mvp0-p2p-review-exception.md`.

### Code-solution review

Code-solution review: REVIEW-OVERRIDE — to be recorded at P1 closure under the
explicit owner-directed MVP0-P2P exception;
`docs/audit/mvp0-p2p-review-exception.md`.

### Required execution sequence

1. After parent approval, score and present P1.A; do not edit source before its
   approval.
2. After P1.A PASS, score and present P1.B; do not edit source before its
   approval.
3. Close P1 only after both children, four parent Reflection passes, coverage
   certification, owner final verification, and status synchronization pass.

### Parent reflection plan

| Pass | Focus | Required result |
|---|---|---|
| 1 | Runtime containment | No P0 lifecycle regression or leaked worklet/handle. |
| 2 | Replication correctness | No verified state before complete hash equality. |
| 3 | Network failure behavior | Discovery/reconnect remains bounded, typed, and redacted. |
| 4 | Product-boundary protection | No production asset, key, UI, server, or iOS scope leaks in. |

### Unit coverage certification

**Pending child implementation.** P1 cannot close until P1.A/P1.B map every
HP/EC above to passing unit tests.

### Owner final verification

**Pending P1.A/P1.B completion.**

### Handoff prompt

`P1 — use this ledger and plan; work only on the approved child scope; stop
after its evidence and do not begin P2.`
