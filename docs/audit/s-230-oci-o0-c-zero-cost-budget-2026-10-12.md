# S-230 / T6d.O0.C — Zero incremental cost envelope

Date: 2026-10-12
Status: DESIGNED / ACCOUNT-VERIFICATION PENDING
Mutation: NONE

## Cost scope

"EUR 0 incremental" means **new recurring OCI + R2 charges**, not total
project cost. Existing DigitalOcean WordPress Droplet, Container Registry,
Spaces/state buckets and any other subscribed product are accounted separately
until explicit, safe retirement. A zero amount cannot be guaranteed solely
from a configured budget alarm.

## Oracle Always Free baseline

Official OCI documentation states that an Always Free tenancy receives the
equivalent of **2 Ampere A1 OCPUs and 12 GiB RAM** (1,500 OCPU hours and
9,000 GiB hours monthly) in the **home region**, subject to capacity. OCI
Always Free Block Volume provides **200 GB total across boot and block**; the
boot disk consumes that allocation. Other tenancies/promotions may differ.
Do not use the separate Oracle general price-list 3,000/18,000 allowance
to size this Always Free POC.

Initial candidate (conditional): **one A1.Flex 2 OCPU / 12 GB VM** and
**50 GB boot volume**; no additional block volume until real usage
and limits have been checked. This leaves nominal storage headroom but cannot
guarantee resource quota if the tenancy already uses free storage/compute.

## Cost guardrail matrix

| Resource / dimension | Default POC limit | Pre-activation proof | Blocking condition |
|---|---|---|---|
| OCI A1 compute | One A1.Flex 2 OCPU/12 GB | Home-region eligibility, 0 used free OCPU hours/GiB hours, available capacity | Any other paid shape or exhausted allowance |
| OCI volumes | 50 GB boot, no additional volume | Actual current boot+block allocation <= 200 GB including candidate | Total would exceed free allocation |
| OCI network | One IPv4 public edge on A1, VCN/subnet/IGW | Validate actual free allowances for public IPv4, egress, any network features | A mandatory paid service or unclear line item |
| OCI security | 80/443 public; SSH restricted to operator CIDR | No public DB, Redis or P2P mTLS ports; plan JSON inspection | Broad SSH, unknown or additional exposed ports |
| Cloudflare R2 | Two private buckets: media and remote state | Up to 10 GB-month storage, 1M Class A / 10M Class B operations, as applicable to selected storage class | Free limits exceeded or billing visibility absent |
| Video/media | Keep media footprint below **8 GB** as local planning warning threshold | Quantify upload size, lifecycle/retention and request volumes; R2 usage checked | Evidence indicates quota exhaustion |
| State backup | Dedicated, private, restorable snapshots | Controlled writer, timestamped offsite copies, restore drill | Unable to recover state or prove writer isolation |
| CI/registry | GHCR digest-pinned Linux ARM64 | Verify repo/package permissions and any billable usage | Unplanned private-registry or runner charges |
| Oracle account | Read-only tenancy/billing limits, alerts | Capture official usage/cost dashboard and alarm evidence | Cannot distinguish trial credits from Always Free |
| Existing DO | Keep running until WordPress acceptance | Billing inventory and retirement plan, independent of zero-incremental target | Treating remaining DO costs as zero |

**Important:** OCI budget alerts are observation tools, not billing hard caps.
O2 provisioning needs account-specific free-tier proof AND explicit owner approval.
No automated conversion to PAYG, no paid shape fallback, no network upgrade.

## Missing inputs to finish O0.C

- Oracle home region, tenancy type, compartment IDs, currently-used OCPU/RAM,
  boot/block and network allocations: **NOT ACCESSIBLE**.
- Cloudflare account plan and existing R2 consumption/billing: **NOT ACCESSIBLE**.
- DigitalOcean invoice/subscription inventory: **NOT RE-VERIFIED**.
- Actual release registry billability and ARM64 CI runner cost: **NOT TESTED**.

## Stop and escalation

Mark O0.C **DESIGNED**, not PASS, until the account inventory resolves every
unknown above. If a free resource is unavailable, issue a budget/capacity NO-GO.
Do not buy or provision a replacement without a separate approval.

References:
- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm
- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier.htm
- https://developers.cloudflare.com/r2/pricing/
- https://www.oracle.com/es/cloud/price-list/
