# AGENTS.md

## Purpose

This file defines the default task-presentation contract for agents working in the `dubbridge` repository.

It works together with the canonical agent guides that govern implementation in
this repository. Read them before executing work:

- `README_AGENT_ORDER.md` — orientation and reading order for agents.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — the mandatory plan → tasks → approval →
  implement workflow.
- `docs/policies/HITL_AUTONOMY_POLICY.md` — human-in-the-loop approval rules.
- `docs/adr/` — architecture decisions that constrain implementation.
- `docs/plan/roadmap.md` — the general plan: slice sequence, dependencies, and where
  each slice/task sits.

For workflow topics, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is authoritative.
`CLAUDE.md` (project and the user's global) remains authoritative only for
topics not overridden there.

## Task Presentation Rule

When a user asks an agent to execute a staged task or a task from a task file, the agent must present the next task before execution when the active workflow requires approval.

The presentation must be concise but operationally complete.

Before presenting or executing any staged task, the agent must verify the
current requirements in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`. This file is a
presentation contract summary, not a replacement for the workflow guide.
Task-type-specific requirements defined there are mandatory even when they are
not restated verbatim below.

When answering questions about development-task completion or before marking a
development task done, the agent must explicitly determine whether the task is
exempt (docs-only, config-only, migration-only, planning, ADR, task-ledger, or
policy-only) or whether the workflow requires the band-resolved independent
review before citing unit coverage certification or owner final verification.

## Required Task Presentation Format

For RRI 26+ approval presentations, use the six-block **Compact Approval Task
Card v2** defined by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and instantiated at
`docs/templates/compact-approval-task-card.md`:

1. `Decision header` — task identity/status, RRI/band, Effort, approval gate,
   Codex/Claude recommendations, resolved primary implementation route, cloud
   takeover trigger/model, penalties, dominant RRI drivers, and link to the
   full RRI evidence.
2. `Scope and acceptance` — objective, in scope, out of scope, primary `HP-#` /
   `EC-#` behaviors for development tasks, evidence, and status sync.
3. `Agent workflow` — the resolved orchestrator, phase-1 reviewer, human gate,
   implementer, Reflection/verifier, phase-2 reviewer, and closure owner. State
   each responsibility, gate, and fallback.
4. `Diagrams` — one agent-workflow diagram; development tasks add one technical
   scope diagram. Never exceed two diagrams.
5. `References` — task, plan, and only materially governing documents.
6. `Approval checkpoint` — required wording or explicit bounded user waiver.

Do not copy the full task definition or RRI variable table into the approval
card. Those remain in the linked task ledger/RRI artifact. RRI 0–25 tasks still
skip the full approval card under the Low-band route.

`Evidence to emit` and `Status artifacts to sync` remain part of the execution
contract; summarize them in the card and keep their full paths in the task ledger.

## Live per-task todo list (Claude Code and Codex)

Block 3 (`Agent workflow`) is a frozen snapshot taken at presentation time.
Every orchestrator — Claude Code and Codex alike — must additionally keep a
**live, per-task todo/checklist** that mirrors block 3's rows and stays
current as the task actually moves through phases: seed it before
implementation starts, keep normally one entry `in_progress` at a time, flip
an entry to `completed` only once that phase's own gate has passed, and keep
a `BLOCKED` or escalated entry visible (never silently dropped) until it is
resolved, user-waived, or reported blocked. If a phase reroutes mid-task
(local implementer escalates to cloud, a reviewer falls back down its chain),
update that entry's named responsible agent/model to the actual participant.

Use whichever native mechanism the orchestrator has — Claude Code's
`TodoWrite` tool, Codex's own plan/task tracking — as long as it renders an
equivalent visible list naming the resolved responsible agent per phase, not
a generic role label. This list is a transparency/tracking artifact, not an
approval or review gate: an entry marked `completed` still requires that
phase's own evidence to exist. RRI 0–25 and docs/config/migration/ADR/plan/
task-ledger/policy-only tasks use the reduced phase set defined in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Live per-task phase todo list`,
which is authoritative for the full contract.

For any task that will invoke an Ollama-backed local role, prepend the explicit
`Restart Ollama + local-stack precheck — <resolved orchestrator>` item before
the first local-model call. This restart is mandatory once per task even when
the server is healthy; retries and later local phases of the same task reuse the
restarted server unless it becomes unavailable or wedged. The authoritative
sequence and task-boundary rules are in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before
implementing`, Step 0.

## Complexity And Model Guidance

**When RRI has been computed**, the `Complexity` field in the task presentation must
use the RRI band name — not the Effort-based mapping below:

| RRI range | Complexity to present |
|---|---|
| 0–25 | Low |
| 26–40 | Moderate |
| 41–55 | Med-high |
| 56–70 | Complex |

The Effort → Complexity mapping is a **fallback** used only when no RRI is available:

- `Effort: S` -> `Complexity: Low`
- `Effort: M` -> `Complexity: Medium`
- `Effort: L` -> `Complexity: High`

Current Codex cloud-takeover defaults (re-verify against official vendor guidance
at task-presentation time):

- RRI 0–25 bounded Low-band cloud escalation: `gpt-5.6-luna` at `low`;
  `gpt-5.6-terra` at `low` if Luna is unavailable.
- RRI 26–40 local-first fallback: `gpt-5.6-terra` at `medium`.
- RRI 41–55: operational-only fallback uses `gpt-5.6-terra` at `high`;
  capability/risk takeover uses `gpt-5.6-sol` at `high`.
- RRI 56+: cloud-primary uses `gpt-5.6-sol`, with effort resolved by the
  canonical workflow table; RRI 86+ remains analysis/decomposition only.

Claude Code model resolution remains provider-current and follows the canonical
workflow guide. Task-local model pins override these defaults until explicitly
updated.

Escalation guidance (Claude side resolves via the dated "Current Claude Code
capability resolution" table in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, not
a pin here):

- use `claude-opus-5` only when the task is long-context heavy, synthesis-heavy, or repeatedly stalls under `claude-sonnet-5`
- if a task is primarily code editing, repo navigation, shell execution or deterministic implementation work, keep Codex as the default

If a task file already defines explicit complexity or model guidance, that task-local guidance overrides this file.

## Human-selected fallback checkpoint

Before any terminal local fallback can invoke D14 or a cloud implementer, emit the
ADR-039 `fallback-selection-v1` artifact bound to the exact fallback packet.
`human-select` is the default: without a complete human model, reasoning-effort,
and selector choice, stop as `awaiting_fallback_selection`. `preauthorized` is
allowed only when those exact fields were frozen in the approved task card or
preflight; validate the receipt against the current packet, then use exactly its
selected model and effort. This bounded checkpoint neither waives HITL nor changes
RRI, reviewer independence, D14's read-only Balanced role, repair budgets, or task
scope. See ADR-039 and `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` for the full
protocol.

## Pseudocode Rule

Include pseudocode only when at least one is true:

- the task has branching logic
- the task transforms data across multiple stages
- the task defines a reusable workflow that benefits from an execution sketch
- the implementation risk is easier to evaluate through a structured outline

Do not add pseudocode for trivial file creation, simple edits, or direct command execution.

## Diagram Rule

Every approval card includes a compact agent-workflow Mermaid diagram. For
development tasks, also include a technical-scope diagram showing the relevant
flow, boundary, dependency direction, state transition, or ownership split.

For non-development tasks, add no second diagram unless at least one is true:

- the task changes architecture boundaries
- the task spans multiple services, crates, workers or repositories
- the task introduces a pipeline, state machine or dependency flow
- the task is easier to approve when shown as a compact technical flow

Never exceed two diagrams. Simple documentation-only tasks normally use only the
agent-workflow diagram.

## Related Documents Rule

The agent must list only the documents that materially constrain the task. Avoid dumping broad reading lists when only a few files are directly relevant.

Priority order:

1. task file
2. linked plan
3. workflow/policy files
4. ADRs
5. prompt files
6. configs/templates

For mobile UI / presentation tasks under `mobile/`, include root `DESIGN.md` in
`Related documents` when it materially constrains the visual work. Treat it as the
mobile design-intent contract, while plan/task files remain authoritative for
behavior, acceptance criteria, and verification.

## Approval Boundary

If the current workflow says the agent must wait for approval before executing a task, the presentation must end with a direct approval checkpoint.

Recommended wording:

`Execution has not started. Approve this task to proceed.`

If no approval is required under the active workflow, the agent may continue under
the gate defined by `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and
`docs/policies/RRI_POLICY.md`.

Under the canonical RRI mapping in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and
`docs/policies/RRI_POLICY.md`, `Effort: S` normally corresponds to the **RRI 0–25**
Low band. Those tasks skip the full approval presentation; use bounded local
Qwen Developer delegation through Ollama only for eligible simple code patches, and
otherwise handle them directly as the primary agent while still following the
low-band gate.

## Band-routed peer review report lines

Every task card must include a phase-1 line, and every development closure report
must include a phase-2 line. The reviewer token is resolved by RRI band at report
time. Docs-only, config-only, migration-only, ADR, plan, task-ledger, and
policy-only tasks record `n/a` with the exemption stated for phase 2.

```
Task-analysis review: <gemma|muse-glimmer|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
Code-solution review: <gemma|muse-glimmer|codex|claude|d14> <artifact path> - <PASS|BLOCKED>
```

- `muse-glimmer` — primary reviewer for RRI 0–25; intermediate fallback for RRI 26–55.
- `gemma` — primary reviewer for RRI 26–55; intermediate fallback for RRI 0–25.
- `codex | claude` — RRI 56+, resolved from caller identity
  (`claude-code → codex`, `codex → claude`, others → `claude`).
- `d14` — final fallback when the preceding reviewer chain is unusable.
- `BLOCKED` — non-pass verdict or the band's full reviewer/fallback chain is
  unavailable. Stops presentation (phase 1) or closure (phase 2) until revised,
  user-waived, or reported blocked.

See `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Band-routed peer review` for the
full contract.

For every band, D14 must first use a responsive provider different from the
primary orchestrator's provider. Same-provider D14 is allowed only as a
recorded degraded fallback after the cross-provider attempt is unusable; see
the authoritative guide's `Context-isolated adjudicator (D14)` section.

## Development Closure Rule

For development-task closure, do not describe certification, final verification,
or status flips as the first completion step. First determine whether the task
must pass the mandatory code-solution review gate resolved by the canonical RRI
band table under `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
and `docs/policies/HITL_AUTONOMY_POLICY.md`, then describe the remaining closure
blocks in order.

## Language

Agent-facing repository instructions must be written in English.

User-facing presentation may be adapted to the user's language, but task metadata, filenames and model identifiers should remain exact.
