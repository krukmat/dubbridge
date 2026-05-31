# Agent Orientation Order

> **Status:** Scaffold. This document consolidates rules that already exist in
> `AGENTS.md`, `CLAUDE.md`, and the user's global `CLAUDE.md`. It was created to
> resolve the dangling reference in `AGENTS.md`. Expand as the process matures; do
> not let it contradict `AGENTS.md` or `CLAUDE.md`.

This file defines the order in which an agent should orient itself before acting in
the `dubbridge` repository.

## Read order (highest authority first)

1. **`CLAUDE.md`** (project) and the user's global `CLAUDE.md` — behavioral and
   workflow rules. These override default behavior.
2. **`AGENTS.md`** — the task-presentation contract shared by Codex and Claude Code.
3. **`docs/policies/HITL_AUTONOMY_POLICY.md`** — when approval is required and what
   autonomy is permitted.
4. **`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`** — the mandatory plan → tasks →
   approval → implement workflow.
5. **`docs/adr/`** — architecture decisions that constrain implementation.
6. **`docs/plan/` and `docs/tasks/`** — the active slice's plan and task ledger.

## Operating order for a task

1. Analyze context, dependencies, and affected files.
2. Ensure a `docs/plan/<name>.md` and `docs/tasks/<name>.md` exist (create if not).
3. Present the next task using the `AGENTS.md` presentation contract.
4. Wait for explicit approval (see the HITL policy).
5. Implement one task at a time, in order.
6. Verify (tests/checks), mark progress in the tasks document, report, and wait.

## Related

- `AGENTS.md`, `CLAUDE.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
