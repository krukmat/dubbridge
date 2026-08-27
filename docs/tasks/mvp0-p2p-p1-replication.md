---
type: TaskList
title: "Tasks: MVP0-P2P P1 maintainable mobile foundation and replication proof"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p1-replication.md
---

# Tasks: MVP0-P2P P1 — Maintainable mobile P2P foundation and replication proof

> **Parent plan:** `docs/plan/mvp0-p2p-p1-replication.md`.
> **External input:** `p2p-mvp/taskpacks/P1.zip`.
> **Dependency gate:** P0 closed PASS — satisfied on 2026-08-27.
> **Status:** The 2026-08-27 maintainability review materially replanned P1.
> Earlier P1/P1.A approvals are superseded before source execution. The owner
> approved revised P1 and accepted ADR-043 on 2026-08-27. That decision
> authorizes child preparation/presentation only; P1.F1 is next and no child
> source execution is authorized yet.

## Task map

| ID | Title | Status | Depends on |
|---|---|---|---|
| P1 | Maintainable mobile P2P foundation + replication proof (planning parent) | APPROVED 2026-08-27 — RRI 94; decomposed; no direct source execution | P0 PASS |
| P1.F1 | Reproducible worklet bundle + versioned RPC contract | Awaiting approval — RRI 54 Med-high; no source execution | Revised P1 + ADR-043 approval — satisfied 2026-08-27 |
| P1.F2 | Mobile service ownership + composition | Deferred — needs current RRI/card/approval | P1.F1 PASS |
| P1.F3a | P0 runtime-scaffold migration + retirement | Deferred — needs current RRI/card/approval | P1.F2 PASS |
| P1.F3b | P0 config/dependency cleanup | Deferred — needs current RRI/card/approval | P1.F3a PASS |
| P1.A1 | Hyperdrive/Corestore Android bundle smoke proof | Deferred — needs current RRI/card/approval | P1.F3b PASS |
| P1.A2 | Transient seed lifecycle + residue cleanup | Deferred — needs current RRI/card/approval | P1.A1 PASS |
| P1.B1 | Isolated Hyperswarm replication transport | Deferred — needs current RRI/card/approval | P1.A2 PASS |
| P1.B2 | Verification, reconnect + fail-closed witness | Deferred — needs current RRI/card/approval | P1.B1 PASS |

The former combined `P1.A — Ephemeral seed fixture and bundle boundary` planning
parent and its A1/A2 interpretation are superseded. Historical artifacts remain
at `docs/audit/mvp0-p2p-p1a-approval-card.md` and
`docs/audit/mvp0-p2p-p1a-rri.md`; they authorize nothing.

## P1 — Maintainable mobile P2P foundation and replication proof

- **Status:** Revised planning parent approved 2026-08-27 with ADR-043 accepted.
  The previous approval remains superseded by the material architecture/scope
  change; no implementation started and no child source work is authorized.
- **Type:** Very-high development/architecture parent; decomposed before
  implementation.
- **Complexity / Effort / RRI:** Very high / XL / 94. Full report:
  `docs/audit/mvp0-p2p-p1-rri.md`.
- **External taskpack declaration:** `gpt-5.6-terra` / high. This is retained as
  external input, but no direct implementation route is authorized because the
  repository RRI is Very high and requires ADR/risk analysis plus decomposition.
- **Allowed future paths:** the exact paths frozen by each child under
  `mobile/App.tsx`, `mobile/src/navigation/RootNavigator.tsx`,
  `mobile/src/p2p/**`, `mobile/scripts/**`, `mobile/__tests__/p2p/**`,
  `mobile/package*.json`, and P1 evidence/status documents. No child inherits
  this whole union automatically.
- **Objective:** establish the ADR-043 mobile/runtime ownership boundary, then
  seed and replicate a synthetic opaque fixture between isolated proof runtimes
  using Hyperdrive/Hyperswarm and accept only verified hash plus cleanup.
- **Out of scope:** backend/API/database work; real DubBridge assets, HLS,
  `StorageAdapter`, product UI, durable product cache/identity, encryption/key/
  envelope design, invitations, availability node, local HTTP, HTTP fallback,
  custom native background service, and iOS.

### Happy paths considered

- **HP-1:** Mobile composes auth and P2P ownership above a navigation-only
  `RootNavigator`; the inert product service starts one bundled/versioned Bare
  runtime only on explicit command and preserves P0 ping behavior.
- **HP-2:** An isolated proof seed writes a synthetic fixture to run-scoped
  transient storage; a separate client discovers/replicates it and reports PASS
  only after SHA-256 equality and verified storage removal.
- **HP-3:** After one bounded disconnect/rejoin, the proof re-establishes the
  client operation and still requires full digest and teardown success.

### Edge cases considered

- **EC-1:** Protocol-version mismatch, malformed reply, timeout, worklet fatal
  error, or invalid runtime transition fails typed/redacted and releases pending
  calls/listeners/handles.
- **EC-2:** Discovery, connection, replication, digest, reconnect, suspend/resume,
  or cleanup failure is terminal; partial transfer or residual run storage can
  never be presented as verified.

### Acceptance criteria

- ADR-043 was accepted with the revised parent on 2026-08-27; every child still
  requires current RRI/card/approval before source edits.
- App composition, stable service/provider ownership, one product runtime,
  reproducible bundling, versioned `bare-rpc`, fatal handling, and suspend/resume
  semantics match ADR-043 without moving runtime ownership into navigation.
- The Android development build proves Hyperdrive → Hyperswarm → Hyperdrive
  replication of a synthetic fixture, SHA-256 equality, bounded reconnect, and
  verified deletion of both proof storage directories.
- P0 behavior and ordinary mobile behavior remain intact; no UI, default network
  activation, or product P2P capability is introduced.
- Every child HP/EC has passing unit-test evidence and a physical Android proof
  before P1 can close.

### Evidence to emit

- Revised P1/child RRI reports, approval cards, ADR-043, and required route
  artifacts.
- Dependency/bundle-drift, protocol/lifecycle, composition ownership,
  transient-storage cleanup, and Android native proof evidence.
- Unit-test, typecheck, lint, and full mobile Jest outputs; P1 handoff at close.

### Status artifacts affected

- This ledger, its plan, `docs/tasks/mvp0-p2p-first.md`,
  `docs/plan/mvp0-p2p-first.md`, `docs/plan/roadmap.md`, `docs/architecture.md`,
  `docs/adr/README.md`, ADR-043,
  `p2p-mvp/RUN_STATE.json`, and `p2p-mvp/handoffs/P1.md`.

### Task-analysis review

Task-analysis review: REVIEW-OVERRIDE — explicit owner-directed MVP0-P2P
exception; `docs/audit/mvp0-p2p-review-exception.md`.

### Code-solution review

Code-solution review: REVIEW-OVERRIDE — to be recorded at P1 closure under the
explicit owner-directed MVP0-P2P exception;
`docs/audit/mvp0-p2p-review-exception.md`.

### Required execution sequence

1. Revised parent approval accepted ADR-043 and this decomposition on
   2026-08-27; it authorizes child preparation only.
2. Score, present, approve, implement, and close P1.F1 → P1.F2 → P1.F3a →
   P1.F3b → P1.A1 → P1.A2 → P1.B1 → P1.B2 in order. Do not edit source for a
   child before its own approval and do not start the next child before
   PASS/status sync.
3. Close P1 only after all eight children, five parent Reflection passes, coverage
   certification, owner final verification, and status synchronization pass.

## P1.F1 — Reproducible worklet bundle and versioned RPC contract

- **Status:** Awaiting explicit current-session approval; no source execution is
  authorized. Parent/ADR gate satisfied 2026-08-27.
- **Effort / RRI:** L / 54 Med-high. Full report:
  `docs/audit/mvp0-p2p-p1-f1-rri.md`.
- **Allowed paths:** `mobile/package.json`, `mobile/package-lock.json`,
  `mobile/scripts/build-bare-worklet.mjs`, `mobile/src/p2p/runtime/protocol.ts`,
  `mobile/src/p2p/runtime/worklet.ts`,
  `mobile/src/p2p/runtime/worklet.bundle.js`,
  `mobile/__tests__/p2p/runtime-protocol.test.ts`, and F1 evidence only.
- **Objective:** replace inline/custom proof transport with a reproducibly built
  Bare backend and typed/versioned DubBridge protocol over `bare-rpc`.
- **HP-F1:** deterministic bundle build plus compatible handshake returns
  protocol/runtime capabilities and preserves a bounded ping.
- **EC-F1:** bundle drift, unsupported protocol, malformed payload, uncaught
  exception/rejection, or invalid lifecycle message fails typed and redacted.
- **Acceptance:** source-to-bundle drift check passes; handshake/validators,
  global fatal handlers, suspend/resume messages, timeouts, and clean shutdown
  have unit evidence; no Hyperdrive/network/app composition is added.
- **Evidence to emit:** current RRI/card/route receipt, dependency and bundle
  digest, HP-F1/EC-F1 tests, typecheck/lint/Jest, Reflection/coverage/owner proof.
- **Status artifacts affected:** this ledger, P1 plan/card, ADR-043 implementation
  references, and child audit artifacts.
- **Handoff prompt:** `P1.F1 — implement only the approved worklet packaging and
  versioned RPC contract; preserve P0 ping and stop before mobile composition or
  Hyperdrive.`

## P1.F2 — Mobile service ownership and composition

- **Status:** Deferred until P1.F1 PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** L / 55 Med-high. Recompute at presentation.
- **Allowed paths:** `mobile/App.tsx`, `mobile/src/navigation/RootNavigator.tsx`,
  the existing P0 probe/bridge, `mobile/src/p2p/runtime/BareRuntimeClient.ts`,
  `mobile/src/p2p/P2PService.ts`, `mobile/src/p2p/P2PProvider.tsx`, focused P2P
  tests `bare-bridge.test.ts`, `p2p-service.test.ts`, and
  `p2p-provider.test.tsx`, and F2 evidence only.
- **Objective:** make the app composition root own auth/P2P providers and give
  product code one inert service/runtime boundary outside navigation while the
  P0 oracle remains available for migration comparison.
- **HP-F2:** `App.tsx` composes `AuthProvider → P2PProvider → RootNavigator`;
  explicit P0 diagnostics initialize/ping/shutdown through one stable service.
- **EC-F2:** mounting/re-rendering/navigation/auth route changes do not
  auto-start, duplicate, or leak a runtime; invalid start/stop remains typed.
- **Acceptance:** `RootNavigator` creates no auth/P2P provider; `P2PService` is
  framework-independent; provider service identity is stable and status uses an
  external-store subscription; the P0 oracle exercises the new boundary; no
  network and no P0 source deletion occurs in F2.
- **Evidence to emit:** current RRI/card/route receipt, composition/service tests,
  P0 regression proof, typecheck/lint/Jest, Reflection/coverage/owner proof.
- **Status artifacts affected:** this ledger, P1 plan/card, ADR-043 implementation
  references, and child audit artifacts.
- **Handoff prompt:** `P1.F2 — implement ADR-043 composition and service
  ownership, prove parity through the existing P0 oracle, and stop before P0
  retirement, Hyperdrive, or network behavior.`

## P1.F3a — P0 runtime-scaffold migration and retirement

- **Status:** Deferred until P1.F2 PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** L / 43 Med-high. Recompute at presentation.
- **Allowed paths:** `mobile/App.tsx`, the four existing P0 files under
  `mobile/src/p2p/`, `mobile/src/p2p/development/P2PDevelopmentHarness.tsx`,
  `mobile/__tests__/p2p/bare-bridge.test.ts`,
  `mobile/__tests__/p2p/p0-migration.test.ts`, and F3a evidence.
- **Objective:** transfer the P0 ping/error characterization to the ADR-043
  runtime and delete the obsolete probe, custom RPC bridge/protocol, inline
  worklet, and superseded test implementation.
- **HP-F3a:** the new development harness proves the same bounded
  `initialize → ping → shutdown` behavior through `BareRuntimeClient` after all
  old runtime source is gone.
- **EC-F3a:** a parity mismatch, missing migrated failure case, stale import, or
  duplicate runtime owner blocks deletion/closure and leaves P0 files intact.
- **Acceptance:** tests move before deletion; no tracked import references the
  retired files; exactly one diagnostic/runtime path remains; typecheck, lint,
  Jest, and Android ping pass after deletion. P0 audit documents are untouched.
- **Evidence to emit:** current RRI/card/route receipt, before/after reference
  inventory, migrated HP/EC map, deletion manifest, checks and Android proof.
- **Status artifacts affected:** this ledger, P1 plan/card, ADR-043 implementation
  references, and child audit artifacts.
- **Handoff prompt:** `P1.F3a — transfer every P0 characterization case to the
  ADR-043 runtime, then retire only the obsolete tracked runtime scaffold; keep
  audit history and stop if parity is incomplete.`

## P1.F3b — P0 config/dependency cleanup

- **Status:** Deferred until P1.F3a PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** M / 30 Moderate. Recompute at presentation.
- **Allowed paths:** `mobile/package.json`, `mobile/package-lock.json`,
  `mobile/app.config.ts`, and F3b evidence only.
- **Objective:** remove obsolete P0 flag/script entries and prove every retained
  P0-added dependency/build setting has a live ADR-043 consumer.
- **HP-F3b:** the replacement diagnostic command/config starts the new harness;
  Bare Kit and every retained native setting have a documented consumer/proof.
- **EC-F3b:** an unused direct dependency or unjustified build flag fails the
  audit and is removed; a native requirement is never removed on static guesswork.
- **Acceptance:** remove `bareRuntimeProbe` and `android:bare-probe` or replace
  them with generically named P2P diagnostics; retain `react-native-bare-kit` and
  `minSdkVersion: 31`; retain `b4a`, `react-native-b4a`, `@types/b4a`, and
  `useLegacyPackaging` only with a direct consumer or passing Android A/B proof,
  otherwise remove them and update the lockfile. Full Android build/ping passes.
- **Evidence to emit:** current RRI/card, dependency/import matrix, `npm explain`
  evidence, native A/B result for provisional settings, lockfile diff, checks.
- **Status artifacts affected:** this ledger, P1 plan/card, P0 native-preflight
  follow-up note, and child audit artifacts.
- **Handoff prompt:** `P1.F3b — remove obsolete P0 configuration and every
  unjustified direct dependency/build flag; retain only evidence-backed native
  foundations and do not touch audit history.`

## P1.A1 — Hyperdrive/Corestore Android bundle smoke proof

- **Status:** Deferred until P1.F3b PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** M / 40 Moderate. Recompute at presentation.
- **Allowed paths:** `mobile/package.json`, `mobile/package-lock.json`, the
  packaged runtime worklet, `mobile/__tests__/p2p/hyperdrive-smoke.test.ts`, and
  A1 evidence.
- **Objective:** prove compatible Corestore/Hyperdrive dependencies can bundle,
  open an empty transient drive on Android, close it, and perform no discovery.
- **HP-A1:** a validated temporary path opens Corestore/Hyperdrive and returns a
  redacted capability receipt before clean close.
- **EC-A1:** dependency load, bundle, invalid path, open, or close failure is
  typed and cannot report drive readiness.
- **Acceptance:** exact versions and bundle result are recorded; the drive opens
  only below the proof cache root, closes deterministically, and creates no
  Hyperswarm/discovery/product persistence.
- **Evidence to emit:** current RRI/card, dependency/bundle proof, HP-A1/EC-A1
  tests, Android smoke log, checks, Reflection/coverage/owner proof.
- **Status artifacts affected:** this ledger, P1 plan/card, and child audit
  artifacts.
- **Handoff prompt:** `P1.A1 — add only the compatible Hyperdrive/Corestore
  Android smoke boundary; no fixture write, Hyperswarm, or product cache.`

## P1.A2 — Transient seed lifecycle and residue cleanup

- **Status:** Deferred until P1.A1 PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** L / 47 Med-high. Recompute at presentation.
- **Allowed paths:** the packaged runtime worklet,
  `mobile/src/p2p/proof/transient-storage.ts`,
  `mobile/src/p2p/proof/P1SeedProofRunner.ts`,
  `mobile/__tests__/p2p/transient-seed.test.ts`, and A2 evidence.
- **Objective:** write/hash a deterministic synthetic fixture in run-scoped
  cache storage and prove close-before-delete, absence, and crash-residue cleanup.
- **HP-A2:** seed receipt returns byte count/SHA-256; shutdown closes handles,
  removes the exact run directory, and verifies absence.
- **EC-A2:** traversal/foreign path, write/hash/close/delete failure, or abandoned
  run is rejected or janitored without touching paths outside the proof root.
- **Acceptance:** fixture content and keys are never logged; cache path ownership
  is validated; cleanup failure makes the proof fail; janitor is bounded by root,
  age/run markers, and tests; no discovery occurs.
- **Evidence to emit:** current RRI/card/route receipt, path/cleanup tests,
  redacted Android seed receipt, checks, Reflection/coverage/owner proof.
- **Status artifacts affected:** this ledger, P1 plan/card, and child audit
  artifacts.
- **Handoff prompt:** `P1.A2 — implement only transient synthetic seed storage,
  verified cleanup, and bounded janitor behavior; stop before Hyperswarm.`

## P1.B1 — Isolated Hyperswarm replication transport

- **Status:** Deferred until P1.A2 PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** L / 55 Med-high. Recompute at presentation.
- **Allowed paths:** P2P dependency files, packaged runtime protocol/worklet,
  `mobile/src/p2p/proof/P1ReplicationProofRunner.ts`,
  `mobile/__tests__/p2p/hyperswarm-replication.test.ts`, and B1 evidence.
- **Objective:** create two proof-only runtime sessions through a factory and
  replicate the complete fixture over transient Hyperswarm discovery.
- **HP-B1:** seed/client sessions discover, connect, replicate every byte, and
  report transport completion without using the product `P2PService` API.
- **EC-B1:** discovery/connect/replication timeout or one-session failure cancels
  both operations, closes swarm/store/runtime resources, and reports no success.
- **Acceptance:** discovery keys and fixture content are not logged/persisted;
  two sessions exist only inside the proof runner; normal provider mounting is
  inert; transport completion is not yet final P1 verification.
- **Evidence to emit:** current RRI/card/route receipt, dependency proof,
  HP-B1/EC-B1 tests, redacted Android transport log, checks and closure evidence.
- **Status artifacts affected:** this ledger, P1 plan/card, and child audit
  artifacts.
- **Handoff prompt:** `P1.B1 — implement only proof-runner seed/client discovery
  and complete replication; keep proof commands out of P2PService and stop before
  reconnect certification.`

## P1.B2 — Verification, reconnect, and fail-closed witness

- **Status:** Deferred until P1.B1 PASS; requires current RRI/card/approval.
- **Effort / prospective RRI:** L / 55 Med-high. Recompute at presentation.
- **Allowed paths:** runtime protocol/client,
  `mobile/src/p2p/proof/P1ReplicationProofRunner.ts`,
  `mobile/__tests__/p2p/replication-witness.test.ts`, Android evidence, and
  B2/P1 status artifacts.
- **Objective:** turn transport completion into a trustworthy P1 verdict with
  hash verification, one bounded reconnect, lifecycle/operation evidence, and
  complete teardown.
- **HP-B2:** after a bounded disconnect/rejoin, the client reads the complete
  fixture, matches SHA-256, closes both sessions, deletes both run directories,
  and only then emits VERIFIED.
- **EC-B2:** digest mismatch, reconnect-budget exhaustion, suspend/resume failure,
  malformed/fatal worklet result, or residual storage emits typed failure and
  can never transition to VERIFIED.
- **Acceptance:** runtime and operation state machines are distinct; evidence is
  redacted and ordered; Android proof plus full mobile checks pass; P1 closes
  only after coverage and owner verification.
- **Evidence to emit:** current RRI/card/route receipt, HP-B2/EC-B2 tests,
  Android lifecycle/digest/cleanup witness, checks, P1 handoff and status sync.
- **Status artifacts affected:** this ledger, P1/general plans and task ledger,
  roadmap, ADR-043 implementation references, `p2p-mvp/RUN_STATE.json`, and
  `p2p-mvp/handoffs/P1.md` at P1 closure.
- **Handoff prompt:** `P1.B2 — implement bounded reconnect, hash and cleanup
  certification only; fail closed on every incomplete state and do not start P2.`

## Parent reflection plan

| Pass | Focus | Required result |
|---|---|---|
| 1 | Ownership direction | App composition owns providers; navigation owns no runtime and product service exposes no proof command. |
| 2 | Runtime/protocol containment | One product worklet, reproducible bundle, version handshake, fatal and suspend/resume paths are explicit. |
| 3 | Storage safety | Every proof path is scoped, handles close before deletion, abandoned runs are bounded, and residue prevents PASS. |
| 4 | Replication correctness | No verified state before complete read, digest equality, reconnect outcome, and teardown. |
| 5 | Product-boundary protection | No asset, key, invite, UI, backend, local HTTP, default network, native service, or iOS scope leaks in. |

## Unit coverage certification

**Pending child implementation.** P1 cannot close until every approved child
HP/EC and parent HP-1/HP-2/HP-3/EC-1/EC-2 maps to passing unit tests.

## Owner final verification

**Pending completion of P1.F1, P1.F2, P1.F3a, P1.F3b, P1.A1, P1.A2, P1.B1,
and P1.B2.**

## Handoff prompt

`P1 — follow accepted ADR-043 and this ledger; work only on the currently
approved child, synchronize its evidence, and stop before the next child or P2.`
