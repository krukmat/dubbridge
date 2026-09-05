---
type: Plan
title: "Plan: MVP0-P2P P2 encrypted publication"
status: ready_for_approval
slice: MVP0-P2P
---

# P2 — encrypted P2P publication after S-120

## Objective

Turn an existing S-120 `PreparationStatus::Ready` HLS derivative into a
ciphertext-only K1 P2P package, publish that same logical package through the
Availability Node, and reach the separate durable `P2P_READY` predicate under the
accepted ADR-044/O4 contract without delaying S-120 Ready or ASR/transcription.

P2 does not implement invitations, viewer claims, device-envelope delivery, mobile
sync, local playback, dashboard UI, or no-HTTP-fallback certification.

## Governing decisions

- ADR-043: accepted mobile/Bare runtime ownership; P2 does not alter it.
- ADR-044: **Accepted 2026-09-05**.
  - D1 `O3 parallel` authorization.
  - D2 `K1` AES-256-GCM package / server-wrapped CK / HPKE-P256 device-envelope
    contract.
  - D3 `O4` PostgreSQL + transactional outbox authority, optional queue
    acceleration, PostgreSQL reconciliation, same-lineage idempotency.
- ADR-032 remains unchanged for review-time HTTP HLS.
- ADR-018 durable audit requirements apply to governance-significant P2 events.

## Parent RRI and mandatory re-scope

The unreduced P2 phase crosses storage, database, migrations, cryptographic key
custody, async/distributed publication, worker orchestration, Availability Node,
recovery, audit, and integration verification. Treating it as one implementation
unit is intentionally forbidden.

Conservative planning score: **RRI 131 — Excessive — Effort XL**.

Planning inputs: C=4, F=5, D=5, T=4, A=0, K=5, P=5, X=5; penalties retained for
security-sensitive key handling, >10-file scope, missing-new-area tests with high
impact, high complexity/domain coupling, and unresolved phase-level operational
architecture. The exact `scripts/rri.py` report is re-run for every executable
child at presentation time; this phase-level score exists only to enforce the
mandatory decomposition.

No direct P2 source execution exists. P2 is decomposed into independently gated
parents below.

## Architecture

```text
S-120 HLS Ready
      |
      | existing pipeline remains complete / ASR may enqueue
      v
P2.T1 durable publication identity + outbox intent (PostgreSQL)
      |
      v
P2.T2 K1 encrypted package construction
      |
      v
P2.T3 Availability Node publication executor
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

The queue is never authority. Availability Node reachability is never readiness.
Unknown remote outcome stays non-ready. The logical package identity and K1 lineage
cannot change merely because work is retried or redelivered.

## Workstream sequence

### P2.T0 — operational/trust contract freeze

Resolve the remaining ADR-044 question-4 implementation contract before P2 deploys:

- Availability Node process/runtime ownership and repository location;
- narrow authenticated publication-control surface;
- how the publisher proves/request-idempotency identity without DB credentials;
- health/observability needed for O4 reconciliation;
- secret/config allow-list and deny-list;
- concrete PostgreSQL publication/outbox state names sufficient for unambiguous
  implementation, without changing O4 authority.

No source implementation in T0. Owner selection is required because this freezes an
architecture/security boundary.

### P2.T1 — durable publication identity + outbox foundation

Create the P2P publication domain/persistence foundation:

- stable logical publication/package identity and K1 lineage reference;
- non-ready publication state machine;
- durable outbox intent created atomically with publication state;
- repository operations for claim/retry/reconcile/confirm transitions;
- migration constraints preventing duplicate logical publication or invalid Ready.

### P2.T2 — K1 package construction

Build encrypted package material from existing S-120 HLS:

- one fresh 256-bit CK for a new logical package lineage;
- AES-256-GCM encryption of each package file with unique per-file nonce;
- deterministic/versioned authenticated context and package manifest;
- manifest/hash evidence over ciphertext package content;
- server-wrapped CK under the configured versioned KEK boundary;
- plaintext CK transient only and never logged/persisted;
- retry of the same logical publication consumes the already-frozen lineage rather
  than silently rotating package identity/CK.

### P2.T3 — Availability Node publication executor

Implement the T0-frozen narrow publication-control contract and ciphertext seeding:

- accepts only authenticated publication commands and non-secret package metadata;
- opens/seeds ciphertext package bytes only;
- idempotently returns existing publication evidence for the same logical identity;
- never receives PostgreSQL credentials, plaintext CK, server KEK, invitation/viewer
  state, business authorization, or backend signing authority.

### P2.T4 — O4 dispatch + reconciliation

Implement delivery/recovery mechanics:

- outbox dispatcher;
- optional existing queue adapter as an accelerator only;
- bounded claim/lease/retry behavior;
- PostgreSQL-driven reconciler for lost dispatch, stale work, unknown outcome,
  remote-success/ACK-loss, and local Ready-commit loss;
- idempotent duplicate-after-Ready behavior;
- no exactly-once claim.

### P2.T5 — S-120 integration + fail-closed Ready transition

Wire P2 downstream of S-120 without changing S-120 semantics:

- create/ensure publication intent only after the required prepared HLS derivative
  exists;
- never block or roll back `PreparationStatus::Ready` or transcription enqueue;
- transition semantic `P2P_READY` only when PostgreSQL durably proves same-lineage
  package construction and external publication confirmation;
- expose only the minimal internal descriptor P3 will later consume.

### P2.T6 — audit, crash-window certification, and closure

Close P2 with executable evidence for all O4 crash windows and K1 confidentiality:

- durable ADR-018 audit rows for the P2 governance events frozen during T0/T6
  implementation planning;
- integration coverage over PostgreSQL + worker + Availability Node contract;
- lost dispatch, duplicate delivery, unknown result, lost ACK, lost Ready commit,
  duplicate-after-Ready;
- ciphertext-only publication inspection;
- S-120/ASR non-regression;
- no P2 source path can advertise Ready from queue ACK/reachability alone.

## Honest Low-band maximization

The P2 parent remains high-risk. Low-band leaves are permitted only where they are
truly pure/mechanical and independently verifiable. Candidate seams to score again
at child presentation time include:

- pure stable-identity/value-object parsing and validation;
- deterministic manifest serialization/hash helpers that do not perform crypto or
  key custody;
- pure retry/backoff decision helpers with no DB/queue side effect;
- redaction/diagnostic formatting helpers;
- test fixtures and deterministic crash-window scenario builders.

Do **not** force these into Low:

- migration/state constraints;
- AES-GCM/KEK handling;
- DB state transitions/outbox claiming;
- Availability Node authentication;
- queue/reconciler side effects;
- `P2P_READY` transition;
- governance audit success-path behavior.

Those surfaces retain their real RRI band and owner gates.

## Affected module boundaries

Candidate implementation surfaces; each child freezes its exact allowed paths before
execution:

- `crates/domain/src/**` — P2P publication identities/states only;
- `crates/db/src/**` — P2 publication/outbox repository;
- `infra/migrations/**` — P2 publication/outbox schema;
- `crates/storage/**` — only if a bounded read/stream seam is needed; existing
  `StorageAdapter` semantics stay authoritative;
- `crates/jobs/**` — optional queue acceleration only;
- `apps/worker-runner/src/**` — package build, dispatch, reconcile integration;
- a T0-selected Availability Node application/package path;
- config/test/docs files strictly needed by the selected child.

P2 must not modify mobile product runtime, invitation APIs, P3 device-envelope logic,
S-125 review playback, or legacy HTTP media behavior.

## Behavioral acceptance

### Happy paths

- **HP-P2-1:** existing S-120 Ready HLS -> one K1 ciphertext package -> confirmed
  Availability Node publication -> authoritative PostgreSQL `P2P_READY`, while ASR
  remains independently enqueueable.
- **HP-P2-2:** process restart after durable publication intent but before external
  dispatch -> the same logical package/lineage is recovered and published exactly
  once logically under at-least-once delivery.
- **HP-P2-3:** optional queue acceleration is removed/unavailable -> PostgreSQL
  outbox/reconciler still recovers and completes the same publication.

### Edge cases

- **EC-P2-1:** timeout/unknown remote result -> remain non-ready; same-lineage
  reconciliation must prove or safely re-drive the publication.
- **EC-P2-2:** remote publication succeeds but ACK is lost -> retry/reconcile returns
  existing same-identity evidence; no second logical package or CK lineage appears.
- **EC-P2-3:** duplicate delivery after Ready -> idempotent no-op; no state regression
  or key/package rotation.
- **EC-P2-4:** plaintext CK/KEK leakage attempt or Availability Node secret-scope
  expansion -> fail closed and no publication success.
- **EC-P2-5:** P2 failure -> S-120 Ready and downstream transcription remain valid;
  only P2 readiness stays unavailable.

## Verification strategy

Every implementation parent must map its HP/EC cases to executable evidence under
`behavior-v2`. P2 closure requires, at minimum:

- migration/repository integration tests with PostgreSQL;
- K1 crypto test vectors and nonce-uniqueness/error coverage;
- Availability Node contract tests;
- worker/outbox/reconciler integration tests;
- deterministic failure injection for all six D3 crash windows;
- audit persistence tests for frozen P2 governance events;
- S-120 Ready + ASR non-regression tests;
- ciphertext inspection proving the Availability Node receives no plaintext media or
  plaintext CK.

## Gates

1. ADR-044 accepted — **satisfied 2026-09-05**.
2. P2 parent planning/decomposition — **this plan**.
3. P2.T0 architecture/security contract — **next owner gate**.
4. Each T1-T6 parent is independently scored/presented/approved before source work.
5. P2 closes only after T0-T6 PASS, integrated Reflection, coverage certification,
   status synchronization, and owner verification.
6. P3 remains blocked until P2 PASS.
