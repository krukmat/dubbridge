---
type: Plan
title: "Plan: MVP0-P2P P2 encrypted publication"
status: in_progress
slice: MVP0-P2P
---

# P2 — encrypted P2P publication after S-120

## Objective

Turn an existing S-120 `PreparationStatus::Ready` HLS derivative into a ciphertext-only K1 P2P package, publish that same logical package through the Availability Node, and reach the separate durable `P2P_READY` predicate under the accepted ADR-044/O4 contract without delaying S-120 Ready or ASR/transcription.

P2 does not implement invitations, viewer claims, device-envelope delivery, mobile sync, local playback, dashboard UI, or no-HTTP-fallback certification.

## Governing decisions

- ADR-043: accepted mobile/Bare runtime ownership; P2 does not alter it.
- ADR-044: **Accepted 2026-09-05**.
  - D1 `O3 parallel` authorization.
  - D2 `K1` AES-256-GCM package / server-wrapped CK / HPKE-P256 device-envelope contract.
  - D3 `O4` PostgreSQL + transactional outbox authority, optional queue acceleration, PostgreSQL reconciliation, same-lineage idempotency.
- P2.T0: **PASS 2026-09-05**.
  - `AN-R1`: dedicated Node.js/TypeScript Availability Node.
  - `AN-A1`: mTLS service identity on the private publication-control surface.
  - accepted semantic state model: `building -> publish_pending -> publishing -> reconciling -> ready`, with `failed` terminal only.
  - accepted minimum ADR-018 audit inventory: intent created; K1 lineage sealed/server-wrapped; external publication confirmed; reconciliation entered; `P2P_READY`; terminal publication failure.
- ADR-032 remains unchanged for review-time HTTP HLS.
- ADR-018 durable audit requirements apply to governance-significant P2 events.

## Parent RRI and mandatory re-scope

The unreduced P2 phase crosses storage, database, migrations, cryptographic key custody, async/distributed publication, worker orchestration, Availability Node, recovery, audit, and integration verification. Treating it as one implementation unit is intentionally forbidden.

Conservative planning score: **RRI 131 — Excessive — Effort XL**.

No direct P2 source execution exists. P2 is decomposed into independently gated parents T0-T6. T0 is complete. The original T1 parent scored **78 High / XL** and is now also **non-executable**; it was decomposed into T1a-T1f after the owner requested lower-complexity tasks.

The next owner gate is **P2.T1a**.

## Architecture

```text
S-120 HLS Ready
      |
      | existing pipeline remains complete / ASR may enqueue
      v
P2.T1a domain contract
      |
      v
P2.T1b PostgreSQL schema
      |
      v
P2.T1c atomic publication + outbox write
      |
      v
P2.T1d read model / outstanding work
      |
      v
P2.T1e guarded transitions / confirmation evidence
      |
      v
P2.T1f persistence certification
      |
      v
P2.T2 K1 encrypted package construction
      |
      v
P2.T3 AN-R1 Availability Node publication executor (mTLS / AN-A1)
      |
      +------ optional queue acceleration ------+
      |                                         |
      v                                         v
P2.T4 O4 dispatcher / recovery / reconciler <---+
      |
      v
same-lineage durable confirmation
      |
      v
PostgreSQL P2P_READY
      |
      v
P3 may later consume readiness + K1 lineage
```

The queue is never authority. Availability Node reachability is never readiness. Unknown remote outcome stays non-ready. Logical package identity and K1 lineage cannot change merely because work is retried or redelivered.

## Workstream sequence

### P2.T0 — operational/trust contract freeze — PASS

Owner-approved contract:

- dedicated Node.js/TypeScript Availability Node, operationally independent from mobile Bare;
- private/non-public publication-control endpoint authenticated with mTLS service identity;
- same-identity/same-package publication is idempotent; same identity with conflicting package/hash fails closed;
- health and publication evidence exist only to support O4 reconciliation, never to replace PostgreSQL authority;
- Availability Node secret deny-list includes PostgreSQL credentials, plaintext CK, KEK, invite/viewer/business authorization, application JWT signing material, and service private credentials in payload/logs;
- semantic publication state is `building -> publish_pending -> publishing -> reconciling -> ready`; unknown external outcome stays `reconciling`; `failed` is terminal only;
- minimum P2 ADR-018 event set accepted as recorded in the T0 selection audit.

Evidence: `docs/audit/mvp0-p2p-p2-t0-selection.md`.

### P2.T1 — persistence parent — SUPERSEDED AS EXECUTABLE GATE

The former RRI-78 parent is now only a grouping container for six lower-RRI leaves. No source work is authorized under `P2.T1` directly.

Canonical decomposition: `docs/audit/mvp0-p2p-p2-t1-decomposition.md`.

#### P2.T1a — pure domain identity/state contract — NEXT OWNER GATE

Planning **RRI 22 Low / Effort S**.

Only pure Rust domain semantics:

- stable logical publication identity;
- stable K1 lineage reference;
- T0 state model;
- pure transition/readiness guards;
- unit tests for lineage stability, unknown/reconciling behavior, invalid direct Ready, terminal/regressive transitions.

No PostgreSQL, migration, repository, crypto, worker, queue, Availability Node, or S-120 source.

Approval card: `docs/audit/mvp0-p2p-p2-t1a-approval-card.md`.

#### P2.T1b — PostgreSQL schema + constraints

Planning **RRI 32 Medium / Effort S/M**.

Only one migration introducing the publication/outbox persistence structures and schema-level identity/lineage/readiness constraints. No Rust repository behavior.

#### P2.T1c — atomic create/ensure + outbox write

Planning **RRI 47 Medium-high / Effort M**.

Only the minimal DB write repository path that creates/ensures the publication and initial outbox obligation in the same PostgreSQL transaction. No scans/transitions/dispatch.

#### P2.T1d — read model / outstanding work

Planning **RRI 36 Medium / Effort S/M**.

Only read-side repository behavior for stable identity lookup and outstanding durable obligations. No mutation/claim/lease.

#### P2.T1e — guarded transitions + confirmation persistence

Planning **RRI 44 Medium-high / Effort M**.

Only lifecycle state mutations, same-lineage confirmation evidence persistence, and the fail-closed durable Ready guard. No external calls.

#### P2.T1f — persistence certification

Planning **RRI 33 Medium / Effort M**.

Only integration/negative evidence for atomicity, restart/re-read, duplicate create, invalid Ready, and secret-deny-list inspection. Defects reopen the responsible implementation leaf rather than expanding certification scope.

### P2.T2 — K1 package construction

Build encrypted package material from existing S-120 HLS:

- one fresh 256-bit CK for a new logical package lineage;
- AES-256-GCM encryption of each package file with unique per-file nonce;
- deterministic/versioned authenticated context and package manifest;
- manifest/hash evidence over ciphertext package content;
- server-wrapped CK under the configured versioned KEK boundary;
- plaintext CK transient only and never logged/persisted;
- retry of the same logical publication consumes the already-frozen lineage rather than silently rotating package identity/CK.

### P2.T3 — Availability Node publication executor

Implement the T0-frozen `AN-R1 + AN-A1` contract:

- Node.js/TypeScript service;
- mTLS-authenticated private control surface;
- accepts only stable publication identity plus non-secret ciphertext metadata/material references;
- opens/seeds ciphertext package bytes only;
- idempotently returns existing publication evidence for the same logical identity;
- never receives PostgreSQL credentials, plaintext CK, server KEK, invitation/viewer state, business authorization, or backend signing authority.

### P2.T4 — O4 dispatch + reconciliation

Implement delivery/recovery mechanics:

- outbox dispatcher;
- optional existing queue adapter as an accelerator only;
- bounded claim/lease/retry behavior;
- PostgreSQL-driven reconciler for lost dispatch, stale work, unknown outcome, remote-success/ACK-loss, and local Ready-commit loss;
- idempotent duplicate-after-Ready behavior;
- no exactly-once claim.

### P2.T5 — S-120 integration + fail-closed Ready transition

Wire P2 downstream of S-120 without changing S-120 semantics:

- create/ensure publication intent only after the required prepared HLS derivative exists;
- never block or roll back `PreparationStatus::Ready` or transcription enqueue;
- transition semantic `P2P_READY` only when PostgreSQL durably proves same-lineage package construction and external publication confirmation;
- expose only the minimal internal descriptor P3 will later consume.

### P2.T6 — audit, crash-window certification, and closure

Close P2 with executable evidence for all O4 crash windows and K1 confidentiality:

- implement/finalize the P2 ADR-018 inventory beginning with T0's accepted minimum;
- integration coverage over PostgreSQL + worker + Availability Node contract;
- lost dispatch, duplicate delivery, unknown result, lost ACK, lost Ready commit, duplicate-after-Ready;
- ciphertext-only publication inspection;
- S-120/ASR non-regression;
- no P2 source path can advertise Ready from queue ACK/reachability alone.

## Behavioral acceptance

### Happy paths

- **HP-P2-1:** existing S-120 Ready HLS -> one K1 ciphertext package -> confirmed Availability Node publication -> authoritative PostgreSQL `P2P_READY`, while ASR remains independently enqueueable.
- **HP-P2-2:** process restart after durable publication intent but before external dispatch -> the same logical package/lineage is recovered and published once logically under at-least-once delivery.
- **HP-P2-3:** optional queue acceleration is removed/unavailable -> PostgreSQL outbox/reconciler still recovers and completes the same publication.

### Edge cases

- **EC-P2-1:** timeout/unknown remote result -> remain non-ready; same-lineage reconciliation must prove or safely re-drive publication.
- **EC-P2-2:** remote publication succeeds but ACK is lost -> retry/reconcile returns existing same-identity evidence; no second logical package or CK lineage appears.
- **EC-P2-3:** duplicate delivery after Ready -> idempotent no-op; no state regression or key/package rotation.
- **EC-P2-4:** plaintext CK/KEK leakage attempt or Availability Node secret-scope expansion -> fail closed and no publication success.
- **EC-P2-5:** P2 failure -> S-120 Ready and downstream transcription remain valid; only P2 readiness stays unavailable.

## Verification strategy

Every implementation leaf must map its HP/EC cases to executable evidence under `behavior-v2`. P2 closure requires migration/repository integration tests, K1 crypto vectors, Availability Node contract tests, worker/outbox/reconciler integration tests, deterministic failure injection for all six D3 crash windows, audit persistence tests, S-120/ASR non-regression, and ciphertext inspection.

## Gates

1. ADR-044 accepted — **satisfied 2026-09-05**.
2. P2 parent planning/decomposition — **satisfied**.
3. P2.T0 architecture/security contract — **PASS 2026-09-05 (`AN-R1 + AN-A1`)**.
4. Original P2.T1 High-band gate — **superseded/non-executable**.
5. P2.T1a domain contract — **next owner gate**.
6. T1b-T1f follow sequentially with independent exact-path RRI/HITL as required.
7. T2-T6 are independently scored/presented/approved after their dependencies pass and may themselves be decomposed further.
8. P2 closes only after T0, T1a-T1f, and T2-T6 PASS, integrated Reflection, coverage certification, status synchronization, and owner verification.
9. P3 remains blocked until P2 PASS.
