#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MODULE_PATH=":dubbridge-p2p-keystore"
MODULE_DIR="$ROOT/mobile/modules/dubbridge-p2p-keystore/android"
ANDROID_DIR="$ROOT/mobile/android"
WORKFLOW="$ROOT/.github/workflows/p3-t2-native.yml"
INSTRUMENTATION_TEST="$MODULE_DIR/src/androidTest/java/com/dubbridge/p2pkeystore/DubBridgeP2PKeyStoreInteropTest.kt"
JS_TEST="$ROOT/mobile/__tests__/p2p/device-identity.test.ts"
TMP_RESULT_DIR="$ROOT/tmp/p3-t2c3"

fail() {
  echo "P3_T2C3_RESULT=FAIL"
  echo "P3_T2C3_REASON=$1"
  exit 1
}

self_check() {
  test -f "$INSTRUMENTATION_TEST" || fail "instrumentation test missing"
  test -f "$JS_TEST" || fail "JS native-only test missing"
  test -f "$WORKFLOW" || fail "disabled GitHub workflow missing"

  grep -Fq "bindingValidationRejectsExpiryAndIdentityDrift" "$INSTRUMENTATION_TEST"     || fail "binding/expiry instrumentation case missing"
  grep -Fq "opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport" "$INSTRUMENTATION_TEST"     || fail "opaque-key HPKE instrumentation case missing"
  grep -Fq "has no software fallback when native methods are absent" "$JS_TEST"     || fail "JS no-software-fallback evidence missing"
  grep -Fq "workflow_dispatch:" "$WORKFLOW"     || fail "native workflow must remain manual-only"
  grep -Fq 'if: ${{ false }}' "$WORKFLOW"     || fail "native GitHub emulator workflow must remain hard-disabled"

  echo "P3_T2C3_SELF_CHECK=PASS"
}

if [ "${1:-}" = "--self-check" ]; then
  self_check
  exit 0
fi

self_check

cd "$ROOT"

EXPECTED_BRANCH="${P3_T2C3_BRANCH:-feature/p2p-mvp-core}"
CURRENT_BRANCH="$(git symbolic-ref --quiet --short HEAD || true)"
[ "$CURRENT_BRANCH" = "$EXPECTED_BRANCH" ]   || fail "expected branch $EXPECTED_BRANCH, found ${CURRENT_BRANCH:-detached}"

if [ -n "$(git status --porcelain --untracked-files=no)" ] && [ "${P3_T2C3_ALLOW_DIRTY:-0}" != "1" ]; then
  fail "tracked working tree is dirty; commit/stash before certification"
fi

TESTED_COMMIT="$(git rev-parse HEAD)"

if [ "$(uname -s)" = "Darwin" ] && [ -x /usr/libexec/java_home ]; then
  JAVA17_HOME="$(/usr/libexec/java_home -v 17 2>/dev/null || true)"
  if [ -n "$JAVA17_HOME" ]; then
    export JAVA_HOME="$JAVA17_HOME"
    export PATH="$JAVA_HOME/bin:$PATH"
  fi
fi

command -v java >/dev/null 2>&1 || fail "Java is unavailable"
JAVA_LINE="$(java -version 2>&1 | head -n 1)"
printf '%s' "$JAVA_LINE" | grep -Eq '"17([."]|$)'   || fail "JDK 17 is required; found: $JAVA_LINE"

command -v node >/dev/null 2>&1 || fail "Node.js is unavailable"
command -v npm >/dev/null 2>&1 || fail "npm is unavailable"
command -v python3 >/dev/null 2>&1 || fail "python3 is unavailable"

ADB=""
if command -v adb >/dev/null 2>&1; then
  ADB="$(command -v adb)"
elif [ -n "${ANDROID_HOME:-}" ] && [ -x "$ANDROID_HOME/platform-tools/adb" ]; then
  ADB="$ANDROID_HOME/platform-tools/adb"
elif [ -x "$HOME/Library/Android/sdk/platform-tools/adb" ]; then
  ADB="$HOME/Library/Android/sdk/platform-tools/adb"
fi
[ -n "$ADB" ] || fail "adb not found; install/configure Android SDK platform-tools"

DEVICES="$("$ADB" devices | awk 'NR > 1 && $2 == "device" { print $1 }')"
DEVICE_COUNT="$(printf '%s\n' "$DEVICES" | sed '/^$/d' | wc -l | tr -d ' ')"
[ "$DEVICE_COUNT" = "1" ]   || fail "exactly one booted Android device/emulator must be connected via adb; found $DEVICE_COUNT"
SERIAL="$(printf '%s\n' "$DEVICES" | sed '/^$/d' | head -n 1)"

BOOTED="$("$ADB" -s "$SERIAL" shell getprop sys.boot_completed | tr -d '\r')"
[ "$BOOTED" = "1" ] || fail "connected Android target is not fully booted"

API_LEVEL="$("$ADB" -s "$SERIAL" shell getprop ro.build.version.sdk | tr -d '\r')"
case "$API_LEVEL" in
  ''|*[!0-9]*) fail "could not determine Android API level" ;;
esac
[ "$API_LEVEL" -ge 31 ] || fail "Android API 31+ required; found API $API_LEVEL"

DEVICE_MODEL="$("$ADB" -s "$SERIAL" shell getprop ro.product.model | tr -d '\r\n')"
if command -v shasum >/dev/null 2>&1; then
  DEVICE_REF="$(printf '%s' "$SERIAL" | shasum -a 256 | awk '{print substr($1,1,12)}')"
else
  DEVICE_REF="$(printf '%s' "$SERIAL" | sha256sum | awk '{print substr($1,1,12)}')"
fi

mkdir -p "$TMP_RESULT_DIR"
RUN_LOG="$TMP_RESULT_DIR/instrumentation.log"
RESULT_JSON="$TMP_RESULT_DIR/result.json"
rm -f "$RUN_LOG" "$RESULT_JSON"

if [ ! -x "$ROOT/mobile/node_modules/.bin/expo" ]; then
  echo "Installing mobile dependencies..."
  (cd "$ROOT/mobile" && npm ci)
fi

if [ "${P3_T2C3_SKIP_PREBUILD:-0}" != "1" ]; then
  echo "Generating Android project..."
  (cd "$ROOT/mobile" && CI=1 npx expo prebuild --platform android --no-install --clean)
fi

[ -x "$ANDROID_DIR/gradlew" ] || fail "mobile/android/gradlew missing after prebuild"

cd "$ANDROID_DIR"
./gradlew projects --console=plain | grep -Fq "Project '$MODULE_PATH'"   || fail "Gradle subproject $MODULE_PATH is not registered"

echo "Building K1 module instrumentation APKs..."
./gradlew "$MODULE_PATH:clean" "$MODULE_PATH:assembleDebug" "$MODULE_PATH:assembleDebugAndroidTest" --stacktrace

echo "Running T2c3 instrumentation on the connected Android target..."
set +e
./gradlew "$MODULE_PATH:connectedDebugAndroidTest" --stacktrace 2>&1 | tee "$RUN_LOG"
GRADLE_STATUS=${PIPESTATUS[0]}
set -e

RESULT_ROOT="$MODULE_DIR/build/outputs/androidTest-results/connected"
RESULT_XML="$(find "$RESULT_ROOT" -type f -name 'TEST-*.xml' 2>/dev/null | head -n 1 || true)"

if [ "$GRADLE_STATUS" -ne 0 ]; then
  cat > "$RESULT_JSON" <<JSON
{"status":"FAIL","tested_commit":"$TESTED_COMMIT","reason":"gradle_instrumentation_failed","api_level":$API_LEVEL,"device_model":"$DEVICE_MODEL","device_ref_sha256":"$DEVICE_REF"}
JSON
  fail "connectedDebugAndroidTest failed; inspect $RUN_LOG"
fi

[ -n "$RESULT_XML" ] || fail "instrumentation finished without a JUnit XML result"

SUMMARY="$(python3 - "$RESULT_XML" <<'PY'
import sys
import xml.etree.ElementTree as ET

path = sys.argv[1]
root = ET.parse(path).getroot()
suites = [root] if root.tag == "testsuite" else list(root.findall(".//testsuite"))
tests = sum(int(s.attrib.get("tests", "0")) for s in suites)
failures = sum(int(s.attrib.get("failures", "0")) for s in suites)
errors = sum(int(s.attrib.get("errors", "0")) for s in suites)
skipped = sum(int(s.attrib.get("skipped", "0")) for s in suites)
names = {tc.attrib.get("name", "") for tc in root.findall(".//testcase")}
expected = {
    "bindingValidationRejectsExpiryAndIdentityDrift",
    "opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport",
}
expected_present = 1 if expected.issubset(names) else 0
print(f"{tests}|{failures}|{errors}|{skipped}|{expected_present}")
PY
)"
IFS='|' read -r TESTS FAILURES ERRORS SKIPPED EXPECTED_PRESENT <<< "$SUMMARY"

[ "$TESTS" -ge 2 ] || fail "instrumentation discovered fewer than two tests"
[ "$FAILURES" = "0" ] || fail "instrumentation reported $FAILURES failures"
[ "$ERRORS" = "0" ] || fail "instrumentation reported $ERRORS errors"
[ "$EXPECTED_PRESENT" = "1" ] || fail "required T2c3 test cases were not present in JUnit output"

RUN_DATE="$(date +%Y-%m-%d)"
EVIDENCE_REL="docs/audit/mvp0-p2p-p3-t2c3-android-certification-$RUN_DATE.md"
EVIDENCE="$ROOT/$EVIDENCE_REL"

cat > "$EVIDENCE" <<EOF
---
type: Audit
title: "MVP0-P2P P3.T2c3 Android Keystore / HPKE local certification"
task: P3.T2
leaf: T2c3
date: $RUN_DATE
status: pass
---

# P3.T2c3 — Android Keystore opaque-key HPKE certification

## Result

**PASS** against tested commit `$TESTED_COMMIT`.

This evidence was produced by the local one-shot runner
`scripts/p3-t2c3-certify-android.sh`. The GitHub emulator workflow remained
hard-disabled and was not used as certification evidence.

## Android target

- model: `$DEVICE_MODEL`
- Android API: `$API_LEVEL`
- device reference: SHA-256 prefix `$DEVICE_REF` (raw ADB serial intentionally omitted)

## Executed instrumentation

Gradle module: `$MODULE_PATH`

JUnit result:

- tests: $TESTS
- failures: $FAILURES
- errors: $ERRORS
- skipped: $SKIPPED

Required cases executed:

1. `bindingValidationRejectsExpiryAndIdentityDrift`
   - binding JSON is parsed before unwrap;
   - profile/device identity drift fails closed;
   - missing authorization identity fails closed;
   - `expires_at_unix <= now` fails closed.

2. `opaqueAndroidKeystoreKeyCompletesHpkeInteropWithoutExport`
   - P-256 K1 private key is generated/loaded through `AndroidKeyStore`;
   - `privateKey.encoded == null` proves the private key is not exportable through the API;
   - native ECDH + RFC9180-compatible HPKE derivation + AES-256-GCM unwrap recovers the expected CK;
   - the CK is used only as transient test material and is not persisted/logged.

## T2c3 acceptance mapping

- **HP-P3.T2-1 native half:** PASS — opaque Android K1 completes the package-bound HPKE unwrap.
- **EC-P3.T2-1 native half:** PASS — expired or identity-drifted binding is rejected before unwrap; no software private-key fallback participates.

This closes the execution evidence for T2c3 only. Aggregate P3.T2 status still
requires repository owner verification/status synchronization.
EOF

cat > "$RESULT_JSON" <<JSON
{"status":"PASS","tested_commit":"$TESTED_COMMIT","evidence":"$EVIDENCE_REL","tests":$TESTS,"failures":$FAILURES,"errors":$ERRORS,"skipped":$SKIPPED,"api_level":$API_LEVEL,"device_model":"$DEVICE_MODEL","device_ref_sha256":"$DEVICE_REF"}
JSON

echo "P3_T2C3_RESULT=PASS"
echo "P3_T2C3_TESTED_COMMIT=$TESTED_COMMIT"
echo "P3_T2C3_EVIDENCE=$EVIDENCE_REL"
echo "P3_T2C3_RESULT_JSON=tmp/p3-t2c3/result.json"
