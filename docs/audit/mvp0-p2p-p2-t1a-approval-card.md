---
type: Audit
title: "Compact Approval Task Card v2 — P2.T1a"
status: ready_for_approval
slice: MVP0-P2P
parent: P2.T1a
---

# Compact Approval Task Card v2 — P2.T1a

## 1. Decision header

**P2.T1a — pure domain publication identity/state contract**  
Status: **READY FOR OWNER REVIEW**  
Planning RRI: **22 Low / Effort S**  
Dependency: **P2.T0 PASS (`AN-R1 + AN-A1`)**  
Source implementation: **NOT AUTHORIZED**

The exact `scripts/rri.py` score must be captured against the frozen path list before source execution.

## 2. Scope

Implement only pure domain semantics for P2 publication:

- stable logical publication identity type;
- stable K1 lineage reference type;
- T0-compatible semantic states (`building`, `publish_pending`, `publishing`, `reconciling`, `ready`, terminal `failed`);
- pure guards for valid/invalid transitions;
- pure readiness rule preventing direct/inferred `ready`;
- unit tests covering lineage stability and fail-closed unknown outcome.

Explicitly excluded:

- PostgreSQL/schema/migrations;
- DB repositories;
- outbox persistence;
- crypto/CK/KEK implementation;
- workers/queues/reconciler;
- Availability Node/mTLS;
- S-120 integration.

## 3. Expected path envelope

- new `crates/domain/src/p2p_publication.rs`;
- `crates/domain/src/lib.rs` export only;
- focused unit tests colocated with the new module;
- T1a status/audit documentation only.

No other production source path is authorized by this card.

## 4. Behavioral acceptance

- **HP-T1a-1:** one logical publication identity retains one lineage reference across allowed non-terminal state transitions.
- **HP-T1a-2:** unknown external outcome is representable as `reconciling` and remains non-ready.
- **EC-T1a-1:** direct transition from initial/non-confirmed state to `ready` is rejected.
- **EC-T1a-2:** transition out of terminal `failed` or regression from `ready` is rejected unless a future explicitly approved replacement contract says otherwise.
- **EC-T1a-3:** domain values contain no plaintext CK/KEK, invite token, mTLS key, or JWT signing material.

## 5. Verification

- unit tests for every allowed/forbidden transition in this leaf;
- lineage/identity immutability tests;
- `cargo test` for the domain crate / focused module;
- normal repository quality gates;
- one scope Reflection confirming no DB/crypto/network implementation entered the leaf.

## 6. Owner checkpoint

Approval authorizes only `P2.T1a` after the exact path/RRI precheck. It does **not** authorize T1b or any later P2 leaf.

Recommended disposition: **approve T1a**.
