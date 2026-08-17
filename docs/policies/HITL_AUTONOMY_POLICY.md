---
type: Policy
title: "Human-in-the-Loop (HITL) Autonomy Policy"
governs: "when explicit human approval is required and what autonomy is permitted"
---

# Human-in-the-Loop (HITL) Autonomy Policy

> **Status:** Scaffold. This policy states the approval/autonomy boundary.
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is highest authority and owns the
> full procedure for every route named below; this file states *when approval
> is required* and *what autonomy is granted*, and points to the guide for
> *how* each route executes. `CLAUDE.md` is authoritative on conflict.

## Principle

The agent plans and proposes; a human approves before implementation. The platform
processes authorized media and enforces fail-closed governance (see
`docs/adr/ADR-008-...md`), so irreversible or outward-facing actions require explicit
human sign-off.

Two advisory-only roles never satisfy this gate on their own behalf: the
Local Architect / Complex Analyst (ADR-037) and the Antares
Security-Specialist Advisor. Both may inform a decision; neither approves,
blocks, computes RRI, or replaces the band-routed review chain. See
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local Architect / Complex Analyst
(ADR-037)` and `§ Antares Security-Specialist Advisor` for their full
authority boundary.

## Always requires explicit approval

- Starting any implementation task with **RRI > 25**, even if a plan was approved
  in a prior session. Approval does not carry across sessions or across tasks.
- Deleting or overwriting files or data.
- Committing, pushing, or any outward-facing action (PRs, external calls).
- Schema migrations and changes to governance-critical invariants.

The only exception to the approval gate is when the user explicitly says "proceed
without asking" (or equivalent) for a clearly bounded scope, or when the
computed RRI is 0–25 and the task stays within the low-band handling rules below.

## Per-task local-stack restart

Every task that will invoke an Ollama-backed local role must restart Ollama
once before its first local-model request, even when the current server
appears healthy, and pass the warm-up probe for every model that task's band
will use. A new repository task ID creates a new restart boundary; retries,
repairs, and later local phases of the same task reuse the restarted server
unless it becomes unavailable or wedged. This is an operational precondition
only — it does not waive or replace HITL approval, independent review,
fallback selection, or any RRI gate. Full sequence, PID/listener checks, and
the resource-recovery protocol for empty-content responses:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before
implementing`, Step 0.

## Local delegation (RRI 0–25)

When the computed RRI falls in the **0–25 Low band**, the agent must not present
the full task for human approval. The default low-band path is **direct execution
by the primary agent**. Local Qwen Developer delegation through Ollama is reserved only for
**simple code patching**: narrow, mechanical code or test edits with a small
allowed path set and low editorial risk. Docs, plans, task ledgers, ADRs,
policies, workflow scripts, and other structure-heavy or interpretation-heavy work
must stay with the primary agent even when the RRI is Low.

The current Low/S developer binding, the full 10-step delegation procedure
(packet construction, `scripts/delegate-low-rri.py` invocation, validation,
personal review, repair-cycle rule, and reporting format), and the tagged-block
response contract are defined in `docs/policies/RRI_POLICY.md § Low RRI
handling` and `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`. This section's
scope is only the approval boundary: Qwen Developer delegation never
evaluates, approves, or marks its own delegated work complete — only the
delegating agent decides whether the task satisfies requirements — and a
missed requirement or failed check permits at most one bounded repair cycle
before escalating to the normal approval workflow.

If penalties are present and the final RRI is still ≤ 25, the low-band handling
still applies. When delegation is used, state all active penalties explicitly in
the delegation packet and final report so the score is transparent.

## Local-first implementation (RRI 26–40 Moderate)

For the **26–40 Moderate** band, the approval gate is the standard one: the
agent must present the task and wait for explicit human approval before
implementation. The band's exception is only its default implementation route.

The **41–55 Med-high** band does **not** use this direct local-first route —
see § Med-high Architect-refined single-attempt gate (RRI 41–55) below for
its own routing.

The default path for Moderate development tasks runs the implementation
through `scripts/local-agent/run_local_task.py` in a disposable git
worktree, with the primary agent as orchestrator of record and a maximum of
**2** evidence-backed local repair attempts before escalating to the
cloud-takeover model recorded in the approved task card. Full route (tool
contract, scope enforcement, DEV success-audit signature, escalation
trigger) and rollback triggers:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
implementation routing (RRI 26–55)` and `§ Handoff prompt format`.

Med-high uses no whole-task repair attempt, **except** for a
module independently qualified under ADR-040 per-module split routing (§
Per-module complexity-split routing (RRI 26–55) below), whose local tramo
uses this Moderate section's 2-attempt budget regardless of the containing
task's band.

## Post-repair-budget Low-band decomposition

Once the whole-task local-agent route above
exhausts its repair budget (2/2 for Moderate; the ADR-038 gate's `GO_LOCAL`
exhausted, or a module's local tramo exhausted, for Med-high), cloud
escalation is **not** the default next step. The default is to
**decompose the remaining work into Low-band (RRI 0–25) subtasks and keep it
local**, maximizing local-model usage; the primary agent's role becomes
orchestration — diagnosing, splitting, dispatching, reviewing, and
assembling — not authoring code directly, even for small, fully-diagnosed
mechanical fixes. Cloud escalation stays available as the fallback
of last resort for this step, never the default.

The full 9-step route (confirm budget exhaustion, diagnose real signatures,
decompose into scored Low-band subtasks, delegate via
`scripts/delegate-low-rri.py`, review every returned patch, the two narrow
direct-edit exceptions — documented tooling-failure and mechanical
lint-driven refactor — resume normal band-appropriate closure, and record
the `### Implementation routing evidence` block) is defined in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band
decomposition`, validated end-to-end on
`S-150-T2c-iv-c` (see `docs/tasks/s-150-translation-dubbing.md` §
S-150-T2c-iv-c "Implementation routing evidence").

**No additional approval checkpoint per subtask.** Re-confirmation is waived
for this decomposition/dispatch/assembly
loop once the containing task is already HITL-approved and has reached this
post-repair-budget point — a bounded waiver for an already-approved
task's implementation mechanics, not a waiver of the RRI 26+ human approval
gate itself, which still fires once, at task presentation, as normal. This
route changes only how the remaining implementation is produced after
the repair budget is exhausted — never the task's RRI, band, review chain,
Reflection pass count, or approval gate.

## Med-high Architect-refined single-attempt gate (RRI 41–55)

ADR-038 governs implementation routing for final **RRI 41–55**.
The approval gate is the standard one: the agent must present the task and wait
for explicit human approval before implementation. Band-resolved independent
review (phases 1 and 2), 3 Reflection passes, and the "Plan + explicit
acceptance criteria" gate all apply.

Route summary: Muse Glimmer advisory refinement (`GO_LOCAL` |
`CLOUD_REQUIRED`) → primary agent's own hash-bound route receipt (may
downgrade `GO_LOCAL` to cloud; may never upgrade `CLOUD_REQUIRED` to local)
→ every result (including `GO_LOCAL`) escalates to the concrete cloud
takeover model with the full ADR-038 §5 evidence bundle — **except** for a
module independently qualified under ADR-040 per-module split routing (§
Per-module complexity-split routing (RRI 26–55) below). Hard exclusions from
`GO_LOCAL` regardless of the Muse Glimmer recommendation: auth/security
work, rights/consent/governance invariants, schema/migrations/release cuts,
unresolved ADR decisions, and unbounded scope (ADR-038 §6). Full route,
implementation surfaces, and evidence-bundle contents:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
implementation routing (RRI 26–55)` and ADR-038.

This gate does not weaken the independent review route defined by the
"Band-routed peer review" section below.

## Per-module complexity-split routing (RRI 26–55, ADR-040)

Under `ADR-040`: for an **approved** development task with final RRI
26–55 whose `allowed_paths` span two or more files, the orchestrator may
split implementation authorship by per-module cyclomatic complexity instead
of routing the whole task through the sections above as one unit. This is a
routing refinement, not a new approval gate — it fires after HITL approval
and phase-1 review, and never changes the task's RRI, band, phase-1/phase-2
reviewer, Reflection pass count, or closure gates.

Trigger, hard domain exclusion, repair budgets, interface-freeze
requirement, and the mandatory integration gate are defined in full in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Per-module complexity-split
routing (RRI 26–55, ADR-040)` and
`docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`.

## Approval checkpoint wording

When approval is required (RRI > 25), end the presentation with:

`Execution has not started. Approve this task to proceed.`

Use the Compact Approval Task Card v2 from the workflow guide. A user may waive
this checkpoint only by explicitly authorizing execution without another
approval for a clearly bounded task; record that waiver in the card or ledger.

## Fallback model-selection checkpoint

ADR-039 adds a bounded authorization checkpoint only when a terminal local review
or implementation route needs D14 or a cloud implementer. It is not a replacement
for the task's HITL approval and it never broadens the approved scope.

`human-select` is the interactive default: a missing model, reasoning
effort, or human selector returns `awaiting_fallback_selection` and the
process stops without invoking the fallback. `preauthorized` is allowed only
when all three fields were frozen in the approved task card or preflight; an
incomplete preauthorization fails closed. The orchestrator must validate the
authorized receipt and packet digest before invoking exactly the selected
model/effort — a missing, stale, or role-mismatched receipt stays blocked.
Full schema and the frozen recommendation matrix: ADR-039 and
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Human-selected fallback checkpoint
(ADR-039)`.

D14 remains a read-only, context-isolated Balanced-tier reviewer. Selecting it
does not authorize cloud implementation; selecting a cloud implementer does not
waive independent review, RRI gates, repair budgets, or scope checks.

## Permitted without prior approval

- Read-only analysis, search, and codebase navigation.
- Drafting plans, task lists, ADRs, and proposals (no code execution).
- Non-destructive fixes to documentation and configuration when explicitly
  authorized to "fix inconsistencies".
- Creating and updating the live per-task todo list required by
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Live per-task phase todo list`.
  This list is a transparency/tracking artifact only — it never substitutes
  for the HITL approval checkpoint, the band-routed review chain, or any
  other closure gate in this policy; marking an entry `completed` does not
  by itself certify that the corresponding gate passed.

## Safety rules

- Do not commit with broken tests; run all tests before commit/push.
- Ask before deleting; surface contradictions instead of proceeding.
- Redact secrets/credentials in logs and traces.
- Report outcomes faithfully: failing tests, skipped steps, and assumptions must be
  stated plainly.

## Band-routed peer review

Every development task is reviewed by an independent reviewer at two phases,
resolved from the task's RRI band:

- **RRI 0–25 (Low):** Muse Glimmer primary, Gemma intermediate fallback, D14 final fallback.
- **RRI 26–55 (Moderate + Med-high):** Gemma primary, Muse Glimmer intermediate fallback, D14 final fallback.
- **RRI 56+ (Complex+):** cross-vendor peer (replaces Gemma), D14 fallback.

Both bindings apply regardless of whether implementation stayed local or
escalated to cloud — the binding governs *who reviews*, independently of
*who authored the code*. The full routing table, current model bindings and
their change history, per-band failure-mode sequencing, the cross-vendor and
D14 provider-resolution rules, and the report-line contract are the single
canonical definition in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed
peer review (two phases)` — this policy does not restate them.

Peer review **does not replace** the human approval gate required by the RRI band
(HITL). It is a separate, independent check — the human approval gate still fires
for every RRI 26+ task after the peer review passes.

Phase-1 (task-analysis) exemptions: docs-only, config-only, migration-only, ADR,
plan, task-ledger, and policy-only tasks record `Task-analysis review: n/a`.

## Gemma Reviewer availability

The review step is **mandatory** for every development task regardless of
band; no path may be skipped, and reviewer unavailability never opens a
human approval gate beyond what the RRI band already requires. Full trigger
conditions, the retry-then-escalate discipline, and the current fallback
chains per band are defined once in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer / Muse Glimmer
Reviewer § Availability`.

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

A completed development section may close without a `Review artifact:`
receipt only via a typed `REVIEW-OVERRIDE:` line (see
`docs/policies/RRI_POLICY.md § Review evidence gate` and
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Review artifact receipt and
REVIEW-OVERRIDE lines`). Two of the three override types —
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
- `docs/policies/RRI_POLICY.md`
- `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md`
- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
- `docs/adr/ADR-039-human-selected-fallback-model-checkpoint.md`
- `docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`
