# S-230 — Oracle Always Free rebaseline

Date: 2026-10-12
Status: DOCUMENTATION DECISION — NO CLOUD MUTATION
Owner direction: prioritize a zero-incremental-cost POC; Oracle Always Free is
primary, Contabo is fallback requiring a separate decision.

## Active architecture and constraints

- OCI Ampere A1 ARM64, initial candidate 2 OCPU/12 GB RAM. Subject to account,
  region and capacity. Aggregate Always Free A1 quota can reach 4 OCPU/24 GB
  across eligible instances; not a reserved capacity promise.
- One VM: isolated WordPress/MariaDB and DubBridge PostgreSQL/Redis/API/Gateway/
  worker/private P2P Availability Node; only Caddy on 80/443 and restricted SSH.
- ARM64 release provenance is required; AMD64-only DOCR evidence is not enough.
- ASR/ffmpeg performance and upload memory must pass O3 measured acceptance.
- R2 private media and separate state buckets are proposed subject to zero-cost
  allowance verification and explicit state restore/single-writer safeguards.
- Desired *incremental* recurring charge: EUR 0 inside allowance limits. Existing
  DigitalOcean charge is outside that incremental target.
- T6d.2a WordPress preservation PASS carries forward. O4 owns DB/files/plugin/
  theme/admin/upload/TLS/DNS restore validation; old host stays up pending an
  explicit owner-approved cutover and subsequent retirement decision.

## Active gating

O0 confirm eligibility/free limits -> O1 OCI IaC no-apply -> O2 free VM with
explicit authorization -> O3 ARM64 full-stack benchmark -> O4 migrate WordPress
-> O5 deploy/certify coexistence and R2 -> T6e/f/g -> T7a/T7p -> T9g.

STOP on required paid component, unavailable Always Free capacity, data integrity
regression, unacceptable measured ASR performance, storage state recovery gaps,
or missing owner authorization. No automatic paid fallback.

## Preservation of historical evidence

T6a/b/c and DO T6d.0/1/2a PASS remain true provider-specific history.
Contabo C0/C1 PASS remain true design and local validation history.
Contabo C2+ work remains code/documentation artifacts, NOT an active deployment
instruction or proof of provisioning.

No Oracle resource has been created or verified under this change.
