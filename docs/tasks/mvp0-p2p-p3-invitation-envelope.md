---
type: TaskList
title: "Tasks: P3 Invitation, audience authorization, and K1 device envelope"
status: planned
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p3-invitation-envelope.md
behavioral_coverage_contract: behavior-v2
---

# P3 — planning task ledger

**Status:** Planned; no task activated or implemented by this documentation update.
**Phase gate:** P2 PASS; Accepted ADR-044.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P3.T0 | Contract and executable-path freeze | planning | M | P2 PASS | Planned; not activated |
| P3.T1 | Invitation persistence, claim, and inbox | development | L | T0 PASS | Planned; not activated |
| P3.T2 | O3 authorization and native K1 envelope delivery | development | L | T1 PASS | Planned; not activated |
| P3.T3 | P3 integration certification and closure | development/evidence | M | T2 PASS | Planned; not activated |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P3 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p3-invitation-envelope.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P3.T0 — Contract and executable-path freeze

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P2 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Freeze invitation/claim/inbox and separate O3 authorization contracts, active-device binding, expiry/revocation predicates, audit map and exact path ownership; score the coherent implementation parent and independently meaningful leaves.

- **HP-P3.T0-1:** Ready descriptor and accepted D2 produce a complete API/schema/envelope contract with owned paths.
- **EC-P3.T0-1:** A missing readiness or Keystore boundary remains explicitly blocked; claim alone never grants envelope access.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P3.T1 — Invitation persistence, claim, and inbox

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Implement hash-only invitation storage, owner-only creation on P2P_READY content, atomic single-viewer claim and scoped inbox; preserve same-viewer idempotency and durable audit.

- **HP-P3.T1-1:** Owner creates an invite; token is returned once; eligible viewer claims and later uses their inbox without the raw token.
- **EC-P3.T1-1:** Concurrent different-viewer claims have one winner; expired/unknown/non-owner/non-ready requests fail closed without logging tokens.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P3.T2 — O3 authorization and native K1 envelope delivery

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Implement distinct backend audience authorization and all accepted D2 release predicates; prove HPKE Base P-256/HKDF-SHA256/AES-256-GCM with non-exportable Android Keystore private key, native unwrap, binding and expiry checks.

- **HP-P3.T2-1:** Eligible claimed viewer and active device receive a package-bound envelope and unwrap through the opaque native key.
- **EC-P3.T2-1:** Wrong device/package/viewer, expired or revoked authorization, non-ready publication, or missing Keystore capability produces no CK release; no software private-key fallback.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P3.T3 — P3 integration certification and closure

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Map every parent P3 HP/EC to executable evidence, including claim races, O3 denial, native Keystore interop and log/storage secret inspection; publish descriptor/native-adapter handoff to P4/P5.

- **HP-P3.T3-1:** Owner create → viewer claim → authorized envelope → native unwrap passes with exact package binding.
- **EC-P3.T3-1:** Possession of ciphertext or a valid claim without current O3 authorization never permits envelope release.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.
