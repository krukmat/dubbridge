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

The Antares Security-Specialist Advisor workflow is likewise advisory-only. The
primary agent or human security specialist owns the CWE hypothesis, threat model,
security rationale, tests, remediation, and final disposition. Antares only
localizes candidate files for an externally supplied CWE against an existing
snapshot and returns its exploration trace. It never computes the canonical RRI,
satisfies the human-approval gate, replaces the band-routed review chain, blocks
CI or closure, or closes work on its own authority. Every material candidate
requires a durable human disposition, and absence or failure of Antares never
grants or withholds approval.

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

Owner directive, 2026-08-12: every task that will invoke an Ollama-backed local
role must restart Ollama once before its first local-model request, even when the
current server appears healthy. The orchestrator records and completes the
`Restart Ollama + local-stack precheck` checklist item before allowing that
request. A new repository task ID creates a new restart boundary; retries,
repairs, and later local phases of the same task reuse the restarted server
unless it becomes unavailable or wedged.

Before the restart, the orchestrator must establish that no local runner for a
different task remains active. An unrelated bounded run is allowed to finish or
is stopped under its own termination contract; it is not killed as collateral
work. The restart is complete only when the previous server PID is gone, a new
`ollama serve` PID is present, port `11434` is listening, and every local model
required by the task passes the workflow guide's warm-up probe. Failure leaves
the operational checklist item blocked and prohibits the task's first local
request. This prerequisite does not waive or replace HITL approval, independent
review, fallback selection, or any RRI gate. The authoritative procedure is
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

The Low/S developer binding is `qwen3.8:27b-mlx` (owner directive, 2026-08-16,
replacing the prior `nemotron-3.5-lightning:30b-a3b-q4_K_M` binding — ADR-036
Amendments 3/4/7), the same model family as the Moderate/M implementer
(ADR-036 Amendment 6).

When Qwen Developer delegation is used, it must not evaluate, approve, or mark its own
delegated work as complete. Only the delegating agent may decide whether the task
satisfies the requirements.

For eligible simple code patches, the delegating agent must:

1. Compute RRI with `scripts/rri.py`.
2. Build a local delegation packet with the task excerpt, acceptance criteria, RRI
   output, allowed paths, relevant file snippets, and stop conditions.
3. Send the packet to Ollama/Qwen with `scripts/delegate-low-rri.py`, which uses
   the 120-second timeout and tagged-block response protocol defined in
   `docs/policies/RRI_POLICY.md`; require complete file contents, not JSON and not
   a unified diff.
4. Validate the tagged response, check the wrapper-built diff with
   `git apply --check`, and reject any patch outside the allowed task scope.
5. Apply only a valid in-scope patch.
6. Personally review the solution against every task requirement and acceptance
criterion; this evaluation must be performed by the delegating agent, not Qwen Developer.
7. Recompute/check actual touched scope; if the result now scores above RRI 25 or
   triggers a higher gate, stop and escalate to the normal approval workflow.
8. Run required verification commands.
9. If requirements are missed or checks fail, run one bounded Qwen Developer repair cycle
   with the failure evidence and the same allowed paths; if it still fails, stop and
   escalate. If Codex takes execution after that gate, use `gpt-5.6-luna` at
   `low`, or `gpt-5.6-terra` at `low` when Luna is unavailable in the active
   environment.
10. Report the RRI, Qwen Developer model used, files changed, the delegating agent's
    requirement-review result, verification commands, and whether a repair cycle
    was needed. If delegation times out, report `Qwen Developer timeout after 120s`.

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
   `DUBBRIDGE_LOCAL_AGENT_MODEL` (default `qwen3.8:27b-mlx`) and the endpoint
   from `OLLAMA_HOST`.
4. Keep the primary agent as orchestrator of record: it owns the task card,
   allowed paths, acceptance tests, reflection passes, closure, and all final
   judgments about correctness.
5. The runner gives the model only `write_file`, `apply_patch`, and `finish`.
   Complete authorized file contents are preloaded; model-issued reads and
   commands are disabled. Every edit is restricted to `allowed_paths`, and any
   forbidden tool or unlisted path terminates as `boundary_violation`. On
   `finish`, the runner formats only edited authorized Rust files via isolated
   copies and executes the card's acceptance commands in order. Failures return
   bounded repair evidence and refreshed authorized contents. (See
   `docs/plan/local-agent-simple-editing.md` for why the earlier Serena path
   was removed.)
6. Enforce the task's `allowed_paths` both at every file-tool call and after
   the local run. Any unlisted access or out-of-scope diff fails closed and is
   never accepted into the primary checkout.
7. Run the approved verification commands before issuing a signed DEV success
   audit. The `local-implementer` signature is valid when the final diff remains
   in scope and acceptance passes. Organization, review, coverage, and closure
   are separate orchestrator-owned workflow phases and do not alter that DEV
   result.
8. If the local run fails the acceptance signal, hits the scope boundary, or
   the local path is unavailable, the primary agent may run at most **2**
   evidence-backed local repair attempts.
9. After the repair budget is exhausted, or if the local runner/model is
   unavailable, escalate to cloud implementation with the ADR-036 escalation
   packet rather than continuing with ad hoc local retries. Use the concrete
   cloud-takeover model already recorded in the approved task card: for Codex,
   the current Balanced default is `gpt-5.6-terra` at `medium`. If the failures
   reveal a capability, ambiguity, coupling, or risk gap, re-run
   `scripts/rri.py` and re-apply the resulting gate before selecting the Premium
   `gpt-5.6-sol` route; an Ollama outage alone does not justify that promotion.

This routing is operative by owner override dated **2026-07-15**. It was
adopted ahead of the original ADR-036 promotion gate so that live
Moderate-band tasks become the evaluation surface.

It was extended to the Med-high band by owner override dated **2026-07-21**
with a tighter 1-attempt repair budget, and subsequently **replaced** for
Med-high by ADR-038 (2026-07-26) — see § Med-high Architect-refined
single-attempt gate (RRI 41–55) below. Med-high no longer uses any repair
attempt at all; the historical 1-attempt figure no longer applies to this
band, **except** for a module independently qualified under ADR-040
per-module split routing (§ Per-module complexity-split routing (RRI 26–55)
below), whose local tramo uses this Moderate section's 2-attempt budget
regardless of the containing task's band.

## Post-repair-budget Low-band decomposition (owner directive 2026-08-16)

**Owner directive, 2026-08-16:** once the whole-task local-agent route above
exhausts its repair budget (2/2 for Moderate; the ADR-038 gate's `GO_LOCAL`
exhausted, or a module's local tramo exhausted, for Med-high), the cloud
escalation in step 9 above is no longer the default next step. The default
is to **decompose the remaining work into Low-band (RRI 0–25) subtasks and
keep it local**, maximizing local-model usage; the primary agent's role
becomes orchestration — diagnosing, splitting, dispatching, reviewing, and
assembling — not authoring code directly, even for small, fully-diagnosed
mechanical fixes. Cloud escalation remains available but is now the
fallback of last resort for this step, not the default.

This directive was validated end-to-end on `S-150-T2c-iv-c` (RRI 39,
Moderate): after a whole-task local-agent attempt and 2 repair attempts
were exhausted (one degraded into a non-functional stub, one hit
`budget_exhausted`), the remaining implementation was decomposed into three
Low-band subtasks (create business-logic file, register the module, create
the test file) and delegated via `scripts/delegate-low-rri.py`, with the
orchestrator applying only a handful of individually-diagnosed one/two-line
fixes directly — and only after the delegation tooling itself (not the
model) failed twice to apply an already-correct model-proposed fix. See
`docs/tasks/s-150-translation-dubbing.md` § S-150-T2c-iv-c "Implementation
routing evidence" for the full evidence trail.

The route:

1. Confirm the whole-task local-agent repair budget for this band is
   genuinely exhausted (Moderate: 2/2 attempts recorded; Med-high: the
   ADR-038 gate result and any qualifying ADR-040 module tramo's own budget).
   This step does not reopen or repeat local-agent attempts — it only marks
   the trigger for switching implementation strategy.
2. Diagnose precisely. Read the actual repository signatures the failed
   attempt(s) needed (types, visibility, real field names, real function
   signatures) — do not guess. This diagnosis is what makes the following
   Low-band packets tight and low-risk; a vague packet reproduces the same
   failure the whole-task attempt already hit.
3. Decompose the remaining work into small Low-band (RRI 0–25) subtasks —
   typically one per file to create/edit, or one per class of fix. Score
   each subtask's RRI with `scripts/rri.py` to confirm it is genuinely Low;
   a subtask that scores above 25 is not eligible for this route and must
   go back through the normal RRI-gated workflow instead.
4. Delegate each subtask via `scripts/delegate-low-rri.py`: `--mode
   full-file` for new files, `--mode before-after` for small, one-function
   edits to existing files. Build each packet with the verified real
   signatures from step 2 so the local model is not guessing at the API
   surface either.
5. Review every returned patch against the acceptance criteria before
   applying it — this is the same personal-review obligation as any other
   Low-band delegation (§ Local delegation (RRI 0-25) above). Run
   `cargo check`/equivalent after each applied subtask to catch the actual
   remaining error surface before writing the next packet.
6. **Documented tooling-failure exception:** if a delegation tool
   (`delegate-low-rri.py`, `run_local_task.py`, or equivalent) fails to
   apply a fix the local model has already correctly diagnosed and proposed
   (for example, a `before-after` wrapper that cannot construct a non-empty
   diff for a valid anchor), the orchestrator may apply that specific,
   already-verified fix directly rather than repeatedly retrying the same
   tooling failure. This is not a substitute for delegation — it requires
   that the model's proposed fix already be visible and correct, and it
   must be recorded explicitly as a tooling-failure exception (not silently)
   in the task's implementation-routing evidence, distinct from any fix the
   orchestrator diagnosed and authored itself.
7. Mechanical, lint-driven restructuring of already-verified logic (for
   example, extracting helper functions solely to satisfy a cognitive-
   complexity gate, with no behavior change) may be applied directly without
   a delegation round — it is refactor of already-authored logic against a
   deterministic rule, not new authorship, and record it as such.
8. Once all subtasks are assembled and verified, resume the task's normal
   band-appropriate closure: band-resolved reviewer (phase 2), Reflection
   passes for the task's original RRI band, unit coverage certification, and
   owner verification, exactly as if the whole task had completed on the
   original local-agent route. This directive changes only how the
   remaining implementation is produced after the repair budget is
   exhausted — it does not change the task's RRI, band, review chain,
   Reflection pass count, or approval gate.
9. Record an implementation-routing evidence block in the task closure
   record naming: the whole-task route and why its budget was exhausted;
   each Low-band subtask with its RRI, delegation mode, and outcome; every
   direct edit the orchestrator made, classified as either a tooling-failure
   exception (step 6) or a mechanical lint-driven refactor (step 7); and the
   net authorship split. Do not report a task closed under this route
   without this block — it is what lets a reviewer distinguish "local model
   authored it, orchestrator assembled it" from "orchestrator authored it
   directly," which is the exact distinction this directive exists to keep
   visible.

**No additional approval checkpoint per subtask.** The owner directive
explicitly waives re-confirmation for this decomposition/dispatch/assembly
loop once the containing task is already HITL-approved and has reached this
post-repair-budget point — this is a bounded waiver for an already-approved
task's implementation mechanics, not a waiver of the RRI 26+ human approval
gate itself, which still fires once, at task presentation, as normal.

This directive is a standing evaluation: deviations from it (the
orchestrator authoring code directly when Low-band delegation was viable,
or skipping the implementation-routing evidence block) are the failure mode
to correct going forward, not a one-off exception.

## Med-high Architect-refined single-attempt gate (RRI 41–55)

ADR-038 (2026-07-26) governs implementation routing for final **RRI 41–55**.
The approval gate is unchanged: the agent must present the task and wait for
explicit human approval before implementation. Band-resolved independent
review (phases 1 and 2), 3 Reflection passes, and the "Plan + explicit
acceptance criteria" gate all still apply.

The route:

1. Compute RRI with `scripts/rri.py`; confirm it falls in 41–55.
2. Present the task and obtain explicit approval.
3. Request a Muse Glimmer (`muse-glimmer:30b-q4_K_M`) advisory refinement via
   `scripts/local-architect/run_analysis.py`'s `med-high-refinement-v1`
   profile. It returns `route_recommendation: GO_LOCAL | CLOUD_REQUIRED`
   bound to the task capsule hash and its own model tag/digest.
4. The primary agent issues its own hash-bound route receipt, evaluated by
   `scripts/local-agent/med_high_gate.py`. The primary may **downgrade**
   GO_LOCAL to cloud; it may **never upgrade** CLOUD_REQUIRED to local — this
   is enforced structurally (the gate requires both sides to independently
   say GO_LOCAL), not by trusting either decision alone.
5. `scripts/local-agent/run_med_high_task.py` records every gate result as a
   cloud handoff; even `GO_LOCAL` is policy-excluded from local development
   — **except** for a module independently qualified under ADR-040
   per-module split routing (§ Per-module complexity-split routing (RRI
   26–55) below), which is a narrower, separately-gated exception and does
   not reopen whole-task Med-high local development.
6. Escalate to the concrete Codex or Claude
   cloud-takeover model recorded in the approved task card, with the full
   ADR-038 §5 evidence bundle: task capsule, refinement artifact,
   primary receipt, effective limits, transcript/checkpoint, partial diff,
   commands/tests run, stop reason, hashes, model identity, elapsed time.
   When Codex executes, an operational-only fallback uses `gpt-5.6-terra` at
   `high`; a hard exclusion, risk/capability `CLOUD_REQUIRED`, or local
   acceptance/scope/organization failure uses `gpt-5.6-sol` at `high`.
7. Run the approved verification commands and the organization gate on the
   cloud-authored implementation.

Hard exclusions from GO_LOCAL regardless of the Muse Glimmer recommendation:
auth/security work, rights/consent/governance invariants, schema/migrations/
release cuts, unresolved ADR decisions, and unbounded scope — see ADR-038 §6.

This gate does not weaken the independent review route defined by the
"Band-routed peer review" section below.

## Per-module complexity-split routing (RRI 26–55, ADR-040)

Owner directive, 2026-08-16, formalized as `ADR-040` (amends ADR-036 and
ADR-038 Amendment 2): for an **approved** development task with final RRI
26–55 whose `allowed_paths` span two or more files, the orchestrator may
split implementation authorship by per-module cyclomatic complexity instead
of routing the whole task through §§ above as one unit.

- **Trigger:** measure raw CC per file (`--auto-cc`, existing RRI `C` table).
  Split only when heterogeneous — at least one module C≥2 (CC≥11) and at
  least one C≤1 (CC≤10). A uniform-tier task is not split.
- **Hard domain exclusion carried from ADR-038 §6:** a module touching auth,
  security, rights/consent/governance invariants, schema/migrations, an
  unresolved ADR decision, or unbounded scope is always cloud-eligible
  regardless of its own CC. This is what keeps this exception from
  reopening the risk ADR-038 Amendment 1 (2026-08-12) closed for Med-high.
- **Disjoint `allowed_paths`:** the two tramos must partition the file set
  with no overlap, or the task is not split.
- **Interface freeze:** before dispatch, the orchestrator records the exact
  boundary contract between the local- and cloud-eligible modules; neither
  implementer redefines it.
- **Routing and repair budgets:** C≤1/non-excluded modules go to the local
  implementer (`run_local_task.py`) with **2** evidence-backed repair
  attempts, uniformly regardless of the task's overall band. C≥2 or
  hard-excluded modules go to the band's resolved cloud model with **1**
  repair attempt, then **one** escalation to the band's higher cloud tier,
  then stop and report blocked for that module.
- **Integration gate (mandatory):** run the task's full verification against
  the merged diff before Reflection. A tramo-attributable failure is a
  bounded repair against that tramo's own budget. A failure attributable to
  the interface contract itself abandons the split and escalates the whole
  task to its normal band route — never retried as a split.
- **Review/approval unaffected:** the band-resolved reviewer (Gemma), the
  Reflection pass count for the task's band, the RRI 26+/41+ human approval
  gate, unit coverage certification, and owner verification all evaluate the
  final unified diff as one task, exactly as if no split had occurred.

Full contract, evidence-block format, and tooling status:
`docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md` and
`docs/policies/RRI_POLICY.md` § Per-module complexity-split routing.

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

1. The responsible script emits a `fallback-selection-v1` artifact, bound by
   SHA-256 to the exact fallback packet, before invoking any fallback model.
2. `human-select` is the interactive default. A missing model, reasoning effort,
   or human selector returns `awaiting_fallback_selection`; the process stops and
   must not invoke D14 or a cloud implementer.
3. `preauthorized` is allowed only when all three selection fields were frozen in
   the approved task card or preflight. Any incomplete preauthorization fails
   closed.
4. The orchestrator must validate the authorized receipt and packet digest, then
   invoke exactly its selected model and effort. A stale, missing, mismatched, or
   role-confused receipt stays blocked.

D14 remains a read-only, context-isolated Balanced-tier reviewer. Selecting it
does not authorize cloud implementation; selecting a cloud implementer does not
waive independent review, RRI gates, repair limits, or scope checks.

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

Every development task is reviewed by an independent reviewer at two phases.
The reviewer is determined by the task's RRI band:

- **RRI 0–25 (Low):** Muse Glimmer (phases 1 and 2, owner directive
  2026-08-11), with Gemma as intermediate fallback. Phase-2 = Muse Glimmer
  Reviewer N-pass; phase-1 = advisory Muse Glimmer review of the task card.
- **RRI 26–55 (Moderate + Med-high):** Gemma (phases 1 and 2, owner
  directive 2026-08-11) — see `docs/policies/RRI_POLICY.md § Local
  pipeline phase-1/phase-2 reviewer bindings`. This reverts the 2026-07-21
  override that had used `qwen3.6:27b-q4_K_M` in this role (that binding
  became the local implementer per ADR-036 Amendment 2, since superseded by
  Amendment 3's rebind to `nemotron-3.5-lightning:30b-a3b-q4_K_M`), with Muse Glimmer as intermediate
  fallback, regardless of whether implementation stayed local or escalated
  to cloud.
- **RRI 56+ (Complex+):** cross-vendor peer (phases 1 and 2). The peer
  replaces Gemma as the code-solution reviewer for this band.

**Cross-vendor primary-review resolution (RRI 56+ only):**
`claude-code → codex | codex → claude | local-provider → claude |
remote-provider → claude | unknown → claude`

**D14 provider resolution (all bands):** when D14 is triggered, first use a
responsive reviewer from a provider different from the primary orchestrator's
provider. Same-provider D14 is allowed only after that cross-provider attempt
is unavailable, unauthenticated, stalled, or invalid/`BLOCKED`; record it as a
degraded fallback with the failed cross-provider evidence.

**Failure modes (RRI 0–25):**
1. Muse Glimmer unavailable, stalled, or returns invalid/`BLOCKED` output →
   fall back to **Gemma** (one immediate retry with the same review packet
   if Muse Glimmer itself is unusable on the first attempt).
2. Muse Glimmer + Gemma both unavailable/unusable → fall back to **D14**
   (Balanced tier, cross-provider first; same-provider only as a recorded
   degraded final fallback).
3. Muse Glimmer + Gemma + D14 all unavailable → write a blocked-artifact
   record and stop. Never self-review. Report the task as blocked.

**Failure modes (RRI 26–55):**
1. Gemma unavailable, stalled, or returns invalid/`BLOCKED`
   output → fall back to **Muse Glimmer** (one immediate retry with the same
   review packet if Gemma itself is unusable on the first attempt).
2. Gemma + Muse Glimmer both unavailable/unusable → fall back to
   **D14** (Balanced tier, cross-provider first; same-provider only as a
   recorded degraded final fallback).
3. Gemma + Muse Glimmer + D14 all unavailable → write a
   blocked-artifact record and stop. Never self-review. Report the task as
   blocked.

**Failure modes (RRI 56+):**
1. Peer CLI unavailable or unauthenticated → fall back to **D14** (Balanced tier,
   cross-provider first; same-provider only as a recorded degraded final fallback).
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

The review step is **mandatory** for all Low (0–25) development tasks. Muse
Glimmer is the preferred reviewer (owner directive, 2026-08-11), with Gemma
inserted as the intermediate fallback before the context-isolated subagent
(D14, `scripts/adjudicator-packet.py`), the required final fallback:
`muse-glimmer:30b-q4_K_M → gemma4:26b-a4b-it-qat → D14`. For Moderate and
Med-high (26–55), the equivalent mandatory review step uses **Gemma** as the
primary reviewer — see § Band-routed peer review above — reverting the
2026-07-21 override that had used `qwen3.6:27b-q4_K_M` in this role (that
binding became the local implementer under ADR-036 Amendment 2, since
superseded by Amendment 3's rebind to `nemotron-3.5-lightning:30b-a3b-q4_K_M`), with Muse Glimmer inserted as the intermediate fallback
before D14: `gemma4:26b-a4b-it-qat → muse-glimmer:30b-q4_K_M → D14`. The
retry-then-escalate discipline described below applies at each step of both
chains.

When Ollama is unavailable, the model is absent, the reviewer stalls, output
is invalid, the review result is `BLOCKED`, or no usable consolidated review
result can be produced, the agent must perform **one immediate retry** with
the same review packet first. If the retry succeeds with a usable result, the
primary path continues normally. If the retry fails for the same class of
reason or still produces no usable result, the agent **must** spawn a
context-isolated subagent as the mandatory fallback reviewer. D14 must first
be cross-provider; same-provider D14 is permitted only after a failed,
evidenced cross-provider attempt and is recorded as degraded. The subagent
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
- `docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`
