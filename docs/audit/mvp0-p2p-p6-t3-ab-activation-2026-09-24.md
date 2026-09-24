---
type: Audit
title: "P6.T3 activation — dashboard flow and canonical visual states"
status: pass
task: P6.T3
block: T3.A+B
date: 2026-09-24
---

# P6.T3 — first block activation

## Result

**P6.T3 is activated. First executable block: T3.A + T3.B.**

Dependency gate is satisfied: P6.T2 is PASS 2026-09-24. This block does not
reopen P3/P4/P5 capability ownership and does not perform final Android visual
certification.

## Frozen outcomes

### T3.A — dashboard integration

- keep the existing authenticated stack and Home → My Content / Invites routes;
- group My Content and Invites as one explicit P2P dashboard area;
- preserve existing route testIDs and remount/refetch behavior;
- no new navigation state and no capability/API changes.

### T3.B — canonical visual states

Use one P2P status presentation component over the shipped Badge primitive and
theme tokens.

Owner states:
- Processing → info
- Ready → success
- Failed → danger

Viewer states:
- Pending → info
- Syncing → info
- Sync error → danger
- Available → success
- Expired → warning

State meaning continues to come from the existing T1/T2 projections. This block
centralizes presentation only; it does not derive authorization or verification.

## Writable envelope

- `mobile/src/screens/HomeScreen.tsx`
- `mobile/__tests__/HomeScreen.test.tsx`
- `mobile/src/p2p/dashboard/P2pStatusBadge.tsx`
- `mobile/__tests__/P2pStatusBadge.test.tsx`
- `mobile/src/screens/MyContentScreen.tsx`
- `mobile/src/screens/InvitesScreen.tsx`
- `mobile/src/p2p/dashboard/MyContentModel.ts`
- `mobile/src/p2p/dashboard/InvitesModel.ts`

Read-only capability owners remain P2PDashboardService, P2PAudienceService,
P2PSyncController, P2PPlaybackController, P2PPlaybackSessionView and auth/session
ownership.

## RRI

Inputs for this bounded UI/presentation block:

| Variable | Score | Basis |
|---|---:|---|
| C | 1 | small presentational branching only |
| F | 3 | 8 source/test paths |
| D | 0 | no domain invariant change |
| T | 1 | existing screen/navigation coverage plus a focused status test |
| A | 0 | T0/T1/T2 contracts are explicit |
| K | 1 | composes existing UI models/components only |
| P | 0 | no authorization or persistent-data boundary change |
| X | 1 | bounded Home + two P2P screens |

Technical profile: L=1, I=1, Q=0, V=1 → bottleneck 1 → ICI 25.
Risk/domain input remains below the ICI anchor. **Final RRI: 25 / Low.**
No decomposition trigger applies.

The owner directive to work this first block also authorizes the implementation.

## Out of scope

- final Android screenshot/device certification;
- T3.C/T3.D/T3.E closure evidence;
- backend/API/schema changes;
- changes to P3/P4/P5 authority;
- HPKE emulator enablement.


## Closure evidence

T3.A+B closed PASS on exact head
`3474b8b2a09f287076aa8dd3ed27c1b2e1f528a1`.

GitHub Actions run `36041983233`: **15/15 PASS**, including mobile,
maintainability, fmt, clippy, test, coverage, roadmap-drift and QA-docs.

The canonical status layer remains presentation-only and Home retains the
existing authenticated navigation contract.
