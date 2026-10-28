# S-230 / T6d.C2 — MVP provider decision (2026-10-28)

Status: **DECISION — CONTABO CLOUD VPS 6 MONTHLY, PRE-PURCHASE BLOCKED**.
Infrastructure/billing/DNS/data mutation: **NONE**.

## Rationale and constraints

- **Selected candidate**: Contabo **Cloud VPS 6 (Core)**, 6 shared vCPU, 12 GB RAM, **200 GB SSD**, EU region, initial **1-month** prepaid term. No paid add-ons. Prefer capacity over the alternative 100 GB NVMe until actual I/O profiling.
- **Why VPS 6 vs VPS 4**: 12 vs 8 GB gives safer headroom for WordPress+MariaDB, DubBridge API/Gateway/worker/P2P, local PostgreSQL+Redis. Six shared vCPU do **not** prove measured ASR throughput or dedicated CPU performance. Restrict heavyweight media jobs to concurrency 1 and configure an explicit CPU-appropriate Whisper model, not implicit large-v3.
- **Existing reusable work**: T6d.C0 topology freeze PASS; T6d.C1 guarded Contabo OpenTofu PASS; T6d.2a WordPress preservation PASS. Product `V154` was identified in the earlier C2 planning; **re-confirm actual current SKU before ordering**.
- **Oracle fallback only**: Always Free Madrid A1 reports `OUT_OF_HOST_CAPACITY` for 2/12 and 1/6; PAYG account upgrade required a roughly EUR 93 temporary card authorization that was rejected. No further PAYG attempts under the current financial constraint.
- **Backup/rollbacks**: leave existing DigitalOcean WordPress online and billable until restore, functional validation, TLS/DNS checks and explicit cutover/retirement approval. R2 media/state buckets stay separate. Snapshots are not off-host database/state backups.

## Price and operator cash gate

- Contabo's advertised price is a **24-month effective promotional monthly rate**, not a valid single-month payable amount.
- Contabo's official Cloud VPS documentation states **1-month minimum, setup fee none** (but checkout is final authority). A September 2026 price comparison quotes approximately **EUR 9/month for a one-month VPS 6 term**; this estimate is **NOT a verified checkout price or guarantee of no card authorization**.
- Compare alternative: OVHcloud VPS-3, 6 vCore / 12 GB / 100 GB NVMe, advertised from EUR 12.58/month incl. Spanish VAT with one-day automated backup (checkout/payment still unverified).
- **Hard stop** if checkout total payable today is above **EUR 15 incl. VAT**, if 12/24-month prepayment is required, if deposits/holds/activation add-ons apply, if recurring total is above EUR 15 for the VPS, or if terms/rate/cancellation are not clearly visible. Owner must review and approve a specific final invoice before purchase. The limit is a decision gate, not evidence that funds are available.
- The displayed promotional rate, the reported EUR 9 estimate, and the existing DigitalOcean spend must **not** be combined or confused.

## Reordered next steps (C2)

1. **C2b — first: zero-mutation purchase feasibility**. Confirm current SKU/product ID, EU location, 1-month period, exact VAT-inclusive payment today, monthly renewal, payment method/authorization hold, setup fee, included IPv4, OS image/SSD option, SSH secret, operator /32, and terms. If any unavailable or above the ceiling, remain blocked and compare OVH VPS-3 without purchase.
2. **C2a — second: remote-state preflight**. Verify Cloudflare R2 account free-tier and payment/verification requirements before any bucket creation. If acceptable, create dedicated **private** state bucket, set up guarded S3-compatible backend, single writer and independently restorable timestamped state backup. No credentials committed.
3. **C2c — third: owner-authorized exact-address plan and provision**. Require C2b and C2a PASS and explicit owner confirmation. Review plan for only the VPS/firewall; no unsolicited cloud or data mutation.
4. **C3–C5**: first recover WordPress in isolation and benchmark local Postgres/Redis + DubBridge + ASR under concurrent load; cutover only after separate approval.

## Evidence

- Repo `docs/audit/s-230-t6d-c0-contabo-r2-contract-2026-09-30.md`, `docs/audit/s-230-t6d-c1-contabo-opentofu-2026-09-30.md`, `docs/audit/s-230-oci-o0-b-capacity-payg-blocker-2026-10-12.md`.
- Official: https://docs.contabo.com/docs/products/cloud-vps/ ; https://contabo.com/es/vps/ ; https://www.ovhcloud.com/es-es/vps/
- Dated independent price estimate: https://toolradar.com/tools/contabo (September 28, 2026 prices).
