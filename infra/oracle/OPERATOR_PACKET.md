# T6d.O1.D — Oracle Always Free low-context execution packet

Status: DRAFTED; execution evidence pending.
Date: 2026-10-12
Mutation budget: ZERO in O0/O1.

## Fixed gate sequence

O0.A DOC VERIFIED → O0.B actual tenancy inventory → O0.C budget inventory
→ O0.D zero-charge owner freeze → O1.A–D OCI IaC/guard proof
→ O2 authorized OCI activation → O3 ARM64 benchmark
→ O4 WordPress migration → O5 DubBridge/R2 certification.

**O1 work may be prepared in parallel with O0.B/C, but O1 cannot be treated as
approved for provisioning until O0.D passes.**

## Read-only / local commands

0. `make oci-account-inspect` (OCI CLI, **read-only**, after local login;
   partial home-region/AD/A1 resource-limit inventory, not a PASS proof).
1. `make oci-guard-tests`: negative synthetic JSON plan tests, no network.
2. `make oci-plan-guard OCI_PLAN_JSON=<local-plan.json>`: exact-create guard,
   with `OCI_ALLOW_CREATES` listing every expected resource address. This
   command reads a pre-existing JSON plan, never creates one.
3. `tofu init -backend=false -input=false` / `tofu validate` **ONLY for
   local provider/schema certification**. Do not invoke `plan` or `apply`
   against local state.
4. State bootstrap: use separate, private R2 backend only after its account,
   IAM credentials, remote backup and writer coordination are frozen in O0.D.
   The previous Contabo `backend.hcl` does not authorize use of its state.

## OCI resource contract (O1.A/B)

Default `enable_resources=false`; no desired cloud resources.
The only allowed initial creation addresses (after O2 owner authorization):

```
oci_core_vcn.poc[0]
oci_core_internet_gateway.poc[0]
oci_core_route_table.poc[0]
oci_core_security_list.poc[0]
oci_core_subnet.poc[0]
oci_core_instance.poc[0]
```

The candidate VM is exactly `VM.Standard.A1.Flex` / 2 OCPU / 12 GB /
50 GB boot. No managed DB, NAT gateway, load balancer, extra volumes,
billed OCI AI service or additional VM is provisioned by this descriptor.
On the public subnet, security list allows 80/443 from internet and SSH
from an exact operator /32; PostgreSQL/Redis/P2P mTLS remain private.
The plan guard blocks any update/delete/replace and any unapproved create.
Resource plan alone does not prove account free-tier eligibility.

## ARM64 release/worker contract

- Rebuild or fetch each API, Gateway, worker, migration and Availability Node
  image as `linux/arm64`, preferably GHCR `@sha256` immutable digest.
- Validate native dependencies including Node/libatomic, RocksDB/Hyperdrive
  and Python/faster-whisper/ffmpeg on real ARM64 hardware.
- Freeze explicit smaller CPU ASR model (benchmark selection in O3) and only
  one heavy concurrent job. Do not silently reuse CPU-heavy `large-v3`.
- Validate persisted ciphertext, drive/index volumes, WordPress DB/files,
  PostgreSQL, Redis and TLS ingress routing.
- Keep DigitalOcean WordPress operational until O4 restored data and
  owner-authorized cutover + rollback proof. No cleanup in O1/O2.

## Stop conditions

Stop if: unknown paid SKU, no Always Free capacity, wrong home region,
allocated 2 OCPU/12 GB already in use, combined boot/block >200 GB,
R2 usage would exceed free quota, plan has unapproved address/action,
public DB/Redis/8443 exposure, state cannot be restored, GHCR ARM64 digests
cannot be verified, or owner has not explicitly authorized O2.

## Evidence format

Emit stable markers only after independently verified actions:
`OCI_STATIC=PASS`, `OCI_PLAN_GUARD=PASS`,
`OCI_SCHEMA=PASS`, `OCI_QUOTA=PASS`, `OCI_BUDGET=PASS`.
A synthetic guard test never implies `OCI_QUOTA` or `OCI_PROVISION=PASS`.
