---
type: Audit
title: "P6.T2.G — Home to Invites navigation and re-entry"
status: pass
task: P6.T2
block: T2.G
date: 2026-09-24
---

# P6.T2.G — navigation and re-entry

## Result

**PASS 2026-09-24.**

Implementation head: `6f4a89e192e2355e11d96efdda4c398aa127d302`.
GitHub Actions run: `36033387423` — **15/15 PASS**.
Mobile: **64/64 suites, 509/509 tests**; `RootNavigator.test.tsx` and `HomeScreen.test.tsx` PASS; P3 T2c3 self-check PASS.

## Navigation contract

```text
Home
 └─ Invites
      ├─ Claim
      ├─ Sync / Retry
      ├─ Play
      └─ Back → Home

re-enter Invites
      ↓
real screen remount
      ↓
fresh GET /api/p2p/inbox
```

## Implemented behavior

- Home quick actions now include `home-open-invites`.
- The existing authenticated native stack owns the new `Invites` route.
- `InvitesScreen` receives an optional Back callback and exposes `invites-back`.
- Back uses `navigation.goBack()`; there is no second navigation store.
- Leaving Invites unmounts the screen and its volatile Claim/Sync/Play UI state.
- Re-entering remounts the real screen and performs a new authoritative inbox fetch.
- Auth loss replaces the authenticated navigator with Login and removes Invites.
- T2.F identity guards continue to protect stale async completions inside the route.

## Behavioral evidence

| Case | Result |
|---|---|
| Home exposes Invites action | PASS |
| Home → Invites | PASS |
| Invites → Back → Home | PASS |
| Re-entry performs second /api/p2p/inbox request | PASS |
| auth becomes unauthenticated → Invites removed | PASS |
| existing Home actions remain green | PASS |

## Reflection log — RRI 55 / Med-high

1. **Navigation ownership:** PASS. Existing native stack is reused; no duplicated navigation state.
2. **Remount/lifecycle:** PASS. Re-entry creates a new Invites screen and re-runs authoritative inbox loading.
3. **Auth boundary:** PASS. Unauthenticated state removes the entire authenticated route tree.

## Gate corrections

The first navigation patch exposed only repository structure issues:
- missing `onOpenInvites` propagation through `DashboardContent`;
- `AuthedNavigator`, `HomeScreen`, and `InvitesScreen` exceeded function-line budgets after the route was added.

The final patch passes all type/lint/maintainability gates; rendering logic was extracted without changing behavior.

## Next

All executable implementation leaves T2.A–G are PASS.
**T2.H — aggregate component/integration evidence + closure** remains.
Owner final verification is still required before P6.T2 can be marked PASS.

HPKE emulator remains disabled.
