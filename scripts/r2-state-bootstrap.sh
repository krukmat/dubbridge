#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IAC_DIR="${ROOT}/infra/contabo"
BACKEND_FILE="${IAC_DIR}/backend.hcl"
BUCKET="${R2_STATE_BUCKET:-dubbridge-poc-v1-opentofu-state}"

fail(){ echo "R2_STATE_BOOTSTRAP=BLOCKED reason=$1" >&2; exit 2; }

command -v aws >/dev/null 2>&1 || fail "missing-aws-cli"
: "${R2_ENDPOINT:?R2_ENDPOINT required}"

if aws --endpoint-url "${R2_ENDPOINT}" s3api head-bucket --bucket "${BUCKET}" >/dev/null 2>&1; then
  echo "R2_STATE_BUCKET=EXISTS name=${BUCKET}"
else
  aws --endpoint-url "${R2_ENDPOINT}" s3api create-bucket --bucket "${BUCKET}" >/dev/null
  echo "R2_STATE_BUCKET_CREATE=PASS name=${BUCKET}"
fi

aws --endpoint-url "${R2_ENDPOINT}" s3api head-bucket --bucket "${BUCKET}" >/dev/null

account_host="${R2_ENDPOINT#https://}"
cat > "${BACKEND_FILE}" <<EOF
bucket = "${BUCKET}"
key    = "dubbridge/poc-v1/opentofu.tfstate"
region = "auto"

endpoints = {
  s3 = "https://${account_host}"
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
tofu init -reconfigure -input=false -backend-config="${BACKEND_FILE}" >/dev/null
tofu validate >/dev/null

echo "R2_STATE_BACKEND_INIT=PASS"
echo "R2_STATE_BOOTSTRAP=PASS"
