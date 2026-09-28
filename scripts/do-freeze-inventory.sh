#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-/tmp/dubbridge-t6d1-inventory}"
DOMAIN="${DO_DOMAIN:-iotforce.es}"
DROPLET_IP="${DO_DROPLET_IP:-46.101.217.151}"
MEDIA_SPACE="${DO_MEDIA_SPACE:-dubbridge-poc-v1}"
SPACES_REGION="${DO_SPACES_REGION:-ams3}"
SPACES_ENDPOINT="https://${SPACES_REGION}.digitaloceanspaces.com"

fail() {
  echo "T6D1_INVENTORY=BLOCKED reason=$1" >&2
  exit 2
}

command -v doctl >/dev/null 2>&1 || fail "missing-doctl"
command -v jq >/dev/null 2>&1 || fail "missing-jq"
command -v aws >/dev/null 2>&1 || fail "missing-aws-cli"

: "${DIGITALOCEAN_ACCESS_TOKEN:?DIGITALOCEAN_ACCESS_TOKEN is required}"

SPACES_ACCESS_KEY_ID="${SPACES_ACCESS_KEY_ID:-${DO_SPACES_ACCESS_KEY:-}}"
SPACES_SECRET_ACCESS_KEY="${SPACES_SECRET_ACCESS_KEY:-${DO_SPACES_SECRET_KEY:-}}"
: "${SPACES_ACCESS_KEY_ID:?SPACES_ACCESS_KEY_ID or DO_SPACES_ACCESS_KEY is required}"
: "${SPACES_SECRET_ACCESS_KEY:?SPACES_SECRET_ACCESS_KEY or DO_SPACES_SECRET_KEY is required}"

rm -rf "${OUT}"
mkdir -p "${OUT}"

doctl compute droplet list --output json > "${OUT}/droplets.raw.json"
doctl compute firewall list --output json > "${OUT}/firewalls.raw.json"
doctl databases list --output json > "${OUT}/databases.raw.json"
doctl compute domain records list "${DOMAIN}" --output json > "${OUT}/dns.raw.json"

jq --arg ip "${DROPLET_IP}" '
  [.[] |
    select(
      ([.networks.v4[]?.ip_address] | index($ip)) != null
    ) |
    {
      id,
      name,
      status,
      region:(.region.slug // .region),
      size_slug:(.size_slug // .size.slug // null),
      image:{
        id:(.image.id // null),
        slug:(.image.slug // null),
        name:(.image.name // null),
        distribution:(.image.distribution // null)
      },
      ipv4:([.networks.v4[]? | select(.type=="public") | .ip_address][0] // null),
      ipv6:([.networks.v6[]? | select(.type=="public") | .ip_address][0] // null),
      tags:(.tags // []),
      vcpus:(.vcpus // null),
      memory_mb:(.memory // null),
      disk_gb:(.disk // null),
      monitoring:(.monitoring // null)
    }
  ]' "${OUT}/droplets.raw.json" > "${OUT}/droplet.freeze.json"

jq '
  [.[] | {
    id,
    name,
    status,
    droplet_ids:(.droplet_ids // []),
    tags:(.tags // []),
    inbound_rules:(.inbound_rules // []),
    outbound_rules:(.outbound_rules // [])
  }]' "${OUT}/firewalls.raw.json" > "${OUT}/firewalls.freeze.json"

jq '
  [.[] | {
    id,
    name,
    engine,
    version,
    region,
    status,
    size:(.size // null),
    num_nodes:(.num_nodes // null),
    private_network_uuid:(.private_network_uuid // null)
  }]' "${OUT}/databases.raw.json" > "${OUT}/databases.freeze.json"

jq '
  [.[] | {
    id,
    type,
    name,
    data,
    ttl,
    priority:(.priority // null),
    port:(.port // null),
    weight:(.weight // null)
  }]' "${OUT}/dns.raw.json" > "${OUT}/dns.freeze.json"

AWS_ACCESS_KEY_ID="${SPACES_ACCESS_KEY_ID}" \
AWS_SECRET_ACCESS_KEY="${SPACES_SECRET_ACCESS_KEY}" \
AWS_DEFAULT_REGION="us-east-1" \
AWS_EC2_METADATA_DISABLED=true \
aws s3api head-bucket \
  --bucket "${MEDIA_SPACE}" \
  --endpoint-url "${SPACES_ENDPOINT}" >/dev/null 2>&1 \
  || fail "media-space-not-accessible"

cat > "${OUT}/media-space.freeze.json" <<EOF
{
  "name": "${MEDIA_SPACE}",
  "region": "${SPACES_REGION}",
  "endpoint": "${SPACES_ENDPOINT}",
  "accessible": true
}
EOF

rm -f "${OUT}"/*.raw.json

droplet_count="$(jq 'length' "${OUT}/droplet.freeze.json")"
[[ "${droplet_count}" == "1" ]] || fail "expected-one-droplet-for-ip"

dns_target_count="$(jq --arg host "poc.${DOMAIN}" --arg ip "${DROPLET_IP}" '
  [.[] | select(.type=="A" and ((.name==$host) or (.name=="poc")) and .data==$ip)] | length
' "${OUT}/dns.freeze.json")"

cat > "${OUT}/summary.json" <<EOF
{
  "schema": "dubbridge-t6d1-inventory-v1",
  "droplet_ip": "${DROPLET_IP}",
  "domain": "${DOMAIN}",
  "media_space": "${MEDIA_SPACE}",
  "spaces_region": "${SPACES_REGION}",
  "droplet_matches": ${droplet_count},
  "dns_target_matches": ${dns_target_count}
}
EOF

echo "T6D1_DROPLET_MATCH=PASS count=${droplet_count}"
echo "T6D1_MEDIA_SPACE=PASS name=${MEDIA_SPACE} region=${SPACES_REGION}"
echo "T6D1_DNS_MATCHES=${dns_target_count}"
echo "T6D1_INVENTORY_DIR=${OUT}"
echo "T6D1_INVENTORY=PASS"
