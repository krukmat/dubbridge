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
| P4.T0 | Lifecycle, cache, and RPC freeze | planning | M | P3 PASS | Planned; not activated |
| P4.T1 | Product replication and bounded resume | development | L | T0 PASS | Planned; not activated |
| P4.T2 | Manifest verification and lifecycle isolation | development | L | T1 PASS | Planned; not activated |
| P4.T3 | P4 certification and P5 handoff | development/evidence | M | T2 PASS | Planned; not activated |


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

**Status:** Planned; not activated.

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

**Status:** Planned; not activated.

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

**Status:** Planned; not activated.

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

**Status:** Planned; not activated.

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
