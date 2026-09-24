---
type: Audit
title: "P6.T2.H — aggregate certification candidate"
status: ready-for-owner-verification
task: P6.T2
block: T2.H
date: 2026-09-24
---

# P6.T2.H — aggregate certification candidate

## Status

**READY FOR OWNER VERIFICATION — not PASS yet.**

All executable implementation leaves T2.A–G are PASS. This artifact consolidates
their behavioral evidence. The remaining mandatory gate is explicit owner final
verification.

The user's direction to continue through the remaining leaves authorized execution.
It is **not** treated as owner final verification of the completed parent.

## Aggregate implementation chain

```text
manual Claim
   ↓ P3
authoritative /api/p2p/inbox
   ↓
Pending
   ↓ Sync / Retry
P4 verified sync
   ↓
Available
   ↓ getVerifiedPackageHandle
P5 current O3 revalidation + playback startup
   ↓
P2PPlaybackSessionView

account/session change
   ↓
invalidate current viewer UI + stale completions
```

## Leaf evidence

| Leaf | Outcome | Implementation evidence |
|---|---|---|
| T2.A | activation/decomposition | PASS; parent RRI 100 decomposed into bounded leaves |
| T2.B | authoritative inbox + projection | `19a5bca9` / run `36019060607` / 15 of 15 |
| T2.C | manual Claim | `c104d40a` / run `36022359070` / 15 of 15 |
| T2.D | Sync / Retry | `a99a712c` / run `36025590927` / 15 of 15 |
| T2.E | verified Available + Play | `8fa7411d` / run `36029185810` / 15 of 15 |
| T2.F | fail-closed account/session lifecycle | `9c4b41e4` / run `36031993001` / 15 of 15 |
| T2.G | Home → Invites → Back / re-entry | `6f4a89e1` / run `36033387423` / 15 of 15 |

Latest T2.G mobile evidence: **64/64 suites, 509/509 tests**.

## Behavioral coverage certification

| Acceptance | Class | Required behavior | Evidence | Status |
|---|---|---|---|---|
| HP-P6.T2-1 | Happy path | viewer claims, sees inbox item, syncs, reaches verified Available and plays the package | `InvitesScreen.test.tsx`; P3 audience tests; P4 sync tests; P5 playback controller/session-view tests; `RootNavigator.test.tsx` | **passed** |
| EC-P6.T2-1 | Edge case | expired, inaccessible, incomplete or unverified content offers no Play; another viewer's invitation is never displayed | `InvitesModel`/screen tests for inactive, expired/revoked, viewer mismatch, descriptor mismatch, unverified READY; T2.F stale-account tests; P5 O3 revalidation tests | **passed** |

### HP trace

1. raw token remains volatile and Claim delegates to existing P3;
2. Claim success refreshes authoritative `/api/p2p/inbox`;
3. exact active descriptor with no local package exposes Sync;
4. Sync/Retry delegates to existing P4;
5. only fully verified P4 READY projects Available;
6. Play first requests a P4 verified handle;
7. P5 independently re-reads current O3 before startup;
8. existing P5 loopback session is rendered by the existing player view.

### EC trace

- another viewer is filtered fail-closed;
- inactive / expired / revoked authorization overrides local cached READY;
- missing or mismatched descriptor cannot expose Play;
- incomplete/unverified READY remains non-playable;
- failed P4 sync exposes Retry, not Play;
- P4 verified-handle failure creates no P5 startup;
- P5 current authorization denial creates no player;
- session expiry delegates to logout;
- account/session identity change removes stale rows/player/action state;
- old inbox/Claim/Sync/Play completions cannot mutate the new identity.

## Aggregate Reflection

### Pass 1 — authority

**PASS.** Backend/P3 owns invitation and authorization facts; P4 owns local verified
availability; P5 owns current authorization revalidation and playback startup. P6
projects and orchestrates those capabilities but does not manufacture authority.

### Pass 2 — secrets and lifecycle

**PASS.** Raw Claim tokens remain React-only and are cleared on success, logout,
remount and account/session change. K1 remains owned by P5. Playback sessions are
loopback-only and existing P5 view teardown remains authoritative.

### Pass 3 — failure and concurrency

**PASS.** Claim, Sync and Play have in-flight guards. Errors do not synthesize
Ready/Available/Play. Identity guards discard stale completions after account or
session changes. Navigation re-entry performs a fresh authoritative inbox read.

## Review override

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner.
- Scope-note: skips only phase-1 and phase-2 peer review for P6.T2 under
  `docs/audit/mvp0-p2p-review-exception.md`; tests, RRI, Reflection, behavioral
  coverage, owner verification and status synchronization remain mandatory.

The override row should be appended to `docs/audit/gemma-review-overrides.md`
only when owner final verification closes T2.

## Remaining gate — owner final verification

Not yet satisfied.

Required owner decision:

```text
P6.T2 owner verification
├─ review aggregate HP/EC evidence
├─ confirm viewer flow acceptable
└─ approve T2 closure
```

Until that explicit verification is given:

- **P6.T2 remains IN PROGRESS**;
- **T2.H remains READY FOR OWNER VERIFICATION**;
- **P6.T3 remains blocked on T2 PASS**.

HPKE emulator remains disabled.
