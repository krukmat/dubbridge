#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IAC_DIR="${ROOT}/infra/digitalocean"
BACKEND_FILE="${IAC_DIR}/backend.hcl"
DROPLET_ID="${DO_REUSE_DROPLET_ID:-144322723}"
DNS_DOMAIN="${DO_DOMAIN:-iotforce.es}"
DNS_RECORD_ID="${DO_DNS_RECORD_ID:-1830236122}"
TARGET_SIZE="${DO_DROPLET_SIZE:-s-2vcpu-4gb}"

fail() {
  echo "T6D3_IMPORT=BLOCKED reason=$1" >&2
  exit 2
}

command -v tofu >/dev/null 2>&1 || fail "missing-opentofu"
command -v doctl >/dev/null 2>&1 || fail "missing-doctl"
command -v jq >/dev/null 2>&1 || fail "missing-jq"

: "${DIGITALOCEAN_ACCESS_TOKEN:?DIGITALOCEAN_ACCESS_TOKEN is required}"
: "${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID is required for remote state}"
: "${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY is required for remote state}"

[[ -f "${BACKEND_FILE}" ]] || fail "missing-backend-hcl"

size="$(doctl compute droplet get "${DROPLET_ID}" --output json | jq -r '.[0].size_slug // .[0].size.slug')"
[[ "${size}" == "${TARGET_SIZE}" ]] || fail "resize-not-complete"

cd "${IAC_DIR}"
tofu init -input=false -backend-config="${BACKEND_FILE}" >/dev/null

COMMON_VARS=(
  -var='region=fra1'
  -var='manage_droplet=true'
  -var='manage_firewall=true'
  -var='manage_database=true'
  -var='manage_media_space=true'
  -var='manage_dns=true'
)

if tofu state list | grep -Fxq 'digitalocean_droplet.app[0]'; then
  echo "T6D3_IMPORT_DROPLET=SKIP_ALREADY_IMPORTED"
else
  tofu import "${COMMON_VARS[@]}" 'digitalocean_droplet.app[0]' "${DROPLET_ID}" >/dev/null
  echo "T6D3_IMPORT_DROPLET=PASS id=${DROPLET_ID}"
fi

if tofu state list | grep -Fxq 'digitalocean_record.poc[0]'; then
  echo "T6D3_IMPORT_DNS=SKIP_ALREADY_IMPORTED"
else
  tofu import "${COMMON_VARS[@]}" 'digitalocean_record.poc[0]' "${DNS_DOMAIN},${DNS_RECORD_ID}" >/dev/null
  echo "T6D3_IMPORT_DNS=PASS id=${DNS_RECORD_ID}"
fi

echo "T6D3_IMPORT=PASS"
