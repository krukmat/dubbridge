---
type: TaskList
title: "Tasks: Compact Approval Task Card v2"
plan: docs/plan/compact-approval-task-card.md
status: done
rri: 28
band: Moderate
effort: M
---

# Tasks: Compact Approval Task Card v2

## T1 - Define and synchronize the compact approval-card contract

- **Status:** [x] Done
- **Type:** policy/docs-only
- **Effort:** M
- **RRI:** 28 -> Moderate
- **Dependencies:** none

### Objective

Replace the long approval presentation with a compact projection focused on
RRI, scope, resolved agents by phase, gates, and diagrams, while retaining full
audit detail in linked artifacts.

### Scope

- In: the nine files named by the linked plan.
- Out: runtime scripts, RRI scoring semantics, historical task cards, and card
  lint automation.

### Acceptance criteria

1. The authoritative guide defines a compact card with no more than six content
   blocks and a resolved per-phase workflow table.
2. RRI detail may live in a linked task/RRI artifact while the approval card
   shows score, band, gates, penalties, dominant drivers, and model routing.
3. The card names orchestrator, phase-1 reviewer, approver, implementer,
   Reflection/verifier, phase-2 reviewer, closure owner, and relevant fallbacks.
4. `AGENTS.md`, `CLAUDE.md`, RRI policy, HITL policy, and the portable proposal
   no longer contradict the RRI 26-55 reviewer route.
5. A reusable template implements the new contract.
6. Documentation QA passes.

### Evidence to emit

- RRI script output recorded below.
- `make qa-docs` result.
- Search result showing no active stale RRI 26-55 cross-vendor routing language.

### Status artifacts affected

- `docs/plan/compact-approval-task-card.md`
- `docs/tasks/compact-approval-task-card.md`

### RRI evidence

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | agent-supplied score | High |
| F files | 3 | `--touches` -> 9 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 3 | agent-supplied (no rubric match) | High |
| P impact | 2 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 28

**Penalties applied:** none

**Final RRI:** 28 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off

**Gates for this band:** Confirm tests exist in the affected area.

**Decomposition:** not triggered

**Advisory:** AGENTS.md: no anchor-rubric match — agent judgment governs D/P/K

**Advisory:** CLAUDE.md: no anchor-rubric match — agent judgment governs D/P/K

The user explicitly waived the human approval checkpoint for this bounded task.

Task-analysis review: n/a - policy/docs-only exemption

### Execution summary

1. Establish plan and ledger.
2. Amend the authoritative compact-card and reviewer-routing rules.
3. Synchronize summaries and policies.
4. Add the reusable template.
5. Run documentation validation and close status artifacts.

### Completion evidence

- Added the authoritative six-block Compact Approval Task Card v2 contract and
  moved full RRI detail to the linked ledger/artifact boundary.
- Added resolved per-phase ownership for orchestrator, reviewers, human gate,
  implementer, Reflection/verifier, fallbacks, and closure.
- Added `docs/templates/compact-approval-task-card.md`.
- Synchronized `AGENTS.md`, `CLAUDE.md`, RRI policy, HITL policy, and the
  portable workflow proposal.
- Consolidated RRI 26-55 independent review as
  `qwen3.6:27b-q4_K_M -> Gemma -> D14 -> BLOCKED` for both non-exempt phase 1
  and phase 2.
- Stale-routing search across active contracts returned no matches.
- `make qa-docs` passed on 2026-07-26: documentation consistency, task coverage
  checks, roadmap drift, and OKF frontmatter all passed.

Code-solution review: n/a - policy/docs-only exemption
