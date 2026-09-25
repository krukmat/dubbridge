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


## D2 implementation

D2 adds two deterministic fixture inputs without changing product projection
logic:

- mock gateway `p2p_seed=states|empty|error|loading` modes for the real
  `/api/p2p/content` and `/api/p2p/inbox` clients;
- `P2PVisualFixtureHarness`, enabled only by
  `EXPO_PUBLIC_P2P_VISUAL_FIXTURES=true`, which writes fixed non-secret
  DOWNLOADING / FAILED / verified READY snapshots through
  `ExpoP2pSyncCache`.

`InvitesModel`, authorization, P4 verified-handle creation and P5 playback
remain untouched. The visual harness does not create manifests, ciphertext,
keys or playback material; Available is therefore a visual projection only and
D4 must not use its Play button as E2E playback evidence.


## D3 implementation

D3 adds four focused Maestro flows and a P2P-only local command:

`START_MOCK_SERVERS=1 npm run screenshots:p2p`

The existing screenshot runner remains the single Android orchestration path.
The P2P-only mode still performs dependency checks, APK bundle refresh/signing,
install, health checks and adb reverse; it skips unrelated legacy visual phases.

Expected visual artifacts:

- `20_p2p_home.png`
- `21_p2p_my_content_states_top.png`
- `22_p2p_my_content_states_bottom.png`
- `23_p2p_invites_states_top.png`
- `24_p2p_invites_states_bottom.png`
- `25_p2p_my_content_empty.png`
- `26_p2p_invites_empty.png`
- `27_p2p_my_content_error.png`
- `28_p2p_invites_error.png`
- `29_p2p_my_content_loading.png`
- `30_p2p_invites_loading.png`

Repository CI can validate code/config/tests, but these PNGs are not claimed
until D4 is run on the owner's local Android emulator.


## D4 local execution observation

The first local Android attempt was operationally valid but did not complete the
full four-flow certification. Environment-only corrections were required:
OpenJDK 17 for the Android build, `npm ci` to restore declared dependencies,
an APK rebuild carrying the E2E configuration, and stopping the unrelated
`local-gateway-1` container that occupied port 8082. These actions did not
change product behavior or P3/P4/P5/P7Local semantics.

The remaining D4 blocker was isolated to `p2p-states.yaml`: scrolling the
Available status into view did not guarantee that the separate Play button was
also inside the viewport. The bounded repair adds an explicit
`scrollUntilVisible` for `invite-play-invite-p2p-available` before asserting
that control.

D4 remains **BLOCKED pending local rerun**. `local-gateway-1` was left in
Exited state and is not restarted by this task; restoring it is a separate
owner-authorized environment action.

Broader harness hardening (port preflight, reproducible APK build automation,
partial screenshot promotion, and path normalization) remains outside this
bounded D4 repair.
