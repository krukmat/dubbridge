---
type: Audit
title: "S-230 T6p-b production descriptor implementation"
status: complete
slice: S-230
task: S-230-T6p-b
date: 2026-09-26
---

# S-230-T6p-b — production P2P descriptor

## Result

**PASS — 2026-09-26.**

Implemented the T6p-a frozen deployment contract without changing P2.C0,
application runtime semantics, Caddy, invitations, sync, or playback.

## Implemented

- Added `apps/availability-node/Dockerfile` using Node 22.23.0, lockfile
  `npm ci`, build-time TypeScript compilation, production dependency pruning,
  and direct Node startup.
- Added private `availability-node` to production Compose with no `ports:`
  and no Caddy route.
- Added immutable revision-tag input `DUBBRIDGE_IMAGE_REVISION`; T6p-c must
  record the resulting image digest.
- Added `p2p-control` bridge shared only by worker-runner and Availability
  Node; it is not Docker-internal so Hyperswarm/DHT egress remains possible.
- Added persistent ciphertext, Hyperdrive, and publication-index volumes with
  worker RW / Availability Node RO ownership for ciphertext.
- Added production mTLS bind mounts from `DUBBRIDGE_P2P_MTLS_DIR`, outside Git.
- Added listener-only healthcheck, 1 CPU / 1 GiB ceiling, and worker dependency
  on healthy Availability Node.
- Replaced broad production `env_file` injection with explicit per-service
  environment maps.
- Real secrets are scoped to their consumers. Shared AppConfig fields required
  only by global production-schema validation use literal non-secret
  `config-validation-unused` and non-routable `config-validation.invalid`
  sentinels instead of leaking real credentials across services.
- API + worker receive the active KEK; Availability Node never does.
- Worker alone receives the client mTLS identity; Availability Node alone
  receives its server key/cert and allowed fingerprints.
- Updated `.env.example` and `config/README.md` for the production contract.

## Scope note

`scripts/test-production-images.sh` was in the permitted T6p-b path set but
was intentionally not modified. T6p-c owns local render/build/runtime evidence
and may reopen T6p-b if the descriptor or image contract fails. This preserves
the separation between authoring the descriptor and certifying it.

## RRI/workflow

Final RRI: **70 Complex / Effort L**.

Decomposition used:

1. T6p-b.1 Availability Node image descriptor.
2. T6p-b.2 production Compose/network/volume wiring.
3. T6p-b.3 explicit secret/config injection.
4. T6p-b.4 environment/operator documentation.
5. T6p-b.5 status/evidence closure.

The owner instruction to continue with the next block authorized this
decomposed T6p-b execution. The implemented surface is config/ops only;
phase-1 and phase-2 reviewer steps are n/a under the config-only exemption.
No product-source behavior was modified.

## Runtime defect amendment — 2026-09-26

T6p-c runtime evidence on Apple Silicon / Linux ARM64 exposed a production-image
defect: `rocksdb-native` could not load because the slim Node runtime image did
not contain `libatomic.so.1`. Docker therefore restarted the Availability Node
with exit code 1 before the listener healthcheck could succeed.

Per the frozen T6p-b/T6p-c ownership rule, this finding **reopened T6p-b** rather
than being patched silently inside T6p-c.

The descriptor fix is intentionally narrow:

- install Debian package `libatomic1` in the Availability Node runtime stage;
- remove apt metadata after installation;
- extend the Availability Node image contract to require the package-install
  declaration;
- extend the runtime image check to prove `libatomic.so.1` is resolvable before
  T6p-c starts the service.

Fix commits:

- `ac76856f860234e3c61f67a4430149f10c13daec` — runtime dependency;
- `a68aa37a29e17a964d9d5f0edf983e2763545b80` — image-contract regression guard.

**T6p-b disposition after amendment: PASS — RECERTIFIED 2026-09-26.** T6p-c completed the full local Docker preflight on exact tested HEAD `c4b8da98c93e80b88ed06a466226fe00273514d2`; the amended image reached healthy, passed the corrected mTLS matrix, persistence, and negative startup checks, and emitted final `T6PC=PASS`. The certified image ID is `sha256:9bc98e5590aa3a5fba1478adc99cc64a6185fde9320f0c5323cd8ad134de7d68`. The older `sha256:6c232317...` image remains obsolete.

## Next gate

**S-230-T6p-c is PASS. The T6p local lane is complete; T7a is READY for deployed backend execution.** The local certification exercised the amended descriptor and recorded the exact tested revision and image ID. Any later descriptor/source change requires a new certification cycle.
