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

`CLAUDE.md` (project and the user's global) is authoritative on conflict.

## Task Presentation Rule

When a user asks an agent to execute a staged task or a task from a task file, the agent must present the next task before execution when the active workflow requires approval.

The presentation must be concise but operationally complete.

Before presenting or executing any staged task, the agent must verify the
current requirements in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`. This file is a
presentation contract summary, not a replacement for the workflow guide.
Task-type-specific requirements defined there are mandatory even when they are
not restated verbatim below.

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
12. `Execution summary`
   - short description of what will be done
   - if applicable, list the ordered steps
13. any task-type-specific pre-task sections required by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
   - for development tasks, include `Happy paths considered`
   - for development tasks, include `Edge cases considered`
14. `Pseudocode`, only if it materially clarifies non-trivial logic
15. `Diagram`
   - required for development tasks
   - for non-development tasks, include only if structure, flow or boundaries are easier to understand visually
16. explicit statement that execution has not started yet and is waiting for approval, when approval is required

## Complexity And Model Guidance

Default mapping:

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

## Approval Boundary

If the current workflow says the agent must wait for approval before executing a task, the presentation must end with a direct approval checkpoint.

Recommended wording:

`Execution has not started. Approve this task to proceed.`

If no approval is required under the active workflow, the agent may still present the task briefly and continue autonomously.

## Language

Agent-facing repository instructions must be written in English.

User-facing presentation may be adapted to the user's language, but task metadata, filenames and model identifiers should remain exact.
