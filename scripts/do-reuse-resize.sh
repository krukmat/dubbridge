#!/usr/bin/env bash
set -euo pipefail

DROPLET_ID="${DO_REUSE_DROPLET_ID:-144322723}"
EXPECTED_IP="${DO_DROPLET_IP:-46.101.217.151}"
EXPECTED_REGION="${DO_REGION:-fra1}"
TARGET_SIZE="${DO_DROPLET_SIZE:-s-2vcpu-4gb}"
BACKUP_DIR="${DO_WORDPRESS_BACKUP_DIR:-/tmp/dubbridge-t6d2a-wordpress-backup}"
EXECUTE="${DO_T6D3_EXECUTE_RESIZE:-0}"

fail() {
  echo "T6D3_RESIZE=BLOCKED reason=$1" >&2
  exit 2
}

command -v doctl >/dev/null 2>&1 || fail "missing-doctl"
command -v jq >/dev/null 2>&1 || fail "missing-jq"
command -v shasum >/dev/null 2>&1 || fail "missing-shasum"
: "${DIGITALOCEAN_ACCESS_TOKEN:?DIGITALOCEAN_ACCESS_TOKEN is required}"

MANIFEST="${BACKUP_DIR}/restore-manifest.txt"
[[ -f "${MANIFEST}" ]] || fail "wordpress-backup-manifest-missing"

LOCAL_DIR="$(awk -F= '$1=="local_backup"{print substr($0,index($0,"=")+1)}' "${MANIFEST}")"
[[ -n "${LOCAL_DIR}" && -d "${LOCAL_DIR}" ]] || fail "wordpress-backup-dir-missing"
[[ -f "${LOCAL_DIR}/SHA256SUMS" ]] || fail "wordpress-backup-checksums-missing"

while read -r hash file; do
  actual="$(shasum -a 256 "${LOCAL_DIR}/${file}" | awk '{print $1}')"
  [[ "${actual}" == "${hash}" ]] || fail "wordpress-backup-checksum-failed"
done < "${LOCAL_DIR}/SHA256SUMS"
echo "T6D3_BACKUP_GATE=PASS"

droplet_json="$(doctl compute droplet get "${DROPLET_ID}" --output json)"
id="$(jq -r '.[0].id' <<<"${droplet_json}")"
region="$(jq -r '.[0].region.slug // .[0].region' <<<"${droplet_json}")"
size="$(jq -r '.[0].size_slug // .[0].size.slug' <<<"${droplet_json}")"
status="$(jq -r '.[0].status' <<<"${droplet_json}")"
ip="$(jq -r '[.[0].networks.v4[]? | select(.type=="public") | .ip_address][0] // ""' <<<"${droplet_json}")"

[[ "${id}" == "${DROPLET_ID}" ]] || fail "droplet-id-mismatch"
[[ "${ip}" == "${EXPECTED_IP}" ]] || fail "droplet-ip-mismatch"
[[ "${region}" == "${EXPECTED_REGION}" ]] || fail "droplet-region-mismatch"

echo "T6D3_DROPLET_ID=${id}"
echo "T6D3_DROPLET_REGION=${region}"
echo "T6D3_DROPLET_SIZE_BEFORE=${size}"
echo "T6D3_DROPLET_STATUS_BEFORE=${status}"
echo "T6D3_PREFLIGHT=PASS"

if [[ "${size}" == "${TARGET_SIZE}" ]]; then
  echo "T6D3_RESIZE=SKIP_ALREADY_TARGET"
  exit 0
fi

[[ "${size}" == "s-1vcpu-2gb" ]] || fail "unexpected-current-size"
[[ "${EXECUTE}" == "1" ]] || {
  echo "T6D3_RESIZE=READY target=${TARGET_SIZE}"
  echo "T6D3_RESIZE=BLOCKED reason=execution-gate-not-enabled" >&2
  exit 3
}

doctl compute droplet-action resize "${DROPLET_ID}"   --size "${TARGET_SIZE}"   --resize-disk=false   --wait >/dev/null

doctl compute droplet-action power-on "${DROPLET_ID}" --wait >/dev/null

after_json="$(doctl compute droplet get "${DROPLET_ID}" --output json)"
after_size="$(jq -r '.[0].size_slug // .[0].size.slug' <<<"${after_json}")"
after_status="$(jq -r '.[0].status' <<<"${after_json}")"
after_ip="$(jq -r '[.[0].networks.v4[]? | select(.type=="public") | .ip_address][0] // ""' <<<"${after_json}")"

[[ "${after_size}" == "${TARGET_SIZE}" ]] || fail "post-resize-size-mismatch"
[[ "${after_ip}" == "${EXPECTED_IP}" ]] || fail "post-resize-ip-mismatch"

echo "T6D3_DROPLET_SIZE_AFTER=${after_size}"
echo "T6D3_DROPLET_STATUS_AFTER=${after_status}"
echo "T6D3_RESIZE=PASS"
