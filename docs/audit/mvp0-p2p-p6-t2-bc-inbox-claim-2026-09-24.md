---
type: Audit
title: "P6.T2 B/C — viewer inbox projection and manual Claim"
status: pass
task: P6.T2
blocks: [T2.B, T2.C]
date: 2026-09-24
---

# P6.T2 — T2.B/C implementation evidence

## Result

**T2.B PASS. T2.C PASS. P6.T2 remains IN PROGRESS.**

No T2.D/E Sync or Play action was implemented in these blocks.

## T2.B — authoritative viewer inbox + product-state projection

Implementation converges two existing authorities:

```text
GET /api/p2p/inbox
       +
P4 getSyncState(accountScope, publication, lineage)
       ↓
Pending / Syncing / Sync error / Available / Expired
```

Properties verified:

- backend authorization has precedence over local cache state;
- another viewer's authorization is filtered fail-closed;
- descriptor mismatch cannot produce Available;
- inactive/expired/revoked authorization cannot produce Play;
- READY is Available only when P4 reports verified manifest + package + complete progress;
- loading, empty, network/API error and Retry are explicit;
- no Claim, Sync or Play mutation is performed by T2.B.

Evidence head: `19a5bca9d61ed1250d80a42b280e84ebdf064769`.
GitHub Actions run: `36019060607` — **15/15 PASS**.
Mobile: **64/64 suites, 481/481 tests**; `InvitesScreen.test.tsx` PASS; P3 T2c3 self-check PASS.

## T2.C — manual Claim via existing P3 capability

Implemented through `useInvitesActions.ts` and the existing
`P2PAudienceService.claimInvitation()` boundary.

Properties verified:

- token is trimmed before delegation;
- blank input cannot submit;
- an in-flight Claim blocks duplicate submit;
- no device-registration, crypto, K1 or authorization rules are duplicated in P6;
- success applies session rotation, clears the raw token, then refreshes
  `GET /api/p2p/inbox`;
- success does not synthesize Available or Play;
- HTTP 404 / 409 / 410 and forbidden/network failures do not fabricate an inbox item;
- session expiry clears the token and delegates to existing logout;
- raw claim token is React state only and cannot be recovered after unmount/remount.

Final implementation head: `c104d40a61997e67e581ba89c854858b3f68f7a0`.
GitHub Actions run: `36022359070` — **15/15 PASS**.
Mobile: **64/64 suites, 489/489 tests**; `InvitesScreen.test.tsx` PASS; P3 T2c3 self-check PASS.

## Gate corrections during T2.C

The first passes exposed only repository-quality/test-harness issues, all corrected before PASS:

1. repeated test render setup exceeded maintainability duplication budget → shared render helper;
2. `useInvitesActions` exceeded the 60-line function lint budget → compact refactor with unchanged behavior;
3. initial async test harness produced overlapping React `act()` scopes → input changes now flush through the repository's existing input-test pattern and async Claim completion is observed before assertion.

No production contract was relaxed to make CI pass.

## Boundary / next

- P3 remains Claim authority owner.
- P4 remains verified sync owner.
- P5 remains playback owner.
- HPKE emulator remains disabled.
- Owner approved execution of P6.T2.C before implementation; aggregate owner verification remains due at T2.H.
- Next leaf: **T2.D — Sync / Retry Sync**.
