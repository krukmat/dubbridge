---
type: Audit
title: "RRI evidence: Element 3 - scripts/antares/* reconciliation (pre-decomposition + subtasks)"
status: approved
task: docs/tasks/handoff-antares-element3-2026-08-05.md
date: 2026-08-05
---

# Element 3 RRI evidence

Task: Element 3 (Phase D) — reconcile `scripts/antares/*`'s invocation model
against `antares tool query --stdin` / `antares tool sweep --stdin`
Mode: pre-execution; no implementation diff
Date: 2026-08-05
Input: `docs/evaluations/antares-phase-b-comparison.md` (Phase B empirical
result — harness cannot consume real Antares wire-format output)

## Pre-decomposition score (undecomposed scope)

Command:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/harness.py \
  --touches scripts/antares/tool_call_parser.py \
  --touches scripts/antares/terminal_state.py \
  --touches scripts/antares/replay_fixtures.py \
  --touches scripts/antares/harness_test.py \
  --touches docs/tasks/handoff-antares-element3-2026-08-05.md \
  --touches docs/plan/antares-local-runtime-adoption.md \
  --cc 9 \
  --D 3 --K 4 --P 4 --T 1 --A 1 --X 3 \
  --penalty arch_decision \
  --platform dubbridge
```

Output:

```text
**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 9 -> score 1 (policy CC table) | High |
| F files | 3 | --touches -> 7 files | High |
| D domain | 3 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 4 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 46
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 58 -> band Complex (56-70) -> Effort L . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Plan first. Human reviews the plan before any implementation.
**Decomposition:** triggered by RRI >= 56 — split before implementing
```

CC 9 is the isolated cyclomatic complexity of the two functions Phase B
proved broken (`dispatch_tool_call` in `harness.py`, `parse_tool_call` in
`tool_call_parser.py`), each measured independently by AST walk — not a
whole-file sum. F=3 (7 touched files), K=4, and the `arch_decision` penalty
are the dominant drivers, consistent with the T3c-1 precedent
(`docs/audit/antares-t3c-1-rri.md`, same code area, RRI 55) — this task
scores higher because it spans more files and carries an explicit
route-decision (subprocess-adoption vs. translation-layer vs. retire) that
T3c-1 did not.

**Decomposition triggered per `docs/policies/RRI_POLICY.md` § Decomposition
triggers: RRI ≥ 56 is an unconditional hard gate.** Split below.

## T2a–T2e disposition (explicit, per handoff acceptance criterion #5)

**Decision: retain, narrowed.** T2a–T2e's code is not wrong or wasted. T2a's
own closure record (`docs/tasks/antares-security-specialist-advisor.md`
§ T2a) already documented, at implementation time (2026-07-29), that "the
translation layer this task's own docstring assigned to T2c does not exist
anywhere in `scripts/antares/`" — T2c was decomposed into T2c-1
(subprocess lifecycle) and T2c-2 (resource budgets), neither of which
implements wire-format translation. Phase B's experiment did not discover a
new defect; it empirically confirmed a gap T2a already flagged as deferred.

T2a–T2e remain valid as the **synthetic-fixture / replay-test path** —
`replay_fixtures.py` and `harness_test.py` already validate only the
internal `{"tool":..., "payload":...}` schema, never live model output. They
are retained unmodified as that path. They are explicitly **not** the live
Antares-invocation path, and were never proven to be one; no task in the T2
chain claimed otherwise once decomposed. This narrowing is a documentation
correction (Subtask C below), not a code change.

## Subtask A — Route decision (docs-only)

Decide, in a plan amendment, whether `scripts/antares/*` adopts a
subprocess-invocation layer over `antares tool query --stdin`/`sweep
--stdin`, or is retired from the live-invocation role entirely in favor of
direct CLI subprocess calls (with T2a–T2e kept only as the test/replay
path per the disposition above). No code changes; decision only, made with
explicit acceptance criteria and Phase B's evidence.

Command:

```bash
python3 scripts/rri.py \
  --touches docs/plan/antares-local-runtime-adoption.md \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --touches docs/tasks/handoff-antares-element3-2026-08-05.md \
  --cc 1 \
  --D 2 --K 2 --P 3 --T 0 --A 1 --X 2 \
  --platform dubbridge
```

Output:

```text
**Final RRI:** 26 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered
```

## Subtask B — Implement the decided route

**PROVISIONAL — not a final resolved-scope score.** Subtask A has not run
yet, so Subtask B's actual diff is not yet knowable. The score below is a
conservative upper-bound placeholder, computed against the larger of the two
possible outcomes of Subtask A (a translation layer added to
`harness.py`/`tool_call_parser.py`), used only to confirm this branch does
not itself force a further decomposition (it doesn't: 48 ≤ 55). It is
**not** "RRI computed against Phase-B-resolved scope" in the sense the
handoff's acceptance criterion #1 means for a task ready to present — that
criterion is satisfied for Subtask A (fully resolved: the decision itself
*is* the scope) but not yet for Subtask B. Subtask B must be **rescored
against its actual resolved diff, as its own gated presentation, after
Subtask A's decision lands** — this artifact does not authorize presenting
or implementing Subtask B on the score below.

Command:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/harness.py \
  --touches scripts/antares/tool_call_parser.py \
  --cc 9 \
  --D 3 --K 3 --P 4 --T 1 --A 0 --X 2 \
  --penalty arch_decision \
  --platform dubbridge
```

Output:

```text
**Final RRI:** 48 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered
```

Depends on Subtask A's decision (must resolve `GO` before this subtask is
scored final and presented).

## Subtask C — T2a–T2e disposition documentation sync

Update `docs/tasks/antares-security-specialist-advisor.md`'s T2a–T2e rows
and any citing plan prose to state the narrowed disposition explicitly
(synthetic-fixture/replay path, not live-invocation path). No production
code changes — `replay_fixtures.py`/`harness_test.py` touched only if their
module docstrings need the same clarification.

Command:

```bash
python3 scripts/rri.py \
  --touches scripts/antares/replay_fixtures.py \
  --touches scripts/antares/harness_test.py \
  --touches docs/tasks/antares-security-specialist-advisor.md \
  --cc 1 \
  --D 1 --K 1 --P 2 --T 1 --A 0 --X 1 \
  --platform dubbridge
```

Output:

```text
**Final RRI:** 18 -> band Low (0-25) -> Effort S . Codex Local Gemma via Ollama . Claude Local Gemma via Ollama . thinking Off
**Gates for this band:** Local delegation: delegate to local Gemma via Ollama; validate and apply only an in-scope diff; review against requirements; verify; report.
**Decomposition:** not triggered
```

Independent of Subtask A/B — can run in parallel or first.

## Subtask A — closure record

- Approved by user 2026-08-05 ("aprobado").
- Task-analysis review: `qwen3.6:27b-q4_K_M` (Ollama, `/tmp/subtask_a_review_result.json`) - PASS (2 MINOR non-blocking findings; addressed inline, no artifact change required).
- Implementation: primary agent (Claude Code), direct authorship — docs-only decision write, no local-agent delegation needed for a plan-amendment task of this size.
- Decision written: `docs/plan/antares-local-runtime-adoption.md` § "Element 3" (decision + justification), § "Decision points" (both rows resolved), dependency graph `DEC` node, Proposed-sequence Phase D row, Approval-boundary note.
- Code-solution review: `qwen3.6:27b-q4_K_M` (Ollama, `/tmp/subtask_a_phase2_result.json`) - PASS, 0 findings.
- `make qa-okf-frontmatter` and `make qa-docs`: both passed post-change.
- Handoff updated: `docs/tasks/handoff-antares-element3-2026-08-05.md` status `ready` → `in_progress`, remaining scope (Subtask B, Subtask C) stated explicitly.
- Scope check: no `scripts/antares/*.py` file touched — confirmed by `git diff --stat` showing only `docs/plan/antares-local-runtime-adoption.md`.
- Result: **Subtask A closed.** Subtask B remains blocked on its own rescore + approval; Subtask C remains blocked on its own approval.

## Subtask C — closure record

- Approved by user 2026-08-05 ("aprobado").
- Task-analysis review: `n/a - docs-only/task-ledger-only exemption` (RRI 18
  Low, docs-only disposition-sync work; exempt per
  `docs/policies/HITL_AUTONOMY_POLICY.md` and
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Band-routed peer review phase-1
  exemptions).
- Implementation: primary agent (Claude Code), direct authorship — per
  `docs/policies/HITL_AUTONOMY_POLICY.md` § "Local delegation (RRI 0-25)":
  "Docs, plans, task ledgers, ADRs, policies, workflow scripts, and other
  structure-heavy or interpretation-heavy work must stay with the primary
  agent even when the RRI is Low." Not delegated to Gemma.
- Change: `docs/tasks/antares-security-specialist-advisor.md` — added a
  "Disposition note (2026-08-05)" subsection under T2e stating the harness
  validates internal-schema composition only, not live Antares wire-format
  compatibility, cross-referencing T2a's existing 2026-08-05 post-hoc
  correction notice and this artifact's own "T2a–T2e disposition" /
  "Subtask A" sections; updated the task-summary table rows for T2a and T2e
  with an inline pointer to the same disposition.
- `replay_fixtures.py` / `harness_test.py` docstrings: inspected, not
  changed — `replay_fixtures.py`'s docstring is already schema-neutral (no
  live-invocation claim); `harness_test.py` has no module docstring to
  correct. Per this Subtask's own scope ("touched only if their module
  docstrings need the same clarification"), no code change was required.
- Code-solution review: `n/a - docs-only/task-ledger-only exemption` (same
  basis as task-analysis review above).
- `make qa-okf-frontmatter` and `make qa-docs`: both passed post-change.
- Scope check: `git diff --stat` confirmed only
  `docs/tasks/antares-security-specialist-advisor.md` changed (34
  insertions, 2 deletions) — no `scripts/antares/*.py` file touched.
- Result: **Subtask C closed.** All three Element 3 subtasks are now
  either closed (A, C) or explicitly blocked pending their own gate
  (B — rescore + approval against the resolved diff).

## Split-target check

Per `docs/policies/RRI_POLICY.md` § Decomposition triggers: "divide until
each subtask scores RRI ≤ 55 with A ∈ {0, 1}."

| Subtask | RRI | Band | A | Status |
|---|---|---|---|---|
| A — Route decision | 26 | Moderate | 1 | Final — resolved scope, ready to present |
| B — Implement route | 48 (provisional upper bound) | Med-high (provisional) | 0 | **Not final** — must be rescored against actual diff after Subtask A resolves, then presented on its own |
| C — T2a–T2e disposition doc sync | 18 | Low | 0 | Final — resolved scope, ready to present |

All three ≤ 55 with A ∈ {0,1} on current evidence, including Subtask B's
conservative upper-bound placeholder — so no branch of this decomposition is
at risk of needing further splitting. Split target satisfied for sizing
purposes. Subtask B's number is not a presentable/approvable score until
recomputed against its resolved post-Subtask-A diff.

## Dependency order

`Subtask A → Subtask B` (B's actual scope depends on A's decision).
`Subtask C` is independent and may run before, after, or in parallel with
A/B.
