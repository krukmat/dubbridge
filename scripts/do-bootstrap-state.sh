#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IAC_DIR="${ROOT}/infra/digitalocean"
REGION="${DO_STATE_REGION:-ams3}"
BUCKET="${DO_STATE_BUCKET:-dubbridge-poc-v1-opentofu-state}"
KEY="${DO_STATE_KEY:-dubbridge/poc-v1/opentofu.tfstate}"
ENDPOINT="https://${REGION}.digitaloceanspaces.com"
BACKEND_FILE="${IAC_DIR}/backend.hcl"

fail() {
  echo "DO_STATE_BOOTSTRAP=BLOCKED reason=$1" >&2
  exit 2
}

command -v aws >/dev/null 2>&1 || fail "missing-aws-cli"
command -v tofu >/dev/null 2>&1 || fail "missing-opentofu"

SPACES_ACCESS_KEY_ID="${SPACES_ACCESS_KEY_ID:-${DO_SPACES_ACCESS_KEY:-}}"
SPACES_SECRET_ACCESS_KEY="${SPACES_SECRET_ACCESS_KEY:-${DO_SPACES_SECRET_KEY:-}}"
: "${SPACES_ACCESS_KEY_ID:?SPACES_ACCESS_KEY_ID or DO_SPACES_ACCESS_KEY is required}"
: "${SPACES_SECRET_ACCESS_KEY:?SPACES_SECRET_ACCESS_KEY or DO_SPACES_SECRET_KEY is required}"

export AWS_ACCESS_KEY_ID="${SPACES_ACCESS_KEY_ID}"
export AWS_SECRET_ACCESS_KEY="${SPACES_SECRET_ACCESS_KEY}"
export AWS_DEFAULT_REGION="us-east-1"
export AWS_EC2_METADATA_DISABLED=true

echo "DO_STATE_BUCKET=${BUCKET}"
echo "DO_STATE_REGION=${REGION}"

if aws s3api head-bucket --bucket "${BUCKET}" --endpoint-url "${ENDPOINT}" >/dev/null 2>&1; then
  echo "DO_STATE_BUCKET_CREATE=SKIP_ALREADY_EXISTS"
else
  aws s3api create-bucket \
    --bucket "${BUCKET}" \
    --acl private \
    --endpoint-url "${ENDPOINT}" >/dev/null
  echo "DO_STATE_BUCKET_CREATE=PASS"
fi

aws s3api put-bucket-versioning \
  --bucket "${BUCKET}" \
  --endpoint-url "${ENDPOINT}" \
  --versioning-configuration Status=Enabled >/dev/null

versioning="$(aws s3api get-bucket-versioning \
  --bucket "${BUCKET}" \
  --endpoint-url "${ENDPOINT}" \
  --query Status --output text)"
[[ "${versioning}" == "Enabled" ]] || fail "bucket-versioning-not-enabled"
echo "DO_STATE_VERSIONING=PASS"

cat > "${BACKEND_FILE}" <<EOF
bucket = "${BUCKET}"
key    = "${KEY}"
region = "${REGION}"

endpoints = {
  s3 = "${ENDPOINT}"
}

use_path_style              = true
skip_credentials_validation = true
skip_region_validation      = true
skip_requesting_account_id  = true
skip_metadata_api_check     = true
skip_s3_checksum            = true
EOF
chmod 600 "${BACKEND_FILE}"

cd "${IAC_DIR}"
rm -rf .terraform
AWS_ACCESS_KEY_ID="${SPACES_ACCESS_KEY_ID}" \
AWS_SECRET_ACCESS_KEY="${SPACES_SECRET_ACCESS_KEY}" \
tofu init -input=false -backend-config="${BACKEND_FILE}" >/dev/null
tofu validate >/dev/null

echo "DO_STATE_BACKEND_INIT=PASS"
echo "DO_STATE_BOOTSTRAP=PASS"
