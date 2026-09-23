---
type: Plan
title: "P6: Minimal My Content and Invites dashboard"
status: in_progress
slice: MVP0-P2P
---

# P6 — Minimal My Content and Invites dashboard

Task ledger: `docs/tasks/mvp0-p2p-p6-dashboard.md`. **Activation gate satisfied 2026-09-22. P6.T0 PASS:** owner verification complete and `c6ce2039` finished 15/15 CI. **P6.T1 T1.A–G PASS**: Block 1 closed on `b52d366c`; Block 2 Create + one-time Copy on `151721a5`; T1.F navigation on `32ac26c0`; T1.G aggregate gap closure on `47bc3e8b` with 15/15 CI, mobile 63/63 suites / 461/461 tests and 90.43% workspace line coverage. T1.H aggregate certification is now prepared on `d19e51f4`; owner verification is approved 2026-09-23; the independent Med-high code-solution review remains before PASS. T2/T3 remain blocked. P5.T3 is deferred to release certification.

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
| P6.T0 | State/action and navigation contract | planning | M | P3 PASS; P4 PASS; P5-DEV | **PASS 2026-09-22** |
| P6.T1 | Owner My Content and invite action | development | M | T0 PASS | **In progress — A–G PASS; H owner-approved, peer review pending; RRI 55 Med-high** |
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

P3 and P4 must be PASS, P5-DEV must be closed, and P6 must close before S-230
T6p-a can activate. P5.T3 is intentionally carried to the release-certification
lane and is not part of this development gate.

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


### P6.T0 contract freeze — 2026-09-22

Evidence: `docs/audit/mvp0-p2p-p6-t0-state-navigation-contract-2026-09-22.md`. Owner/content state comes only from `/api/p2p/content`; viewer access from `/api/p2p/inbox`; local availability from verified P4 sync state. Invite requires authoritative P2P Ready + descriptor. Play requires active authorization + exact descriptor + verified READY package and remains subject to P5 O3 revalidation. Navigation is frozen to Home → My content / Invites with existing P3/P4/P5 services retaining capability ownership.


### P6.T1 Block 1 result — 2026-09-22

Evidence: `docs/audit/mvp0-p2p-p6-t1-block1-my-content-2026-09-22.md`.

`MyContentScreen` now projects only backend-authoritative P2P owner states and
uses exact descriptor identity for Invite presentation. The first implementation
was refactored after the maintainability gate rejected declaration concentration;
the final model/state-hook/view split is green.

Head `b52d366c`: 15/15 CI PASS; mobile 63/63 suites, 452/452 tests; workspace
line coverage 90.43%.

P6.T1 is not closed. Blocks 1–3 now cover T1.A–G; only T1.H aggregate
certification and owner verification remains before T1 PASS.


### P6.T1 Block 2 result — 2026-09-22

Evidence: `docs/audit/mvp0-p2p-p6-t1-block2-create-copy-invite-2026-09-22.md`.

`MyContentScreen` now routes the Ready owner action through the existing P3
`POST /api/assets/{id}/p2p/invitations` contract via `P2PAudienceService`.
Eligibility remains exact-descriptor and server-authoritative; a 403/404/409
refreshes owner content rather than trusting stale UI state.

The raw invitation token remains only in volatile React state, permits one
outstanding visible token at a time, copies through `expo-clipboard`, clears on
Done/unmount, and is never added to persistence or logs.

Implementation head `151721a5`: 15/15 CI PASS; mobile 63/63 suites,
456/456 tests; workspace line coverage 90.43%.

### P6.T1 Block 3 result — 2026-09-22

Evidence: `docs/audit/mvp0-p2p-p6-t1-block3-navigation-evidence-2026-09-22.md`.

T1.F wires the existing authenticated stack as Home → My Content → Back, reuses
the existing gateway/auth providers, refreshes the owner projection on remount and
does not revive the one-time raw token after leaving the screen.

T1.G performed the remaining component/integration gap review. The only uncovered
contract edges were stale Create rejection across the full 403/404/409 set and
the single-visible-token lock across multiple Ready items. Both are now explicit
tests; production behavior was unchanged. Existing DB integration evidence also
proves `/api/p2p/content` is owner-scoped and excludes other-owner assets.

Exact evidence head `47bc3e8b`: 15/15 CI PASS; mobile 63/63 suites,
461/461 tests; P3 T2c3 certification harness PASS; workspace line coverage 90.43%.

P6.T1 remains IN PROGRESS. T1.A–G are PASS; only T1.H aggregate certification
and owner verification remains pending.


### P6.T1 T1.H certification candidate — 2026-09-23

Evidence: `docs/audit/mvp0-p2p-p6-t1-certification-candidate-2026-09-23.md`.

The aggregate owner slice has been re-read against the frozen T1 contract and the
parent P6 owner-side acceptance. Three Med-high Reflection passes are recorded,
the behavior-v2 mapping is complete, and exact candidate head `d19e51f4`
already completed 15/15 CI with mobile 63/63 suites / 461/461 tests and 90.43%
workspace line coverage.

T1.H is **not PASS yet**. Owner final verification was explicitly approved on
2026-09-23. Repository policy still requires the independent RRI-55 code-solution
reviewer. T2 remains blocked until that reviewer evidence is recorded.
