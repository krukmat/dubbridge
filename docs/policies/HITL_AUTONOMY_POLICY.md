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
computed RRI is 0–25** (see the local delegation rule below).

## Local delegation (RRI 0–25)

When the computed RRI falls in the **0–25 Low band**, the agent must not present
the full task for human approval. Instead, it delegates the task to the local
Gemma model through Ollama and remains the reviewer/orchestrator of record.
Gemma must not evaluate, approve, or mark its own delegated work as complete.
Only the delegating agent may decide whether the task satisfies the requirements.

The delegating agent must:

1. Compute RRI with `scripts/rri.py`.
2. Build a local delegation packet with the task excerpt, acceptance criteria, RRI
   output, allowed paths, relevant file snippets, and stop conditions.
3. Send the packet to Ollama/Gemma with `scripts/delegate-low-rri.py`, which uses
   the 120-second timeout and structured-output protocol defined in
   `docs/policies/RRI_POLICY.md`; require structured JSON with a unified diff.
4. Validate the JSON, check the diff with `git apply --check`, and reject any patch
   outside the allowed task scope.
5. Apply only a valid in-scope patch.
6. Personally review the solution against every task requirement and acceptance
   criterion; this evaluation must be performed by the delegating agent, not Gemma.
7. Recompute/check actual touched scope; if the result now scores above RRI 25 or
   triggers a higher gate, stop and escalate to the normal approval workflow.
8. Run required verification commands.
9. If requirements are missed or checks fail, run one bounded Gemma repair cycle
   with the failure evidence and the same allowed paths; if it still fails, stop and
   escalate.
10. Report the RRI, Gemma model used, files changed, the delegating agent's
    requirement-review result, verification commands, and whether a repair cycle
    was needed. If delegation times out, report `Gemma timeout after 120s`.

If penalties are present and the final RRI is still ≤ 25, local delegation still
applies; state all active penalties explicitly in the delegation packet and final
report so the score is transparent.

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
