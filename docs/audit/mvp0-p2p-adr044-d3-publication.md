---
type: Audit
title: "ADR044-D3 publication/outbox decision audit"
status: complete
slice: MVP0-P2P
parent: ADR044-D3
---

# ADR044-D3 — publication/outbox decision audit

Date: 2026-09-05

## Outcome

The repository owner explicitly selected **O4 — transactional outbox + optional queue accelerator + PostgreSQL reconciler** at the `ADR044-D3-OWNER` checkpoint.

O4 is not a second source of truth and is not a distributed exactly-once claim. It is the long-term form of the transactional-outbox option:

- PostgreSQL is authoritative for logical publication identity, publication state, durable intent, and the `P2P_READY` transition;
- the outbox entry is committed transactionally with the publication intent/state before any external side effect is required;
- delivery is at-least-once and idempotent;
- an existing/future queue may accelerate normal dispatch and worker scaling, but queue state or acknowledgement is never product truth or publication-success evidence;
- a PostgreSQL-driven reconciler is the recovery safety net for lost dispatch, stuck work, unknown external outcome, and acknowledgement-loss windows;
- Availability Node operations converge on one stable logical package identity and K1 key lineage;
- only durable confirmation of that same logical publication may transition the authoritative record to `P2P_READY`.

ADR-044 remains `Proposed`. This D3 decision does not accept the ADR and does not authorize P2 source work.

## Parent / leaf RRI

- `ADR044-D3`: **RRI 60 — Complex — Effort L** (`arch_decision +12`).
- `D3-S1` through `D3-S4`: **RRI 14 — Low — Effort S**.
- `D3-S5`: **RRI 13 — Low — Effort S**.
- `D3-SYNC`: **RRI 17 — Low — Effort S**.

The parent retained the architecture checkpoint. Low leaves did not select an option.

## S1 — frozen constraints

The selected contract preserves the approved D3 constraints:

1. `PreparationStatus::Ready` remains S-120 HLS readiness and does not wait for P2P publication or delay ASR/transcription enqueue.
2. P2P readiness is a separate durable predicate and is false until publication is confirmed and durably recorded.
3. PostgreSQL remains authoritative for structured publication metadata/current state.
4. External dispatch is never equivalent to publication success.
5. Delivery is at-least-once/idempotent; no distributed exactly-once guarantee is claimed.
6. Unknown external outcome remains non-ready until deterministic reconciliation establishes the result or safely re-drives the same logical publication.
7. Retries reuse the same logical package identity and D2/K1 server-wrapped CK lineage; retries cannot silently create a new package/key lineage.
8. Availability Node remains ciphertext-only and non-authoritative for business state, DB state, authorization, KEK, plaintext CK, invite/viewer data, or backend signing material.
9. D2/K1 device-envelope release remains fail-closed on the D3-defined P2P-ready predicate.
10. ADR-032 review playback remains unchanged.

## S2 — option evidence

The neutral option set before owner selection was:

| Option | Durable work model | Primary benefit | Main cost |
|---|---|---|---|
| `O1 transactional-outbox` | Publication record + separate outbox intent in one PostgreSQL transaction | Explicit crash-consistent intent and clean separation of work/product state | Requires outbox/dispatcher; external ack-loss still needs idempotency/reconciliation |
| `O2 durable-state-reconciler` | Publication row acts as product state and work source | Fewer concepts/records | Product state, leases, attempts and recovery become tightly coupled |
| `O3 queue-primary-with-reconciliation` | PostgreSQL authoritative state; queue drives normal dispatch; reconciler repairs drift | Reuses queue-oriented orchestration | Two execution mechanisms and more stale/race invariants |

During the owner checkpoint, a hybrid was considered. The owner selected the following coherent extension rather than choosing O1/O3 independently.

## Owner-selected O4 contract

### O4 — transactional outbox + queue accelerator + reconciler

`O4` combines the consistency boundary of O1 with an optional queue as an operational accelerator and a reconciler as a recovery safety net.

### Authority hierarchy

```text
PostgreSQL publication state + transactional outbox
        = durable authority

queue
        = optional/replayable delivery accelerator

reconciler
        = PostgreSQL-driven recovery mechanism

Availability Node
        = ciphertext publication executor/evidence source
```

No queue or Availability Node state may independently establish `P2P_READY`.

### Durable-intent transaction

Before publication may be required, one local PostgreSQL transaction durably establishes:

- the stable logical package/publication identity;
- the current non-ready publication state;
- the K1 package/key lineage reference required for retry continuity;
- one durable outbox intent identifying the logical publication work.

The transaction does **not** include the external Availability Node side effect.

### Dispatch semantics

- A dispatcher claims durable outbox work.
- It may call a publication worker directly or enqueue the same logical publication onto the repository's queue/job mechanism when that improves scaling/backpressure.
- Queue enqueue/ack is not publication success.
- Queue loss does not lose the obligation because the durable outbox/publication state remains authoritative.
- Queue duplicates are expected and converge through idempotent same-identity processing.

The architecture contract permits the queue to be bypassed for an initially small P2 implementation; O4 does not require building a generic event bus or new queue technology.

### Idempotency boundary

A stable logical publication identity exists before the first external side effect and is reused by:

- outbox retry;
- queue duplicate/redelivery;
- reconciler re-drive;
- Availability Node publish/status reconciliation.

For retries of one lineage, the encrypted package identity, manifest/hash evidence, Hyperdrive/publication identity where applicable, and K1 server-wrapped CK lineage remain stable.

The Availability boundary must provide one of these equivalent semantics:

1. idempotent publish by stable logical identity, returning the existing result when already published; or
2. deterministic status/reconciliation sufficient to prove the existing result before deciding whether a safe same-identity re-drive is required.

A future explicit package replacement is a new lineage and requires a separately approved transition; retry is not replacement.

### Reconciler responsibility

The reconciler is driven by authoritative PostgreSQL state, not queue visibility. It must be able to discover at least:

- durable outbox work never dispatched;
- work dispatched/enqueued but never completed;
- abandoned/stale worker claims according to later implementation rules;
- authoritative non-ready publication after queue success/ack;
- unknown external outcome after timeout/disconnect/crash;
- remote success whose acknowledgement was lost before local Ready commit.

D3 does not choose reconciliation cadence, lease durations, retry counts, schema names, or queue technology.

## Crash-window closure

| Window | O4 required behavior |
|---|---|
| `W0` before durable transaction commits | No publication obligation exists. |
| `W1` transaction committed, no dispatch | Outbox remains durable; dispatcher/reconciler re-drives it. |
| `W2` crash during external operation | Remains non-ready/unknown; same logical identity is reconciled/retried. |
| `W3` remote success, ACK lost | No new package/CK; same-identity query/retry converges on existing publication. |
| `W4` ACK received, crash before local Ready commit | PostgreSQL remains non-ready; reconciliation reconfirms same publication then writes Ready. |
| `W5` Ready already committed, duplicate arrives | Duplicate is a no-op/idempotent confirmation; it cannot rotate lineage or regress state. |

## Fail-closed `P2P_READY`

`P2P_READY` is true only after the authoritative PostgreSQL record durably proves all of these semantic facts for the current lineage:

1. the logical package/publication lineage is current and not abandoned/replaced;
2. K1 encrypted package construction completed for that lineage;
3. the external publication result for that same lineage is confirmed, not merely dispatched/enqueued;
4. persisted publication identifiers/evidence correspond to that lineage;
5. no unresolved unknown-outcome condition remains for the current lineage.

Any missing, stale, conflicting, or unknown fact means **not P2P-ready**. D2 device-envelope release therefore fails closed.

## Responsibility split

### PostgreSQL / control plane

Owns authoritative publication identity/state, durable outbox intent, K1 lineage references, durable success transition and the semantic `P2P_READY` predicate.

### Outbox dispatcher / worker

Claims durable work, optionally uses the queue, builds/reuses the same encrypted package lineage, invokes/reconciles Availability Node publication and persists confirmed results through control-plane repositories.

### Queue

Optional acceleration only: scheduling, fan-out/backpressure and worker distribution. It is replaceable and non-authoritative. Its ACK never means P2P publication succeeded.

### Reconciler

Scans authoritative durable state/outbox to repair lost/stuck/unknown work and converges using the same logical publication identity. It is a safety mechanism, not a competing source of truth.

### Availability Node

Publishes/seeds ciphertext and returns deterministic result/evidence needed for idempotent confirmation. It does not own business `P2P_READY` state or control-plane secrets.

## Alternatives not selected

- `O1` pure transactional outbox remains the conceptual consistency core, but O4 explicitly allows the queue as a replaceable accelerator and requires the reconciler as a durable recovery safety net.
- `O2` was not selected because combining product state with work leasing/retry mechanics would tighten coupling as the system evolves.
- `O3` was not selected because making the queue primary would increase dual-mechanism correctness burden. O4 keeps queue usage optional and subordinate to PostgreSQL/outbox authority.

## Integrated Reflection — parent RRI 60

### Pass 1 — transactional/crash-window correctness

**PASS.** Durable business/publication state and durable intent are established in one PostgreSQL transaction before an external obligation exists. Every post-commit crash window retains authoritative recoverable work. O4 makes no cross-system atomicity or exactly-once claim.

### Pass 2 — idempotency / unknown-outcome recovery

**PASS.** Outbox retry, queue duplicate and reconciler re-drive all use one stable logical publication identity. Unknown outcome never transitions readiness and remote-success/ACK-loss is resolved through deterministic same-lineage reconciliation.

### Pass 3 — S-120 independence / D2 interaction

**PASS.** S-120 `PreparationStatus::Ready` and transcription enqueue remain independent. P2P has a separate durable readiness predicate, and K1 device-envelope release consumes that predicate fail-closed.

### Pass 4 — scope / status

**PASS.** O4 selects semantic authority, recovery and delivery roles only. It does not select SQL/table/field names, route/RPC encodings, queue product/technology, retry constants, Availability Node credentials/deployment, certification profile, or complete audit-event inventory. ADR-044 stays `Proposed`; D4 and P2 remain separately gated.

## Verification / environment

- Cloud local-model/device/emulator precheck: `n/a`; no local evidence simulated.
- Phase-1 / phase-2 review: `n/a` under the docs/ADR/task-ledger exemption.
- Canonical checks are expected through repository CI after the final synchronization commit.
- `AGENTS.md` / `AGENTS.override.md` parity is a required final check.

## Owner checkpoint history

- 2026-09-05: D3 parent presented at RRI 60.
- 2026-09-05: owner replied `aprobado`; S1-S4 executed and stopped at owner selection.
- 2026-09-05: after comparing O1/O2/O3 and discussing the long-term hybrid, owner explicitly replied `apruebo o4. documentalo`.

That response is the authoritative `ADR044-D3-OWNER` selection.
