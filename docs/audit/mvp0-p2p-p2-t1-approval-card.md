---
type: Audit
title: "Compact Approval Task Card v2 — P2.T1"
status: ready_for_approval
slice: MVP0-P2P
parent: P2.T1
---

# Compact Approval Task Card v2 — P2.T1

## 1. Decision header

**P2.T1 — durable publication identity + transactional outbox persistence**  
Status: **READY FOR OWNER REVIEW**  
Planning RRI: **78 High / Effort XL**  
Dependency: **P2.T0 PASS (`AN-R1 + AN-A1`)**  
Source implementation: **NOT AUTHORIZED**

The exact repository `scripts/rri.py` score must be captured against the final frozen file list immediately before source execution. The planning score is intentionally not presented as a fabricated script run.

## 2. Scope

T1 implements only the durable PostgreSQL/domain foundation required by ADR-044/O4:

- stable logical P2P publication identity and K1 lineage reference;
- semantic publication state compatible with T0's accepted state model;
- durable transactional outbox obligation;
- atomic create/ensure publication + outbox transaction;
- repository reads/transitions needed to expose pending/reconciling work to later T4;
- persistence invariants preventing duplicate active logical publication and invalid direct `ready` state.

Explicitly excluded from T1:

- AES-GCM content encryption or KEK wrapping;
- Availability Node implementation or mTLS networking;
- queue dispatch/reconciler side effects;
- S-120 trigger integration;
- P3 invite/audience/device-envelope behavior;
- final ADR-018 audit implementation/certification.

## 3. Expected path envelope

Freeze before execution to a bounded set equivalent to:

- new `crates/domain/src/p2p_publication.rs`;
- `crates/domain/src/lib.rs` export only;
- new `crates/db/src/p2p_publication_repo.rs`;
- `crates/db/src/lib.rs` export only;
- one new `infra/migrations/<next>_create_p2p_publications_and_outbox.sql` migration;
- focused domain tests and PostgreSQL repository integration tests colocated with or dedicated to those modules;
- this T1 audit/task/status documentation.

No `apps/**`, `mobile/**`, `crates/storage/**`, `crates/jobs/**`, Availability Node, or crypto source belongs in this parent.

## 4. Behavioral acceptance

### Happy paths

- **HP-T1-1:** one database transaction creates one stable non-ready publication identity plus one durable outbox obligation.
- **HP-T1-2:** after process restart/re-read, the same logical identity/lineage and outstanding obligation are observable without relying on queue state or an external side effect.

### Edge cases

- **EC-T1-1:** duplicate create for the same logical lineage is idempotent or returns an explicit conflict; it cannot create a second active logical package.
- **EC-T1-2:** repository API and database invariants cannot produce `ready` without persisted same-lineage external-publication confirmation.
- **EC-T1-3:** unknown/stale in-flight work remains representable as non-ready/reconcilable for later T4 recovery.
- **EC-T1-4:** T1 persistence contains no plaintext CK/KEK, raw invite token, mTLS private key, application JWT signing material, or Availability Node business state.

## 5. Implementation invariants

1. PostgreSQL remains authoritative for publication identity/state.
2. Publication row + initial outbox intent are created atomically in one transaction.
3. Outbox state is work-delivery state, not product readiness.
4. `ready` cannot be inferred from enqueue, claim, dispatch, or Availability Node reachability.
5. Retrying/re-reading T1 state never creates a new package/K1 lineage implicitly.
6. Unknown outcome remains non-ready and can later enter/continue `reconciling`.
7. T1 does not add exactly-once distributed claims.
8. S-120 `PreparationStatus::Ready` and ADR-032 remain untouched.

## 6. Verification / Reflection

Required evidence after implementation:

- focused domain unit tests for state/identity invariants;
- PostgreSQL migration/repository integration tests for atomic publication+outbox creation;
- duplicate/idempotency and invalid-ready negative tests;
- restart/re-read proof using persisted state;
- secret-field/schema inspection against the T0 deny-list;
- normal repository quality gates and behavior-v2 mapping.

Required integrated Reflection passes for the High-band parent:

1. transactional atomicity + crash-before/after-commit semantics;
2. state-machine/readiness invariants + O4 authority hierarchy;
3. lineage/idempotency + secret-data boundary;
4. scope discipline: no T2/T3/T4/T5 implementation creep.

## 7. Owner checkpoint

Approval authorizes preparation/execution of the **bounded T1 source parent only after** the exact path list and actual repository RRI result are recorded. If the exact RRI or path set expands materially, execution stops and the gate is re-presented.

Recommended disposition: **approve T1 scope**, subject to the exact-path RRI pre-execution check above.

Execution has not started.
