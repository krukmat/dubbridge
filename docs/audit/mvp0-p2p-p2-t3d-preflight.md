---
type: Audit
title: "P2.T3d preflight — Availability Node contract/mTLS/security certification"
status: complete
task: P2.T3d
---

# P2.T3d preflight

Prepared per `docs/playbooks/P2_T3D_ORCHESTRATOR_RUNBOOK.md` §§ 3-4, on top
of the closed `P2.T3c` implementation at current HEAD
(`feature/p2p-mvp-core`). This is analysis only — no test files have been
written yet. Owner RRI/approval decision (runbook § 5) still required before
implementation.

## 1. T3c closure references used as input

- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § "P2.T3c — persistent
  Hyperdrive publication and stable replay" — `[x] Done`, owner-verified
  2026-09-13.
- `T3c-Integ` closure record (same file) — unified verification that a
  package built by the real Rust pipeline (`build_package` +
  `package_writer::materialize`) is accepted end-to-end by the real
  Availability Node `createPublicationExecutor`.
- CONS-T1 (`docs/tasks/mvp0-p2p-s230-consistency-remediation.md`) — fixed a
  2026-09-15..18 regression where the JS test fixture's `makeRequest()` used
  a stale `package_ref` (production dispatcher was never affected). 86/86
  Availability Node tests passing at HEAD as of this preflight.

No pre-existing uncommitted changes touch `apps/availability-node/` except
`apps/availability-node/test/package-publication-integration.test.js`
(CONS-T1's fix, already in the working tree, unrelated to T3d's scope).

## 2. Real API surface T3d certifies

- **Publisher entrypoint:** `createPublicationExecutor(config)` from
  `apps/availability-node/src/publication_executor.ts` — the real executor,
  not a stub. Takes `{ packageRoot, driveStorageRoot, indexRoot,
  hyperswarmJoinTimeoutMs? }`.
- **HTTP/mTLS transport:** `createPrivatePublicationServer(credentials,
  allowedClientFingerprints, publisher)` from `src/server.ts`, composing
  `src/mtls.ts`'s `createPrivateMtlsServer` (TLS 1.3, `requestCert: true`,
  `rejectUnauthorized: true`) with `createClientFingerprintPolicy`
  (SHA-256 fingerprint allowlist).
- **Contract layer:** `src/contract.ts` — `PATH_PREFIX = "/v1/publications/"`,
  `METHOD_PUT`, `parsePublicationRequest`/`parsePublicationResponse`,
  `createPublicationError`, `PublicationContractError`.
- **Error code -> HTTP status map** (`contract.ts:10`, authoritative,
  cross-checked against `docs/fixtures/mvp0-p2p-publication-contract-v1.json`):

  | Code | Status | Meaning |
  |---|---|---|
  | `invalid_contract` | 400 | malformed contract/request shape |
  | `service_identity_rejected` | 403 | mTLS identity authenticated but not allow-listed |
  | `publication_conflict` | 409 | same publication_id, different lineage/digest |
  | `package_invalid` | 422 | package_ref/manifest/path/digest/ciphertext validation failure (covers traversal, symlink escape, corrupt manifest, size/hash mismatch, tampered ciphertext) |
  | `publication_unavailable` | 503 | transient Hyperdrive/storage/Hyperswarm-join failure |

## 3. Fixture ciphertext construction

Use the **existing** `crates/p2p/src/bin/package_build_and_materialize_fixture.rs`
test-only binary (already used by `T3c-Integ`'s
`package-publication-integration.test.js`), invoked via `cargo run -p
dubbridge-p2p --bin package_build_and_materialize_fixture`. It runs the real
`build_package` + `materialize` Rust pipeline and returns JSON
(`ok`, `publication_id`, `lineage_id`, `manifest_digest_sha256`,
`package_dir`). This is real ciphertext on disk, not a hand-written JS
fixture reproducing the shape — required by the runbook § 4 ("Un stub no
demuestra publicación ni lectura real de Hyperdrive").

## 4. Hyperdrive access by public identifier

`openDrive`/`closeDrive`/`flushDrive`/`announceOnSwarm` from
`src/hyperdrive_store.ts`, already used identically by
`package-publication-integration.test.js`. `opened.drive.get("/manifest.json")`
etc. reads the persisted ciphertext bytes; `opened.publicKeyHex` is the
stable public identifier returned as `external_publication_id` in the
success evidence.

## 5. Local deterministic transport (mTLS harness)

`private-publication-ingress.test.js` already contains a complete,
reusable-by-example local CA/server/client certificate generation harness
(`openssl` via `execFileSync`, ephemeral `mkdtemp` dir, cleaned up in
`finally`) plus an HTTPS client helper (`makeRequest`) that connects to
`127.0.0.1` with `servername: 'localhost'`. No public internet reachability
required. T3d's `publication-contract.test.js` reuses this exact pattern
rather than inventing a new one.

## 6. Publisher reconstruction preserving its root

`package-publication-integration.test.js`'s `HP-T3c-2` test already
demonstrates the pattern: build a fresh `createPublicationExecutor(config)`
over the same persistent `driveStorageRoot`/`indexRoot` (not a fresh
executor sharing in-memory state) to simulate a process restart. T3d reuses
this for HP-T3d-1's replay leg, but through the HTTP/mTLS server instead of
calling the executor function directly.

## 7. Fault injection seams (no production code change)

- **503 / `publication_unavailable`:** `PublicationExecutorConfig.
  hyperswarmJoinTimeoutMs` is a real constructor parameter — passing a very
  small value (e.g. `1`) against a Hyperswarm join that cannot complete that
  fast deterministically triggers the `catch` block at
  `publication_executor.ts:94` / `:143`, which throws
  `publication_unavailable` before any index record is committed. This is an
  existing seam, not a new one.
  - Storage-failure variant: point `driveStorageRoot` or `packageRoot` at a
    path with revoked write permission (`fs.chmodSync(dir, 0o000)`,
    restored in `finally`) to trigger a real fs-level failure through
    `openDrive`/writes, surfacing as `publication_unavailable` via the same
    catch path in `server.ts:167`/`:178` (`PublicationContractError` ->
    `publication_unavailable` fallback for non-`PublicationContractError`
    exceptions is NOT automatic — only `PublicationContractError` instances
    preserve their own `.code`; a raw fs `EACCES` thrown by `openDrive`
    would NOT be a `PublicationContractError` and must be confirmed against
    actual executor behavior before relying on it as the 503 path. Flag for
    phase-1 packet: verify which of the two injection methods (Hyperswarm
    timeout vs. fs permission) actually surfaces as 503 through the full
    HTTP stack before freezing the EC-T3d-3 test on it.**
- **422 / `package_invalid` (traversal/symlink/corrupt/size/hash/secret):**
  same tamper pattern as `package-publication-integration.test.js`'s
  `EC-T3c-2` (overwrite a materialized ciphertext file post-build) plus
  `containment.test.js`'s existing symlink-escape fixture pattern, run
  through the real executor via HTTP instead of direct function call.
- **409 / `publication_conflict`:** same two-build-same-publication_id
  pattern as `EC-T3c-1a`/`EC-T3c-1b`, through HTTP.
- **403 / `service_identity_rejected`** and **TLS-handshake rejection:**
  already fully proven by `private-publication-ingress.test.js` at the
  transport layer with a stub publisher; T3d's job is to prove the same
  transport behavior composes correctly with the **real** executor (not
  re-invent the mTLS harness).

## 8. Logs, response, persisted metadata, and drive secret scan

Deny-list is frozen in `docs/fixtures/mvp0-p2p-publication-contract-v1.json`
§ `secret_deny_list` (8 fields: `postgres_credentials`, `plaintext_ck`,
`server_kek`, `invite_state`, `viewer_state`, `business_authorization`,
`application_jwt_signing_material`, `service_private_key_payload`).
`secret_boundary.test.ts` already proves this at the `contract.ts` parse/
serialize layer with synthetic canary values. T3d's job is the
**end-to-end** negative scan: after a real HTTP publish, inspect (a) the
HTTP response body, (b) anything written to `console.*`/stderr during the
test run, (c) the persisted index record file
(`publication_index_io.ts`'s on-disk JSON), and (d) the raw Hyperdrive
content bytes — confirming none of the 8 canary strings appear anywhere,
using synthetic canary values embedded in a deliberately-malformed request
(never real secret material).

## 9. Criterion -> concrete test mapping

| Criterion | Test | Real components exercised | Assertion evidence |
|---|---|---|---|
| HP-T3d-1 | `publication-contract.test.js` | mTLS harness (§5) + real executor (§2) + real Rust package (§3) | `201` on first publish; close server, reconstruct a second `createPrivatePublicationServer` over the same `driveStorageRoot`/`indexRoot`, replay same request -> `200`, identical `evidence` (publication_id, lineage_id, manifest_digest_sha256, external_publication_id, evidence_id, confirmed_at) |
| HP-T3d-2 | same file | `openDrive`+`drive.get()` (§4) | after HP-T3d-1's publish, open the drive by `external_publication_id`/`publicKeyHex`, assert manifest + declared files present and byte-identical to what `verifyPackage` accepted; assert response body carries no readiness/authorization fields (schema check against `parsePublicationResponse`'s declared shape) |
| EC-T3d-1 | same file, reused pattern from `private-publication-ingress.test.js` | mTLS harness (§5), real executor as publisher instead of stub | no-cert / rogue-cert -> TLS handshake failure, zero executor invocations (instrument a call counter wrapping the real executor); unlisted allow-listed-CA client -> `403` `service_identity_rejected`, zero executor invocations |
| EC-T3d-2 | same file | real executor + real Rust builds (§3, §7) | (a) same publication_id, different lineage (two real builds) -> `409`, original evidence unchanged, no second drive write (`core.length` unchanged, mirroring `EC-T3c-1a`); (b) same publication_id+lineage, different digest -> `409`, same evidence-unchanged check (mirrors `EC-T3c-1b`); (c) tampered ciphertext post-build -> `422`, no drive created (mirrors `EC-T3c-2`); (d) symlink-escape package_ref (containment violation) -> `422`, no drive created; (e) malformed JSON body / wrong content-type / oversized body -> `400` `invalid_contract`, per `server.ts`'s own request-shape gate (NOT `422` — confirms the runbook's own warning that not every malformed-input case is `package_invalid`) |
| EC-T3d-3 | same file | injected Hyperswarm-timeout or fs-permission fault (§7, pending the flagged verification) | `503` `publication_unavailable`; negative secret scan (§8) across response, logs, persisted index record, and drive bytes |
| Integration (criterion 6) | same file, run alongside existing suite | full `apps/availability-node/test/*.test.js` + the two `docs/audit/mvp0-p2p-p2-t3a-*.test.js` files | all pass together; every `mkdtemp` root removed in `finally`; no test requires public network reachability |

## 10. fs-permission 503 variant — resolved by code inspection

`openDrive` (`hyperdrive_store.ts:68-88`) re-throws the raw, untyped error
from `store.ready()`/`drive.ready()` (`throw err`, not a
`PublicationContractError`) on failure. `publication_executor.ts`'s
first-publish branch (line ~112) calls `openDrive` with no local
`try/catch`, so that raw error propagates up through `execute()` and
`withPublicationLock` to `server.ts`'s `catch (e)` at line 163: any `e` that
is **not** `instanceof PublicationContractError` falls through to the
`else` branch and is sent as `publication_unavailable` (503) — confirmed by
direct code read, not assumed. Both the Hyperswarm-join-timeout variant
(typed, explicit) and the fs-permission variant (untyped, falls through the
same generic-error branch) reach 503 through the real stack. Either is
sufficient for EC-T3d-3; use the Hyperswarm-timeout variant as the primary
frozen case (deterministic, no filesystem permission manipulation needed)
and treat the fs-permission variant as optional extra coverage, not
required to freeze the packet.

## 11. RRI (runbook § 5)

```
python3 scripts/rri.py \
  --touches apps/availability-node/test/publication-contract.test.js \
  --touches apps/availability-node/test/fixtures.js \
  --cc 4 --D 3 --K 3 --P 2 --T 0 --A 1 --X 2
```

No `_DUBBRIDGE_RUBRIC` row matches `apps/availability-node/*` — agent
judgment governs D/K/P per the script's own advisory. Justification: D=3
(security-domain certification: mTLS client-identity enforcement, secret
non-leakage boundary — comparable to `crates/auth/*`'s and
`crates/connectors/*`'s D=3/4 rubric rows for the same class of concern, not
inflated by analogy to the CONS-T2 rubric-gap finding, which was a missing
test-path carve-out for a *different* kind of change — production-floor
inheritance on trivial test edits — not applicable here since this task's
core purpose *is* certifying the security boundary itself); K=3 (couples
mTLS transport, the real executor, real Rust-built ciphertext, and
Hyperdrive read-back — four previously-separate T3a/T3b/T3c-Integ-proven
components composed together for the first time); P=2 (validates real
fail-closed invariants — 403/409/422/503 — but does not itself change
production code or schema); T=0 (adds test coverage, does not reduce any);
A=1 (scope and paths frozen by the ledger and this preflight, low
ambiguity); X=2 (test-only files, `apps/availability-node/test/*`, no
production/deploy exposure, bounded blast radius).

**Result:** `Final RRI: 70 = max(ici_band 70, risk_band 20) -> band Complex
(56-70)`. Driven by the ICI bottleneck (`Q=level4(D)=3` -> `B=3` -> `ICI=75`
-> banded to 70), not by the risk/domain band (20). Full report:
`.agent/p2-t3d/parent-rri.md`.

**Honest Low-band maximization pass (mandatory per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Honest Low-band maximization
before presentation`):** tested two candidate splits.

- `publication-contract.test.js` alone (dropping `fixtures.js`, cc=3,
  D=3/K=2/P=1/X=1): still RRI 70 Complex — D=3 alone saturates the
  bottleneck regardless of K/P/X.
- `fixtures.js` alone (pure data/helper file, no security assertions,
  D=2/K=1/P=0/X=0): RRI 55 Med-high — lower, but still not Low, and the two
  files are not independently meaningful: `fixtures.js` exists only to
  supply certificates/config to `publication-contract.test.js`'s security
  assertions, so splitting them would fragment one certification invariant
  into two unverifiable-alone fragments — exactly what the workflow guide's
  honest-low-max procedure prohibits (step 5: "do not... split one
  invariant across unverifiable fragments").

**Conclusion:** `honest-low-max: residual` — no genuinely separable Low
leaf exists. The parent RRI 70 Complex is the real, non-inflated score and
governs routing: mandatory decomposition before implementation is not
applicable here in the sense of splitting into Low subtasks (none exist),
but the band still requires explicit HITL approval before implementation
and RRI 56+ decomposition discipline — in this case, that means presenting
the task as a single Complex-band approval-gated unit rather than inventing
an artificial split. Per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
Reflection design pattern`, RRI 56-70 requires 4 Reflection passes at
implementation time, and per § Band-routed peer review, RRI 56+ review is
`gpt-oss:20b` (Complex profile) primary / cross-vendor peer (codex)
fallback / D14 final fallback — not the Low-band chain used for CONS-T2's
two fixes.

## 12. Phase-1 (task-analysis) review

`scripts/peer-workflow-review.py`'s dry-run resolved `reviewer=codex` for
this RRI 70 packet — found to be a script defect (documented separately,
`docs/audit/peer-workflow-review-complex-band-defect-2026-09-18.md`): the
script's own docstring states the correct RRI 56+ contract
(`gpt-oss:20b` Complex profile primary, cross-vendor fallback), but
`main()`'s `cross_vendor` branch calls the cross-vendor peer directly,
skipping the gpt-oss-primary attempt entirely. Worked around by invoking
`gpt-oss:20b` directly at the Complex profile
(`num_ctx=49152`, `num_predict=10240`, `think=medium`, `temperature=1.0`,
`top_p=1.0`) per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory
workflow before implementing` Step 0. Warm-up probe: `done_reason: stop`,
non-empty content. Review invocation: `done_reason: stop`, valid JSON.

**Verdict: PASS.** One MINOR finding: recommended verifying the
same-`publication_id`-different-lineage/digest 409-conflict reproduction
path before freezing EC-T3d-2's tests. **Disposition: accepted, already
mitigated** — this is exactly the pattern already exercised and passing in
`package-publication-integration.test.js`'s `EC-T3c-1a`/`EC-T3c-1b`
(re-run to confirm: `node --test test/package-publication-integration.test.js`
from `apps/availability-node/`, 5/5 passing, including both conflict
tests). No revision to the preflight needed; T3d's own `EC-T3d-2` tests
will exercise the same already-proven pipeline path through the HTTP layer
instead of the direct executor call.

Artifact: `.agent/p2-t3d/phase1-review-v1.json`.

```
Task-analysis review: gpt-oss .agent/p2-t3d/phase1-review-v1.json - PASS
```

## Next step

Present the six-block Compact Approval Task Card v2 and stop for explicit
owner approval — RRI 56+ requires HITL approval before any implementation
starts, even though this preflight (analysis only) and phase-1 review are
both complete.
