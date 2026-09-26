---
type: Audit
title: "MVP0-P2P T5/T6 documentation gap and local-dev-readiness findings"
status: open
---

# T5/T6 documentation gap and local-dev-readiness findings — 2026-09-14

Scope: read-only analysis recorded for a future session. No source, runtime
configuration, architecture decision, release approval, commit, or deployment
is authorized by this record.

## Context

`docs/tasks/mvp0-p2p-p2-encrypted-publication.md` and `docs/plan/roadmap.md`
were reviewed against `feature/p2p-mvp-core` git history after ~90 commits by
`Matias Kruk <kruk.matias@gmail.com>` landed directly on the branch on
2026-09-14 (07:48–13:45), outside this agent session. Owner directive for the
next session: making the P2 P2P runtime **local-dev-ready is priority #1**,
ahead of the documentation remediation below.

## Finding 1 — T5/T6 status ledger is stale relative to implemented code

- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md:4750-4772`: T5a-T5d and
  T6a-T6e are listed status **"Planned"**.
- `docs/plan/roadmap.md:402`: "T5a-d, T6a-e remain Planned (11 leaves with no
  code started)" — contradicted by the git log below.
- The 3 commits pulled at the end of this session (`a9e6403`, `7e956f8`,
  `ed4e73a`, all 2026-09-14 13:50-14:09) are security/zeroization fixes to
  already-implemented P2P activation code — they do **not** touch either doc
  file, so this gap is unchanged as of the latest pull.

### Implemented scope found in git log (day range `d4fdb01..95e36da`)

61 files changed, +5761/-297 lines, 4 new SQL migrations (0034 claim leases,
0035 ready descriptor evidence, 0036 audit correlation, 0037 package seal
audit). `cargo check --workspace --all-features` compiles clean at HEAD.

- **T5** (S-120 integration + `P2P_READY`): T5a activation characterization
  tests (`0235fba`, `b630132`); T5b activation seam post-Ready (`57de5c0`,
  `6c18bf3`, `4cb82d8`); T5c ready-descriptor domain+repo+migration
  (`899427e..533eb71`, `4aa89ce`); T5d non-regression coverage (`2406402`,
  `7ef0503`, `ee114d9`).
- **T6** (audit + crash-window certification): T6a audit correlation
  migration+shape (`0bd4fb1`, `5fecd1a`, `bf40733`, `6109ad9`, `0fed4fe`,
  `19cf5c5`, `29beab1`); T6b/T6c crash recovery window certification
  (`dbdf82c`, `d156cc0`, `b04e5d8`); T6d ciphertext-only/secret-boundary
  certification (`2d23c11`, `f33f8d3`, `eb24e3f`, `f981e32`, incl.
  `apps/availability-node/test/secret_boundary.test.ts`); package-seal
  audited transitions (`6e3ed79`, `458d1d1`, `4c9edbb`, `6a8567b`, `eb39293`,
  `2ed2858`).

### Governance gaps against `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` / `docs/policies/HITL_AUTONOMY_POLICY.md`

1. Schema migrations (0034-0037) are in HITL policy's "Always requires
   explicit approval" list — no RRI or approval-card evidence found in any
   commit message.
2. No commit message references RRI, band, reviewer (Gemma/gpt-oss/D14), or
   an approved task card.
3. No `docs/audit/gemma-evidence/` artifact exists for T5/T6 (one exists for
   T4b).
4. No `### Reflection log`, `### Behavioral coverage certification`, or
   `### Owner final verification` block exists for T5/T6 — the ledger still
   shows them `Planned`, so none of the closure blocks were ever added.
5. "Sync status artifacts before reporting completion" was not followed —
   `roadmap.md` still asserts 11 leaves with no code.
6. No Compact Approval Task Card v2 exists in-repo for T5/T6.

### Precedent already in-repo: T4b-T4f retrospective closure

`docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § "P2.T4b-T4f retrospective
integrated closure record — Done 2026-09-14" (commit `6a6d0c7`, 12:07) shows
the correct remediation pattern: explicit owner waiver cited in text, a JSON
review artifact (`docs/audit/mvp0-p2p-p2-t4b-retrospective-review.json`),
partial Reflection log, HP/EC table, and one documented non-blocking residual
(`P2.T4e-cov`). T5/T6 (implemented 08:03-13:45, straddling T4's 12:07 waiver)
have no equivalent commit.

### Recommended remediation (secondary priority — after local-dev-readiness)

1. Confirm with the owner whether T5/T6 had out-of-band authorization.
2. Run the full test suite (`cargo test --workspace --all-features` /
   `make qa-test`) — only `cargo check` has been run so far.
3. Verify T5/T6 code against original acceptance criteria line-by-line.
4. Run retroactive band-routed peer review (Gemma/gpt-oss/D14 per RRI band).
5. Close T5a-T5d/T6a-T6e in the ledger mirroring the T4b-T4f pattern:
   Reflection log, behavioral coverage certification, owner verification.
6. Correct `docs/plan/roadmap.md:402`.
7. Review for possibly-missing tasks — the Availability Node bootstrap
   itself (Finding 2 below) is a strong candidate for a new task, since it
   was never in T5/T6's original scope.

## Finding 2 — P2P runtime is not local-dev-ready (owner-declared priority #1)

Concrete blockers found by direct code read, in order of severity:

### Blocker A — Availability Node has no entrypoint (root blocker)

`apps/availability-node/src/server.ts` (237 lines, read in full) exports
`handlePublicationIngress`, `createPublicationHandler`,
`createPrivatePublicationServer` — but contains **no `.listen()` call, no env
var reads, no bootstrap/main function**. `package.json`'s
`"start": "node dist/server.js"` would run a file with no executable
entrypoint; the binary cannot start at all today, locally or otherwise.

The real executor exists and is fully implemented but never wired in:
`apps/availability-node/src/publication_executor.ts` —
`createPublicationExecutor(config)` composes `verifyPackage`,
`withPublicationLock`, `openDrive`/`closeDrive`/`flushDrive`/
`announceOnSwarm`, `decideAndPersist`. `mtls.ts` defines
`PrivateMtlsCredentials` (real `key`/`cert`/`ca`) and
`createClientFingerprintPolicy(allowedFingerprints)`.

### Blocker B — Rust client requires real HTTPS/mTLS certs, no dev bypass

`crates/connectors/src/p2p_availability.rs:110-148`
(`AvailabilityPublicationClient::from_mtls_pem`) strictly rejects any
non-`https://` scheme and requires valid CA + client-identity PEM material.
There is no local/insecure mode — local development needs real generated
dev-certs, not a config flag.

### Blocker C — No docker-compose service

`infra/local/docker-compose.yml` (read in full) defines postgres, redis,
minio, minio-init, api, worker-runner, asr-worker-py, translation-worker-py,
tts-worker-py — **no `availability-node` service**.

### Blocker D — Undocumented environment variables

`apps/worker-runner/src/p2p_publication_runtime.rs:21-45` reads 9 env vars
directly via `std::env::var` (bypassing `crates/config`'s typed fail-closed
loader): `DUBBRIDGE_P2P_AVAILABILITY_URL`, `DUBBRIDGE_P2P_AVAILABILITY_CA_PEM`,
`DUBBRIDGE_P2P_AVAILABILITY_IDENTITY_PEM`, `DUBBRIDGE_P2P_CIPHERTEXT_ROOT`,
`DUBBRIDGE_P2P_HTTP_TIMEOUT_SECS` (default 10),
`DUBBRIDGE_P2P_LEASE_SECS` (default 30), `DUBBRIDGE_P2P_RETRY_SECS`
(default 5), `DUBBRIDGE_P2P_DISPATCH_INTERVAL_MS` (default 1000),
`DUBBRIDGE_P2P_MAX_ATTEMPTS` (default 5). None appear in `.env.example`,
`config/local.toml`, or `config/README.md`. If `DUBBRIDGE_P2P_AVAILABILITY_URL`
is absent, the runtime returns `Ok(None)` (inert) — fail-safe but not
fail-closed via typed config, and undocumented either way.

### Suggested local-dev-ready work order

1. Build the Availability Node bootstrap/entrypoint (wire
   `publication_executor.ts` + `mtls.ts` into `server.ts`'s missing
   `.listen()`).
2. Generate local mTLS dev certificates (CA + server + client identity) and
   document the generation step.
3. Add an `availability-node` service to
   `infra/local/docker-compose.yml`.
4. Document the 9 `DUBBRIDGE_P2P_*` env vars in `.env.example`,
   `config/local.toml`, `config/README.md`.
5. Run an end-to-end local smoke test (worker-runner dispatcher → mTLS →
   Availability Node → Hyperdrive publish → durable evidence).

This has not been scored (no RRI computed) or presented as a task card —
that is the next session's first action once weekly token budget allows.

## Related

- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § P2.T4b-T4f retrospective
  integrated closure record (remediation precedent)
- `docs/plan/roadmap.md` (§ MVP0-P2P row, § Known planning gaps)
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
