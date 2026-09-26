---
type: Audit
title: "S-230 T6p-a deployment ownership and configuration freeze"
status: complete
slice: S-230
task: S-230-T6p-a
date: 2026-09-26
---

# S-230-T6p-a — deployment ownership/configuration freeze

## Result

**PASS — 2026-09-26.**

T6p-a freezes only deployment-specific ownership and configuration for the
already-implemented MVP0-P2P publication plane. It consumes P2.C0 and the
implemented P2 surfaces without redefining package, crypto, publication,
audit, invitation, mobile-sync, playback, or `P2P_READY` semantics.

Activation inputs were satisfied before this freeze:

- `S-230-T7local CLOSED — OWNER ACCEPTED`;
- `S-230-T7c PASS`;
- MVP0-P2P `DEV-HANDOFF SATISFIED`, pinned at `84ea5edc`;
- P2.C0 remains a satisfied contractual input.

The owner-approved T7local amendment is preserved exactly: C4 is
owner-accepted and E4 freshness is owner-waived. This audit does not relabel
either item as technical runtime evidence.

## T6p-a.1 — runtime and path inventory

Current production state inspected at task start:

- `infra/production/docker-compose.yml` has caddy, redis, migration, api,
  gateway, and worker-runner, but no Availability Node.
- `apps/availability-node/` is implemented and tested, but has no production
  Dockerfile.
- `.env.example` already documents the runtime P2P variables.
- `infra/local/docker-compose.yml` proves the local topology: worker ->
  private HTTPS/mTLS Availability Node, shared ciphertext package root,
  persistent Hyperdrive/index storage.
- `apps/availability-node/src/bootstrap.ts` owns bind host/port, package
  root, drive/index roots, TLS material, allowed client fingerprints, and
  Hyperswarm join timeout.
- `apps/worker-runner/src/p2p_publication_runtime.rs` owns the private
  Availability Node URL, CA/client identity, ciphertext root, lease/retry
  timing, dispatch interval, and attempt budget.
- `apps/worker-runner/src/p2p_activation.rs` and
  `apps/api/src/routes/p2p_envelope.rs` both consume the same active
  KEK id/version/material.

No runtime source change is required by T6p-a itself.

## T6p-a.2 — Availability Node placement and image freeze

Frozen deployment contract:

1. Availability Node is a distinct service in the existing single-Droplet
   production Docker Compose deployment.
2. Its publication-control listener is private to the Compose network.
3. Internal service address is `https://availability-node:8443`.
4. It is **not** routed by Caddy and has **no host-published port**.
5. Container bind host is `0.0.0.0`; isolation is provided by Compose
   network membership plus absence of a published port.
6. Production image is built from a new
   `apps/availability-node/Dockerfile`.
7. Node runtime is pinned to the implemented package requirement:
   `22.23.0`; the Docker base must use the matching
   `node:22.23.0-bookworm-slim` line.
8. Dependencies are installed with `npm ci` from the lockfile and TypeScript
   is compiled at image build time. Production startup must not run
   `npm install`/`npm ci`.
9. Deployable image identity is immutable: tag with the exact Git revision;
   T6p-c records the resulting image digest and T6p-d deploys that digest.
10. Restart policy: `unless-stopped`.
11. POC resource ceiling for Availability Node:
    `cpus: "1.0"`, `mem_limit: 1g`. This is compatible with the already
    frozen 8-GB single-droplet target and leaves the CPU/RAM-heavy worker as
    the dominant workload.

T6p-b owns creation of the Dockerfile and production Compose wiring.

## T6p-a.3 — mTLS identity and rotation freeze

Identity ownership is frozen as follows:

| Material | Consumer | Container path | Access |
|---|---|---|---|
| private P2P CA certificate | Availability Node + worker-runner | `/run/dubbridge/p2p-mtls/ca.pem` | read-only |
| Availability Node server key | Availability Node only | `/run/dubbridge/p2p-mtls/server-key.pem` | read-only, secret |
| Availability Node server certificate | Availability Node only | `/run/dubbridge/p2p-mtls/server-cert.pem` | read-only |
| worker client cert + private key identity | worker-runner only | `/run/dubbridge/p2p-mtls/client-identity.pem` | read-only, secret |
| allowed worker certificate fingerprints | Availability Node only | environment value | non-secret identifier |

Production secret material is provisioned outside Git and mounted read-only.
The local generator under `infra/local/p2p/` remains local-only and is not
promoted into the production deployment path.

Rotation policy for this POC:

- the private CA remains stable for the POC window;
- server leaf certificates may rotate under the same CA without changing
  worker trust;
- client rotation uses the already-supported comma-separated fingerprint
  allow-list: add old+new fingerprints, deploy the new worker identity, prove
  publication, then remove the old fingerprint;
- no CN/SAN authorization fallback is introduced; authorization remains the
  SHA-256 client-certificate fingerprint after a trusted TLS handshake;
- TLS remains minimum v1.3 as implemented.

## T6p-a.4 — versioned KEK boundary freeze

The K1 KEK boundary is frozen without changing the P2.C0 crypto contract:

- `DUBBRIDGE_P2P_KEK_HEX`, `DUBBRIDGE_P2P_KEK_ID`, and
  `DUBBRIDGE_P2P_KEK_VERSION` are available only to API and worker-runner.
- Availability Node never receives plaintext CK, wrapped CK, KEK material,
  KEK identifiers as secret configuration, or device-envelope state.
- POC v1 freezes one active KEK tuple for the release window:
  `KEK_ID=<operator assigned>`, `KEK_VERSION=1`.
- Mid-POC KEK rotation is **not supported** by the current runtime because
  persisted lineages are resolved against one configured KEK id/version.
  A different configured version fails closed.
- Therefore routine KEK rotation is out of T6p-b/c/d scope. Emergency
  rotation blocks publication/release until a separate multi-KEK resolver or
  explicit lineage-migration task is approved and certified. No silent
  re-encryption or lineage mutation is permitted.

This makes the current runtime limitation explicit instead of pretending the
single-resolver implementation already supports online rotation.

## T6p-a.5 — persistent ciphertext storage freeze

Production Compose will own three persistent named volumes:

| Volume | worker-runner | Availability Node | Purpose |
|---|---|---|---|
| `p2p-ciphertext-data` | RW | RO | sealed K1 ciphertext package materialized by the backend |
| `p2p-availability-drive-data` | none | RW | persistent Hyperdrive/Corestore data |
| `p2p-availability-index-data` | none | RW | stable publication index/evidence |

Container paths remain the implemented defaults:

- `/var/lib/dubbridge/p2p-ciphertext`;
- `/var/lib/dubbridge/p2p-drive`;
- `/var/lib/dubbridge/p2p-index`.

The volumes survive container restart/recreate and ordinary
`docker compose down`. Production runbooks must not use `down -v` or
otherwise delete these volumes as part of a normal restart/rollback.

No plaintext media, plaintext CK, KEK, JWT signing material, DB credentials,
or service private key may be persisted in these volumes.

## T6p-a.6 — network, health, ports, and resources freeze

Network topology:

- introduce one dedicated Compose bridge network named `p2p-control`;
- connect only `worker-runner` and `availability-node` to it;
- worker-runner may retain its existing application network connectivity;
- Availability Node receives no Caddy/public-network route;
- `p2p-control` is **not** marked Docker `internal`, because Availability
  Node requires outbound Internet connectivity for Hyperswarm/DHT;
- absence of a host port keeps the publication-control endpoint private.

Health semantics:

- container health checks only prove the Availability Node process/listener
  is alive on internal port 8443;
- the health check is not a publication-success/readiness oracle;
- Availability Node reachability never means `P2P_READY`;
- PostgreSQL same-lineage confirmation remains the sole authoritative
  `P2P_READY` predicate.

Frozen runtime values retained from the implemented configuration unless a
later evidence task proves they are invalid:

- publication-control port: `8443`;
- HTTP timeout: `10s`;
- claim lease: `30s`;
- retry delay: `5s`;
- dispatch interval: `1000ms`;
- maximum attempts: `5`;
- Hyperswarm join timeout: `15000ms`;
- Availability Node resource ceiling: `1 CPU / 1 GiB`.

## T6p-a.7 — secret exposure matrix

T6p-b must implement a service allow-list; it must not add the Availability
Node to the current broad shared `env_file` pattern.

| Secret/config class | migration | api | gateway | worker | availability-node |
|---|---:|---:|---:|---:|---:|
| PostgreSQL credentials | yes | yes | no | yes | **no** |
| Redis credentials | no | yes | no | yes | **no** |
| Spaces credentials | no | yes | no | yes | **no** |
| JWT signing secret | no | yes | no | no | **no** |
| OAuth client secret | no | no | yes | no | **no** |
| translation provider secret | no | no | no | yes | **no** |
| K1 KEK material/id/version | no | yes | **no** | yes | **no** |
| worker mTLS client identity | no | no | no | yes | **no** |
| Availability server private key | no | no | no | no | yes |
| P2P CA certificate | no | no | no | yes | yes |
| allowed client fingerprints | no | no | no | no | yes |

The source file may remain operator-managed, but Compose must explicitly
inject only the variables required by each service. A shared source file is
not authorization to expose every value to every container.

## Exact writable-path ownership after the freeze

### T6p-a — this freeze

Only status/evidence documents are writable:

- `docs/audit/s-230-t6p-a-input-freeze-2026-09-26.md`
- `docs/audit/s-230-t6p-a-rri-2026-09-26.md`
- `docs/tasks/s-230-poc-v1-digitalocean.md`
- `docs/plan/s-230-poc-v1-digitalocean.md`
- `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
- `docs/plan/roadmap.md`
- `docs/audit/go-live-octubre-2026-mirror.html`

### T6p-b — production descriptor implementation

Writable implementation paths are frozen to:

- `apps/availability-node/Dockerfile` (new)
- `infra/production/docker-compose.yml`
- `.env.example`
- `config/README.md`
- `scripts/test-production-images.sh`
- its own audit/status documents

`infra/production/Caddyfile` is explicitly **not writable** for T6p-b:
Availability Node must not acquire a public proxy route.

T6p-b must implement the service-specific environment allow-list rather than
adding Availability Node to a broad shared `env_file`.

### T6p-c — local deployment-contract evidence

Writable implementation/evidence paths are frozen to:

- `infra/production/p2p/preflight.sh` (new)
- `scripts/test-production-images.sh` only if the T6p-b image-contract case
  needs an evidence-only correction;
- `docs/audit/s-230-t6p-c-local-evidence.md` (new)
- S-230 status documents.

Any production-descriptor defect discovered here reopens T6p-b; T6p-c does
not silently patch production wiring and then certify its own fix.

### T6p-d — DigitalOcean deployment evidence

No planned product/config source write is authorized. Writable paths are:

- `docs/audit/s-230-t6p-d-do-deployment.md` (new)
- `docs/tasks/s-230-poc-v1-digitalocean.md`
- `docs/plan/s-230-poc-v1-digitalocean.md`
- `docs/plan/roadmap.md`
- `docs/audit/go-live-octubre-2026-mirror.html`

A deployment finding that requires config/source modification reopens T6p-b
or T6p-c and requires re-certification before T6p-d resumes.

## Boundary checks

T6p-a explicitly does **not**:

- create the production Availability Node Dockerfile;
- edit production Compose;
- deploy to DigitalOcean;
- expose the publication endpoint through Caddy;
- redefine P2.C0, K1, `availability-publication-v1`, or `P2P_READY`;
- alter invitation/device-envelope/mobile-sync/playback semantics;
- claim invited playback or product go-live;
- fabricate T7local freshness evidence.

## Closure

All deployment-specific decisions required by the T6p-a contract are frozen:
placement, immutable-image strategy, mTLS identity and leaf rotation, KEK
policy, persistent volumes, private network/port ownership, health semantics,
resource ceiling, secret exposure, and exact writable paths for T6p-b/c/d.

**Next executable block: `S-230-T6p-b`.**
