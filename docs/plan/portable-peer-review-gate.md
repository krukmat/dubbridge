---
type: Plan
title: "Portable Peer-Review Gate — Band-Routed Two-Phase Review"
status: active
slice: PPR
---

# Portable Peer-Review Gate

> **Status:** Active (PPR-1 approved)
> **Origin:** Phase F of the Portable Agent Workflow Port Plan originally designed
> for fenix (`fenix/docs/plans/portable_agent_workflow_port_plan.md`). This plan
> brings the peer-review layer back into DubBridge and reshapes it as a
> **band-routed, two-phase** model suited to the existing Gemma + D14 stack.

## Purpose

Add a second axis of review independence to the workflow: the reviewer is
determined by **both** the task's RRI band and the review phase, rather than
being a flat cross-vendor scheme for all work.

The central invariant being added:

> Before a task is presented (phase 1) and before a code task is closed (phase 2),
> an independent reviewer runs whose identity is resolved from the task's RRI band.
> The reviewer is never the authoring agent or its own local model.

## The two-axis model (band × phase)

| Review phase | RRI 0–40 (Low + Moderate) | RRI 41+ (Med-high + Complex) |
|---|---|---|
| **Phase 1 — Task-analysis review** (before presentation) | **Gemma** (advisory) | **Cross-vendor peer**; D14 fallback |
| **Phase 2 — Code-solution review** (after implementation) | **Gemma Reviewer** (existing N-pass) | **Cross-vendor peer replaces Gemma**; D14 fallback |

Cross-vendor resolution (RRI 41+ only):

```
claude-code      -> codex
codex            -> claude
local-provider   -> claude
remote-provider  -> claude
unknown          -> claude
```

Report line contract (one line per phase, in task card and closure report):

```
Task-analysis review: <gemma|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
Code-solution review: <gemma|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
```

`d14` appears when the resolved peer CLI was unavailable and D14 handled the
review. `BLOCKED` stops presentation or closure until the work is revised, the
user explicitly waives, or the task is reported as blocked.

## Scope

- **In:** Policy contract, report lines, routing table, failure-mode wording,
  wiring into `AGENT_WORKFLOW_GUIDE.md`'s closure checklist, script
  (`peer-workflow-review.py`), Makefile target, and hook/CI wiring.
- **Out:** No change to product code (`apps/`, `crates/`). No weakening of the
  existing HITL approval gate or the four existing closure blocks (Gemma
  Reviewer/D14, Reflection log, coverage cert, owner verification).

## Task decomposition

| ID | Task file | Title | Type | Status |
|---|---|---|---|---|
| PPR-1 | `docs/tasks/peer-review-policy-contract.md` | Define band-routed, two-phase peer-review policy and reporting contract | docs | done |
| PPR-2 | *(no ledger — RRI 19, Low)* | Implement `scripts/peer-workflow-review.py` + adapters + tests | development | done |
| PPR-3 | *(no ledger — RRI 22, Low)* | Wire `make qa-peer-workflow-review`, pre-push routing, and CI job | config | done |
| PPR-4 | `docs/tasks/peer-review-codex-bin-resolution.md` | Make Codex peer-review resolution explicit and executable from Claude Code | development | proposed |

## Dependencies

```
PPR-1 (policy contract) ──> PPR-2 (script implements the contract)
PPR-2 ──> PPR-3 (wiring requires the script to exist)
PPR-3 ──> PPR-4 (Codex resolution hardens the implemented routing)
```

PPR-1 is self-contained (docs/policy only); PPR-2 and PPR-3 will be authored
as separate task ledgers once PPR-1 is closed.

## Constraints

- `AGENT_WORKFLOW_GUIDE.md` is the highest authority; everything this plan adds
  is subordinate to it.
- Peer review is additive to, and never replaces, the HITL approval gate or the
  existing four closure blocks.
- In the RRI 41+ band the cross-vendor peer **replaces** Gemma as reviewer;
  D14 remains the mandatory fallback.
- Enforcement is a workflow/reporting contract until PPR-2 and PPR-3 land. No
  hook denial is created by PPR-1 alone.

## Risks

- **R1** — Codex CLI may not be inherited on `PATH` by Claude Code even when the
  VS Code extension ships a working executable. PPR-4 defines an explicit
  `CODEX_BIN` resolution path before the `PATH` fallback, validates the selected
  executable, and preserves the D14/blocked-artifact fallback when invocation
  cannot run.
- **R2** — Band boundary at RRI 40/41 must resolve deterministically (inclusive
  `0–40 → Gemma`, `41+ → cross-vendor`), matching the existing Moderate/Med-high
  boundary in `RRI_POLICY.md`.
- **R3** — Enforcement creep: keep PPR-1 as a reporting/workflow contract only
  until PPR-3 introduces the Makefile target.
