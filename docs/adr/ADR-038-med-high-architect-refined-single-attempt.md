---
type: ADR
title: "ADR-038: Architect-refined single local attempt for Med-high tasks"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-038: Architect-refined Single Local Attempt for Med-high Tasks

- **Status:** Accepted
- **Date:** 2026-07-26
- **Deciders:** DubBridge owner and platform-agent workflow maintainers
- **Scope:** agent workflow and local delegation only; no application-runtime or
  product-architecture change
- **Amends:** ADR-036 local implementation routing and ADR-037 Local Architect
  invocation timing
- **Owner approval:** the owner accepted the proposed route in the active session
  with `si, de acuerdo` after reviewing the exact Qwen27 -> primary decision ->
  Qwen35/cloud workflow and its limits.

## Context

The 2026-07-21 owner override extended the RRI 26-40 local-first route to the
Med-high band (RRI 41-55). That extension reused the Moderate runner with a
nominally tighter repair budget, but the executable runner remained global: up
to 30 model turns, two post-test repairs, and a per-chat wall limit of 1,800
seconds. A real RRI 55 session reached turn 23 without producing a patch before
the operator stopped it. This single observation does not by itself satisfy the
rolling rollback-rate trigger, but it demonstrates that direct Med-high
local-first authoring can spend substantial time before producing evidence.

ADR-036 originally rejected general local implementation for RRI 41+ because
ambiguity, blast radius, and verification burden are precisely what raise the
band. ADR-037 already permits Qwen3.6-27B to decompose a high-RRI problem, but it
places that role upstream of operative planning. The useful middle ground is a
bounded routing consultation after a task is approved and before any local code
authoring, followed by at most one short local implementation session.

The aggregate change scores RRI 93 (Very high) when policy, ADR, runner,
supervision, tests, and status synchronization are treated as one patch. It must
therefore be delivered as the subtasks in
`docs/tasks/med-high-local-refinement.md`, each scoring RRI <= 55.

## Amendment 1 (2026-08-12): Med-high local execution disabled

Owner directive: Nemotron is scoped to Low/S and Moderate/M only. Therefore the
Med-high refinement and primary receipt remain required evidence, but their
combined `GO_LOCAL` result is policy-excluded from launching a local developer.
The supervisor emits the normal cloud handoff bundle for every valid Med-high
route, including `GO_LOCAL`; reviewer independence, approval, and fallback
selection remain unchanged.

## Decision

### 1. Preserve Low and Moderate routing

- RRI 0-25 keeps the existing Low-band gate.
- RRI 26-40 remains local-first when the normal eligibility gates pass.
- This ADR changes only RRI 41-55 Med-high implementation routing.

### 2. Add one advisory refinement before Med-high implementation

After phase-1 readiness review and human approval, the orchestrator freezes the
approved task capsule and invokes the ADR-037 Local Architect exactly once with
`qwen3.6:27b-q4_K_M`. The invocation is tool-free and read-only. Its structured
`med-high-refinement-v1` artifact must contain:

- the task/capsule hash and exact model tag/digest;
- `GO_LOCAL` or `CLOUD_REQUIRED`;
- refined in-scope and out-of-scope boundaries;
- ordered implementation steps and deterministic acceptance tests;
- material risks, unknowns, stop conditions, and claim provenance.

The Local Architect recommends a route; it does not approve or implement the
task. An unavailable, timed-out, malformed, stale, or hash-mismatched artifact
is equivalent to `CLOUD_REQUIRED`.

### 3. Keep the primary agent as the fail-closed route authority

The primary agent independently verifies the artifact against the approved card,
governing ADRs, repository evidence, and local eligibility gates. It emits a
hash-bound route receipt with its identity, timestamp, rationale, and decision.

- The primary may downgrade Qwen27 `GO_LOCAL` to cloud.
- The primary may never upgrade `CLOUD_REQUIRED` to local.
- A material scope or acceptance change invalidates the original approval and
  requires a new capsule, RRI computation, review, and approval.
- The Local Architect artifact is not silently reused as the official phase-1
  review artifact. The two roles use separate artifacts/invocations unless a
  later explicit policy amendment defines a combined schema and authority.

### 4. Define "one local round" precisely

Only `GO_LOCAL` plus a valid primary receipt may start the exact local
implementer binding `qwen3.6:35b-a3b`. The Med-high budget is:

- one implementation session total;
- at most 8 model/tool turns;
- at most 300 seconds total wall time for the supervised session;
- zero post-acceptance-test repair cycles;
- no silent model substitution.

The implementer may inspect, edit, and run tests within those limits. Once it
calls `finish`, a failing acceptance gate ends the session; it does not receive a
repair turn. Turn, wall, transport, scope, organization, boundary, no-diff, or
test failure immediately selects the cloud route.

### 5. Preserve evidence during cloud escalation

Every non-successful local route emits a handoff bundle containing the immutable
task capsule, refinement artifact, primary receipt, effective limits, transcript
or last checkpoint, partial diff, commands/tests, stop reason, hashes, model
identity, and elapsed time. Codex or Claude consumes that bundle instead of
re-exploring the task from scratch.

### 6. Exclude high-confidence surfaces from `GO_LOCAL`

The primary must choose cloud regardless of Qwen27's recommendation for:

- authentication or security boundaries;
- rights, consent, publication, or other fail-closed governance invariants;
- schema/data migrations and release cuts;
- an unresolved ADR-level decision;
- unbounded cross-module scope, non-deterministic acceptance, critical unknowns,
  or a target file that fails the local delegation size gate.

### 7. Keep review and approval rigor unchanged

The new refinement is an implementation-routing gate. It does not reduce the
Med-high human approval requirement, three Reflection passes, phase-2 review,
owner verification, or status synchronization. Reviewer fallback is not routing
fallback: failure of the advisory refinement routes implementation to cloud,
while official reviewer failure follows the review policy's own chain.

## Risk analysis

| Risk | Failure mode | Mitigation |
|---|---|---|
| False-positive `GO_LOCAL` | Local model starts an unsuitable task | Primary verification, hard exclusions, hash-bound receipt, short one-shot budget |
| False-negative `CLOUD_REQUIRED` | A locally solvable task spends cloud capacity | Accepted cost of fail-closed routing; evidence can refine later policy |
| Stale or tampered artifact | Receipt authorizes a different task | Capsule/artifact SHA-256 binding and model-digest validation |
| Slow or stuck local session | Repeats the observed 23-turn stall | Supervisor-enforced 8-turn/300-second process-group cutoff |
| Hidden repair loop | "One round" becomes multiple attempts | Runner budget is one session and zero repairs; tests lock the semantics |
| Scope drift after approval | Refinement changes what the owner approved | Any material scope/acceptance change requires a new capsule and gate |
| Same-model authority conflation | Advisory output is treated as review/approval | Separate artifacts and explicit role labels; primary and human retain authority |
| Lost escalation context | Cloud repeats exploration | Automatic handoff bundle includes all local evidence and hashes |

## Consequences

### Positive

- Med-high tasks receive local architecture/decomposition value without an
  unbounded local coding attempt.
- A narrowly viable task can still use local implementation once.
- Cloud fallback begins with reusable evidence and a refined scope.

### Negative

- The route adds one local analysis invocation before implementation.
- The primary must verify and attest the recommendation.
- Some solvable local tasks will intentionally escalate when evidence is
  incomplete or the five-minute budget is insufficient.

## Alternatives considered

- **Keep direct local-first for all RRI 41-55:** rejected because observed
  latency and the runner's global budgets do not match the band's risk.
- **Route every Med-high task directly to cloud:** safe but discards the useful
  ADR-037 decomposition lane and all bounded local implementation opportunities.
- **Let Qwen27 implement:** rejected; ADR-037 keeps the dense model advisory and
  tool-free, and ADR-036 records why it is unsuitable for agentic loops.
  **Superseded note (2026-08-11):** ADR-036 Amendment 2 records the owner's
  explicit decision to re-enter this exact rejected option — Qwen3.6-27B is
  now the local implementer. This historical rejection is preserved as the
  record of the original engineering rationale (dense/bandwidth-bound decode
  on base M5) and is not deleted; it no longer reflects the operative
  binding. See Amendment 1 below.
- **Allow one repair after a failed acceptance test:** rejected because it means
  two implementation attempts, contradicting the owner's one-round constraint.

## Amendment 1 (2026-08-11): Rebind advisory refinement and implementer per ADR-036/037 Amendment set

**Reason:** ADR-036 Amendment 2 moves the local implementer binding to
`qwen3.6:27b-q4_K_M` (was `qwen3.6:35b-a3b`); ADR-037 Amendment 1 moves the
Local Architect / Complex Analyst binding to `muse-glimmer:30b-q4_K_M` (was
`qwen3.6:27b-q4_K_M`). This ADR names both bindings explicitly in its
operative sections and must be amended in the same change to stay accurate,
per the ADR change-propagation contract in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.

**Changes to §2 (Add one advisory refinement before Med-high implementation):**
every reference to invoking "the ADR-037 Local Architect exactly once with
`qwen3.6:27b-q4_K_M`" now reads "the ADR-037 Local Architect exactly once
with `muse-glimmer:30b-q4_K_M` (per ADR-037 Amendment 1)". The
`med-high-refinement-v1` artifact contract (task/capsule hash, model
tag/digest, `GO_LOCAL`/`CLOUD_REQUIRED`, boundaries, steps, acceptance tests,
risks/unknowns/stop-conditions/provenance) is unchanged.

**Changes to §4 (Define "one local round" precisely):** "the exact local
implementer binding `qwen3.6:35b-a3b`" now reads "the exact local implementer
binding `qwen3.6:27b-q4_K_M` (per ADR-036 Amendment 2)". The 8-turn/300-second/
0-repair budget and no-silent-substitution rule are unchanged.

**Changes to Risk analysis table:** the "Same-model authority conflation"
row's mitigation ("Separate artifacts and explicit role labels; primary and
human retain authority") is strengthened by the rebind itself: the advisory
refiner (Muse Glimmer) and the implementer (Qwen3.6-27B) are now different
model families by construction, not only by role label — matching ADR-036
§5's cross-family independence rule.

**Not changed by this amendment:** the fail-closed route authority (§3), the
hard exclusions from `GO_LOCAL` (§6), review/approval rigor (§7), and the
evidence-preservation contract (§5) apply identically to the new bindings.

## Amendment 2 (2026-08-16): Narrow per-module exception to "no local attempt"

**Reason:** owner directive, formalized as `ADR-040` — per-module
complexity-split implementation routing. This amendment is a targeted
qualification of Amendment 1 (2026-08-12), not a reversal of it.

**Changes to Amendment 1 (2026-08-12: Med-high local execution disabled):**
Amendment 1's rule — that a `GO_LOCAL` refinement/receipt result is
policy-excluded from starting a local developer, and every Med-high task
routes to cloud — now reads: *unless the task qualifies for `ADR-040`
per-module complexity-split routing*, in which case the modules `ADR-040`
independently determines to be low-CC (raw CC ≤ 10) **and** not hard-excluded
by §6 below may be authored by the local implementer, under `ADR-040`'s own
2-attempt repair budget. Every other module in the task — including every
module `ADR-040` classifies as cloud-eligible, and the whole task when
`ADR-040`'s split trigger is not met — still follows this ADR's existing
cloud-only route unchanged.

**§6 (Exclude high-confidence surfaces from `GO_LOCAL`) is not weakened by
this amendment — it is extended.** `ADR-040` §4 requires every module
matching this section's hard-exclusion criteria to route cloud regardless of
its own cyclomatic complexity. A module inside a `GO_LOCAL`-refined,
low-CC-qualifying task that nonetheless touches an excluded surface (auth,
security, rights/consent/governance invariants, schema/migrations, an
unresolved ADR decision, or unbounded scope) is cloud-eligible under
`ADR-040`, never local-eligible, exactly as this section already requires for
the whole-task case.

**§4 (Define "one local round" precisely) does not apply to an `ADR-040`
module-split local tramo.** §4's 8-turn/300-second/zero-repair budget governs
a whole-task Med-high local attempt, which Amendment 1 already disabled.
`ADR-040`'s local tramo is a different, narrower mechanism — restricted to
modules independently verified as both low-CC and domain-non-sensitive — and
uses `ADR-040` §8's own repair budget (2 attempts), not this section's.

**§7 (Keep review and approval rigor unchanged) is unaffected.** The
band-resolved reviewer (Gemma), 3 Reflection passes, the RRI 41+ human
approval gate, owner verification, and status synchronization all continue
to evaluate the task's final unified diff as a whole, per `ADR-040` §10.

**Not changed by this amendment:** the fail-closed route authority (§3), the
refinement/receipt evidence requirement (§2, §5), and the Amendment 1
policy-exclusion of `GO_LOCAL` for every module `ADR-040` does not
independently qualify for local routing.

## Amendment 3 (2026-08-23): Reopen local-first for the low sub-band (RRI 41-45)

**Reason:** owner directive during S-150-T3c presentation
(`docs/tasks/s-150-translation-dubbing.md` § S-150-T3c), given directly by the
DubBridge owner as final authority over this workflow policy. The owner was
shown, and explicitly acknowledged, that this reopens the exact whole-task
local-attempt path Amendment 1 (2026-08-12) disabled after a real RRI 55
session stalled at turn 23 without producing a patch, and confirmed the
reopening anyway for the narrower 41-45 sub-band. This amendment is a
permanent policy change ("de ahora en más"), not a one-off waiver for a single
task.

**Changes to Amendment 1 (2026-08-12: Med-high local execution disabled):**
Amendment 1's rule — that a `GO_LOCAL` refinement/receipt result is
policy-excluded from starting a local developer, and every Med-high task
routes to cloud — now applies only to **RRI 46-55**. For **RRI 41-45**, a
`GO_LOCAL` refinement/receipt result routes the whole task through the
Moderate local-first path (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
Local-first and Architect-refined implementation routing (RRI 26-55)` and
`docs/policies/HITL_AUTONOMY_POLICY.md § Local-first implementation (RRI
26-40 Moderate)`), including:

- `scripts/local-agent/run_local_task.py` in a disposable worktree, primary
  agent as orchestrator of record;
- at most 2 evidence-backed local repair attempts;
- on 2/2 exhaustion, the default next step is `docs/playbooks/
  AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band decomposition` —
  decompose the remaining work into scored Low-band (RRI 0-25) subtasks and
  keep authoring local via `scripts/delegate-low-rri.py`, primary agent as
  orchestrator only (diagnosing, splitting, dispatching, reviewing,
  assembling — never authoring substantive logic directly);
- cloud escalation (this ADR's existing cloud-takeover packet, § Preserve
  evidence during cloud escalation) remains the fallback of last resort if
  and only if that decomposition route itself cannot proceed — never the
  default next step after repair-budget exhaustion.

A `CLOUD_REQUIRED` result in the 41-45 sub-band is unaffected: it still routes
directly to cloud per §§2-3, with no local attempt, exactly as the rest of
Med-high.

**§4 (Define "one local round" precisely) does not apply to a 41-45
local-first attempt.** §4's 8-turn/300-second/zero-repair budget continues to
govern only the 46-55 sub-band's now-narrower cloud-only rule (there is no
local attempt to bound in 46-55) and the ADR-040 module tramo (Amendment 2).
The 41-45 sub-band instead uses the Moderate local-first runner's own budget
(2 evidence-backed repair attempts, then decomposition) — a materially
different, less time-bounded mechanism than §4's single-session cutoff. This
reintroduces, for a narrower RRI window, exactly the risk profile
(potentially long-running local sessions before escalation) that motivated
Amendment 1; the owner accepted this tradeoff explicitly for 41-45 while
requiring it not extend to 46-55.

**§6 (Exclude high-confidence surfaces from `GO_LOCAL`) is unchanged and
fully applies to 41-45.** The primary must still choose cloud regardless of
Muse Glimmer's recommendation for auth/security, rights/consent/governance
invariants, schema/migrations/release cuts, an unresolved ADR decision, or
unbounded scope — this amendment only changes what happens *after* a
genuine, hard-exclusion-clean `GO_LOCAL` result in the 41-45 sub-band.

**§7 (Keep review and approval rigor unchanged) is unaffected.** The
band-resolved reviewer (Gemma primary, Muse Glimmer intermediate, D14 final
per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`), 3
Reflection passes, the RRI 41+ human approval gate, owner verification, and
status synchronization all continue to apply to the whole 41-55 range
unchanged — this amendment affects only *who authors the code*, never the
review/approval chain.

**Interaction with Amendment 2 (ADR-040 per-module split):** unaffected.
Amendment 2's per-module tramo remains available to a 46-55 task whose
`allowed_paths` qualify, exactly as before. A 41-45 task now has two
available local-authoring mechanisms — whole-task local-first (this
amendment) or, if its files happen to also satisfy ADR-040's heterogeneous-
complexity trigger, a per-module split — the orchestrator selects whichever
applies; this amendment does not require evaluating ADR-040 first.

**Not changed by this amendment:** the fail-closed route authority (§3), the
refinement/receipt evidence requirement (§2, §5), the hard exclusions (§6),
review/approval rigor (§7), and the Amendment 2 per-module exception for
46-55. `S-150-T3c` itself (final RRI 50, in the unchanged 46-55 sub-band)
remains cloud-only under this ADR exactly as before this amendment.

## Amendment 4 (2026-08-30): Low-band decomposition before cloud takeover in 46-55

**Reason:** owner directive during P1.A2 presentation
(`docs/tasks/mvp0-p2p-p1-replication.md` § P1.A2), given directly by the
DubBridge owner as final authority over this workflow policy: when Amendment
1's whole-task cloud-only rule applies, or the Moderate/41-45 local-first
repair budget is exhausted, the orchestrator must first attempt to increase
local participation by decomposing the remaining work into scored Low-band
(RRI 0-25) subtasks via `scripts/delegate-low-rri.py`, escalating to cloud
only for the residue that itself scores Moderate or higher. This is a
generalization of `§ Post-repair-budget Low-band decomposition` (already the
default for Moderate/41-45 in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`/`docs/policies/
HITL_AUTONOMY_POLICY.md`) to the 46-55 sub-band's cloud-only trigger.

**Changes to Amendment 1 (2026-08-12: Med-high local execution disabled):**
Amendment 1's rule stands unchanged in substance — a `GO_LOCAL`
refinement/receipt result never starts a **whole-task** local developer in
46-55, and §4's 8-turn/300-second/zero-repair budget for a single Med-high
local round is not reopened. What changes is what happens **between** that
exclusion firing and the cloud-takeover packet being emitted: the
orchestrator must attempt Low-band decomposition of the task's remaining
scope first.

**Mechanism:**

1. On a 46-55 `GO_LOCAL` or `CLOUD_REQUIRED` result (or on Moderate/41-45
   local-first repair-budget exhaustion carrying a residual scope still in
   46-55), the orchestrator decomposes the approved task's remaining
   implementation into candidate subtasks scoped to the smallest coherent
   file/behavior unit it can identify from the approved `allowed_paths` and
   acceptance criteria.
2. Each candidate subtask is scored independently with `scripts/rri.py`.
   Subtasks scoring RRI 0-25 are dispatched via `scripts/delegate-low-rri.py`
   (`--mode full-file` for new files, `--mode before-after` for small edits),
   primary agent as orchestrator only — diagnosing, splitting, dispatching,
   reviewing, and assembling, never authoring substantive logic directly.
3. Any candidate subtask that does not score RRI 0-25 (Moderate, Med-high, or
   higher) is **not** forced into artificial decomposition to fit Low — it
   routes to this ADR's existing cloud-takeover packet (§ Preserve evidence
   during cloud escalation) on its own, carrying its own RRI and evidence.
4. Cloud is the fallback of last resort only for: (a) subtasks that
   themselves score above Low, and (b) any Low-band subtask whose delegation
   attempt fails its bounded repair cycle. It is never the default first
   response to the 46-55 exclusion firing.

**§6 (Exclude high-confidence surfaces from `GO_LOCAL`) is not weakened by
this amendment.** A candidate subtask touching an excluded surface
(auth/security, rights/consent/governance invariants, schema/migrations, an
unresolved ADR decision, or unbounded scope) is never Low-band eligible
regardless of its measured RRI — it routes cloud exactly as the whole task
would have.

**§7 (Keep review and approval rigor unchanged) is unaffected.** The
containing task's band-resolved reviewer (Gemma primary, Muse Glimmer
intermediate, D14 final), 3 Reflection passes, the RRI 41+ human approval
gate, owner verification, and status synchronization continue to evaluate
the final unified diff as a whole. No additional per-subtask approval is
required once the containing task is already HITL-approved — this amendment
changes only who authors the remaining code, never RRI, band, reviewer,
Reflection count, or the approval gate. A `### Implementation routing
evidence` block recording the decomposition attempt (including a `no
decomposition possible` result and its reason, when applicable) is required
in the closure record, per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
Post-repair-budget Low-band decomposition`.

**Interaction with Amendment 2 (ADR-040 per-module split) and Amendment 3
(41-45 local-first):** unaffected and evaluated first when applicable.
ADR-040's per-module split and Amendment 3's 41-45 local-first path remain
available exactly as before; this amendment applies only once those routes
do not apply or are exhausted, as the step before — not instead of — the
existing cloud-takeover packet.

**Not changed by this amendment:** the fail-closed route authority (§3), the
refinement/receipt evidence requirement (§2, §5), the hard exclusions (§6),
review/approval rigor (§7), and Amendment 1's prohibition on a whole-task
local developer in 46-55.

## Related

- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md`
- `docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`
- `docs/plan/med-high-local-refinement.md`
- `docs/tasks/med-high-local-refinement.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/RRI_POLICY.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
