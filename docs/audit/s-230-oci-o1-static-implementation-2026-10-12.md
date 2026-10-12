# S-230 / T6d.O1 — Static implementation checkpoint

Date: 2026-10-12
Branch: main
Disposition: **IMPLEMENTED IN PART — NO RUNTIME OR PROVIDER VALIDATION CLAIMED**
Cloud mutation: NONE.

## Deliverables

- `infra/oracle/main.tf`: Oracle OCI provider pinned to 9.9.0,
  `enable_resources=false` default; optional A1 2 OCPU/12 GB, 50 GB boot,
  one VCN/IGW/route/subnet/restrictive security list, public Caddy 80/443,
  SSH to operator /32; all resources have `prevent_destroy`.
- `infra/oracle/terraform.tfvars.example`: no-resource default.
- `scripts/oci-plan-guard.py`: separate fail-closed JSON plan policy:
  exact creation address allowlist, reject update/replace/delete, reject
  wrong shape/CPU/RAM/boot and insecure security-list ingress.
- `scripts/test-oci-plan-guard.py`: offline synthetic cases for happy path,
  unapproved creates, paid shape, oversized RAM/boot, unexpected public
  ports, world-open SSH, changes/delete/replace and stale allowlist.
- `infra/oracle/OPERATOR_PACKET.md`: explicit execution and ARM64 release
  requirements, with O0/O2 gates.
- `infra/oracle/README.md`: provider-neutral no-apply operational guide.
- `make oci-guard-tests` and `make oci-plan-guard OCI_PLAN_JSON=<file>`.
- `.gitignore`: exclude untracked OCI state, local tfvars and plans.

## Pending closure proof — do NOT mark O1 PASS

1. Run the Python test suite and record count/result on actual pulled main.
2. With local OpenTofu installed, `tofu init -backend=false`,
   `tofu validate` and commit the provider lockfile (do not use local state
   for production).
3. Verify the real OCI provider schema and confirm final resource attributes.
4. O0.B/C/D require account-specific home-region free quota, R2 usage,
   state/backup proof and explicit owner acceptance before O2.
5. Build/publish verified linux/arm64 OCI image digests and test on actual
   Ampere A1 when a VM is authorized.

This checkpoint certifies **source changes only**. No OCI tenancy, instance,
registry or media bucket has been touched.

## Source grounding

Official OCI Always Free tenancy: 1500 OCPU hours, 9000 GB-hours
(equivalent to 2 OCPU/12 GB); combined 200 GB boot+block.
Official provider registry release 9.9.0 (2026-10-08).
Official OCI provider docs for `oci_core_instance`, `oci_core_vcn`,
`oci_core_security_list`, `oci_core_subnet`, route table, IGW.

- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm
- https://registry.terraform.io/providers/oracle/oci/9.9.0
- https://docs.oracle.com/en-us/iaas/tools/terraform-provider-oci/latest/docs/r/core_instance.html
