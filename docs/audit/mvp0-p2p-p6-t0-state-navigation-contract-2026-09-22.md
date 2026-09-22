---
type: Audit
title: "MVP0-P2P P6.T0 state, action and navigation contract"
date: 2026-09-22
task: P6.T0
status: closure_ready_owner_verification_pending
rri: 25
band: Low
---

# P6.T0 — state, action and navigation contract

## Result

**CLOSURE-READY. Owner verification pending.**

P6 activation prerequisites are satisfied:

- P3 PASS;
- P4 PASS;
- P5-DEV SATISFIED;
- P5.T3/P5-CERT intentionally remains outside the development activation gate.

P6.T0 is planning-only. No product source is changed by this contract.

## RRI and writable scope

T0 writable scope is documentation-only:

- `docs/tasks/mvp0-p2p-p6-dashboard.md`;
- `docs/plan/mvp0-p2p-p6-dashboard.md`;
- this evidence record;
- go-live mirror status synchronization.

The DubBridge RRI rubric gives `docs/*` D/P/K floors of zero. With no runtime
behavioral diff, T0 remains **RRI 25 / Low**. This score does not transfer to
T1/T2/T3; each implementation leaf must be rescored from its actual source
paths at activation.

## Authoritative inputs

P6 must compose existing authorities; it must not create another product-state
store.

### Owner facts

Source:

```text
GET /api/p2p/content
  -> P2PDashboardService.listOwnerContent()
```

Canonical projection is produced by `p2p_dashboard.rs`:

- `processing` — publication is not authoritative Ready, or exact Ready
  descriptor is absent;
- `ready` — publication is Ready **and** an exact
  asset/publication/lineage descriptor exists;
- `failed` — publication is terminal Failed.

Generic asset/S-120 status never substitutes for this P2P projection.

### Viewer facts

Source:

```text
GET /api/p2p/inbox
  -> P2PDashboardService.listInbox()
```

Each item provides:

- claimed invitation identity/status;
- current durable authorization identity;
- `authorizationActive`;
- exact Ready descriptor when the package identity still matches.

The backend inbox is scoped to the authenticated claimed viewer only.

### Local package facts

Source:

```text
P2PSyncController.getSyncState({
  accountScope,
  publicationId,
  lineageId
})
```

A local package is playable only after the existing P4 invariant reaches:

```text
phase == READY
manifestVerified == true
packageVerified == true
progress complete
```

`VerifiedP2pPackageHandle` remains the only P4→P5 package capability.

## Frozen owner presentation

| Backend P2P state | Product label | Invite action |
|---|---|---|
| processing | Processing | disabled / hidden |
| ready + exact descriptor | Ready | enabled |
| failed | Failed | disabled / hidden |

**Invite eligibility is exactly `state === ready && descriptor != null`.**

The UI may not enable Invite from generic asset `Ready/finalized` state.

## Frozen viewer presentation

Authorization/access state has precedence over local cache state.

| Current facts | Product state | Allowed action |
|---|---|---|
| invitation expired/revoked OR `authorizationActive == false` | Expired | none |
| auth active, descriptor absent | Pending | refresh only |
| auth active + descriptor, no snapshot / IDLE / CANCELLED | Pending | Sync |
| auth active + descriptor, FAILED | Sync error | Retry Sync |
| auth active + descriptor, DISCOVERING/DOWNLOADING/VERIFYING/RETRYING | Syncing | no Play |
| auth active + descriptor + verified READY | Available | Play |

A previously verified local package becomes **non-playable immediately in the
UI projection** when current access is inactive. P5 still independently
revalidates current O3 when Play is invoked; UI eligibility is never an
authorization boundary.

## Action predicates

### Create invite

Requires:

- owner-scoped `P2pOwnerContent`;
- state `ready`;
- exact Ready descriptor.

Action delegates to existing P3 audience service/API. The one-time raw invite
token is presented for copy once; P6 must not persist it.

### Claim

MVP0 uses **manual token entry in Invites**. A new P2P deep-link protocol is
not required by T1/T2.

Claim delegates to the existing P3 claim operation and then refreshes inbox
facts. Claim success does not itself enable Play.

### Sync

Requires:

- current active authorization;
- exact descriptor;
- current account scope.

Delegates to `P2PSyncController.startSync`. UI does not infer Ready from
download completion.

### Play

Requires for UI eligibility:

- current active authorization;
- exact descriptor;
- verified P4 READY state for the same account/publication/lineage.

The action obtains a `VerifiedP2pPackageHandle` and delegates to the existing
P5 playback controller, which re-reads current O3 before native unwrap/start.

## Navigation and component ownership

P6 adds only two product destinations to the existing authenticated stack:

```text
Home
 ├─ My content
 │    └─ owner P2P publication state + Create/Copy invite
 └─ Invites
      └─ manual Claim + viewer inbox + Sync/Retry + Play
```

Frozen ownership:

- `mobile/src/navigation/RootNavigator.tsx`
  - owns route declarations and route wiring only;
- `mobile/src/screens/HomeScreen.tsx`
  - owns entry actions only; it does not load P2P dashboard state;
- `mobile/src/screens/MyContentScreen.tsx` (new in T1)
  - owns owner list/loading/empty/error and Invite presentation;
- `mobile/src/screens/InvitesScreen.tsx` (new in T2)
  - owns claim input, viewer list/loading/empty/error, sync/play action
    presentation;
- `mobile/src/p2p/P2PDashboardService.ts`
  - remains the read façade for backend-authoritative owner/inbox facts;
- `P2PAudienceService`, `P2PSyncController`, and
  `P2PPlaybackController`
  - remain action/capability owners; screens orchestrate them but do not
    duplicate their invariants.

No Redux/global P6 store, new service container, or second sync state machine is
introduced.

## Error, empty and session behavior

P6 reuses shipped primitives:

- `Screen`;
- `ScreenHeader`;
- `Card`;
- `Badge`;
- `Button`;
- `StateView`;
- existing dark tokens from `mobile/src/theme/tokens.ts` / `DESIGN.md`.

Required states:

- loading — standard `StateView`;
- empty — explicit My Content / Invites empty copy;
- network/API error — redacted stable copy + Retry;
- session expiry — existing Auth logout/session flow;
- sync failure — item remains non-playable and offers Retry Sync when access
  remains active;
- access expiry/revocation — Expired, no Sync/Play regardless of cached bytes.

## T1 exact source scope

Expected writable source paths:

- `mobile/src/screens/MyContentScreen.tsx` — new;
- `mobile/__tests__/MyContentScreen.test.tsx` — new;
- `mobile/src/navigation/RootNavigator.tsx`;
- `mobile/src/screens/HomeScreen.tsx`.

Existing P3/P6 APIs/services are read dependencies and should not change unless
T1 discovers a contract defect.

## T2 exact source scope

Expected writable source paths:

- `mobile/src/screens/InvitesScreen.tsx` — new;
- `mobile/__tests__/InvitesScreen.test.tsx` — new;
- `mobile/src/navigation/RootNavigator.tsx`;
- `mobile/src/screens/HomeScreen.tsx`.

Existing `P2PDashboardService`, `P2PAudienceService`,
`P2PSyncController`, `P2PPlaybackController`, sync state and playback
session components are dependencies, not duplicate implementation targets.

## Acceptance mapping

### HP-P6.T0-1

> Owned ready content permits Invite; an authorized verified invitation permits
> Play.

**PASS by contract:** Invite maps only from authoritative P2P `ready` +
descriptor. Play maps only from active authorization + exact descriptor +
verified READY package, while P5 still revalidates O3 at execution.

### EC-P6.T0-1

> S-120 Ready alone or synced-but-unverified data never enables the
> corresponding P2P action.

**PASS by contract:** generic asset status is excluded from Invite eligibility;
all non-verified sync states are excluded from Play.

## Handoff

After owner verification, P6.T0 may become PASS and P6.T1 may activate.

P6.T1 must rescore its actual source paths and implement only the frozen owner
surface. P6.T2 remains blocked on T1 PASS.
