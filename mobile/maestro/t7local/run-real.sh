#!/usr/bin/env bash
# S-230-T7local B3->C4 real-stack Maestro runner.
# It never edits product/source state directly and never writes credentials to evidence.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
t7_dir="$repo_root/mobile/maestro/t7local"
gateway="${T7LOCAL_HOST_GATEWAY_URL:-http://localhost:8082}"
run_id="${T7LOCAL_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
proof="${T7LOCAL_PROOF_REFERENCE:-t7local-$run_id}"
filename="${T7LOCAL_FILENAME:-t7local-$run_id.mp4}"
owner="${T7LOCAL_OWNER:-DubBridge T7local}"
source_lang="${T7LOCAL_SOURCE_LANG:-en}"
target_lang="${T7LOCAL_TARGET_LANG:-es-ES}"
summary_dir="${T7LOCAL_OUTPUT_DIR:-/tmp/dubbridge-t7local-$run_id}"

die() {
  echo "T7LOCAL_MAESTRO=BLOCKED $*" >&2
  exit 1
}

json_value() {
  local key="$1"
  python3 -c 'import json,sys; d=json.load(sys.stdin); v=d.get(sys.argv[1]); print("" if v is None else v)' "$key"
}

json_object() {
  python3 -c 'import json,sys; print(json.dumps(dict(zip(sys.argv[1::2],sys.argv[2::2]))))' "$@"
}

auth_json() {
  local payload="$1"
  curl -fsS -X POST "$gateway/auth/login"     -H "content-type: application/json"     -d "$payload"
}

api_json() {
  local method="$1" path="$2" payload="${3:-}"
  if [[ -n "$payload" ]]; then
    curl -fsS -X "$method" "$gateway$path"       -H "authorization: Bearer $token"       -H "content-type: application/json"       -d "$payload"
  else
    curl -fsS -X "$method" "$gateway$path"       -H "authorization: Bearer $token"
  fi
}

for command in curl python3 adb maestro docker-compose; do
  command -v "$command" >/dev/null 2>&1 || die "missing dependency: $command"
done

[[ "$(git -C "$repo_root" rev-parse --abbrev-ref HEAD)" == "feature/p2p-mvp-core" ]] ||
  die "wrong branch"
git -C "$repo_root" diff --quiet || die "working tree has unstaged changes"
git -C "$repo_root" diff --cached --quiet || die "working tree has staged changes"
[[ "${EXPO_PUBLIC_E2E_ENABLED:-false}" != "true" ]] || die "EXPO_PUBLIC_E2E_ENABLED must be disabled"

curl -fsS "$gateway/health/ready" >/dev/null || die "gateway not ready at $gateway"

email_supplied=0
password_supplied=0
[[ -n "${T7LOCAL_EMAIL:-}" ]] && email_supplied=1
[[ -n "${T7LOCAL_PASSWORD:-}" ]] && password_supplied=1
[[ "$email_supplied" -eq "$password_supplied" ]] || die "provide both T7LOCAL_EMAIL and T7LOCAL_PASSWORD, or neither"

if [[ "$email_supplied" -eq 1 ]]; then
  email="$T7LOCAL_EMAIL"
  password="$T7LOCAL_PASSWORD"
else
  email="t7local-$run_id@dubbridge.dev"
  password="T7local-$run_id-A9zQ!"
  register_payload="$(json_object email "$email" password "$password" workspaceName "T7local-$run_id")"
  curl -fsS -X POST "$gateway/auth/register"     -H "content-type: application/json"     -d "$register_payload" >/dev/null ||
    die "real account registration failed"
fi

login_payload="$(json_object email "$email" password "$password")"
login_response="$(auth_json "$login_payload")" || die "real account login failed"
token="$(printf '%s' "$login_response" | json_value token)"
[[ -n "$token" ]] || die "login response missing token"

# A registered workspace is not itself an org membership. Create a real org owned
# by this subject so project/review routes use the same public API contract as product.
org_response="$(api_json POST /api/orgs "$(json_object name "T7local-$run_id")")" ||
  die "organization setup failed"
org_id="$(printf '%s' "$org_response" | json_value id)"
[[ -n "$org_id" ]] || die "organization response missing id"

project_response="$(api_json POST "/api/orgs/$org_id/projects"   "{"name":"T7local-$run_id","asset_ids":[]}")" ||
  die "project setup failed"
project_id="$(printf '%s' "$project_response" | json_value id)"
[[ -n "$project_id" ]] || die "project response missing id"

api_json PUT "/api/orgs/$org_id/projects/$project_id/target-languages"   "{"source_lang":"$source_lang","target_languages":["$target_lang"]}" >/dev/null ||
  die "target-language setup failed"

serial="${T7LOCAL_EMULATOR_SERIAL:-}"
if [[ -z "$serial" ]]; then
  emulator_count="$(adb devices | awk '/emulator-[0-9]+[[:space:]]+device/{count++} END{print count+0}')"
  [[ "$emulator_count" -eq 1 ]] || die "expected exactly one emulator or set T7LOCAL_EMULATOR_SERIAL"
  serial="$(adb devices | awk '/emulator-[0-9]+[[:space:]]+device/{print $1; exit}')"
fi
adb -s "$serial" get-state >/dev/null 2>&1 || die "emulator unavailable: $serial"
adb -s "$serial" shell pm path com.dubbridge.mobile >/dev/null 2>&1 ||
  die "com.dubbridge.mobile is not installed"
export ANDROID_SERIAL="$serial"
export MAESTRO_CLI_NO_ANALYTICS=1

T7LOCAL_FILENAME="$filename" T7LOCAL_EMULATOR_SERIAL="$serial"   "$t7_dir/prepare-media.sh" >/dev/null ||
  die "real media preparation failed"

maestro test   -e T7LOCAL_EMAIL="$email"   -e T7LOCAL_PASSWORD="$password"   -e T7LOCAL_OWNER="$owner"   -e T7LOCAL_PROOF_REFERENCE="$proof"   -e T7LOCAL_FILENAME="$filename"   "$t7_dir/ingest-real.yaml" ||
  die "B3/C1 Maestro ingestion flow failed"

c1_output="$("$t7_dir/db-probe.sh" wait-c1 "$proof")" || die "C1 durable evidence failed"
printf '%s\n' "$c1_output" | head -n 1
asset_id="$(printf '%s\n' "$c1_output" | tail -n 1)"

api_json POST "/api/orgs/$org_id/projects/$project_id/assets"   "{"asset_id":"$asset_id"}" >/dev/null ||
  die "linking real asset to project failed"

"$t7_dir/db-probe.sh" wait-c2 "$asset_id" || die "C2 preparation evidence failed"

review_output="$("$t7_dir/db-probe.sh" wait-review-task "$asset_id")" ||
  die "no real review task appeared for the current asset"
printf '%s\n' "$review_output" | head -n 1
review_task_id="$(printf '%s\n' "$review_output" | tail -n 1)"

maestro test   -e T7LOCAL_EMAIL="$email"   -e T7LOCAL_PASSWORD="$password"   -e T7LOCAL_REVIEW_TASK_ID="$review_task_id"   "$t7_dir/review-publish-real.yaml" ||
  die "C3/C4 review-publication Maestro flow failed"

"$t7_dir/db-probe.sh" verify-c3 "$review_task_id" || die "C3 durable decision evidence failed"
"$t7_dir/db-probe.sh" verify-c4 "$review_task_id" || die "C4 durable publication evidence failed"

maestro test   -e T7LOCAL_EMAIL="$email"   -e T7LOCAL_PASSWORD="$password"   -e T7LOCAL_ASSET_ID="$asset_id"   "$t7_dir/playback-real.yaml" ||
  die "C4 normal HLS playback flow failed"

mkdir -p "$summary_dir"
chmod 700 "$summary_dir"
head_sha="$(git -C "$repo_root" rev-parse HEAD)"
cat >"$summary_dir/summary.env" <<EOF
T7LOCAL_RUN_ID=$run_id
T7LOCAL_HEAD=$head_sha
T7LOCAL_EMULATOR_SERIAL=$serial
T7LOCAL_GATEWAY=$gateway
T7LOCAL_PROOF_REFERENCE=$proof
T7LOCAL_FILENAME=$filename
T7LOCAL_ORG_ID=$org_id
T7LOCAL_PROJECT_ID=$project_id
T7LOCAL_ASSET_ID=$asset_id
T7LOCAL_REVIEW_TASK_ID=$review_task_id
B3=PASS
C1=PASS
C2=PASS
C3=PASS
C4=PASS
EOF

echo "T7LOCAL_MAESTRO=PASS"
echo "HEAD=$head_sha"
echo "RUN_ID=$run_id"
echo "ASSET_ID=$asset_id"
echo "REVIEW_TASK_ID=$review_task_id"
echo "SUMMARY=$summary_dir/summary.env"
