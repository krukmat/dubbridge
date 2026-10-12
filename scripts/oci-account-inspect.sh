#!/usr/bin/env bash
# Read-only account preflight; uses locally configured OCI CLI identity.
# No credentials, OCI OCIDs, SSH keys or raw API responses are printed.
set -euo pipefail

fail() {
  echo "OCI_ACCOUNT_INSPECT=BLOCKED reason=$1" >&2
  exit 2
}

command -v oci >/dev/null 2>&1 || fail "missing-oci-cli"
command -v python3 >/dev/null 2>&1 || fail "missing-python3"
: "${OCI_COMPARTMENT_OCID:?set OCI_COMPARTMENT_OCID for the POC compartment}"

workspace="$(mktemp -d)"
trap 'rm -rf "$workspace"' EXIT

oci iam region-subscription list --all --output json > "$workspace/regions.json" ||
  fail "region-subscriptions-unavailable"
home="$(python3 - "$workspace/regions.json" <<'PY'
import json, sys
items = json.load(open(sys.argv[1], encoding='utf-8')).get('data', [])
homes = [x.get('region-name', '') for x in items if x.get('is-home-region')]
if len(homes) != 1 or not homes[0]:
    sys.exit(3)
print(homes[0])
PY
)" || fail "home-region-not-unambiguous"

oci iam availability-domain list --compartment-id "$OCI_COMPARTMENT_OCID"   --region "$home" --output json > "$workspace/domains.json" ||
  fail "availability-domains-unavailable"
oci compute shape list --compartment-id "$OCI_COMPARTMENT_OCID"   --region "$home" --all --output json > "$workspace/shapes.json" ||
  fail "shapes-unavailable"
oci limits value list --compartment-id "$OCI_COMPARTMENT_OCID"   --service-name compute --region "$home" --all --output json > "$workspace/limits.json" ||
  fail "compute-limits-unavailable"

python3 - "$workspace" "$home" <<'PY'
import json, pathlib, sys
root, home = pathlib.Path(sys.argv[1]), sys.argv[2]
def read(filename):
    return json.loads((root / filename).read_text(encoding="utf-8")).get("data", [])
domains = read("domains.json")
shapes = read("shapes.json")
limits = read("limits.json")
a1 = any(x.get("shape") == "VM.Standard.A1.Flex" for x in shapes)
print(f"OCI_HOME_REGION={home}")
print(f"OCI_AD_COUNT={len(domains)}")
print("OCI_A1_SHAPE_LISTED=" + ("YES" if a1 else "NO"))
for key in ("standard-a1-core-count", "standard-a1-memory-count"):
    match = [x for x in limits if x.get("name", "").lower() == key]
    vals = sorted({str(x.get("value")) for x in match})
    print("OCI_LIMIT_" + key.upper().replace("-", "_") + "=" + (",".join(vals) if vals else "UNKNOWN"))
print("OCI_ACCOUNT_INSPECT=PARTIAL_READ_ONLY")
print("OCI_FREE_TIER_ELIGIBILITY=UNVERIFIED")
print("OCI_AVAILABLE_HOST_CAPACITY=UNVERIFIED")
print("OCI_TOTAL_BOOT_BLOCK_USED=UNVERIFIED")
print("OCI_SPEND_CAP=UNVERIFIED")
PY
