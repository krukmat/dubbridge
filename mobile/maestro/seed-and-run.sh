#!/usr/bin/env bash
# S-055 V7a — Stack bring-up + cleanup trap for the Maestro screenshot suite.
#
# Usage (from repo root):
#   bash mobile/maestro/seed-and-run.sh
#
# Prerequisites:
#   - Android emulator booted (adb devices shows a running emulator)
#   - debug APK built at mobile/android/app/build/outputs/apk/debug/app-debug.apk
#   - mock-oauth-server and mock-gateway-server are started externally, OR
#     set START_MOCK_SERVERS=1 to have this script start them
#
# Port map (must match screenshot-env.sh):
#   apps/gateway / mock-gateway  :8081
#   apps/api                     :8080
#   Metro (JS bundler)           :8082
#   mock-oauth                   :9000  (host-only; gateway contacts it directly)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APK_PATH="$REPO_ROOT/mobile/android/app/build/outputs/apk/debug/app-debug.apk"
APP_ID="com.dubbridge.mobile"
GATEWAY_URL="http://127.0.0.1:8081"
API_URL="http://127.0.0.1:8080"
METRO_PORT=8082
HEALTH_TIMEOUT=30   # seconds to wait for each service
METRO_TIMEOUT=60    # seconds to wait for Metro to be ready
START_MOCK_SERVERS="${START_MOCK_SERVERS:-0}"
# Set SKIP_METRO=1 when the APK has an embedded bundle and doesn't need Metro
SKIP_METRO="${SKIP_METRO:-0}"

# PIDs of processes started by this script — killed in cleanup
_STARTED_PIDS=()

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

die() {
  echo "[seed-and-run] ERROR: $*" >&2
  exit 1
}

info() {
  echo "[seed-and-run] $*"
}

# Wait for an HTTP endpoint to return HTTP 200.
# Usage: wait_for_http <label> <url> <timeout_seconds>
wait_for_http() {
  local label="$1" url="$2" timeout="$3"
  local elapsed=0
  info "Waiting for $label at $url (timeout ${timeout}s)..."
  while ! curl -sf --max-time 2 "$url" > /dev/null 2>&1; do
    if (( elapsed >= timeout )); then
      die "$label did not become ready at $url within ${timeout}s. Is the service running?"
    fi
    sleep 2
    (( elapsed += 2 ))
  done
  info "$label is ready."
}

# Wait for Metro bundler: poll /:8082/status until it returns "packager-status:running"
wait_for_metro() {
  local elapsed=0
  info "Waiting for Metro on :$METRO_PORT (timeout ${METRO_TIMEOUT}s)..."
  while true; do
    local status
    status=$(curl -sf --max-time 2 "http://127.0.0.1:$METRO_PORT/status" 2>/dev/null || true)
    if [[ "$status" == *"packager-status:running"* ]]; then
      info "Metro is ready on :$METRO_PORT."
      return 0
    fi
    if (( elapsed >= METRO_TIMEOUT )); then
      die "Metro did not become ready on :$METRO_PORT within ${METRO_TIMEOUT}s."
    fi
    sleep 2
    (( elapsed += 2 ))
  done
}

# ---------------------------------------------------------------------------
# Cleanup trap — runs on EXIT (normal or error)
# ---------------------------------------------------------------------------

cleanup() {
  info "Running cleanup..."
  for pid in "${_STARTED_PIDS[@]:-}"; do
    if kill -0 "$pid" 2>/dev/null; then
      info "Stopping PID $pid"
      kill "$pid" 2>/dev/null || true
    fi
  done
  # Remove any temp files written by this script
  rm -f /tmp/dubbridge-seed-output.json
  info "Cleanup done."
}

trap cleanup EXIT

# ---------------------------------------------------------------------------
# 1. Load screenshot environment (ports, env vars, SDK paths)
# ---------------------------------------------------------------------------

# shellcheck source=screenshot-env.sh
. "$REPO_ROOT/mobile/maestro/screenshot-env.sh"

# ---------------------------------------------------------------------------
# 2. Dependency checks
# ---------------------------------------------------------------------------

info "Checking dependencies..."

command -v adb    > /dev/null 2>&1 || die "'adb' not found. Install Android SDK platform-tools and add to PATH."
command -v node   > /dev/null 2>&1 || die "'node' not found. Install Node.js (>=18)."
command -v curl   > /dev/null 2>&1 || die "'curl' not found."
command -v maestro > /dev/null 2>&1 || die "'maestro' not found. Install via: brew install mobile-dev-inc/tap/maestro"

# ---------------------------------------------------------------------------
# 3. Verify Android emulator is booted and unlocked
# ---------------------------------------------------------------------------

info "Checking for a running Android emulator..."
EMULATOR_SERIAL=$(adb devices 2>/dev/null | awk '/emulator-[0-9]+[[:space:]]+device/{print $1; exit}')
if [[ -z "$EMULATOR_SERIAL" ]]; then
  die "No running Android emulator found. Start one with Android Studio or 'emulator -avd <AVD_NAME>' and try again."
fi
info "Emulator detected: $EMULATOR_SERIAL"

# Unlock screen (no-op if already unlocked)
adb -s "$EMULATOR_SERIAL" shell input keyevent 82 2>/dev/null || true

# ---------------------------------------------------------------------------
# 4. Install or refresh the debug APK
# ---------------------------------------------------------------------------

info "Checking debug APK..."
[[ -f "$APK_PATH" ]] || die "Debug APK not found at $APK_PATH. Run 'npx expo prebuild --platform android && cd mobile/android && ./gradlew assembleDebug' first."

info "Installing APK: $APK_PATH"
adb -s "$EMULATOR_SERIAL" install -r "$APK_PATH" \
  || die "APK install failed. Try 'adb uninstall $APP_ID' then re-run."

# ---------------------------------------------------------------------------
# 5. Start mock servers (optional — set START_MOCK_SERVERS=1)
# ---------------------------------------------------------------------------

if [[ "$START_MOCK_SERVERS" == "1" ]]; then
  info "Starting mock-oauth-server on :9000..."
  node "$REPO_ROOT/scripts/e2e-seed/mock-oauth-server.mjs" &
  _STARTED_PIDS+=($!)

  info "Starting mock-gateway-server on :8081..."
  node "$REPO_ROOT/scripts/e2e-seed/mock-gateway-server.mjs" &
  _STARTED_PIDS+=($!)
fi

# ---------------------------------------------------------------------------
# 6. Health checks — gateway and API must be ready before Maestro runs
# ---------------------------------------------------------------------------

wait_for_http "gateway (:8081)" "$GATEWAY_URL/health/ready" "$HEALTH_TIMEOUT"
if [[ "$START_MOCK_SERVERS" != "1" ]]; then
  wait_for_http "api (:8080)"   "$API_URL/health/live"       "$HEALTH_TIMEOUT"
fi

# ---------------------------------------------------------------------------
# 7. Start Metro on :8082 if not already running
# ---------------------------------------------------------------------------

if [[ "$SKIP_METRO" == "1" ]]; then
  info "SKIP_METRO=1 — skipping Metro (APK has embedded bundle)."
elif curl -sf --max-time 2 "http://127.0.0.1:$METRO_PORT/status" 2>/dev/null | grep -q "packager-status:running"; then
  info "Metro already running on :$METRO_PORT."
else
  info "Starting Metro on :$METRO_PORT..."
  (
    cd "$REPO_ROOT/mobile"
    EXPO_PUBLIC_E2E_ENABLED=true npx expo start --port "$METRO_PORT" --clear > /tmp/dubbridge-metro.log 2>&1
  ) &
  _STARTED_PIDS+=($!)
  wait_for_metro
fi

# ---------------------------------------------------------------------------
# 8. adb reverse — map emulator ports to host services
# ---------------------------------------------------------------------------

info "Setting adb reverse mappings..."
adb -s "$EMULATOR_SERIAL" reverse tcp:8081 tcp:8081  # gateway / mock-gateway
adb -s "$EMULATOR_SERIAL" reverse tcp:8080 tcp:8080  # apps/api
adb -s "$EMULATOR_SERIAL" reverse tcp:8082 tcp:8082  # Metro

info "adb reverse mappings set: 8081 (gateway), 8080 (api), 8082 (Metro)."

# ---------------------------------------------------------------------------
# 9. Stack is ready
# ---------------------------------------------------------------------------

info "Stack is up. Emulator: $EMULATOR_SERIAL | Gateway: $GATEWAY_URL | Metro: :$METRO_PORT"
info ""
info "---"

# ---------------------------------------------------------------------------
# V7b — Seed handoff code
# ---------------------------------------------------------------------------

info "Minting handoff code via POST $GATEWAY_URL/e2e/issue-handoff ..."
SEED_JSON=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request to $GATEWAY_URL/e2e/issue-handoff failed. Is the mock-gateway running?"

DEEPLINK=$(SEED_JSON="$SEED_JSON" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") \
  || die "Could not parse bootstrap_deeplink from seed response: $SEED_JSON"

[[ -n "$DEEPLINK" ]] || die "Seed returned an empty bootstrap_deeplink. Response: $SEED_JSON"
info "Handoff code obtained. Deeplink: ${DEEPLINK:0:60}..."

# ---------------------------------------------------------------------------
# Full Maestro suite (S-055 phases 1–2 + S-060 phases 3–5)
# ---------------------------------------------------------------------------

MAESTRO_OUT_1="/tmp/dubbridge-maestro-auth-$$"
MAESTRO_OUT_2="/tmp/dubbridge-maestro-authed-$$"
MAESTRO_OUT_3="/tmp/dubbridge-maestro-asset-list-$$"
MAESTRO_OUT_3E="/tmp/dubbridge-maestro-asset-list-empty-$$"
MAESTRO_OUT_4="/tmp/dubbridge-maestro-asset-detail-$$"
MAESTRO_OUT_5="/tmp/dubbridge-maestro-asset-ingestion-$$"
MAESTRO_OUT_5B="/tmp/dubbridge-maestro-asset-ingestion-no-rights-$$"
MAESTRO_OUT_6="/tmp/dubbridge-maestro-projects-$$"
MAESTRO_OUT_7="/tmp/dubbridge-maestro-compliance-$$"
MAESTRO_OUT_8="/tmp/dubbridge-maestro-review-$$"
mkdir -p "$MAESTRO_OUT_1" "$MAESTRO_OUT_2" "$MAESTRO_OUT_3" "$MAESTRO_OUT_3E" \
         "$MAESTRO_OUT_4" "$MAESTRO_OUT_5" "$MAESTRO_OUT_5B" "$MAESTRO_OUT_6" \
         "$MAESTRO_OUT_7" "$MAESTRO_OUT_8"

# --- S-055 Phase 1: auth surface ---

info "Phase 1 — auth surface (auth-surface.yaml)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_1" \
  "$REPO_ROOT/mobile/maestro/auth-surface.yaml" \
  || die "Phase 1 (auth-surface.yaml) failed. Check $MAESTRO_OUT_1 for details."

info "Phase 1 passed."

# --- S-055 Phase 2: authenticated audit (home screen) ---

info "Phase 2 — authenticated audit (authenticated-audit.yaml)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_2" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK" \
  "$REPO_ROOT/mobile/maestro/authenticated-audit.yaml" \
  || die "Phase 2 (authenticated-audit.yaml) failed. Check $MAESTRO_OUT_2 for details."

info "Phase 2 passed."

# --- S-060 Phase 3: asset list (SC-LIST-1 populated) ---

info "Minting handoff code for Phase 3 (asset list — populated)..."
DEEPLINK_LIST=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request for Phase 3 failed."
DEEPLINK_LIST=$(SEED_JSON="$DEEPLINK_LIST" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 3."

info "Phase 3 — asset list populated (asset-list.yaml / SC-LIST-1)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_3" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_LIST" \
  "$REPO_ROOT/mobile/maestro/asset-list.yaml" \
  || die "Phase 3 (asset-list.yaml) failed. Check $MAESTRO_OUT_3 for details."

info "Phase 3 passed."

# --- S-060 Phase 3b: asset list empty (SC-LIST-2) ---

info "Minting empty handoff code for Phase 3b (asset list — empty)..."
DEEPLINK_LIST_EMPTY=$(curl -sf --max-time 10 -X POST \
  "$GATEWAY_URL/e2e/issue-handoff?asset_seed=empty") \
  || die "Seed request for Phase 3b failed."
DEEPLINK_LIST_EMPTY=$(SEED_JSON="$DEEPLINK_LIST_EMPTY" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 3b."

info "Phase 3b — asset list empty (asset-list.yaml / SC-LIST-2)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_3E" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_LIST_EMPTY" \
  "$REPO_ROOT/mobile/maestro/asset-list.yaml" \
  || die "Phase 3b (asset-list.yaml empty) failed. Check $MAESTRO_OUT_3E for details."

info "Phase 3b passed."

# --- S-060 Phase 4: asset detail (SC-DETAIL-1) ---

info "Minting handoff code for Phase 4 (asset detail)..."
DEEPLINK_DETAIL=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request for Phase 4 failed."
DEEPLINK_DETAIL=$(SEED_JSON="$DEEPLINK_DETAIL" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 4."

info "Phase 4 — asset detail (asset-detail.yaml / SC-DETAIL-1)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_4" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_DETAIL" \
  "$REPO_ROOT/mobile/maestro/asset-detail.yaml" \
  || die "Phase 4 (asset-detail.yaml) failed. Check $MAESTRO_OUT_4 for details."

info "Phase 4 passed."

# --- S-060 Phase 5: asset ingestion upload (SC-INGEST-1) ---

info "Minting handoff code for Phase 5 (asset ingestion)..."
DEEPLINK_INGEST=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request for Phase 5 failed."
DEEPLINK_INGEST=$(SEED_JSON="$DEEPLINK_INGEST" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 5."

info "Phase 5 — asset ingestion (asset-ingestion.yaml / SC-INGEST-1)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_5" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_INGEST" \
  "$REPO_ROOT/mobile/maestro/asset-ingestion.yaml" \
  || die "Phase 5 (asset-ingestion.yaml) failed. Check $MAESTRO_OUT_5 for details."

info "Phase 5 passed."

# --- S-060 Phase 5b: asset ingestion no-rights (SC-INGEST-2) ---

info "Minting handoff code for Phase 5b (ingest no-rights — SC-INGEST-2)..."
DEEPLINK_INGEST_NR=$(curl -sf --max-time 10 -X POST \
  "$GATEWAY_URL/e2e/issue-handoff?ingest_seed=no_rights") \
  || die "Seed request for Phase 5b failed."
DEEPLINK_INGEST_NR=$(SEED_JSON="$DEEPLINK_INGEST_NR" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 5b."

info "Phase 5b — asset ingestion no-rights (asset-ingestion-no-rights.yaml / SC-INGEST-2)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_5B" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_INGEST_NR" \
  "$REPO_ROOT/mobile/maestro/asset-ingestion-no-rights.yaml" \
  || die "Phase 5b (asset-ingestion-no-rights.yaml) failed. Check $MAESTRO_OUT_5B for details."

info "Phase 5b passed."

# --- S-100 Phase 6: project list + detail (SC-ORG-1, SC-PROJECT-1, SC-MEMBER-2) ---

info "Minting handoff code for Phase 6 (project surfaces)..."
DEEPLINK_PROJECTS=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request for Phase 6 failed."
DEEPLINK_PROJECTS=$(SEED_JSON="$DEEPLINK_PROJECTS" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 6."

info "Phase 6 — project surfaces (projects.yaml / SC-ORG-1, SC-PROJECT-1)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_6" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_PROJECTS" \
  "$REPO_ROOT/mobile/maestro/projects.yaml" \
  || die "Phase 6 (projects.yaml) failed. Check $MAESTRO_OUT_6 for details."

info "Phase 6 passed."

# --- S-110 Phase 7: mobile compliance and consent ---

info "Minting handoff code for Phase 7 (compliance surfaces)..."
DEEPLINK_COMPLIANCE=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request for Phase 7 failed."
DEEPLINK_COMPLIANCE=$(SEED_JSON="$DEEPLINK_COMPLIANCE" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 7."

info "Phase 7 — compliance surfaces (compliance.yaml)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_7" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_COMPLIANCE" \
  "$REPO_ROOT/mobile/maestro/compliance.yaml" \
  || die "Phase 7 (compliance.yaml) failed. Check $MAESTRO_OUT_7 for details."

info "Phase 7 passed."

# --- S-160 Phase 8: review and publication flow (SC-REVIEW-1/2, SC-PUBLISH-1/2) ---

info "Minting handoff code for Phase 8 (review flow)..."
DEEPLINK_REVIEW=$(curl -sf --max-time 10 -X POST "$GATEWAY_URL/e2e/issue-handoff") \
  || die "Seed request for Phase 8 failed."
DEEPLINK_REVIEW=$(SEED_JSON="$DEEPLINK_REVIEW" node -e "
  const raw = process.env.SEED_JSON;
  try { const p = JSON.parse(raw); process.stdout.write(p.auth.bootstrap_deeplink); }
  catch(e) { process.exit(1); }
") || die "Could not parse bootstrap_deeplink for Phase 8."

info "Phase 8 — review and publication (review.yaml / SC-REVIEW-1/2, SC-PUBLISH-1/2)..."
maestro test \
  --test-output-dir "$MAESTRO_OUT_8" \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK_REVIEW" \
  "$REPO_ROOT/mobile/maestro/review.yaml" \
  || die "Phase 8 (review.yaml) failed. Check $MAESTRO_OUT_8 for details."

info "Phase 8 passed."

# ---------------------------------------------------------------------------
# Copy screenshots
# ---------------------------------------------------------------------------

SCREENSHOTS_DIR="$REPO_ROOT/mobile/artifacts/screenshots"
mkdir -p "$SCREENSHOTS_DIR"

info "Copying screenshots to $SCREENSHOTS_DIR ..."
find "$MAESTRO_OUT_1" "$MAESTRO_OUT_2" "$MAESTRO_OUT_3" "$MAESTRO_OUT_3E" \
     "$MAESTRO_OUT_4" "$MAESTRO_OUT_5" "$MAESTRO_OUT_5B" "$MAESTRO_OUT_6" \
     "$MAESTRO_OUT_7" "$MAESTRO_OUT_8" -name "*.png" | while IFS= read -r png; do
  cp "$png" "$SCREENSHOTS_DIR/"
  info "  Copied: $(basename "$png")"
done

PNG_COUNT=$(find "$SCREENSHOTS_DIR" -name "*.png" | wc -l | tr -d ' ')
[[ "$PNG_COUNT" -gt 0 ]] || die "No PNGs found after Maestro run. Check Maestro output dirs."
info "$PNG_COUNT screenshot(s) written to $SCREENSHOTS_DIR"

# ---------------------------------------------------------------------------
# Sanitize Maestro reports (redact handoff_code / session_ref)
# ---------------------------------------------------------------------------

info "Sanitizing Maestro reports..."

sanitize_dir() {
  local dir="$1"
  find "$dir" -name "*.json" | while IFS= read -r f; do
    # URL-encoded and plain query-param forms
    sed -i '' \
      -e 's/handoff_code=[^&" ]*/handoff_code=[REDACTED]/g' \
      -e 's/session_ref=[^&" ]*/session_ref=[REDACTED]/g' \
      "$f"
    # JSON value forms
    sed -i '' \
      -e 's/"handoff_code":"[^"]*"/"handoff_code":"[REDACTED]"/g' \
      -e 's/"session_ref":"[^"]*"/"session_ref":"[REDACTED]"/g' \
      "$f"
  done
}

sanitize_dir "$MAESTRO_OUT_1"
sanitize_dir "$MAESTRO_OUT_2"
sanitize_dir "$MAESTRO_OUT_3"
sanitize_dir "$MAESTRO_OUT_3E"
sanitize_dir "$MAESTRO_OUT_4"
sanitize_dir "$MAESTRO_OUT_5"
sanitize_dir "$MAESTRO_OUT_5B"
sanitize_dir "$MAESTRO_OUT_6"
sanitize_dir "$MAESTRO_OUT_7"
sanitize_dir "$MAESTRO_OUT_8"

# Assert absence of sensitive values post-sanitization
LEAK_HANDOFF=$(grep -r 'handoff_code=' \
  "$MAESTRO_OUT_1" "$MAESTRO_OUT_2" "$MAESTRO_OUT_3" "$MAESTRO_OUT_3E" \
  "$MAESTRO_OUT_4" "$MAESTRO_OUT_5" "$MAESTRO_OUT_5B" "$MAESTRO_OUT_6" \
  "$MAESTRO_OUT_7" "$MAESTRO_OUT_8" 2>/dev/null \
  | grep -v 'handoff_code=\[REDACTED\]' || true)
LEAK_SESSION=$(grep -r 'session_ref=' \
  "$MAESTRO_OUT_1" "$MAESTRO_OUT_2" "$MAESTRO_OUT_3" "$MAESTRO_OUT_3E" \
  "$MAESTRO_OUT_4" "$MAESTRO_OUT_5" "$MAESTRO_OUT_5B" "$MAESTRO_OUT_6" \
  "$MAESTRO_OUT_7" "$MAESTRO_OUT_8" 2>/dev/null \
  | grep -v 'session_ref=\[REDACTED\]' || true)
LEAK_SESSION_JSON=$(grep -r '"session_ref":"' \
  "$MAESTRO_OUT_1" "$MAESTRO_OUT_2" "$MAESTRO_OUT_3" "$MAESTRO_OUT_3E" \
  "$MAESTRO_OUT_4" "$MAESTRO_OUT_5" "$MAESTRO_OUT_5B" "$MAESTRO_OUT_6" \
  "$MAESTRO_OUT_7" "$MAESTRO_OUT_8" 2>/dev/null \
  | grep -v '"session_ref":"\[REDACTED\]"' || true)

if [[ -n "$LEAK_HANDOFF" || -n "$LEAK_SESSION" || -n "$LEAK_SESSION_JSON" ]]; then
  echo "[seed-and-run] SANITIZER LEAK DETECTED:" >&2
  [[ -n "$LEAK_HANDOFF" ]]      && echo "$LEAK_HANDOFF" >&2
  [[ -n "$LEAK_SESSION" ]]      && echo "$LEAK_SESSION" >&2
  [[ -n "$LEAK_SESSION_JSON" ]] && echo "$LEAK_SESSION_JSON" >&2
  die "Sanitizer missed sensitive values in Maestro reports. See above."
fi

info "Sanitization verified: no handoff_code or session_ref values remain in reports."

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

info ""
info "Suite complete — 8 phases, $(echo "$PNG_COUNT") screenshot(s)."
info "  Screenshots    : $SCREENSHOTS_DIR"
info "  Phase 1 out    : $MAESTRO_OUT_1   (01_auth_login)"
info "  Phase 2 out    : $MAESTRO_OUT_2   (02_home)"
info "  Phase 3 out    : $MAESTRO_OUT_3   (03_asset_list — populated)"
info "  Phase 3b out   : $MAESTRO_OUT_3E  (03_asset_list — empty)"
info "  Phase 4 out    : $MAESTRO_OUT_4   (04_asset_detail)"
info "  Phase 5 out    : $MAESTRO_OUT_5   (05_upload → 06_ingest_complete)"
info "  Phase 5b out   : $MAESTRO_OUT_5B  (07_ingest_no_rights — SC-INGEST-2)"
info "  Phase 6 out    : $MAESTRO_OUT_6   (08_home_for_projects → 09_project_list → 10_project_detail)"
info "  Phase 7 out    : $MAESTRO_OUT_7   (11_compliance_center → 12_consent_active → 13_consent_revoked)"
info "  Phase 8 out    : $MAESTRO_OUT_8   (14_review_inbox → 15_review_detail → 16_review_approved → 17_review_published)"
