# Required Reasoning Index (RRI) Policy

> **Status:** Active. Adopted by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` as the
> canonical method for complexity-and-risk scoring, model-tier selection, and
> autonomy-gate determination. `AGENT_WORKFLOW_GUIDE.md` is the highest authority;
> this file is the detailed procedure it delegates to.

## Purpose

RRI estimates how much reasoning, context, caution, and verification a task
requires before an AI agent may safely implement it.

RRI **determines the approval gate and evidence required** before an agent may
implement a task. For bands **RRI 26+**, the HITL approval checkpoint is
mandatory; what the band controls is what evidence the agent must bring to it.
For band **RRI 0–25**, the agent uses show-and-proceed — no approval checkpoint
(see `docs/policies/HITL_AUTONOMY_POLICY.md` for the full rule).

## Formula

```
RRI = 100 × ((0.18·C + 0.12·F + 0.15·D + 0.15·T + 0.12·A + 0.12·K + 0.10·P + 0.06·X) / 5)
    + Penalties
```

Weight verification: 0.18 + 0.12 + 0.15 + 0.15 + 0.12 + 0.12 + 0.10 + 0.06 = **1.00** ✓

Each variable is scored **0–5**. The base term is therefore in **[0, 100]**.
Penalties push the score above 100.

## Variables

### How to obtain each variable

Objective variables must be **measured**, not estimated.
Subjective variables must be **judged using the anchor rubric** below so that
two independent agents score the same task to the same number.

| Var | Name | Nature | How to obtain |
|---|---|---|---|
| **C** | Cyclomatic complexity | Objective (proxy) | Estimate via `CC = E − N + 2P` (count: `if`, `else if`, `match` arm, `while`, `for`, `loop`, `?` branches, `&&`/`\|\|` in conditions). Or use `clippy::cognitive_complexity` as a proxy. |
| **F** | Files affected | **Objective** | `git diff --name-only <base>...HEAD` — count the files. |
| **D** | Domain complexity | Subjective — anchor rubric | Classify the task's target path/crate using the anchor table. |
| **T** | Test-coverage risk | Semi-objective | Check `cargo llvm-cov` output for the affected file/module. If no tests exist in the area, score high. |
| **A** | Task ambiguity | Subjective | Is there a task file with acceptance criteria + happy/edge examples (required by the workflow guide)? Score near 0. Vague tasks score 5. |
| **K** | Coupling / side effects | Subjective — anchor rubric | Classify using the anchor table. |
| **P** | Public API / security / data impact | Subjective — anchor rubric (ADR-anchored) | Classify using the anchor table. |
| **X** | Context size required | Subjective | How many files/modules must the agent hold in mind? |

### Scoring bands per variable

**C — Cyclomatic complexity**

| Score | CC range |
|---|---|
| 0 | 1–5 |
| 1 | 6–10 |
| 2 | 11–20 |
| 3 | 21–30 |
| 4 | 31–50 |
| 5 | 50+ |

**F — Files affected**

| Score | Files |
|---|---|
| 0 | 1 |
| 1 | 2 |
| 2 | 3–5 |
| 3 | 6–10 |
| 4 | 11–20 |
| 5 | 20+ |

**D — Domain complexity**

| Score | Domain |
|---|---|
| 0 | Documentation, naming, formatting |
| 1 | Simple logic, constants, copy |
| 2 | Normal business logic |
| 3 | Integrations, workflows, state management |
| 4 | Platform-specific core logic, async orchestration, agent orchestration, permissions |
| 5 | Security, authentication, compliance, financial or critical data logic |

**T — Test-coverage risk**

| Score | Test state |
|---|---|
| 0 | Strong specific tests exist for the area |
| 1 | Reasonable tests exist |
| 2 | Partial tests exist |
| 3 | Weak or fragile tests |
| 4 | No tests in the affected area |
| 5 | No tests and critical logic |

**A — Task ambiguity**

| Score | Ambiguity |
|---|---|
| 0 | Exact task with acceptance criteria and happy/edge examples |
| 1 | Mostly clear |
| 2 | Some missing details |
| 3 | Requires significant interpretation |
| 4 | Very open-ended |
| 5 | Vague ("improve this", "make it better") |

**K — Coupling / side effects**

| Score | Coupling |
|---|---|
| 0 | Pure function |
| 1 | Isolated class or module |
| 2 | Internal module with contained side effects |
| 3 | Database, API, filesystem, external service, or framework integration |
| 4 | Async behavior, events, queues, transactions, platform side effects |
| 5 | Distributed system behavior or critical external side effects |

**P — Public API / security / permissions / data impact**

| Score | Impact |
|---|---|
| 0 | No impact |
| 1 | Minor internal impact |
| 2 | Changes internal behavior |
| 3 | Changes internal API |
| 4 | Changes public API, permissions, ownership, data visibility, or persisted data |
| 5 | Security, authentication, authorization, data loss, compliance, or critical business risk |

**X — Context size required**

| Score | Scope |
|---|---|
| 0 | One function |
| 1 | One class or file |
| 2 | 2–5 files |
| 3 | One complete module |
| 4 | Several modules or crates |
| 5 | Multi-repository or global architecture context |

## DubBridge anchor rubric

Use this table to derive the **minimum floor** for D, P, and K when the task
touches these paths or crates. Score higher if the specific change within the
path warrants it; never score lower than the floor.

| Task touches | D floor | P floor | K floor | ADR anchor |
|---|---|---|---|---|
| `docs/**`, naming, formatting, `config/*.toml` (non-secret) | 0 | 0 | 0 | — |
| `config/*.toml` with env-wiring logic, `config/README.md` | 1 | 1 | 1 | ADR-026 |
| Internal crate business logic (`crates/qc`, `crates/media` builders, `crates/providers`, `crates/domain`) | 2 | 2 | 2 | — |
| `crates/db`, `crates/storage`, `crates/jobs`, `crates/connectors`, `crates/ingestion`, `crates/observability`, async orchestration, HTTP proxy logic | 3 | 3 | 3 | ADR-006, ADR-018 |
| `apps/gateway/src/auth/**`, token handling, session/cookie management, CSRF | 4 | 4 | 4 | ADR-024 |
| `crates/auth`, JWT boundary, principal propagation | 4 | 4 | 4 | ADR-023 |
| `crates/audit`, rights-ledger path (`crates/domain` rights types), `infra/migrations/**` | 4 | 5 | 4 | ADR-008, ADR-018 |
| Secrets, credential storage, authentication/authorization system boundary | 5 | 5 | 5 | ADR-023, ADR-024, ADR-025 |

## Penalties

Apply each penalty independently; they are additive.

| Condition | Penalty |
|---|---|
| Refactor and functional behavior change combined in the same task | +8 |
| Tests missing **and** public/security/data impact is high (P ≥ 4) | +10 |
| Cyclomatic complexity > 30 (C ≥ 4) **and** domain complexity ≥ 3 (D ≥ 3) | +10 |
| Task touches authn, authz, permissions, security, ownership, or sensitive data | +10 |
| Task is likely to affect more than 10 files (F ≥ 4) | +8 |
| An architecture or process/policy decision is required | +12 |
| No clear verification strategy exists | +15 |

## Bands, autonomy gates, and model tiers

The HITL approval requirement applies at every band **except RRI 0–25**, which
uses show-and-proceed (see below). For all other bands, what the band controls is
the evidence and gates the agent must satisfy before and after that approval.

| RRI | Label | Gate | Tier | Thinking |
|---|---|---|---|---|
| **0–25** | Low | **Auto-execute:** present the RRI table and a one-line summary of intended actions, then begin implementation immediately — no approval checkpoint, no pause. | Economy | Off |
| **26–40** | Moderate | Confirm tests exist in the affected area. | Balanced | Off |
| **41–55** | Med-high | Plan + explicit acceptance criteria required before approval. | Balanced → Premium | On |
| **56–70** | Complex | Plan first. Do not implement before producing and approving a clear plan. Human reviews the plan. | Premium | On |
| **71–85** | High | Characterization tests + explicit acceptance criteria + human reviews the **diff** (not just the plan). | Premium | On |
| **86–100** | Very high | Do not implement directly. Produce an ADR + risk analysis + decompose into subtasks. | Premium | On |
| **> 100** | Excessive | Architecture/design work must happen first. Re-scope before any implementation. | Premium | On |

### Model tier resolution

Tier names map to the current DubBridge agent environment as follows. Resolve
against official vendor documentation at task-presentation time — do not rely on
stale memory for "latest" or "best".

| Tier | Claude Code | Codex |
|---|---|---|
| Economy | Claude Haiku (current family) | Economy coding model |
| Balanced | Claude Sonnet (current family) | Balanced coding model |
| Premium | Claude Opus (current family) | Premium reasoning/coding model |

Thinking mode: activate for Balanced→Premium and above when the task requires
multi-step reasoning that cannot be validated incrementally. Do **not** activate
for config edits, doc updates, or tasks where the strategy is fully pre-defined.

## Decomposition triggers

Split a task into subtasks before implementing if **any** of the following apply:

- RRI > 70, or base RRI > 100 (before penalties).
- F ≥ 4 **and** K ≥ 3 — large change surface with high coupling; isolate each seam.
- C ≥ 4 **and** D ≥ 3 — the +10 penalty activates; separate complex logic into a
  testable unit first.
- The +8 penalty is active (refactor + behavior change combined) — always separate
  refactor from functional change into distinct tasks/commits.
- T ≥ 4 **and** P ≥ 4 (no tests + high impact) — first subtask must be
  characterization tests; implementation is the second subtask.

**Split target:** divide until each subtask scores RRI ≤ 55 with A ∈ {0, 1}
(own acceptance criteria + happy/edge examples per the workflow guide).

## Reporting format

Before every implementation, present the RRI as a table:

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0–5 | How obtained (CC formula / clippy / estimate) | High / Medium / Low |
| F files | 0–5 | `git diff` count | High |
| D domain | 0–5 | Anchor rubric row | High / Medium |
| T coverage | 0–5 | llvm-cov output or "no tests found" | High / Medium |
| A ambiguity | 0–5 | Task file has/lacks criteria + examples | High |
| K coupling | 0–5 | Anchor rubric row | High / Medium |
| P impact | 0–5 | Anchor rubric row + ADR cited | High |
| X context | 0–5 | Files/modules required | Medium |

Then state:
- **Base value:** `100 × (Σ / 5) = ___`
- **Penalties applied:** list each triggered penalty and its value.
- **Final RRI:** base + penalties = ___ → band ___ → tier ___ / thinking ___.
- **Gates for this band:** list the gates that apply.

Low-confidence scores on D, P, or K are themselves a signal: treat the variable
as one step higher when confidence is Low.

## Related

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — highest authority; adopts this policy
- `docs/policies/HITL_AUTONOMY_POLICY.md` — approval requirements and show-and-proceed rule
- `docs/tasks/rri-integration.md` — integration task ledger
