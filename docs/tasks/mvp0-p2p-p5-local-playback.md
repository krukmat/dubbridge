---
type: TaskList
title: "Tasks: P5 Local HLS playback through the existing player"
status: planned
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p5-local-playback.md
behavioral_coverage_contract: behavior-v2
---

# P5 — planning task ledger

**Status:** Planned; no task activated or implemented by this documentation update.
**Phase gate:** P4 PASS.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P5.T0 | Gateway/session contract freeze | planning | M | P4 PASS | Planned; not activated |
| P5.T1 | Loopback ciphertext decryption gateway | development | L | T0 PASS | Planned; not activated |
| P5.T2 | Existing VideoPlayer and deterministic teardown | development | M | T1 PASS | Planned; not activated |
| P5.T3 | Playback and secret-boundary certification | development/evidence | M | T2 PASS | Planned; not activated |


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

**Status:** Planned; not activated.

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

**Status:** Planned; not activated.

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

**Status:** Planned; not activated.

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

**Status:** Planned; not activated.

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
