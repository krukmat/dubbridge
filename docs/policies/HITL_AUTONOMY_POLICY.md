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

The Local Architect / Complex Analyst role (ADR-037) produces advisory
analysis only; it never evaluates, approves, or satisfies any human-approval
gate on its own behalf or on behalf of the task it informed — see
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local Architect / Complex Analyst
(ADR-037)`.

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

## Local-first implementation (RRI 26–40 Moderate)

For the **26–40 Moderate** band, the approval gate is unchanged: the agent
must present the task and wait for explicit human approval before
implementation. What changes is the default implementation route.

The **41–55 Med-high** band does **not** use this direct local-first route —
see § Med-high Architect-refined single-attempt gate (RRI 41–55) below for
its own routing.

The default path for Moderate development tasks is:

1. Compute RRI with `scripts/rri.py`.
2. Present the task and obtain explicit approval.
3. Run the implementation through `scripts/local-agent/run_local_task.py` in a
   disposable git worktree, resolving the implementer from
   `DUBBRIDGE_LOCAL_AGENT_MODEL` (default `qwen3.6:35b-a3b`) and the endpoint
   from `OLLAMA_HOST`.
4. Keep the primary agent as orchestrator of record: it owns the task card,
   allowed paths, acceptance tests, reflection passes, closure, and all final
   judgments about correctness.
5. The runner uses a simple tool contract — `read_file` (whole file),
   `write_file` (create or overwrite), `apply_patch` (single-unique-anchor
   replacement), `run_command`, `finish`. There is no language-server preflight;
   the implementer reads the file it changes directly. (See
   `docs/plan/local-agent-simple-editing.md` for why the earlier Serena path
   was removed.)
6. Enforce the task's `allowed_paths` after the local run. Any out-of-scope
   diff fails closed and is never accepted into the primary checkout.
7. Run the approved verification commands and the organization gate before
   issuing a signed success audit. The `local-implementer` signature is valid
   only when scope, acceptance, and organization gates all pass.
8. If the local run fails the acceptance signal, hits the scope boundary, or
   the local path is unavailable, the primary agent may run at most **2**
   evidence-backed local repair attempts.
9. After the repair budget is exhausted, or if the local runner/model is
   unavailable, escalate to cloud implementation with the ADR-036 escalation
   packet rather than continuing with ad hoc local retries.

This routing is operative by owner override dated **2026-07-15**. It was
adopted ahead of the original ADR-036 promotion gate so that live
Moderate-band tasks become the evaluation surface.

It was extended to the Med-high band by owner override dated **2026-07-21**
with a tighter 1-attempt repair budget, and subsequently **replaced** for
Med-high by ADR-038 (2026-07-26) — see § Med-high Architect-refined
single-attempt gate (RRI 41–55) below. Med-high no longer uses any repair
attempt at all; the historical 1-attempt figure no longer applies to this
band.

## Med-high Architect-refined single-attempt gate (RRI 41–55)

ADR-038 (2026-07-26) governs implementation routing for final **RRI 41–55**.
The approval gate is unchanged: the agent must present the task and wait for
explicit human approval before implementation. Band-resolved independent
review (phases 1 and 2), 3 Reflection passes, and the "Plan + explicit
acceptance criteria" gate all still apply.

The route:

1. Compute RRI with `scripts/rri.py`; confirm it falls in 41–55.
2. Present the task and obtain explicit approval.
3. Request a Qwen27 (`qwen3.6:27b-q4_K_M`) advisory refinement via
   `scripts/local-architect/run_analysis.py`'s `med-high-refinement-v1`
   profile. It returns `route_recommendation: GO_LOCAL | CLOUD_REQUIRED`
   bound to the task capsule hash and its own model tag/digest.
4. The primary agent issues its own hash-bound route receipt, evaluated by
   `scripts/local-agent/med_high_gate.py`. The primary may **downgrade**
   GO_LOCAL to cloud; it may **never upgrade** CLOUD_REQUIRED to local — this
   is enforced structurally (the gate requires both sides to independently
   say GO_LOCAL), not by trusting either decision alone.
5. If the gate resolves GO_LOCAL: `scripts/local-agent/run_med_high_task.py`
   supervises exactly **one** session on the exact `qwen3.6:35b-a3b` binding,
   as its own OS process group, bounded to **8 turns**, **300 seconds**
   wall clock, and **0 repair attempts**. No silent model substitution. A
   timeout kills the full process group (not just the immediate PID) and
   preserves the last checkpoint and partial diff.
6. If the gate resolves CLOUD_REQUIRED, or the one local attempt does not
   reach success (timeout, failing acceptance, scope/boundary/organization
   violation, or model substitution), escalate to Codex or Claude with the
   full ADR-038 §5 evidence bundle: task capsule, refinement artifact,
   primary receipt, effective limits, transcript/checkpoint, partial diff,
   commands/tests run, stop reason, hashes, model identity, elapsed time.
7. Run the approved verification commands and the organization gate before
   issuing a signed success audit, exactly as for Moderate. The
   `local-implementer` signature is valid only when scope, acceptance, and
   organization gates all pass.

Hard exclusions from GO_LOCAL regardless of the Qwen27 recommendation:
auth/security work, rights/consent/governance invariants, schema/migrations/
release cuts, unresolved ADR decisions, and unbounded scope — see ADR-038 §6.

This gate does not weaken the independent review route defined by the
"Band-routed peer review" section below.

## Approval checkpoint wording

When approval is required (RRI > 25), end the presentation with:

`Execution has not started. Approve this task to proceed.`

Use the Compact Approval Task Card v2 from the workflow guide. A user may waive
this checkpoint only by explicitly authorizing execution without another
approval for a clearly bounded task; record that waiver in the card or ledger.

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

- **RRI 0–25 (Low):** Gemma (phases 1 and 2). Phase-2 = existing Gemma
  Reviewer N-pass; phase-1 = advisory Gemma review of the task card.
- **RRI 26–55 (Moderate + Med-high):** `qwen3.6:27b-q4_K_M` (phases 1 and 2,
  owner directive 2026-07-21) — see `docs/policies/RRI_POLICY.md § Local
  pipeline phase-2 reviewer override`. Replaces Gemma (Moderate) and the
  cross-vendor peer (Med-high) as the default reviewer for this band,
  regardless of whether implementation stayed local or escalated to cloud.
- **RRI 56+ (Complex+):** cross-vendor peer (phases 1 and 2). The peer
  replaces Gemma as the code-solution reviewer for this band.

**Cross-vendor resolution (RRI 56+ only):**
`claude-code → codex | codex → claude | local-provider → claude |
remote-provider → claude | unknown → claude`

**Failure modes (RRI 26–55):**
1. `qwen3.6:27b-q4_K_M` unavailable, stalled, or returns invalid/`BLOCKED`
   output → fall back to **Gemma** (one immediate retry with the same review
   packet if Gemma itself is unusable on the first attempt).
2. `qwen3.6:27b-q4_K_M` + Gemma both unavailable/unusable → fall back to
   **D14** (Balanced tier).
3. `qwen3.6:27b-q4_K_M` + Gemma + D14 all unavailable → write a
   blocked-artifact record and stop. Never self-review. Report the task as
   blocked.

**Failure modes (RRI 56+):**
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

The review step is **mandatory** for all Low (0–25) development tasks. Gemma
is the preferred reviewer; the context-isolated subagent (D14,
`scripts/adjudicator-packet.py`) is the required fallback. For Moderate and
Med-high (26–55), the equivalent mandatory review step uses
`qwen3.6:27b-q4_K_M` in place of Gemma as the primary reviewer — see §
Band-routed peer review above — with Gemma inserted as the intermediate
fallback (owner directive 2026-07-21) before D14: `qwen3.6:27b-q4_K_M` → Gemma
→ D14. The retry-then-escalate discipline described below applies at each
step of that chain.

When Ollama is unavailable, the model is absent, the reviewer stalls, output
is invalid, the review result is `BLOCKED`, or no usable consolidated review
result can be produced, the agent must perform **one immediate retry** with
the same review packet first. If the retry succeeds with a usable result, the
primary path continues normally. If the retry fails for the same class of
reason or still produces no usable result, the agent **must** spawn a
context-isolated subagent as the mandatory fallback reviewer. The subagent
receives an isolation packet (diff + acceptance criteria + any usable partial
findings) and its output is advisory, exactly as the primary reviewer's would
be. The primary agent reconciles and records `disposition_divergence` in the
audit log.

Reviewer unavailability or unusable local review output does not open a human
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

## Review evidence override (urgency, human-only)

Owner directive, 2026-07-22 (GEG-1): a completed development section may
close without a `Review artifact:` receipt only via a typed
`REVIEW-OVERRIDE:` line (see `docs/policies/RRI_POLICY.md § Review evidence
gate` and `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Review artifact receipt
and REVIEW-OVERRIDE lines`). Two of the three override types —
`pipeline-failure` and `not-applicable` — are ordinary agent-supplied
evidence, no different in kind from the other autonomous escapes in this
policy. The third, **`urgency`**, is not: it requires a `Waiver-by: <human
name>` companion field naming the person who authorized skipping review, and
an agent **may not self-issue it**. An agent invoking `urgency` without a
prior, explicit human waiver is out of scope of the autonomy this policy
grants — treat it the same as any other unauthorized skip of a mandatory
review gate.

## Related

- `CLAUDE.md`, `AGENTS.md`, `README_AGENT_ORDER.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md`
