# Human-in-the-Loop (HITL) Autonomy Policy

> **Status:** Scaffold. This policy consolidates the approval and autonomy rules
> already stated in the project and global `CLAUDE.md` and in `AGENTS.md`. It exists
> to resolve the dangling reference in `AGENTS.md`. `CLAUDE.md` is authoritative on
> conflict.

## Principle

The agent plans and proposes; a human approves before implementation. The platform
processes authorized media and enforces fail-closed governance (see
`docs/adr/ADR-008-...md`), so irreversible or outward-facing actions require explicit
human sign-off.

## Always requires explicit approval

- Starting any implementation task with **RRI > 25**, even if a plan was approved
  in a prior session. Approval does not carry across sessions or across tasks.
- Deleting or overwriting files or data.
- Committing, pushing, or any outward-facing action (PRs, external calls).
- Schema migrations and changes to governance-critical invariants.

The only exception to the approval gate is when the user explicitly says "proceed
without asking" (or equivalent) for a clearly bounded scope, **or when the
computed RRI is 0–25** (see show-and-proceed rule below).

## Auto-execute (RRI 0–25)

When the computed RRI falls in the **0–25 Low band**, the agent must:

1. Present the full RRI breakdown table.
2. State the band, tier, and a one-line summary of what will be done.
3. Begin implementation immediately — no approval checkpoint, no pause.
4. Note explicitly: `RRI 0–25 — auto-executing.`

If penalties are present and the final RRI ≤ 25, auto-execute still applies;
state all active penalties explicitly so the score is transparent.

## Approval checkpoint wording

When approval is required (RRI > 25), end the presentation with:

`Execution has not started. Approve this task to proceed.`

## Permitted without prior approval

- Read-only analysis, search, and codebase navigation.
- Drafting plans, task lists, ADRs, and proposals (no code execution).
- Non-destructive fixes to documentation and configuration when explicitly
  authorized to "fix inconsistencies".

## Safety rules

- Do not commit with broken tests; run all tests before commit/push.
- Ask before deleting; surface contradictions instead of proceeding.
- Redact secrets/credentials in logs and traces.
- Report outcomes faithfully: failing tests, skipped steps, and assumptions must be
  stated plainly.

## Related

- `CLAUDE.md`, `AGENTS.md`, `README_AGENT_ORDER.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md`
