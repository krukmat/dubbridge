---
type: ADR
title: "ADR-037: Qwen3.6-27B as Local Architect and Complex Analyst"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-037: Qwen3.6-27B as Local Architect and Complex Analyst

- **Status:** Accepted
- **Date:** 2026-07-19
- **Deciders:** DubBridge platform team
- **Scope:** direct project advisory analysis before operative planning; no
  application runtime, crate, production-data, reviewer-routing, or
  implementation-authority change
- **Extends:** ADR-036 local model stack with a bounded project-analysis role
- **Does not replace:** ADR-034 reviewer contract, ADR-036 implementer/reviewer
  routing, RRI gates, or human final authority
- **Implementation references:**
  - `docs/plan/adr037-local-architect-direct-project.md`
  - `docs/tasks/adr037-local-architect-direct-project.md`

## Context

ADR-036 rejected dense `Qwen3.6-27B` as a continuous agentic implementer on the
32 GB Apple Silicon target because interactive tool loops are bandwidth-bound. It
explicitly left a slower batch-analysis role as a re-entry condition. The missing
role is not another implementer or reviewer. It is a bounded, reasoning-led surface
for architecture synthesis and complex causal analysis before the primary agent
authors the repository's operative ADR, plan, and task cards.

The source proposal at `/Users/matias/Downloads/qwen3.6-27b-local-architect.md`
mixes three distinct authorities: architect, technical judge, and
post-implementation reviewer. DubBridge already assigns the latter two
responsibilities to the primary agent, Gemma Reviewer or the band-routed peer, and
the human owner. Reusing those terms would create ambiguous approval and review
chains and, when Qwen3.6-35B-A3B implements, would also mistake same-family analysis
for independent review.

The model binding proposed here is not currently installed on the target machine.
The local Ollama inventory observed on 2026-07-19 contains `qwen3.6:35b-a3b`,
`gemma4:26b-a4b-it-qat`, `gemma3:27b`, and `nomic-embed-text:latest`, but not
`qwen3.6:27b-q4_K_M`. Installation, exact-tag resolution, digest capture, and
hardware measurements therefore require separately approved tasks and are not
assumed facts in this decision.

The owner rejected an offline/historical pilot-first path on 2026-07-19 and selected
direct use on real DubBridge work instead. This ADR therefore accepts direct project
evaluation while preserving the model as a non-operative advisory role.

## Decision

### 1. Add one advisory role: Local Architect / Complex Analyst

DubBridge accepts `Qwen3.6-27B` under the single role name **Local Architect /
Complex Analyst**.

The role may:

- frame an ambiguous but bounded technical problem;
- recover requirements, constraints, contracts, invariants, and dependencies from
  supplied evidence;
- compare realistic architecture or diagnosis options and their trade-offs;
- analyze difficult failures whose cause spans components or states;
- propose migration, rollback, experiment, and validation strategies;
- draft a non-operative ADR outline, plan outline, and implementation handoff for
  the primary agent to verify and rewrite into canonical repository artifacts.

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

### 2. Bind the role to a specific local model, without silent substitution

- **Requested binding:** `qwen3.6:27b-q4_K_M`
- **Runtime:** Ollama via `OLLAMA_HOST`
- **Operating context:** 8K-16K tokens by default; 24K is a bounded ceiling for
  complex cases; 32K is stress-measurement only unless later evidence amends this
  ADR.
- **Residency:** ADR-036's one-large-model-resident rule applies. Qwen3.6-27B must
  be unloaded before Qwen3.6-35B-A3B or Gemma 4 26B is loaded.

The first operational task must resolve whether the exact tag exists and record its
digest, quantization, size, and backend. Failure to resolve or load that binding
blocks the Local Architect lane. The workflow must not silently use `gemma3:27b`,
`qwen3.6:35b-a3b`, another quantization, or a cloud model under the Local Architect
label. A different binding requires a recorded ADR amendment or an explicitly labeled
comparison lane.

### 3. Use it directly on real project work

The Local Architect is invoked only for a concrete work item, before operative
planning. The default first work item is `S-140` because `S-130` is complete locally
and `S-140` is the next natural consumer in the roadmap. The owner may choose a
different eligible roadmap item before the packet is frozen.

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

1. work item ID, objective, non-goals, and explicit questions;
2. current behavior and required behavior;
3. constraints and already accepted decisions;
4. relevant ADRs and versioned repository excerpts;
5. known failures, observations, and counter-evidence;
6. expected output schema and evaluation criteria;
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
11. non-operative ADR outline and primary-agent handoff.

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
real work item + bounded evidence
          |
          v
Qwen3.6-27B Local Architect / Complex Analyst
          |
          v
non-operative analysis artifact
          |
          v
primary agent verifies facts and authors target ADR / plan / tasks
          |
          v
RRI gate + official phase-1 review + human approval when required
          |
          v
band-routed implementer -> official phase-2 review -> owner verification
```

The Local Architect is upstream of task presentation. Its artifact may inform the
primary agent but is not itself a task-analysis review. It never reviews the
resulting implementation. This preserves cross-family independence when
Qwen3.6-35B-A3B is the Moderate-band implementer.

### 7. Evaluate through bounded Medium-agent tasks

For this direct-use lane, "Medium agent" means a bounded executor assigned a task
whose final RRI is `26-40` (`Moderate`, `Effort M`, Balanced capability), not a new
authority role. Medium agents may resolve the model, implement the one-shot wrapper,
freeze a project packet, execute the analysis, and record outcome evidence. They
receive one task card, fixed allowed paths, explicit inputs, deterministic stop
conditions, and named evidence outputs.

Medium agents may not accept architecture, change this ADR, reinterpret reviewer
routing, author canonical target decisions from the model output, or start downstream
implementation. Any task that scores `41+` must stop, be decomposed, or be rerouted
under the normal higher-band workflow.

### 8. Operational criteria replace pilot promotion gates

This ADR does not require an offline corpus, blind comparison, shadow phase, or
promotion gate. Each direct project use is evaluated against operational criteria:

- exact model identity and packet hash are recorded;
- zero critical invented repository facts;
- zero conflicts with controlling accepted ADRs;
- no claim of implementation, review, approval, or production-readiness authority;
- critical constraints are recovered, with a target of at least 90%;
- at least one material recommendation is accepted or the artifact usefully confirms
  an existing design by surfacing risks, invariants, or validation criteria;
- rejected or partial recommendations are recorded with reasons;
- runtime remains acceptable for the selected work item, with 16K analysis normally
  expected to complete within about 20 minutes and median decode near or above
  6 tok/s when measured.

These are operating health criteria, not benchmark claims. A single artifact can be
invalidated without reverting the ADR. Repeated poor utility or hardware pressure can
disable the lane until correction or retest.

### 9. Circuit breaker disables only the consultative lane

Disable the Local Architect lane for the affected work item when any artifact:

- invents a critical repository fact;
- presents an assumption as fact;
- ignores or contradicts a controlling accepted ADR;
- omits security, recovery, rollback, or migration analysis where material;
- recommends an out-of-scope change;
- claims approval, implementation, review, certification, or production readiness;
- relies on an external API or current external fact without supplied evidence.

The primary agent records the incident, ignores the artifact for canonical planning,
and continues through the normal workflow without Local Architect input. Re-enabling
the lane requires a bounded correction or retest task. This does not change ADR-036's
implementer or reviewer bindings.

## Consequences

### Positive

- Dense local reasoning is used where latency is tolerable and privacy or cloud cost
  matters.
- Architecture analysis becomes attributable, reproducible, and tied to real project
  outcomes.
- The authority boundary prevents same-family Qwen outputs from masquerading as an
  independent review.
- Medium agents can collect operational evidence without owning the final policy or
  architecture decision.

### Negative

- Direct use spends project time before offline quality is known.
- The 27B dense model adds disk, model-residency, and local hardware pressure.
- The primary agent must verify every adopted claim, so the artifact can improve
  thinking but cannot remove review or approval work.
- A bad artifact may be useless for a work item and must be discarded without
  blocking the normal workflow.

### Neutral / governance

- No product architecture, runtime service, crate boundary, or production-data flow
  changes in this ADR.
- No reviewer-routing change. ADR-034 and ADR-036 remain controlling for official
  review and implementation paths.
- The direct-use ledger is evidence-gathering and orchestration guidance, not a new
  authority hierarchy.

## Alternatives considered

### Keep ADR-036 only

Rejected. ADR-036 leaves dense 27B reasoning out of the stack even for slow,
bounded, local architecture synthesis where tool-loop latency is less important.

### Historical/offline pilot first

Rejected by owner direction on 2026-07-19. The project will evaluate usefulness on
real DubBridge work instead, with primary verification and circuit breakers
preserving authority boundaries.

### Use Qwen3.6-27B as implementer

Rejected by ADR-036. Dense 27B is too slow for continuous local implementation loops
on the target hardware and would duplicate the Qwen3.6-35B-A3B implementer lane.

### Use Qwen3.6-27B as reviewer or technical judge

Rejected. It would blur ADR-034 and ADR-036 reviewer routing and would not provide
the desired independence when Qwen3.6-35B-A3B implements.

### Use cloud frontier models for this role

Not adopted by this ADR. Cloud models may still be used through the normal primary
agent workflow where policy allows, but the purpose here is a local, private,
bounded advisory lane.

## Follow-up tasks

Follow-up tasks are tracked in
`docs/tasks/adr037-local-architect-direct-project.md`.
