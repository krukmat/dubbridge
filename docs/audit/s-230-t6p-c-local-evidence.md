---
type: Audit
title: "S-230 T6p-c local P2P deployment evidence"
status: in_progress
slice: S-230
task: S-230-T6p-c
date: 2026-09-26
---

# S-230-T6p-c — local P2P deployment-contract evidence

## Current disposition

**IN PROGRESS — deterministic harness implemented; Docker runtime evidence pending.**

T6p-c must not be recorded as PASS until the runtime harness emits
`T6PC=PASS` on a clean checkout. The execution environment used to author
this task does not provide Docker / Docker Compose, so no container/image/mTLS
or persistent-volume result is fabricated.

The production descriptor itself was not modified by T6p-c. Any runtime
finding that requires changing `apps/availability-node/Dockerfile`,
`infra/production/docker-compose.yml`, `.env.example`, or
`config/README.md` reopens T6p-b before certification can resume.

## RRI

Recomputed against the current `scripts/rri.py` v2 formula for the executable
T6p-c surface:

- C=2 — bounded shell harness with several fail-closed branches;
- F=1 — two implementation/evidence-script paths;
- D=3 — deployment/integration evidence;
- T=3 — multi-layer render/build/runtime/negative verification;
- A=0 — T6p-a already froze the contract;
- K=3 — Compose + mTLS + storage + image coupling;
- P=4 — secret/identity boundary is security-sensitive;
- X=3 — multiple runtime surfaces;
- penalty: `auth_security +10`;
- no architecture-decision penalty: architecture was frozen by T6p-a.

Technical profile: L=2, I=3, Q=3, V=3 -> bottleneck 3 -> ICI 75 ->
ICI-band input 70. Risk/domain input is below the ICI ceiling.

**Final RRI: 70 — Complex / Effort L.**

The owner approved the c.1–c.8 plan before implementation.

## T6p-c.1 — preflight harness

**PASS (implementation).**

Added `infra/production/p2p/preflight.sh` with three modes:

- `contract`: source-level checks without Docker;
- `runtime`: exact-head render/build/runtime certification;
- `all`: contract + runtime.

The runtime lane refuses a dirty checkout, derives the exact Git HEAD, creates
an isolated Compose project, generates ephemeral test-only mTLS material, uses
dummy non-production secrets, and cleans its isolated containers/volumes.

## Runtime attempt 1 — local Mac

Observed by the owner on 2026-09-26:

- `T6PC_CONTRACT=PASS`;
- runtime stopped before Compose render with
  `ERROR: docker compose plugin unavailable`.

This is a host-tooling limitation, not a production-descriptor failure.
The preflight now auto-detects either Compose v2 plugin form
(`docker compose`) or the standalone `docker-compose` binary. T6p-c
remains IN PROGRESS until one backend is available and the runtime lane
completes.

## T6p-c.2 — Compose render certification

**STRUCTURAL PASS / RUNTIME PENDING.**

The harness verifies on rendered Compose JSON:

- Availability Node exists;
- it has no host-published port;
- it is attached to `p2p-control`;
- `p2p-control` is not Docker-internal;
- ciphertext is AN read-only and worker read-write;
- required revision/fingerprint/KEK substitution fails closed when omitted.

Repository-level inspection at the authoring HEAD also confirms:

- no production `env_file:`;
- Availability Node has no `ports:`;
- Caddy contains no Availability Node / `:8443` route;
- the 1 CPU / 1 GiB ceiling remains present.

Docker-render evidence is still required before PASS.

## T6p-c.3 — secret-boundary certification

**STRUCTURAL PASS / RUNTIME PENDING.**

The runtime harness inspects the rendered per-service environment and asserts:

- Availability Node has no DB, Redis, Spaces, JWT, OAuth, translation,
  KEK, or worker-client-identity variables;
- worker-runner has KEK + client mTLS identity;
- API has KEK and no client mTLS identity;
- required revision/fingerprint/KEK inputs fail closed at Compose render time.

No real secret is used by the harness.

## T6p-c.4 — Availability Node image

**PASS — owner-local Docker evidence captured 2026-09-26.**

Observed:
- exact HEAD: `369792d6e7ed241e1162f67f7de4dd62bf09c076`;
- local OCI image ID:
  `sha256:6c23231778a85fae15ed5e7182a0eb711d64ca33a3af8af1cfe7fb8264a54146`;
- build completed successfully;
- `contract availability` PASS;
- `run availability` PASS;
- runtime Node/entrypoint image contract PASS;
- terminal marker: `T6PC_IMAGE_CONTRACT=PASS`.

Added the `availability` case to `scripts/test-production-images.sh`.

Contract checks cover:

- build + runtime base exactly `node:22.23.0-bookworm-slim`;
- `npm ci`;
- TypeScript build plus production dependency pruning;
- `EXPOSE 8443`;
- `ENTRYPOINT ["node", "dist/main.js"]`.

Runtime checks require the built image to report Node `v22.23.0` and the exact
entrypoint. The T6p-c runtime harness records:

- `T6PC_HEAD=<exact commit>`;
- `T6PC_IMAGE_ID=sha256:...`.

This local OCI image ID is the content-addressed image evidence consumed by the
T6p-d deployment preparation; T6p-d must additionally record the deployed
registry/runtime digest if its transport changes the identity surface.

## T6p-c.5 — runtime/network/mTLS proof

**IN PROGRESS — container started; health/network/mTLS assertions not yet observed.**

Owner-local run reached:
- Availability Node container recreate;
- container start completed.

No `T6PC_PRIVATE_NETWORK=PASS` or mTLS markers have been observed yet, so
T6p-c.5 remains open.

The harness will prove:

- Availability Node becomes container-healthy;
- the running container uses the exact built image ID;
- it is connected to exactly one network, `p2p-control`;
- `p2p-control` permits egress and port 8443 has no host binding;
- runtime Node version is 22.23.0;
- TLS minimum remains 1.3 through the server implementation;
- CA-trusted + allow-listed worker certificate reaches the HTTP handler;
- a CA-trusted but non-allow-listed client certificate is rejected with
  `service_identity_rejected`;
- a client without a certificate fails at TLS before HTTP.

## T6p-c.6 — persistent-volume proof

**PENDING LOCAL DOCKER.**

The harness will:

1. seed the ciphertext named volume from an isolated helper;
2. prove Availability Node can read it but cannot write to it;
3. write markers into drive/index volumes;
4. force-recreate the Availability Node container without deleting volumes;
5. prove ciphertext, drive, and index markers survive the recreate.

The harness cleanup uses an isolated test Compose project. It never changes the
production rule that normal restart/rollback must not use `docker compose down -v`.

## T6p-c.7 — negative/fail-closed matrix

**PARTIAL STRUCTURAL / RUNTIME PENDING.**

Implemented checks cover:

- missing `DUBBRIDGE_IMAGE_REVISION` -> Compose render failure;
- missing allowed client fingerprints -> Compose render failure;
- missing KEK material -> Compose render failure;
- wrong client fingerprint -> application-layer identity rejection;
- no client certificate -> TLS failure;
- missing Availability Node server key -> process startup failure;
- ciphertext write attempt from Availability Node -> failure.

The latter four require Docker runtime execution.

## T6p-c.8 — evidence / closure

**PENDING.**

Run from a clean checkout with Docker Desktop / Docker Compose available:

```bash
git pull
bash infra/production/p2p/preflight.sh all | tee /tmp/dubbridge-t6pc-evidence.log
```

Required terminal markers for closure:

```text
T6PC_CONTRACT=PASS
T6PC_RENDER=PASS
T6PC_SECRET_BOUNDARY=PASS
T6PC_IMAGE_CONTRACT=PASS
T6PC_PRIVATE_NETWORK=PASS
T6PC_MTLS_ALLOWED=PASS
T6PC_MTLS_WRONG_FINGERPRINT=PASS
T6PC_MTLS_NO_CLIENT_CERT=PASS
T6PC_PERSISTENCE=PASS
T6PC_NEGATIVE_MISSING_SERVER_KEY=PASS
T6PC=PASS
```

The exact image evidence has now been captured above. T6p-c still requires
the remaining runtime/network/mTLS/persistence/negative markers and final
`T6PC=PASS` before T6p-d becomes READY.

## CI context

The preceding T6p-b HEAD CI run `36235272159` was red in fmt, clippy,
workspace tests, and mobile. The logged failures point to
`apps/api/src/routes/workspace.rs`,
`apps/worker-runner/src/review_enqueue.rs`, subtitle/worker tests, and the
T7local mobile harness — none are T6p-b/T6p-c implementation paths. That red
run is therefore tracked as pre-existing/out-of-scope CI debt, not as P2P
descriptor runtime evidence.

T6p-c-triggered branch CI may still reproduce those failures; its result does
not replace the required Docker preflight evidence.

## Status

`T6p-a PASS -> T6p-b PASS -> T6p-c IN PROGRESS (local Docker evidence pending) -> T6p-d BLOCKED`.
