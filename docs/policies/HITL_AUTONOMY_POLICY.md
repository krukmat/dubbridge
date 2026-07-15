---
type: Policy
title: "Human-in-the-Loop (HITL) Autonomy Policy"
governs: "when explicit human approval is required and what autonomy is permitted"
---

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
without asking" (or equivalent) for a clearly bounded scope, or when the
computed RRI is 0–25 and the task stays within the low-band handling rules below.

## Local delegation (RRI 0–25)

When the computed RRI falls in the **0–25 Low band**, the agent must not present
the full task for human approval. The default low-band path is **direct execution
by the primary agent**. Local Gemma delegation through Ollama is reserved only for
**simple code patching**: narrow, mechanical code or test edits with a small
allowed path set and low editorial risk. Docs, plans, task ledgers, ADRs,
policies, workflow scripts, and other structure-heavy or interpretation-heavy work
must stay with the primary agent even when the RRI is Low.

When Gemma delegation is used, Gemma must not evaluate, approve, or mark its own
delegated work as complete. Only the delegating agent may decide whether the task
satisfies the requirements.

For eligible simple code patches, the delegating agent must:

1. Compute RRI with `scripts/rri.py`.
2. Build a local delegation packet with the task excerpt, acceptance criteria, RRI
   output, allowed paths, relevant file snippets, and stop conditions.
3. Send the packet to Ollama/Gemma with `scripts/delegate-low-rri.py`, which uses
   the 120-second timeout and tagged-block response protocol defined in
   `docs/policies/RRI_POLICY.md`; require complete file contents, not JSON and not
   a unified diff.
4. Validate the tagged response, check the wrapper-built diff with
   `git apply --check`, and reject any patch outside the allowed task scope.
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

If penalties are present and the final RRI is still ≤ 25, the low-band handling
still applies. When delegation is used, state all active penalties explicitly in
the delegation packet and final report so the score is transparent.

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

## Band-routed peer review

Every development task is reviewed by an independent reviewer at two phases.
The reviewer is determined by the task's RRI band:

- **RRI 0–40 (Low + Moderate):** Gemma (phases 1 and 2). Phase-2 = existing
  Gemma Reviewer N-pass; phase-1 = advisory Gemma review of the task card.
- **RRI 41+ (Med-high + Complex):** cross-vendor peer (phases 1 and 2). The
  peer replaces Gemma as the code-solution reviewer for this band.

**Cross-vendor resolution (RRI 41+ only):**
`claude-code → codex | codex → claude | local-provider → claude |
remote-provider → claude | unknown → claude`

**Failure modes (RRI 41+):**
1. Peer CLI unavailable or unauthenticated → fall back to **D14** (Balanced tier).
2. Peer + D14 both unavailable → write a blocked-artifact record and stop. Never
   self-review. Report the task as blocked.

Peer review **does not replace** the human approval gate required by the RRI band
(HITL). It is a separate, independent check — the human approval gate still fires
for every RRI 26+ task after the peer review passes.

Phase-1 (task-analysis) exemptions: docs-only, config-only, migration-only, ADR,
plan, task-ledger, and policy-only tasks record `Task-analysis review: n/a`.

See `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review` for the
full routing table, report line contract, and enforcement note.

## Gemma Reviewer availability

The review step is **mandatory** for all Low (0–25) and Moderate (26–40)
development tasks. Gemma is the preferred reviewer; the context-isolated subagent
(D14, `scripts/adjudicator-packet.py`) is the required fallback.

When Ollama is unavailable, the model is absent, Gemma stalls, output is invalid,
the review result is `BLOCKED`, or no usable consolidated review result can be
produced, the agent must perform **one immediate retry** with the same review
packet first. If the retry succeeds with a usable Gemma result, the Gemma path
continues normally. If the retry fails for the same class of reason or still
produces no usable result, the agent **must** spawn a context-isolated subagent
as the mandatory fallback reviewer. The subagent receives an isolation packet
(diff + acceptance criteria + any usable partial findings) and its output is
advisory, exactly as Gemma's would be. The primary agent reconciles and records
`disposition_divergence` in the audit log.

Gemma unavailability or unusable local review output does not open a human
approval gate beyond what the RRI band already requires. The review is never
skipped.

See `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer` for the full
authority boundary, trigger conditions, and evidence format.

## Reviewability budget escape

The reviewability budget gate (`make qa-review-budget`) fails closed when a
change is too large for Gemma to evaluate in-context. Staying inside the budget
by splitting the change is the default and requires no approval. When a change is
genuinely irreducible, the delivering agent may **autonomously** take the
documented escape — a `D14-OVERRIDE: <reason>` line in the commit body or task
entry — which routes the change to the non-Gemma (D14) reviewer instead. This
escape does **not** open a human approval gate and does **not** skip review: the
D14 reviewer still runs and `disposition_divergence` is still recorded. Using the
escape to avoid review, or recording it without a genuine reason, is a policy
violation.

See `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Reviewability budget gate` for the
budget derivation and override mechanics.

## Related

- `CLAUDE.md`, `AGENTS.md`, `README_AGENT_ORDER.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md`
