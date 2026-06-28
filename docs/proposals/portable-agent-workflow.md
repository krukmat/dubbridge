---
type: Proposal
title: "Portable Agent Workflow Extraction"
status: Proposed
---
# Portable Agent Workflow Extraction

## Purpose

This document extracts the reusable agent workflow from this repository into a
project-agnostic operating contract. It is written for non-human coding agents
that need deterministic rules for planning, approval, implementation, review,
documentation, and closure in an existing software project.

The workflow is intentionally conservative. It assumes that an agent can forget
repository rules between sessions, that documentation can drift from code, and
that large or ambiguous changes should fail closed into planning or human review
rather than proceed on memory.

## Source Map

These DubBridge files define the source workflow. When porting the workflow, copy
the patterns and replace project-specific names, paths, model pins, ADR numbers,
and product invariants.

| Source | Portable role | Copy strategy |
|---|---|---|
| `AGENTS.md` | Shared task-presentation contract for agents | Copy and neutralize project names. |
| `README_AGENT_ORDER.md` | Agent orientation order | Copy and make the target project's workflow guide the highest authority. |
| `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | Canonical workflow authority | Copy as the main playbook; replace stack-specific commands and paths. |
| `docs/policies/HITL_AUTONOMY_POLICY.md` | Human approval and autonomy policy | Copy and adapt approval thresholds only if the target project has a different risk tolerance. |
| `docs/policies/RRI_POLICY.md` | Required Reasoning Index policy | Copy the formula and bands; replace the project-specific anchor rubric. |
| `scripts/rri.py` | Deterministic RRI calculator | Copy with tests; update env var prefix and optional dedicated project profile. |
| `docs/knowledge/README.md` | OKF document vocabulary | Copy the closed-vocabulary concept; adjust document types if needed. |
| `scripts/check_okf_frontmatter.py` | OKF frontmatter gate | Copy and update paths/type vocabulary. |
| `DESIGN.md` | Agent-readable design intent | Copy the structure, not the visual identity. |
| `scripts/agent-preflight.py` | Session preflight anti-forgetting gate | Copy and parameterize paths/env names. |
| `docs/plan/*` | Plan artifact examples | Copy structure only. |
| `docs/tasks/*` | Task ledger examples | Copy structure only. |
| `docs/daily/README.md` and `docs/daily/TEMPLATE.md` | Daily operational ledger | Optional; useful for multi-session agent work. |
| `Makefile` | Local QA command surface | Copy target names conceptually; replace implementation commands. |
| `.githooks/pre-push`, `scripts/hooks/*` | Local enforcement hooks | Copy the change-detection pattern; replace stack gates. |
| `.github/workflows/ci.yml` | Remote enforcement | Copy job categories; replace concrete tooling. |
| `.github/workflows/push-review.yml` | Post-pipeline advisory review | Optional advanced pattern; requires self-hosted/local-model support. |

## Non-Portable DubBridge Content

Do not copy these items literally into an unrelated project:

- Product concepts: governed media review, rights ledger, asset ingestion, HLS
  playback, ASR transcription, publication workspace, mobile-only authenticated
  product surface.
- DubBridge paths: `crates/domain`, `crates/audit`, `apps/gateway`,
  `apps/worker-runner`, `workers/asr-worker-py`, `mobile/src/theme/tokens.ts`.
- DubBridge ADR IDs and invariants: ADR-006, ADR-008, ADR-018, ADR-021,
  ADR-023, ADR-024, ADR-025, ADR-026, ADR-029, ADR-033, ADR-034.
- DubBridge env var names: `DUBBRIDGE_*`.
- DubBridge model defaults if the target project has a different approved local
  model policy.
- Hardcoded local paths such as `/Users/matias/dubbridge`.

Replace these with the target project's architecture, risk anchors, runtime
boundaries, and local tooling.

## Core Workflow

### Required Agent Operating Order

Every coding agent should follow this sequence before changing files:

1. Read the repository workflow authority.
2. Identify affected files and governing docs.
3. Ensure a plan artifact exists for staged work.
4. Ensure a task ledger exists for staged work.
5. Compute RRI with the script, not by hand.
6. Apply the RRI gate.
7. Present the task and wait for approval when required.
8. Implement one task at a time.
9. Verify with the relevant local checks.
10. Update status artifacts.
11. Complete development closure evidence before marking work done.

The important portable principle is that a plan approval does not approve every
future implementation step. For RRI 26 or higher, approval is per task and per
session.

### Authority Order

Create a root orientation file equivalent to `README_AGENT_ORDER.md`:

```md
# Agent Orientation Order

1. Project workflow guide: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
2. Shared agent contract: `AGENTS.md`
3. Human approval policy: `docs/policies/HITL_AUTONOMY_POLICY.md`
4. RRI scoring policy: `docs/policies/RRI_POLICY.md`
5. Architecture decisions: `docs/adr/`
6. Roadmap, plan, and task ledgers: `docs/plan/`, `docs/tasks/`
7. Product, BDD, design, config, and CI docs that constrain the current task
```

Make one source highest-authority. In DubBridge this is
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`. The target project should do the same
to avoid contradictions between `AGENTS.md`, Claude/Codex local instructions,
and older project docs.

## Plan And Task Ledgers

### Plan Artifact

Create one plan per slice, feature, migration, or cross-cutting workstream:

```text
docs/plan/<scope>.md
```

Minimum sections:

- OKF frontmatter.
- Purpose.
- Objective.
- Scope included.
- Scope excluded.
- Governing constraints.
- Design decisions.
- Affected components.
- Task decomposition.
- Dependencies.
- Risks and open questions.
- Verification strategy.

The plan explains why the work exists and how it is decomposed. It should not be
used as a substitute for per-task approval when RRI requires it.

### Task Ledger

Create one task ledger for each plan:

```text
docs/tasks/<scope>.md
```

Minimum sections per task:

- Task ID and title.
- Status.
- Type: development, docs, config, migration, planning, ADR, task-ledger, or
  policy.
- Effort derived from RRI band.
- Numeric RRI where applicable.
- Dependencies.
- Goal.
- Inputs.
- Outputs.
- Acceptance criteria.
- Files expected to change.
- Agent handoff prompt.
- Happy path examples with stable `HP-#` IDs for development tasks.
- Edge case examples with stable `EC-#` IDs for development tasks.

Development tasks are incomplete unless they include at least one happy path and
one edge case. The examples must be behavioral, not implementation phrasing.

Good:

```text
HP-1: valid credentials create a session and return a scoped token.
EC-1: unknown user and wrong password return the same generic error.
```

Weak:

```text
HP-1: call login().
EC-1: add an error branch.
```

### Behavioral Coverage Contract

For strict projects, task ledgers can opt into:

```yaml
Behavioral coverage contract: unit-v1
```

When enabled, a docs gate should reject completed development tasks unless every
`HP-#` and `EC-#` has unit test evidence in the task completion record.

Portable validator behavior:

- Only completed development tasks are enforced.
- Docs-only, config-only, migration-only, ADR, plan, task-ledger, and policy-only
  tasks are exempt unless their main risk is behavioral correctness.
- Each case must map to a concrete test reference.
- `N/A` is not allowed for development-task happy paths or edge cases.

## RRI: Required Reasoning Index

### Purpose

RRI estimates how much reasoning, context, caution, and verification a task
requires before an agent may safely implement it.

Use RRI to derive:

- complexity band;
- effort;
- model/capability tier;
- thinking mode;
- autonomy gate;
- decomposition requirement;
- review and closure evidence.

Agents must run the RRI script. They must not compute the score by hand.

### Formula

```text
RRI = 100 * ((0.18*C + 0.12*F + 0.15*D + 0.15*T
              + 0.12*A + 0.12*K + 0.10*P + 0.06*X) / 5)
      + Penalties
```

Variables are scored 0-5:

| Var | Meaning | How to obtain |
|---|---|---|
| C | Cyclomatic complexity | Measure raw CC with platform tooling or pass a supplied score. |
| F | Files affected | Count touched paths before implementation or git diff after implementation. |
| D | Domain complexity | Judge with anchor rubric. |
| T | Test coverage risk | Measure coverage or inspect tests in the affected area. |
| A | Task ambiguity | Lower when task has criteria and HP/EC examples. |
| K | Coupling and side effects | Judge with anchor rubric. |
| P | Public API, security, permissions, data impact | Judge with anchor rubric. |
| X | Context size | Count files/modules the agent must hold in mind. |

### Bands

| RRI | Label | Effort | Gate |
|---|---|---|---|
| 0-25 | Low | S | No full approval packet. Primary agent executes directly, or local model delegation only for simple code patches. |
| 26-40 | Moderate | M | Present task and wait for explicit approval. Confirm tests exist. |
| 41-55 | Med-high | L | Present task, acceptance criteria, and plan before approval. |
| 56-70 | Complex | L | Decompose before implementation. Human reviews plan. |
| 71-85 | High | XL | Characterization tests, explicit criteria, human reviews diff. |
| 86-100 | Very high | XL | Do not implement directly. Produce ADR, risk analysis, and subtasks. |
| >100 | Excessive | XL | Architecture/design work first. Re-scope before implementation. |

### Decomposition Triggers

Split the work before implementation if any of these apply:

- Final RRI is 56 or higher.
- RRI is above 70.
- Base RRI is above 100.
- Many files plus high coupling.
- High cyclomatic complexity plus high domain complexity.
- Refactor and behavior change are combined.
- No tests and high public/security/data impact.

The split target is each subtask scoring RRI 55 or lower with ambiguity score 0
or 1.

### Portable Script Requirements

Copy `scripts/rri.py` and its tests. Keep these capabilities:

- markdown output for task presentations;
- JSON output for tooling;
- platform detection;
- generic anchor rubric;
- dedicated project profile support;
- automatic F score;
- raw CC to C score mapping;
- band, effort, thinking, and gate output;
- penalty detection;
- decomposition trigger reporting.

Supported portable profiles from DubBridge:

| Profile | Marker | Complexity measurer |
|---|---|---|
| Rust | `Cargo.toml` | `cargo clippy` cognitive complexity |
| Go | `go.mod` | `gocyclo` |
| React Native / JS / TS | `package.json` | ESLint complexity rule |
| Python | `pyproject.toml`, `setup.py`, `setup.cfg` | `radon cc` |
| Generic | none | agent supplies score |

Recommended dependencies:

- `python3`
- `radon` for Python projects
- `gocyclo` for Go projects
- `eslint` for JS/TS projects
- `cargo` and `clippy` for Rust projects

### Required RRI Report Shape

Before implementation, the agent should paste the script's markdown output
directly into the task presentation or local delegation packet. Do not reformat
or recompute the score manually.

The report must include:

- platform/profile;
- variable table with score, evidence, and confidence;
- base value;
- penalties;
- final RRI;
- band;
- effort;
- model/capability tier;
- thinking mode;
- gates;
- decomposition status;
- advisories.

If any D, P, or K confidence is low, raise that variable one step before
accepting the score. Low confidence is itself risk evidence.

### Model Tier Resolution

Keep the workflow stable by separating two decisions:

1. capability tier from the RRI band;
2. concrete model ID from current vendor guidance or local environment.

Portable tier labels:

- Economy: mechanical low-complexity work;
- Balanced: standard implementation and review work;
- Premium: architecture, synthesis, deep debugging, or high-risk work.

Rules:

- RRI 0-25 does not resolve to a cloud vendor recommendation; it uses the
  primary agent or approved local model path.
- RRI 26+ should resolve the tier to concrete current model IDs for each agent
  environment used by the project.
- If naming a current vendor model as "recommended", verify against official
  vendor documentation when there is any reasonable chance the recommendation
  changed.
- Do not silently replace a task-local pinned model. Either use the pin or update
  the task metadata in an explicit documentation change.
- Thinking mode should be on only when multi-step reasoning cannot be validated
  incrementally. Do not turn it on for simple config, docs, or pre-defined edits.

## Approval Policy

### Always Require Human Approval

Require explicit approval before:

- implementing any task with RRI 26 or higher;
- deleting or overwriting files/data;
- committing or pushing;
- outward-facing actions such as PR creation, external deployment, production API
  calls, or customer-visible operations;
- schema migrations;
- governance-critical invariant changes;
- security/auth/permission boundary changes unless the task is already explicitly
  approved.

Approval does not carry across sessions or tasks.

### Permitted Without Prior Approval

These actions are normally allowed before explicit task approval:

- read-only analysis;
- repository search and code navigation;
- reading affected files and governing docs;
- drafting plans, task lists, ADRs, and proposals;
- computing RRI;
- running non-destructive validation commands;
- non-destructive documentation/config fixes only when the user explicitly asked
  for that bounded cleanup.

This does not authorize code implementation when RRI is 26 or higher.

### Safety Rules

Portable safety rules:

- Ask before deleting files or data.
- Do not commit with broken tests.
- Run the relevant checks before commit or push.
- Redact secrets and credentials in logs, reports, prompts, and artifacts.
- Surface contradictions instead of guessing through them.
- Report failed, skipped, or unavailable verification honestly.

### Low Band Handling

For RRI 0-25:

- Do not present the full approval packet.
- Execute directly as primary agent unless the task is an eligible simple code
  patch.
- Use local model delegation only for narrow, mechanical code/test edits with a
  small allowed path set.
- Do not delegate docs, plans, task ledgers, ADRs, policies, workflow scripts, or
  structure-heavy work to a local model.

This distinction matters: low-risk does not mean "always delegate".

## Task Presentation Contract

When approval is required, present:

1. Task ID and title.
2. Status.
3. Effort.
4. Complexity.
5. Recommended model for Codex and Claude Code.
6. Objective.
7. Context.
8. Related documents.
9. Inputs.
10. Outputs.
11. Acceptance criteria.
12. Execution summary.
13. Happy paths considered for development tasks.
14. Edge cases considered for development tasks.
15. Reflection strategy for RRI 26 or higher development tasks.
16. Pseudocode only if it clarifies non-trivial logic.
17. Mermaid diagram for development tasks.
18. Direct approval checkpoint.

Use this final line when approval is required:

```text
Execution has not started. Approve this task to proceed.
```

### Related Documents Rule

List only documents that materially constrain the task. Typical sources:

- task file;
- linked plan;
- workflow guide;
- HITL/RRI policies;
- ADRs;
- architecture docs;
- roadmap;
- BDD/product specs;
- design docs;
- configs and CI files.

Avoid dumping broad reading lists.

## Reflection

For development tasks with RRI 26 or higher, require documented Reflection
passes before completion. Each pass is:

1. Draft.
2. Critique.
3. Revise.
4. Certify readiness to proceed.

Pass count:

| RRI | Label | Required passes |
|---|---|---|
| 26-40 | Moderate | 2 |
| 41-55 | Med-high | 3 |
| 56-70 | Complex | 4 |

Reflection focus should include:

- correctness against all HP/EC cases;
- fail-closed behavior;
- side effects;
- boundary error handling;
- coverage gaps;
- performance/memory/UX concerns when relevant.

Docs-only, config-only, migration-only, planning, ADR, task-ledger, and
policy-only tasks are normally exempt.

## Testing Rules

Portable testing expectations:

- Prefer TDD where practical: write or update tests first, then implement.
- Target at least 90% line coverage for the implemented scope unless the target
  project has a different explicit gate.
- Prefer real backends and representative integration seams over mocks when the
  behavior depends on persistence, APIs, filesystem, queues, or external service
  boundaries.
- Keep coverage gates aligned between local QA and CI. If the threshold changes,
  update workflow docs and CI in the same change.
- Do not mark development work done while relevant tests are failing.
- For high-impact areas with weak tests, add characterization tests before
  changing behavior.

## Development Closure

A development task is not done until the applicable gates are checked in this
order:

1. Gemma Reviewer or D14 adjudicator, if required.
2. Reflection log, if required.
3. Unit coverage certification.
4. Owner final verification.
5. Status flip to Done.

Do not start a completion summary with coverage certification. First determine
whether the review gate applies.

### Unit Coverage Certification

Required format:

```md
### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid input creates session | `path/to/file.rs::test_name` | passed |
| EC-1 | Edge case | unknown state fails closed | `path/to/file.rs::test_name` | passed |
```

Portable adaptation:

- Change `.rs` references to the target language pattern if needed.
- Keep exact, discoverable test references.
- Require `passed` as the recorded result.

### Owner Final Verification

Required format:

```md
### Owner final verification

- Owner: `<name-or-handle>`
- Date: `YYYY-MM-DD`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `<exact commands>`
```

## Local Model Roles

DubBridge separates two local model roles. The separation is portable and should
be preserved.

| Role | When | Can write files? | Can approve? |
|---|---|---|---|
| Local Developer | Low-RRI simple code patch delegation | Only through validated tagged file contents returned to the orchestrator | No |
| Local Reviewer | Post-implementation read-only review | No | No |

The primary agent remains responsible for:

- validating local model output;
- applying patches;
- reviewing against requirements;
- running verification;
- reconciling findings;
- marking tasks complete.

### Local Developer Protocol

Portable requirements:

- Local model receives a bounded packet.
- Packet includes task excerpt, acceptance criteria, RRI output, allowed paths,
  relevant snippets, and stop conditions.
- Local model returns tagged text, not JSON and not a diff.
- Returned file paths must be checked against an allowlist.
- The wrapper builds the diff deterministically.
- The wrapper runs `git apply --check` before apply.
- Only one bounded repair cycle is allowed.
- Timeout, invalid format, out-of-scope path, or failed verification escalates.

DubBridge implementation:

- `scripts/delegate-low-rri.py`
- `scripts/gemma_local.py`

Portable env vars should be renamed from `DUBBRIDGE_*` to the target project
prefix.

### Local Reviewer Protocol

Portable requirements:

- Review is read-only.
- Reviewer may report findings, but may not approve or close work.
- Run several passes by default.
- Require quorum.
- If local model is unavailable or quorum fails, use a context-isolated
  adjudicator.
- Primary agent reconciles findings and records disposition.

### Local Reviewer Evidence Block

For Low and Moderate development tasks, record review evidence before closure:

```md
### Local Reviewer evidence

- Model: `<resolved local review model or D14 fallback>`
- Command: `<exact command>`
- Passes run / succeeded: `<N>/<N>` or `<succeeded>/<N> degraded`
- Quorum: `met | failed`
- Aggregate status: `PASS | FINDINGS | BLOCKED`
- Consensus findings: `<count>`
- Pass-specific findings: `<count>`
- Disagreement: `<count>`
- Degraded: `true | false`
- Artifacts: `<path to result files, if persisted>`
- Isolated adjudicator: `spawned | not triggered` - trigger: `<condition or n/a>`
- disposition_divergence: `none | partial | full | null`
- Primary-agent disposition: `<accepted findings / rejected false positives / repaired>`
```

The evidence block should use project-neutral naming such as "Local Reviewer"
when the target project is not specifically using Gemma.

DubBridge implementation:

- `scripts/gemma-code-review.py`
- `scripts/adjudicator-packet.py`
- `make qa-gemma-review`

## D14 / Context-Isolated Adjudication

Use a fresh model/session when:

- local reviewer is unavailable;
- local reviewer quorum fails;
- consensus finding is blocking or major;
- review passes disagree on severity or location;
- task band is Med-high or higher;
- change exceeds local reviewability budget and an override is documented.

Isolation packet should include only:

- final diff;
- acceptance criteria;
- reconciled local findings;
- necessary file context.

Do not include the development transcript or chain-of-thought. The adjudicator is
advisory, not authoritative.

## Reviewability Budget

The reviewability budget prevents handing a local model a diff too large for its
context window.

Portable rule:

- Count only code lines the local reviewer would receive.
- Exclude docs, config, and markdown if the reviewer does not inspect them.
- Derive the budget from model context size and reserved generation headroom.
- If over budget, split the change.
- If truly irreducible, record a documented override and route to D14.

DubBridge implementation:

- `scripts/check-review-budget.py`
- `make qa-review-budget`

Override marker:

```text
D14-OVERRIDE: <reason>
```

This is not a review skip. It routes review to a non-local-model adjudicator.

## OKF: Open Knowledge Format

OKF makes repository knowledge machine-readable and enforceable.

### Portable Contract

All canonical docs under `docs/` should have YAML frontmatter:

```yaml
---
type: <closed vocabulary value>
title: <human-readable title>
status: <status when relevant>
---
```

Recommended closed vocabulary:

| Type | Location |
|---|---|
| ADR | `docs/adr/ADR-*.md` |
| Playbook | `docs/playbooks/*.md` |
| Policy | `docs/policies/*.md` |
| Plan | `docs/plan/*.md`, except `roadmap.md` |
| Roadmap | `docs/plan/roadmap.md` |
| TaskList | `docs/tasks/*.md` |
| Architecture | `docs/architecture.md` |
| Proposal | `docs/proposals/*.md` |
| Audit | `docs/audit/*.md` |
| Prompt | `docs/prompts/*.md` |

Recommended exclusions:

- `docs/daily/*`;
- `TEMPLATE.md`;
- pure index READMEs;
- non-Markdown BDD `.feature` files unless the project explicitly supports them.

### Validator Requirements

The validator should enforce:

- frontmatter exists and is parseable;
- `type` is in the closed vocabulary;
- `type` matches file location;
- ADR frontmatter status matches prose status;
- `governed_by` ADR refs resolve;
- no canonical doc silently escapes the vocabulary.

DubBridge implementation:

- `docs/knowledge/README.md`
- `scripts/check_okf_frontmatter.py`
- `make qa-okf-frontmatter`
- included in `make qa-docs`

## DESIGN.md

`DESIGN.md` is an agent-readable design intent contract for UI work.

### Portable Contract

For UI or presentation tasks, the agent must read root `DESIGN.md` before
planning or editing.

`DESIGN.md` should define:

- product surface purpose;
- visual tone;
- color tokens;
- typography;
- spacing;
- radius/shape;
- component vocabulary;
- layout principles;
- loading, empty, error, and disabled states;
- interaction expectations;
- explicit do/don't rules;
- relationship to runtime tokens/components.

Important authority rule:

- Runtime tokens and shipped components remain the implementation source of truth.
- `DESIGN.md` governs design intent and agent behavior.
- If they drift, shipped runtime tokens win until documentation is synchronized.

### Portable Lint

Optional:

```bash
npx -y @google/design.md lint DESIGN.md
```

DubBridge implementation:

- root `DESIGN.md`
- `make qa-design` as an opt-in gate
- workflow rule requiring `DESIGN.md` for `mobile/` UI/presentation work

For the target project, replace:

- brand palette;
- product surface description;
- runtime token path;
- component vocabulary;
- mobile/web-specific constraints.

## Session Preflight Anti-Forgetting

This is the main defense against catastrophic forgetting in fresh Claude or
Codex sessions.

### Problem

Agents can start a new session without remembering repository-specific workflow
rules. A plain instruction file is not enough if the tool environment allows
editing before the workflow is loaded.

### Portable Solution

Provide a shared preflight script:

```bash
python3 scripts/agent-preflight.py --print-summary --mark
python3 scripts/agent-preflight.py --check
```

The script should:

- print a compact startup summary;
- mark a session-local sentinel under `.agent/session-preflight.json`;
- validate that the sentinel belongs to the current repo root;
- validate script/sentinel version;
- fail closed when missing, invalid, stale, or marked for another repo;
- avoid task-specific approval decisions.

The summary should remind the agent to:

- read the highest-authority workflow guide;
- identify affected files and governing docs;
- include architecture, ADRs, roadmap, plan, task ledger, BDD/product docs,
  policies, and configs when they constrain the task;
- ensure plan/task ledger exists for staged work;
- run the RRI calculator before implementation;
- wait for approval when RRI is 26 or higher;
- read `DESIGN.md` before UI/presentation work;
- run required review before development closure;
- sync status docs before reporting completion.

### Hook Wiring

Wire the same script into each agent environment:

```text
SessionStart:
  run: python3 scripts/agent-preflight.py --print-summary --mark

PreToolUse for Write/Edit:
  run: python3 scripts/agent-preflight.py --check
  if failed: deny the write/edit tool and tell the agent to run preflight
```

Portable caution:

- Do not hardcode absolute paths.
- Resolve repo root with `git rev-parse --show-toplevel`.
- Guard hooks so they only run inside the intended repository.
- Add `.agent/` to `.gitignore`.

DubBridge implementation:

- `scripts/agent-preflight.py`
- `scripts/agent_preflight_test.py`
- `.agent/session-preflight.json`
- Claude `SessionStart` and `PreToolUse` hooks
- Codex `SessionStart` and `PreToolUse` hooks

The preflight proves only that the session loaded the workflow summary. It does
not prove task-specific RRI, approval, or closure. Those remain per-task gates.

## ADR Discipline

If the target project uses ADRs, enforce propagation.

Portable rules:

- Accepted ADRs should not be deleted. Supersede or deprecate them.
- ADR status in frontmatter must match prose and index.
- New ADRs must be added to the ADR index.
- ADR references in docs, code comments, and migrations must resolve.
- When an ADR decision changes, review every canonical doc that describes that
  decision.
- Semantic consistency is human-owned; reference integrity can be automated.

Recommended propagation table:

| ADR change | Must review/update |
|---|---|
| New ADR | ADR index, architecture docs if boundaries change, roadmap, affected plan/tasks |
| Status change | ADR frontmatter, prose status, index status, docs that cite it |
| Scope change | ADR index, architecture, roadmap, affected plan/tasks |
| Decision change | Every canonical doc whose prose describes the decision |
| Superseded | Both ADRs, index rows, docs citing superseded ADR |
| Deletion/renumbering | Avoid for Accepted ADRs; otherwise update all refs atomically |

DubBridge implementation:

- `docs/adr/README.md`
- `scripts/check-doc-consistency.sh`
- `make qa-docs`

## BDD And Product Specs

BDD files are the behavioral bridge between product intent and executable
evidence.

Portable pattern:

- Store canonical `.feature` files under `docs/bdd/`.
- Give scenarios stable IDs.
- Map scenarios to tasks and executable evidence.
- Preserve test IDs or product-flow invariants that external tests depend on.
- Retrospective slices may map to shipped unit/integration evidence if no E2E
  flow exists.

Recommended `docs/bdd/README.md` table columns:

- Scenario ID.
- Description.
- Task.
- Executable evidence.
- UI/E2E flow, if applicable.
- HP/EC classification.

Agents should read BDD/product specs when they constrain the task. BDD files can
remain outside OKF frontmatter at first if they are non-Markdown `.feature`
files.

## Daily Operational Ledger

Daily notes are optional but useful when agents work over many sessions.

Portable purpose:

- connect current repo state to the day's work;
- record broken pipelines;
- record post-pipeline review findings;
- record drift between roadmap, ledgers, and git;
- capture blockers, debt, risks, and optimizations;
- seed tomorrow's work.

Recommended location:

```text
docs/daily/YYYY-MM-DD.md
```

Recommended automation:

```bash
bash scripts/daily-open.sh
```

Daily docs are operational notes, not canonical knowledge. It is reasonable to
exclude them from OKF frontmatter validation.

## QA Gates

### Local Command Surface

Expose stable targets via `make`, `just`, or an equivalent command runner.

Recommended portable targets:

| Target | Purpose |
|---|---|
| `qa-fmt` | formatting |
| `qa-lint` | static lint |
| `qa-test` | unit/integration tests |
| `qa-check` | compile/typecheck |
| `qa-local` | fast local baseline |
| `qa-coverage` | coverage threshold |
| `qa-docs` | docs consistency, OKF, task coverage, roadmap drift |
| `qa-rri` | RRI script tests |
| `qa-maintainability` | diff-based maintainability budget |
| `qa-review-budget` | local-model reviewability budget |
| `qa-config-secrets` | committed config secret scan |
| `qa-ci` | full local CI mirror |
| `install-hooks` | install git hooks |

Replace concrete commands by stack:

- Rust: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo check`,
  `cargo llvm-cov`, `cargo deny`.
- JS/TS: `npm run typecheck`, `npm run lint`, `npm test`.
- Python: `ruff`, `mypy`, `pytest`, `coverage`.
- Go: `gofmt`, `go vet`, `go test`, coverage tooling.

### Git Hooks

Use hooks to catch obvious misses before push:

- pre-commit: fast formatting/lint/docs checks;
- pre-push: detect changed path categories and run relevant gates.

Portable pre-push path categories:

- backend code;
- frontend/mobile code;
- docs/canonical workflow files;
- dependency manifests;
- migrations;
- configs;
- hooks/CI files.

DubBridge has two hook surfaces:

- `.githooks/pre-push` for the configured Git hooks path;
- `scripts/hooks/pre-commit` and `scripts/hooks/pre-push` for install-copy style.

When porting, pick one hook-install strategy and document it.

### CI

Remote CI should mirror local gates and add heavier checks.

Recommended jobs:

- docs consistency;
- roadmap drift;
- format;
- lint;
- tests;
- typecheck/compile;
- dependency policy;
- config secret scan;
- coverage;
- release build;
- maintainability;
- frontend/mobile gate;
- RRI tests.

Schedule-heavy jobs separately if daily scheduled CI should avoid expensive
nonessential tasks.

## Maintainability Gate

DubBridge uses a diff-based maintainability gate to reject generated-code bloat,
large pasted blocks, repeated lines, generic names, risky runtime calls, and long
lines in added backend/mobile code.

Portable value:

- enforce quality on new diff only;
- avoid breaking legacy code all at once;
- maintain separate budgets for source and test files;
- exempt generated files using explicit generated markers.

DubBridge implementation:

- `scripts/check-maintainability.py`
- `make qa-maintainability`

Adapt path classifiers and budgets to the target stack.

## Config And Secrets Gate

Committed config files should not contain secret-looking keys.

Portable rule:

- committed config profiles can contain non-secret defaults;
- secrets must come from injected environment, secret manager, or deployment
  config;
- reject keys whose path segment looks like password, secret, token, or key.

DubBridge implementation:

- `scripts/check-config-secrets.sh`
- `make qa-config-secrets`

Adapt the scanner if the target project stores public keys or non-secret tokens
with names that would otherwise trigger false positives.

## Post-Pipeline Push Review

This is an advanced optional layer.

Portable concept:

- primary CI remains authoritative;
- after CI completes, a self-hosted runner with local model access inspects run
  metadata, failed jobs, annotations, logs, and artifacts;
- findings are normalized into candidate tasks;
- pure Low findings may be routed to local developer delegation;
- Moderate+ or non-pure-Low findings become human-visible follow-up work;
- reports are committed or uploaded as artifacts.

DubBridge implementation:

- `.github/workflows/push-review.yml`
- `scripts/gemma-push-review.py`
- `scripts/push_review_commit.py`
- `make qa-gemma-push-review`

Do not treat push review as code review replacement. It is advisory and
post-pipeline.

Daily consumption rule:

- inspect the newest push-review report at daily open and close;
- record non-Low or non-pure-Low findings as human-visible follow-up work;
- keep delegated Low patches visible as `in_review` until a non-developer review
  is reconciled;
- if no report exists, record explicit absence instead of silence.

## Handoff Prompts

Keep handoffs small. A task was already approved or is Low-band.

Human-agent handoff prompt should contain only:

1. Task ID and one-line goal.
2. Governing docs, paths only.
3. One file and line range with the logic to change.
4. Exact acceptance criteria.
5. Stop condition: what to do last and what not to start next.

For local model delegation, use a structured delegation packet instead of a
human-agent handoff prompt.

## Language Policy

Portable recommendation:

- repository instruction files: English;
- code comments: English;
- task metadata and model IDs: exact identifiers;
- user-facing responses: user's language when appropriate.

This keeps machine-readable workflow artifacts consistent across agents.

## Communication And Epistemic Discipline

Portable agent communication should follow a doubt-first model:

- Do not agree by default. Verify claims against files, tool output, or trusted
  docs before treating them as true.
- Cite the source for claims about workflow rules, code behavior, status, and
  scores.
- If a claim cannot be sourced, say that it is an assumption.
- If the user request is ambiguous and a wrong assumption would be risky, ask a
  concise clarifying question.
- Challenge the agent's own output before reporting completion: check whether
  the result could be stale, incomplete, or contradicted by a governing doc.

This section is especially important for session-resuming agents. It reduces
silent drift caused by stale context or partial memory.

## Recommended Migration Plan For An Existing Project

### Phase 1: Minimal Control Plane

Add:

- `AGENTS.md`
- `README_AGENT_ORDER.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`
- `scripts/rri.py`
- `scripts/rri_test.py`

Result:

- agents know the workflow;
- tasks get scored;
- approval boundaries become explicit.

### Phase 2: Documentation Enforcement

Add:

- `docs/knowledge/README.md`
- `scripts/check_okf_frontmatter.py`
- `docs/plan/roadmap.md`
- `docs/plan/`
- `docs/tasks/`
- `docs/adr/README.md`
- docs consistency checks.

Result:

- plans and ledgers become machine-readable;
- ADR references stop dangling;
- roadmap drift becomes visible.

### Phase 3: Anti-Forgetting Hooks

Add:

- `scripts/agent-preflight.py`
- `scripts/agent_preflight_test.py`
- `.agent/` in `.gitignore`
- SessionStart hook;
- PreToolUse write/edit hook.

Result:

- fresh Claude/Codex sessions load workflow at startup;
- write/edit tools fail fast when preflight was not marked.

### Phase 4: Design And Product Contracts

Add:

- `DESIGN.md`
- `docs/bdd/`
- BDD mapping README;
- optional design lint.

Result:

- UI agents cannot improvise outside the design system;
- product behavior maps to tests and task cases.

### Phase 5: Local And CI Gates

Add:

- Makefile/justfile QA surface;
- git hooks;
- CI jobs;
- coverage;
- dependency policy;
- maintainability gate;
- config secret gate.

Result:

- workflow moves from convention to enforceable system.

### Phase 6: Local Model Review And Delegation

Add only after the previous phases are stable:

- local model transport wrapper;
- Low-RRI delegation wrapper;
- read-only reviewer wrapper;
- reviewability budget;
- D14 adjudicator packet builder;
- optional post-pipeline push review.

Result:

- simple safe patches can be delegated locally;
- code review is advisory but structured;
- large or ambiguous work escalates instead of overflowing context.

## Minimum File Tree For Port

```text
.
|-- AGENTS.md
|-- README_AGENT_ORDER.md
|-- DESIGN.md
|-- Makefile
|-- .gitignore
|-- .githooks/
|   `-- pre-push
|-- docs/
|   |-- architecture.md
|   |-- adr/
|   |   `-- README.md
|   |-- bdd/
|   |   `-- README.md
|   |-- daily/
|   |   |-- README.md
|   |   `-- TEMPLATE.md
|   |-- knowledge/
|   |   `-- README.md
|   |-- plan/
|   |   `-- roadmap.md
|   |-- playbooks/
|   |   `-- AGENT_WORKFLOW_GUIDE.md
|   |-- policies/
|   |   |-- HITL_AUTONOMY_POLICY.md
|   |   `-- RRI_POLICY.md
|   |-- proposals/
|   |-- prompts/
|   `-- tasks/
|-- scripts/
|   |-- agent-preflight.py
|   |-- agent_preflight_test.py
|   |-- check-doc-consistency.sh
|   |-- check_okf_frontmatter.py
|   |-- check-roadmap-drift.sh
|   |-- check-task-unit-coverage.sh
|   |-- check-maintainability.py
|   |-- check-config-secrets.sh
|   |-- rri.py
|   `-- rri_test.py
`-- .github/
    `-- workflows/
        `-- ci.yml
```

Local model files are optional:

```text
scripts/
|-- gemma_local.py
|-- delegate-low-rri.py
|-- gemma-code-review.py
|-- adjudicator-packet.py
|-- check-review-budget.py
|-- gemma-push-review.py
`-- push_review_commit.py
```

## Gap Review Checklist

Use this checklist after porting the workflow.

### Authority And Startup

- [ ] Exactly one workflow guide is highest authority.
- [ ] `AGENTS.md` does not contradict the workflow guide.
- [ ] SessionStart hook prints and marks preflight.
- [ ] PreToolUse write/edit hook checks preflight.
- [ ] `.agent/` is ignored by git.
- [ ] Hooks do not contain hardcoded source-project absolute paths.

### Planning

- [ ] Every staged workstream has a plan.
- [ ] Every staged workstream has a task ledger.
- [ ] Development tasks have HP/EC examples.
- [ ] Task ledgers record RRI and effort derived from RRI.
- [ ] Roadmap links to plan/task evidence.

### Approval

- [ ] RRI script runs before task presentation or delegation.
- [ ] RRI 26+ tasks require explicit approval before editing.
- [ ] RRI 56+ tasks decompose before implementation.
- [ ] Deletions, commits, pushes, migrations, and outward-facing actions require
      explicit approval.

### Implementation And Review

- [ ] One task is implemented at a time.
- [ ] Local model developer cannot approve or apply its own work.
- [ ] Local model reviewer is read-only.
- [ ] D14 fallback exists for local model failure/quorum failure or high-risk
      review triggers.
- [ ] Reviewability budget fails closed or routes to D14 with a documented
      override.

### Documentation

- [ ] OKF vocabulary is defined.
- [ ] OKF validator runs in docs gate.
- [ ] ADR index and ADR files stay consistent.
- [ ] Dangling ADR refs are rejected.
- [ ] Roadmap drift check runs.
- [ ] Daily notes are excluded or intentionally included in OKF.

### Design And Product

- [ ] `DESIGN.md` exists if UI work exists.
- [ ] UI task workflow requires reading `DESIGN.md`.
- [ ] Runtime token/component authority is documented.
- [ ] BDD/product specs are linked from relevant tasks.
- [ ] Stable scenario IDs or test IDs are preserved.

### QA

- [ ] Local QA targets exist.
- [ ] Git hooks run fast relevant gates.
- [ ] CI mirrors local gates.
- [ ] Coverage gate is defined.
- [ ] Dependency policy gate exists where applicable.
- [ ] Secret/config scan exists.
- [ ] Maintainability gate is tuned to the target stack.

### Closure

- [ ] Development closure checks review before coverage certification.
- [ ] Reflection log is required for RRI 26+ development tasks.
- [ ] Unit coverage certification maps every HP/EC to passing tests.
- [ ] Owner final verification records exact commands.
- [ ] Plan, task ledger, roadmap, ADR refs, and daily notes are synchronized
      before reporting completion.

## Final Porting Guidance

Start small. The minimal useful system is:

1. workflow authority;
2. RRI calculator;
3. task ledger;
4. approval gate;
5. preflight anti-forgetting script;
6. docs/OKF gate.

Add local model delegation only after the target project already has stable
plans, task ledgers, tests, and CI. Otherwise the local model layer will amplify
process gaps rather than reduce work.

The central invariant to preserve is this:

```text
No implementation starts until the agent has loaded the workflow, identified the
governing context, computed risk, and satisfied the approval gate for that task.
```
