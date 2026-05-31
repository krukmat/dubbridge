# CLAUDE.md

## Purpose

This file defines how Claude Code should present staged tasks in the `dubbridge` repository.

It is intentionally aligned with `AGENTS.md` so Codex and Claude Code follow the same task-presentation contract.

## Canonical Agent Guides

These documents are the authoritative guides for how agents plan and implement work
in this repository. Read them in this order before acting on any task:

1. `README_AGENT_ORDER.md` — orientation and reading order for agents.
2. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — the mandatory workflow
   (analyze → plan → tasks → approval → implement → mark progress).
3. `docs/policies/HITL_AUTONOMY_POLICY.md` — when explicit human approval is
   required and what autonomy is permitted.
4. `AGENTS.md` — the shared task-presentation contract.
5. `docs/adr/` — architecture decisions that constrain implementation.
6. `docs/plan/roadmap.md` — the general plan: slice sequence, dependencies, and
   where each slice/task sits. Read it to locate any task before implementing.

On conflict, this `CLAUDE.md` and the user's global `CLAUDE.md` take precedence over
the guides above.

## Task Presentation Contract

Before executing a task that belongs to a staged plan or task list, present the task first when the workflow or the user requires approval.

Use this structure:

1. `Task ID` and `Task title`
2. `Status`
3. `Effort`
4. `Complexity`
5. `Recommended model`
   - Codex recommendation
   - Claude Code recommendation
6. `Objective`
7. `Context`
8. `Related documents`
9. `Inputs`
10. `Outputs`
11. `Acceptance criteria`
12. `Execution summary`
13. `Pseudocode` if applicable
14. `Diagram` if applicable
15. explicit approval wait-state when required

## Complexity And Model Defaults

Default mapping:

- `Effort: S` -> `Complexity: Low`
- `Effort: M` -> `Complexity: Medium`
- `Effort: L` -> `Complexity: High`

Default model recommendations:

- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4`

Escalate Claude Code to `Claude Opus 4.1` only for heavy synthesis, long-context comparison, or repeated failure under Sonnet 4.

If the task file defines explicit complexity or model guidance, follow the task file.

## Context Rule

The context section must explain:

- why the task exists
- where it sits in the current stage or plan
- what it unlocks next

Keep it brief and decision-oriented.

## Related Documents Rule

List only the documents that directly govern the task.

Typical sources:

- task file
- linked plan
- workflow guides
- autonomy or policy files
- ADRs
- prompt files
- configs or templates

## Pseudocode Rule

Add pseudocode only when it improves approval quality for non-trivial logic, transformations, workflows or decision trees.

Skip pseudocode for straightforward document creation, direct shell operations or single-file edits.

## Diagram Rule

Add a Mermaid diagram only when boundaries, flows or architecture are materially easier to evaluate visually.

Skip diagrams for simple documentation tasks unless the subject itself is architectural.

## Approval Line

When approval is required, end with:

`Execution has not started. Approve this task to proceed.`

## Language

Repository instruction files are written in English.

User-facing explanations may be localized, but task metadata and file references should remain precise.
