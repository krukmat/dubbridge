---
type: TaskList
title: "Tasks: P5 Local HLS playback through the existing player"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p5-local-playback.md
behavioral_coverage_contract: behavior-v2
---

# P5 — planning task ledger

**Status:** In progress. T0 is closed; T1/T2 now have automated evidence that removes the prior missing-test blocker, but formal closure still awaits owner verification and task-governance synchronization. T3 Android certification remains outstanding.
**Phase gate:** P4 PASS remains the formal upstream gate; this evidence remediation does not override it.
**Effort:** provisional per work package below; no new RRI record is fabricated by the automated-evidence remediation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P5.T0 | Gateway/session contract freeze | planning | M | P4 PASS | `[x]` Done 2026-09-18 |
| P5.T1 | Loopback ciphertext decryption gateway | development | L | T0 PASS | Automated evidence PASS 2026-09-18; formal closure pending owner verification/governance sync |
| P5.T2 | Existing VideoPlayer and deterministic teardown | development | M | T1 PASS | Automated evidence PASS 2026-09-18; formal closure pending owner verification/governance sync |
| P5.T3 | Playback and secret-boundary certification | development/evidence | M | T2 PASS | Next evidence step; Android device/emulator certification still required |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P5 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p5-local-playback.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P5.T0 — Gateway/session contract freeze

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P4 PASS

**Status:** `[x]` Done — retro-certified 2026-09-18 under owner waiver D0-a.
RRI 25 Low (`scripts/rri.py`,
`mobile/src/p2p/playback/{P2PPlaybackLease,P2PPlaybackController}.ts`).

**Reflection:** Low band, cycle folded into review evidence below.

**Behavioral coverage certification:** HP-P5.T0-1 and EC-P5.T0-1 map to
passing `unit` evidence — 6/6 tests; the tested authorization branch
(wrong-viewer) is covered.

**Named residual (owner-accepted):** other `assertAuthorization` OR-branches
(asset/publication/lineage mismatch, expiry) are implemented but not
individually exercised by a test; no standalone contract/decision artifact
exists separate from the code. Non-blocking — same class as `P2.T4e-cov`.

**Owner final verification:** Matias, 2026-09-18 — closed on "cierra los
otros", accepting the named residual as described in
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.
Commands run: `npm test -- mobile/src/p2p/playback`.

**Acceptance criteria:** Define loopback/session ownership, package path mapping, native unwrap → transient CK handoff, expiry/error handling and teardown evidence; freeze executable paths.

- **HP-P5.T0-1:** An authorized verified package maps to a scoped local playback session.
- **EC-P5.T0-1:** Unverified package or unavailable current authorization has no valid startup transition.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P4 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P5.T1 — Loopback ciphertext decryption gateway

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** Automated evidence PASS 2026-09-18; formal Done is pending owner
verification and task-governance synchronization. The previous blocker is
resolved by executable coverage of the actual `ProductPlaybackRuntime`,
including a real OS loopback TCP-listener proof plus component coverage for
session-token scoping, traversal denial, ciphertext/AAD tamper denial,
missing-key denial, deterministic teardown, and CK zeroization on stop/start
failure. Production P5 runtime code was unchanged. Evidence:
`docs/audit/mvp0-p2p-p5-t1-t2-evidence-remediation-2026-09-18.md`.

**Acceptance criteria:** Serve only verified package HLS through loopback; use accepted K1 authenticated decryption and transient authorized CK; validate relative package paths and scope local requests to the session.

- **HP-P5.T1-1:** Verified manifest and segments decrypt at serve time for the authorized session.
- **EC-P5.T1-1:** Traversal, foreign session, altered ciphertext/AAD or missing key denies delivery without remote media fallback.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P5.T2 — Existing VideoPlayer and deterministic teardown

**Type:** development

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** Automated evidence PASS 2026-09-18; formal Done is pending owner
verification and task-governance synchronization. The prior transitive blocker
is removed: T1 now proves runtime CK zeroization/listener/package teardown,
while the existing lease tests prove idempotent release and teardown-before-
retry. Additional controller tests now cover asset/publication/lineage/viewer/
expiry authorization mismatches before K1 unwrap/playback startup. Evidence:
`docs/audit/mvp0-p2p-p5-t1-t2-evidence-remediation-2026-09-18.md`.

**Acceptance criteria:** Connect existing player to scoped loopback URL; release transient CK and gateway on stop/sign-out/error according to frozen lifecycle; preserve existing review playback.

- **HP-P5.T2-1:** Video plays end to end; stop closes listener/session and releases the key reference.
- **EC-P5.T2-1:** Gateway/native unwrap failure prevents playback; no HTTP/S3 URL substitution and no plaintext key in disk/logs.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P5.T3 — Playback and secret-boundary certification

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** Next evidence step. Do not mark PASS until Android certification evidence is recorded; T1/T2 formal closure must also be synchronized first.

**Acceptance criteria:** Prove package playback, tamper denial, teardown and review-path non-regression with executable evidence and network capture; hand off play capability/state to P6.

- **HP-P5.T3-1:** A complete short video plays from local verified ciphertext using the existing player.
- **EC-P5.T3-1:** Disabling remote audience media still permits the valid local path; key or gateway failure stops it outright.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P5.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.
