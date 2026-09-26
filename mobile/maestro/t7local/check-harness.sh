#!/usr/bin/env bash
# Deterministic static gate for the real-stack T7local Maestro harness.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
dir="$repo_root/mobile/maestro/t7local"

for script in "$dir/prepare-media.sh" "$dir/db-probe.sh" "$dir/run-real.sh"; do
  bash -n "$script"
done
python3 -m py_compile "$dir/run-real.py"

scan_files=(
  "$dir/login-real.yaml"
  "$dir/ingest-real.yaml"
  "$dir/review-publish-real.yaml"
  "$dir/playback-real.yaml"
  "$dir/prepare-media.sh"
  "$dir/db-probe.sh"
  "$dir/run-real.sh"
  "$dir/run-real.py"
)

for banned in   "/e2e/seed"   "e2e@dubbridge.dev"   "review-task-seed-"   "asset-seed-"   "localhost:8081"   "EXPO_PUBLIC_E2E_ENABLED=true"; do
  if grep -F "$banned" "${scan_files[@]}" >/dev/null; then
    echo "T7local harness contains banned mock/seed contract: $banned" >&2
    exit 1
  fi
done

grep -F 'setClipboard: ${T7LOCAL_EMAIL}' "$dir/login-real.yaml" >/dev/null
grep -F 'setClipboard: ${T7LOCAL_PASSWORD}' "$dir/login-real.yaml" >/dev/null
grep -F 'pressKey: Enter' "$dir/login-real.yaml" >/dev/null
grep -F 'upload-field-proof-reference' "$dir/ingest-real.yaml" >/dev/null
grep -F 'review-task-card-${T7LOCAL_REVIEW_TASK_ID}' "$dir/review-publish-real.yaml" >/dev/null
grep -F 'asset-card-${T7LOCAL_ASSET_ID}' "$dir/playback-real.yaml" >/dev/null
grep -F '"wait-c2"' "$dir/run-real.py" >/dev/null
grep -F '"verify-c4"' "$dir/run-real.py" >/dev/null
grep -F '"--certify-c3"' "$dir/run-real.py" >/dev/null
grep -F '"verdict": "approved"' "$dir/run-real.py" >/dev/null
grep -F '"verify-c3"' "$dir/run-real.py" >/dev/null
grep -F 'http://localhost:8082' "$dir/run-real.py" >/dev/null

echo "T7LOCAL MAESTRO HARNESS CHECK: PASS"
