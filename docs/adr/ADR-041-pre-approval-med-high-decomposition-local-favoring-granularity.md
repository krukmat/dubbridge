---
type: ADR
title: "ADR-041: Pre-Approval Med-High Task Decomposition for Local-Favoring Granularity"
status: Proposed
supersedes: ""
superseded_by: ""
---

# ADR-041: Pre-Approval Med-High Task Decomposition for Local-Favoring Granularity

- **Status:** Proposed
- **Date:** 2026-08-23
- **Deciders:** proposed by primary orchestrator (Claude Code) for DubBridge owner
  review; not yet ratified
- **Scope:** agent workflow and task-authoring practice only; no application-runtime
  or product-architecture change
- **Amends (pending acceptance):** extends `docs/policies/RRI_POLICY.md` §
  Decomposition triggers with a new, non-mandatory, favored evaluation step for
  RRI 41-55; does not alter ADR-038's routing rule for any resulting subtask
  that itself still scores 41-55 — that subtask simply follows whatever
  ADR-038 already prescribes for its own final RRI (cloud-only for 46-55;
  local-first on `GO_LOCAL` for 41-45 per ADR-038 Amendment 3, 2026-08-23);
  operates on a different axis than ADR-040 (which splits files inside one
  already-approved, already-scored task — this ADR splits the work item
  itself, before scoring/approval, into separately scored subtasks)
- **Open decision points requiring owner sign-off before Accepted:**
  1. Whether this evaluation step is *mandatory* (must be performed and recorded
     for every 41-55 candidate, even when the answer is "no genuine seam") or
     *discretionary* (orchestrator judgment call, recorded only when performed).
  2. Whether a subtask produced by this mechanism that lands back in 41-55 may
     make one further recursive attempt, or must terminate after one evaluation
     (§5 currently proposes: terminate after one).
  3. Whether the guardrail in §3 (hard-excluded domains never benefit from
     splitting) reuses ADR-038 §6's exclusion list verbatim, or needs a
     task-level variant distinct from ADR-040's file-level application of the
     same list.
  4. Whether this needs its own tooling (a `scripts/local-agent/*` gate,
     analogous to `module_split_gate.py`) or stays a manual orchestrator
     judgment call recorded in the evidence block, as ADR-040 itself started
     before its gate script existed.

## Context

`docs/policies/RRI_POLICY.md` § Decomposition triggers already mandates splitting
a task when final RRI is 56+ ("This is the default hard gate for Complex, High,
Very high, and Excessive tasks") — but its own stated split target is explicit:

> **Split target:** divide until each subtask scores RRI ≤ 55 with A ∈ {0, 1}

This target stops at the top of Med-high. Nothing in the existing decomposition
triggers evaluates whether a 41-55 task could instead be drawn so that some
resulting pieces land at ≤ 40 (Moderate, local-first under ADR-036) or ≤ 25 (Low,
Qwen-eligible) — the policy is silent on splitting *within* or *out of* Med-high
for the purpose of favoring local execution, only on splitting *down to* Med-high
from above it.

`docs/policies/HITL_AUTONOMY_POLICY.md` § Post-repair-budget Low-band
decomposition confirms the one adjacent mechanism that does decompose into
lower bands is explicitly scoped away from Med-high:

> **41–55 Med-high does not use this route** [...] Med-high has no whole-task
> repair attempt, **except** a module qualified under ADR-040 per-module split
> routing [...] A Med-high whole-task `GO_LOCAL` advisory never starts a local
> developer and never creates a whole-task local repair budget.

ADR-040 itself operates on a different axis: it fires *after* HITL approval and
phase-1 review, on an *already-scored* task, and splits **files** between a local
and a cloud tramo without changing the task's own RRI, band, reviewer, or
Reflection count. It never re-scores a piece of work into a genuinely lower band
with its own independent approval card.

The result is a real gap: a work item that is naturally separable into a
genuinely simple part and a genuinely complex part — but was drafted and scored
as one 41-55 task before any file-level split was considered — has no formal
evaluation step asking whether it should have been *authored* as multiple tasks
in different bands in the first place, only whether its files can be routed
differently after the fact.

**Worked counter-example already in the repository:** `docs/tasks/
s-150-translation-dubbing.md` already decomposes translation/dubbing work along
natural contract-vs-runtime-vs-persistence seams — S-150-T3a (provider/schema
contract), S-150-T3b (runtime integration), S-150-T3c (persistence wiring) — yet
all three independently score Med-high (RRI 41, 44, and 53 respectively per
`docs/tasks/s-230-poc-v1-digitalocean.md`'s S-230-T3b table). This shows the
mechanism proposed here is an **evaluation obligation, not a guaranteed
local-favoring outcome**: a genuine attempt at seam-finding can still conclude
that every resulting piece belongs in Med-high. The proposal is worth
formalizing anyway so that conclusion is a recorded, evidenced judgment rather
than something no one checked.

## Decision

### 1. Trigger — when to evaluate

At Step 3 (Tasks) of `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`'s mandatory
workflow, before a task card is finalized for HITL presentation, if the
provisional `scripts/rri.py` score for the drafted scope lands at **41-55**, the
orchestrator evaluates this decomposition strategy before accepting the
monolithic Med-high framing.

### 2. Evaluation procedure — identifying genuine seams

Reuse the per-variable breakdown `scripts/rri.py` already produces (C, F, D, T,
A, K, P, X). Ask whether the variable(s) driving the 41-55 score are
**concentrated** in an identifiable sub-scope or **spread uniformly** across the
whole drafted change:

- **Concentrated** (e.g., one module carries the branching complexity or
  domain sensitivity, the rest is plumbing/wiring/tests): draft a split where
  the concentrated-risk sub-scope becomes its own task — which may legitimately
  remain Med-high or higher on its own merit — and the remaining sub-scope(s)
  are drafted as independent tasks with complete, independent acceptance
  criteria.
- **Uniform** (every part of the drafted scope carries comparable complexity,
  ambiguity, or domain sensitivity): do not split. Record `no_split:
  uniform_drivers` and proceed on ADR-038's existing whole-task cloud-only
  route.

### 3. Non-gaming guardrails

Carrying ADR-040 §4's anchor-rubric-floor principle up from file level to task
level:

- A subtask inherits the D/P/K anchor-rubric floor of its own scope regardless
  of how small it is cut. Splitting a hard-excluded domain (auth/security,
  rights/consent/governance invariants, schema/migrations/release cuts,
  unresolved ADR decisions, unbounded scope — ADR-038 §6's list) into smaller
  pieces does not lower that floor; such a subtask stays Med-high (or higher)
  on its own, and this mechanism does not apply to it.
- Every resulting subtask must carry its own complete, independently testable
  `HP-#`/`EC-#` acceptance criteria per the workflow guide's Step 3 task
  requirement. A subtask that exists only to shrink file or line count, without
  an independently verifiable behavior boundary, does not qualify.
- Each subtask's RRI is computed independently via `scripts/rri.py` against its
  own drafted scope — never inherited, estimated, or assumed from the parent
  task's score.
- Inter-task sequencing and dependencies must be recorded explicitly, per the
  existing ordered task-list requirement.

### 4. Outcome — independent routing per subtask

Once split, each resulting subtask is routed entirely by its own
independently-computed band, under existing rules unchanged by this ADR:
0-25 → primary-agent/Qwen patch delegation; 26-40 → `run_local_task.py`
local-first (ADR-036); 41-55 → ADR-038's cloud-only route. Any subtask may
separately qualify for ADR-040's per-module file-split within its own band.

### 5. Termination — no recursive re-splitting

This evaluation fires once, at task-authoring time. If a resulting subtask's
own provisional RRI is still 41-55, it is **not** re-split recursively under
this mechanism — it is accepted as its own Med-high task under ADR-038 (or
uses ADR-040's per-module split if it independently qualifies). This mirrors
ADR-040 §9's "do not attempt a third tier" discipline: a bounded evaluation
depth, not an open-ended search for ever-smaller pieces. (Open decision point
#2 above asks the owner to confirm this bound.)

### 6. Relationship to existing mechanisms

| Mechanism | Fires | Splits | Can change a piece's own band? |
|---|---|---|---|
| `RRI_POLICY.md` § Decomposition triggers (56+, mandatory) | Before implementation | The task itself | Yes, down to ≤55 (not required to reach Moderate/Low) |
| **This ADR (proposed, 41-55, favored)** | Before implementation, task-authoring time | The task itself | Yes, down to ≤40 or ≤25 where a genuine seam exists |
| ADR-040 per-module split | After HITL approval + phase-1 review | Files inside one already-scored task | No — same task/band throughout |
| Moderate post-repair-budget decomposition | Mid-execution, after 2/2 local repair failures (26-40 only) | Remaining unimplemented work | Yes, to Low-band subtasks (Med-high explicitly excluded) |

### 7. Evidence block

```md
### Med-high decomposition evidence (ADR-041)

- Evaluated: yes|no — reason: <n/a, exempt category | provisional RRI 41-55>
- Driving RRI variable(s): <e.g. C=3, A=3>
- Concentration: concentrated in <sub-scope> | spread uniformly — no split
- Resulting subtasks: <task-id> — RRI <score> <band> | ...
- Guardrail check: <no hard-excluded domain fragmented | n/a>
```

## Implementation surfaces

**Not yet built.** This ADR is Proposed; no script, gate, or workflow-guide
cross-reference exists yet. If Accepted, the propagation contract in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § ADR change propagation applies:
add a subsection to `docs/policies/RRI_POLICY.md` § Decomposition triggers, a
routing-table row alongside § Local-first and Architect-refined implementation
routing in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, and the equivalent
cross-reference in `docs/policies/HITL_AUTONOMY_POLICY.md` § Med-high
Architect-refined single-attempt gate — mirroring how ADR-040 is cross-linked
in all three today.

## Consequences

**Positive**

- Unlocks local-first (cheaper, faster) authorship for genuinely separable
  low-complexity portions of what would otherwise be drafted as one monolithic
  Med-high, cloud-only task.
- Formalizes an evaluation step and an evidence trail instead of leaving
  seam-finding to ad hoc, unrecorded judgment.
- Reuses existing RRI tooling and the anchor rubric — no new complexity metric.

**Negative**

- Adds a task-authoring-time evaluation step to every 41-55 candidate.
- Residual RRI-gaming risk from cosmetic splitting, mitigated by §3 but not
  eliminated — the same residual risk ADR-040 already accepts for its own
  file-level split.
- Guaranteed-outcome risk: as the S-150-T3a/T3b/T3c precedent shows, a genuine
  evaluation can still conclude every resulting piece belongs in Med-high; the
  mechanism is an evaluation obligation, not a guaranteed local-favoring
  outcome.
- A fourth decomposition mechanism for orchestrators to keep straight
  alongside the three that already exist (hence the comparison table in §6).

**Neutral**

- Most Med-high tasks with genuinely uniform complexity are unaffected — they
  record `no_split: uniform_drivers` and proceed on ADR-038's existing route
  unchanged.

## Alternatives considered

- **Extend ADR-040 itself to cover pre-approval, task-level splitting.**
  Rejected: ADR-040's model (interface-freeze capsule, disjoint-paths
  invariant, mandatory integration gate) is built around splitting files
  inside one already-approved, already-scored task. Retrofitting it to also
  cover pre-approval work-item decomposition — different RRI per piece,
  different HITL card per piece — would conflate two structurally different
  mechanisms in one document.
- **Lower `RRI_POLICY.md`'s unconditional decomposition-trigger floor from 56
  to 41, forcing all Med-high tasks to split.** Rejected: the 56+ trigger is
  deliberately unconditional and evidence-based (`F≥4∧K≥3`, `C≥4∧D≥3`, etc.).
  Making 41-55 splitting mandatory rather than evaluated would force
  decomposition even onto genuinely uniform-complexity tasks, adding overhead
  with no complexity-arbitrage benefit — the same rejection rationale
  ADR-040 §3 already uses for its own heterogeneity-required trigger.
- **Leave it to ad hoc orchestrator judgment, without a formal ADR.**
  Rejected: this is what today's silence already permits, and it is exactly
  the ambiguity that prompted this proposal. Without a recorded evidence block
  and explicit guardrails, ad hoc splitting has no defense against RRI-gaming
  and no consistent record for future audits.

## Related

- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md` (Amendments 1
  and 2 — the cloud-only rule this ADR does not reopen for whole tasks)
- `docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`
  (adjacent, file-level, post-approval mechanism)
- `docs/policies/RRI_POLICY.md` § Decomposition triggers, § DubBridge anchor
  rubric
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Local-first and
  Architect-refined implementation routing (RRI 26-55), § Post-repair-budget
  Low-band decomposition
- `docs/policies/HITL_AUTONOMY_POLICY.md` § Med-high Architect-refined
  single-attempt gate, § Post-repair-budget Low-band decomposition
- `docs/tasks/s-150-translation-dubbing.md` (S-150-T3a/T3b/T3c — the worked
  real-world example motivating this proposal)
