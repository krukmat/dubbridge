---
type: Plan
title: "Plan: Workflow-policy consolidation after documentation reduction"
status: completed
slice: workflow-policy-consolidation
---

# Plan: Workflow-policy consolidation after documentation reduction

> **Status:** Complete. T0–T7 closed on 2026-08-20; T7 is the bounded,
> user-approved follow-up that reduced Codex's always-loaded context.
> **Tasks ledger:** `docs/tasks/workflow-policy-consolidation.md`
> **Progress:** T0 through T7 are complete.

## Objective

Complete the workflow-policy documentation reduction without retaining
contradictory active instructions, stale implementation status, inaccurate archive
claims, or a stale generated `AGENTS.override.md` projection.

The existing uncommitted reduction is the input to this work. Each implementation
task must preserve its useful reductions and make narrow, evidence-backed edits;
it must not revert the working tree wholesale.

## Decisions

- **D1 — one authority rule:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is the
  highest authority for covered workflow topics. `CLAUDE.md` applies only where
  the guide is silent. Every active orientation or policy summary must state the
  same relationship.
- **D2 — one post-local-failure route:** a Moderate whole-task repair exhaustion
  first triggers scored Low-band decomposition. Cloud implementation is available
  only as the documented last resort. Med-high has no whole-task local
  `GO_LOCAL` authoring route; only an independently eligible ADR-040 module
  tramo may be local.
- **D3 — one local developer binding:** active documentation uses **Qwen
  Developer** (`qwen3.8:27b-mlx`) for eligible local code authoring. Gemma and
  Muse Glimmer remain reviewer roles as resolved by band. Historical references
  remain only where explicitly marked as historical.
  T3's narrow update to `scripts/rri.py` and its paired unit tests was explicitly
  authorized by the user on 2026-08-20 because the calculator emitted the stale
  developer binding; it is an exception to the otherwise excluded local-model
  script changes.
- **D4 — current state stays active; history stays archived:** ADR-031/S-200's
  backend-issued HS256 bearer-JWT model is the current mobile-auth description.
  Superseded opaque-session details belong in clearly labelled historical text.
- **D5 — archives do not overclaim:** an archive either contains the promised
  complete relocated material or is labelled as a rationale/excerpt. This plan
  prefers the latter when the omitted material is not needed for the active
  procedure.
- **D6 — generated output is never hand-maintained:** after source edits,
  regenerate `AGENTS.override.md` with `scripts/generate-agents-override.py`.
- **D7 — bootstrap, not corpus:** `AGENTS.override.md` contains the compact
  cross-agent contract in `AGENTS.md` only. The workflow guide, HITL policy,
  roadmap, architecture overview, ADRs, and active plan/task ledger remain
  canonical sources loaded on demand through explicit routing instructions.

## Scope

### Included

- Active workflow authority, routing, model-role, and fallback wording.
- The generated `AGENTS.override.md` projection and its source documents.
- Current mobile-auth status in the roadmap and architecture overview.
- Accuracy of claims made by the new workflow-detail archive.
- Deterministic documentation and focused prompt-anchor validation.

### Excluded

- Changing the underlying RRI formula, ADR decisions, local-model scripts, or
  deployment/runtime code.
- Rewriting historical records merely to use current terminology.
- Any commit, push, or external action.

## Execution order

| Order | Task | Why it precedes the next task |
|---:|---|---|
| 0 | T0 — Capture plan and task ledger | Establishes the bounded repair set and this order. |
| 1 | T1 — Normalize authority and generated projection sources | Removes the precedence ambiguity that governs every later wording decision. |
| 2 | T2 — Normalize implementation-routing and fallback wording | Applies the approved authority rule to the local/cloud route. |
| 3 | T3 — Normalize local-role names | Aligns role terminology with the already implemented bindings. |
| 4 | T4 — Reconcile current mobile-auth status | Separates ADR-031/S-200 current state from superseded history. |
| 5 | T5 — Correct archive scope claims | Ensures reduction did not create a misleading preservation record. |
| 6 | T6 — Regenerate, validate, and close | Validates the complete resulting source set once, at the end. |
| 7 | T7 — Replace the full projection with a bounded bootstrap | Removes repeated always-loaded context while preserving deterministic drift checks and task-time source loading. |

```mermaid
flowchart LR
  T0["T0 plan + ledger"] --> T1["T1 authority"] --> T2["T2 routing"] --> T3["T3 role names"]
  T3 --> T4["T4 current auth state"] --> T5["T5 archive integrity"] --> T6["T6 projection + QA"]
  T6 --> T7["T7 bounded Codex bootstrap"]
```

## Affected-document map

| Concern | Primary documents | Supporting evidence |
|---|---|---|
| Authority | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`, `README_AGENT_ORDER.md`, `AGENTS.md` | `CLAUDE.md` only for uncovered topics |
| Routing and roles | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`, `README_AGENT_ORDER.md` | `docs/policies/RRI_POLICY.md`, `scripts/delegate-low-rri.py` |
| Mobile auth status | `docs/plan/roadmap.md`, `docs/architecture.md` | ADR-031, `docs/tasks/s-200-mobile-jwt-credential-auth.md` |
| Archive integrity | `docs/audit/agent-workflow-guide-detail-archive.md` | active Step 0 in the workflow guide |
| Generated bootstrap | `AGENTS.override.md` | `AGENTS.md`, `README_AGENT_ORDER.md`, and `scripts/generate-agents-override.py` |

## Implementation constraints

- Treat each task as docs/policy work; calculate RRI immediately before an
  executable task is presented if the active workflow requires it.
- Re-read the named authoritative source and supporting evidence before editing;
  source prose, not this plan, decides any contested wording.
- Do not preserve a conflict merely because both versions are shorter.
- Keep active documents directive-only. Move history to the existing archive only
  when it is useful context and accurately labelled.
- Keep `AGENTS.override.md` within the generator's explicit byte budget so a
  future source expansion fails closed instead of silently reintroducing a large
  always-loaded prompt.

## Verification and closure

T6 ran:

```bash
python3 scripts/generate-agents-override.py --write
make qa-docs
make qa-roadmap-drift
python3 -m unittest scripts/local-agent/prompt_anchors_test.py scripts/local-agent/prompt_builder_test.py
git diff --check
```

That closeout remains the evidence for T0–T6. T7 reopens only the generated
instruction-loading mechanism; it does not reopen the policy decisions already
closed. When this work is committed, its staged change must include
`docs/audit/agent-workflow-guide-detail-archive.md` and
`docs/audit/roadmap-history.md`.

T7 closed with a byte-exact `AGENTS.md` bootstrap, a 24 KiB fail-closed
generator ceiling, 15 focused generator tests, 5 drift tests, 96% focused
generator coverage, both documentation QA targets, and a final Gemma phase-2
`PASS`. Exact routing, Reflection, coverage, and owner-verification evidence is
recorded in `docs/tasks/workflow-policy-consolidation.md § T7`.
