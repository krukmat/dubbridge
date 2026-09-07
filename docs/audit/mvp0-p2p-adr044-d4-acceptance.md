---
type: Audit
title: "ADR044-D4 acceptance audit"
status: complete
slice: MVP0-P2P
parent: ADR044-D4
---

# ADR-044 D4 acceptance audit

Date: 2026-09-05  
Branch: `feature/p2p-mvp-core`

## Purpose

Record the final consolidated review and owner disposition for ADR-044 after the
three decision gates completed:

- D1: `O3 parallel` audience authorization;
- D2: `K1` content-key/device-envelope contract;
- D3: `O4` transactional-outbox publication/recovery contract.

## Owner authorization

The owner instructed: `trabaja con todo lo necesario para desbloquear P2`.
Because the only remaining ADR prerequisite was D4 and D4 introduced no new
substantive architecture option, this instruction authorized the bounded final
review and acceptance if no closure blocker remained. It does not authorize P2
source implementation.

## Review matrix

| Area | Result | Basis |
|---|---|---|
| Control-plane authority | PASS | O3 keeps authorization backend-owned; ADR-032 remains separate |
| Key custody/confidentiality | PASS | K1: AES-256-GCM package encryption, server-wrapped CK, HPKE/P-256 device envelope, no silent software-key downgrade |
| Device/runtime boundary | PASS | private key stays in Android Keystore; Bare may hold only transient authorized CK; Availability Node ciphertext-only |
| Publication durability | PASS | O4: PostgreSQL + transactional outbox authoritative, queue optional, reconciler recovery |
| Failure semantics | PASS | unknown outcome remains non-ready; retries preserve logical package/K1 lineage |
| Existing pipeline isolation | PASS | S-120 `PreparationStatus::Ready` and ASR remain independent of P2P publication |
| ADR-032 compatibility | PASS | review-time HTTP HLS contract unchanged |
| Scope completeness | PASS | open questions 4-7 are phase-specific closure gates, not contradictions in the audience-delivery boundary |

## Reflection

### 1. Security / authority

No path allows Hyperdrive key possession, ciphertext possession, queue status, or
Availability Node state to replace backend authorization. K1 key release consumes
both current O3 authorization and current O4 readiness fail-closed.

### 2. Crash / revocation / idempotency

K1 does not overclaim retroactive key erasure. O4 does not overclaim distributed
exactly-once. The combined contract is implementable with bounded failure semantics:
revocation stops future releases; duplicate/unknown publication converges on the
same logical lineage before readiness can be asserted.

### 3. Scope and downstream decisions

The remaining open items are intentionally deferred with explicit owners:
Availability Node operational trust in P2, audit inventory in P2/P3 closure,
device/cache lifecycle in P4, and no-HTTP-fallback certification profile in P7.
They do not require reopening D1-D3.

## Decision

**ADR-044 accepted.**

This removes the architecture prerequisite that made P2 unpresentable. P2 is now
eligible to receive its own plan, RRI, decomposition, approval card, and owner HITL.
No P2 source work is authorized by this audit or by ADR acceptance alone.

## Environment / review evidence

- Local Ollama/models/device/emulator: `n/a`; no local evidence simulated.
- Phase-1/phase-2 peer review: `n/a` for this docs/ADR-only task.
- RRI: conservative parent **48 Med-high / Effort L**, including `arch_decision +12`.
- Three integrated D4 reflection passes: PASS.
