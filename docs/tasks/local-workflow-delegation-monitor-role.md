---
type: TaskList
title: "Tasks: Local workflow-delegation monitor role (future analysis)"
plan: n/a — parked, no plan file yet
status: parked
slice: local-workflow-delegation-monitor-role
---

# Tasks: Local Workflow-Delegation Monitor Role (Future Analysis)

## Status

`[ ] Parked` — not scheduled to any slice, no RRI assigned, no plan file
authored. This ledger exists only to record the open question for later
refinement, per explicit owner direction not to design or implement it now.

## Creation-task RRI and review exemption

This documentation package is task-ledger-only work, exempt from the
development-task review gates.

- `Task-analysis review: n/a - plan/task-ledger-only exemption`
- `Code-solution review: n/a - plan/task-ledger-only exemption`

## Origin

Raised 2026-08-20 during analysis of `S-230-T4g` (migration image contract
tests). The owner proposed piloting T4g's execution with the entire workflow
delegated to, and monitored by, "the local architect": Claude checks in with
it at 5-minute intervals, takes control if it stops responding, re-syncs so
it can continue, and — after a couple of failed process executions with no
visible progress — takes over monitoring entirely to finish the task.

## Problem statement

The proposal conflates two roles that currently share a model
(`muse-glimmer:30b-q4_K_M`) but have distinct, non-overlapping authority
under the present framework:

- **Local Architect / Complex Analyst (ADR-037)** — bounded, advisory-only,
  invoked once before a target ADR/plan/tasks document is authored. Per
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local Architect / Complex
  Analyst`: "It is not an implementer, not a technical judge, and does not
  replace D14 or human approval... this role is **not** a phase-1/phase-2
  reviewer anywhere... may not author the target document itself, and does
  not satisfy the human-approval gate."
- **Muse Glimmer Reviewer** — the Low-band phase-1/phase-2 code-solution
  reviewer in the band-routed peer-review chain, a separate authority scope
  from the Local Architect despite the shared model binding.

Neither role, nor any documented combination of them, is an orchestrator. Per
`AGENT_WORKFLOW_GUIDE.md`: "In every band the primary agent stays orchestrator
of record, and the human approval gate, band-resolved independent review, and
Reflection pass count are fixed by the band — never by where the code was
authored."

Separately, `scripts/local-architect/run_analysis.py` (verified by direct
read 2026-08-20) is a single bounded synchronous call to Ollama's
`/api/chat` — it produces one JSON artifact and holds no persistent,
pollable execution state. There is currently no technical mechanism to
"check in" on progress at an interval, because nothing runs continuously on
the local-model side to check in on.

The owner's proposal therefore implies a **third role** that does not exist
in the current framework: a more autonomous local-workflow delegate, with
some bounded authority to drive execution and a defined handback/takeover
protocol when it stalls or fails repeatedly. Whether and how to define that
role is the open question this ledger parks for later analysis — it is not
answered here and no implementation should proceed against it until it is.

## Open questions for future analysis

- Would this be a new ADR (a new role definition, akin to ADR-037/ADR-038),
  or an amendment to ADR-037's boundary?
- What bounded authority, if any, would it hold beyond today's advisory-only
  Local Architect and reviewer roles — and how would that avoid collapsing
  the orchestrator-of-record principle?
- Is "monitoring at an interval" even the right mechanism given local-model
  calls are single-shot, or does the underlying intent (maximize local-model
  autonomy, minimize primary-agent authoring per
  `feedback_cloud_orchestration_only`) call for a different mechanism
  entirely — e.g., a supervised retry/loop wrapper around existing bounded
  calls, rather than a new persistent "role"?
- How would Claude "taking control" on non-response interact with existing
  repair-attempt budgets (2 for Moderate, ADR-038's single-attempt gate for
  Med-high) and the mandatory D14 fallback path?
- Does this require its own HITL checkpoint distinct from the existing
  approval gate, given it would change who/what drives execution mid-task?

## Related

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local Architect / Complex Analyst (ADR-037)`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review (two phases)`
- `docs/adr/ADR-037-local-architect-complex-analyst-role.md`
- `docs/tasks/s-230-poc-v1-digitalocean.md` § S-230-T4g (origin conversation)
- `scripts/local-architect/run_analysis.py` (verified single-shot invocation mechanism)
