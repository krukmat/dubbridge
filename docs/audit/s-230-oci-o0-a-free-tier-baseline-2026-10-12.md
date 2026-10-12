# OCI O0.A — Official Always Free baseline (2026-10-12)

Status: DOC-VERIFIED; account-specific eligibility and capacity NOT VERIFIED.
Cloud mutation: NONE.

## Verified against Oracle official documentation

- **Always Free tenancy**: Ampere A1 Flex allocation equivalent to **2 OCPU
  and 12 GB RAM** (1,500 OCPU hours and 9,000 GB hours monthly). Do not confuse
  this with the broader promotional 3,000/18,000 allowance mentioned in other
  ARM documentation for differently enrolled accounts.
- Compute and boot/block resources are home-region constrained.
- 200 GB total Always Free block storage includes boot volumes. The default
  boot volume consumes around 50 GB, not extra free storage.
- A1 capacity is best effort; "out of host capacity" is a possible hard blocker.
- Free-tier accounts must not plan on charged VM/volume/network resources. The
  presence of budget alerts does not inherently prevent billing.
- The current owner has not supplied Oracle tenancy/compartment/region IDs or
  read-only quota/usage evidence. No OCI account-level conclusion is possible.

Official sources:
- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm
- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier.htm
- https://docs.oracle.com/en-us/iaas/Content/General/service-limits/default.htm

## Exit for O0.B–D

1. Read-only inspect actual home region and effective compute/volume limits.
2. Capture storage already allocated, eligible networking costs, R2 usage limits
   and existing DigitalOcean billable resources separately.
3. Choose OCI state storage strategy and confirm safe backup/restore.
4. Prove no additional required recurring service falls outside Always Free.
5. If not all checks pass, leave O0 BLOCKED; do not provision anything.

No test run on an actual OCI instance and no financial guarantee is implied.
