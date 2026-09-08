---
type: TaskList
title: "Tasks: P7 Exact-artifact end-to-end P2P certification"
status: planned
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p7-certification.md
behavioral_coverage_contract: behavior-v2
---

# P7 — planning task ledger

**Status:** Planned; no task activated or implemented by this documentation update.
**Phase gate:** P2-P6 PASS; S-230-T7p PASS.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P7.T0 | Certification profile and artifact manifest | planning/config | M | P2-P6 PASS; S-230-T7p PASS | Planned; not activated |
| P7.T1 | No-fallback controls and control-plane checks | operational/evidence | M | T0 PASS | Planned; not activated |
| P7.T2 | Physical owner-to-viewer end-to-end proof | operational/evidence | M | T1 PASS | Planned; not activated |
| P7.T3 | Certification verdict and T9g handoff | operational/evidence | S | T2 evidence complete (PASS or failure) | Planned; not activated |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P7 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p7-certification.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P7.T0 — Certification profile and artifact manifest

**Type:** planning/config

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P2-P6 PASS; S-230-T7p PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Bind backend revision/image digests, RC checksum, device/runtime, fixture and profile; define audience-media denial, retained control-plane probes and redacted evidence capture.

- **HP-P7.T0-1:** The manifest identifies exactly the deployed backend and physically proved RC used by the run.
- **EC-P7.T0-1:** Artifact drift or a profile that disables control-plane APIs blocks the run; no reuse of mismatched evidence.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P2-P6 PASS; S-230-T7p PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P7.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P7.T1 — No-fallback controls and control-plane checks

**Type:** operational/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Execute positive control-plane and negative remote-media probes; collect network/server evidence showing audience HTTP/S3 denial with loopback available.

- **HP-P7.T1-1:** Authentication, upload/assets, invites and audit work while remote audience-media requests fail.
- **EC-P7.T1-1:** A remote-media request succeeds or an essential control-plane route fails: record NOT_CERTIFIED and stop the certification path.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P7.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P7.T2 — Physical owner-to-viewer end-to-end proof

**Type:** operational/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Run owner Login → Upload → S-120 → P2P Publish → Ready → Invite and viewer Login → Claim → Invites → Sync → Verify → READY → Play on exact artifacts; capture backend readiness, network and Android evidence.

- **HP-P7.T2-1:** The short video plays fully from verified replicated ciphertext on physical Android with no remote-media fallback.
- **EC-P7.T2-1:** Any HTTP/S3 audience-media bytes, incorrect identity, incomplete verification or artifact mismatch yields MVP0_P2P_NOT_CERTIFIED.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P7.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P7.T3 — Certification verdict and T9g handoff

**Type:** operational/evidence

**Effort:** S (provisional; re-score/decompose at activation)

**Depends on:** T2 evidence complete (PASS or failure)

**Status:** Planned; not activated.

**Acceptance criteria:** Emit MVP0_P2P_CERTIFIED only when every required case passes; otherwise emit MVP0_P2P_NOT_CERTIFIED with blocker IDs. Link exact evidence, release CI/X28, X29, rollback/log/soak status for T9g; P7 never issues GO itself.

- **HP-P7.T3-1:** Complete exact-artifact evidence yields certification and a reviewable T9g handoff.
- **EC-P7.T3-1:** Missing/failing evidence yields NOT_CERTIFIED/NO-GO input; backend preview is labeled and never presented as invited playback.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 evidence complete (PASS or failure);
freeze and score exact paths, preserve the accepted boundary, and deliver only
P7.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.
