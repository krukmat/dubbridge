---
type: TaskList
title: "Tasks: P6 Minimal My Content and Invites dashboard"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p6-dashboard.md
behavioral_coverage_contract: behavior-v2
---

# P6 — planning task ledger

**Status:** **In progress. P6.T0 PASS 2026-09-22; P6.T1 PASS 2026-09-23.** T1 closed under the standing MVP0-P2P owner review exception with owner self-review/approval and green exact-head CI evidence. **P6.T2 is IN PROGRESS: T2.A/B/C/D PASS; T2.E–H remain pending. P6.T3 remains blocked on T2 PASS.**
**Phase gate:** **SATISFIED 2026-09-22 — P3 PASS + P4 PASS + P5-DEV.** P5.T3/P5-CERT is not an activation prerequisite.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P6.T0 | State/action and navigation contract | planning | M | P3 PASS; P4 PASS; P5-DEV | **PASS 2026-09-22 — owner-verified; c6ce2039 15/15 CI** |
| P6.T1 | Owner My Content and invite action | development | M | T0 PASS | **PASS 2026-09-23 — T1.A–H closed; owner-reviewed; REVIEW-OVERRIDE applied** |
| P6.T2 | Viewer claim, Invites, sync and play actions | development | XL parent / decomposed leaves | T1 PASS | **IN PROGRESS — T2.A/B/C/D PASS; T2.E–H pending** |
| P6.T3 | Dashboard flow and visual certification | development/evidence | M | T2 PASS | Planned; not activated |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P6 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p6-dashboard.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P6.T0 — State/action and navigation contract

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P3 PASS; P4 PASS; P5-DEV. P5.T3 may remain pending.

**Status:** **PASS 2026-09-22.** RRI 25 Low, docs-only planning contract; owner verification complete and exact head `c6ce2039` completed 15/15 CI. Evidence: `docs/audit/mvp0-p2p-p6-t0-state-navigation-contract-2026-09-22.md`.

**Acceptance criteria:** Map each owner/viewer state to authoritative facts, permitted actions and loading/empty/error behavior; freeze navigation and exact component ownership.

- **HP-P6.T0-1:** Owned ready content permits Invite; an authorized verified invitation permits Play.
- **EC-P6.T0-1:** S-120 Ready alone or synced-but-unverified data never enables the corresponding P2P action.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P3 PASS, P4 PASS, and P5-DEV; do not require P5.T3;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P6.T1 — Owner My Content and invite action

**Type:** development

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** **PASS 2026-09-23.** Blocks T1.A–G are green; T1.H aggregate certification is closed using the standing MVP0-P2P owner-directed peer-review exception (`docs/audit/mvp0-p2p-review-exception.md`). Owner Matias Kruk explicitly stated that he reviewed T1 himself and approved closure. Functional/evidence head `b6df0a7f` completed 15/15 CI; mobile 63/63 suites / 461/461 tests; workspace line coverage 90.43%. Create Invite preserves P3 authority, 403/404/409 stale-state rejection is fail-closed, the raw token remains transient, and Home → My Content navigation clears it on unmount. Activation RRI: **55 / Med-high**; no auth decision is delegated to UI.

**Acceptance criteria:** Render owned content and correct P2P publication state; create/copy the one-time invite through P3; use existing design primitives.

- **HP-P6.T1-1:** Owner sees Processing → Ready and can create an invite for their ready package.
- **EC-P6.T1-1:** Other-owner assets or failed/non-ready publications do not expose an invite action; server rejection remains enforced.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.


### Review evidence override

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner
- Scope-note: skips only phase-1 and phase-2 peer review for P6.T1 under `docs/audit/mvp0-p2p-review-exception.md`; tests, RRI, 3 Reflection passes, behavioral coverage, owner verification, and status synchronization remain mandatory.

### Reflection log

Required passes: 3 (RRI 55 → Med-high).

- **Pass 1 — authority:** PASS. Owner state/Invite eligibility remains backend/P3-authoritative; exact descriptor identity and owner scoping remain enforced.
- **Pass 2 — secret/session lifecycle:** PASS. Raw invite token remains volatile, Copy/Done/Back/remount/session-expiry behavior is fail-closed, and the single-visible-token lock is covered.
- **Pass 3 — integrated navigation/evidence:** PASS. Home → My Content re-entry reloads authoritative state; auth loss removes the route; all T1 HP/EC cases map to executable evidence.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-P6.T1-1 | Happy path | Owner sees authoritative state and exact Ready content can create/copy a P3 invite | component + integration | `mobile/__tests__/MyContentScreen.test.tsx`; `mobile/__tests__/RootNavigator.test.tsx` | passed |
| EC-P6.T1-1 | Edge case | Foreign-owner, non-ready, mismatched or stale content cannot retain Invite authority | integration + component | `crates/db/tests/p2p_dashboard_repo.rs::owner_content_is_scoped_to_owned_assets_and_preserves_publication_state`; `mobile/__tests__/MyContentScreen.test.tsx` | passed |

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-09-23
- Statement: I reviewed P6.T1 myself and approve closure against the recorded happy-path/edge-case evidence and green CI result.
- Commands run: manual owner review; no additional CLI command was reported. Automated verification used GitHub Actions run `35825349299` on `b6df0a7f` (15/15 PASS; mobile 63/63 suites, 461/461 tests; coverage 90.43%).

## P6.T2 — Viewer claim, Invites, sync and play actions

**Type:** development

**Effort:** XL parent; executable leaves S/L after decomposition

**Depends on:** T1 PASS

**Status:** **IN PROGRESS — T2.A/B/C/D PASS; T2.E–H pending.** Parent RRI **100 / Very high**; source implementation remains prohibited at parent scope and continues only through the frozen leaves below.

**Acceptance criteria:** Connect claim/inbox, verified sync and existing playback through the frozen state model; handle expiry, loading/retry and account changes.

- **HP-P6.T2-1:** Viewer claims, syncs, sees Available after verification, then plays the package.
- **EC-P6.T2-1:** Expired, inaccessible, incomplete or unverified content offers no Play; another viewer invitation is never displayed.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** T1 PASS is verified and T2.A activation is complete. Execute only one frozen leaf at a time, preserving the P3/P4/P5 capability boundaries below. Do not implement the RRI-100 parent as one patch. Stop on a contract conflict or unmet dependency; do not silently advance T3.

### P6.T2 execution breakdown — activated 2026-09-23

| Block | Scope | RRI | Status |
|---|---|---:|---|
| T2.A | Activation, exact scope, parent/leaf RRI, dependency freeze | docs-only activation | **PASS** |
| T2.B | Authoritative viewer inbox + product-state projection | 55 / Med-high | **PASS 2026-09-24 — `19a5bca9`, 15/15 CI** |
| T2.C | Manual Claim via existing P3 audience capability | 55 / Med-high | **PASS 2026-09-24 — `c104d40a`, 15/15 CI** |
| T2.D | Sync / Retry Sync via existing P4 controller | 55 / Med-high | **PASS 2026-09-24 — `a99a712c`, 15/15 CI** |
| T2.E | Available + Play via verified P4 handle and existing P5 controller/view | 55 / Med-high | Pending |
| T2.F | Fail-closed expiry/session/account lifecycle | 55 / Med-high | Pending |
| T2.G | Home → Invites navigation + remount behavior | 55 / Med-high | Pending |
| T2.H | Aggregate component/integration evidence + closure | 25 / Low | Pending |

Activation evidence:
`docs/audit/mvp0-p2p-p6-t2-activation-2026-09-23.md`.

### T2.B/C implementation evidence

Evidence: `docs/audit/mvp0-p2p-p6-t2-bc-inbox-claim-2026-09-24.md`.

- **T2.B PASS:** authoritative `/api/p2p/inbox` projection joined with P4 local sync facts, including Pending / Syncing / Sync error / Available / Expired precedence, loading/empty/error/retry states and other-viewer fail-closed filtering. Exact implementation head `19a5bca9`, Actions run `36019060607`: **15/15 PASS**; mobile **64/64 suites, 481/481 tests**.
- **T2.C PASS:** manual raw-token Claim delegates only to `P2PAudienceService.claimInvitation()`; blank/double submit is blocked, success rotates session + clears the token + refreshes the authoritative inbox, claim errors remain fail-closed, session expiry logs out, and remount cannot recover the raw token. Exact implementation head `c104d40a`, Actions run `36022359070`: **15/15 PASS**; mobile **64/64 suites, 489/489 tests**.
- Owner approved execution of P6.T2.C before implementation. Aggregate T2 owner verification remains a T2.H closure obligation.
- Claim success does **not** directly produce Available or Play; T2.D/E retain the P4/P5 gates.
- HPKE emulator remains disabled.

### T2.D implementation evidence

Evidence: `docs/audit/mvp0-p2p-p6-t2-d-sync-retry-2026-09-24.md`.

- **T2.D PASS:** Pending + exact descriptor exposes Sync; P4 FAILED exposes Retry Sync; inactive/expired or descriptor-mismatched items expose no Sync.
- Actions delegate only to `P2PSyncController.startSync(descriptor, accountScope)` with `accountScope = auth.userId`.
- A per-descriptor in-flight lock prevents duplicate P4 jobs; failures remain fail-closed and never synthesize Available/Play.
- After start/retry completion or error, the screen refreshes authoritative inbox + P4 snapshot and T2.B re-projects the resulting state.
- Exact implementation head `a99a712c`, Actions run `36025590927`: **15/15 PASS**; mobile **64/64 suites, 495/495 tests**.
- Owner approved execution of P6.T2.D before implementation. Aggregate T2 owner verification remains a T2.H closure obligation.
- T2.E remains the only leaf allowed to wire verified Available to Play. HPKE emulator remains disabled.

### T2 frozen source ownership

Writable T2 source/test envelope:

- `mobile/src/screens/InvitesScreen.tsx` — new product surface;
- `mobile/src/p2p/dashboard/InvitesModel.ts` — pure viewer-state/action projection only;
- `mobile/src/p2p/dashboard/useInvitesState.ts` — inbox + P4 local-state read orchestration;
- `mobile/src/p2p/dashboard/useInvitesActions.ts` — Claim/Sync/Play orchestration only;
- `mobile/__tests__/InvitesScreen.test.tsx` — new component/integration evidence;
- `mobile/src/navigation/RootNavigator.tsx`;
- `mobile/src/screens/HomeScreen.tsx`;
- `mobile/__tests__/RootNavigator.test.tsx`;
- `mobile/__tests__/HomeScreen.test.tsx`.

The three dashboard helper files are a bounded maintainability expansion of T0's expected T2 scope, learned from T1's max-lines refactor. They may project/orchestrate existing capabilities but may not duplicate P3/P4/P5 authorization, sync verification, key handling, or playback invariants.

Read-only dependencies unless a separately proven contract defect is found:

- `P2PDashboardService.listInbox()`;
- `P2PAudienceService.claimInvitation()`;
- `P2PSyncController.startSync/getSyncState/getVerifiedPackageHandle()`;
- `P2PPlaybackController.start/stop()`;
- `P2PPlaybackSessionView`;
- `P2PProvider/useP2PService/useP2PSyncController`;
- AuthProvider `sessionRef` + `userId`.

Frozen account binding: **`accountScope = auth.userId`**. This matches P4 cache isolation and P5's authorization assertion `viewerSubjectId === handle.accountScope`.

HPKE emulator remains disabled and outside P6.T2.

## P6.T3 — Dashboard flow and visual certification

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Certify all P6 parent HP/EC with component/integration and Android flow evidence; inspect loading/empty/error/expired states against DESIGN.md and shipped tokens.

- **HP-P6.T3-1:** Owner and invited viewer complete their respective actions using the minimal screens.
- **EC-P6.T3-1:** Account switch clears prior content/action state; denied responses cannot leave a stale enabled Play or Invite action.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.


### P6.T1 execution breakdown — 2026-09-22

| Block | Scope | Status |
|---|---|---|
| T1.A | Formal T0 closure / T1 activation | **PASS** |
| T1.B | My Content authoritative states + loading/empty/error | **PASS** |
| T1.C | Exact Ready descriptor Invite eligibility | **PASS** |
| T1.D | P3 Create Invite integration | **PASS** |
| T1.E | One-time token + Copy Invite | **PASS** |
| T1.F | Navigation | **PASS** |
| T1.G | Remaining component/integration tests | **PASS** |
| T1.H | Aggregate T1 certification/owner verification | **PASS — owner-reviewed; REVIEW-OVERRIDE** |

Block 1 evidence:
`docs/audit/mvp0-p2p-p6-t1-block1-my-content-2026-09-22.md`.

Block 2 evidence:
`docs/audit/mvp0-p2p-p6-t1-block2-create-copy-invite-2026-09-22.md`.

Block 3 evidence (T1.F+G):
`docs/audit/mvp0-p2p-p6-t1-block3-navigation-evidence-2026-09-22.md`.

T1.H closure evidence:
`docs/audit/mvp0-p2p-p6-t1-certification-candidate-2026-09-23.md`.
