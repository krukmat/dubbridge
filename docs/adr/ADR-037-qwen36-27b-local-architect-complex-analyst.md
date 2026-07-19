---
type: ADR
title: "ADR-037: Qwen3.6-27B as Local Architect and Complex Analyst"
status: Proposed
supersedes: ""
superseded_by: ""
---

# ADR-037: Qwen3.6-27B as Local Architect and Complex Analyst

- **Status:** Proposed
- **Date:** 2026-07-19
- **Deciders:** DubBridge platform team
- **Scope:** agent workflow / local advisory analysis; no application runtime,
  crate, production-data, reviewer-routing, or implementation-authority change
- **Extends:** ADR-036 local model stack with an offline batch-analysis role
- **Does not replace:** ADR-034 reviewer contract, ADR-036 implementer/reviewer
  routing, RRI gates, or human final authority

## Context

ADR-036 rejected dense `Qwen3.6-27B` as a continuous agentic implementer on the
32 GB Apple Silicon target because interactive tool loops are bandwidth-bound. It
explicitly left an offline batch-analysis role as a re-entry condition. The missing
role is not another implementer or reviewer. It is a bounded, slower, reasoning-led
surface for architecture synthesis and complex causal analysis before the primary
agent authors the repository's operative ADR, plan, and task cards.

The source proposal at `/Users/matias/Downloads/qwen3.6-27b-local-architect.md`
mixes three distinct authorities: architect, technical judge, and post-implementation
reviewer. DubBridge already assigns the latter two responsibilities to the primary
agent, Gemma Reviewer or the band-routed peer, and the human owner. Reusing those
terms would create ambiguous approval and review chains and, when Qwen3.6-35B-A3B
implements, would also mistake same-family analysis for independent review.

The model binding proposed here is not currently installed on the target machine.
The local Ollama inventory observed on 2026-07-19 contains
`qwen3.6:35b-a3b`, `gemma4:26b-a4b-it-qat`, `gemma3:27b`, and
`nomic-embed-text:latest`, but not `qwen3.6:27b-q4_K_M`. Installation, exact-tag
resolution, digest capture, and hardware measurements therefore belong to the
pilot and are not assumed facts in this decision.

## Decision

### 1. Add one advisory role: Local Architect / Complex Analyst

DubBridge will evaluate `Qwen3.6-27B` under the single role name
**Local Architect / Complex Analyst**.

The role may:

- frame an ambiguous but bounded technical problem;
- recover requirements, constraints, contracts, invariants, and dependencies from
  supplied evidence;
- compare realistic architecture or diagnosis options and their trade-offs;
- analyze difficult failures whose cause spans components or states;
- propose migration, rollback, experiment, and validation strategies;
- draft a non-operative ADR, plan outline, and implementation handoff for the
  primary agent to verify and rewrite into canonical repository artifacts.

The role may not:

- edit source code, tests, configuration, policies, ledgers, or canonical ADRs;
- run shell commands or operate a repository worktree;
- act as an implementation agent, code reviewer, task-analysis reviewer, technical
  judge, approver, coverage certifier, or owner verifier;
- replace Gemma Reviewer, the RRI 41+ cross-vendor peer, D14, the primary agent, or
  the human decision maker;
- declare a design approved, implemented, production-ready, or verified.

Its output is advisory evidence. The primary agent remains responsible for checking
repository facts, resolving conflicts with accepted ADRs, computing RRI, creating
canonical plans/tasks, and presenting any required approval gate.

### 2. Bind the pilot to a specific local model, without silent substitution

- **Requested binding:** `qwen3.6:27b-q4_K_M`
- **Runtime:** Ollama via `OLLAMA_HOST`
- **Operating context:** 8K–16K tokens by default; 24K is a bounded ceiling for
  complex cases; 32K is measurement-only stress unless the pilot promotes it.
- **Residency:** ADR-036's one-large-model-resident rule applies. Qwen3.6-27B must
  be unloaded before Qwen3.6-35B-A3B or Gemma 4 26B is loaded.

The pilot must first resolve whether the exact tag exists and record its digest,
quantization, size, and backend. Failure to resolve or load that binding blocks the
pilot. It must not silently use `gemma3:27b`, `qwen3.6:35b-a3b`, another quantization,
or a cloud model under the Local Architect label. A different binding requires a
recorded ADR amendment or an explicitly labeled comparison lane.

### 3. Activation is selective and occurs before operative planning

Invoke the Local Architect only when at least one condition is documented:

1. an ADR or cross-boundary design decision is likely;
2. two or more credible solutions have materially different trade-offs;
3. the problem spans modules, crates, workers, services, or state transitions;
4. concurrency, consistency, security, recoverability, migration, or operations are
   material to the outcome;
5. a difficult failure needs multi-cause analysis from bounded evidence;
6. ambiguous requirements must become testable contracts and invariants;
7. sensitive context should remain local;
8. a high-RRI problem needs architecture or decomposition before execution.

Do not invoke it for implementation, code review, routine test generation,
mechanical edits, broad repository exploration, current external-API verification,
or tasks whose answer is already fixed by an accepted ADR and a precise task card.

### 4. The input packet is bounded, immutable, and attributable

Every invocation receives a packet containing:

1. case ID, objective, non-goals, and explicit questions;
2. current behavior and required behavior;
3. constraints and already accepted decisions;
4. relevant ADRs and versioned repository excerpts;
5. known failures, observations, and counter-evidence;
6. expected output schema and evaluation rubric;
7. repository revision or snapshot identifier;
8. an input-manifest hash.

The caller, not the model, selects the evidence. Missing context must be reported as
`UNKNOWN` or as a stop condition. The model must not fill gaps with invented
repository or external facts.

### 5. The output is a structured analysis artifact, never an approval artifact

Every substantial result must contain:

1. decision or diagnosis summary;
2. requirements, constraints, and non-goals;
3. current-state findings;
4. candidate options and explicit trade-offs;
5. recommended option or next diagnostic experiment;
6. component responsibilities, interfaces, contracts, and invariants;
7. data/control flow where applicable;
8. failure modes, security, operations, migration, and rollback;
9. validation strategy with measurable acceptance criteria;
10. risks, unknowns, and questions requiring human or source verification;
11. non-operative ADR draft and primary-agent handoff.

Every substantive claim must be labeled as one of:

- `FACT`
- `REPOSITORY EVIDENCE`
- `ASSUMPTION`
- `RECOMMENDATION`
- `UNKNOWN`

The artifact records the model tag/digest, prompt version, runtime parameters,
input-manifest hash, start/end timestamps, and generation statistics available from
Ollama.

### 6. Preserve the existing authority and reviewer chain

```text
bounded problem + evidence
          |
          v
Qwen3.6-27B Local Architect / Complex Analyst
          |
          v
non-operative analysis artifact
          |
          v
primary agent verifies facts and authors ADR / plan / tasks
          |
          v
RRI gate + official phase-1 review + human approval when required
          |
          v
band-routed implementer -> official phase-2 review -> owner verification
```

The Local Architect is upstream of task presentation. Its artifact may inform the
primary agent but is not itself a task-analysis review. It never reviews the resulting
implementation. This preserves cross-family independence when Qwen3.6-35B-A3B is
the Moderate-band implementer.

### 7. Evaluate through bounded Medium-agent tasks

For this pilot, “Medium agent” means a bounded executor assigned a task whose final
RRI is `26–40` (`Moderate`, `Effort M`, Balanced capability), not a new authority
role. Medium agents may execute measurements, curate cases, run blinded trials, and
implement a narrow tool-free invocation wrapper. They receive one task card, fixed
allowed paths, explicit inputs, deterministic stop conditions, and named evidence
outputs. They may not promote the role, change this ADR, or reinterpret reviewer
routing.

The pilot is decomposed so every evidence-producing task is expected to remain in
the Moderate band. RRI is recomputed from the exact task card immediately before
presentation. Any task that scores `41+` must stop, be decomposed, or be rerouted
under the normal higher-band workflow.

### 8. Promotion requires runtime, artifact-quality, and workflow-utility gates

The quality rubric totals 100 points:

| Dimension | Weight |
|---|---:|
| Factual and repository-evidence fidelity | 20 |
| Requirements and constraint recovery | 15 |
| Options and trade-off quality | 15 |
| Contracts, boundaries, and invariants | 15 |
| Failure, security, operations, and rollback analysis | 15 |
| Utility of the proposed plan/handoff | 10 |
| Honest uncertainty and escalation | 5 |
| Relevance and concision | 5 |

Automatic failure occurs when an artifact invents a critical repository fact,
presents an assumption as fact, ignores a controlling accepted ADR, omits security
or rollback for a materially risky decision, recommends an out-of-scope change,
claims approval/production readiness, or relies on an external API claim without
supplied evidence.

Promotion to an optional workflow role requires all of:

- mean blind quality score `>= 80/100` across the frozen corpus;
- zero automatic failures in security-, data-, or recovery-sensitive cases;
- `>= 90%` of critical constraints recovered and zero critical invented facts;
- `>= 75%` of offline artifacts rated useful by human evaluators;
- `>= 70%` of shadow recommendations adopted or partially adopted by the primary
  agent;
- `>= 25%` reduction in median human time from problem packet to approvable plan;
- no increase in architectural decisions reopened after implementation starts;
- five repeated runtime samples complete without sustained swap growth attributable
  to the model, with median decode `>= 6 tok/s` and a typical 16K analysis completing
  within 20 minutes;
- two human evaluators score every critical case; disagreements of 15 or more points
  are adjudicated before the report is finalized.

These are pilot gates, not benchmark claims. The final report must retain raw values
and may recommend `NO-GO` or `RETEST` rather than forcing promotion.

### 9. Rollback disables only the consultative path

After promotion, disable Local Architect activation when any critical hallucination
occurs, more than 20% of a rolling 20-artifact window contradicts controlling ADRs,
usefulness falls below 60%, its guidance repeatedly reopens decisions after
implementation begins, or sustained swap/thermal degradation makes the lane
operationally harmful.

Rollback does not change ADR-036's implementer or reviewer bindings. It simply
removes this optional pre-planning analysis step and returns directly to the primary
agent's normal planning workflow.

## Consequences

### Positive

- Dense local reasoning can be used where latency is tolerable and privacy or cloud
  cost matters.
- Architecture analysis becomes attributable, reproducible, and measurable instead
  of an informal prompt exchange.
- The authority boundary prevents same-family Qwen outputs from masquerading as an
  independent review.
- Moderate agents can collect the evidence without owning the final policy decision.

### Negative

- The 27B dense model adds disk, model-residency, and evaluation costs to an already
  multi-model local stack.
- The structured packet and blind evaluation require editorial work before any
  productivity benefit is known.
- A one-shot, tool-free analyst cannot independently discover missing repository
  context; packet quality is part of the result.

### Neutral

- No application runtime or crate boundary changes.
- No automatic activation is introduced by this ADR.
- ADR-034, ADR-036, RRI approval gates, phase reviews, and owner verification remain
  unchanged.

## Alternatives considered

- **Use Qwen3.6-27B as a default implementer:** rejected; ADR-036 already records
  the hardware/latency mismatch, and implementation authority is outside this role.
- **Use it as an official reviewer or technical judge:** rejected; same-family
  independence is weak and DubBridge already has band-routed reviewers.
- **Use Qwen3.6-35B-A3B for both implementation and architecture:** retained as a
  comparison lane, not assumed equivalent; the pilot must test whether the dense
  model's slower analysis creates measurable quality or planning benefit.
- **Adopt directly without a pilot:** rejected; the exact model is not installed and
  no repository-specific quality evidence exists.
- **Never add a dense local role:** retained as the `NO-GO` outcome if the runtime or
  utility gates fail.

## Implementation references

- `docs/plan/adr037-local-architect-pilot.md`
- `docs/tasks/adr037-local-architect-pilot.md`

