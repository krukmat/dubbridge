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
> *how* each route executes. For every conflict covered by the guide, the guide
> controls; `CLAUDE.md` applies only where the guide is silent.

## Principle

The agent plans and proposes; a human approves before implementation. The
platform processes authorized media and enforces fail-closed governance
(ADR-008), so irreversible or outward-facing actions require explicit human
sign-off.

Two advisory-only roles never satisfy this gate on their own behalf: Local
Architect / Complex Analyst (ADR-037) and the Antares Security-Specialist
Advisor. Both may inform a decision; neither approves, blocks, computes RRI,
or replaces the band-routed review chain. Full authority boundary:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local Architect / Complex Analyst
(ADR-037)` and `§ Antares Security-Specialist Advisor`.

## Always requires explicit approval

- Starting any implementation task with **RRI > 25**, even if a plan was
  approved in a prior session — approval does not carry across sessions or
  tasks.
- Deleting or overwriting files or data.
- Committing, pushing, or any outward-facing action (PRs, external calls).
- Schema migrations and changes to governance-critical invariants.

Exceptions: the user explicitly says "proceed without asking" for a clearly
bounded scope, or the computed RRI is 0–25 within the low-band rules below.

## Per-task local-stack restart

Every task invoking an Ollama-backed local role must restart Ollama once
before its first local-model request, even if the server looks healthy, and
pass the warm-up probe for every model the task's band will use. A new
repository task ID creates a new restart boundary; retries, repairs, and
later local phases of the same task reuse the restarted server unless it
becomes unavailable or wedged. Operational precondition only — it does not
waive HITL approval, independent review, fallback selection, or any RRI
gate. Full sequence and resource-recovery protocol:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before
implementing`, Step 0.

## Local delegation (RRI 0–25)

For the **0–25 Low band**, do not present the full task for approval. The
default path is **direct execution by the primary agent**; local Qwen
Developer delegation through Ollama is reserved for **simple code
patching** only (narrow, mechanical edits, small `allowed_paths`, low
editorial risk) — docs, plans, task ledgers, ADRs, policies, and other
structure-heavy work stay with the primary agent even at Low RRI.

Delegation binding, the full procedure, and the response contract:
`docs/policies/RRI_POLICY.md § Low RRI handling` and
`docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`. This section's scope is
only the approval boundary: Qwen Developer never evaluates or approves its
own delegated work — only the delegating agent decides — and a missed
requirement permits at most one bounded repair cycle before escalating to
normal approval.

Penalties present with a final RRI still ≤ 25 keep the low-band handling;
state active penalties explicitly in the delegation packet and final report.

## Local-first implementation (RRI 26–40 Moderate)

The **26–40 Moderate** approval gate is standard (present and wait for
explicit approval); the band's exception is only its default implementation
route. **Med-high 46–55 does not use this route** — see § Med-high
Architect-refined single-attempt gate below. **Med-high 41–45 does use this
route** on a `GO_LOCAL` result (ADR-038 Amendment 3, 2026-08-23) — its
approval gate, review chain, and Reflection count stay Med-high's own;
only the implementation-authoring mechanics follow this section.

Default Moderate path: `scripts/local-agent/run_local_task.py` in a
disposable git worktree, primary agent as orchestrator of record, at most
**2** evidence-backed local repair attempts. On 2/2 exhaustion, remaining
work is first decomposed into scored Low-band subtasks; cloud takeover from
the approved card is the last resort when that route cannot proceed. An
operational local-runner/model failure or scope-boundary failure follows its
separate approved cloud-takeover condition. Full route and rollback triggers:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
implementation routing (RRI 26–55)` and § Handoff prompt format.

Med-high 46–55 has no whole-task repair attempt, **except** a module
qualified under ADR-040 per-module split routing (below), whose local tramo
uses this section's 2-attempt budget regardless of the containing task's
band. Med-high 41–45 is the exception: a `GO_LOCAL` result gives it the
same whole-task 2-attempt budget as Moderate (ADR-038 Amendment 3).

## Post-repair-budget Low-band decomposition

Once the Moderate whole-task local-agent route exhausts its repair budget
(2/2), the default next step is **not cloud escalation** — it is to
**decompose the remaining work into Low-band (RRI 0–25) subtasks and keep it
local**, with the primary agent as orchestrator only (diagnosing, splitting,
dispatching, reviewing, assembling — never authoring code directly). Cloud
escalation stays the fallback of last resort, not the default.

An ADR-040-qualified local module follows its own two-attempt local budget and
may use this decomposition route for its remaining module work. A Med-high
46–55 whole-task `GO_LOCAL` advisory is policy-excluded from local
implementation; it never creates a whole-task local repair budget. A Med-high
41–45 whole-task `GO_LOCAL` advisory (ADR-038 Amendment 3) is not excluded —
it creates a local repair budget exactly like Moderate, including this
post-repair-budget decomposition step on 2/2 exhaustion.

**46–55 also runs this decomposition step (ADR-038 Amendment 4,
2026-08-30).** Because 46–55 has no whole-task repair budget to exhaust, the
trigger is instead any 46–55 `GO_LOCAL` or `CLOUD_REQUIRED` result: before
emitting the cloud-takeover packet, the orchestrator decomposes the
remaining scope into candidate subtasks, scores each independently with
`scripts/rri.py`, dispatches every RRI 0–25 candidate via
`scripts/delegate-low-rri.py`, and routes only the above-Low residue (or any
subtask touching a § Med-high hard-exclusion surface) to cloud. This does
not reopen a whole-task local attempt in 46–55 and does not weaken Amendment
1 — it only inserts the same Low-band-maximization step Moderate already
uses, applied to 46–55's cloud-only trigger instead of a repair-budget
exhaustion.

Full 9-step route (budget confirmation, diagnosis, decomposition, delegation
via `scripts/delegate-low-rri.py`, patch review, the two narrow direct-edit
exceptions — documented tooling-failure and mechanical lint-driven refactor
— resumed closure, and the `### Implementation routing evidence` block):
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band
decomposition`, validated end-to-end on `S-150-T2c-iv-c` (see
`docs/tasks/s-150-translation-dubbing.md` § S-150-T2c-iv-c).

**No additional approval checkpoint per subtask** — a bounded waiver for an
already-HITL-approved task's implementation mechanics once this point is
reached, not a waiver of the RRI 26+ approval gate itself. Changes only how
the remaining implementation is produced — never RRI, band, review chain,
Reflection count, or approval gate.

## Med-high Architect-refined single-attempt gate (RRI 41–55)

ADR-038 governs routing for final **RRI 41–55**. The approval gate is
standard; band-resolved independent review (phases 1 and 2) and 3 Reflection
passes apply.

Route: Muse Glimmer advisory refinement (`GO_LOCAL`|`CLOUD_REQUIRED`) →
primary agent's hash-bound route receipt (may downgrade, never upgrade). For
**RRI 46–55**, every result (including `GO_LOCAL`) first goes through the
Amendment 4 Low-band decomposition attempt above (§ Post-repair-budget
Low-band decomposition), then escalates any above-Low residue to the cloud
takeover model with the full ADR-038 §5 evidence bundle — **except** a
module qualified under ADR-040 per-module split routing (below). For **RRI
41–45** (ADR-038 Amendment 3, 2026-08-23), a `GO_LOCAL` result instead
routes the whole task through the Moderate local-first path (§ Local-first
implementation above) — `CLOUD_REQUIRED` still escalates to cloud in both
sub-bands. Hard exclusions from `GO_LOCAL` regardless of Muse Glimmer's
recommendation, unchanged for both sub-bands: auth/security, rights/consent/
governance invariants, schema/migrations/release cuts, unresolved ADR
decisions, unbounded scope (ADR-038 §6). Full route, implementation
surfaces, evidence bundle:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
implementation routing (RRI 26–55)` and ADR-038.

This gate does not weaken the independent review route in § Band-routed
peer review below.

## Per-module complexity-split routing (RRI 26–55, ADR-040)

For an **approved** task (26–40 or 41–55) whose `allowed_paths` span two or
more files, the orchestrator may split implementation authorship by
per-module cyclomatic complexity instead of routing the whole task as one
unit — a routing refinement, not a new approval gate: fires after HITL
approval and phase-1 review, and never changes RRI, band, reviewer,
Reflection count, or closure gates.

Trigger, hard domain exclusion, repair budgets, interface-freeze, and the
mandatory integration gate: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
Per-module complexity-split routing (RRI 26–55, ADR-040)` and
`docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`.

## Approval checkpoint wording

When approval is required (RRI > 25), end with:

`Execution has not started. Approve this task to proceed.`

Use the Compact Approval Task Card v2. A user may waive this checkpoint only
by explicitly authorizing execution without another approval for a clearly
bounded task; record the waiver in the card or ledger.

## Fallback model-selection checkpoint

ADR-039 adds a bounded authorization checkpoint only when a terminal local
review or implementation route needs D14 or a cloud implementer — not a
replacement for HITL approval and never a scope broadener.

`human-select` is the interactive default: a missing model, effort, or
selector returns `awaiting_fallback_selection` and stops. `preauthorized` is
allowed only when all three fields were frozen in the approved card or
preflight; incomplete preauthorization fails closed. The orchestrator
validates the receipt and packet digest before invoking exactly the selected
model/effort — missing, stale, or role-mismatched stays blocked. Full
schema and matrix: ADR-039 and `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
Human-selected fallback checkpoint (ADR-039)`.

D14 remains a read-only, context-isolated Balanced-tier reviewer. Selecting
it does not authorize cloud implementation; selecting a cloud implementer
does not waive independent review, RRI gates, repair budgets, or scope
checks.

## Permitted without prior approval

- Read-only analysis, search, and codebase navigation.
- Drafting plans, task lists, ADRs, and proposals (no code execution).
- Non-destructive fixes to documentation/configuration when explicitly
  authorized to "fix inconsistencies".
- Creating and updating the live per-task todo list
  (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Live per-task phase todo
  list`) — a transparency/tracking artifact only; it never substitutes for
  the HITL checkpoint, the band-routed review chain, or any closure gate.

## Safety rules

- Do not commit with broken tests; run all tests before commit/push.
- Ask before deleting; surface contradictions instead of proceeding.
- Redact secrets/credentials in logs and traces.
- Report outcomes faithfully: failing tests, skipped steps, and assumptions
  stated plainly.

## Band-routed peer review

Every development task is reviewed at two phases, resolved from RRI band:

- **RRI 0–25 (Low):** Muse Glimmer primary, Gemma intermediate, D14 final.
- **RRI 26–55 (Moderate + Med-high):** Gemma primary, Muse Glimmer
  intermediate, D14 final.
- **RRI 56+ (Complex+):** cross-vendor peer (replaces Gemma), D14 fallback.

Both bindings apply regardless of whether implementation stayed local or
escalated to cloud — the binding governs *who reviews*, independent of *who
authored the code*. Full routing table, binding history, failure-mode
sequencing, provider-resolution rules, and report-line contract: single
canonical definition in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
Band-routed peer review (two phases)` — not restated here.

Peer review **does not replace** the RRI-band human approval gate — it is a
separate, additional check. Phase-1 exemptions: docs/config/migration/ADR/
plan/task-ledger/policy-only tasks record `Task-analysis review: n/a`.

## Gemma Reviewer availability

Mandatory for every development task regardless of band; no path may be
skipped, and reviewer unavailability never opens a human approval gate
beyond what the RRI band already requires. Trigger conditions, retry
discipline, and current fallback chains:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer / Muse Glimmer
Reviewer § Availability`.

## Reviewability budget escape

`make qa-review-budget` fails closed when a change is too large for Gemma to
evaluate in-context. Staying inside the budget by splitting is the default,
no approval needed. For a genuinely irreducible change, the delivering agent
may **autonomously** take the documented escape — a `D14-OVERRIDE: <reason>`
line in the commit body or task entry, routing the change to D14 instead of
Gemma. This is for reviewability, not for skipping review: D14 still runs
and `disposition_divergence` is still recorded. Using the escape to avoid
review, or without a genuine reason, is a policy violation. Full derivation
and mechanics: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Reviewability
budget gate`.

## Review evidence override (urgency, human-only)

A completed development section may close without a `Review artifact:`
receipt only via a typed `REVIEW-OVERRIDE:` line
(`docs/policies/RRI_POLICY.md § Review evidence gate`;
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Review artifact receipt and
REVIEW-OVERRIDE lines`). `pipeline-failure` and `not-applicable` are
ordinary agent-supplied evidence. **`urgency`** is not: it requires a
`Waiver-by: <human name>` field, and an agent **may not self-issue it** —
invoking `urgency` without a prior explicit human waiver is out of scope of
this policy's autonomy, treated as an unauthorized skip of a mandatory
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
