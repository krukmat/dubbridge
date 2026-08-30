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
> authorizes child preparation/presentation only. P1.F1 was separately approved,
> implemented, and closed PASS after owner verification on 2026-08-27. P1.F2
> was separately approved, implemented, and closed PASS after owner verification
> on 2026-08-27. The owner approved the required decomposition of the former
> executable P1.F3a on 2026-08-27: P1.F3a.1 (parity migration) was approved,
> implemented, and closed PASS after owner verification on 2026-08-27.
> P1.F3a.2 (retirement) was implemented via owner-directed Low-band
> decomposition and closed PASS/Done on 2026-08-27; see
> `docs/audit/mvp0-p2p-p1-f3a2-decomposition.md` for the full subtask
> execution log. P1.F3b is the next gated child; it has no source-execution
> authorization until its own current RRI, approval card, and phase-1 review
> are presented and approved.

## Task map

| ID | Title | Status | Depends on |
|---|---|---|---|
| P1 | Maintainable mobile P2P foundation + replication proof (planning parent) | APPROVED 2026-08-27 — RRI 94; decomposed; no direct source execution | P0 PASS |
| P1.F1 | Reproducible worklet bundle + versioned RPC contract | PASS — owner verified 2026-08-27 | Revised P1 + ADR-043 approval — satisfied 2026-08-27 |
| P1.F2 | Mobile service ownership + composition | PASS — owner verified 2026-08-27 | P1.F1 PASS — satisfied 2026-08-27 |
| P1.F3a | P0 runtime-scaffold migration + retirement (planning parent) | Decomposed — no direct source execution | P1.F2 PASS |
| P1.F3a.1 | P0 characterization migration | PASS — owner verified 2026-08-27 | P1.F2 PASS |
| P1.F3a.2 | P0 runtime-scaffold retirement | PASS — Done 2026-08-27 | P1.F3a.1 PASS — satisfied 2026-08-27 |
| P1.F3b | P0 config/dependency cleanup | Implemented + audited 2026-08-27 — blocked on Android device proof (X28) | P1.F3a.2 PASS — satisfied 2026-08-27 |
| P1.A1 | Hyperdrive/Corestore Android bundle smoke proof (planning parent) | Decomposed 2026-08-28 — no direct source execution | P1.F3b PASS |
| P1.A1a | Add Corestore/Hyperdrive deps + bundle check | PASS — Done 2026-08-28 | P1.F3b PASS |
| P1.A1b.0 | Proof-storage contract preflight | PASS — Done 2026-08-30 | P1.A1a PASS — satisfied 2026-08-28 |
| P1.A1b | Transient drive open/close logic (HP-A1) | Ready — RRI 50 Med-high; needs phase-1/card/approval | P1.A1b.0 PASS — satisfied 2026-08-30 |
| P1.A1c | Typed error handling (EC-A1) | Blocked | P1.A1b PASS |
| P1.A1d | Tests + evidence + closure | Blocked | P1.A1c PASS |
| P1.A2 | Transient seed lifecycle + residue cleanup | Deferred — needs current RRI/card/approval | P1.A1d PASS |
| P1.B1 | Isolated Hyperswarm replication transport | Deferred — needs current RRI/card/approval | P1.A2 PASS |
| P1.B2 | Verification, reconnect + fail-closed witness | Deferred — needs current RRI/card/approval | P1.B1 PASS |

The former combined `P1.A — Ephemeral seed fixture and bundle boundary` planning
parent and its A1/A2 interpretation are superseded. Historical artifacts remain
at `docs/audit/mvp0-p2p-p1a-approval-card.md` and
`docs/audit/mvp0-p2p-p1a-rri.md`; they authorize nothing.

## P1 — Maintainable mobile P2P foundation and replication proof

- **Status:** Revised planning parent approved 2026-08-27 with ADR-043 accepted.
  The previous approval remains superseded by the material architecture/scope
  change. P1.F1 and P1.F2 are closed PASS; P1.F3a is a planning parent and
  P1.F3a.1 is next, but no later-child source work is authorized.
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
2. Score, present, approve, implement, and close P1.F1 → P1.F2 → P1.F3a.1 →
   P1.F3a.2 → P1.F3b → (P1.A1a → P1.A1b.0 → P1.A1b → P1.A1c → P1.A1d) → P1.A2 → P1.B1 →
   P1.B2 in order. P1.A1 has four executable children plus the closed
   documentation-only P1.A1b.0 preflight. Do not edit source for a child before
   its own approval/delegation and do not start the next child before PASS/status
   sync.
3. Close P1 only after all executable children (including all four P1.A1
   children), five parent Reflection passes, coverage
   certification, owner final verification, and status synchronization pass.

## P1.F1 — Reproducible worklet bundle and versioned RPC contract

- **Status:** PASS — implementation, automated acceptance, and repository-owner
  verification completed on 2026-08-27. Parent/ADR gate satisfied.
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

### Code-solution review

Code-solution review: REVIEW-OVERRIDE — explicit owner-directed MVP0-P2P
exception; `docs/audit/mvp0-p2p-review-exception.md`.

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner
- Scope-note: skips only P1.F1 phase-2 peer review; tests, coverage, Reflection,
  scope checks, and owner final verification remain mandatory.

### Implementation evidence

`docs/audit/mvp0-p2p-p1-f1-implementation.md` records the ADR-038 route,
dependency/bundle digests, exact verification output, and the three complete
Reflection passes. Automated acceptance and owner verification are PASS; P1.F1
is closed and P1.F2's dependency is satisfied.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-F1 | Happy path | deterministic bundle plus compatible handshake/capabilities, bounded ping, lifecycle events, and clean shutdown | `mobile/__tests__/p2p/runtime-protocol.test.ts::HP-F1 builds the committed worklet bundle deterministically`; `mobile/__tests__/p2p/runtime-protocol.test.ts::HP-F1 negotiates capabilities, pings, and shuts down without pending work`; `mobile/__tests__/p2p/runtime-protocol.test.ts::HP-F1 carries requests and lifecycle events through bare-rpc`; `mobile/__tests__/p2p/runtime-protocol.test.ts::HP-F1 worklet replies to handshake/ping/shutdown and emits suspend/resume` | passed |
| EC-F1 | Edge case | drift, version/payload/capability errors, timeout/channel closure, invalid lifecycle, fatal exceptions/rejections, redaction, and pending cleanup fail closed | `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 rejects unsupported versions and malformed payloads with typed errors`; `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 rejects incomplete capabilities and redacts remote failure details`; `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 times out, closes the channel, and clears pending work`; `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 validates lifecycle events and rejects invalid states`; `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 emits a redacted fatal receipt`; `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 returns typed errors for unsupported worklet requests` | passed |

### Owner final verification

- **Owner:** Matias (repository owner)
- **Date:** 2026-08-27
- **Statement:** After receiving the published workflow/evidence report, the
  owner explicitly directed Codex to verify that P1.F1 was fully closed so work
  could proceed to F2. The owner accepts the HP-F1/EC-F1 certification, automated
  evidence, phase-2 override, and published commit `8fa5b5a` as P1.F1 PASS.
- **Commands run:** `cd mobile && npm run build:bare-worklet`; `cd mobile && npm
  run check:bare-worklet`; `cd mobile && npm test -- --runInBand
  --runTestsByPath __tests__/p2p/runtime-protocol.test.ts`; `cd mobile && npm
  test -- --runInBand --runTestsByPath __tests__/p2p/runtime-protocol.test.ts
  --coverage --collectCoverageFrom='src/p2p/runtime/protocol.ts'
  --collectCoverageFrom='src/p2p/runtime/worklet.ts'`; `cd mobile && npm run
  typecheck`; `cd mobile && npm run lint`; `cd mobile && npm test --
  --runInBand`; `python3 scripts/check-maintainability.py --base
  origin/feature/p2p-mvp-core`; `make qa-docs`; `git diff --check`; `git
  rev-parse HEAD`; `git rev-parse origin/feature/p2p-mvp-core` — all relevant
  gates PASS and both revisions resolved to
  `8fa5b5a7f2d406da26028b2933901e87dc25153a` before closure edits.

## P1.F2 — Mobile service ownership and composition

- **Status:** Closed PASS after repository-owner verification on 2026-08-27.
- **Effort / RRI:** L / 55 Med-high. Full report:
  `docs/audit/mvp0-p2p-p1-f2-rri.md`; approval card:
  `docs/audit/mvp0-p2p-p1-f2-approval-card.md`.
- **Allowed paths:** `mobile/App.tsx`, `mobile/src/navigation/RootNavigator.tsx`,
  the existing P0 probe/bridge, `mobile/src/p2p/runtime/BareRuntimeClient.ts`,
  `mobile/src/p2p/P2PService.ts`, `mobile/src/p2p/P2PProvider.tsx`, focused P2P
  tests `bare-bridge.test.ts`, `p2p-service.test.ts`, and
  `p2p-provider.test.tsx`, and F2 evidence only.
- **Verification companion:** the owner explicitly authorized `P1.F2.V`, a
  separate low-band, test-only adjustment to
  `mobile/__tests__/mobile.auth-flow.test.tsx` so the existing integration test
  mounts the required provider composition. Its RRI is 22 and its scope does
  not expand F2's frozen implementation package:
  `docs/audit/mvp0-p2p-p1-f2v-rri.md`.
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

### Task-analysis review

Task-analysis review: gemma
`docs/audit/mvp0-p2p-p1-f2-phase1-review.md` - PASS

- **Reviewer:** local `gemma4:26b-a4b-it-qat`; 3/3 usable reduced-profile
  passes, no findings.
- **Fallbacks:** Muse Glimmer not triggered; D14 not triggered. The prior
  MVP0-P2P review exception is not invoked for P1.F2.

### Code-solution review

Code-solution review: gemma
`docs/audit/mvp0-p2p-p1-f2-phase2-review-remediation.json` - PASS

- The first review found an asynchronous lifecycle race; its remediation and
  the final three-pass disposition are recorded in
  `docs/audit/mvp0-p2p-p1-f2-implementation.md`.

### Reflection log

Required passes: 3 (`55` → `Med-high`). Complete Draft → Critique → Revise
records are in `docs/audit/mvp0-p2p-p1-f2-implementation.md`.

### Unit coverage certification

HP-F2/EC-F2 and the owner-authorized test-only P1.F2.V companion are mapped
to passing unit/integration evidence in
`docs/audit/mvp0-p2p-p1-f2-implementation.md` (direct scope coverage: 98.37%
lines).

### Owner final verification

The repository owner confirmed final verification on 2026-08-27. P1.F2 is
closed PASS. Do not begin P1.F3a.1 until its current RRI, approval card, and
explicit approval are recorded.

## P1.F3a — P0 runtime-scaffold migration and retirement (planning parent)

- **Status:** Decomposed by owner approval on 2026-08-27. The prior prospective
  RRI 43 was the base score; the mandatory `refactor_and_behavior` penalty
  makes the combined scope RRI 51 and requires this split. P1.F3a itself
  authorizes no source work.
- **Objective:** preserve the P0 characterization contract in a replacement
  harness before separately retiring the obsolete runtime scaffold.
- **Dependency:** P1.F2 PASS — satisfied. P1.F3a.1 must PASS before P1.F3a.2
  can be presented.

### P1.F3a.1 — P0 characterization migration

- **Status:** Closed PASS after repository-owner verification on 2026-08-27.
  The RRI/card and in-session owner approval are recorded in the F3a.1 audit
  artifacts.
- **Allowed paths:** `mobile/App.tsx`,
  `mobile/src/p2p/development/P2PDevelopmentHarness.tsx`,
  `mobile/__tests__/p2p/p2p-development-harness.test.ts`, and F3a.1 evidence only. The
  P0 probe, bridge, protocol, inline worklet, and their existing tests remain
  unchanged as the parity oracle.
- **Objective:** make the development-only harness exercise the ADR-043
  `BareRuntimeClient` path and transfer every P0 characterization case without
  retiring any P0 source.
- **HP-F3a.1:** the new harness, when explicitly enabled, completes bounded
  `initialize → ping → shutdown` through the existing service/runtime seam;
  the P0 oracle still passes unchanged.
- **EC-F3a.1:** malformed/remote replies, startup release, and late/closed
  operations are asserted as the replacement protocol's typed, redacted
  failure classes; an unmapped P0 case or a duplicate active owner fails the
  task.
- **Acceptance:** record a P0-to-ADR-043 behavioral map before retirement;
  keep all P0 files byte-for-byte unchanged; only the new harness is wired in
  `App.tsx`; no network or product API is added; focused migration tests,
  typecheck, lint, and full Jest pass.
- **Evidence to emit:** current RRI/card/route receipt, characterization map,
  unchanged-P0 inventory, focused-test output, and checks.
- **Status artifacts affected:** this ledger, P1 plan, ADR-043 implementation
  references, and F3a.1 audit artifacts.
- **Handoff prompt:** `P1.F3a.1 — move every P0 behavior into the ADR-043
  development harness while leaving the P0 scaffold untouched as an oracle;
  stop before any deletion or config cleanup.`

#### Review and verification

Task-analysis review: REVIEW-OVERRIDE
`docs/audit/mvp0-p2p-review-exception.md` — explicit owner-directed MVP0-P2P
exception; the unavailable local chain is recorded in
`docs/audit/mvp0-p2p-p1-f3a1-phase1-review.md`.

Code-solution review: REVIEW-OVERRIDE
`docs/audit/mvp0-p2p-review-exception.md` — tests, coverage, scope checks, and
owner final verification remain mandatory.

`docs/audit/mvp0-p2p-p1-f3a1-implementation.md` records all three Reflection
passes, the P0 behavioral map/inventory, 100% direct harness line coverage,
and passing focused/typecheck/lint/full-Jest verification.

### Owner final verification

- **Owner:** Matias (repository owner)
- **Date:** 2026-08-27
- **Statement:** I verified every happy path and edge case defined for
  P1.F3a.1 has unit test evidence that replicates the expected behavior. I
  independently re-verified (not just re-read) the F3a.1 implementation
  evidence: all six declared P0 file SHA-256 checksums match the current
  working tree exactly (no diff), `npm run typecheck` and `npm run lint`
  pass, and the three focused suites (`p2p-development-harness.test.ts`,
  `bare-bridge.test.ts`, `p2p-provider.test.tsx`) pass 17/17. This confirms
  P1.F3a.1 PASS and authorizes presenting P1.F3a.2.
- **Commands run:** `shasum -a 256` on the six declared P0 files; `npm run
  typecheck`; `npm run lint`; `npx jest __tests__/p2p/p2p-development-harness.test.ts
  __tests__/p2p/bare-bridge.test.ts __tests__/p2p/p2p-provider.test.tsx --runInBand`.

### P1.F3a.2 — P0 runtime-scaffold retirement

- **Status:** [x] Done — 2026-08-27. Implemented via Low-band decomposition
  (owner-directed, per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` §
  Post-repair-budget Low-band decomposition, applied proactively) into four
  RRI 0-25 subtasks (F3a.2-i/ii/iii/iv). Deleted `bare-worklet.ts`,
  `bare-protocol.ts`, `bare-bridge.ts`, `AndroidBareRuntimeProbe.tsx`,
  `bare-bridge.test.ts`, `p2p-provider.test.tsx`. Verified: `npm run
  typecheck` clean, `npm run lint` clean, `npx jest --runInBand` 24/24
  suites / 262/262 tests passing, including the ADR-043 boundary suite
  (`__tests__/p2p/`: 3 suites, 27 tests). Full execution log:
  `docs/audit/mvp0-p2p-p1-f3a2-decomposition.md`. Whole-task record
  retained: `docs/audit/mvp0-p2p-p1-f3a2-rri.md` (RRI 29 Moderate,
  superseded for implementation-routing purposes only by the
  decomposition).
- **Allowed paths:** the four retired P0 source files under `mobile/src/p2p/`,
  `mobile/__tests__/p2p/bare-bridge.test.ts`,
  `mobile/__tests__/p2p/p2p-provider.test.tsx`, and F3a.2 evidence only.
- **Objective:** delete the obsolete probe, custom bridge/protocol, inline
  worklet, and superseded tests after the independent migration evidence is
  PASS.
- **HP-F3a.2:** the retained development harness proves the same bounded
  lifecycle after every P0 runtime source file has gone.
- **EC-F3a.2:** a stale import, duplicate runtime owner, missing migration
  evidence, or failed Android ping blocks deletion/closure.
- **Acceptance:** remove every tracked source/test import of retired symbols;
  leave P0 audit history untouched; retain exactly one diagnostic/runtime path;
  typecheck, lint, focused/full Jest, and a new physical Android ping proof
  pass after deletion. `bareRuntimeProbe` configuration/script cleanup remains
  exclusively P1.F3b.
- **Evidence to emit:** current RRI/card/route receipt, before/after reference
  inventory, F3a.1 parity map, deletion manifest, checks, and Android proof.
- **Status artifacts affected:** this ledger, P1 plan, ADR-043 implementation
  references, and F3a.2 audit artifacts.
- **Handoff prompt:** `P1.F3a.2 — after P1.F3a.1 PASS, retire only the tracked
  P0 runtime scaffold and superseded tests; preserve audit history and stop if
  any import, parity condition, or Android proof is incomplete.`

## P1.F3b — P0 config/dependency cleanup

- **Status:** Implemented and audited 2026-08-27; **not PASS** — blocked on the
  two Android device-dependent criteria (full build/ping, and the executed
  `useLegacyPackaging` native A/B), folded into roadmap X28's hardware
  verification pass. Rename delivered and verified; the dependency/build audit
  concluded **retain** for every contested item, so nothing was removed and
  `mobile/package-lock.json` is unchanged. Notably `react-native-b4a` was
  retained despite having zero JS imports: it is a `peerOptional` of `b4a`,
  selected by `b4a`'s `react-native` export condition and wired by autolinking,
  so removal would have degraded the RPC data path **silently** — exactly what
  EC-F3b forbids. Two files outside `allowed_paths` were mechanically forced
  (`mobile/App.tsx` as the renamed key's sole consumer;
  `mobile/src/p2p/runtime/worklet.bundle.js` because `bare-pack --linked`
  embeds `mobile/package.json` verbatim). Full evidence:
  `docs/audit/mvp0-p2p-p1-f3b-implementation.md`.
- **Effort / RRI:** prospective `M / 30 Moderate`; **measured 24 Low**
  (`docs/audit/mvp0-p2p-p1-f3b-rri.md`) — the audit's retain-everything result
  reduced the diff to renames plus one regenerated artifact. Owner directed
  direct execution without a card.
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

### P1.F3b-fix-1 — `protocol.ts` import-syntax fix

- **Status:** `[x] Done 2026-08-28.` Emulator access (`fenix_t7`, Android 34)
  became available and exposed a real Metro-bundling blocker:
  `mobile/src/p2p/runtime/protocol.ts:2` used TypeScript import-equals syntax
  (`import RPC = require("bare-rpc");`), which Metro/Babel cannot transform.
  Changed to `import RPC from "bare-rpc";` — safe because `esModuleInterop` is
  active and `bare-rpc` ends `export = RPC`. The committed
  `worklet.bundle.js` drifted (it embeds a transpiled copy of `protocol.ts`)
  and was regenerated. Owner directed direct execution without a card.
- **Effort / RRI:** `S / 17 Low` — single-line import-syntax change plus a
  regenerated build artifact. Below the RRI 26 approval gate and below the
  RRI 26 Reflection-log threshold.
- **Allowed paths:** `mobile/src/p2p/runtime/protocol.ts`,
  `mobile/src/p2p/runtime/worklet.bundle.js`.
- **HP-fix-1:** the fixed import compiles and the RPC contract still
  handshakes, pings, and shuts down cleanly; the regenerated bundle matches a
  deterministic rebuild.
- **EC-fix-1:** the changed import yields a usable `RPC` value at runtime, not
  an interop-broken namespace object; typed errors still raised on malformed
  payloads.
- **Verification:** `npm run typecheck` clean; `npx jest __tests__/p2p/`
  27/27 passed; `npm run check:bare-worklet` no drift
  (`sha256=3e199894…9654`); Metro rebuild bundles with no `SyntaxError`.
- **Not closed by this fix:** the on-device `initialize → ping → shutdown`
  run still fails, root-caused to a confirmed upstream `bare-module@6.3.2`
  bundle-evaluation-order defect (fixed upstream in `6.4.0`; no
  `react-native-bare-kit` release has picked it up). A minimal,
  dependency-free bundle reproduces the identical crash, ruling out this
  repository's source as the cause. The self-built `libbare-kit.so`
  workaround was evaluated and rejected by the owner on 2026-08-28. Both
  remain tracked in roadmap **X28**, not here.
- **Closure evidence:** `docs/audit/mvp0-p2p-p1-f3b-implementation.md`
  §§ 9, 10 (review override, unit coverage certification, owner final
  verification).

## P1.A1 — Hyperdrive/Corestore Android bundle smoke proof (planning parent)

- **Status:** Decomposed 2026-08-28 at the owner's explicit request. The
  documentation-only P1.A1b.0 preflight closed PASS on 2026-08-30; it raised
  P1.A1b's executable scope to RRI 50 Med-high. No direct source execution
  under this parent ID — see `P1.A1a`-`P1.A1d` and P1.A1b.0.
- **Objective (unchanged, inherited by children):** prove compatible
  Corestore/Hyperdrive dependencies can bundle, open an empty transient drive
  on Android, close it, and perform no discovery — a bundling/dependency
  smoke proof, not a runtime execution proof (device runtime remains blocked
  by the unrelated upstream `bare-module@6.3.2` defect, tracked as X28).
- **Task-analysis review (parent-level, before decomposition):** gemma
  (`gemma4:26b-a4b-it-qat`, 3/3 passes) - PASS. Findings (2 consensus minor,
  0 blocking), incorporated into every child's acceptance criteria below:
  1. "redacted capability receipt" was ambiguous — bounded to `capability` +
     `schema_version` fields only (P1.A1b).
  2. Risk of conflating the X28 transport defect with a bundling test
     failure — any failure attributable to X28 must be classified
     `Environment/Blocked`, not a test failure (P1.A1c).
  3. (likely-false-positive, adopted anyway as good practice) "invalid path"
     EC case must run through a local-only/stubbed filesystem driver, never
     triggering network/Hyperswarm code (P1.A1c).
  4. (likely-false-positive) receipt schema ambiguity for automated
     assertions — same fix as #1.
- **Sequencing:** P1.A1a -> P1.A1b.0 -> P1.A1b -> P1.A1c -> P1.A1d, in order. P1.A2
  depends on P1.A1d PASS (all four children PASS), not on this parent ID
  directly.

### P1.A1a — Add Corestore/Hyperdrive dependencies + bundle check

- **Status:** PASS — Done 2026-08-28. Full evidence:
  `docs/audit/mvp0-p2p-p1-a1-implementation.md` § P1.A1a.
- **Effort / RRI:** S / 14 Low.
- **Allowed paths:** `mobile/package.json`, `mobile/package-lock.json`.
- **Objective:** pin compatible Corestore/Hyperdrive dependency versions; no
  new logic. `npm run check:bare-worklet` and `npm run typecheck` must stay
  clean after the dependency add.
- **HP-A1a:** dependency versions resolve, install cleanly, and the existing
  worklet bundle check reports no drift.
- **EC-A1a:** an incompatible/unresolvable version pin fails `npm install`
  loudly (no silent partial lockfile state).
- **Evidence to emit:** exact resolved versions, `npm run check:bare-worklet`
  output, `npm run typecheck` output.
- **Handoff prompt:** `P1.A1a — add only Corestore/Hyperdrive to
  mobile/package.json + package-lock.json; no new source logic; confirm
  check:bare-worklet and typecheck stay clean.`

### P1.A1b.0 — Proof-storage contract preflight

- **Status:** PASS — Done 2026-08-30. It froze the contract in
  `docs/audit/mvp0-p2p-p1-a1b-storage-contract.md` and superseded P1.A1b's
  preliminary RRI 53 with its final presentation-time RRI 50.
- **Effort / RRI:** S / 10 Low. Full report:
  `docs/audit/mvp0-p2p-p1-a1b-preflight-rri.md`.
- **Allowed paths:** `docs/tasks/mvp0-p2p-p1-replication.md`,
  `docs/plan/mvp0-p2p-p1-replication.md`,
  `docs/plan/roadmap.md`, and P1.A1b audit artifacts only. No `mobile/`
  source, package, lockfile, generated bundle, Android, or device changes.
- **Objective:** turn the P1.A1b phase-1 blockers into one ADR-043-aligned,
  implementation-ready storage/RPC contract without opening a drive or starting
  network activity.
- **HP-A1b.0:** the contract identifies the authoritative host-side proof-cache
  root, its representation at the Bare boundary, the exact RPC request/response
  schema, direct storage-import/dependency ownership, and close/failure order.
- **EC-A1b.0:** reject any proposal that accepts a caller-controlled root,
  relies on an undocumented transitive `bare-fs` import, exposes a product P2P
  API, starts Hyperswarm/discovery, creates durable storage, or treats X28 as a
  source-test failure.
- **Acceptance:**
  1. Record a bounded source-of-truth and validation rule for a single
     `Paths.cache/dubbridge-p2p/proofs/<run-id>` child, including URI-versus-
     native-path handling at the host/Bare boundary.
  2. Freeze the P1.A1b command name, request schema, exactly-two-field success
     receipt (`capability`, `schema_version`), typed error surface, and close
     ordering; neither key material nor filesystem paths may appear in receipts.
  3. Record whether `bare-fs` must become a direct dependency; if it does, add a
     separate dependency-only child before P1.A1b rather than silently widening
     it.
  4. Amend P1.A1b's allowed paths and HP/EC/test evidence to the resulting
     minimal, testable source surface, then recalculate its RRI and require a
     new phase-1 PASS.
- **Evidence to emit:** installed package/version evidence, a frozen contract
  record, revised downstream task scope, and the replacement P1.A1b RRI/phase-1
  artifacts.
- **Status artifacts affected:** this ledger, the P1 plan, roadmap, and
  `docs/audit/mvp0-p2p-p1-a1b-*.md`.
- **Task-analysis review:** n/a — documentation/contract-only task.
- **Code-solution review:** n/a — documentation/contract-only task.
- **Handoff prompt:** `P1.A1b.0 — resolve and record the proof-storage and RPC
  contract for P1.A1b only; do not edit mobile source or dependencies, start a
  drive, or add network behavior.`

### P1.A1b — Transient drive open/close logic (HP-A1)

- **Status:** Ready — P1.A1b.0 PASS satisfied 2026-08-30. It still requires a
  fresh phase-1 PASS, Compact Approval Task Card v2, and explicit approval
  before source execution.
- **Effort / RRI:** L / 50 Med-high. Full report:
  `docs/audit/mvp0-p2p-p1-a1b-rri-v2.md`. The former S / 25 estimate and
  blocked RRI 53 assessment are historical only.
- **Allowed paths:** `mobile/src/p2p/proof/P1ProofRuntimeFactory.ts` (new),
  `mobile/src/p2p/runtime/protocol.ts`, `mobile/src/p2p/runtime/worklet.ts`,
  `mobile/src/p2p/runtime/worklet.bundle.js`, and
  `mobile/__tests__/p2p/runtime-protocol.test.ts` only.
- **Objective:** use an explicit proof-only runtime factory to pass one
  host-derived cache-directory URI as immutable worklet bootstrap data, then
  open and deterministically close an empty Hyperdrive/Corestore drive without
  discovery or a product-facing API.
- **HP-A1:** the factory derives `new Directory(Paths.cache,
  "dubbridge-p2p", "proofs", runId).uri` from a validated generated `runId`,
  starts a proof worklet with that URI as `Bare.argv[0]`, and the new
  `OPEN_CLOSE_TRANSIENT_DRIVE` RPC command returns exactly
  `{ capability: "transient-hyperdrive-corestore", schema_version: 1 }` after
  `drive.close()` succeeds.
- **EC-A1b:** an invalid `runId` or missing/non-`file:` bootstrap URI fails
  closed with `PROOF_STORAGE_CONFIG_INVALID` before Corestore/Hyperdrive opens;
  it does not create a worklet storage handle, Hyperswarm/discovery activity,
  product persistence, or a product P2P command.
- **Acceptance:**
  1. The URI is created host-side and passed unchanged only in the worklet's
     start arguments; no RPC payload, receipt, log, or product API contains a
     filesystem path.
  2. The worklet imports only Corestore/Hyperdrive for storage; it adds no direct
     application dependency on `bare-fs`. The bundle check must prove their
     existing Bare `fs` mapping resolves.
  3. `drive.close()` is the sole normal close operation because Hyperdrive closes
     its Corestore; `store.close()` is used only when construction fails before a
     drive exists. Errors remain redacted; P1.A1c owns later granular
     dependency/open/close error taxonomy.
  4. Focused unit tests cover the exact startup arguments/receipt and the invalid
     configuration boundary; `npm run check:bare-worklet`, typecheck, lint, and
     the focused P2P Jest suite stay clean.
- **Evidence to emit:** HP-A1 proof log with no path/key material, exact
  Corestore/Hyperdrive/bundle versions, bundle resolution/check output, focused
  tests, and phase-2/coverage/owner evidence at closure.
- **Handoff prompt:** `P1.A1b — add only the proof runtime factory and the
  versioned open/close command defined in the P1.A1b storage contract. Pass the
  host-created cache URI only through Bare argv, return exactly the two-field
  receipt, and add no network, product API, direct bare-fs dependency, or
  persistence.`

### P1.A1c — Typed error handling (EC-A1)

- **Status:** Blocked on P1.A1b PASS.
- **Effort / RRI:** S / 23 Low.
- **Allowed paths:** the packaged runtime worklet (`mobile/src/p2p/runtime/**`).
- **Objective:** typed, fail-closed error handling for dependency load,
  bundle, invalid-path, open, and close failures.
- **EC-A1:** dependency load, bundle, invalid path, open, or close failure is
  typed and cannot report drive readiness. The invalid-path case is exercised
  through a local-only/stubbed filesystem driver — verified to trigger no
  network/Hyperswarm code path (Gemma finding #3). Any failure attributable
  to the X28 upstream transport/worklet execution defect is classified
  `Environment/Blocked`, not a test failure (Gemma finding #2).
- **Evidence to emit:** EC-A1 proof log covering each typed failure mode.
- **Handoff prompt:** `P1.A1c — add typed fail-closed error handling for
  Hyperdrive/Corestore load/bundle/open/close failures; invalid-path case via
  a local-only stub driver only; classify X28-attributable failures as
  Environment/Blocked, never a test failure.`

### P1.A1d — Tests + evidence + closure

- **Status:** Blocked on P1.A1c PASS.
- **Effort / RRI:** S / 10 Low.
- **Allowed paths:** `mobile/__tests__/p2p/hyperdrive-smoke.test.ts` (new),
  `docs/audit/mvp0-p2p-p1-a1-implementation.md` (new).
- **Objective:** certify HP-A1/EC-A1 with Jest coverage and close the P1.A1
  chain.
- **Acceptance:** `hyperdrive-smoke.test.ts` exercises HP-A1 and every EC-A1
  typed-failure branch; unit coverage certification maps each to a passing
  test; owner final verification recorded.
- **Evidence to emit:** test run output, unit coverage certification table,
  owner final verification block.
- **Status artifacts affected:** this ledger (mark P1.A1a-d PASS/Done, parent
  P1.A1 Done), `docs/plan/mvp0-p2p-p1-replication.md`, roadmap MVP0-P2P row.
- **Handoff prompt:** `P1.A1d — add hyperdrive-smoke.test.ts covering HP-A1
  and every EC-A1 branch; write the P1.A1 evidence doc; certify coverage and
  record owner verification.`

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

**Pending completion of P1.F1, P1.F2, P1.F3a.1, P1.F3a.2, P1.F3b, P1.A1,
P1.A2, P1.B1, and P1.B2.**

## Handoff prompt

`P1 — follow accepted ADR-043 and this ledger; work only on the currently
approved child, synchronize its evidence, and stop before the next child or P2.`
