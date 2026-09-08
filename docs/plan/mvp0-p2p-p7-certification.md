---
type: Plan
title: "P7: Exact-artifact end-to-end P2P certification"
status: planned
slice: MVP0-P2P
---

# P7 — Exact-artifact end-to-end P2P certification

Task ledger: `docs/tasks/mvp0-p2p-p7-certification.md`. Documentation prepared 2026-09-08;
implementation remains blocked on **P2-P6 PASS; S-230-T7p PASS** and the per-task workflow gate.

## Objective

Deliver G7 certification of the deployed owner-to-invited-viewer Android flow, with legacy HTTP/S3 audience-media fallback disabled.

## Scope and accepted decisions

One owner, one viewer/device, one short video; exact-artifact functional certification. No performance/offline/multi-device certification, GA claim or waiver of security/CI/release gates.

No change to accepted O3/K1/O4, ADR-032 review delivery, or C0 contracts.
The parent P7 HP/EC set in `docs/tasks/mvp0-p2p-first.md` remains required;
leaf examples below supplement it rather than narrowing acceptance.

## Affected areas and module dependencies

Exact S-230 T6p-d deployment revision/images, T7p Android RC/device proof, P2-P6 evidence and X29 disposition. Candidate areas: certification scripts/config and end-to-end tests selected at activation, plus audit artifacts. P7 consumes release artifacts; it does not independently authorize deployment.

These are candidate areas, not an executable writable-path grant. The phase
activation task must inspect the then-current source, reserve exact paths and
freeze interfaces before any development leaf is presented or delegated.

## Phase-specific decisions to freeze

Freeze the certification profile and evidence capture method: disable remote audience-media delivery while retaining auth/assets/invites/audit APIs and local loopback HLS. Record negative controls proving HTTP/S3 fallback cannot succeed. If configuration/profile changes alter the RC or deployed artifact, rebuild and repeat affected T7p/T6p evidence before certification; never certify a different artifact.

## Planned work and verification

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P7.T0 | Certification profile and artifact manifest | planning/config | M | P2-P6 PASS; S-230-T7p PASS | Planned; not activated |
| P7.T1 | No-fallback controls and control-plane checks | operational/evidence | M | T0 PASS | Planned; not activated |
| P7.T2 | Physical owner-to-viewer end-to-end proof | operational/evidence | M | T1 PASS | Planned; not activated |
| P7.T3 | Certification verdict and T9g handoff | operational/evidence | S | T2 evidence complete (PASS or failure) | Planned; not activated |


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

P7 certification is required before S-230 T9g; it is not a T6p-a prerequisite.

## Calendar and estimation limits

Target: certification during 2026-10-27 through 2026-10-30 after T7p.
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
- `docs/tasks/s-230-poc-v1-digitalocean.md` — T6p-d/T7p artifacts and T9g release decision.
