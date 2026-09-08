---
type: Plan
title: "P4: Verified mobile ciphertext synchronization"
status: planned
slice: MVP0-P2P
---

# P4 — Verified mobile ciphertext synchronization

Task ledger: `docs/tasks/mvp0-p2p-p4-mobile-sync.md`. Documentation prepared 2026-09-08;
implementation remains blocked on **P3 PASS** and the per-task workflow gate.

## Objective

Deliver G4 and the sync half of CU-04 using the ADR-043 product service/runtime boundary, full-package replication, and manifest verification.

## Scope and accepted decisions

Foreground full-package sync and resume, ciphertext cache, explicit product lifecycle. No playback, decryption, dashboard, background/offline certification, or progressive streaming.

No change to accepted O3/K1/O4, ADR-032 review delivery, or C0 contracts.
The parent P4 HP/EC set in `docs/tasks/mvp0-p2p-first.md` remains required;
leaf examples below supplement it rather than narrowing acceptance.

## Affected areas and module dependencies

P3 authorized descriptor, P2 manifest contract, existing P2PProvider/P2PService/BareRuntimeClient and product worklet. Candidate areas: `mobile/src/p2p`, mobile P2P tests, and existing mobile native lifecycle seams.

These are candidate areas, not an executable writable-path grant. The phase
activation task must inspect the then-current source, reserve exact paths and
freeze interfaces before any development leaf is presented or delegated.

## Phase-specific decisions to freeze

Freeze cache location/quota/cleanup, account and device isolation, sign-out wipe, cancellation/resume behavior, descriptor expiry handling, and versioned RPC mapping. Protocol names in design inputs are hypotheses. Cache possession never establishes playback authorization.

## Planned work and verification

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P4.T0 | Lifecycle, cache, and RPC freeze | planning | M | P3 PASS | Planned; not activated |
| P4.T1 | Product replication and bounded resume | development | L | T0 PASS | Planned; not activated |
| P4.T2 | Manifest verification and lifecycle isolation | development | L | T1 PASS | Planned; not activated |
| P4.T3 | P4 certification and P5 handoff | development/evidence | M | T2 PASS | Planned; not activated |


The companion ledger defines acceptance, HP/EC, evidence and handoff per task.
Development rows are planning work packages, not pre-scored executable leaves.
At activation, score the coherent parent with `scripts/rri.py`, then perform
honest Low-band maximization using independently verifiable seams; retain the
parent approval/review envelope. Never assume a Low score for crypto, native
integration, authorization or distributed state based on this docs-only edit.

## Closure and downstream gate

All required parent/leaf HP/EC must map to passing executable evidence at the
appropriate layer. Apply the current band-routed review (or recorded applicable
exception), Reflection, behavioral certification and owner-verification gates.
Synchronize this plan, its ledger, the parent plan/ledger and roadmap. No phase
PASS is implied by plan availability or provisional effort.

P3-P6 must all close before S-230 T6p-a can activate.

## Calendar and estimation limits

Target: development PASS by 2026-10-15.
Effort labels are provisional work-package sizing, not elapsed-day estimates.
Activation must record owner, actual leaf scope, estimates and contingency;
this document does not establish capacity or guarantee the October date.

## References

- `docs/tasks/mvp0-p2p-first.md` — parent acceptance examples and phase dependencies.
- `docs/plan/mvp0-p2p-first.md` — release profile and calendar targets.
- `docs/plan/mvp0-p2p-design-inputs.md` — product inputs; suggested APIs/RPC are non-binding.
- `docs/adr/ADR-044-p2p-audience-delivery-boundary.md` — O3/K1/O4 authority and secrets.
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md` — product runtime ownership.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and `docs/policies/HITL_AUTONOMY_POLICY.md` — activation and closure.
