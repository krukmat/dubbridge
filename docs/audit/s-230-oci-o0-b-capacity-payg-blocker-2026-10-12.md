# S-230 / T6d.O0.B — OCI Free capacity and PAYG activation decision (2026-10-12)

Status: **BLOCKED — OCI A1 HOST CAPACITY, NO PAYG**.
Account mutation: **none completed**. VM creation: **none**.

## Operator evidence (supplied on 2026-10-12)

- Home region: `eu-madrid-3`, `READY`.
- A1 regional cores: limit 16, used 0, available 16. These are **regional service quotas, not Always Free entitlements**.
- A1 regional memory limit 96 GB; effective free memory usage not inspected.
- Always Free combined boot/block limit 200 GB; usage across compartments not established.
- Root compartment instance list showed no instances.
- Capacity report in `ylYt:EU-MADRID-3-AD-1`:
  - `VM.Standard.A1.Flex`, 2 OCPU / 12 GB: `OUT_OF_HOST_CAPACITY`.
  - `VM.Standard.A1.Flex`, 1 OCPU / 6 GB: `OUT_OF_HOST_CAPACITY`.

The reported free capacity is a **point-in-time status, not a reservation**. Quota does not imply hardware availability.

## PAYG discovery and correction

The operator attempted PAYG account upgrade; Oracle requested a payment-card authorization of approximately **EUR 93**; the bank rejected authorization because the required available balance was absent. Oracle's official documentation states a **USD 100-equivalent temporary authorization** is required for upgrading. This is not a subscription charge, but it **requires sufficient spendable funds at the time of authorization**.

**PAYG is not a viable immediate activation path under the owner's cash constraint.** Prior advice to upgrade without validating authorization requirements is superseded. Stop retrying the upgrade. No verified billing upgrade or OCI paid resource creation took place.

## Decision: no incremental cash required

1. Keep the existing DigitalOcean WordPress site online and unchanged. Its **pre-existing costs continue**; zero new cost does not mean zero current hosting spend.
2. Continue DubBridge integration/performance work locally; local development is **not a production-hosting replacement**.
3. Keep OCI Always Free A1 2 OCPU / 12 GB / 50 GB as the *conditional* candidate. Use `make oci-capacity-watch` from the operator Mac for a compact report. This command creates **capacity reports only**, never VM/network/volume resources.
4. If `OCI_A1_CAPACITY=AVAILABLE_NOT_RESERVED`, still re-verify free eligibility, storage/network allocations, cost controls, account IAM, and explicit owner approval before O2 provisioning. Capacity can disappear.
5. If `BLOCKED`, do **not** create a smaller unsupported production stack or buy a fallback. No auto-upgrade, no unsolicited PAYG authorization, no trial-credit workaround.
6. An E2.1.Micro Always Free VM (1 GB RAM and 1/8 OCPU) is **not** viable for WordPress + full DubBridge + media processing co-residency. Revisit only for a separately scoped and measured lightweight service.
7. Contabo and other paid providers remain **unapproved** pending explicit verification of **upfront cash, payment authorization hold, billing period/setup fees, monthly cost, actual available resources, and WordPress migration safety**. Avoid treating providers' advertised monthly price as activation cost.

## Blocker / next gates

- T6d.O0.B: **PARTIAL quota verified / A1 host capacity BLOCKED / PAYG NOT FEASIBLE**.
- T6d.O0.C: **DESIGNED, account free-usage/billing proof pending**.
- T6d.O0.D: **BLOCKED**.
- T6d.O2+ and WordPress cutover: **NOT AUTHORIZED**.
- Work without external cost: local O1 guard tests, ARM64 build validation, local benchmark preparatory work, and read-only capacity checks.

Sources:
- https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm
- https://docs.oracle.com/en-us/iaas/Content/Billing/Tasks/changingpaymentmethod.htm
- https://docs.oracle.com/en-us/iaas/tools/oci-cli/latest/oci_cli_docs/cmdref/compute/compute-capacity-report/create.html
