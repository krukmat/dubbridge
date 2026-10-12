#!/usr/bin/env bash
# Checks OCI A1 host availability in the verified home region.
# Generates a capacity report only: does not provision compute instances.
set -euo pipefail

fail() {
  echo "OCI_A1_CAPACITY=UNKNOWN reason=$1" >&2
  exit 2
}

command -v oci >/dev/null 2>&1 || fail "missing-oci-cli"
command -v python3 >/dev/null 2>&1 || fail "missing-python3"
: "${OCI_REGION:=eu-madrid-3}"

TENANCY="${OCI_COMPARTMENT_OCID:-$(awk -F= '/^[[:space:]]*tenancy[[:space:]]*=/{gsub(/[[:space:]]/, "", $2); print $2; exit}' "${OCI_CONFIG_FILE:-$HOME/.oci/config}")}"
[[ "$TENANCY" == ocid1.tenancy.* ]] || fail "missing-root-tenancy-ocid"

ad_json="$(oci iam availability-domain list -c "$TENANCY" --region "$OCI_REGION" --query 'data[].name' --output json)" || fail "availability-domains-query-failed"
ad_lines="$(printf '%s\n' "$ad_json" | python3 -c 'import json,sys; data=json.load(sys.stdin); assert isinstance(data,list); [print(x) for x in data if isinstance(x,str) and x]')" || fail "availability-domains-json-invalid"
[[ -n "$ad_lines" ]] || fail "no-availability-domains"

found=0
while IFS= read -r ad; do
  [[ -n "$ad" ]] || continue
  response="$(oci compute compute-capacity-report create \
    -c "$TENANCY" \
    --availability-domain "$ad" \
    --shape-availabilities '[{"instanceShape":"VM.Standard.A1.Flex","instanceShapeConfig":{"ocpus":2,"memoryInGBs":12}}]' \
    --region "$OCI_REGION" \
    --query 'data."shape-availabilities"[0]."availability-status"' \
    --raw-output)" || fail "capacity-report-failed"
  case "$response" in
    AVAILABLE) found=1 ;;
    OUT_OF_HOST_CAPACITY|HARDWARE_NOT_SUPPORTED) ;;
    *) fail "unexpected-capacity-status" ;;
  esac
  printf 'OCI_A1_AD=%s status=%s\n' "$ad" "$response"
done <<< "$ad_lines"

if (( found )); then
  echo 'OCI_A1_CAPACITY=AVAILABLE_NOT_RESERVED'
else
  echo 'OCI_A1_CAPACITY=BLOCKED'
fi
