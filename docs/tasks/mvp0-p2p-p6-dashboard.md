---
type: TaskList
title: "Tasks: P6 Minimal My Content and Invites dashboard"
status: planned
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p6-dashboard.md
behavioral_coverage_contract: behavior-v2
---

# P6 — planning task ledger

**Status:** Planned; no task activated or implemented by this documentation update.
**Phase gate:** P3-P5 PASS.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P6.T0 | State/action and navigation contract | planning | M | P3-P5 PASS | Planned; not activated |
| P6.T1 | Owner My Content and invite action | development | M | T0 PASS | Planned; not activated |
| P6.T2 | Viewer claim, Invites, sync and play actions | development | L | T1 PASS | Planned; not activated |
| P6.T3 | Dashboard flow and visual certification | development/evidence | M | T2 PASS | Planned; not activated |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P6 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p6-dashboard.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P6.T0 — State/action and navigation contract

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P3-P5 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Map each owner/viewer state to authoritative facts, permitted actions and loading/empty/error behavior; freeze navigation and exact component ownership.

- **HP-P6.T0-1:** Owned ready content permits Invite; an authorized verified invitation permits Play.
- **EC-P6.T0-1:** S-120 Ready alone or synced-but-unverified data never enables the corresponding P2P action.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P3-P5 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P6.T1 — Owner My Content and invite action

**Type:** development

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Render owned content and correct P2P publication state; create/copy the one-time invite through P3; use existing design primitives.

- **HP-P6.T1-1:** Owner sees Processing → Ready and can create an invite for their ready package.
- **EC-P6.T1-1:** Other-owner assets or failed/non-ready publications do not expose an invite action; server rejection remains enforced.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P6.T2 — Viewer claim, Invites, sync and play actions

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Connect claim/inbox, verified sync and existing playback through the frozen state model; handle expiry, loading/retry and account changes.

- **HP-P6.T2-1:** Viewer claims, syncs, sees Available after verification, then plays the package.
- **EC-P6.T2-1:** Expired, inaccessible, incomplete or unverified content offers no Play; another viewer invitation is never displayed.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P6.T3 — Dashboard flow and visual certification

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** Planned; not activated.

**Acceptance criteria:** Certify all P6 parent HP/EC with component/integration and Android flow evidence; inspect loading/empty/error/expired states against DESIGN.md and shipped tokens.

- **HP-P6.T3-1:** Owner and invited viewer complete their respective actions using the minimal screens.
- **EC-P6.T3-1:** Account switch clears prior content/action state; denied responses cannot leave a stale enabled Play or Invite action.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P6.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.
