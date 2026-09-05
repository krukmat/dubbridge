---
type: Audit
title: "P2.T1 decomposition into lower-RRI leaves"
status: ready_for_approval
slice: MVP0-P2P
parent: P2.T1
---

# P2.T1 — lower-RRI decomposition

## Purpose

The original `P2.T1` bundled domain modeling, SQL schema, transactional repository behavior, recovery-facing queries, guarded state transitions, and persistence certification into one **RRI 78 High / XL** implementation parent.

That parent is now **non-executable**. It is decomposed into six sequential leaves so that each source change stays within one narrow technical seam and no leaf crosses domain + migration + repository + certification simultaneously.

These are conservative **planning** scores based on the current RRI rubric. They are not represented as `scripts/rri.py` execution evidence. Each leaf must be re-scored with the exact frozen path set immediately before source execution; the actual/current result governs.

## New task map

| ID | Objective | Planning RRI | Effort | Depends on |
|---|---|---:|---|---|
| `P2.T1a` | Pure domain identity/state contract + unit tests | **22 Low** | S | T0 PASS |
| `P2.T1b` | PostgreSQL schema + constraints only | **32 Medium** | S/M | T1a PASS |
| `P2.T1c` | Atomic `create/ensure publication + outbox` repository write | **47 Medium-high** | M | T1b PASS |
| `P2.T1d` | Outstanding-work/read model repository queries only | **36 Medium** | S/M | T1c PASS |
| `P2.T1e` | Guarded persistence transitions + same-lineage confirmation fields | **44 Medium-high** | M | T1d PASS |
| `P2.T1f` | Persistence integration certification + restart/idempotency negatives | **33 Medium** | M | T1e PASS |

**Maximum planning RRI after decomposition: 47.** No leaf is High-band by planning score.

## Boundaries by leaf

### P2.T1a — domain contract

Only pure Rust domain semantics:

- stable logical publication identity and K1 lineage reference types;
- T0-compatible semantic publication states;
- pure transition/readiness guards;
- unit tests for illegal direct `ready`, lineage stability, and unknown/reconciling semantics.

No DB crate, SQL, migrations, apps, jobs, crypto, Availability Node, or integration wiring.

### P2.T1b — schema

Only the next PostgreSQL migration:

- publication row/table structure;
- outbox row/table structure;
- stable identity / lineage columns;
- non-secret confirmation metadata fields required later by T1e;
- uniqueness/check/FK constraints that can be expressed safely at schema level.

No repository code or worker behavior.

### P2.T1c — atomic create/ensure write

Only the minimal DB repository write path that:

- creates or idempotently ensures the logical publication;
- creates the initial outbox obligation in the **same PostgreSQL transaction**;
- returns the stable logical identity;
- cannot silently create a second active lineage.

No outstanding-work scans, state-transition API, queue, dispatch, or external publication.

### P2.T1d — read model / outstanding work

Only repository reads required by later O4 execution:

- read publication by stable identity;
- list/read outstanding durable obligations using authoritative PostgreSQL state;
- expose enough non-secret state for a later T4 worker/reconciler to decide what needs work.

No claim/lease side effects and no state transitions.

### P2.T1e — guarded transitions

Only durable persistence transitions:

- non-ready lifecycle transitions defined by T0;
- persist same-lineage external confirmation evidence fields;
- guard `ready` so it cannot be written without required same-lineage durable confirmation;
- keep unknown external outcome representable as non-ready/reconciling.

No network calls or Availability Node implementation.

### P2.T1f — certification

Only integration/negative evidence over the completed T1 persistence seam:

- restart/re-read of same identity and outstanding intent;
- duplicate create/idempotency behavior;
- invalid direct-ready rejection;
- atomicity around publication + outbox creation;
- schema/serialization inspection against the T0 secret deny-list;
- behavior-v2 mapping for T1 acceptance.

If T1f discovers a production defect, it reopens the responsible leaf instead of expanding T1f into unbounded corrective implementation.

## Why this is safer

1. **Lower blast radius:** each leaf changes one layer or one repository responsibility.
2. **Clear rollback/review:** schema, writes, reads, transitions, and certification are independently inspectable.
3. **No artificial Low-band crypto/security downgrade:** K1 remains in T2; this decomposition only reduces persistence complexity.
4. **Preserves O4 atomicity:** splitting implementation does not split the runtime transaction; T1c still creates publication state + outbox atomically.
5. **Preserves fail-closed readiness:** T1a defines the semantic rule, T1b provides schema guards, and T1e owns the only repository transition to durable `ready`.
6. **No hidden T4 creep:** dispatch, queue acceleration, leases, retries, and reconciler side effects remain outside T1.

## Gate consequence

The old `P2.T1` RRI-78 approval card is superseded as an executable gate. The next owner gate becomes **`P2.T1a` only**. Completion of T1a does not authorize T1b automatically; each leaf is independently re-scored/presented before source execution.
