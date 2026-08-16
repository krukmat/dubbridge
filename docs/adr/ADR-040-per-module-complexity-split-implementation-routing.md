---
type: ADR
title: "ADR-040: Per-module complexity-split implementation routing (RRI 26-55)"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-040: Per-module Complexity-split Implementation Routing (RRI 26-55)

- **Status:** Accepted
- **Date:** 2026-08-16
- **Deciders:** DubBridge owner and platform-agent workflow maintainers
- **Scope:** agent workflow and local delegation only; no application-runtime or
  product-architecture change
- **Amends:** ADR-036 (local-first implementation band — adds a per-module
  routing granularity inside RRI 26-40) and ADR-038 (Amendment 2 — qualifies
  the "no local attempt or repair at this band" rule for RRI 41-55 with a
  narrow, evidence-gated exception)
- **Owner approval:** the owner confirmed, across an interactive session on
  2026-08-16, the following design parameters in order: (1) the mechanism
  applies to both Moderate (26-40) and Med-high (41-55), explicitly accepting
  that this requires qualifying ADR-038's "no local attempt" rule for
  Med-high; (2) the per-module complexity threshold reuses the existing RRI
  `C` variable table rather than a new metric; (3) local tramo repair budget
  is 2 attempts, cloud tramo repair budget is 1 attempt with one escalation
  to the band's higher cloud tier before stopping; (4) an integration-gate
  failure attributable to the interface contract itself (not to either
  tramo's authored code) abandons the split and escalates the entire task to
  the band's normal whole-task route, rather than retrying the split with a
  revised contract.

## Context

ADR-036 routes an entire RRI 26-40 development task to a single local
implementer (`qwen3.6:35b-a3b` via `scripts/local-agent/run_local_task.py`).
ADR-038 routes an entire RRI 41-55 development task to cloud, explicitly
because "the ambiguity, blast radius, and verification burden... are
precisely what raise the band" — Amendment 1 (2026-08-12) hardens this to
"no local attempt or repair at this band" and policy-excludes even a
`GO_LOCAL` refinement result from starting a local developer.

Both routes treat the approved task as one atomic authoring unit. In
practice, a single approved task frequently spans multiple files with
heterogeneous internal complexity — e.g. a task that adds a new field
threaded through three thin plumbing files (getters/setters, a struct field,
a serialization arm) and one file containing the actual branching validation
logic. Routing the whole task to the same implementer either wastes cloud
capacity on trivial modules or exposes the local implementer to the one
module that actually carries the task's risk.

The owner proposed splitting the approved task's file set by measured
per-module cyclomatic complexity (CC): route low-CC modules to the local
implementer and high-CC modules to cloud, within the same approved task. This
ADR formalizes that mechanism, including for Med-high, where it necessarily
qualifies ADR-038's blanket exclusion — see Amendment 2 to ADR-038 for the
explicit textual change to that ADR.

## Decision

### 1. Eligibility

The mechanism applies only to a development task that:

- has an approved (HITL) task card with final RRI in **26-55** (Moderate or
  Med-high);
- has already passed phase-1 task-analysis review (Gemma, per the existing
  26-55 binding — unchanged by this ADR);
- has an `allowed_paths` set spanning **two or more distinct files**.

A single-file task is never split; it follows the existing whole-task route
for its band (ADR-036 for Moderate, ADR-038 for Med-high) unchanged.

### 2. Per-module complexity measurement

Before dispatch, measure raw cyclomatic complexity per file using the same
mechanism `scripts/rri.py --auto-cc` already uses (`cargo clippy` cognitive
complexity for Rust, or manual CC counting per `docs/policies/RRI_POLICY.md`).
For a file, take the **highest** CC among the functions to be created or
materially changed in that file. Map the raw value to the existing RRI `C`
score using the unmodified table in `docs/policies/RRI_POLICY.md` § Variables
(C = 0 for CC 1-5, up to C = 5 for CC 50+). No new complexity metric is
introduced.

### 3. Split trigger — heterogeneity required

Split the task's implementation only if the per-module C scores are
**heterogeneous**: at least one module scores **C ≥ 2** (raw CC ≥ 11) and at
least one module scores **C ≤ 1** (raw CC ≤ 10). If every module falls in the
same tier, do not split — route the whole task per the existing band rule.
Splitting a uniformly-simple or uniformly-complex task adds integration
overhead with no complexity-arbitrage benefit.

### 4. Hard domain exclusion (carried from ADR-038 §6)

Regardless of its own CC score, any module is treated as **cloud-eligible**
(never local-eligible) if its path matches an anchor-rubric floor of D, P, or
K ≥ 4 in `docs/policies/RRI_POLICY.md` § DubBridge anchor rubric — this
includes auth/security paths, rights/consent/governance-invariant paths,
schema/migration paths, and any path ADR-038 §6 already hard-excludes from
`GO_LOCAL`. Cyclomatic complexity measures branching, not domain sensitivity;
a trivially low-CC function inside a hard-excluded path (e.g. a one-line
auth-token comparison) still carries the domain's risk profile and must not
be routed local merely because it is syntactically simple. This preserves
ADR-038's original rationale for Med-high even where this ADR now permits a
local tramo elsewhere in the same task.

### 5. Disjoint-paths invariant

The two tramos' `allowed_paths` **must partition the task's file set with no
overlap**. If a clean partition is not possible — e.g. a shared file needs
edits that logically belong to both a low-CC and a high-CC concern — the task
is **not split**; it falls back to the existing whole-task route for its
band. This mechanism never attempts to merge two implementers' edits to the
same file.

### 6. Interface freeze before dispatch

Before either tramo is dispatched, the primary agent (orchestrator of
record) records the exact interface contract at the boundary between the
local-eligible and cloud-eligible modules — function signatures, shared
types, error contracts — in a **module-split capsule**. Neither implementer
may redefine this contract; each authors only its own side against the
frozen boundary. This is the same "freeze before dispatch" discipline
ADR-038 already uses for its task capsule, applied here to the narrower
inter-module boundary instead of the whole task.

### 7. Routing

| Module class | Route |
|---|---|
| C ≤ 1, no hard exclusion | Local: `scripts/local-agent/run_local_task.py`, `DUBBRIDGE_LOCAL_AGENT_MODEL` (default `qwen3.6:35b-a3b`), `allowed_paths` restricted to exactly that module's files |
| C ≥ 2, or hard-excluded | Cloud: the band's already-resolved cloud model (Moderate: `gpt-5.6-terra` at `medium`, or `claude-sonnet-5`; Med-high: `gpt-5.6-sol` at `high`, or `claude-sonnet-5`) |

### 8. Repair budgets

- **Local tramo:** **2** evidence-backed repair attempts — the same number
  ADR-036 already grants a whole Moderate task. This applies uniformly
  whether the overall task's band is Moderate or Med-high, because a module
  that qualified for the local tramo has already been verified as low-CC and
  domain-non-sensitive; its risk profile is a Moderate module's, not a
  Med-high module's, regardless of the containing task's overall band.
- **Cloud tramo:** **1** repair attempt at the initially resolved model/tier.
  If that attempt fails, escalate **once** to the band's higher cloud tier
  before stopping:
  - Moderate: Codex `gpt-5.6-terra`/`medium` → `gpt-5.6-sol`/`high`; Claude
    Code `claude-sonnet-5` → `claude-opus-5`. The Claude Code escalation is a
    **narrow, scoped exception** to the Moderate row of the capability table
    in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Current Claude Code
    capability resolution (that row otherwise states no in-band escalation,
    "stays on Sonnet 5"). The exception applies **only** to a module-split
    cloud tramo's repair-exhaustion escalation, not to whole-task Moderate
    implementation.
  - Med-high: Codex `gpt-5.6-sol`/`high` → `gpt-5.6-sol`/`xhigh` (same model,
    higher reasoning effort); Claude Code `claude-sonnet-5` →
    `claude-opus-5` (this reuses the escalation the Med-high row already
    defines — no exception needed here).
  - If the escalated attempt also fails, **stop and report blocked** for
    that module. Do not attempt a third tier or model.

### 9. Integration gate (mandatory, whole-task, post-both-tramos)

After both tramos finish (or after only one, if the task did not split),
before Reflection, run the task's full verification commands (at minimum
`cargo check` / `cargo test` across the unified diff, plus any
operator-authored acceptance commands) against the **merged** diff from both
tramos.

- A failure attributable to one tramo's authored code is a bounded repair
  charged against that tramo's own budget (§8) — it does not create a new,
  separate counter.
- A failure attributable to the **interface contract itself** (a mismatch
  neither tramo's code alone could have avoided, because the frozen contract
  from §6 was wrong) **abandons the split**: escalate the entire original
  task to the band's normal whole-task route (ADR-036 for Moderate, ADR-038
  for Med-high) as if no split had been attempted. Record
  `split_abandoned: contract_mismatch` in the evidence. The split is not
  retried with a revised contract — a wrong boundary contract is an
  orchestrator planning failure, and re-attempting the same decomposition is
  more likely to repeat it than to fix it.

### 10. Review, Reflection, and closure remain whole-task

The band-resolved phase-1/phase-2 reviewer (Gemma, per the existing 26-55
binding — unchanged), the Reflection pass count for the task's overall band
(2 for Moderate, 3 for Med-high), unit coverage certification, and owner
final verification all evaluate the **final unified diff as one task**. This
mechanism changes only who authors which files, never who reviews or
approves the result — the same principle ADR-038 already states for its own
cloud-takeover routing.

### 11. Evidence

The task closure record adds a `### Module-split routing evidence` block:

```md
### Module-split routing evidence

- Split triggered: yes|no — reason: <heterogeneous CC | uniform CC, no split | disjoint-paths violation, no split>
- Modules: <file> — C=<score> (CC=<raw>) — <local|cloud> — <reason, incl. hard-exclusion if applicable>
- Interface contract: <link to module-split capsule, or n/a if not split>
- Local tramo repair attempts used: <0-2>
- Cloud tramo repair attempts used / escalated: <0-1> / <yes, to <tier>|no>
- Integration gate: <PASS | repaired (tramo: <local|cloud>) | split_abandoned: contract_mismatch>
```

## Implementation surfaces

- **Built:** `scripts/local-agent/module_split_gate.py` (tested in
  `module_split_gate_test.py`, 29 cases covering HP-1/HP-2/EC-1..EC-4 plus
  additional boundary cases discovered during Reflection). `evaluate_split()`
  takes a task capsule (`allowed_paths`, `cc_by_path`), reuses
  `scripts/rri.py`'s `cc_to_score()` and DubBridge anchor rubric
  (`RubricRow`/`first_matching_row`) for the §4 hard-exclusion check, and
  returns a fail-closed `no_split` (with typed reason) or `split` (local/cloud
  path partition + initial repair budgets) decision — structurally identical
  in shape to `scripts/local-agent/med_high_gate.py`. `next_cloud_action()`
  implements the §8 cloud-tramo repair-budget state machine. Built directly
  by the primary agent under the ADR-038 Med-high cloud-only route (Amendment
  1 policy-excludes local implementation for this task itself; see the task
  closure record at `docs/tasks/module-split-gate-tooling.md` for the full
  Muse Glimmer refinement / primary receipt evidence trail).
- **Not yet built:** a module-split capsule format for the §6 interface
  freeze (analogous to the existing ADR-038 task capsule), and the
  `run_local_task.py` / `run_med_high_task.py` integration wiring that
  actually dispatches the two tramos using the gate's decision. Until these
  exist, an orchestrator using this route invokes `module_split_gate.py` for
  the split decision itself, but still records the interface-freeze capsule
  and dispatches both tramos manually — the evidence block should say so
  rather than claiming full automated-gate enforcement end-to-end.

## Consequences

**Positive**

- Avoids spending cloud capacity on trivial plumbing modules inside an
  otherwise complex task, and avoids exposing the local implementer to the
  one module that actually carries a task's risk.
- Reuses existing metrics (RRI `C` table, anchor rubric) and existing
  implementers (`run_local_task.py`, the band's already-resolved cloud
  model) — no new complexity metric or new model binding.
- Keeps review, Reflection, coverage, and closure whole-task, so the
  band-resolved reviewer's independence and the HITL approval gate are
  unaffected by how authorship was split.

**Negative**

- Introduces integration risk (interface-contract mismatch) that a
  single-implementer route does not have; mitigated by the §6 freeze and the
  §9 mandatory integration gate, but not eliminated.
- Qualifies ADR-038's "no local attempt or repair at this band" rule for
  Med-high, which was adopted deliberately and recently (Amendment 1,
  2026-08-12). This ADR narrows that exception to modules independently
  verified as low-CC and domain-non-sensitive; it does not reopen general
  Med-high local implementation.
- Adds a second repair-budget shape (2 local / 1 cloud-plus-one-escalation)
  and a new mandatory gate (integration) to track per task, increasing
  process overhead for tasks that do split.

**Neutral**

- Most Moderate and Med-high tasks (single-file, or uniform per-module CC)
  are unaffected — the split trigger in §3 is not met and they continue on
  their existing ADR-036 / ADR-038 whole-task route.

## Alternatives considered

- **Split by file count instead of CC.** Rejected: file count does not
  correlate with the risk this workflow is trying to route around;
  cyclomatic complexity is the metric the RRI policy already treats as the
  proxy for reasoning burden.
- **Recompute a per-module RRI instead of reusing the raw `C` table.**
  Rejected as unnecessary overhead: a per-module RRI would require
  re-deriving F/D/T/A/K/P/X for a sub-file unit, none of which are
  well-defined below whole-task granularity, whereas C is already
  file/function-scoped.
- **Let a wrong interface contract be repaired in place instead of
  abandoning the split.** Rejected: the contract is an orchestrator-owned
  planning artifact, not either tramo's authored code; repairing it in place
  risks the same planning error recurring, and the whole-task fallback route
  already exists and is well-understood.
- **Give the cloud tramo the same 2-attempt budget as local.** Rejected by
  the owner in favor of 1 attempt plus one tier escalation — cloud attempts
  are more expensive per attempt, and a single well-specified attempt at an
  appropriately-sized tier was judged sufficient before escalating.

## Related

- `ADR-036-local-first-agentic-implementation-band.md`
- `ADR-038-med-high-architect-refined-single-attempt.md` (see Amendment 2)
- `docs/policies/RRI_POLICY.md` § Variables (C table), § DubBridge anchor
  rubric, § Decomposition triggers
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Local-first and
  Architect-refined implementation routing (RRI 26-55)
- `docs/policies/HITL_AUTONOMY_POLICY.md` § Local-first implementation (RRI
  26-40 Moderate), § Med-high Architect-refined single-attempt gate (RRI
  41-55)
