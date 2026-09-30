#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IAC_DIR="${ROOT}/infra/contabo"
BACKEND_FILE="${IAC_DIR}/backend.hcl"
OUT_DIR="${INFRA_STATE_BACKUP_DIR:-${ROOT}/artifacts/deploy/state-backups}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="${OUT_DIR}/opentofu-state-${STAMP}.json"

fail() {
  echo "INFRA_STATE_BACKUP=BLOCKED reason=$1" >&2
  exit 2
}

command -v tofu >/dev/null 2>&1 || fail "missing-opentofu"
[[ -f "${BACKEND_FILE}" ]] || fail "missing-backend-hcl"
: "${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID is required for R2 state}"
: "${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY is required for R2 state}"

mkdir -p "${OUT_DIR}"
cd "${IAC_DIR}"
tofu init -input=false -backend-config="${BACKEND_FILE}" >/dev/null

tmp="$(mktemp)"
trap 'rm -f "${tmp}"' EXIT
tofu state pull > "${tmp}"

python3 - "${tmp}" <<'PY'
import json
import sys
with open(sys.argv[1], encoding="utf-8") as fh:
    doc = json.load(fh)
if not isinstance(doc, dict) or "serial" not in doc or "lineage" not in doc:
    raise SystemExit("invalid OpenTofu state payload")
print(f"INFRA_STATE_SERIAL={doc['serial']}")
print(f"INFRA_STATE_LINEAGE={doc['lineage']}")
PY

cp "${tmp}" "${OUT}"
shasum -a 256 "${OUT}" > "${OUT}.sha256"

echo "INFRA_STATE_BACKUP_FILE=${OUT}"
echo "INFRA_STATE_BACKUP_SHA256_FILE=${OUT}.sha256"
echo "INFRA_STATE_BACKUP=PASS"
