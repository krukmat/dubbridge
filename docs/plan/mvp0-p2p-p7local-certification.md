---
type: Plan
title: "P7Local: local end-to-end P2P certification"
status: planned
slice: MVP0-P2P
---

# P7Local — local end-to-end P2P certification

P7Local is a pre-release certification stage executed **after P6 PASS** and before the final P7 exact-artifact/deployment certification. It does not replace P5.T3 device certification and it does not relax P7.

## Objective

Prove the complete mobile product journey locally on Android using the production P3/P4/P5/P6 seams:

`owner ready content -> invite -> viewer claim -> verified P2P sync -> Available -> loopback local playback -> deterministic teardown`.

The purpose is to catch product/integration defects before the same journey is repeated against the exact release candidate in P7.

## Preconditions

- P3, P4, P5 and P6 are PASS.
- Android candidate is built from a recorded commit SHA.
- Local backend exposes the same P2P audience contracts consumed by mobile.
- A real short encrypted publication exists and is reachable through Hyperdrive/Hyperswarm.
- Audience-media HTTP/S3 fallback is disabled for the certification run.
- Owner and viewer accounts are distinct.

## Required scenarios

### Happy path

1. Owner sees the publication as Ready in My Content.
2. Owner creates a one-time invitation.
3. Viewer claims the invitation on Android.
4. Viewer sync progresses through discovery/download/verification.
5. UI becomes Available only after manifest/package/progress verification is complete.
6. Play uses the P5 loopback URL and existing VideoPlayer.
7. Stop tears down playback and a second start can create a fresh session.

### Fail-closed checks

- Expired/revoked/invalid invitation cannot claim.
- Incomplete or unverified local data cannot expose Play.
- Altered ciphertext/AAD/manifest fails before media is served.
- Missing/failed device-envelope unwrap stops playback; no remote-media substitution.
- Account switch/sign-out removes prior account P2P state/action ownership.
- A viewer cannot see or play another viewer's invitation/session.

## Evidence pack

Record only non-secret evidence:

- repository commit SHA and APK SHA-256;
- backend/local stack identity and relevant configuration hashes;
- owner/viewer test account identifiers in redacted form;
- publication/invitation/authorization IDs, never invitation token or CK;
- stage/result log for invite, claim, sync, verify, play and teardown;
- Android screenshots for user-visible state transitions;
- network evidence showing media delivery through `127.0.0.1` and no audience-media HTTP/S3 fallback;
- negative-case result codes and teardown outcome.

Raw CK, wrapped key material, invitation token, private-key material and device-envelope plaintext must never be captured in logs or evidence.

## PASS criteria

P7Local is PASS only when the happy path and the required fail-closed checks succeed on one recorded Android candidate and all evidence is attributable to the same run. Any workaround that substitutes remote audience media, bypasses verification or skips account isolation is a FAIL.

P7Local PASS is preparation for P7 only. P7 must repeat the release-critical journey against the exact deploy/backend and Android release candidate used for the final gate.
