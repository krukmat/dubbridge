---
type: Plan
title: "Task-aware agent token efficiency"
status: closed
---

# Task-aware agent token efficiency

## Objective

Make productive token use a standing instruction for Codex and Claude Code:
favor deterministic automation and eligible local AI, reserve cloud work for
where it adds value, and seek concrete human assistance when it can reduce
elevated consumption without weakening quality or existing gates.

## Scope and decisions

- Keep the operative procedure in
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`; add compact reminders in
  `AGENTS.md` and `CLAUDE.md`, then regenerate `AGENTS.override.md`.
- Adapt resource use to the task's scope, RRI, evidence, and host capacity.
  Preserve model bindings, review independence, retry budgets, local prechecks,
  and existing approval/fallback requirements.
- Use existing task records and telemetry; do not introduce a quota, billing
  monitor, new approval gate, executable change, or global user configuration.
- Keep context and delegation proportional. Human assistance is a specific
  request to resolve uncertainty or provide unavailable evidence, not a routine
  transfer of agent work to the user.

## Dependencies and verification

No product/runtime dependency changes. Governing constraints:
`docs/policies/RRI_POLICY.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`, and
the workflow guide. The existing generator and documentation QA verify
bootstrap synchronization and documentation consistency.

## Execution and status

One coherent docs/policy task, `ATE-T1`, in
`docs/tasks/agent-token-efficiency.md`. No useful independent implementation
split; the parent and sole leaf share RRI 25 (Low), including the process
decision penalty. Product roadmap, ADR status, and existing slice ledgers are
unaffected. `ATE-T1` completed on 2026-09-22; canonical instructions, both
entry-point summaries, and the generated bootstrap are synchronized.
Verification evidence is recorded in the task ledger.
