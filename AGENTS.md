# AGENTS.md

## Purpose

This file defines the default task-presentation contract for agents working in the `dubbridge` repository.

It works together with the canonical agent guides that govern implementation in
this repository. Read them before executing work:

- `README_AGENT_ORDER.md` — orientation and reading order for agents.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — the mandatory plan → tasks → approval →
  implement workflow.
- `docs/policies/HITL_AUTONOMY_POLICY.md` — human-in-the-loop approval rules.
- `docs/adr/` — architecture decisions that constrain implementation.
- `docs/plan/roadmap.md` — the general plan: slice sequence, dependencies, and where
  each slice/task sits.

For workflow topics, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is authoritative.
`CLAUDE.md` (project and the user's global) remains authoritative only for
topics not overridden there.

## Task Presentation Rule

When a user asks an agent to execute a staged task or a task from a task file, the agent must present the next task before execution when the active workflow requires approval.

The presentation must be concise but operationally complete.

Before presenting or executing any staged task, the agent must verify the
current requirements in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`. This file is a
presentation contract summary, not a replacement for the workflow guide.
Task-type-specific requirements defined there are mandatory even when they are
not restated verbatim below.

When answering questions about development-task completion or before marking a
development task done, the agent must explicitly determine whether the task is
exempt (docs-only, config-only, migration-only, planning, ADR, task-ledger, or
policy-only) or whether the workflow requires `Gemma Reviewer` / D14 review
before citing unit coverage certification or owner final verification.

## Required Task Presentation Format

Before execution, present:

1. `Task ID` and `Task title`
2. `Status`
3. `Effort`
4. `Complexity`
5. `Recommended model`
   - one recommendation for Codex
   - one recommendation for Claude Code
6. `Objective`
7. `Context`
   - why this task exists
   - what stage or plan it belongs to
8. `Related documents`
   - source task file
   - linked plan file
   - any policies, ADRs, prompts or configs that materially govern the task
9. `Inputs`
10. `Outputs`
11. `Acceptance criteria`
12. `Evidence / metrics to emit`
   - required when the task is expected to produce benchmarks, evaluation data,
     review artifacts, screenshots, audit records, or reportable measurements
13. `Status artifacts to sync`
   - required when the task can change a ledger, report, plan, ADR status, or
     downstream blocker/promotion state
14. `Execution summary`
   - short description of what will be done
   - if applicable, list the ordered steps
15. any task-type-specific pre-task sections required by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
   - for development tasks, include `Happy paths considered`
   - for development tasks, include `Edge cases considered`
16. `Pseudocode`, only if it materially clarifies non-trivial logic
17. `Diagram`
   - required for development tasks
   - for non-development tasks, include only if structure, flow or boundaries are easier to understand visually
18. explicit statement that execution has not started yet and is waiting for approval, when approval is required

When the workflow guide requires `Evidence to emit` or `Status artifacts to sync`,
those items are part of the task's execution contract, not optional closure notes.
They should make it obvious, before implementation starts, which metrics/reports
will be updated during the task and which status-bearing docs must be kept in sync.

## Complexity And Model Guidance

**When RRI has been computed**, the `Complexity` field in the task presentation must
use the RRI band name — not the Effort-based mapping below:

| RRI range | Complexity to present |
|---|---|
| 0–25 | Low |
| 26–40 | Moderate |
| 41–55 | Med-high |
| 56–70 | Complex |

The Effort → Complexity mapping is a **fallback** used only when no RRI is available:

- `Effort: S` -> `Complexity: Low`
- `Effort: M` -> `Complexity: Medium`
- `Effort: L` -> `Complexity: High`

Default recommended models:

- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4`

Escalation guidance:

- use `Claude Opus 4.1` only when the task is long-context heavy, synthesis-heavy, or repeatedly stalls under Sonnet 4
- if a task is primarily code editing, repo navigation, shell execution or deterministic implementation work, keep Codex as the default

If a task file already defines explicit complexity or model guidance, that task-local guidance overrides this file.

## Pseudocode Rule

Include pseudocode only when at least one is true:

- the task has branching logic
- the task transforms data across multiple stages
- the task defines a reusable workflow that benefits from an execution sketch
- the implementation risk is easier to evaluate through a structured outline

Do not add pseudocode for trivial file creation, simple edits, or direct command execution.

## Diagram Rule

For development tasks, always include a compact Mermaid diagram in the task presentation.
The goal is conceptual clarity before approval: show the relevant flow, boundary,
dependency direction, state transition, or ownership split even when the system
architecture itself is unchanged.

For non-development tasks, include a diagram only when at least one is true:

- the task changes architecture boundaries
- the task spans multiple services, crates, workers or repositories
- the task introduces a pipeline, state machine or dependency flow
- the task is easier to approve when shown as a compact flow

Do not add diagrams for simple documentation-only tasks unless the document itself is about architecture.

## Related Documents Rule

The agent must list only the documents that materially constrain the task. Avoid dumping broad reading lists when only a few files are directly relevant.

Priority order:

1. task file
2. linked plan
3. workflow/policy files
4. ADRs
5. prompt files
6. configs/templates

For mobile UI / presentation tasks under `mobile/`, include root `DESIGN.md` in
`Related documents` when it materially constrains the visual work. Treat it as the
mobile design-intent contract, while plan/task files remain authoritative for
behavior, acceptance criteria, and verification.

## Approval Boundary

If the current workflow says the agent must wait for approval before executing a task, the presentation must end with a direct approval checkpoint.

Recommended wording:

`Execution has not started. Approve this task to proceed.`

If no approval is required under the active workflow, the agent may continue under
the gate defined by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and
`docs/policies/RRI_POLICY.md`.

Under the canonical RRI mapping in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and
`docs/policies/RRI_POLICY.md`, `Effort: S` normally corresponds to the **RRI 0–25**
Low band. Those tasks skip the full approval presentation; use local Gemma
delegation through Ollama only for eligible simple code patches, and otherwise
handle them directly as the primary agent while still following the low-band gate.

## Band-routed peer review report lines

Every task card must include a phase-1 line, and every development closure report
must include a phase-2 line. The reviewer token is resolved by RRI band at report
time. Docs-only, config-only, migration-only, ADR, plan, task-ledger, and
policy-only tasks record `n/a` with the exemption stated for phase 2.

```
Task-analysis review: <gemma|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
Code-solution review: <gemma|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
```

- `gemma` — RRI 0–40 (both phases).
- `codex | claude` — RRI 41+, resolved from caller identity
  (`claude-code → codex`, `codex → claude`, others → `claude`).
- `d14` — RRI 41+ where the resolved peer CLI was unavailable and D14 handled the review.
- `BLOCKED` — non-pass verdict or peer + D14 both unavailable. Stops presentation
  (phase 1) or closure (phase 2) until revised, user-waived, or reported blocked.

See `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review` for the
full contract.

## Development Closure Rule

For development-task closure, do not describe certification, final verification,
or status flips as the first completion step. First determine whether the task
must pass the mandatory code-solution review gate (Gemma for RRI 0–40; cross-vendor
peer for RRI 41+, with D14 fallback) under `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
and `docs/policies/HITL_AUTONOMY_POLICY.md`, then describe the remaining closure
blocks in order.

## Language

Agent-facing repository instructions must be written in English.

User-facing presentation may be adapted to the user's language, but task metadata, filenames and model identifiers should remain exact.
