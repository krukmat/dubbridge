#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IAC_DIR="${ROOT}/infra/digitalocean"
BACKEND_FILE="${IAC_DIR}/backend.hcl"
PLAN_FILE="${IAC_DIR}/.t6-plan.bin"
PLAN_JSON="${IAC_DIR}/.t6-plan.json"

fail() {
  echo "DO_PLAN=BLOCKED reason=$1" >&2
  exit 2
}

command -v tofu >/dev/null 2>&1 || fail "missing-opentofu"
cd "${IAC_DIR}"
[[ -f terraform.tfvars ]] || fail "missing-terraform-tfvars"

if [[ ! -f "${BACKEND_FILE}" ]]; then
  tofu init -backend=false -input=false >/dev/null
  tofu validate >/dev/null
  echo "DO_PLAN_VALIDATE=PASS"
  echo "DO_PLAN=BLOCKED reason=remote-state-not-bootstrapped"
  exit 2
fi

: "${DIGITALOCEAN_TOKEN:?DIGITALOCEAN_TOKEN is required}"
: "${SPACES_ACCESS_KEY_ID:?SPACES_ACCESS_KEY_ID is required}"
: "${SPACES_SECRET_ACCESS_KEY:?SPACES_SECRET_ACCESS_KEY is required}"
: "${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID is required for remote state}"
: "${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY is required for remote state}"

tofu init -input=false -backend-config="${BACKEND_FILE}" >/dev/null
tofu validate >/dev/null

set +e
tofu plan -input=false -detailed-exitcode -out="${PLAN_FILE}" >/dev/null
status=$?
set -e
[[ "${status}" -eq 0 || "${status}" -eq 2 ]] || fail "tofu-plan-failed"

tofu show -json "${PLAN_FILE}" > "${PLAN_JSON}"

python3 - "${PLAN_JSON}" "${DO_ALLOW_CREATE:-0}" <<'PY'
import json, sys
path, allow_create = sys.argv[1], sys.argv[2] == "1"
doc = json.load(open(path, encoding="utf-8"))
blocked = []
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
        if not allow_create:
            blocked.append((address, actions, "unapproved-create"))
print("DO_PLAN_SUMMARY=" + ",".join(f"{k}:{v}" for k,v in summary.items()))
for address, actions, reason in blocked:
    print(f"DO_PLAN_BLOCKED_RESOURCE={address} actions={actions} reason={reason}", file=sys.stderr)
sys.exit(3 if blocked else 0)
PY

echo "DO_PLAN_VALIDATE=PASS"
echo "DO_PLAN_DESTRUCTIVE_GUARD=PASS"
echo "DO_PLAN=PASS"
