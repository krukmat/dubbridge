---
type: Audit
title: "MVP0-P2P P6.T0 preflight"
status: draft
slice: MVP0-P2P
---

# P6.T0 preflight — state/action/navigation contract

This is a preflight only. **P6.T0 remains blocked until P3-P5 are PASS.** No P6 task is activated by this record.

## Current mobile seams confirmed

### Owner action already available

`P2PAudienceService.createInvitation(accessToken, assetId, ttlSeconds?)` calls the production owner invitation endpoint. Backend creation is authoritative and is expected to reject non-owned/non-ready assets.

### Viewer invitation/claim seams already available

Mobile currently exposes:

- `listInvitations(accessToken)`;
- `claimInvitation(accessToken, token)` with Android device registration;
- `getAuthorization(accessToken, authorizationId)`;
- transient device-envelope unwrap for playback.

Claim returns invitation + authorization + ready descriptor for the immediate flow.

### P4 local state already authoritative for sync

`P2PSyncController` owns local sync and exposes persisted `P2pSyncSnapshot` state. The only valid `READY` snapshot requires:

- `phase === READY`;
- manifest verified;
- package verified;
- file and byte progress complete.

P6 must project `Syncing/Available/Failed` from this state rather than from transport completion alone.

### P5 playback seam already available

P5 consumes a verified package handle and current authorization, starts a scoped loopback playback session and owns deterministic teardown. P6 should invoke this seam rather than constructing media URLs.

## Contract gaps found before activation

### GAP-1 — no owner P2P publication read model for `Processing / Ready / Failed`

The current mobile P2P API has create-invite, claim/list invitation, authorization and device-envelope operations, but no owner-facing endpoint that lists authoritative P2P publication state.

`GET /api/assets` currently supplies the general asset status only. That status is not equivalent to the P2P publication lifecycle and must not be reused as a proxy for `Processing/Ready/Failed`.

The backend has an authoritative `p2p_ready_repo::get_ready_descriptor_by_asset` read, but it is consumed internally by the claim path and is not exposed as an owner dashboard projection.

**Activation consequence:** P6.T0 must freeze an owner read contract before P6.T1. The UI must not infer Ready from asset/review state or by probing invitation creation.

### GAP-2 — claimed viewer invitation cannot be fully reconstructed after restart

`listInvitations()` returns invitation identity/status only. A successful claim returns the authorization ID and ready descriptor, but the current invitation list does not return either value and there is no list/recovery API that maps a claimed invitation back to its active authorization + descriptor.

Therefore an app restart cannot reliably reconstruct `claimed -> sync state -> Available -> Play` using backend facts alone unless the claim result is persisted locally as another source of truth.

**Activation consequence:** P6.T0 must choose one authoritative recovery contract before P6.T2:

1. backend projection returns claimed invitation + active authorization + descriptor; or
2. a deliberately persisted, account-scoped claim record is introduced with explicit invalidation/expiry rules.

Do not silently cache the one-time token or CK to solve this gap.

## Draft state projection for P6.T0

This table is a draft for activation, not a frozen contract.

| Surface | UI state | Minimum authoritative facts | Allowed action |
|---|---|---|---|
| My Content | Processing | owned asset has P2P publication in non-terminal/non-ready state | none |
| My Content | Ready | authoritative same-lineage ready publication/descriptor for owner asset | Invite |
| My Content | Failed | authoritative terminal publication failure | Retry/error presentation only if backend contract supports it |
| Invites | Pending | invitation pending + not expired/revoked | Claim |
| Invites | Syncing | claimed authorization current + P4 snapshot not READY/FAILED/CANCELLED | progress/cancel as frozen by T0 |
| Invites | Available | current authorization + P4 verified READY snapshot for same account/publication/lineage | Play |
| Invites | Expired | backend invitation/authorization expired or revoked | no Play |
| Invites | Failed | P4 FAILED or authorization/playback precondition failed | retry/error according to frozen contract |

## Navigation preflight

Current authenticated navigation contains Home, Assets, Upload, Organizations/Projects, Compliance/Consent and Review. There are no `MyContent` or `Invites` routes yet.

Proposed minimum ownership for activation:

- `Home` adds entry points only; it does not own P2P state.
- `MyContentScreen` owns owner projection and invite action.
- `InvitesScreen` owns viewer invitation projection and claim/sync/play actions.
- P3/P4/P5 controllers remain domain seams; screens do not bypass them.
- Account switch/sign-out clears screen state and P2P runtime/cache ownership through the existing provider/controller lifecycle.

## Activation gate

P6.T0 may activate only after P3-P5 PASS. At activation, resolve GAP-1 and GAP-2, freeze exact API/state contracts and writable paths, then implement tests before P6.T1 UI work.
