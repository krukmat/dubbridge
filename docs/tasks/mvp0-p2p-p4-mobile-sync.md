---
type: TaskList
title: "Tasks: P4 Verified mobile ciphertext synchronization"
status: planned
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p4-mobile-sync.md
behavioral_coverage_contract: behavior-v2
---

# P4 — planning task ledger

**Status:** Planned; no task activated or implemented by this documentation update.
**Phase gate:** P3 PASS.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P4.T0 | Lifecycle, cache, and RPC freeze | planning | M | P3 PASS | `[x]` Done 2026-09-18 |
| P4.T1 | Product replication and bounded resume | development | L | T0 PASS | Implemented + verified — Med-high review pending |
| P4.T2 | Manifest verification and lifecycle isolation | development | L | T1 PASS | `[x]` Done 2026-09-18 |
| P4.T3 | P4 certification and P5 handoff | development/evidence | M | T2 PASS | Blocked — no certification artifact, see verification note |
| P4.T1-r1 | Product storage file-URI → path at the Corestore boundary (repair; blocks P5.T3) | development | S (provisional) | P4.T1 source present | Planned — RRI/card/approval pending; not implemented |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P4 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p4-mobile-sync.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P4.T0 — Lifecycle, cache, and RPC freeze

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P3 PASS

**Status:** `[x]` Done — retro-certified 2026-09-18 under owner waiver D0-a (peer
review waived; verification not waived). RRI 25 Low (`scripts/rri.py`,
`mobile/src/p2p/sync/{SyncState,SyncCache,P2PSyncController}.ts`).

**Reflection:** Low band, cycle folded into review evidence below — no gap found.

**Behavioral coverage certification:** HP-P4.T0-1 and EC-P4.T0-1 both map to
passing `unit` evidence — 14/14 tests in `mobile/src/p2p/sync/__tests__`
directly exercise the sync/resume/verify state machine, the READY invariant,
per-account cache isolation, and sign-out cancellation.

**Owner final verification:** Matias, 2026-09-18 — closed on "cierra los
otros" after reviewing the named residual-free verdict in
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.
Commands run: `npm test -- mobile/src/p2p/sync`.

**Acceptance criteria:** Freeze persistent ciphertext-cache lifecycle and versioned sync/verification operations, owner paths and evidence mapping; preserve inert startup and proof-runner isolation.

- **HP-P4.T0-1:** A valid descriptor has a defined sync/resume/verify lifecycle and account-owned cache.
- **EC-P4.T0-1:** Sign-out or stale-account callbacks cannot expose another account cache or promote READY.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P3 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P4.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P4.T1 — Product replication and bounded resume

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** Implemented + verified; formal Done is pending the required
Med-high band independent code-solution review. Execution RRI 55 (Med-high) is
recorded in `docs/audit/p4-t1-bounded-reconnect-rri-2026-09-22.md`.
The previously named bounded-reconnect gap is implemented in
`P2pProductSync.ts`: one automatic reconnect by default, transport-only retry,
verified partial-cache reuse, bounded exhaustion, and cancellation/sign-out
domination. GitHub Actions mobile evidence on
`0c3d484648565755348011b85a6a84acca41fb0c` is 60/60 suites and 434/434 tests
PASS, including the focused product-sync suite. Full evidence:
`docs/audit/p4-t1-bounded-reconnect-evidence-2026-09-22.md`.

**Acceptance criteria:** Wire explicit product sync through P2PService → BareRuntimeClient → product worklet, bounded reconnect/cancel and reusable partial ciphertext cache.

- **HP-P4.T1-1:** An interrupted foreground replication resumes without corrupting or restarting the whole package.
- **EC-P4.T1-1:** Unavailable peers or cancellation terminate bounded work without READY, unbounded retry, or activating proof topology.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P4.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P4.T2 — Manifest verification and lifecycle isolation

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** `[x]` Done — retro-certified 2026-09-18 under owner waiver D0-a.
RRI 25 Low (`scripts/rri.py`,
`mobile/src/p2p/sync/{PackageVerifier,VerifiedPackageHandle}.ts`,
`mobile/src/p2p/runtime/replication-verify.ts`).

**Reflection:** Low band, cycle folded into review evidence below.

**Behavioral coverage certification:** HP-P4.T2-1 and EC-P4.T2-1 map to
passing `unit` evidence — 14/14 tests; digest mismatch, missing files, and
identity mismatch each fail closed under a dedicated test.

**Named residual (owner-accepted):** no test isolates cancellation during
`VERIFYING` specifically (only `DOWNLOADING` is tested); "secrets never reach
Bare" is a structural/interface guarantee, not an explicit negative-assertion
test. Non-blocking, same class as `P2.T4e-cov`.

**Owner final verification:** Matias, 2026-09-18 — closed on "cierra los
otros", accepting the named residual as described in
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.
Commands run: `npm test -- mobile/src/p2p/sync`.

**Acceptance criteria:** Verify complete package against expected canonical manifest/hash and file digests before READY; enforce frozen sign-out/cache/device lifecycle and no-secret runtime boundary.

- **HP-P4.T2-1:** Full ciphertext package verifies and reports READY only after every required file passes.
- **EC-P4.T2-1:** Corrupted/missing bytes, manifest mismatch or sign-out during verification cannot become READY; private key, KEK, JWT-signing material and DB credentials never reach Bare.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P4.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P4.T3 — P4 certification and P5 handoff

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** Blocked — not closeable. **Verification note (2026-09-18):** no
discrete certification artifact exists (P5 has an analogous
`P5DeviceCertification.ts`; P4 has none), and "account-change" as distinct
from sign-out is not separately exercised by any test. Full evidence:
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.
Needs the missing artifact/test before closure.

**Acceptance criteria:** Certify sync/resume/corruption/account-change HP/EC on the product runtime; record verified package handle and state contract consumed by P5.

- **HP-P4.T3-1:** Product runtime discovers, syncs, verifies and hands off the expected package after interruption recovery.
- **EC-P4.T3-1:** Replicated bytes without control-plane permission remain unplayable; transport success alone does not establish readiness.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P4.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P4.T1-r1 — Product storage file-URI → path at the Corestore boundary

**Type:** development (defect repair; blocks P5.T3 SYNC)

**Effort:** S (provisional — run `scripts/rri.py` on the exact paths before presentation)

**Depends on:** P4.T1 product worklet source (`de199ff`); does not require P4.T1 closure.

**Status:** Planned — not scored, not presented, not approved, not implemented.
Owner instruction 2026-09-22: no local-AI roles for this line of work for now
(weekly usage 94 %); resume from `docs/prompts/p4-t1-r1-storage-uri-fix.md`.

**Defect (confirmed):** `ProductPackageRuntime.open` passes the host `file:` URI
(`Bare.argv[0]` + `/accounts/<scope>`) straight to `new Corestore(...)`, which treats
strings as filesystem paths → on Android `ENOENT stat "file:"` in `drive.ready()`.
Evidence: `docs/audit/mvp0-p2p-p5-t3-android-certification-blocked-2026-09-22.md`
§ "Corrida diagnóstica instrumentada".

**Selected option (recommended, pending owner approval):** A — keep the host→worklet
`file:` URI contract; convert exactly once with `bare-url` `fileURLToPath` immediately
before `new Corestore` in `openPackage`; map conversion failure to
`PRODUCT_STORAGE_CONFIG_INVALID` before any storage/network handle exists.

**Allowed paths:**
- `mobile/src/p2p/runtime/product-package-runtime.ts`
- `mobile/src/p2p/runtime/worklet.bundle.js` (regenerated via `npm run build:bare-worklet` only)
- `mobile/__tests__/p2p/product-storage-path.test.ts` (new)
- `mobile/package.json`, `mobile/package-lock.json` — only if the owner approves
  declaring `bare-url@^2.5.2` as a direct dependency (must resolve to the installed 2.5.2)
- `docs/audit/mvp0-p2p-p1-a1b-storage-contract.md` (amendment note only)

**Out of scope:** `transient-drive.ts` proof path (same latent defect; residual,
dev-only), host `BareRuntimeClient`/Expo URI construction, RPC/protocol/codec,
the TS2339 residual from `f2fa64c`, P4.T1/P4.T3 closure.

**Acceptance criteria:**
- **HP-P4.T1-r1-1:** A valid root URI whose path contains a space (`%20`) opens a
  real Corestore/Hyperdrive (RocksDB) at `<decoded root>/accounts/<scope>`; no
  relative `file:` directory is created in the process cwd. (RED on current code.)
- **HP-P4.T1-r1-2:** Distinct account scopes resolve to distinct absolute
  directories, identical to the host cleaner's `Directory(root, "accounts", scope)`.
- **EC-P4.T1-r1-1:** Non-empty authority (`file://evil/…`), encoded `/` (`%2F`) or
  NUL (`%00`), or non-`file:` scheme → `PRODUCT_STORAGE_CONFIG_INVALID`; no
  Corestore/Hyperswarm constructed.
- **EC-P4.T1-r1-2:** `%2520` decodes once to a literal `%20` (no double decoding).
- **Device:** a fresh invitation on the Android emulator no longer fails with
  `ENOENT stat "file:"`; record the next observed stage/result as-is (may expose a
  later blocker; not a PASS claim).

**Test notes:** follow the existing `bare-crypto` idiom —
`jest.mock("bare-url", () => require("node:url"))`; mock `hyperswarm` with a fake
whose `join().flushed()` resolves `true` (no network); use `@jest-environment node`
and `process.chdir` into a temp dir so a RED run cannot pollute `mobile/`. A real
Corestore open under `jest-expo` was verified feasible on 2026-09-22.

**Evidence to emit:** RED→GREEN test output; `npm run check:bare-worklet`;
`npm run typecheck` compared to the 3-error TS2339 baseline; `npm run lint`;
Android emulator run with redacted markers; RRI report.

**Status artifacts affected:** this ledger; P5 ledger/plan P5.T3 status; the P5.T3
audit; `docs/audit/mvp0-p2p-p1-a1b-storage-contract.md` amendment.

