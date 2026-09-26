---
type: Audit
title: "MVP0-P2P P3.T2c3 local Android certification runbook"
task: P3.T2
leaf: T2c3
date: 2026-09-22
status: ready
---

# P3.T2c3 — local Android certification runbook

## Why this path exists

The GitHub-hosted emulator attempts did not demonstrate an HPKE or AndroidKeyStore
defect. One attempt failed in workflow shell orchestration; the later attempt
built the module/test APKs but the hosted emulator ran zero tests and aborted
instrumentation after a system crash.

The owner therefore keeps `.github/workflows/p3-t2-native.yml` hard-disabled.
T2c3 is certified through a one-shot local Android target instead.

## Runner

```bash
bash scripts/p3-t2c3-certify-android.sh --self-check
bash scripts/p3-t2c3-certify-android.sh
```

The execution runner:

1. requires `feature/p2p-mvp-core` and a clean tracked working tree;
2. requires JDK 17, Node/npm, Python 3 and Android platform-tools;
3. requires exactly one already-booted Android target on ADB, API 31+;
4. generates the ignored Expo Android project locally;
5. targets the known Gradle subproject `:dubbridge-p2p-keystore`;
6. builds the module and instrumentation APK;
7. executes only the module's `connectedDebugAndroidTest`;
8. rejects a zero-test run or missing required T2c3 test names;
9. on PASS writes a redacted audit artifact under `docs/audit/` and a machine-readable summary under ignored `tmp/p3-t2c3/result.json`.

The runner never prints or stores CK, KEK, private-key bytes, JWTs, invitation
tokens or raw ADB serials.

## Certification boundary

T2c3 PASS requires both instrumentation cases to execute successfully:

- `bindingValidationRejectsExpiryAndIdentityDrift`;
- `opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport`.

A build-only result, zero discovered tests, JVM-only crypto test, or hosted
emulator crash is **not** T2c3 PASS.

After a successful run, commit only the generated audit evidence and any
status-sync documents. Do not re-enable the GitHub emulator workflow unless the
owner explicitly requests it.
