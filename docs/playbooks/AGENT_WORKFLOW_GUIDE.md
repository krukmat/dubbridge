---
type: Playbook
title: "Agent Workflow Guide"
governs: "all agent-facing workflow decisions in the repository"
---

# Agent Workflow Guide

> **Status:** Authoritative. This guide is the highest-authority source for **all**
> agent-facing decisions: workflow, process, implementation discipline, task
> presentation structure, model selection, complexity scoring, testing rules, commit
> rules, handoff format, ADR propagation, and language policy.
> It overrides `CLAUDE.md` (project and global) and `AGENTS.md` without exception.
> `CLAUDE.md` applies only for topics not covered here.

## Mandatory workflow before implementing

1. **Analyze** — read context, dependencies, and affected files.
   - For **mobile UI / presentation tasks** under `mobile/`, also read the root
     `DESIGN.md` before planning or implementation. `DESIGN.md` governs visual
     intent and component-usage expectations for the mobile surface. It does not
     replace task files, runtime tokens in `mobile/src/theme/tokens.ts`, or the
     workflow authority of this guide.
2. **Plan** — create `docs/plan/<plan-name>.md` with: objective, affected files,
   design decisions, and module dependencies.
3. **Tasks** — create `docs/tasks/<tasks-name>.md` with: an ordered task list,
   inter-task dependencies, acceptance criteria per task, an **Effort** field
   (S/M/L/XL), a short agent handoff prompt, and for each development task a
   small behavioral example set covering both:
   - at least one **happy path example** with a stable `HP-#` ID — a concrete
     success flow the task must implement or preserve;
   - at least one **edge case example** with a stable `EC-#` ID — a concrete
     boundary, invalid-input, or failure flow the task must handle or reject.
4. **Gate by RRI** — compute RRI with `scripts/rri.py`. For RRI 0–25, skip the
   full human approval presentation. Use local Gemma delegation through Ollama
   only for eligible simple code patches; otherwise execute directly as the
   primary agent. For RRI 26+, show the plan and tasks and wait for explicit
   approval before starting implementation, even if a plan was approved in a
   prior session.
5. **Implement** — one task at a time, in the defined order.
6. **Mark progress** — update the tasks document after each completed task (it is
   the crash-safe progress ledger).
7. **Sync status artifacts before reporting completion** — before telling the user
   a task is done, update every materially affected status document in the same
   workflow pass. Completion is not valid until those documents are consistent.

## Task definition requirements

- For development tasks, the `docs/tasks/*.md` entry is not complete unless it
  includes explicit examples for both the intended happy path and the relevant
  edge cases.
- These examples do not need to be long. One or two bullets per category is
  enough if they are concrete and testable.
- Every development-task example must have a stable case ID:
  - happy path examples use `HP-1`, `HP-2`, etc.;
  - edge case examples use `EC-1`, `EC-2`, etc.
- Write the examples in behavioral terms, not implementation terms. Prefer
  statements such as `HP-1: valid ingest token + owned blob -> artifact finalized`
  over `call finalize_ingestion()`.
- The pre-task sections `Happy paths considered` and `Edge cases considered`
  should be derived from these task-definition examples, then refined if new
  constraints are discovered during analysis.
- Skip this requirement for docs-only, config-only, migration-only, or planning
  tasks unless the task's main risk is behavioral correctness.
- A task ledger can opt into automated enforcement by declaring
  `Behavioral coverage contract: unit-v1`. For ledgers with that marker, `make
  qa-docs` rejects completed development tasks whose `HP-#` / `EC-#` cases are not
  certified with unit test evidence. Legacy completed tasks without the marker are
  grandfathered until they are migrated into the contract.

## Per-task discipline

- Present the next task using the `AGENTS.md` presentation contract before executing
  it when approval is required. For RRI 0–25, do not present the full task for
  approval. If the task is an eligible simple code patch, prepare a local
  delegation packet for Gemma and report after review and verification; otherwise
  execute directly and report normally.
- **Pre-task summary for development tasks:** when the task will write or modify
  code, the task presentation must include two explicit sections:
  - **Happy paths considered** — the primary success flows the agent expects to
    implement and verify for the task.
  - **Edge cases considered** — the boundary and failure conditions the agent
    expects to handle or verify for the task.
  - **Reflection strategy** — when the task's RRI is 26 or higher, the task
    presentation must state the Reflection strategy that will be used, derived
    from the `Reflection design pattern for development tasks` section below.
    The presentation must name the required pass count for the task's RRI band
    and briefly summarize the intended focus of each pass.
  - **Diagram** — a compact Mermaid diagram that explains the concept to be
    implemented: the flow, boundary, dependency direction, state transition, or
    ownership split that the task relies on. The diagram may be minimal, but it is
    required for development tasks even when the architecture itself is unchanged.
  These sections are required at task start for development tasks so approval
  covers not just the objective but also the intended behavioral coverage. Skip
  them for docs-only, config, migration-only, or planning tasks unless the user
  explicitly asks for them.
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
- Work on the approved or delegated task only; show a summary before switching to
  the next.
- **Post-task summary for development tasks:** when the completed task involves writing
  or modifying code, the summary must include two explicit sections:
  - **Happy paths covered** — the primary success flows exercised by the implementation
    and tests (e.g., "valid command → session created in Requested state").
  - **Edge cases covered** — the boundary and failure conditions explicitly handled in
    logic and tests (e.g., "None credential_ref → MissingCredentialRef before any IO").
  For both sections, include **code evidence**: point to the concrete files,
  functions, and tests that prove the claimed coverage, using file references and
  concise explanations of what each reference demonstrates.
  This section is required only for development tasks. Skip it for docs-only,
  config, migration-only, or planning tasks.
- **Unit coverage certification for development tasks:** before marking a
  development task `[x] Done`, add a `Unit coverage certification` section that
  maps every approved `HP-#` and `EC-#` case to at least one unit test reference in
  the form `` `path/to/file.rs::test_name` ``. The referenced test must replicate
  the behavior described by that case and the recorded result must be `passed`.
  `N/A` is not allowed for development-task happy paths or edge cases. If a case
  cannot be unit-tested, refactor the implementation until it can be unit-tested
  or revise the task definition before closure.
- The same completion record must include `Owner final verification` with owner,
  date, verification statement, and exact commands run. The owner is responsible
  for certifying that each referenced unit test genuinely covers the claimed
  behavior; the automated gate verifies the structure and referenced test
  existence.

Required completion format for development tasks:

```md
### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid input creates session | `apps/gateway/src/auth/login.rs::valid_login_creates_session` | passed |
| EC-1 | Edge case | unknown state fails closed | `apps/gateway/src/auth/login.rs::unknown_state_returns_unauthorized` | passed |

### Owner final verification

- Owner: `<name-or-handle>`
- Date: `YYYY-MM-DD`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `<exact test commands>`
```

## ADR change propagation

An ADR change that occurs outside a task ledger (e.g. a replan, a hotfix, or a
cross-cutting amendment) is still subject to this contract. Apply the matching row
in the same change — not as a follow-up.

| ADR change | Must review and update in the same change |
|---|---|
| **New ADR** | `docs/adr/README.md` index row; ADR frontmatter block (`type: ADR`, `title:`, `status:`); `docs/architecture.md` if it adds or alters a runtime/crate boundary; `docs/plan/roadmap.md` if it changes slice scope or dependencies; the affected `docs/plan/*` and `docs/tasks/*` files |
| **Status change** (`Proposed` → `Accepted` → `Superseded` / `Deprecated`) | ADR frontmatter `status:` field (must mirror the prose `- **Status:**` token); index `Status` column; every canonical doc (`architecture.md`, `roadmap.md`, plan/tasks) that cites the ADR as authority for a decision |
| **Scope narrowed or broadened** | index scope annotation; `docs/architecture.md`; `docs/plan/roadmap.md`; affected plan/tasks; `README.md` if the change is outward-facing |
| **Content / decision change** (the decision itself, not just status or scope) | every canonical doc whose prose describes that decision — **this is semantic and not machine-verifiable**; Layer 2/3 confirm references still resolve, but human review owns whether the prose is still accurate |
| **Superseded by ADR-YYY** | both ADRs' frontmatter (`status:` / `supersedes:` / `superseded_by:`); both ADRs' prose `Status` field; the index row for each; every doc citing the superseded ADR |
| **Deletion or renumbering** | see the deletion rule below; update the index, every doc citation, **and every code/migration comment** (`.rs`, `.sql`) in the same change |

**Deletion rule.** An `Accepted` ADR is part of the auditable decision record and
must **not be deleted** — mark it `Superseded by ADR-YYY` or `Deprecated` instead.
A `Proposed` ADR that was never adopted may be deleted only after every reference
(docs *and* code/migration comments) is removed in the same change. Renumbering is
a delete + create and must update all references atomically.

**Definition of done for any ADR change:**
- [ ] The ADR file's prose `- **Status:**` line is updated.
- [ ] The ADR file's frontmatter `status:` mirrors the prose token; `supersedes:` /
      `superseded_by:` are set where applicable (frontmatter parity).
- [ ] `docs/adr/README.md` index row matches (status token + title).
- [ ] Every doc in the matching propagation row above has been reviewed and updated
      if its content describes the changed decision.
- [ ] No code or migration comment cites a missing ADR number.
- [ ] `make qa-docs` passes (index parity, completeness, dangling refs in docs and
      code/migrations, superseded-successor existence, OKF frontmatter parity).

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
| XL | Very high — RRI-driven reasoning, risk, and verification burden | Cross-boundary redesign with explicit risk analysis |

**Canonical effort mapping (required):** `Effort` must reflect the computed **RRI
band**, not a separate subjective estimate of likely elapsed time or annoyance. See
`docs/policies/RRI_POLICY.md` §Bands, autonomy gates, and model tiers for the
canonical crosswalk.

The S/M/L/XL descriptions above are illustrative; the RRI band is authoritative for
assignment.

Effort, capability tier, and autonomy gate are each derived in parallel from the RRI
band; never derive capability or gate from Effort.

Rules:
- Do not use `Effort` to encode toolchain pain, waiting time, or expected operator
  frustration when the computed RRI is lower.
- If a task is operationally tedious but its RRI remains in a lower band, keep the
  lower `Effort` and explain the operational caveat in prose.
- If an existing task ledger has `Effort` that disagrees with the computed RRI band,
  update the ledger so `Effort`, complexity presentation, and model guidance are
  internally consistent in the same documentation change.

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

The **RRI 0–25 Low band** is the exception to vendor model resolution: it uses
local Gemma delegation through Ollama. Resolve the local model from
`DUBBRIDGE_LOW_RRI_MODEL`, defaulting to `gemma4:26b-a4b-it-qat`, and the
Ollama endpoint from `OLLAMA_HOST`, defaulting to `http://localhost:11434`.

When preparing a task for presentation or local delegation, the agent must compute
a complexity score and derive the recommended model tier or local delegation
target from it. Do not guess; use the procedure below.

### RRI — canonical scoring method (adopted 2026-06-04)

This guide adopts the **Required Reasoning Index (RRI)** as the canonical method
for deriving complexity, risk, model tier, and autonomy gates. The full procedure
(formula, scoring rubric, repo-specific anchor rubric, penalty table, bands, and
decomposition triggers) lives in `docs/policies/RRI_POLICY.md`.

**Adoption note:** RRI supersedes the single-axis cyclomatic-complexity scoring
that previously drove the tier mapping. No ADR is required — RRI is a workflow
policy, not a runtime architecture decision. `AGENTS.md` and `CLAUDE.md` are
**not** changed; this guide overrides both "without exception" on complexity
scoring and model selection, so the adoption is binding from this file alone.

**How Steps 1 and 2 below relate to RRI:**
- The cyclomatic-complexity formula in Step 1 maps directly to the **`C` variable**
  of the RRI formula. Step 1 remains the procedure for computing `C`.
- The tier mapping in Step 2 is now driven by the **RRI band** (not the raw CC
  label). The tier names (Economy / Balanced / Premium) and thinking-mode rules
  are unchanged; only the input that selects the tier changes.
- Step 3 is updated to include the RRI score in the task presentation for RRI 26+,
  or in the local delegation packet and final report for RRI 0–25.

Before presenting or delegating any task: **run `scripts/rri.py`** — do not compute the RRI by hand.
The script measures F automatically and maps raw CC to the C score via the policy
table. Paste its markdown output directly into the task presentation for RRI 26+,
or into the local delegation packet and final report for RRI 0–25.

```bash
# Task-presentation time (before code is written — diff is empty):
python3 scripts/rri.py \
  --touches <path1> --touches <path2> \
  --cc <raw-cyclomatic-complexity> \
  --D <0-5> --K <0-5> --P <0-5> \
  --T <0-5> --A <0-5> --X <0-5> \
  [--penalty refactor_and_behavior] [--penalty arch_decision] [--penalty no_verification]

# Post-implementation (diff available; omit --touches):
python3 scripts/rri.py --cc <raw> --D <0-5> --K <0-5> --P <0-5> \
  --T <0-5> --A <0-5> --X <0-5>
```

Measure C and T before invoking: use `radon`/`mccabe` (Python) or
`clippy::cognitive_complexity` (Rust) for C; use `cargo llvm-cov` for T.
The script applies D/P/K floors from the anchor rubric and auto-detects four
penalties — agent supplies only the three intent-based ones. See
`docs/policies/RRI_POLICY.md § Script automation` for the full agent-vs-script
split and `--json` output for tooling use.

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

| CC range | Cyclomatic (C) label | RRI `C` variable score |
|---|---|---|
| 1–5 | Low | 0–1 |
| 6–10 | Medium | 1–2 |
| 11–20 | High | 2–3 |
| > 20 | Very High | 4–5 |

> **Subsumed by RRI:** the CC range above is the `C` variable of the RRI formula.
> Use the full RRI score (not just `C`) to determine the model tier and autonomy
> gates. See `docs/policies/RRI_POLICY.md` for the complete scoring procedure.

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

Mapping (now driven by RRI band — see the canonical crosswalk in
`docs/policies/RRI_POLICY.md` §Bands, autonomy gates, and model tiers):

> **Subsumed by RRI:** the complexity label alone no longer determines the tier.
> The RRI band (which incorporates `C`, `F`, `D`, `T`, `A`, `K`, `P`, `X`, and
> penalties) selects the canonical crosswalk row. The tier names and thinking-mode
> rules are unchanged; only the input that selects the tier has changed.

Agent-specific resolution rules:

- For RRI 0–25, use the local Ollama/Gemma delegation protocol in
  `docs/policies/RRI_POLICY.md § Low RRI local delegation`; do not resolve to a
  cloud vendor model. For the step-by-step handoff discipline for local-model
  work, see `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`.
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
- `Effort` must be derived from the computed RRI band using the canonical effort
  mapping above; it does not replace the complexity formula. If an existing task's
  recorded `Effort` disagrees with the computed RRI band, fix the task metadata
  instead of carrying the inconsistency forward into the presentation.
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

### Step 3 — State it in the task presentation or delegation packet

For RRI 26+, include this block in the task presentation. For RRI 0–25, include it
in the local Gemma delegation packet and final report instead of presenting the
full task for approval:

```
| RRI              | <score> → band <label> → gates: <list>                  |
| Complexity score | <CC range or decision-weight score> → <cyclomatic/decision-weight label> |
| Claude Code      | <resolved model or pinned model> — thinking <On / Off>  |
| Codex            | <resolved model or pinned model>                        |
```

Present the full RRI variable table (variable | score | evidence | confidence)
before this summary block when a human approval presentation is required. For
RRI 0–25, place the same table in the local delegation packet and final report.
See `docs/policies/RRI_POLICY.md` for the reporting format.

The recommendation is **not** a competition between vendors. Every presentation
must provide:

- one concrete current recommendation for OpenAI / Codex
- one concrete current recommendation for Claude Code / Anthropic

Both recommendations must be derived from the same computed complexity and the
same tier-mapping rules in this guide. Do not present only one vendor unless the
task file explicitly scopes the task to a single vendor environment.

For RRI 0–25, replace both vendor recommendations with the resolved local Gemma
model and note that the active agent remains the reviewer/orchestrator.

Presentation rules:

- Always show the computed `Complexity score`, even if the task file already
  declares `Complexity:`.
- For development tasks, always include a Mermaid diagram in the task presentation.
  Its purpose is conceptual clarity at approval time, not only architecture-change
  review; use the smallest diagram that makes the implementation shape obvious.
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

## Reflection design pattern for development tasks

When a development task has an RRI of 26 or higher, the agent must apply
**Reflection** passes before reporting the task complete. Each pass is a complete
Draft → Critique → Revise loop.

Required pass count by RRI band:

| RRI band | Label | Required Reflection passes |
|---|---|---|
| 26–40 | Moderate | 2 |
| 41–55 | Med-high | 3 |
| 56–70 | Complex | 4 |

MANDATORY:
---------
For RRI 56+, decomposition is mandatory before implementation. Follow the
decomposition and human-review gates in `docs/policies/RRI_POLICY.md`, split the
task to the policy target, and only then implement the approved subtasks. Apply
at least the Complex band minimum of 4 Reflection passes to any 56+ development
subtask that proceeds after decomposition.

Task-presentation requirement for development tasks:

- When a development task's RRI is 26 or higher and the task is being presented
  for approval, the presentation must include a `Reflection strategy` section.
- That section must be derived from this table and must state:
  - the task's RRI and band;
  - the required number of Reflection passes for that band;
  - a short pass-by-pass plan describing the intended Draft → Critique → Revise
    focus for each pass.
- The section should be concrete enough that the approver can see how the agent
  intends to use Reflection on correctness, fail-closed behavior, side effects,
  and coverage risk for the specific task being proposed.

Each Reflection pass consists of:

1. **Draft** — produce the initial implementation following the task's acceptance
   criteria, happy paths, and edge cases. In later passes, treat the current revised
   implementation as the draft.
2. **Critique** — re-read the draft as if reviewing someone else's code. Check for:
   - logical correctness against every `HP-#` and `EC-#` case;
   - missing or incorrect error handling at system boundaries;
   - unintended side effects on adjacent modules or state;
   - whether applicable design patterns or concepts should be used to improve
     execution performance, memory usage, and UX/UI quality when the task has a
     user-facing surface;
   - test coverage gaps against the 90% gate.
3. **Revise** — apply concrete fixes identified in the critique step. If no fixes are
   needed, state that explicitly (one sentence).
4. **Certify** — proceed to unit coverage certification only after at least one
   complete Draft → Critique → Revise loop has been recorded for every required
   Reflection pass.

The passes must be documented in the task completion record as a
`### Reflection log` section placed before `### Unit coverage certification`.
Minimum format:

```md
### Reflection log

Required passes: <N> (`<RRI>` → `<band>`)

#### Pass 1

- **Draft verdict:** <one-line summary of current state>
- **Critique findings:** <bullet list of issues found, or "no issues found">
- **Revisions applied:** <bullet list of changes made, or "none">

#### Pass 2

- **Draft verdict:** <one-line summary of current state>
- **Critique findings:** <bullet list of issues found, or "no issues found">
- **Revisions applied:** <bullet list of changes made, or "none">
```

For RRI 0–25 tasks delegated to local Gemma, the delegating agent applies the
Reflection cycle to Gemma's output during the mandatory review step. Record the
reflection log in the final report, not inside the delegated task.

Skip the Reflection cycle for: docs-only, config-only, migration-only, or planning
tasks. For tasks at the boundary (RRI exactly 25–26), apply judgment: if the task
writes non-trivial logic, apply the cycle.

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

Keep handoff prompts minimal. The task was already presented and approved, or it
is in the RRI 0–25 local-delegation band — do not re-explain it.

A human-agent handoff prompt must contain only:

1. Task ID + one-line goal
2. Governing docs (task file + plan file, paths only)
3. The one file + line range with the logic to change
4. Exact acceptance criteria (bullets only, no prose)
5. Stop condition: what the agent must do last and must NOT start next

For RRI 0–25 local Gemma delegation, build a delegation packet instead of the
human-agent handoff prompt. It must contain only: task excerpt, acceptance
criteria, RRI output, allowed paths, relevant file snippets, and stop conditions.
Send the packet with `scripts/delegate-low-rri.py`, which performs the local
Ollama request with the repository timeout. Gemma must return the tagged-block
contract with complete file contents for each changed file; the delegating agent
must validate the tagged response, let the wrapper build and check the diff,
personally review the solution against the requirements, run verification, and
perform at most one bounded repair cycle before escalating. Gemma must not
evaluate or approve its own delegated work.

For harder but still Low-RRI attempts, the wrapper supports explicit generation
knobs such as `--temperature` / `DUBBRIDGE_LOW_RRI_TEMPERATURE` and `--think` /
`--no-think` / `DUBBRIDGE_LOW_RRI_THINK`. Keep thinking mode off by default; use
it only for a bounded experiment because it can consume the token budget before
the tagged response is completed.

## Reviewability budget gate

Local Gemma roles evaluate a change inside a fixed context window
(`DEFAULT_NUM_CTX`) while reserving generation headroom (`DEFAULT_NUM_PREDICT`).
A change larger than that effective window either overflows the context silently
or truncates Gemma's response (`done_reason == "length"`). The before-after mode
and the push-review token-limit handler protect against this *after* it happens;
the **reviewability budget gate** (`make qa-review-budget`,
`scripts/check-review-budget.py`) is the *proactive* counterpart that runs before
delegation.

The gate fails closed when the added/changed code lines of the change exceed a
budget **derived from the context window** — not a fixed constant — so it tracks
`DUBBRIDGE_REVIEW_NUM_CTX` / `DUBBRIDGE_REVIEW_NUM_PREDICT` rather than drifting
from them. `DUBBRIDGE_REVIEW_MAX_DIFF_LINES` overrides the derived value when an
operator needs an explicit ceiling, and `DUBBRIDGE_REVIEW_PACKET_OVERHEAD_TOKENS`
tunes the fixed prompt/contract overhead the derivation reserves. Only code paths
Gemma actually receives are counted; docs, config, and markdown are excluded,
mirroring the `qa-gemma-review` packet filter.

**Non-Gemma agents are responsible for staying inside this budget.** When a
change is too large, the delivering agent must split it into smaller delegation
units. If the change is genuinely irreducible (mechanical rename, atomic
migration), the agent takes the **documented escape**: record a
`D14-OVERRIDE: <reason>` line in the commit body or task entry, which passes the
gate and routes the change to the non-Gemma context-isolated reviewer (D14)
instead of Gemma. The override reason is captured for the audit log; an override
without a reason does not satisfy the gate. The escape is for reviewability, not
for skipping review — the D14 reviewer still runs and the primary agent records
`disposition_divergence`.

## Language

- User-facing communication: Spanish.
- Plans, task documents, prompts, ADRs, and code/comments: precise technical English.

## Communication format

Agent communication must follow a **Socratic doubt model**:

- **Do not consent by default.** Do not affirm, validate, or agree with a user statement unless you have verified it independently. A question is not a position; treat it as a question.
- **Doubt with trusted sources.** Every claim about the codebase, a policy rule, a score, or a fact must be grounded in a source you can cite (a file, a line, a tool output). If you cannot cite a source, say so explicitly rather than asserting.
- **No hallucination.** Do not infer positions from tone or phrasing. Do not attribute intent, agreement, or correctness to a message that does not state them. If a message is ambiguous, ask — do not deduce.
- **Challenge your own output.** Before reporting a result, ask whether it could be wrong and whether the source you used is current. The RRI self-scoring error in T1 (estimated ~16/28 by hand; script returned 27) is the canonical example of why this matters.

## Gemma Reviewer

**Gemma Reviewer** is a read-only local model role that runs after implementation
and before the primary agent's final Reflection cycle for Low (0–25) and Moderate
(26–40) RRI development tasks. It is distinct from **Gemma Developer**, which is
the patch-delegation path for eligible simple code patches.

### Authority boundary

- Gemma Reviewer may report findings (correctness, fail-closed, side-effect, and
  missing-test issues). It may not write files, apply patches, approve tasks,
  certify coverage, or mark tasks complete.
- A finding — including a `BLOCKING` one — never fails the review gate by itself.
  Gemma Reviewer is advisory evidence; the primary agent owns the final judgment.
- Gemma-authored Low-RRI patches require an independent primary-agent review even
  when Gemma Reviewer also runs.

### When it runs

For development tasks with RRI 0–40, after implementation is complete:

1. Implementation completes (primary agent or eligible Gemma Developer).
2. Gemma Reviewer runs N sequential passes (default 3, `--passes N`,
   env `DUBBRIDGE_REVIEW_PASSES`) via `scripts/gemma-code-review.py`.
   - **≥2 of N passes succeed** → deterministic reconciliation produces an
     aggregate (exactly 2/N ⇒ `degraded: true`). Findings are classified as
     `consensus`, `pass-specific`, `severity-inconsistent`,
     `location-inconsistent`, or `likely-false-positive`.
   - **`--passes 1`** → reproduces the previous single-pass behavior exactly.
   - **<2 passes succeed or Gemma unavailable** → see Availability below.
3. The primary agent runs its Reflection cycle, treating Gemma Reviewer findings
   as one input and recording the disposition in `### Reflection log`.

Gemma Reviewer does not add a separate sign-off step; it feeds the existing
Reflection cycle.

### Availability

The review step is **mandatory for all Low/Moderate development tasks**.
Gemma is the preferred path; the **context-isolated subagent** (D14) is the
required fallback.

- **Gemma available, quorum met (≥2 passes):** run `make qa-gemma-review`.
- **Gemma unavailable or quorum fails (<2 passes):** the agent must spawn a
  context-isolated subagent as the mandatory fallback reviewer. The subagent
  receives an isolation packet (diff + acceptance criteria + any partial
  findings) and its output is advisory, exactly as Gemma's. The primary agent
  reconciles and records `disposition_divergence` in the audit log.
- **Neither path may be skipped.** No additional human approval gate beyond
  what the RRI band already requires is opened by using the fallback.

Docs-only, config-only, migration-only, ADR, plan, task-ledger, and policy-only
work are exempt from this review requirement.

### Context-isolated adjudicator (D14)

When the D14 trigger fires, the disposition of findings is adjudicated by a
fresh subagent or fresh session — fed **only** the final diff, the acceptance
criteria, and the reconciled findings — never the development transcript or
chain-of-thought. The `scripts/adjudicator-packet.py` module implements the
trigger gate (`should_adjudicate()`) and the isolation packet builder
(`build_adjudicator_packet()`).

**Trigger conditions (any one fires):**

| Condition | Detail |
|---|---|
| Gemma blocked / quorum failure | `gemma_blocked=True` — mandatory fallback |
| Consensus blocking or major finding | any `consensus` finding with `severity` in `blocking`, `major` |
| Band ≥ Med-high | slice band is `Med-high` or `Complex` |
| Inter-pass disagreement | `severity_inconsistent_count > 0` or `location_inconsistent_count > 0` |

**Model:** the subagent must be spawned at the **Balanced** tier — a capable
but token-efficient model, not Premium. The adjudicator role is read-only and
analytical (diff + criteria + findings), not generative or synthesis-heavy;
a Premium model is wasteful and must not be used unless the primary agent
explicitly overrides with a documented reason recorded in the audit log.
Resolve the concrete Balanced-tier model from the active environment per
`docs/policies/RRI_POLICY.md` §Model tier resolution; do not pin a model ID
in this guide.

**Authority:** the adjudicator is advisory — it never closes the task. The
primary agent reconciles its disposition against the adjudicator's and records
`disposition_divergence` (`"none"`, `"partial"`, or `"full"`) in the audit log.
Simulated self-review ("re-read as if reviewing someone else's code") is
retained **only** when none of the above trigger conditions are present.

### Scope

Does not apply to docs-only, config-only, migration-only, ADR, plan,
task-ledger, or policy-only work.

### Completion evidence block

Task completion records for Low/Moderate development tasks must include:

```md
### Gemma Reviewer evidence

- Model: `<resolved DUBBRIDGE_REVIEW_MODEL, else DUBBRIDGE_LOW_RRI_MODEL>`
- Command: `<exact command, e.g. make qa-gemma-review>`
- Passes run / succeeded: `<N>/<N>` (e.g. `3/3`, `2/3 degraded`)
- Quorum: `met | failed`
- Aggregate status: `PASS | FINDINGS | BLOCKED`
- Consensus findings: `<count>` | Pass-specific: `<count>` | Disagreement: `<count>`
- Degraded: `true | false`
- Artifacts: `<path to result.json and per-pass result.passK.json, if persisted>`
- Isolated adjudicator: `spawned | not triggered` — trigger: `<condition or n/a>`
- disposition_divergence: `none | partial | full | null`
- Primary-agent disposition: `<accepted findings / rejected false positives / repaired>`
```

`--passes 1` collapses to the single-pass form (no reconciliation fields, no
per-pass artifacts). Run the reviewer with `make qa-gemma-review` (local only;
not required in GitHub-hosted CI until an Ollama-capable runner is available).
For task ledgers that declare `Behavioral coverage contract: unit-v1`, `make
qa-docs` rejects completed development sections that omit required `Gemma
Reviewer evidence` for RRI 0–40 or omit the `Reflection log` required for
RRI 26+.

## Push Reviewer

**Gemma Push Reviewer** is a separate post-pipeline audit role. It is not a
code-review replacement, not a patch approver, and not a final RRI authority.

### Authority boundary

- Push Reviewer starts only from completed GitHub pipeline evidence (`workflow_run`
  or local replay against a completed run).
- It may collect run metadata, job status, failed-step summaries, annotations,
  and available logs/artifacts before model analysis.
- It may normalize findings into candidate tasks, pass them through
  `scripts/rri.py`, and dispatch only pure Low eligible incidents to Gemma Developer.
- It may not compute the final RRI itself, accept a delegated patch, certify
  coverage, or close the work item.
- Post-development review of any delegated patch remains a non-Gemma-agent responsibility.

### Daily consumption

- Daily opening and close should inspect the newest push-review summary when one
  exists.
- Non-pure-Low or Moderate+ findings must be carried into the daily ledger as
  non-Gemma review work or HITL decisions.
- Delegated pure Low patches must remain visible as `in_review` until their
  post-development review is completed and recorded.

## Development task closure checklist

A development task is not done until the closure gates for its band have been
checked in order. Evaluate the review gate first; do not start the closure
summary with unit coverage certification or owner final verification.

Low/Moderate review gate:

- **RRI 0–40 development tasks:** `Gemma Reviewer` is mandatory unless the task
  is exempt (`docs-only`, `config-only`, `migration-only`, `ADR`, `plan`,
  `task-ledger`, or `policy-only`).
- **RRI 0–25 direct primary-agent development tasks:** record whether the review
  ran through Gemma quorum or the D14 fallback before describing any completion
  certification.
- **RRI 0–25 delegated Gemma Developer tasks:** the delegating agent records the
  mandatory review and reflection in the final report rather than inside the
  delegated task entry, but the task is still not complete until that review is
  reconciled.

Moderate+ task-entry checklist:

```
[ ] 1. Gemma Reviewer / D14 adjudicator
       - Run `make qa-gemma-review`
       - If Gemma unavailable OR band ≥ Med-high: spawn D14 context-isolated
         subagent (mandatory fallback — not optional)
       - Record `### Gemma Reviewer evidence` block in the task entry

[ ] 2. Reflection log
       - Moderate (26–40): 2 passes
       - Med-high (41–55): 3 passes
       - Complex  (56–70): 4 passes
       - Record `### Reflection log` block in the task entry

[ ] 3. Unit coverage certification
       - Table: Case ID | Type | Behavior | Unit test evidence | Result
       - Every HP-# and EC-# must map to at least one passing test
       - Record `### Unit coverage certification` block in the task entry

[ ] 4. Owner final verification
       - Owner, date, statement, exact commands run
       - Record `### Owner final verification` block in the task entry
```

Only after the applicable review gate and all required completion blocks are
checked may the task status be flipped to `[x] Done` and the completion
reported to the user.

## Related

- `CLAUDE.md`, `AGENTS.md`, `README_AGENT_ORDER.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md` — RRI formula, anchor rubric, bands, and gates
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md` — patch delegation vs. review delegation
- `docs/gemma-local-improve.md` — active local Gemma contract summary
