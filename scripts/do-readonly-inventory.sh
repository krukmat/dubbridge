#!/usr/bin/env bash
set -euo pipefail

OUT="${1:-/tmp/dubbridge-do-inventory}"
mkdir -p "${OUT}"

command -v doctl >/dev/null 2>&1 || { echo "DO_INVENTORY=BLOCKED reason=missing-doctl" >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "DO_INVENTORY=BLOCKED reason=missing-jq" >&2; exit 2; }
: "${DIGITALOCEAN_ACCESS_TOKEN:?DIGITALOCEAN_ACCESS_TOKEN is required by doctl}"

doctl compute droplet list --output json > "${OUT}/droplets.json"
doctl compute firewall list --output json > "${OUT}/firewalls.json"
doctl databases list --output json > "${OUT}/databases.json"
doctl compute domain records list iotforce.es --output json > "${OUT}/dns.json"

jq '[.[] | {id,name,status,region:(.region.slug // .region),networks}]' "${OUT}/droplets.json" > "${OUT}/droplets.sanitized.json"
jq '[.[] | {id,name,status}]' "${OUT}/firewalls.json" > "${OUT}/firewalls.sanitized.json"
jq '[.[] | {id,name,engine,version,region,status}]' "${OUT}/databases.json" > "${OUT}/databases.sanitized.json"
jq '[.[] | {id,type,name,data,ttl}]' "${OUT}/dns.json" > "${OUT}/dns.sanitized.json"

rm -f "${OUT}/droplets.json" "${OUT}/firewalls.json" "${OUT}/databases.json" "${OUT}/dns.json"

echo "DO_INVENTORY_READONLY=PASS"
echo "DO_INVENTORY_DIR=${OUT}"
