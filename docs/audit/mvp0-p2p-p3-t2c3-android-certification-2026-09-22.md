---
type: Audit
title: "MVP0-P2P P3.T2c3 Android Keystore / HPKE local certification"
task: P3.T2
leaf: T2c3
date: 2026-09-22
status: pass-owner-verified
---

# P3.T2c3 — Android Keystore opaque-key HPKE certification

## Result

**PASS** against tested commit `d14ff8b6646bde4e635d4c2d5bc6e4ff784615e8`.

This evidence was produced by the local one-shot runner
`scripts/p3-t2c3-certify-android.sh`. The GitHub emulator workflow remained
hard-disabled and was not used as certification evidence.

## Android target

- model: `sdk_gphone64_arm64`
- Android API: `34`
- device reference: SHA-256 prefix `04ab3fc382bf` (raw ADB serial intentionally omitted)

## Executed instrumentation

Gradle module: `:dubbridge-p2p-keystore`

JUnit result:

- tests: 2
- failures: 0
- errors: 0
- skipped: 0

Required cases executed:

1. `bindingValidationRejectsExpiryAndIdentityDriftBeforeUnwrap`
   - binding JSON is parsed before unwrap;
   - device identity drift fails closed;
   - missing authorization identity fails closed;
   - `expires_at_unix <= now` fails closed.

2. `opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport`
   - P-256 K1 private key is generated/loaded through `AndroidKeyStore`;
   - `privateKey.encoded == null` proves the private key is not exportable through the API;
   - native ECDH + RFC9180-compatible HPKE derivation + AES-256-GCM unwrap recovers the expected CK;
   - the CK is used only as transient test material and is not persisted/logged.

## T2c3 acceptance mapping

AndroidKeyStore P-256 → privateKey.encoded == null → native ECDH →
HPKE P-256 / HKDF-SHA256 / AES-256-GCM unwrap → CK recovered transiently.

Binding JSON parsed → profile/device/package/auth IDs valid →
expires_at_unix > now → unwrap allowed.

- **HP-P3.T2-1 native half:** PASS — opaque Android K1 completes the package-bound HPKE unwrap.
- **EC-P3.T2-1 native half:** PASS — expired or identity-drifted binding is rejected before unwrap; no software private-key fallback participates.

This closes the execution evidence for T2c3. Aggregate P3.T2 subsequently received explicit owner verification and final 15/15 CI PASS on `56e9412abf73bffefedff80300a9b9ddad15072e`.

## Harness correction and verification

The initial run at `1578e7d7b7727d83b9a3fed3684800bdc8c9ac22` executed
2 tests with 0 failures/errors/skipped, but the harness rejected the binding
test because its expected name omitted the existing `BeforeUnwrap` suffix.
Commit `d14ff8b6646bde4e635d4c2d5bc6e4ff784615e8` aligns the harness to the
exact existing method name. Product crypto, test bodies and assertions are
unchanged. This is an environment/harness correction, not a crypto fix.

The correction also escapes Markdown backticks, includes actual test names
in `result.json`, rejects skipped execution, and redacts target identifiers
from retained instrumentation logs and generated Gradle reports.

Verification: `bash -n scripts/p3-t2c3-certify-android.sh`, the runner's
`--self-check`, and the complete runner all passed. Focused checks confirmed
literal Markdown rendering; rejection of zero/missing/skipped/failing/error
JUnit results; and identifier redaction with protobuf field lengths retained.
Final inspection found no raw target identifier in retained runner artifacts,
module instrumentation reports or this audit. RRI for the bounded harness
correction: **25 / Low**; no product/security invariant changed.

## Machine-readable result

```json
{
  "status": "PASS",
  "tested_commit": "d14ff8b6646bde4e635d4c2d5bc6e4ff784615e8",
  "evidence": "docs/audit/mvp0-p2p-p3-t2c3-android-certification-2026-09-22.md",
  "tests": 2,
  "failures": 0,
  "errors": 0,
  "skipped": 0,
  "test_names": [
    "bindingValidationRejectsExpiryAndIdentityDriftBeforeUnwrap",
    "opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport"
  ],
  "api_level": 34,
  "device_model": "sdk_gphone64_arm64",
  "device_ref_sha256": "04ab3fc382bf"
}
```

## Aggregate boundary

T2c1 PASS; T2c2 PASS; T2c3 PASS.
Final documentation/evidence head `56e9412abf73bffefedff80300a9b9ddad15072e`: **15/15 CI PASS**.
Owner verification: **PASS 2026-09-22**.

**P3.T2 PASS.** P3.T3 is unblocked but not activated.
