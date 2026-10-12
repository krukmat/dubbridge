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
