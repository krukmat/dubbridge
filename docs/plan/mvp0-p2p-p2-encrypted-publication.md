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
- P2.C0: **PASS 2026-09-06**.
  - `p2p-manifest-v1` uses restricted RFC 8785/JCS canonical UTF-8 JSON, normalized relative paths, deterministic ordering, SHA-256 lowercase-hex digests, and shared golden fixtures.
  - `p2p-aad-v1` + AES-256-GCM use one 96-bit CSPRNG nonce per file/CK lineage; retries reuse sealed ciphertext rather than re-encrypt the same lineage.
  - CK is generated once per new lineage and wrapped under a versioned server KEK; retry never silently rotates CK/KEK/lineage.
  - `availability-publication-v1` is a private mTLS PUT contract with same-lineage/digest idempotency and conflict-safe evidence.
  - P2 audit correlation uses publication/lineage correlation without fabricating ingestion tokens.
  - `p2p-ready-descriptor-v1` is the minimal internal handoff to P3 after authoritative `P2P_READY`.
  - T2-T6 are decomposed into exact-path leaves; optional queue acceleration is outside the October critical path.
- ADR-032 remains unchanged for review-time HTTP HLS.
- ADR-018 durable audit requirements apply to governance-significant P2 events.

Canonical C0 evidence:

- `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md`
- `docs/audit/mvp0-p2p-p2-c0-rri.md`
- `docs/fixtures/mvp0-p2p-manifest-v1.json`
- `docs/fixtures/mvp0-p2p-publication-contract-v1.json`

## Parent RRI and mandatory re-scope

The unreduced P2 phase crosses storage, database, migrations, cryptographic key custody, async/distributed publication, worker orchestration, Availability Node, recovery, audit, and integration verification. Treating it as one implementation unit is intentionally forbidden.

Conservative planning score: **RRI 131 — Excessive — Effort XL**.

P2 is decomposed into independently gated parents T0-T6. T0, the decomposed T1 persistence leaves, and C0 are complete. The original T1 parent scored **78 High / XL**, became a non-executable container, and its T1a-T1f leaves are **Done and owner-approved as P2.T1 on 2026-09-06**.

The remaining P2 implementation begins only through the C0-frozen exact-path leaves under T2-T6. Each executable leaf is scored with `scripts/rri.py` immediately before presentation/execution; C0 completion does not authorize source work.

## Architecture

```text
S-120 HLS Ready
      |
      | existing pipeline remains complete / ASR may enqueue
      v
P2.T1 durable publication/outbox persistence ✅
      |
      v
P2.C0 shared contract + fixture + path freeze ✅
      |
      +----------+----------------+----------------+
      v          v                v                v
P2.T2 K1      P2.T3 AN-R1      P2.T4 recovery   P2.T6 audit/test
builder       executor          kernel/client    harness
      |
      +----------+----------------+----------------+
                         |
                         v
              P2.T4 integration + P2.T5 S-120 join
      |
      v
same-lineage durable confirmation
      |
      v
PostgreSQL P2P_READY
      |
      v
P3 consumes p2p-ready-descriptor-v1
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

### P2.T1 — persistence parent — SUPERSEDED AS EXECUTABLE GATE

The former RRI-78 parent is now only a grouping container for six lower-RRI leaves. T1a-T1f are Done and were accepted by the owner as the completed P2.T1 outcome on 2026-09-06.

Canonical decomposition: `docs/audit/mvp0-p2p-p2-t1-decomposition.md`.

#### P2.T1a — pure domain identity/state contract — DONE

Planning **RRI 22 Low / Effort S**.

Only pure Rust domain semantics:

- stable logical publication identity;
- stable K1 lineage reference;
- T0 state model;
- pure transition/readiness guards;
- unit tests for lineage stability, unknown/reconciling behavior, invalid direct Ready, terminal/regressive transitions.

No PostgreSQL, migration, repository, crypto, worker, queue, Availability Node, or S-120 source.

#### P2.T1b — PostgreSQL schema + constraints — DONE

Planning **RRI 32 Medium / Effort S/M**.

Only one migration introducing the publication/outbox persistence structures and schema-level identity/lineage/readiness constraints. No Rust repository behavior.

#### P2.T1c — atomic create/ensure + outbox write — DONE

Planning **RRI 47 Medium-high / Effort M**.

Only the minimal DB write repository path that creates/ensures the publication and initial outbox obligation in the same PostgreSQL transaction. No scans/transitions/dispatch.

#### P2.T1d — read model / outstanding work — DONE

Planning **RRI 36 Medium / Effort S/M**.

Only read-side repository behavior for stable identity lookup and outstanding durable obligations. No mutation/claim/lease.

#### P2.T1e — guarded transitions + confirmation persistence — DONE

Planning **RRI 44 Medium-high / Effort M**.

Only lifecycle state mutations, same-lineage confirmation evidence persistence, and the fail-closed durable Ready guard. No external calls.

#### P2.T1f — persistence certification — DONE

Planning **RRI 33 Medium / Effort M**.

Only integration/negative evidence for atomicity, restart/re-read, duplicate create, invalid Ready, and secret-deny-list inspection. Defects reopen the responsible implementation leaf rather than expanding certification scope.

### P2.C0 — shared implementation contract freeze — PASS

**Owner-approved 2026-09-06; RRI 66 Complex / Effort L; four Reflection passes PASS.**

C0 freezes:

- manifest-v1 canonical serialization, normalized path order/digest encoding, and shared golden fixtures;
- AES-256-GCM, canonical AAD, unique 96-bit nonce allocation, and same-lineage retry behavior;
- generate-once CK sealing/retry semantics and versioned KEK resolver/rotation/zeroization boundary;
- ciphertext handoff to the Availability Node plus request, response, error, idempotency, and confirmation evidence;
- audit event/transaction/correlation map and the minimal P3 descriptor;
- exact path ownership, shared-file ownership, migration reservation order, and integration order for every T2-T6 implementation leaf.

C0 intentionally introduces no source or migration changes. Full contract and leaf/path matrix: `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md`.

### P2.T2 — K1 package construction — DECOMPOSED

C0 decomposition: `T2a` dedicated `crates/p2p` manifest/path contract; `T2b` S-120 package reader; `T2c` AES-GCM/AAD/nonce invariant; `T2d` generate-once CK + KEK wrapping; `T2e` additive K1 persistence (`0033`); `T2f` package assembly/seal; `T2g` crypto/golden certification.

Build encrypted package material from existing S-120 HLS:

- one fresh 256-bit CK for a new logical package lineage;
- AES-256-GCM encryption of each package file with unique per-file nonce;
- deterministic/versioned authenticated context and package manifest;
- manifest/hash evidence over ciphertext package content;
- server-wrapped CK under the configured versioned KEK boundary;
- plaintext CK transient only and never logged/persisted;
- retry of the same logical publication consumes the already-sealed lineage rather than silently rotating package identity/CK.

### P2.T3 — Availability Node publication executor — DECOMPOSED

C0 decomposition: `T3a` Node/TS service + v1 contract; `T3b` private mTLS; `T3c` persistent Hyperdrive + idempotency/conflict behavior; `T3d` contract/security certification.

Implement the T0/C0-frozen `AN-R1 + AN-A1` contract:

- Node.js/TypeScript service;
- mTLS-authenticated private control surface;
- accepts only stable publication identity plus non-secret ciphertext package reference/metadata;
- opens/seeds ciphertext package bytes only;
- idempotently returns existing publication evidence for the same logical identity;
- never receives PostgreSQL credentials, plaintext CK, server KEK, invitation/viewer state, business authorization, or backend signing authority.

### P2.T4 — O4 dispatch + reconciliation — DECOMPOSED

C0 decomposition: `T4a` pure recovery kernel; `T4b` PostgreSQL claims/leases (`0034`); `T4c` mTLS AN client; `T4d` outbox dispatcher; `T4e` worker/reconciler join; `T4f` recovery certification.

Implement delivery/recovery mechanics:

- PostgreSQL outbox dispatcher;
- bounded claim/lease/retry behavior;
- PostgreSQL-driven reconciler for lost dispatch, stale work, unknown outcome, remote-success/ACK-loss, and local Ready-commit loss;
- idempotent duplicate-after-Ready behavior;
- no exactly-once claim.

Optional queue acceleration is **outside the October P2 critical path** and receives a later separately-scored task if needed.

### P2.T5 — S-120 integration + fail-closed Ready transition — DECOMPOSED

C0 decomposition: `T5a` current-order characterization; `T5b` fail-contained activation; `T5c` authoritative ready/P3 descriptor read model; `T5d` S-120/ASR non-regression.

Wire P2 downstream of S-120 without changing S-120 semantics:

- existing preparation reaches `Ready` first;
- existing `prepare_transcription_post_ready(...)` call occurs before P2 activation;
- P2 activation failure cannot undo/delay S-120 Ready or suppress the transcription enqueue attempt;
- transition semantic `P2P_READY` only when PostgreSQL durably proves same-lineage package construction and external publication confirmation;
- expose only `p2p-ready-descriptor-v1` for future P3.

### P2.T6 — audit, crash-window certification, and closure — DECOMPOSED

C0 decomposition: `T6a` backward-compatible P2 audit correlation (`0035`); `T6b` six durable audit event mappings; `T6c` deterministic six-window crash harness; `T6d` ciphertext/secret-boundary certification; `T6e` P2 evidence/status closure.

Close P2 with executable evidence for all O4 crash windows and K1 confidentiality:

- implement/finalize the P2 ADR-018 inventory beginning with T0's accepted minimum;
- never fabricate `ingest_token` for P2 correlation;
- integration coverage over PostgreSQL + worker + Availability Node contract;
- lost dispatch, duplicate delivery, unknown result, lost ACK, lost Ready commit, duplicate-after-Ready;
- ciphertext-only publication inspection;
- S-120/ASR non-regression;
- no P2 source path can advertise Ready from queue ACK/reachability alone.

## Behavioral acceptance

### Happy paths

- **HP-P2-1:** existing S-120 Ready HLS -> one K1 ciphertext package -> confirmed Availability Node publication -> authoritative PostgreSQL `P2P_READY`, while ASR remains independently enqueueable.
- **HP-P2-2:** process restart after durable publication intent but before external dispatch -> the same logical package/lineage is recovered and published once logically under at-least-once delivery.
- **HP-P2-3:** optional queue acceleration is absent -> PostgreSQL outbox/reconciler still recovers and completes the same publication.

### Edge cases

- **EC-P2-1:** timeout/unknown remote result -> remain non-ready; same-lineage reconciliation must prove or safely re-drive publication.
- **EC-P2-2:** remote publication succeeds but ACK is lost -> retry/reconcile returns existing same-identity evidence; no second logical package or CK lineage appears.
- **EC-P2-3:** duplicate delivery after Ready -> idempotent no-op; no state regression or key/package rotation.
- **EC-P2-4:** plaintext CK/KEK leakage attempt or Availability Node secret-scope expansion -> fail closed and no publication success.
- **EC-P2-5:** P2 failure -> S-120 Ready and downstream transcription remain valid; only P2 readiness stays unavailable.

## Verification strategy

Every implementation leaf must map its HP/EC cases to executable evidence under `behavior-v2`. P2 closure requires migration/repository integration tests, K1 crypto vectors, cross-runtime golden-fixture conformance, Availability Node contract/mTLS tests, worker/outbox/reconciler integration tests, deterministic failure injection for all six D3 crash windows, audit persistence tests, S-120/ASR non-regression, and ciphertext inspection.

## Remaining workstreams and joins

C0 has replaced the former provisional decomposition with the exact-path leaf matrix in `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md`:

- **T2:** `T2a -> T2b/T2c -> T2d -> T2e -> T2f -> T2g`.
- **T3:** `T3a -> T3b -> T3c -> T3d`.
- **T4:** `T4a -> T4b`; `T4c` after T3 contract; then `T4d -> T4e -> T4f`.
- **T5:** `T5a` characterization before source hook; `T5b -> T5c -> T5d` after T2/T4 integration.
- **T6:** `T6a -> T6b`, then integrated `T6c/T6d -> T6e`.

These are calendar workstreams. Source authorship still follows the repository's one-task-at-a-time rule. ADR-040 per-module split authorship may be used only inside one eligible approved RRI 26–55 task with common base SHA, frozen interfaces, disjoint writable paths, one writer per path, shared files owned by the orchestrator, and whole-task integration/verification.

## S-230 October integration

C0 is now an explicit input to the deployment lane:

- `S-230-T6p-a` completion requires `S-230-T5 PASS + P2.T0 PASS + P2.C0 PASS`.
- `T6p-b` cannot begin until `T6p-a PASS`, C0's package contract remains frozen, and the T3 Availability Node contract subtree is PASS/stable.
- `T6p-c` proves local deployment wiring only.
- `T6p-d` remains blocked on `S-230-T6 PASS + T6p-c PASS + P2 PASS` and proves backend publication only, not invited Android playback.

## Gates

1. ADR-044 accepted — **satisfied 2026-09-05**.
2. P2 parent planning/decomposition — **satisfied**.
3. P2.T0 architecture/security contract — **PASS 2026-09-05 (`AN-R1 + AN-A1`)**.
4. Original P2.T1 High-band gate — **superseded/non-executable**.
5. P2.T1a-T1f durable persistence outcome — **Done / owner-approved 2026-09-06**.
6. P2.C0 shared contract/path freeze — **PASS 2026-09-06**.
7. T2-T6 exact-path leaves are independently scored, presented/approved where required, executed, verified, and joined after C0.
8. P2 closes only after T0, T1a-T1f, C0, and T2-T6 PASS, integrated Reflection, coverage/certification evidence, status synchronization, and owner verification.
9. P3 remains blocked until P2 PASS.
