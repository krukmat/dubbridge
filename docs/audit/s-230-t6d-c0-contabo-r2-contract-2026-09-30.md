# S-230-T6d.C0 — Contabo/R2 go-live contract freeze

Date: 2026-09-30
Branch: `main`
Status: **PASS**
Cloud mutation: **NONE**

## Purpose

Freeze the replacement production-like POC infrastructure contract before any
Contabo or Cloudflare provisioning.

## Frozen topology

- one Contabo VPS 6-class target: 6 vCPU / 12 GB RAM / approximately 200 GB;
- Caddy owns public HTTP/TLS;
- WordPress and DubBridge share compute only;
- WordPress keeps separate DB/files/secrets and migrates first in T6d.C3a;
- DubBridge PostgreSQL and Redis are local/private for the POC;
- Availability Node remains private with the existing 1 CPU / 1 GiB ceiling;
- media/artifacts use a private Cloudflare R2 bucket;
- OpenTofu state uses a second, private R2 bucket;
- new OCI releases use GHCR and immutable digest references.

## State safety

Cloudflare documents R2 as an S3-compatible remote backend. R2 does not expose
S3 bucket versioning, therefore this POC contract does not claim state
versioning. State-changing automation must:

1. enforce a single-writer execution path;
2. fetch/copy the current state to a timestamped backup object before mutation;
3. abort if that backup cannot be verified;
4. not enable native S3 lockfile behavior until T6d.C1 proves compatible
   conditional-write semantics against R2.

## Runtime constraints

- ASR heavy-job concurrency = 1;
- implicit `large-v3` is not a valid go-live configuration;
- T6d.C3b must select and certify an explicit CPU-appropriate ASR profile;
- internal datastore/control-plane ports are never public;
- deploy/restart/rollback must preserve persistent WordPress and DubBridge data.

## Cost boundary

Planning target: approximately EUR 8–11/month before tax and variable R2 usage.
T6d.C2 records the actual checkout SKU/price before purchase.

## Exit criteria

- topology and ownership frozen: PASS;
- WordPress migration ordering frozen: PASS;
- R2 media/state separation frozen: PASS;
- GHCR release registry frozen: PASS;
- network/private-port boundary frozen: PASS;
- ASR resource policy frozen: PASS;
- cost gate frozen: PASS;
- cloud mutation: NONE.

T6d.C0 is PASS. T6d.C1 is the next executable child.
