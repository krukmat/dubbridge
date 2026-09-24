---
type: Plan
title: "P6: Minimal My Content and Invites dashboard"
status: in_progress
slice: MVP0-P2P
---

# P6 — Minimal My Content and Invites dashboard

Task ledger: `docs/tasks/mvp0-p2p-p6-dashboard.md`. **Activation gate satisfied 2026-09-22. P6.T0 PASS. P6.T1 PASS 2026-09-23:** T1.A–H are closed; the standing MVP0-P2P owner-directed review exception replaces phase-1/phase-2 peer review, and owner Matias Kruk explicitly reviewed and approved T1. Functional/evidence head `b6df0a7f` completed 15/15 CI, mobile 63/63 suites / 461/461 tests and 90.43% workspace line coverage. **P6.T2 PASS 2026-09-24:** T2.A–H are closed with owner final verification and the standing MVP0-P2P REVIEW-OVERRIDE. **P6.T3 is unblocked / not activated.** P5.T3 is deferred to release certification.

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
| P6.T1 | Owner My Content and invite action | development | M | T0 PASS | **PASS 2026-09-23 — A–H closed; RRI 55 Med-high; owner-reviewed + REVIEW-OVERRIDE** |
| P6.T2 | Viewer claim, Invites, sync and play actions | development | XL parent / decomposed | T1 PASS | **PASS 2026-09-24 — A–H closed; owner-verified** |
| P6.T3 | Dashboard flow and visual certification | development/evidence | M | T2 PASS | **Unblocked / not activated** |


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

P6.T1 is closed PASS 2026-09-23. Blocks 1–3 cover T1.A–G and T1.H closes aggregate certification under the standing MVP0-P2P review exception plus owner self-review.


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

P6.T1 is PASS 2026-09-23. T1.A–H are closed; P6.T2 is unblocked / not activated.


### P6.T1 T1.H certification candidate — 2026-09-23

Evidence: `docs/audit/mvp0-p2p-p6-t1-certification-candidate-2026-09-23.md`.

The aggregate owner slice has been re-read against the frozen T1 contract and the
parent P6 owner-side acceptance. Three Med-high Reflection passes are recorded,
the behavior-v2 mapping is complete, and exact candidate head `d19e51f4`
already completed 15/15 CI with mobile 63/63 suites / 461/461 tests and 90.43%
workspace line coverage.

T1.H is **PASS 2026-09-23**. The standing owner-directed MVP0-P2P review exception
waives the phase-2 peer-review artifact for P6.T1, and the owner explicitly confirmed
that he reviewed T1 himself and approved closure. T2 is now unblocked but not activated.


### P6.T2 activation — 2026-09-23

Evidence: `docs/audit/mvp0-p2p-p6-t2-activation-2026-09-23.md`.

T2.A verified T1 PASS and the existing P3/P4/P5 seams before source work:

- backend-authoritative viewer facts: `P2PDashboardService.listInbox()`;
- Claim: `P2PAudienceService.claimInvitation()`;
- verified local state: `P2PSyncController.startSync/getSyncState/getVerifiedPackageHandle()`;
- playback: `P2PPlaybackController.start()` + `P2PPlaybackSessionView`;
- account scope: existing `AuthProvider.userId`, already consumed by `P2PProvider`.

The coherent parent scores **RRI 100 / Very high**, driven by combining authorization precedence, async P4 sync state, P5 playback and account/session lifecycle. It must not execute as one patch. The decomposition is frozen as T2.B–T2.H; B–G each score 55 / Med-high because each is limited to UI-level projection/orchestration over an existing capability, while T2.H is 25 / Low evidence/closure work.

No P3/P4/P5 capability implementation is reopened. HPKE emulator remains disabled.


### P6.T2 B/C result — 2026-09-24

Evidence: `docs/audit/mvp0-p2p-p6-t2-bc-inbox-claim-2026-09-24.md`.

T2.B is PASS on `19a5bca9` / run `36019060607` (15/15 CI; mobile 64/64 suites, 481/481 tests). It adds the authoritative viewer inbox projection over existing P3/P4 facts without reopening those capability owners.

T2.C is PASS on `c104d40a` / run `36022359070` (15/15 CI; mobile 64/64 suites, 489/489 tests). Manual Claim keeps the raw token in volatile React state, delegates to existing P3 audience capability, refreshes the authoritative inbox after success, and does not infer Available/Play.

P6.T2 remains IN PROGRESS. **T2.D PASS** on `a99a712c` / run `36025590927` (15/15 CI; mobile 64/64 suites, 495/495 tests). **T2.E PASS** on `8fa7411d` / run `36029185810` (15/15 CI; mobile 64/64 suites, 501/501 tests): Available alone exposes Play, P4 must first produce a verified package handle, and P5 revalidates authorization before returning the loopback playback session consumed by the existing player view. **T2.F PASS** on `9c4b41e4` / run `36031993001` (15/15 CI; mobile 64/64 suites, 507/507 tests): session/account identity changes invalidate stale inbox/playback/action state and late completions from the prior identity are discarded. **T2.G PASS** on `6f4a89e1` / run `36033387423` (15/15 CI; mobile 64/64 suites, 509/509 tests): Home→Invites→Back uses the existing authenticated stack, re-entry remounts and refetches `/api/p2p/inbox`, and auth loss removes the route. **T2.H PASS / P6.T2 PASS 2026-09-24.** HP/EC mapping, aggregate Reflection and exact leaf evidence are recorded in `docs/audit/mvp0-p2p-p6-t2-h-certification-candidate-2026-09-24.md`. Owner final verification was supplied by the explicit directive `cierra P6.T2`; candidate head `085148dc` completed 15/15 CI before closure. The standing MVP0-P2P REVIEW-OVERRIDE applies only to phase-1/phase-2 peer review. P6.T3 is now unblocked / not activated. HPKE emulator remains disabled.
