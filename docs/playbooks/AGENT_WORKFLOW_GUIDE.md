# Agent Workflow Guide

> **Status:** Scaffold. This guide consolidates the mandatory workflow already
> defined in the project and global `CLAUDE.md`. It exists to resolve the dangling
> reference in `AGENTS.md`. Keep it in sync with `CLAUDE.md`; `CLAUDE.md` wins on
> conflict.

## Mandatory workflow before implementing

1. **Analyze** — read context, dependencies, and affected files.
2. **Plan** — create `docs/plan/<plan-name>.md` with: objective, affected files,
   design decisions, and module dependencies.
3. **Tasks** — create `docs/tasks/<tasks-name>.md` with: an ordered task list,
   inter-task dependencies, acceptance criteria per task, an **Effort** field
   (S/M/L/XL), and a short agent handoff prompt.
4. **Present and wait** — show the plan and tasks and wait for explicit approval
   before starting implementation, even if a plan was approved in a prior session.
5. **Implement** — one task at a time, in the defined order.
6. **Mark progress** — update the tasks document after each completed task (it is
   the crash-safe progress ledger).
7. **Sync status artifacts before reporting completion** — before telling the user
   a task is done, update every materially affected status document in the same
   workflow pass. Completion is not valid until those documents are consistent.

## Per-task discipline

- Present the next task using the `AGENTS.md` presentation contract before executing
  it when approval is required.
- After each task: verify the relevant tests/checks, update the status docs,
  document deviations or evidence, and state unresolved risks or blockers.
- Treat status-document synchronization as part of the task itself, not follow-up
  cleanup. Do not report a task complete while any governing status document still
  shows stale state.
- When a task completion changes the status of a slice, dependency, ADR, or blocked
  downstream task, update all materially affected status documents in the same
  workflow before reporting completion. This includes, as applicable:
  `docs/tasks/*`, `docs/plan/roadmap.md`, linked slice plans, dependent task files,
  and ADR status/implementation references.
- At minimum, check whether the completed task changes any of:
  `docs/tasks/*`, `docs/plan/roadmap.md`, the linked `docs/plan/*` slice file,
  dependent task ledgers, ADR status/implementation references, and any handoff
  prompt or blocking-gate language that names the completed work.
- When an ADR is created, amended, or deleted as part of a task, apply the
  **ADR change propagation** contract below in the same workflow pass.
- Work on the approved task only; show a summary before switching to the next.

## ADR change propagation

An ADR change that occurs outside a task ledger (e.g. a replan, a hotfix, or a
cross-cutting amendment) is still subject to this contract. Apply the matching row
in the same change — not as a follow-up.

| ADR change | Must review and update in the same change |
|---|---|
| **New ADR** | `docs/adr/README.md` index row; `docs/architecture.md` if it adds or alters a runtime/crate boundary; `docs/plan/roadmap.md` if it changes slice scope or dependencies; the affected `docs/plan/*` and `docs/tasks/*` files |
| **Status change** (`Proposed` → `Accepted` → `Superseded` / `Deprecated`) | index `Status` column; every canonical doc (`architecture.md`, `roadmap.md`, plan/tasks) that cites the ADR as authority for a decision |
| **Scope narrowed or broadened** | index scope annotation; `docs/architecture.md`; `docs/plan/roadmap.md`; affected plan/tasks; `README.md` if the change is outward-facing |
| **Content / decision change** (the decision itself, not just status or scope) | every canonical doc whose prose describes that decision — **this is semantic and not machine-verifiable**; Layer 2/3 confirm references still resolve, but human review owns whether the prose is still accurate |
| **Superseded by ADR-YYY** | both ADRs' `Status` field; the index row for each; every doc citing the superseded ADR |
| **Deletion or renumbering** | see the deletion rule below; update the index, every doc citation, **and every code/migration comment** (`.rs`, `.sql`) in the same change |

**Deletion rule.** An `Accepted` ADR is part of the auditable decision record and
must **not be deleted** — mark it `Superseded by ADR-YYY` or `Deprecated` instead.
A `Proposed` ADR that was never adopted may be deleted only after every reference
(docs *and* code/migration comments) is removed in the same change. Renumbering is
a delete + create and must update all references atomically.

**Definition of done for any ADR change:**
- [ ] The ADR file's `Status` field is updated.
- [ ] `docs/adr/README.md` index row matches (status token + title).
- [ ] Every doc in the matching propagation row above has been reviewed and updated
      if its content describes the changed decision.
- [ ] No code or migration comment cites a missing ADR number.
- [ ] `make qa-docs` passes (index parity, completeness, dangling refs in docs and
      code/migrations, superseded-successor existence).

### What this contract does and does not guarantee

**Guaranteed by `make qa-docs` (deterministic, Layers 2/3):**
- Every cited ADR file exists.
- Index↔file status tokens agree.
- The index is complete (no file without a row, no row without a file).
- A `Superseded` ADR names an existing successor.
- No code or migration comment cites a missing ADR.

**Not guaranteed (Layer 1 + human review only):**
- That the *prose* of a canonical doc still accurately describes an ADR whose
  decision changed. Referential integrity is automatable; semantic consistency is
  not. The propagation table tells the author *which prose to re-read*; it does not
  prove the update was made correctly.

## Effort scale

| Level | Agent reasoning | Example |
|-------|-----------------|---------|
| S  | Mechanical — transcription, copy, merge | Config files from an explicit spec |
| M  | Moderate — contracts, logic, edge cases | Boundary tests; small services |
| L  | High — multiple subsystems, architecture | Process supervisor with replay tests |
| XL | Very high — unpredictable external tooling | Native toolchain conflict resolution |

## Model and thinking-mode selection

This section is the canonical source for complexity scoring, model-tier
selection, and thinking-mode guidance. `AGENTS.md` defines the presentation
fields, but agents must derive the values from this guide rather than from
agent-specific defaults.

Concrete vendor model IDs change over time. Agents must therefore separate:

1. the **capability decision** (`Economy`, `Balanced`, `Premium`) derived from
   the formulas in this guide, from
2. the **concrete model resolution** (the current OpenAI / Anthropic model ID
   that best fits that capability at the time of presentation).

Do not collapse these into one undocumented guess.

When presenting a task, the agent must compute a complexity score and derive the
recommended model tier from it. Do not guess; use the procedure below.

### Step 1 — Compute complexity

**For development tasks (code to write or modify):**

Compute the **cyclomatic complexity** (McCabe, 1976) of the functions that will be
created or materially changed:

```
CC = E − N + 2P
```

where E = edges, N = nodes, P = connected components in the control-flow graph.
Practically: start at 1 and add 1 for each `if`, `else if`, `match` arm, `while`,
`for`, `loop`, `?` propagation that branches, `&&`, `||` in a condition.

| CC range | Complexity label |
|---|---|
| 1–5 | Low |
| 6–10 | Medium |
| 11–20 | High |
| > 20 | Very High |

**For non-development tasks (analysis, planning, research, config, docs):**

Use the **decision-weight heuristic** — count the number of irreversible decisions
plus external dependencies the task requires:

| Score | Complexity label |
|---|---|
| 0–2 | Low |
| 3–5 | Medium |
| 6–9 | High |
| ≥ 10 | Very High |

Irreversible decisions include: schema changes, public API changes, CI gate changes,
deletion of authoritative files, policy changes. External dependencies include: live
DB, external APIs, CLI tools with version-sensitive behavior, network-bound ops.

### Step 2 — Map to model tier (cost / capability balance)

Prefer capability tiers over pinned model IDs in this guide. Model names change
over time; the workflow should stay stable across agents and providers.

| Tier | Best for |
|---|---|
| Economy | Low-complexity, mechanical tasks |
| Balanced | Medium-complexity, standard implementation work |
| Premium | High / Very High complexity, architecture, synthesis, deep debugging |

Mapping:

| Complexity | Codex capability | Claude Code capability | Thinking mode |
|---|---|---|---|
| Low | Economy coding model | Economy coding model | Off |
| Medium | Balanced coding model | Balanced coding model | Off |
| High | Premium reasoning/coding model | Balanced or Premium reasoning/coding model | **On** |
| Very High | Premium reasoning/coding model | Premium reasoning/coding model | **On** |

Agent-specific resolution rules:

- Resolve each capability label to the best currently available model in the
  active agent environment.
- When naming a concrete vendor model ID, verify the current vendor guidance
  first if there is any reasonable chance the recommendation has changed. Do not
  rely on stale memory for "latest", "best", "recommended", or similar claims.
- For OpenAI recommendations, prefer official OpenAI documentation. For Claude /
  Claude Code recommendations, prefer official Anthropic documentation.
- The final recommendation must be produced in this order:
  1. compute complexity with the formula in Step 1
  2. map complexity to capability tier with Step 2
  3. resolve that tier to the best current vendor model
  4. present the resolved model and note any task-local override
- `Effort` may inform cost discipline and escalation judgment, but it does not
  replace the complexity formula. If `Effort` and computed complexity pull in
  different directions, compute complexity first and then add a one-line
  rationale for the final recommendation.
- If a task file explicitly pins a model, that task-local guidance overrides the
  default tier mapping.
- If a task file pins a model that appears stale relative to current vendor
  guidance, do not silently swap it during task presentation. Either:
  - present the pinned model as the task-local override, or
  - update the task metadata explicitly in an approved documentation change.
- If the user asks for the latest or most recent model, verify against official
  provider documentation before naming a specific model.
- Do not silently replace a task-local pinned model with a newer one. Either use
  the pinned model or update the task metadata explicitly.

**Thinking mode** for the selected balanced/premium reasoning model:
activate when the task requires multi-step reasoning that cannot be validated
incrementally — e.g., architecture trade-offs with more than two interacting
constraints, novel algorithmic design, or diagnosis of non-deterministic failures.
Do **not** activate for: writing tests for already-specified logic, config edits,
doc updates, or any task where the strategy is fully pre-defined.

### Step 3 — State it in the task presentation

Include in every task presentation:

```
| Complexity score | <CC range or decision-weight score> → <label> |
| Claude Code      | <resolved model or pinned model> — thinking <On / Off> |
| Codex            | <resolved model or pinned model>                        |
```

The recommendation is **not** a competition between vendors. Every presentation
must provide:

- one concrete current recommendation for OpenAI / Codex
- one concrete current recommendation for Claude Code / Anthropic

Both recommendations must be derived from the same computed complexity and the
same tier-mapping rules in this guide. Do not present only one vendor unless the
task file explicitly scopes the task to a single vendor environment.

Presentation rules:

- Always show the computed `Complexity score`, even if the task file already
  declares `Complexity:`.
- If the task file provides explicit complexity or model guidance, state that it
  is a task-local override when presenting the task.
- If the presentation uses a resolved model from the current agent environment,
  prefer the actual resolved model identifier over a generic tier label.
- When a concrete model identifier is presented as "recommended", it must be
  traceable either to:
  - current official vendor guidance, or
  - a task-local explicit pin documented in the task file.
- Add a one-line rationale if the mapping is non-obvious (e.g., a Medium CC task
  escalated to High because of a Very High external-dependency count).

## Testing and commit rules

- TDD where practical: test first, implement, run tests.
- Target at least **90% line coverage** for the implemented scope. Treat coverage
  as an enforced quality gate, not a reporting-only metric.
- Prefer real backends over mocks; features should talk to the real backend.
- **Do not commit if any test is broken.** Run all tests before commit and push.
- Keep the automated coverage gate aligned with CI configuration. If the required
  threshold changes, update both the workflow guide and `.github/workflows/ci.yml`
  in the same change.
- Mirror critical QA gates locally before changes reach the remote. The repository
  pre-push hook at `.githooks/pre-push` should enforce the fast deterministic Rust
  gates (`fmt`, `clippy`, `test`, `cargo check`) and run dependency-policy checks
  when Cargo manifests change. CI keeps the full blocking baseline, including the
  90% coverage gate. Enable the hook with `git config core.hooksPath .githooks`.
- Ask for confirmation before deleting anything.

## Handoff prompt format

Keep handoff prompts minimal. The task was already presented and approved — do not re-explain it.

A handoff prompt must contain only:

1. Task ID + one-line goal
2. Governing docs (task file + plan file, paths only)
3. The one file + line range with the logic to change
4. Exact acceptance criteria (bullets only, no prose)
5. Stop condition: what the agent must do last and must NOT start next

## Language

- User-facing communication: Spanish.
- Plans, task documents, prompts, ADRs, and code/comments: precise technical English.

## Related

- `CLAUDE.md`, `AGENTS.md`, `README_AGENT_ORDER.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
