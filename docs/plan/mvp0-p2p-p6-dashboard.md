---
type: Plan
title: "P6: Minimal My Content and Invites dashboard"
status: planned
slice: MVP0-P2P
---

# P6 — Minimal My Content and Invites dashboard

Task ledger: `docs/tasks/mvp0-p2p-p6-dashboard.md`. Documentation prepared 2026-09-08;
implementation remains blocked on **P3-P5 PASS** and the per-task workflow gate.

## Objective

Deliver G6/CU-03 with the minimum owner/viewer state and actions required for the October demonstration.

## Scope and accepted decisions

MY CONTENT: Processing/Ready/Failed; INVITES: Pending/Syncing/Available/Expired; owner invite, viewer claim/sync/play. No analytics, payments, Studio Web, community, multi-device management or design-system replacement.

No change to accepted O3/K1/O4, ADR-032 review delivery, or C0 contracts.
The parent P6 HP/EC set in `docs/tasks/mvp0-p2p-first.md` remains required;
leaf examples below supplement it rather than narrowing acceptance.

## Affected areas and module dependencies

P3 owner/inbox authorization, P4 verified sync state, P5 session/play capability; existing mobile navigation/auth providers and design primitives. Candidate areas: current mobile screens/navigation, `mobile/src/p2p`, API clients and mobile tests. Read `DESIGN.md` and shipped theme tokens at activation.

These are candidate areas, not an executable writable-path grant. The phase
activation task must inspect the then-current source, reserve exact paths and
freeze interfaces before any development leaf is presented or delegated.

## Phase-specific decisions to freeze

Freeze state projection and action eligibility from canonical backend and runtime facts, routes/deep-link or manual claim entry, retry/error presentation and account-reset behavior. UI visibility never replaces backend authorization; Available/can_play requires verified data and valid access. Reuse shipped dark theme and existing components.

## Planned work and verification

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P6.T0 | State/action and navigation contract | planning | M | P3-P5 PASS | Planned; not activated |
| P6.T1 | Owner My Content and invite action | development | M | T0 PASS | Planned; not activated |
| P6.T2 | Viewer claim, Invites, sync and play actions | development | L | T1 PASS | Planned; not activated |
| P6.T3 | Dashboard flow and visual certification | development/evidence | M | T2 PASS | Planned; not activated |


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
- `DESIGN.md` — existing mobile visual intent; shipped theme tokens resolve visual drift.
