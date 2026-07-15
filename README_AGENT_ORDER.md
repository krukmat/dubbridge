# Agent Orientation Order

> **Status:** Scaffold. This document consolidates rules that already exist in
> `AGENTS.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `CLAUDE.md`, and the user's global `CLAUDE.md`. It was created to
> resolve the dangling reference in `AGENTS.md`. Expand as the process matures; do
> not let it contradict the workflow guide, `AGENTS.md`, or `CLAUDE.md`.

This file defines the order in which an agent should orient itself before acting in
the `dubbridge` repository.

## Read order (highest authority first)

1. **`CLAUDE.md`** (project) and the user's global `CLAUDE.md` — behavioral rules
   not overridden by the workflow guide.
2. **`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`** — the highest-authority workflow
   source for task flow, gating, presentation requirements, completion, and
   status-artifact synchronization.
3. **`AGENTS.md`** — the task-presentation contract shared by Codex and Claude Code;
   read it as a summary that must stay consistent with the workflow guide.
4. **`docs/policies/HITL_AUTONOMY_POLICY.md`** — when approval is required and what
   autonomy is permitted.
5. **`docs/adr/`** — architecture decisions that constrain implementation.
6. **`docs/plan/` and `docs/tasks/`** — the active slice's plan and task ledger.

## Operating order for a task

1. Analyze context, dependencies, and affected files.
2. Ensure a `docs/plan/<name>.md` and `docs/tasks/<name>.md` exist (create if not).
3. Compute RRI with `scripts/rri.py`; present the next task using the `AGENTS.md`
   presentation contract only when approval is required.
4. If the computed RRI requires approval, wait for explicit approval (see the HITL
   policy). If the task is in the RRI 0–25 Low band (normally `Effort: S` under the
   canonical mapping), use local Gemma delegation through Ollama only for eligible
   simple code patches; otherwise handle the task directly as the primary agent.
5. Implement one task at a time, in order.
6. Before implementation, identify any evidence/metrics the task must emit and any
   status artifacts it must synchronize as part of execution.
7. Verify (tests/checks), emit the named evidence, sync the named status artifacts,
   mark progress in the tasks document, report, and wait.

## Related

- `AGENTS.md`, `CLAUDE.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
