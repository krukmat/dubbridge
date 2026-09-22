---
type: AuditEvidence
title: "P6.T1 Block 2 — Create Invite and one-time Copy"
status: pass
slice: MVP0-P2P
task: P6.T1.D+E
date: 2026-09-22
---

# P6.T1 Block 2 — Create Invite + one-time Copy evidence

## Scope

This evidence closes only the second P6.T1 implementation block:

- **T1.D — P3 Create Invite integration**
- **T1.E — one-time raw token + Copy Invite**

Navigation (T1.F), remaining aggregate tests/evidence (T1.G), and T1 certification /
owner verification (T1.H) remain out of scope and pending.

## Delivered behavior

- `MyContentScreen` invokes the existing P3 owner invitation contract through
  `P2PAudienceService.createInvitation`.
- The request uses the existing
  `POST /api/assets/{asset_id}/p2p/invitations` backend route; no parallel invite
  API or client-side authorization path was introduced.
- The UI continues to expose Create Invite only for an exact backend-authoritative
  Ready descriptor.
- A `403`, `404`, or `409` on creation triggers an owner-content refresh so stale
  Ready state cannot remain the sole basis for eligibility.
- `session_expired` on creation follows the existing logout boundary.
- The raw invitation token exists only in volatile screen state after a successful
  create response. It is not written to AsyncStorage, SecureStore, filesystem,
  logs, or another durable store.
- Only one visible raw token can be outstanding in the screen at a time; additional
  Create actions remain inert until the token is dismissed.
- Copy uses `expo-clipboard` compatible with Expo SDK 56.
- Dismiss/unmount clears the only UI-held raw-token reference; remount does not
  reconstruct the token.

## Security / authority notes

P3 remains authoritative. The server already hashes the invitation token before
persistence and returns the raw token only in the successful create response.
P6 does not weaken that boundary and does not treat UI visibility as authorization.

No HPKE emulator path was enabled or changed by this block.

## Validation

Final implementation head:

`151721a56b9cc4f3d970693c9ba150c16abfff1a`

GitHub Actions run:

`35786712119`

Result:

- **15/15 jobs PASS**
- mobile: **63/63 suites PASS**
- mobile: **456/456 tests PASS**
- `MyContentScreen.test.tsx`: PASS
- P3 T2c3 local-certification harness check: PASS
- workspace line coverage: **90.43%**
- maintainability: PASS
- fmt / clippy / cargo-check / release-build / deny / config-secrets / qa-docs /
  peer-workflow-review / s3-integration / python-complexity: PASS

## Failure / correction trace

Two intermediate heads were intentionally not accepted:

- `fdbca34e`: mobile lint rejected an overlong `useMyContentInvite` function.
  The hook was decomposed without changing scope.
- `6b064a38`: production lint/type checks were clean, but the new screen tests had
  an un-awaited React `act()` transition on token dismissal; that contaminated
  subsequent tests. Test synchronization was corrected.
- `151721a5`: exact implementation head passed the full CI matrix.

No failed intermediate head is used as closure evidence.

## Behavioral mapping

**HP-P6.T1-1:** Owner sees authoritative Ready content and can create a real P3
invite. The successful response exposes a one-time token that can be copied.

**EC-P6.T1-1:** Non-ready / descriptor-mismatched content still exposes no action,
and a stale Ready action rejected by P3 refreshes authoritative content. Server
rejection therefore remains fail-closed.

Additional one-time-token evidence verifies that remounting the screen cannot
reconstruct the raw token and that no create request is replayed automatically.

## Status

- T1.A — PASS
- T1.B — PASS
- T1.C — PASS
- **T1.D — PASS**
- **T1.E — PASS**
- T1.F — Pending
- T1.G — Pending
- T1.H — Pending

**P6.T1 remains IN PROGRESS.**
