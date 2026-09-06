---
type: TaskList
title: "Tasks: MVP0-P2P P2 encrypted publication"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p2-encrypted-publication.md
behavioral_coverage_contract: behavior-v2
---

# P2 — encrypted P2P publication

## Parent status

- ADR-044: **Accepted 2026-09-05** — dependency satisfied.
- P1: **Done** — dependency satisfied.
- Parent P2 RRI: **131 Excessive / Effort XL** — no direct implementation.
- Parent disposition: mandatory re-scope into T0-T6 below.
- `P2.T0`: **PASS 2026-09-05** — owner selected `AN-R1 + AN-A1` and accepted the proposed publication-state and minimum audit contract.
- Original `P2.T1` RRI 78 parent: **SUPERSEDED / NON-EXECUTABLE** by lower-RRI decomposition on 2026-09-05.
- `P2.T1a`-`P2.T1f`: **Done / owner-approved as the complete P2.T1 persistence outcome on 2026-09-06**.
- Next gate: `P2.C0` shared contract, golden-fixture, and path-ownership freeze. No T2-T6 source is authorized by the T1 approval.
- Review exception: existing owner-directed MVP0-P2P P0-P7 phase-1/phase-2 review override remains in force; it does not waive RRI/HITL/Reflection/tests.

## Task map

The child scores below are conservative planning scores used to choose the gate. Every executable child must be re-scored against its frozen exact paths immediately before source execution; no score may be silently lowered to obtain a cheaper route.

| ID | Objective | Planning RRI | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| P2.T0 | Freeze Availability Node trust/operation + concrete O4 implementation contract | 64 Complex | L | **PASS** | ADR-044 Accepted |
| P2.T1 | Durable publication/outbox persistence parent | 78 High | XL | **SUPERSEDED — container only** | T0 PASS |
| P2.T1a | Pure domain publication identity/state contract + unit tests | **22 Low** | S | **Done / owner-approved 2026-09-06** | T0 PASS |
| P2.T1b | PostgreSQL publication/outbox schema + constraints only | **32 Medium** | S/M | **Done / owner-approved 2026-09-06** | T1a |
| P2.T1c | Atomic create/ensure publication + outbox repository write | **47 Medium-high** | M | **Done / owner-approved 2026-09-06** | T1b |
| P2.T1d | Outstanding-work/read model repository queries | **36 Medium** | S/M | **Done / owner-approved 2026-09-06** | T1c |
| P2.T1e | Guarded persistence transitions + same-lineage confirmation evidence | **44 Medium-high** | M | **Done / owner-approved 2026-09-06** | T1d |
| P2.T1f | Persistence integration certification + negative/restart evidence | **33 Medium** | M | **Done / owner-approved 2026-09-06** | T1e |
| P2.C0 | Shared package/crypto/AN/audit contract, golden fixtures, and path ownership | TBD exact-path | S/M | **NEXT GATE** | T1 Done |
| P2.T2 | K1 encrypted package construction + server-wrapped CK custody | 82 High | XL | Pending decomposition | C0 PASS |
| P2.T3 | Availability Node ciphertext publication executor | 72 High | XL | Pending decomposition | C0 PASS; T2 contract |
| P2.T4 | O4 outbox dispatch, optional queue acceleration + reconciler | 84 High | XL | Pending decomposition | C0 PASS; T1 Done; T3 contract |
| P2.T5 | S-120 downstream integration + fail-closed P2P_READY transition | 74 High | XL | Pending decomposition | C0 PASS; T2 + T4 integration |
| P2.T6 | ADR-018 audit inventory, crash-window integration certification + P2 closure | 76 High | XL | Pending decomposition | C0 PASS; T1 + T2-T5 |

The old T1/T1a planning cards are retained as historical pre-implementation records. The owner has accepted the delivered T1a-T1f outcome; they are not reopened for retrospective review. The next presentation artifact belongs to P2.C0.

Every High/Complex implementation parent is decomposed again before source edits. Low-band maximization applies only at real pure/mechanical seams; crypto/key-custody and distributed recovery are not artificially downgraded.

## P2.T0 — Availability Node / O4 implementation contract — PASS

Owner approval on 2026-09-05 froze:

1. **Runtime:** `AN-R1` — dedicated Node.js/TypeScript service at the P2 Availability Node boundary, using the Hyperdrive/Hyperswarm JS ecosystem and remaining operationally independent from mobile Bare.
2. **Publication-control authentication:** `AN-A1` — mTLS service identity on a private/non-public control endpoint. The Availability Node does not reuse the backend HS256 JWT signing secret.
3. **Authority:** PostgreSQL publication state + transactional outbox remain authoritative. Queue use is optional acceleration only; Availability Node is an executor/evidence source only.
4. **Idempotency:** same logical publication identity + same ciphertext/manifest identity returns stable publication evidence; conflicting package/hash under the same identity fails closed.
5. **State semantics:** `building -> publish_pending -> publishing -> reconciling -> ready`; `failed` is terminal/non-retryable only. Unknown remote outcome is `reconciling`, never `ready` merely because dispatch occurred or an ACK was lost.
6. **Minimum ADR-018 audit inventory:** publication intent created; K1 lineage sealed/server-wrapped; external publication confirmed; reconciliation entered for unknown outcome; `P2P_READY` transition; terminal publication failure after bounded policy exhaustion.
7. **Secret deny-list:** no PostgreSQL credentials, plaintext CK, server KEK, invite/viewer/business-authorization state, application JWT signing material, or raw service credentials cross into publication payloads/logs.

Evidence:

- `docs/audit/mvp0-p2p-p2-t0-approval-card.md`
- `docs/audit/mvp0-p2p-p2-t0-selection.md`

Four integrated T0 Reflection passes are recorded PASS. T0 was docs/architecture only; it did not authorize or perform P2 source work.

## P2.T1 — durable publication/outbox persistence — DONE OUTCOME / SUPERSEDED CONTAINER

The former RRI-78 executable parent is now only a grouping label for T1a-T1f. All six leaves are Done and owner-approved as one completed persistence outcome on 2026-09-06.

### P2.T1a — pure domain identity/state contract — DONE

Scope:

- stable logical publication identity type;
- stable K1 lineage reference type;
- T0-compatible semantic publication states;
- pure transition/readiness guards;
- focused unit tests for invalid direct `ready`, terminal/regressive transitions, lineage stability, and unknown/reconciling semantics.

Expected source envelope: new `crates/domain/src/p2p_publication.rs`, `crates/domain/src/lib.rs` export, colocated unit tests, and T1a documentation only.

Explicitly excludes PostgreSQL, migrations, repositories, outbox persistence, crypto, workers, queue/reconciler, Availability Node, and S-120 integration.

### P2.T1b — PostgreSQL schema + constraints — DONE

Scope only the next migration:

- publication persistence structure;
- outbox persistence structure;
- stable logical identity + lineage columns;
- non-secret confirmation metadata needed by T1e;
- safe uniqueness/check/FK constraints.

No Rust repository implementation or worker behavior.

### P2.T1c — atomic create/ensure publication + outbox write — DONE

Scope only the minimal DB write repository behavior:

- create or idempotently ensure a logical publication;
- create the initial outbox obligation in the same PostgreSQL transaction;
- return the stable logical identity;
- prevent implicit second active lineage.

No scans, state-transition API, dispatch, queue, or Availability Node call.

### P2.T1d — outstanding-work/read queries — DONE

Scope only read-side repository behavior:

- fetch publication by stable identity;
- observe outstanding durable publication obligations;
- expose non-secret PostgreSQL truth required by later T4 recovery.

No claim/lease or state mutation.

### P2.T1e — guarded persistence transitions — DONE

Scope only durable state mutation:

- apply T0 lifecycle transitions;
- persist same-lineage external-confirmation evidence fields;
- guard `ready` so it cannot be written without required durable confirmation;
- retain unknown outcome as non-ready/reconciling.

No network/external side effect.

### P2.T1f — persistence certification — DONE

Scope only integration/negative evidence:

- atomic publication + outbox creation;
- restart/re-read of same identity and outstanding intent;
- duplicate create/idempotency;
- invalid direct-ready rejection;
- schema/serialization secret-deny-list inspection;
- behavior-v2 mapping for T1 acceptance.

If certification finds a production defect, reopen the responsible leaf rather than turning T1f into an unbounded corrective implementation task.

Canonical decomposition evidence: `docs/audit/mvp0-p2p-p2-t1-decomposition.md`.

## P2.C0 — shared contract and concurrency freeze — NEXT GATE

**Type:** planning/contract only

**RRI:** TBD against exact paths before presentation

**Depends on:** P2.T1 Done
**Status:** [ ] Next gate

Freeze manifest-v1 canonicalization and golden fixtures; AES-GCM/AAD/nonce rules; generate-once CK and versioned KEK handling; ciphertext handoff and AN idempotency/evidence; audit transaction/correlation mapping; the P3 descriptor; and one-writer exact-path ownership for every downstream leaf.

**Acceptance:** each T2-T6 parent has a decomposition with independently meaningful leaves, explicit join tasks, no shared-path collision, and an exact integration order. Queue acceleration remains optional. `scripts/rri.py` is run only when each executable leaf's exact paths are known.

**Concurrency boundary:** analysis, planning, fixtures, and test design can run concurrently. The current workflow still permits one executable task ID at a time. ADR-040 multi-author source work is limited to eligible approved RRI 26-55 tasks with disjoint paths, one orchestrator, one base SHA, frozen interfaces, and whole-task integration.

## P2.T2 — K1 encrypted package builder

### Scope

- Read existing S-120 prepared package through approved storage seams.
- Generate new lineage CK once.
- AES-256-GCM encrypt each package file with unique nonce.
- Versioned canonical AAD/manifest and ciphertext hashes.
- Persist only server-wrapped CK + metadata.
- Reuse frozen lineage on dispatch/retry; package replacement is a new explicit lineage, never an implicit retry side effect.

### HP / EC

- **HP-T2-1:** valid HLS package becomes a complete ciphertext-only K1 package with reproducible manifest/hash evidence.
- **EC-T2-1:** nonce collision/reuse, missing file, manifest mismatch, wrap failure, or storage failure leaves publication non-ready and no plaintext CK persisted.
- **EC-T2-2:** logs/errors/redaction never reveal plaintext CK/KEK/media plaintext.

Crypto/key-custody code is never forced into Low-band delegation.

## P2.T3 — Availability Node executor

Implement only the T0-selected `AN-R1 + AN-A1` runtime/auth contract. Seed/open ciphertext package bytes and return deterministic idempotent publication evidence.

- **HP-T3-1:** authenticated same-identity publish returns stable Hyperdrive/publication evidence.
- **EC-T3-1:** same identity + different package/hash fails conflict-safe.
- **EC-T3-2:** service has no DB/business/key authority and cannot decrypt package.

## P2.T4 — O4 dispatcher + optional queue accelerator + reconciler

- Claim committed outbox work.
- Dispatch directly or through optional existing queue acceleration.
- Re-drive stale/unknown publication from PostgreSQL truth.
- Reconcile lost dispatch, remote-success/ACK-loss, local Ready-commit loss.
- Duplicate-after-Ready is idempotent.

- **HP-T4-1:** committed-before-dispatch crash is recovered.
- **HP-T4-2:** queue unavailable -> reconciliation/direct dispatch still converges.
- **EC-T4-1:** timeout/unknown result never yields Ready.
- **EC-T4-2:** duplicate queue delivery never rotates lineage or regresses Ready.

## P2.T5 — S-120 integration + P2P_READY

- P2 trigger/downstream seam after required prepared HLS exists.
- Preserve S-120 Ready/ASR behavior.
- Consume T1-T4 confirmation to transition Ready fail-closed.
- Produce minimal internal P2 descriptor for future P3; no viewer/device envelope API.

- **HP-T5-1:** S-120 Ready remains observable and ASR can start while P2 publication proceeds independently.
- **HP-T5-2:** confirmed same-lineage publication becomes P2P_READY.
- **EC-T5-1:** P2 failure does not undo/delay S-120 Ready; only P2 remains unavailable.
- **EC-T5-2:** queue ACK/reachability without durable confirmation stays non-ready.

## P2.T6 — audit + crash-window certification + closure

- Finalize/implement ADR-018 P2 audit inventory beginning from T0's minimum set.
- Cross-component integration tests for PostgreSQL, worker, optional queue path, and Availability Node contract.
- Six D3 crash-window scenarios.
- Ciphertext-only and secret-deny-list evidence.
- Status synchronization and owner verification.

- **HP-T6-1:** clean publication passes full path and durable audit evidence.
- **EC-T6-1:** each injected crash window converges without false Ready/second lineage.
- **EC-T6-2:** audit persistence failure on required success path fails closed where ADR-018 requires it.

## Parent closure criteria

P2 is PASS only when T0, T1, C0, and T2-T6 are closed, every behavior maps to passing executable evidence, integrated Reflection covers confidentiality, crash consistency, readiness separation, and scope, canonical docs are synchronized, and the repository owner performs final verification.

P3 remains blocked until that P2 PASS is recorded.
