---
type: TaskList
title: "Tasks: ADR044-D3 publication/outbox decision"
status: complete
slice: MVP0-P2P
parent: ADR044-D3
---

# ADR044-D3 — publication/outbox parent envelope

## Decision header

- **Status:** COMPLETE 2026-09-05.
- **Parent RRI:** **60 → Complex → Effort L**.
- **Decision:** resolve ADR-044 question 3: durable P2P publication state, intent/outbox semantics, idempotent recovery, and the exact meaning of P2P-ready.
- **Dependency:** ADR044-D2 PASS (`K1`).
- **Owner approval:** explicit `aprobado` authorized S1-S4; explicit `apruebo o4. documentalo` selected O4 at the mandatory owner checkpoint.
- **Selected contract:** **O4 — transactional outbox + optional queue accelerator + PostgreSQL reconciler**.
- **ADR status:** remains `Proposed`.
- **P2 source authorization:** none.
- **Local/device precheck:** `n/a` — cloud environment; docs/ADR/task-ledger-only.
- **Phase-1/phase-2 review:** `n/a` — docs/ADR/task-ledger exemption.
- **Full evidence:** `docs/audit/mvp0-p2p-adr044-d3-publication.md`.

## Task map

| Task | Objective | Status | RRI | Depends on |
|---|---|---|---:|---|
| `ADR044-D3-S1` | Freeze canonical readiness/publication/recovery constraints | Complete 2026-09-05 | 14 | D3 approval |
| `ADR044-D3-S2` | Neutral publication-lifecycle / intent matrix | Complete 2026-09-05 | 14 | S1 |
| `ADR044-D3-S3` | Neutral idempotency / crash-recovery matrix | Complete 2026-09-05 | 14 | S1 |
| `ADR044-D3-S4` | Runtime-responsibility / fail-closed readiness matrix | Complete 2026-09-05 | 14 | S1 |
| `ADR044-D3-OWNER` | Owner selection | Complete 2026-09-05 — `O4` | human | S2-S4 |
| `ADR044-D3-S5` | Mechanical codification in audit + ADR | Complete 2026-09-05 | 13 | OWNER |
| `ADR044-D3-SYNC` | Canonical status propagation, ADR remains Proposed/P2 blocked | Complete 2026-09-05 | 17 | S5 |

## Frozen constraints

D3 closes with these invariant requirements:

1. `PreparationStatus::Ready` remains the existing S-120 HLS-readiness signal. P2P publication must not delay S-120 Ready or its downstream transcription enqueue.
2. P2P readiness is a separate durable predicate and is false until publication is externally confirmed and durably recorded.
3. PostgreSQL is source of truth for logical publication identity, structured publication metadata, durable work intent and current P2P publication state.
4. Dispatch, enqueue, queue acknowledgement and Availability Node reachability are never equivalent to publication success.
5. Delivery is at-least-once and idempotent; D3 makes no distributed exactly-once claim.
6. A crash/timeout with unknown remote outcome remains fail-closed until deterministic reconciliation establishes the result or safely re-drives the same logical publication.
7. Retries reuse one stable logical package identity and D2/K1 server-wrapped CK lineage. Retry never silently creates a new package/key lineage.
8. Availability Node remains ciphertext-only and non-authoritative for business state, DB state, authorization, KEK, plaintext CK, invite/viewer data, or backend signing keys.
9. D2/K1 device-envelope release consumes the D3 P2P-ready predicate as a fail-closed precondition.
10. ADR-032 review playback is unchanged.

## Evidence leaves

### S1 — constraint register

Complete. S1 froze dual readiness, PostgreSQL authority, cross-system non-atomicity, durable recovery, idempotent same-lineage retry, Availability Node secret denial and D2 release dependency. Detailed register: D3 audit.

### S2 — neutral option comparison

The owner received these complete contracts:

| Option | Durable model | Main benefit | Main cost |
|---|---|---|---|
| `O1 transactional-outbox` | Publication state + separate outbox intent committed in one PostgreSQL transaction | Explicit crash-consistent durable intent and clean work/product separation | Outbox/dispatcher plus external reconciliation still required |
| `O2 durable-state-reconciler` | Publication row is both product state and work source | Fewer conceptual records | Product state, leases, retries and recovery become tightly coupled |
| `O3 queue-primary-with-reconciliation` | PostgreSQL state + queue normal path + reconciler | Reuses queue orchestration | Two execution mechanisms and greater stale/race burden |

No leaf selected a winner.

### S3 — crash / idempotency evidence

Every acceptable contract had to close these windows:

| Window | Required outcome |
|---|---|
| `W0` before durable intent commit | no publication obligation |
| `W1` after durable commit, before dispatch | same durable work is recoverable |
| `W2` during external operation | non-ready/unknown; same identity retried/reconciled |
| `W3` remote success, ACK lost | no second logical package; same identity converges on existing result |
| `W4` ACK received, before local Ready commit | remains non-ready and recoverable until authoritative Ready commit |
| `W5` duplicate after Ready | idempotent no-op; no CK/package rotation or state regression |

### S4 — responsibility evidence

- PostgreSQL/control plane owns authoritative logical publication identity/state, durable work evidence and the Ready transition.
- Worker/package builder creates or reuses the same K1 lineage and performs dispatch/reconciliation.
- `StorageAdapter` remains the binary-artifact seam, not product-state authority.
- Availability Node writes/seeds ciphertext and returns publication evidence only.
- P3 key release consumes authoritative D3 readiness and never infers readiness from Hyperdrive/queue/cache possession.

## D3-OWNER — selected O4

The owner selected **O4** after reviewing the long-term hybrid option.

O4 is defined as:

> **Transactional outbox is the durable consistency authority; an existing/future queue may be used only as a replaceable delivery accelerator; a PostgreSQL-driven reconciler is the recovery safety net. Queue state never establishes publication success. Only durable confirmation of the same logical publication may transition the authoritative PostgreSQL record to `P2P_READY`.**

### Authority hierarchy

```text
PostgreSQL publication state + transactional outbox
        = durable authority

queue
        = optional delivery accelerator

PostgreSQL reconciler
        = recovery safety net

Availability Node
        = ciphertext executor/evidence source
```

### Durable transaction

Before an external publication obligation exists, one PostgreSQL transaction establishes:

- stable logical package/publication identity;
- non-ready publication state;
- K1 lineage reference;
- durable outbox publication intent.

The external Availability Node operation is intentionally outside that transaction.

### Dispatch and queue

The outbox dispatcher may invoke a worker directly or enqueue the same logical publication onto the repository's queue/job mechanism. Queue usage is an implementation/scale choice; D3 does not require a new generic event bus or new queue technology.

Queue enqueue/ACK is never publication success. Queue loss cannot lose the publication obligation because authoritative outbox/publication state remains durable in PostgreSQL.

### Reconciliation

The reconciler operates from authoritative PostgreSQL state and must eventually detect/recover:

- committed outbox work never dispatched;
- dispatched/enqueued work never completed;
- later-defined stale/abandoned worker claims;
- queue success while PostgreSQL is still non-ready;
- unknown external outcome after timeout/disconnect/crash;
- remote success whose acknowledgement was lost before local Ready commit.

### Idempotency

Outbox retry, queue duplicate/redelivery, reconciler re-drive and Availability Node confirmation all use the same stable logical publication identity. The encrypted package identity, manifest/hash evidence and K1 wrapped-key lineage remain stable for retries.

A future explicit package replacement is a new lineage and requires a separately approved transition; it cannot occur as an incidental retry effect.

## Fail-closed `P2P_READY`

The semantic predicate is true only when authoritative PostgreSQL state durably proves:

1. current logical lineage is not abandoned/replaced;
2. K1 encrypted package construction completed for that lineage;
3. external publication of that same lineage is confirmed, not merely dispatched/enqueued;
4. persisted publication identifiers/evidence correspond to that lineage;
5. no unresolved unknown-outcome condition remains.

Any missing, stale, conflicting or unknown fact means **not P2P-ready** and therefore blocks D2 device-envelope release.

## Alternatives not selected

- `O1` remains O4's consistency core, but the owner chose to make queue acceleration and reconciliation explicit in the long-term contract.
- `O2` was not selected because it couples product state too closely with leasing/retry mechanics.
- `O3` was not selected because queue-primary operation creates a larger dual-mechanism correctness burden. O4 keeps the queue subordinate and replaceable.

## Integrated Reflection — parent RRI 60

1. **Transactional/crash-window correctness — PASS.** Local publication state and durable intent are atomic in PostgreSQL; external side effects remain explicitly non-atomic and recoverable.
2. **Idempotency/unknown outcome — PASS.** Stable logical identity spans outbox, queue, reconciler and Availability Node; unknown outcome never produces readiness.
3. **S-120 independence / D2 interaction — PASS.** S-120 Ready and ASR remain independent; separate P2P readiness gates K1 release.
4. **Scope/status — PASS.** D3 selects semantic authority/recovery only. It does not select SQL/table/field names, route/RPC encoding, queue product, retry constants, Availability Node credentials/deployment, certification profile, or the complete audit-event inventory. ADR-044 remains `Proposed`; D4/P2 remain separately gated.

## Parent RRI

Parent inputs remain C1/F3/D5/T0/A0/K4/P4/X4 plus `arch_decision +12`: **RRI 60 → Complex → Effort L**.

S1-S4: **RRI 14 Low S**. S5: **RRI 13 Low S**. SYNC: **RRI 17 Low S**.

## Acceptance

- S1-S4 neutral evidence: **PASS**.
- Explicit owner selection: **PASS — O4**.
- Separate S-120/P2P readiness: **PASS**.
- Durable intent / crash-window recovery: **PASS**.
- Unknown outcome fail-closed: **PASS**.
- K1 lineage continuity on retry: **PASS**.
- Availability Node ciphertext-only/non-authoritative: **PASS**.
- S5 mechanical codification: **PASS**.
- Four parent Reflection passes: **PASS**.
- D4/P2 scope creep: **none**.
- Repository CI / parity verification: recorded after final sync head.

## Approval history

- 2026-09-05: parent D3 presented at RRI 60.
- 2026-09-05: owner replied `aprobado`; S1-S4 authorized and executed.
- 2026-09-05: execution stopped at mandatory `D3-OWNER` checkpoint.
- 2026-09-05: owner selected `O4` with `apruebo o4. documentalo`.
- 2026-09-05: S5, four integrated Reflections and SYNC executed mechanically from that selection.

D3 is complete. ADR-044 remains `Proposed`; D4 and P2 require their own gates.
