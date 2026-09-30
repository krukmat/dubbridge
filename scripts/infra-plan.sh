#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IAC_DIR="${ROOT}/infra/contabo"
BACKEND_FILE="${IAC_DIR}/backend.hcl"
PLAN_FILE="${IAC_DIR}/.t6-contabo-plan.bin"
PLAN_JSON="${IAC_DIR}/.t6-contabo-plan.json"

fail() {
  echo "INFRA_PLAN=BLOCKED reason=$1" >&2
  exit 2
}

command -v tofu >/dev/null 2>&1 || fail "missing-opentofu"
[[ -f "${IAC_DIR}/terraform.tfvars" ]] || fail "missing-terraform-tfvars"

cd "${IAC_DIR}"

# Provider install + syntax/schema validation can run before remote state exists.
if [[ ! -f "${BACKEND_FILE}" ]]; then
  tofu init -backend=false -input=false >/dev/null
  tofu validate >/dev/null
  echo "INFRA_PLAN_VALIDATE=PASS"
  echo "INFRA_PLAN=BLOCKED reason=remote-state-not-bootstrapped"
  exit 2
fi

: "${CNTB_OAUTH2_CLIENT_ID:?CNTB_OAUTH2_CLIENT_ID is required}"
: "${CNTB_OAUTH2_CLIENT_SECRET:?CNTB_OAUTH2_CLIENT_SECRET is required}"
: "${CNTB_OAUTH2_USER:?CNTB_OAUTH2_USER is required}"
: "${CNTB_OAUTH2_PASS:?CNTB_OAUTH2_PASS is required}"
: "${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID is required for R2 state}"
: "${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY is required for R2 state}"

tofu init -input=false -backend-config="${BACKEND_FILE}" >/dev/null
tofu validate >/dev/null

set +e
tofu plan -input=false -detailed-exitcode -out="${PLAN_FILE}" >/dev/null
status=$?
set -e
[[ "${status}" -eq 0 || "${status}" -eq 2 ]] || fail "tofu-plan-failed"

tofu show -json "${PLAN_FILE}" > "${PLAN_JSON}"

python3 - "${PLAN_JSON}" "${INFRA_ALLOW_CREATE:-}" <<'PY'
import json
import sys

path = sys.argv[1]
allow_raw = sys.argv[2].strip()
allow = {item.strip() for item in allow_raw.split(",") if item.strip()}

with open(path, encoding="utf-8") as fh:
    doc = json.load(fh)

blocked = []
seen_creates = set()
summary = dict(create=0, update=0, delete=0, replace=0, noop=0)

for rc in doc.get("resource_changes", []):
    actions = rc.get("change", {}).get("actions", [])
    aset = set(actions)
    address = rc.get("address", "?")

    if actions == ["no-op"]:
        summary["noop"] += 1
    if "update" in aset:
        summary["update"] += 1

    if "create" in aset and "delete" in aset:
        summary["replace"] += 1
        blocked.append((address, actions, "replace"))
    elif "delete" in aset:
        summary["delete"] += 1
        blocked.append((address, actions, "delete"))
    elif "create" in aset:
        summary["create"] += 1
        seen_creates.add(address)
        if address not in allow:
            blocked.append((address, actions, "unapproved-create"))

unused_allow = allow - seen_creates
for address in sorted(unused_allow):
    blocked.append((address, [], "allowlist-address-not-in-plan"))

print("INFRA_PLAN_SUMMARY=" + ",".join(f"{k}:{v}" for k, v in summary.items()))
print("INFRA_PLAN_ALLOWED_CREATE=" + (",".join(sorted(allow)) if allow else "NONE"))

for address, actions, reason in blocked:
    print(
        f"INFRA_PLAN_BLOCKED_RESOURCE={address} actions={actions} reason={reason}",
        file=sys.stderr,
    )

sys.exit(3 if blocked else 0)
PY

echo "INFRA_PLAN_VALIDATE=PASS"
echo "INFRA_PLAN_DESTRUCTIVE_GUARD=PASS"
echo "INFRA_PLAN_CREATE_ALLOWLIST=PASS"
echo "INFRA_PLAN=PASS"
