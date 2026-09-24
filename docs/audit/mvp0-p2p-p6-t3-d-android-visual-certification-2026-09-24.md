---
type: Audit
title: "P6.T3.D — Android visual certification preparation"
status: in_progress
task: P6.T3
block: T3.D
date: 2026-09-24
---

# P6.T3.D — Android visual certification preparation

## D1 frozen scope

T3.D reuses the existing Android + Maestro screenshot infrastructure. It does
not introduce a second UI test framework and it does not execute the complete
P7Local owner-to-viewer E2E.

Visual states to certify:

### My Content
- Processing
- Ready, including visible Create invite action
- Failed
- Empty
- Error / Retry
- Loading

### Invites
- Pending
- Syncing
- Sync error
- Available, including visible Play action
- Expired
- Empty
- Error / Retry
- Loading

### Navigation/layout
- Home P2P sharing section
- Home → My Content
- Home → Invites
- long Invites content remains scrollable
- no clipped primary action or status badge

## Fixture boundary

Authoritative facts come from the existing mock gateway through the real
`/api/p2p/content` and `/api/p2p/inbox` clients.

Local viewer states remain projected by the product `InvitesModel`. A
development-only visual fixture harness may seed non-secret P4 sync snapshots
into the existing `ExpoP2pSyncCache` for fixed E2E identities. It must not
override `InvitesModel`, authorization, verified-handle logic, P5 playback,
or production configuration.

Fixture execution is gated by
`EXPO_PUBLIC_P2P_VISUAL_FIXTURES=true` and is enabled only by the local
screenshot runner.

## Execution boundary

- D1–D3 may be implemented in-repo.
- D4 requires the owner's local Android emulator/toolchain and is intentionally
  not claimed by repository CI.
- D5/D6 consume the actual D4 screenshots/logs.
- HPKE emulator remains disabled.

## RRI / decomposition

The unsplit D1–D3 envelope crosses mock HTTP, Expo filesystem state and Maestro
orchestration, so it is treated as a high-coupling integration parent and is
executed only through bounded leaves:

- D1 docs/contract: Low.
- D2 fixed E2E fixtures + dev-only cache seeding: bounded integration leaf.
- D3 Maestro YAML + runner/docs wiring: bounded tooling leaf.

Owner authorization for this envelope is the explicit 2026-09-24 directive
`sigue con tu plan`.
