---
type: TaskList
title: "Tasks: review-gate diff path scoping"
description: "Scope qa-gemma-review, qa-peer-workflow-review, and qa-review-budget to an explicit path list so an unrelated task's uncommitted diff cannot contaminate a review packet."
status: draft
slice: review-gate-diff-path-scoping
---

# Tasks: Review-Gate Diff Path Scoping

## Context

During `T2a` (Antares run-analysis fix) closure, `make qa-gemma-review` built
its packet from `git diff HEAD` against a working tree that also held
unrelated uncommitted changes from another in-flight task
(`agent-session-preflight-gate`). Gemma returned 3 findings; all 3 pointed at
`scripts/agent-preflight.py` / `scripts/agent_preflight_test.py` (the other
task's files), none at `scripts/local-architect/run_analysis.py` (the
reviewed change). The `PASS` receipt
(`docs/audit/gemma-evidence/antares-t2a-run-analysis-fix.json`) was correct
only because the `path` field on each finding was manually cross-checked
against the actual diff — nothing in the gate itself would have caught a
false `PASS` if the two tasks' files had overlapped, or would have caught a
false negative if the reviewed task's own files had been the ones silently
un-reviewed.

Root cause, confirmed by reading the gates directly rather than assumed:
`qa-gemma-review` (`Makefile:97-122`) and `qa-peer-workflow-review`
(`Makefile:167-188`) both build their diff with `git diff $(BASE)` /
`git diff "$(BASE)"` — no pathspec — so the packet is always "everything
uncommitted in the tree," never "this task's own changes." This is what
unlocks the fix: a shared, opt-in path-scoping variable, reusing a
mechanism the repo already has and already tests, applied uniformly to
every gate that has the same exposure.

This does not replace commit/stash discipline before switching tasks (the
actual root cause of leaving two tasks' diffs uncommitted at once) — it is
the tooling-side defense-in-depth layer, scoped to what tooling can actually
fix.

## Governing documents

- `Makefile` (`qa-gemma-review`, `qa-peer-workflow-review`, `qa-review-budget` targets)
- `scripts/check-review-budget.py` (already implements `--files`, tested in `scripts/check_review_budget_test.py`)
- `scripts/check-maintainability.py` (`changed_files`, `added_lines_for` — the pathspec pattern being reused)
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Reviewability budget gate`

## RRI

Computed via `scripts/rri.py --touches Makefile --cc 1 --D 1 --K 2 --P 2 --T 2 --A 0 --X 1`:

```
Final RRI: 19 -> band Low (0-25) -> Effort S
Advisory: Makefile — no anchor-rubric match; agent judgment governs D/P/K
```

**Task type:** `config` — Makefile wiring that reuses an existing, already-tested
flag (`check-review-budget.py --files`) and native `git` pathspec filtering; no
new logic, no new abstraction. Per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, config-only tasks are exempt from the
HP-#/EC-# behavioral-example requirement, phase-1 task-analysis review, the
Reflection cycle, and the mandatory Step-1 code-solution review — the exemption
applies to the *nature* of the change (mechanical wiring, not domain logic),
independent of which subsystem it touches.

**Execution surface:** RRI 0-25 defaults to direct primary-agent execution.
Local Gemma delegation is not used: per
`docs/policies/HITL_AUTONOMY_POLICY.md § Local delegation (RRI 0-25)`, "docs,
plans, task ledgers, ADRs, policies, **workflow scripts**, and other
structure-heavy or interpretation-heavy work must stay with the primary agent
even when the RRI is Low." The `Makefile` is exactly this class of file.

## Dependencies

None. This is a single, self-contained task:

- **Upstream:** no other open task blocks it. It is unrelated to the in-flight
  `agent-session-preflight-gate` or `antares-security-specialist-advisor` work
  currently uncommitted in the same tree (that coexistence is what exposed the
  bug being fixed here, not a dependency on it).
- **Downstream:** no task currently depends on this landing first.
- **Internal ordering:** the edits below are not separable sub-tasks — they
  implement one design decision (`REVIEW_PATHS` as the single shared scoping
  variable) and must land together, since a partial application (e.g. only the
  Gemma target updated) would leave the identical exposure open in the other
  gates and defeat the point of using one shared name.

## T1 — Add `REVIEW_PATHS` scoping to the three diff-consuming review gates

- **Status:** [x] Done
- **Type:** config

### Goal

Add one opt-in Makefile variable, `REVIEW_PATHS` (default empty — no behavior
change), and apply it identically everywhere a review gate currently builds an
unscoped `git diff` against the whole working tree.

### Edits (ordered; land together)

1. **`Makefile`**, near the existing `PEER_REVIEW_BASE` / `GEMMA_REVIEW_BASE`
   declarations (~line 9-12): add
   ```makefile
   REVIEW_PATHS ?=
   ```
2. **`qa-gemma-review`** (`Makefile:101-102`), the pre-check that decides
   whether to skip the review for lack of code changes — append the pathspec so
   the pre-check agrees with the packet built two steps later:
   ```makefile
   code_changes=$$(git diff --name-only $(GEMMA_REVIEW_BASE) -- $(REVIEW_PATHS) 2>/dev/null \
       | grep -vE '^(docs/|[^/]+\.md$$)' || true); \
   ```
3. **`qa-gemma-review`** (`Makefile:107`), the actual packet-building diff:
   ```makefile
   git diff $(GEMMA_REVIEW_BASE) -- $(REVIEW_PATHS); \
   ```
4. **`qa-peer-workflow-review`** (`Makefile:176`):
   ```makefile
   git diff "$(PEER_REVIEW_BASE)" -- $(REVIEW_PATHS) | python3 scripts/peer-workflow-review.py "$$@" --content - || review_status=$$?; \
   ```
5. **`qa-review-budget`** (`Makefile:56-57`) — **not** a bare interpolation.
   `check-review-budget.py --files` uses `nargs="*"`; passing the flag with a
   value that expands to nothing yields `args.files == []`, which is `is not
   None` and therefore short-circuits the script's own `changed_files(base)`
   fallback (`check-review-budget.py:182-184`), silently disabling the budget
   gate (always 0 counted lines) whenever `REVIEW_PATHS` is unset. `git diff --
   $(EMPTY)` and `argparse --files $(EMPTY)` do not have the same empty
   semantics — the flag must be omitted entirely when unset:
   ```makefile
   qa-review-budget:
   	python3 scripts/check-review-budget.py $(if $(REVIEW_PATHS),--files $(REVIEW_PATHS))
   ```
6. **`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Reviewability budget gate`**
   (~line 753): add one line documenting `REVIEW_PATHS` as the shared
   opt-in scoping variable across `qa-gemma-review`, `qa-peer-workflow-review`,
   and `qa-review-budget`, alongside the existing `DUBBRIDGE_REVIEW_*` env vars
   already documented there.

### Acceptance criteria

- `REVIEW_PATHS` unset: all three targets behave exactly as today (whole-tree
  diff) — regression check, not a new behavior.
- `REVIEW_PATHS=Makefile` with unrelated dirty files elsewhere in the tree:
  - `qa-gemma-review`'s packet contains only `Makefile` hunks.
  - `qa-peer-workflow-review`'s `--content -` input contains only `Makefile` hunks.
  - `qa-review-budget` counts only `Makefile`'s added/changed lines (verify via
    `check-review-budget.py`'s existing `--files` path, not new code).
- `qa-review-budget` gate remains active (non-trivially enforced, not
  short-circuited to 0) when `REVIEW_PATHS` is unset — regression check for the
  `nargs="*"` pitfall in edit 5.

### Verification

Run the modified targets against a working tree that deliberately holds two
unrelated diffs (reproduce the T2a conditions), not a clean tree — a clean-tree
run cannot exercise the bug this task fixes. Given the tree is *currently*
dirty with unrelated in-flight work
(`agent-session-preflight-gate`, `antares-security-specialist-advisor`,
`plan-risk-analysis-standard`), stage/verify with `REVIEW_PATHS=Makefile`
explicitly rather than stashing that work aside, so the verification run
itself exercises the exact contamination scenario instead of avoiding it:

```bash
REVIEW_PATHS=Makefile GEMMA_REVIEW_TASK_ID= make qa-gemma-review   # dry, no task-id receipt
REVIEW_PATHS=Makefile make qa-review-budget
```

### Evidence to emit

None beyond the verification command output above — this task does not itself
produce review/benchmark evidence; it changes tooling that scopes *future*
evidence collection.

### Status artifacts affected

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Reviewability budget gate` (edit 6).
- No ADR, roadmap, or other task ledger references this Makefile behavior by
  name today, so no further propagation is required.

### Out of scope

- Commit/stash discipline before switching tasks (the process-level root
  cause) — not a tooling change, tracked as a practice, not code.
- `qa-maintainability` / `check-maintainability.py`'s own `discover_base`
  (`origin/main`-preferring) base-resolution logic — unaffected; this task only
  adds path scoping, it does not change base-commit resolution for any gate.
- `scripts/local-agent/scope_check.py` — a different mechanism (post-hoc
  boundary-violation detection for autonomous local-agent sessions, including
  untracked/ignored files), not reused here because it solves a different
  problem than pre-hoc packet scoping.

### Closure record

All 6 edits landed together as specified. Verified against the actual dirty
working tree (not a synthetic reproduction) — at verification time the tree
held this task's own `Makefile`/`AGENT_WORKFLOW_GUIDE.md` changes alongside
unrelated uncommitted work from `agent-session-preflight-gate` and
`antares-security-specialist-advisor`, i.e. the exact contamination
conditions this task fixes.

**Verification commands and results:**

```
$ git diff --name-only HEAD          # unscoped: 10 files across 3 unrelated tasks
$ git diff --name-only HEAD -- Makefile   # scoped: Makefile only
```

```
$ make -n REVIEW_PATHS=Makefile qa-gemma-review | grep "git diff"
code_changes=$(git diff --name-only HEAD -- Makefile ...)
git diff HEAD -- Makefile
$ make -n REVIEW_PATHS=Makefile qa-peer-workflow-review | grep "git diff"
git diff "HEAD" -- Makefile | python3 scripts/peer-workflow-review.py ...
```

```
$ REVIEW_PATHS=Makefile make qa-review-budget
python3 scripts/check-review-budget.py --files Makefile
Reviewability budget gate passed (0/6283 reviewable diff lines).
$ make qa-review-budget            # REVIEW_PATHS unset — regression check
python3 scripts/check-review-budget.py
Reviewability budget gate passed (0/6283 reviewable diff lines).
```

The `0/6283` count in both runs reflects a separate, pre-existing narrowness
in `check-maintainability.classify_path` (reused by `check-review-budget.py`):
it only classifies `apps|crates/*.rs` and `mobile/(src|__tests__)/*.ts(x)` as
"reviewable code," so neither `Makefile` nor `scripts/*.py` count toward the
budget regardless of scoping. This is unrelated to the fix in this task (which
only threads `REVIEW_PATHS` into the existing, already-tested `--files`
parameter) and is out of scope here — noted for a future task if the budget
gate's code-path classifier should be widened. What was verified and confirmed
correct: (a) `REVIEW_PATHS` unset reproduces today's exact command (regression
guard for the `nargs="*"` pitfall called out in edit 5), and (b) `REVIEW_PATHS`
set threads through to `--files` / the `git diff -- <paths>` pathspec on all
three targets, confirmed via `make -n` dry-run recipe expansion plus a direct
`git diff --name-only` scoping check on the real, currently-contaminated tree.

**Review evidence:** `REVIEW-OVERRIDE: not-applicable` — config-only task,
exempt from Step 1 per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Development
task closure checklist`. Ledger row:
`docs/audit/gemma-review-overrides.md` (`review-gate-diff-path-scoping`, 2026-07-30).

**Reflection log / unit coverage certification / owner final verification:**
not applicable — these closure blocks apply to development tasks; this task is
declared `config` type and exempt from the development-task closure checklist
per the same section cited above.

## Agent workflow

| Phase | Participant | Gate / output |
|---|---|---|
| Analysis | Primary agent | Root cause confirmed by reading `Makefile`, `check-review-budget.py`, `check-maintainability.py` directly (see Context) |
| Phase-1 review | n/a | Exempt — config-only task |
| Human approval | User | RRI Low does not mandate the approval card; presented anyway because the change touches shared CI/QA tooling |
| Implementation | Primary agent | Direct edit — Gemma excluded (workflow-script exclusion, see RRI section) |
| Phase-2 review | n/a | Exempt — config-only task |
| Closure | Primary agent | Verification commands above; mark `[x] Done` in this file |

**Approval checkpoint:** RRI is Low (19); the workflow does not require the
formal HITL gate. This task is still presented for confirmation because it
modifies shared CI/QA-gate Makefile targets. Execution has not started.
