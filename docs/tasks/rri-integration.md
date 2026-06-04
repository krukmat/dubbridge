# Tasks: RRI Integration — Required Reasoning Index adoption

**Plan:** `docs/plan/rri-integration.md`
**Scope:** Phase 0 (documentation + policy only; no code, no CI changes)

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 → T1 → T2
```

---

## T0 — Create plan + task ledger

- **Status:** [x] Done — 2026-06-04
- **Effort:** S
- **Complexity:** Low
- **RRI:** ~5 (Low) — base: F1(0.12)+X2(0.06) scaled → ~5; penalties: 0
- **Thinking:** Off
- **Model:** Claude Haiku 4.5 / Codex economy
- **Depends on:** — (first task)
- **Objective:** Create `docs/plan/rri-integration.md` and this ledger as the
  crash-safe progress record and todo-list for the integration.
- **Inputs:** approved plan (conversation), style of existing plan/task files.
- **Outputs:**
  - `docs/plan/rri-integration.md`
  - `docs/tasks/rri-integration.md` (this file)
- **Acceptance criteria:**
  - Both files exist and reflect T0–T2 with dependencies, effort, RRI, and
    acceptance criteria.
  - Plan records all closed decisions (scope, A2, no ADR, AGENTS.md/CLAUDE.md
    untouched, CI untouched).
  - `make qa-docs` passes.
- **Completion record (2026-06-04):**
  - Created `docs/plan/rri-integration.md`: objective, affected files, closed
    decisions table, dependency map, design decisions.
  - Created `docs/tasks/rri-integration.md` (this file): ledger T0–T2.
  - Verification: `make qa-docs` — passed.

---

## T1 — Create `docs/policies/RRI_POLICY.md`

- **Status:** [x] Done — 2026-06-04
- **Effort:** M
- **Complexity:** Low (synthesis-heavy — use Balanced model for quality)
- **RRI:** ~8 (Low) — base: D2(0.15)+A1(0.12)+X3(0.06) scaled → ~8; penalties: 0
- **Thinking:** Off
- **Model:** Claude Sonnet 4.6 / Codex balanced
- **Depends on:** T0
- **Objective:** Create the canonical source for the RRI formula, per-variable
  scoring rubric, repo-specific anchor rubric (by crate/path, anchored to ADRs),
  penalty table, bands with autonomy gates, model-tier mapping, and decomposition
  triggers.
- **Inputs:**
  - `docs/policies/HITL_AUTONOMY_POLICY.md` (structural pattern)
  - RRI formula, rubric, and band analysis from the planning conversation
  - `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (tier names to reuse:
    Economy/Balanced/Premium)
- **Outputs:**
  - `docs/policies/RRI_POLICY.md` (new file)
- **Acceptance criteria:**
  - File exists at `docs/policies/RRI_POLICY.md`.
  - Weights sum verified to 1.00 in the document.
  - Anchor rubric covers all current DubBridge crate paths and maps each to D/P/K
    floors using concrete ADR citations (ADR-008, ADR-023, ADR-024, ADR-026).
  - Bands 0–25 / 26–40 / 41–55 / 56–70 / 71–85 / 86–100 / >100 each have a
    label, autonomy gate, and model tier.
  - Decomposition triggers are listed (RRI>70, F≥4∧K≥3, C≥4∧D≥3, +8 active,
    T≥4∧P≥4).
  - Reporting format (variable table with score / evidence / confidence) is
    specified.
  - No contradiction with `AGENT_WORKFLOW_GUIDE.md`.
  - `make qa-docs` passes.
- **Completion record (2026-06-04):**
  - Created `docs/policies/RRI_POLICY.md`: formula (weights verified = 1.00),
    per-variable scoring bands, DubBridge anchor rubric (8 path/crate rows with
    ADR citations), penalty table (7 penalties), bands table (7 bands with gates
    + tiers), decomposition triggers, reporting format.
  - Marked T1 `[x] Done` in ledger.
  - Verification: `make qa-docs` — passed.

---

## T2 — Amend `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` to adopt RRI

- **Status:** [x] Done — 2026-06-04
- **Effort:** M
- **Complexity:** Low-Medium
- **RRI:** ~32 (Moderate) — base: K2(0.12)+P4(0.10)+X4(0.06) scaled → ~20;
  +12 (policy/process decision required) → **32**
- **Thinking:** Off (escalate to On if cross-doc propagation becomes entangled)
- **Model:** Claude Sonnet 4.6 (escalate to Opus 4.8 if needed) / Codex balanced
- **Depends on:** T1
- **Objective:** Add an RRI adoption section to `AGENT_WORKFLOW_GUIDE.md` that:
  (a) declares RRI as the canonical complexity-and-risk scoring method,
  (b) maps cyclomatic complexity `C` to the RRI variable,
  (c) drives the model-tier recommendation from the RRI band,
  (d) delegates the detailed procedure to `docs/policies/RRI_POLICY.md`,
  (e) includes a brief adoption note (replaces the single-axis table, no ADR).
  The precedence rule ("this guide is the highest authority") is preserved and
  unchanged.
- **Inputs:**
  - `docs/policies/RRI_POLICY.md` (must exist — T1 output)
  - Current "Model and thinking-mode selection" section in the guide (lines ~145–256)
- **Outputs:**
  - Amended `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- **Acceptance criteria:**
  - New RRI section added after or within "Model and thinking-mode selection".
  - Section references `docs/policies/RRI_POLICY.md` for the full procedure.
  - The old single-axis CC→tier table is subsumed (not deleted — noted as subsumed
    by RRI's `C` variable) so the guide remains readable without jumping to the
    policy file.
  - Adoption note states: RRI supersedes single-axis scoring; no ADR required;
    `AGENTS.md`/`CLAUDE.md` untouched (guide overrides both on this topic).
  - Precedence declaration ("overrides … without exception") is intact and
    unmodified.
  - `make qa-docs` passes.
  - Gate (Moderate band): cross-doc consistency verified — guide references
    RRI_POLICY.md correctly; no new contradiction introduced in AGENTS.md,
    CLAUDE.md, or any doc that cites the guide's model-selection section.
- **Completion record (2026-06-04):**
  - Added subsection "RRI — canonical scoring method (adopted 2026-06-04)"
    between the intro paragraph and Step 1: declares adoption, references
    `RRI_POLICY.md`, explains CC→C mapping and tier→RRI-band mapping.
  - Annotated Step 1 CC table: added `RRI C variable score` column + subsumed note.
  - Annotated Step 2 tier mapping table: expanded with `RRI band` column + subsumed note.
  - Updated Step 3 presentation block: added `RRI` row + reference to
    `RRI_POLICY.md` reporting format.
  - Updated `## Related` section: added `RRI_POLICY.md` entry.
  - Precedence declaration ("without exception") verified intact (line 7).
  - Cross-doc check: `AGENTS.md`/`CLAUDE.md` untouched; no contradictions.
  - Verification: `make qa-docs` — passed.

---

## Agent handoff prompt (delegation-ready)

> Implement Phase-0 RRI integration tasks **T1 → T2** from
> `docs/tasks/rri-integration.md` in the `dubbridge` repo.
> T0 is complete (this ledger and `docs/plan/rri-integration.md` exist).
> Governing docs: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (highest authority),
> `docs/policies/HITL_AUTONOMY_POLICY.md` (structural pattern for T1).
> Scope locked: no changes to `AGENTS.md`, `CLAUDE.md`, CI, hooks, or scripts.
> For T1: create `docs/policies/RRI_POLICY.md`; weights must sum to 1.00;
> anchor rubric must cite ADR-008/023/024/026 by path/crate.
> For T2: add the RRI adoption section to the workflow guide; preserve the
> precedence declaration; delegate procedure to RRI_POLICY.md; run `make qa-docs`.
> Present each task for explicit approval before editing.
> Mark progress in this ledger after each task. Stop after T2.
