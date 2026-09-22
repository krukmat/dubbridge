---
type: Audit
title: "MVP0-P2P P3.T3d closure and downstream handoff"
task: P3.T3
leaf: T3d
date: 2026-09-22
status: pass
---

# P3.T3d — closure and downstream handoff

## Closure result

**PASS. P3.T3 closed. Aggregate P3 = PASS.**

Owner verification is explicit in the 2026-09-22 instruction to close the task.
No runtime code changed in T3d; this leaf closes the evidence map and publishes
the already-implemented descriptor/native-adapter handoff.

The immediately preceding evidence head `d5644b0b` completed **15/15 CI PASS**.
The implementation head for T3c, `6c3a565c`, also completed **15/15 CI PASS**,
with T3a/T3b/T3c **3/3 PASS** under both normal tests and coverage; workspace
line coverage remained **90.43%**.

## Parent P3 acceptance map

| Parent criterion | Evidence | Result |
|---|---|---|
| HP-1 owner creates invite for Ready content; raw token once; hash-only persistence | T1 implementation evidence + T3a integration + T3c storage inspection | PASS |
| HP-2 eligible viewer claims and receives descriptor/K1 envelope; same viewer/device repeat is idempotent | T1 claim/idempotency evidence + T3a integration | PASS |
| HP-3 inbox exposes only authenticated viewer invitations; raw token no longer required | T1 implementation evidence | PASS |
| EC-1 expired/unknown/foreign claim fails closed | T1 fail-closed/race evidence | PASS |
| EC-2 raw token and plaintext CK are never persisted/logged | T3c secret-boundary certification | PASS |
| EC-3 non-Ready content or non-owner invite is rejected before descriptor/key material | T1 authorization/readiness evidence | PASS |

## P3.T3 acceptance map

### HP-P3.T3-1

Owner create → viewer claim → O3 authorization → package-bound device envelope
is exercised by T3a. Native unwrap and exact binding/expiry validation are
covered by the already-closed T2c2/T2c3 Android Keystore certification.

Result: **PASS**.

### EC-P3.T3-1

T3b proves that wrong viewer, missing/revoked/expired authorization, revoked
device, non-ready publication and missing durable delivery evidence produce no
device envelope. T3c additionally certifies the secret boundary.

Result: **PASS**.

## Claim-race coverage

T1a exercises concurrent different-viewer claim contention. The frozen
single-winner/idempotent-winner contract is therefore covered before aggregate
P3 closure.

Result: **PASS**.

## Downstream handoff to P4

The control-plane handoff is `P2pClaim.descriptor` /
`P2pReadyDescriptor` in `mobile/src/api/p2p.ts`.

The descriptor carries:

- `assetId`;
- `publicationId`;
- `lineageId`;
- `manifestVersion`;
- `manifestDigestSha256`;
- `externalPublicationId`;
- `ckWrapRef`;
- KEK identity/version metadata;
- Ready timestamp.

P4 consumes this descriptor through `P2pProductSync.sync(...)`, binds cache
identity to account/publication/lineage, verifies the manifest against the
descriptor, copies ciphertext only, verifies the complete package, and only
then transitions to `READY`.

**P4 activation gate is satisfied by P3 PASS.**

## Downstream handoff to P5

The authorization/native-key handoff is:

`P2PAudienceService.getTransientContentKey`
→ `AndroidKeystoreDeviceIdentity.unwrapEnvelope`
→ transient CK
→ `P2PPlaybackController.start`.

Before playback the controller re-reads current O3 authorization and checks
asset/publication/lineage/viewer/expiry against the verified package handle.
The CK is transient session material, is not persisted/logged by the audience
service, and its JS reference is cleared in the playback controller finally
block. Native private-key custody remains Android Keystore-only.

This handoff does not claim P5.T3 physical certification.

## Secret / emulator constraint

GitHub-hosted Android HPKE emulator execution remains hard-disabled. P3 closure
uses the already-recorded local Android T2c3 certification and does not re-enable
that workflow.

## Final state

- P3.T0 — PASS
- P3.T1 — PASS
- P3.T2 — PASS
- P3.T3a — PASS
- P3.T3b — PASS
- P3.T3c — PASS
- P3.T3d — PASS
- **P3 aggregate — PASS**

Next formal dependency: **P4.T3 closure / aggregate P4 PASS**.
