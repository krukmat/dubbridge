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
- Source authorization: **none for T1**. `P2.T1` is the next explicit owner gate.
- Review exception: existing owner-directed MVP0-P2P P0-P7 phase-1/phase-2 review override remains in force; it does not waive RRI/HITL/Reflection/tests.

## Task map

The child scores below are conservative planning scores used to choose the gate. Every executable child must be re-scored against its frozen exact paths immediately before source execution; no score may be silently lowered to obtain a cheaper route.

| ID | Objective | Planning RRI | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| P2.T0 | Freeze Availability Node trust/operation + concrete O4 implementation contract | 64 Complex | L | **PASS** | ADR-044 Accepted |
| P2.T1 | Durable publication identity, state machine, migration + transactional outbox repository | 78 High | XL | **NEXT OWNER GATE** | T0 PASS |
| P2.T2 | K1 encrypted package construction + server-wrapped CK custody | 82 High | XL | Pending | T1 PASS |
| P2.T3 | Availability Node ciphertext publication executor | 72 High | XL | Pending | T0 + T2 PASS |
| P2.T4 | O4 outbox dispatch, optional queue acceleration + reconciler | 84 High | XL | Pending | T1 + T3 PASS |
| P2.T5 | S-120 downstream integration + fail-closed P2P_READY transition | 74 High | XL | Pending | T2 + T4 PASS |
| P2.T6 | ADR-018 audit inventory, crash-window integration certification + P2 closure | 76 High | XL | Pending | T1-T5 PASS |

Every High/Complex implementation parent is decomposed again before source edits. Low-band maximization applies only at real pure/mechanical seams.

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

## P2.T1 — durable publication + transactional outbox persistence — NEXT OWNER GATE

### Scope

Freeze and implement only the PostgreSQL/domain foundation required by O4:

- stable logical publication/package identity and lineage reference;
- semantic publication state with T0-frozen readiness semantics;
- migration for publication state and durable outbox intent;
- atomic create/ensure publication + outbox transaction;
- repository operations required for outstanding-work lookup and later claim/reconcile/confirmation transitions;
- database invariants preventing duplicate active logical publication or an invalid direct transition to `ready`.

T1 does **not** implement K1 media encryption, Availability Node runtime, mTLS transport, queue dispatch, S-120 trigger wiring, P3 invitation/device envelope, or final P2 audit certification.

### Candidate path envelope

Exact paths are frozen and re-scored at execution presentation. Expected bounded surfaces are:

- new `crates/domain/src/p2p_publication.rs` plus `crates/domain/src/lib.rs` export;
- new `crates/db/src/p2p_publication_repo.rs` plus `crates/db/src/lib.rs` export;
- one new `infra/migrations/<next>_create_p2p_publications_and_outbox.sql` migration;
- focused domain + PostgreSQL repository integration tests;
- T1 task/audit/status documentation only.

No `apps/**`, `mobile/**`, `crates/storage/**`, `crates/jobs/**`, crypto implementation, or Availability Node source belongs in T1.

### Behavioral acceptance

- **HP-T1-1:** one transaction creates one non-ready logical publication identity plus one durable publication outbox obligation.
- **HP-T1-2:** restart/re-read observes the same identity/lineage and outstanding obligation; no side effect is required to rediscover the work.
- **EC-T1-1:** duplicate create for the same logical lineage converges idempotently or returns an explicit conflict; it never creates a second logical package.
- **EC-T1-2:** PostgreSQL constraints/repository API cannot mark the publication `ready` without durable same-lineage external-confirmation evidence.
- **EC-T1-3:** unknown/stale in-flight work remains non-ready and is representable for later T4 reconciliation.
- **EC-T1-4:** T1 persistence contains no plaintext CK/KEK, raw invite token, service private key, or application signing secret.

### Planning gate

Planning RRI remains **78 High / Effort XL**. The exact repository RRI command must be run against the final frozen file set before source execution; if that score differs materially, the higher/current result governs. Until that execution presentation is owner-approved, **T1 source is not authorized**.

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

P2 is PASS only when T0-T6 are individually closed, every behavior maps to passing executable evidence, integrated Reflection covers confidentiality, crash consistency, readiness separation, and scope, canonical docs are synchronized, and the repository owner performs final verification.

P3 remains blocked until that P2 PASS is recorded.
