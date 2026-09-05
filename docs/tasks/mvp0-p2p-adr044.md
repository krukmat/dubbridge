---
type: TaskList
title: "Tasks: MVP0-P2P ADR-044 decision closure"
status: done
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-first.md
---

# Tasks: MVP0-P2P ADR-044 decision closure

## Purpose

Resolve and accept the audience-delivery architecture required before P2 can be
planned/presented. D1-D3 freeze the substantive architecture; D4 performs the
consolidated review and explicit owner disposition.

No task in this ledger authorizes P2 source work. ADR acceptance removes only the
architecture prerequisite; P2 still requires its own plan, RRI, task decomposition,
Compact Approval Task Card, and explicit HITL approval.

## Task map

| Task | Objective | Status | Depends on |
|---|---|---|---|
| `ADR044-D1` | Resolve ADR-032 grant composition | **Complete 2026-09-05 — owner selected `O3 parallel`** | P1 PASS |
| `ADR044-D2` | Resolve content-key / device-envelope contract | **Complete 2026-09-05 — owner selected `K1`** | D1 PASS |
| `ADR044-D3` | Resolve publication/outbox state and recovery semantics | **Complete 2026-09-05 — owner selected `O4`** | D2 PASS |
| `ADR044-D4` | Review consolidated ADR-044 and explicitly accept/reject it; propagate status | **Complete 2026-09-05 — ACCEPT** | D1-D3 PASS |

## D1 — grant composition

- **Outcome:** `O3 parallel`.
- **Meaning:** after a valid invitation claim, a distinct backend-owned audience
  authorization gates key release. The invitation claim alone, ADR-032
  `PlaybackGrant`, Hyperdrive key, or ciphertext possession is insufficient.
- **ADR-032:** unchanged and authoritative for review-time HTTP HLS.
- **Parent RRI:** 55 Med-high / Effort L.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d1-grant-composition.md`.

## D2 — content key / device envelope

- **Outcome:** `K1`.
- **Content:** one fresh 256-bit CK per package; AES-256-GCM media encryption.
- **Server custody:** CK persisted only server-wrapped under versioned
  AES-256-GCM KEK; plaintext CK never persisted/logged.
- **Device envelope:** HPKE Base using `DHKEM(P-256, HKDF-SHA256)` /
  `HKDF-SHA256` / `AES-256-GCM`.
- **Device key:** non-exportable P-256 ECDH key in Android Keystore. StrongBox
  is optional, no external hardware is required, and missing required
  Keystore/ECDH capability fails closed. No silent K2/software-key fallback.
- **Binding:** invitation + viewer + active device + asset/package + O3 audience
  authorization + expiry/revocation.
- **Bare:** may receive only a transient authorized playback CK, never device
  private key/backend secrets.
- **Parent RRI:** 70 Complex / Effort L.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md`.

## D3 — publication / outbox / recovery

- **Outcome:** `O4 — transactional outbox + optional queue accelerator +
  PostgreSQL reconciler`.
- **Parent RRI:** 60 Complex / Effort L.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d3-publication.md`.

Frozen semantics:

1. `PreparationStatus::Ready` remains S-120 HLS readiness and never waits for P2P.
2. P2P has a separate durable `P2P_READY` predicate.
3. PostgreSQL + transactional outbox are the durable authority.
4. Queue delivery is optional/replayable acceleration, never product truth.
5. PostgreSQL reconciliation is the recovery safety net.
6. Delivery is at-least-once and idempotent under one stable logical publication
   identity and K1 lineage.
7. Unknown external outcome remains non-ready.
8. Availability Node is ciphertext-only and never owns business/auth/key/database
   authority.

## D4 — consolidated acceptance

- **Outcome:** **ACCEPT** on 2026-09-05.
- **Owner instruction:** `trabaja con todo lo necesario para desbloquear P2`.
- **Interpretation:** explicit authorization to execute the bounded D4 review and
  accept ADR-044 if the review found no blocker; no P2 source authorization.
- **RRI:** conservative 48 Med-high / Effort L, retaining `arch_decision +12`.
- **Review:** three integrated passes all PASS: authority/security, lifecycle/
  recovery, and scope/downstream-gate consistency.
- **Task:** `docs/tasks/mvp0-p2p-adr044-d4.md`.
- **Audit:** `docs/audit/mvp0-p2p-adr044-d4-acceptance.md`.

Open decision items 4-7 remain phase-specific gates rather than ADR blockers:
Availability Node operational trust (P2 deployment completion), P2P audit-event
inventory (P2/P3 closure), persistent device/cache lifecycle (P4), and the no-HTTP-
fallback certification profile (P7).

## Current result

```text
D1  O3 parallel    ✅ complete
D2  K1             ✅ complete
D3  O4             ✅ complete
D4  ACCEPT         ✅ complete

ADR-044 = Accepted
P2      = architecture-unblocked; requires own plan/RRI/HITL before source work
P3      = blocked on P2 PASS
```
