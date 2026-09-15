---
type: Playbook
title: "P5.T3 Android device certification"
status: active
---

# P5.T3 Android device certification

Purpose: execute the remaining P5 device evidence without waiting for P6 product screens. This playbook uses the development-only P5 certification harness and the production P3/P4/P5 seams. It does not create a mock content key, alternate decrypt path or remote media fallback.

## Preconditions

- Branch: `feature/p2p-mvp-core` at the exact revision being certified.
- Android emulator/device available and the app can reach the configured DubBridge gateway.
- Backend has one owner-ready P2P publication and a fresh one-time viewer invitation.
- The viewer account used in the app is the intended invitation recipient.
- JDK/Android build environment is already working; do not run `expo prebuild --clean` or an unnecessary Gradle `clean` as part of certification.

## Launch

From `mobile/`:

```bash
npm run android:p2p-cert
```

Normal builds do not show the harness. With the certification flag enabled, log in as the intended viewer through the regular app. After authentication, the development-only `P5 device certification` surface appears.

## Happy path

1. Paste a fresh one-time P2P invitation token into the secure field.
2. Press `Run P5 certification`.
3. The harness executes the production path in order: `claim -> sync -> verify -> playback`.
4. PASS requires the short video to render through the existing `VideoPlayer` from a `127.0.0.1` P5 loopback session.
5. Press `Stop certification playback` and verify playback stops cleanly.

The harness deliberately reports stable stage/failure codes only. Do not add raw envelope, CK, access token, invitation token or runtime payload logging to troubleshoot a failure.

## Required evidence

Record the exact commit SHA, Android device/emulator identity, gateway/backend profile, redacted owner/viewer aliases, and actual result for each control below.

| Control | Required result |
|---|---|
| Valid invitation + verified package | Video plays through local loopback using existing player |
| Remote audience-media unavailable | Valid local playback still succeeds; no HTTP/S3 substitution |
| Stop/unmount | Loopback session/listener and transient playback ownership are released |
| Key/gateway failure | Playback fails closed; no fallback URL is substituted |
| Tampered/incomplete package | Verification/playback is denied |
| Review playback | Existing ADR-032 review path remains functional and independent |

Screenshots are supporting evidence only. Runtime/network/device evidence is required for loopback-only delivery, teardown, tamper and secret-boundary claims.

## Failure codes

- `CLAIM_FAILED`: invitation/device authorization did not complete.
- `SYNC_FAILED`: P4 package replication/sync did not complete.
- `VERIFICATION_FAILED`: P4 did not produce a verified package handle.
- `PLAYBACK_FAILED`: authorization/K1 unwrap/P5 session startup failed.
- `STOP_FAILED`: explicit playback teardown failed.
- `CONFIG_INVALID`, `AUTH_REQUIRED`, `INVITE_REQUIRED`: local harness setup/input issue.

A failure code is diagnostic classification, not a waiver. Capture the owning subsystem evidence and return the defect to P3, P4 or P5 as applicable.

## PASS boundary

P5 can move from `READY FOR DEVICE TEST` to PASS only after the required T3 executable/device evidence is recorded under `docs/audit/` and the phase/parent status artifacts are synchronized. A successful harness launch or runtime ping alone is insufficient.

P7Local later reuses the same underlying capabilities as part of the complete owner-to-viewer product flow; it does not replace this P5 phase evidence.
