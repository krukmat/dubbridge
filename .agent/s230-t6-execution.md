# S-230 T6 — ACTIVE Oracle Always Free execution context

## ACTIVE — Oracle Always Free go-live rebaseline (2026-10-12)

This decision **supersedes Contabo as the deployment target**, without
invalidating its prior provider-scoped implementation/validation. DigitalOcean
remains historical. No infrastructure, DNS, billing or existing service is
changed as part of this documentation update.

- Primary OCI Ampere A1 ARM64 Always Free, initial candidate **2 OCPU / 12 GB**
  (subject to free-tier eligibility/region capacity). Aggregate A1 allowance
  up to 4 OCPU/24 GB; free boot/block volumes and egress require accounting.
- One VM for isolated WordPress + MariaDB and DubBridge + PostgreSQL, Redis,
  private Availability Node and Caddy. Only 80/443 public; SSH restricted.
- Private Cloudflare R2 media and separate state storage only after verifying
  free quota usage and state recovery/concurrency; OCI native alternatives can
  be evaluated within O0 if genuinely zero cost.
- GitHub Container Registry **linux/arm64**, immutable OCI digests. Prior
  amd64 artifacts do not establish ARM64 runtime compatibility.
- Explicit CPU ASR profile and concurrency 1; O3 must benchmark worker/ffmpeg,
  native libraries, upload memory peak and WordPress+DubBridge co-residency.
- WordPress backup T6d.2a stays authoritative. OCI O4 owns restoration,
  functional validation and reversible cutover; old DigitalOcean host stays
  live until owner approves shutdown in a separate step.
- Incremental target **EUR 0/month** only when all services remain inside
  free allowances; pre-existing DigitalOcean billing remains separate.
- No automatic Contabo purchase, paid upgrade, provisioning or DNS change.
- Active dependencies: **T6d.O0 → O1 → O2 → O3 → O4 → O5 → T6e → T6f
  → T6g → T7a → T7p → T9g**. Provisioning at O2 requires explicit
  owner approval. Historical C0/C1 PASS are NOT Oracle PASS.

## History

## Historical Contabo execution packet (superseded; not executable)

Use this file as the first context for T6 execution. Do not read the full
S-230 history unless a concrete blocker requires it.

## Current gate

- T5: PASS.
- T6p local lane: CLOSED through T6p-c PASS.
- T6a/T6b/T6c: PASS historical DigitalOcean preparation/release evidence.
- T6d.0/T6d.1: PASS historical DO state/inventory evidence.
- T6d.2a: PASS — WordPress backup verified locally.
- T6d.3: SUPERSEDED, never executed.
- **T6d.C0: PASS 2026-09-30 — Contabo/R2 contract frozen.**
- Next executable child: **T6d.C1**.
- T7a remains BLOCKED until aggregate **T6 PASS**.

## Active go-live decisions

- compute: one Contabo VPS 6-class target, 6 vCPU / 12 GB / ~200 GB;
- WordPress and DubBridge share the host but remain isolated by hostname,
  databases, writable roots and secrets;
- T6d.C3a migrates WordPress before T6d.C3b deploys DubBridge;
- Caddy is the only public HTTP/TLS edge (80/443);
- SSH is operator-restricted;
- API, PostgreSQL, Redis and Availability Node 8443 are private;
- DubBridge PostgreSQL and Redis are local to the VPS for the POC;
- Cloudflare R2 is private S3-compatible application storage;
- a separate private R2 bucket stores OpenTofu state;
- R2 state is single-writer and must be snapshotted to a timestamped backup key
  before each state-changing operation; do not assume bucket versioning or
  lockfile semantics without proof;
- new immutable OCI releases publish to GHCR and are deployed by digest;
- production never builds images on the VPS;
- ASR heavy concurrency is 1; implicit large-v3 is not an accepted production
  setting;
- recurring-cost planning envelope is approximately EUR 8–11/month before tax
  and variable R2 usage.

## Active child sequence

```text
T6d.C0 PASS
  -> T6d.C1 Contabo OpenTofu descriptor + guarded plan (NO APPLY)
  -> T6d.C2 provision VPS
  -> T6d.C3 single-host runtime layout
  -> T6d.C3a migrate/validate WordPress
  -> T6d.C3b deploy DubBridge coexistence
  -> T6d.C4 R2 application storage
  -> T6d.C5 readiness + rollback checkpoint
  -> T6e deploy/migrate/readiness
  -> T6f real-video E2E
  -> T6g operational closeout
  -> T6 PASS
  -> T7a
```

T6d.C4 may be implemented in parallel after T6d.C2 but T6d.C5 requires both
C3b and C4 PASS.

## Provider-neutral agent interface

Later children implement this public surface:

```text
make infra-plan
make infra-provision
make deploy REV=<git-sha>
make smoke
make status
make rollback RELEASE=<release-id>
```

Provider-specific Contabo/Cloudflare commands stay behind these targets.

## Stop rules

Stop before mutation when:

- exact Contabo SKU/price exceeds the frozen envelope without owner review;
- required credentials/inputs are missing;
- plan contains unexpected destroy/replace;
- state backup cannot be proven before a state-changing operation;
- release does not resolve to exact GHCR OCI digests;
- WordPress migration/rollback proof is incomplete before DubBridge coexistence;
- a required secret or network boundary cannot be satisfied.

During deploy/smoke, fail closed on migration, health, TLS, network-boundary,
R2 round-trip, WordPress regression, or authoritative downstream-state failure.
A 2xx alone is not evidence.

Do not patch unrelated product code inside T6 to force deployment evidence to
pass. Record the finding and return to its owning task.

## Runtime certification lineage

T6p-c's exact locally runtime-certified P2P baseline is
`c4b8da98c93e80b88ed06a466226fe00273514d2`, Availability Node image
`sha256:9bc98e5590aa3a5fba1478adc99cc64a6185fde9320f0c5323cd8ad134de7d68`.
If the new deployment descriptor changes a T6p-c-certified runtime/config
surface, rerun the bounded local certification before deployed P2P
certification.
