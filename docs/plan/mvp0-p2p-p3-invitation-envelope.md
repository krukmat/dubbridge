---
type: Plan
title: "P3: Invitation, audience authorization, and K1 device envelope"
status: planned
slice: MVP0-P2P
---

# P3 — Invitation, audience authorization, and K1 device envelope

Task ledger: `docs/tasks/mvp0-p2p-p3-invitation-envelope.md`. Documentation prepared 2026-09-08;
implementation remains blocked on **P2 PASS; Accepted ADR-044** and the per-task workflow gate.

## Objective

Deliver CU-02 and the claim half of CU-04 through backend-owned O3 authorization and the accepted K1 device envelope.

## Scope and accepted decisions

One owner, one invited viewer, one active device. No email/bulk invites, public revoke endpoint, multi-device, trusted-time, or replacement of ADR-032.

No change to accepted O3/K1/O4, ADR-032 review delivery, or C0 contracts.
The parent P3 HP/EC set in `docs/tasks/mvp0-p2p-first.md` remains required;
leaf examples below supplement it rather than narrowing acceptance.

## Affected areas and module dependencies

Existing API authentication/ownership, PostgreSQL, P2 ready descriptor and server-wrapped CK; Android native Keystore adapter. Candidate areas: `apps/api`, `crates/domain`, `crates/db`, `crates/p2p`, `mobile` native adapter and tests. Reserve migration numbers against the current ledger at activation.

These are candidate areas, not an executable writable-path grant. The phase
activation task must inspect the then-current source, reserve exact paths and
freeze interfaces before any development leaf is presented or delegated.

## Phase-specific decisions to freeze

Freeze concrete API/schema names, token-expiry policy, O3 authorization lifecycle, audience/device audit events, envelope binding serialization, and native adapter boundary. Preserve all D2 predicates. The Keystore capability proof must precede integrated envelope closure; inability to use an opaque P-256 private key requires STOP/reopening D2, never a K2 fallback.

## Planned work and verification

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P3.T0 | Contract and executable-path freeze | planning | M | P2 PASS | Planned; not activated |
| P3.T1 | Invitation persistence, claim, and inbox | development | L | T0 PASS | Planned; not activated |
| P3.T2 | O3 authorization and native K1 envelope delivery | development | L | T1 PASS | Planned; not activated |
| P3.T3 | P3 integration certification and closure | development/evidence | M | T2 PASS | Planned; not activated |


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
- `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md` — exact K1 predicates and opaque native-key proof.
- `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md` — ready descriptor and sealed-lineage input.
