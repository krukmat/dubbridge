---
type: Audit
title: "P6.T2.F — fail-closed expiry/session/account lifecycle"
status: pass
task: P6.T2
block: T2.F
date: 2026-09-24
---

# P6.T2.F — fail-closed lifecycle

## Result

**PASS 2026-09-24.**

Implementation head: `9c4b41e42dac2bb57ba51453eb4a3eeec9c08665`.
GitHub Actions run: `36031993001` — **15/15 PASS**.
Mobile: **64/64 suites, 507/507 tests**; `InvitesScreen.test.tsx` PASS; P3 T2c3 self-check PASS.

## Lifecycle contract

```text
userId/sessionRef changes
        ↓
invalidate visible viewer state immediately
        ↓
clear Claim / Sync / Play transient UI state
        ↓
drop playbackSession → P2PPlaybackSessionView unmount
        ↓
discard late completions from previous identity
        ↓
load authoritative inbox for the new identity only
```

Backend authorization remains authoritative over any local P4 cache.

## Implemented behavior

- `useInvitesState` binds each load to the captured `userId + sessionRef`.
- Identity change resets the visible inbox to Loading before the replacement fetch resolves.
- A late previous-account inbox response cannot mutate the current view.
- Claim state is cleared on identity change; a late Claim completion cannot rotate or refresh the new account and cannot release a newer Claim lock.
- Sync busy/error state is cleared on identity change; late prior-account Sync completion cannot refresh the current account.
- Playback busy/error/session state is cleared on identity change.
- A late prior-account verified-handle completion cannot enter P5 startup.
- Clearing the playback session unmounts the existing `P2PPlaybackSessionView`; its existing P5 tests already prove teardown on unmount.
- expired/revoked authorization precedence remains stronger than cached verified READY.

## Behavioral evidence

| Case | Result |
|---|---|
| expired + cached READY → Expired / no action | PASS |
| revoked + cached READY → Expired / no action | PASS |
| visible Available/player + account change → prior row/player removed | PASS |
| old inbox finishes after account switch → ignored | PASS |
| old P4 verified handle finishes after account switch → no P5 start | PASS |
| raw Claim token + account switch → token cleared | PASS |

## Reflection log — RRI 55 / Med-high

1. **Authorization precedence:** PASS. Local cache/playback state cannot survive a changed viewer authority context.
2. **Async race isolation:** PASS. Inbox, Claim, Sync and Play paths are identity-scoped and stale completions are fail-closed.
3. **Lifecycle ownership:** PASS. P4 cache clearing remains owned by `P2PProvider`; playback teardown remains owned by `P2PPlaybackSessionView`/P5. P6 only invalidates its product-facing state.

## Gate corrections

- T2.F test additions initially crossed the maintainability duplicate-line budget; fixture setup was deduplicated.
- Lifecycle guards initially pushed `useInvitesState` and `usePlaybackAction` over line/complexity budgets; behavior was split into bounded helpers with no contract relaxation.

## Out of scope / next

- no Home → Invites route wiring in this leaf;
- no new auth/P4/P5 state machine;
- no remote fallback;
- HPKE emulator remains disabled.

Next leaf: **T2.G — Home → Invites navigation + re-entry**.
