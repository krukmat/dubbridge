---
type: Audit
title: "P6.T2 activation — viewer claim, sync and play"
status: pass
task: P6.T2
block: T2.A
date: 2026-09-23
---

# P6.T2 — T2.A activation

## Result

**T2.A PASS 2026-09-23. P6.T2 is activated but source implementation has not started.**

Activation base: `be9654654a75bbf1ed66d16144b709e0945b67da`.

That head completed GitHub Actions run `35826652538` successfully before T2 activation.

T1 is PASS, so the T2 dependency gate is satisfied. P6.T3 remains blocked until
aggregate T2 PASS.

## Contract retained from T0

Viewer state remains a projection of two independent authorities:

```text
GET /api/p2p/inbox                 P4 local sync state
        |                                 |
        +-------- authorization ----------+
                         |
                  product projection
                         |
       Pending / Syncing / Available / Expired
```

Authorization/access has precedence over local cache state.

- inactive/expired/revoked → **Expired**, no Sync/Play;
- active + no descriptor → **Pending**, refresh only;
- active + descriptor + IDLE/CANCELLED/no snapshot → **Pending**, Sync;
- active + descriptor + FAILED → **Sync error**, Retry Sync;
- active + descriptor + DISCOVERING/DOWNLOADING/VERIFYING/RETRYING → **Syncing**;
- active + exact descriptor + verified P4 READY → **Available**, Play.

Claim success alone never enables Play.

## Dependency verification

### P3 / audience capability

Existing `P2PAudienceService.claimInvitation(accessToken, token)`:

- ensures the existing device identity;
- delegates to the established P3 claim API;
- returns the established authorization/claim result.

T2 does not reproduce token, device, K1 or authorization rules.

### P4 / sync capability

Existing `P2PSyncController` supplies:

- `startSync(descriptor, accountScope)`;
- `getSyncState(identity)`;
- `getVerifiedPackageHandle(descriptor, accountScope)`;
- account-scoped cache cleanup.

P4 READY already requires:

```text
phase == READY
manifestVerified == true
packageVerified == true
progress complete
```

T2 may display this result; it may not weaken or recreate it.

### P5 / playback capability

Existing `P2PPlaybackController.start(accessToken, authorizationId, handle)`
re-reads O3 authorization and checks asset/publication/lineage/viewer/expiry before
K1 unwrap and loopback playback.

Existing `P2PPlaybackSessionView` owns the VideoPlayer session lease and
deterministic stop/retry/unmount teardown.

T2 only obtains the P4 verified handle and delegates.

### Account binding

`AuthProvider.userId` is the frozen P6 viewer `accountScope`.

This matches:

- `P2PProvider` account-cache cleanup;
- P4 cache identity;
- P5 requirement `authorization.viewerSubjectId === handle.accountScope`.

No independent account identifier is introduced.

## Parent RRI

The coherent parent is intentionally scored before decomposition.

Inputs against current `scripts/rri.py` policy:

| Variable | Score | Basis |
|---|---:|---|
| C | 2 | multi-state UI orchestration; complexity must be distributed across helpers |
| F | 3 | 9 frozen source/test paths |
| D | 4 | authorization precedence + async sync/play/session orchestration |
| T | 1 | strong P3/P4/P5 tests exist; new T2 component evidence is still required |
| A | 0 | T0 state/action contract + explicit HP/EC remove ambiguity |
| K | 4 | composes backend inbox, P4 runtime/cache and P5 playback lifecycle |
| P | 4 | viewer data visibility/action eligibility; UI is not the auth boundary |
| X | 4 | screen + auth + dashboard + sync + playback/navigation context |

Technical profile:

```text
L=2  I=4  Q=4  V=1
bottleneck=4
ICI=100
risk/domain input=30
FINAL RRI=100 / Very high / XL
```

Penalties: none. The task changes no auth implementation, crypto, database schema or
public backend API; therefore no manual auth-security penalty is added.

Decomposition trigger: **RRI >= 56**.

The RRI-100 parent is non-executable as one patch.

## Frozen decomposition

| Block | Outcome | RRI | Primary writable scope |
|---|---|---:|---|
| T2.B | authoritative inbox + product-state projection | 55 | InvitesModel, useInvitesState, InvitesScreen, tests |
| T2.C | manual Claim + authoritative refresh | 55 | useInvitesActions, InvitesScreen, tests |
| T2.D | Sync / Retry Sync | 55 | useInvitesActions, InvitesModel, InvitesScreen, tests |
| T2.E | verified Available + Play | 55 | useInvitesActions, InvitesScreen, tests |
| T2.F | expiry/session/account fail-closed lifecycle | 55 | state/actions/model/screen tests |
| T2.G | Home → Invites → Back / re-entry | 55 | RootNavigator, HomeScreen and their tests |
| T2.H | aggregate test/evidence closure | 25 | tests + P6 evidence/status docs |

B–G remain Med-high because they orchestrate only already-owned capabilities and
do not reopen their internal authorization/verification/key/playback logic.

## Exact writable envelope

- `mobile/src/screens/InvitesScreen.tsx`
- `mobile/src/p2p/dashboard/InvitesModel.ts`
- `mobile/src/p2p/dashboard/useInvitesState.ts`
- `mobile/src/p2p/dashboard/useInvitesActions.ts`
- `mobile/__tests__/InvitesScreen.test.tsx`
- `mobile/src/navigation/RootNavigator.tsx`
- `mobile/src/screens/HomeScreen.tsx`
- `mobile/__tests__/RootNavigator.test.tsx`
- `mobile/__tests__/HomeScreen.test.tsx`

The three helper files are a bounded maintainability refinement of T0's expected
T2 scope. They may contain projection/orchestration only.

Read-only capability owners unless a separately evidenced contract defect appears:

- `mobile/src/p2p/P2PDashboardService.ts`
- `mobile/src/p2p/P2PAudienceService.ts`
- `mobile/src/p2p/sync/P2PSyncController.ts`
- `mobile/src/p2p/playback/P2PPlaybackController.ts`
- `mobile/src/p2p/playback/P2PPlaybackSessionView.tsx`
- `mobile/src/p2p/P2PProvider.tsx`

## Review / approval envelope

The standing MVP0-P2P owner exception continues to waive phase-1/phase-2 model
peer review for P0–P7; it does not waive RRI, tests, Reflection, behavioral
coverage or owner verification.

T2.A itself is activation/planning only. Implementation must proceed leaf by leaf;
no T2.B source work is included in this commit.

## Out of scope

- no new deep-link protocol; Claim remains manual-token entry;
- no new sync state machine;
- no new playback controller/player;
- no backend/API/schema change;
- no multi-device/background/offline expansion;
- **HPKE emulator remains disabled until explicit owner instruction.**

## Status

```text
P6.T2
├─ A PASS
├─ B Pending
├─ C Pending
├─ D Pending
├─ E Pending
├─ F Pending
├─ G Pending
└─ H Pending
```
