# S-230 / T6d.O1 — OCI Always Free IaC (NO APPLY)

This directory belongs to the **Oracle primary** deployment path. The
Contabo and DigitalOcean infrastructure directories are historical/fallback
references; they must not be executed as part of Oracle provisioning.

## Baseline

- Oracle OCI Always Free tenancy home-region Ampere A1 Flex.
- Initial shape: `VM.Standard.A1.Flex`, 2 OCPU, 12 GB.
- Boot volume: **50 GB** (counts against the 200 GB combined Always Free
  boot/block allowance).
- One VCN, public subnet, IGW, route table, restrictive security list, one
  instance. No Kubernetes/NAT gateway/load balancer/managed DB.
- Public TCP 80/443 only, plus TCP 22 restricted to a declared operator /32.
  DB/Redis/Availability Node must not bind to the host public address.
- Docker images must be `linux/arm64` pinned OCI digests.
- No resource is created by default (`enable_resources=false`).

## Operational gates

1. O0.B/D: read-only inspect **actual** tenancy, home region, compartment,
   Always Free usage and A1 capacity; verify 0 additional recurring cost.
2. O1.A/B/C/D: static IaC review, provider lockfile, plan guard negative tests,
   preflight safety packet. **No apply.**
3. O2: state bootstrap and **explicit owner authorization** after approved
   exact resource-address plan. No automatic paid fallback.
4. O3: benchmark ARM64, native dependencies and constrained ASR throughput.

No Oracle credentials, tenancy identifiers, SSH private keys or state
contents belong in Git.

## Files

- `main.tf`: OCI resources and strict shape/network inputs; disabled by default
- `terraform.tfvars.example`: safe non-creating example
- `scripts/oci-plan-guard.py`: independent fail-closed plan JSON gate
- `scripts/test-oci-plan-guard.py`: synthetic offline negative tests

The guard is a prerequisite to approval, **not** sufficient evidence for
the real OCI provider/tenant plan. The actual plan must also be reviewed.

Provider: https://registry.terraform.io/providers/oracle/oci
Official OCI Always Free: https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm

## O0.B read-only account snapshot

After configuring OCI CLI on the operator workstation, set only the target
compartment identifier in your local shell and execute:

```bash
export OCI_COMPARTMENT_OCID="ocid1.compartment.oc1..YOUR_COMPARTMENT"
make oci-account-inspect
```

The script runs **read-only** region, AD, shape and compute-limit discovery
via OCI CLI credentials; it prints only bounded summary markers, not OCIDs or
private keys. The output is *partial* by design: it does not prove that free
capacity is actually available, account billing is safe, or storage used in
other compartments. Do not mark O0.B PASS from this script alone.

## Account setup checkpoint — 2026-10-12

Owner confirms account login and home region **Spain Central (Madrid 3)**
(`eu-madrid-3`). This is owner-provided confirmation, **not** independent
OCI CLI quota, availability or free-tier eligibility evidence.

On the operator Mac: `brew install oci-cli`, `oci setup config`, use
region `eu-madrid-3`, register the generated API **public** key in Oracle
User settings → Tokens and keys, then test authentication with
`oci iam region-subscription list`. Never share the private API key or
`~/.oci/config`. O0.B remains partial until actual read-only checks pass.

## CLI authentication checkpoint — 2026-10-12

The operator's local OCI CLI successfully returned home region
`eu-madrid-3` with status `READY` from
`oci iam region-subscription list --output table`. Do not interpret this
as actual A1 availability or free-tier limit certification.

For initial **read-only** inventory in the tenancy's root compartment,
extract the configured tenancy OCID locally without printing it:

```bash
cd ~/dubbridge
git pull
export OCI_COMPARTMENT_OCID="$(awk -F= '/^tenancy[[:space:]]*=/ {gsub(/[[:space:]]/, "", $2); print $2; exit}' ~/.oci/config)"
make oci-account-inspect
```

If the configured CLI profile is not DEFAULT, set the matching compartment
OCID locally and run the check with the same profile used for authentication.
Do not paste `~/.oci/config` or any OCID/private key into chat or source.
