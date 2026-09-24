---
type: Audit
title: "P6.T2.E — verified Available + Play"
status: pass
task: P6.T2
block: T2.E
date: 2026-09-24
---

# P6.T2.E — Available + Play

## Result

**PASS 2026-09-24.**

Implementation head: `8fa7411d7086716a8465ac926179d227a2fc3ff6`.
GitHub Actions run: `36029185810` — **15/15 PASS**.
Mobile: **64/64 suites, 501/501 tests**; `InvitesScreen.test.tsx` PASS; P3 T2c3 self-check PASS.

A later concurrent S-230 documentation commit `4bfef67a` is a direct child of
the implementation head and does not modify P6 source or P6 status artifacts.

## Boundary

T2.E does not create another playback path. It composes the existing P4 → P5
handoff:

```text
T2.B projection == Available
          ↓ Play
P4 getVerifiedPackageHandle()
          ↓ verified handle only
P5 P2PPlaybackController.start()
          ↓ current O3 re-read + transient K1 + loopback startup
P2PPlaybackSession
          ↓
P2PPlaybackSessionView → existing VideoPlayer
```

No direct HTTP/S3 audience-media fallback is introduced.

## Implemented behavior

- **Play is rendered only for `projection.action === "play"`.**
- T2.B produces that action only for active authorization + exact descriptor +
  fully verified P4 READY.
- The action handler revalidates account/viewer/descriptor eligibility.
- It calls `getVerifiedPackageHandle(descriptor, auth.userId)` before any P5 start.
- A failure to obtain the verified handle creates no playback session.
- It calls P5 with the current session token, authoritative authorization ID and
  verified P4 handle.
- P5 remains responsible for current O3 authorization validation, K1 unwrap and
  loopback session startup.
- Successful session rotation is applied before rendering the session.
- The existing `P2PPlaybackSessionView` consumes the returned loopback session
  and retains its existing teardown-on-error/unmount behavior.
- Authorization denial creates no player and refreshes authoritative inbox state.
- Session expiry creates no player and delegates to existing logout.
- A descriptor-level in-flight guard prevents duplicate Play startup.

## Behavioral evidence

| Case | Evidence | Result |
|---|---|---|
| verified Available → P4 handle → P5 start → player session | `InvitesScreen.test.tsx` + existing P5 controller/view tests | PASS |
| unverified READY → no Play | `InvitesScreen.test.tsx` | PASS |
| P4 handle unavailable → no P5 start / no player | `InvitesScreen.test.tsx` | PASS |
| P5 current authorization denied → no player + inbox refresh | `InvitesScreen.test.tsx` | PASS |
| P5 session expired → logout / no player | `InvitesScreen.test.tsx` | PASS |
| duplicate Play while startup in flight → one P4/P5 startup | `InvitesScreen.test.tsx` | PASS |
| P5 authorization mismatch before K1/startup | `mobile/__tests__/p2p/p2p-playback-controller.test.ts` | PASS |
| player consumes only scoped loopback session + lifecycle callbacks | `mobile/__tests__/p2p/p2p-playback-session-view.test.tsx` | PASS |

## Reflection log — RRI 55 / Med-high

1. **Authority pass:** PASS. UI never manufactures playback eligibility; P4
   verified READY is required and P5 independently revalidates current O3.
2. **Secret/playback-boundary pass:** PASS. P6 receives only the verified handle
   and returned playback session; K1 and loopback startup remain owned by P5.
3. **Failure/lifecycle pass:** PASS. Failed P4/P5 startup cannot create a player;
   existing P5 view owns playback teardown. Remaining cross-account/session
   lifecycle consolidation is explicitly retained for T2.F.

The standing MVP0-P2P owner review exception remains applicable. The owner
directed continuation into the next P6.T2 leaf after T2.D closure; aggregate
parent owner verification remains due at T2.H.

## Gate corrections

Two repository-quality adjustments were required before PASS:

1. added playback orchestration exceeded the mobile declaration/import budget
   (23 > 20) → reused the existing claim construction shape and removed a helper;
2. playback/UI functions exceeded the 60-line lint budget by 4–6 lines →
   compacted existing structure with no behavior change.

No authorization, verification or playback rule was relaxed.

## Out of scope / next

- no account-switch/expiry lifecycle consolidation beyond the direct P5 failure paths;
- no Home → Invites navigation wiring;
- no new playback controller or player;
- no remote audience-media fallback;
- HPKE emulator remains disabled.

Next leaf: **T2.F — fail-closed expiry/session/account lifecycle**.
