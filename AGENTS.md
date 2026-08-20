# AGENTS.md

## Purpose

This file defines the default task-presentation contract for agents working
in the `dubbridge` repository. It is a **summary**, not a replacement for the
canonical guides. Use it to route into those sources only when the current
task requires them:

- `README_AGENT_ORDER.md` — orientation and reading order.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — authoritative for all workflow
  topics: the mandatory plan → tasks → approval → implement flow, model
  resolution, and closure gates.
- `docs/policies/HITL_AUTONOMY_POLICY.md` — human-in-the-loop approval rules.
- `docs/adr/` — architecture decisions that constrain implementation.
- `docs/plan/roadmap.md` — slice sequence, dependencies, and where each
  slice/task sits.

`CLAUDE.md` (project and global) is authoritative only for topics the
workflow guide does not cover.

## Context Loading Policy

`AGENTS.override.md` is the always-loaded Codex bootstrap and is a generated,
byte-exact projection of this file. Do not inline the full workflow guide,
policies, roadmap, architecture overview, ADR corpus, or task ledgers into it.
Load canonical detail only when the current task requires it:

- Before presenting or executing staged work, read
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, the active `docs/plan/<slice>.md`,
  and `docs/tasks/<slice>.md` completely enough to apply their current gates.
- Read `docs/policies/HITL_AUTONOMY_POLICY.md` and
  `docs/policies/RRI_POLICY.md` when approval, autonomy, scoring, routing,
  fallback, or review evidence is in scope.
- Read `docs/plan/roadmap.md` for slice status/dependencies and
  `docs/architecture.md` plus only the applicable files in `docs/adr/` when a
  task touches product sequencing, runtime boundaries, or architecture
  decisions.
- Read task-specific BDD/product/design/config sources only when they
  materially constrain the requested work. For mobile UI/presentation work,
  `DESIGN.md` is mandatory.

Do not bulk-load canonical documents merely because they are linked here. A
link is a routing instruction, not duplicated operative prose; once loaded,
the canonical source controls any summary conflict.

## Non-negotiable Safety And Closure

- Preserve user-owned worktree changes and keep edits inside the authorized
  task scope. Never commit, push, delete, overwrite user data, or perform an
  external write without the applicable explicit approval.
- Run `scripts/rri.py` before presenting or delegating executable staged work.
  RRI 26+ requires the workflow guide's approval/review route; RRI 0–25 skips
  the full card. Any Ollama-backed role requires the per-task restart/precheck.
- Before reporting completion, run relevant verification, record mandatory
  review/coverage evidence for development tasks, and synchronize every
  materially affected plan, task, roadmap, ADR, or status artifact.
- Do not commit with failing tests. Ask before deletion. Report failures,
  skipped gates, assumptions, and unresolved risks plainly.

## Task Presentation Rule

Present the next task before execution when the active workflow requires
approval. Verify current requirements in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
before presenting or executing any staged task — task-type-specific
requirements there are mandatory even when not restated here.

Before marking a development task done, explicitly determine whether it is
exempt (docs/config/migration/planning/ADR/task-ledger/policy-only) or
whether the band-resolved independent review is required before citing unit
coverage certification or owner final verification.

## Required Task Presentation Format

For RRI 26+, use the six-block **Compact Approval Task Card v2** defined by
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Step 3` and instantiated at
`docs/templates/compact-approval-task-card.md`: Decision header, Scope and
acceptance, Agent workflow, Diagrams, References, Approval checkpoint (full
block contents in the workflow guide). Do not copy the full task definition
or RRI variable table into the card — those stay in the linked task
ledger/RRI artifact. RRI 0–25 tasks skip the full approval card under the
Low-band route.

## Live per-task todo list (Claude Code and Codex)

Every orchestrator must keep a **live, per-task todo/checklist** mirroring
the card's `Agent workflow` block, kept current as the task moves through
phases — seeded before implementation, normally one entry `in_progress` at a
time, flipped to `completed` only once that phase's gate has passed, with a
`BLOCKED`/escalated entry kept visible until resolved. Full contract, phase
sets by band, and the mandatory `Restart Ollama + local-stack precheck`
prepend for any task invoking a local role:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Live per-task phase todo list` and
§ Mandatory workflow before implementing, Step 0.

## Complexity And Model Guidance

**When RRI has been computed**, the `Complexity` field must use the RRI band
name:

| RRI range | Complexity to present |
|---|---|
| 0–25 | Low |
| 26–40 | Moderate |
| 41–55 | Med-high |
| 56–70 | Complex |

Fallback only when no RRI is available: `Effort: S` → `Low`, `M` → `Medium`,
`L` → `High`.

Codex and Claude Code model resolution both live in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (§ "Current Codex cloud-takeover
resolution", § "Current Claude Code capability resolution") — this file
carries no copy of concrete model names, so there is exactly one place to
re-verify against vendor guidance. Task-local model pins override those
defaults until explicitly updated. If a task file already defines explicit
complexity/model guidance, it overrides this file.

## Human-selected fallback checkpoint

Before any terminal local fallback invokes D14 or a cloud implementer, emit
the ADR-039 `fallback-selection-v1` artifact bound to the exact fallback
packet. `human-select` is the default — without a complete human
model/effort/selector, stop as `awaiting_fallback_selection`.
`preauthorized` is allowed only when those fields were frozen in the
approved card or preflight. Neither waives HITL nor changes RRI, reviewer
independence, D14's read-only Balanced role, repair budgets, or task scope.
See ADR-039 and `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Human-selected
fallback checkpoint`.

## Pseudocode Rule

Include pseudocode only for branching logic, multi-stage data
transformations, a reusable workflow that benefits from an execution sketch,
or risk that's easier to evaluate as a structured outline. Skip it for
trivial file creation, simple edits, or direct command execution.

## Diagram Rule

Every approval card includes a compact agent-workflow Mermaid diagram;
development tasks add a technical-scope diagram (flow, boundary, dependency
direction, state transition, or ownership split). Non-development tasks add
a second diagram only if the task changes architecture boundaries, spans
multiple services/crates/workers/repos, introduces a pipeline/state machine,
or is materially easier to approve as a compact flow. Never exceed two
diagrams; simple docs-only tasks normally use only the agent-workflow one.

## Related Documents Rule

List only documents that materially constrain the task, in priority order:
task file, linked plan, workflow/policy files, ADRs, prompt files,
configs/templates. For mobile UI/presentation tasks under `mobile/`, include
root `DESIGN.md` when it materially constrains the visual work — it governs
design intent; plan/task files remain authoritative for behavior, acceptance
criteria, and verification.

## Approval Boundary

When the workflow requires approval before executing a task, end the
presentation with:

`Execution has not started. Approve this task to proceed.`

If no approval is required, continue under the gate defined by
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and `docs/policies/RRI_POLICY.md`.
`Effort: S` normally corresponds to the **RRI 0–25** Low band, which skips
the full approval presentation — use bounded local Qwen Developer delegation
only for eligible simple code patches, otherwise handle directly as the
primary agent under the low-band gate.

## Band-routed peer review report lines

Every task card needs a phase-1 line; every development closure report needs
a phase-2 line, reviewer resolved by RRI band. Docs/config/migration/ADR/
plan/task-ledger/policy-only tasks record `n/a` for phase 2.

```
Task-analysis review: <gemma|muse-glimmer|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
Code-solution review: <gemma|muse-glimmer|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
```

`muse-glimmer` — primary for RRI 0–25, intermediate fallback for 26–55.
`gemma` — primary for 26–55, intermediate fallback for 0–25. `codex|claude`
— RRI 56+, resolved from caller identity (`claude-code → codex`,
`codex → claude`, others → `claude`). `d14` — final fallback when the
preceding chain is unusable, always via a responsive cross-provider reviewer
first (same-provider only as a recorded degraded fallback). `BLOCKED` —
non-pass verdict or the whole chain unavailable; stops presentation/closure
until revised, user-waived, or reported blocked. Full contract:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`.

## Development Closure Rule

Do not describe certification, final verification, or status flips as the
first completion step. First determine whether the task must pass the
mandatory code-solution review gate resolved by the RRI band table in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and
`docs/policies/HITL_AUTONOMY_POLICY.md`, then describe the remaining closure
blocks in order.

## Language

Agent-facing repository instructions: English. User-facing presentation may
be adapted to the user's language; task metadata, filenames, and model
identifiers stay exact.
