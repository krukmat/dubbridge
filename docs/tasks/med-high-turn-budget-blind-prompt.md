---
type: TaskList
title: "Tasks: local-agent system prompt is blind to cwd and its own turn budget"
description: "Add the missing cwd/relative-path rule and interpolated per-band turn budget to TOOL_CALLING_SYSTEM_PROMPT so the local implementer stops burning its session on hallucinated-path reconnaissance."
plan: docs/plan/med-high-turn-budget-blind-prompt.md
status: implemented
slice: med-high-turn-budget-blind-prompt
---

# Tasks: Local-Agent System Prompt Is Blind to `cwd` and Its Own Turn Budget

> **Plan:** `docs/plan/med-high-turn-budget-blind-prompt.md`

## Status Legend
- [ ] Not started · [x] Done · [~] In progress · [!] Blocked

Build order: **TB1** (single task; no decomposition required at RRI 33).

---

## Task TB1 — Interpolate turn budget and state the cwd rule in the system prompt

- **Status:** [x] Done
- **Type:** development
- **Effort:** M (RRI band-derived, Moderate)
- **Depends on:** —

### Objective

Add two missing facts to `TOOL_CALLING_SYSTEM_PROMPT`
(`scripts/local-agent/run_local_task.py:111`): the session's exact turn
budget (currently computed but never shown to the model) and an explicit
rule that all tool paths — `read_file`, `write_file`, `apply_patch`, and
`run_command`'s `argv` — resolve relative to the worktree root the model is
already operating in, never an absolute host path.

### Happy paths considered

- **HP-1:** a Moderate-band session (`max_total_turns=30`) receives a system
  prompt whose rendered text contains the literal substring `30 turns` (or
  equivalent exact phrasing agreed at implementation) — `run_loop`'s
  `messages[0]["content"]` is asserted directly in a unit test, not just
  `TOOL_CALLING_SYSTEM_PROMPT` in isolation, since the interpolation happens
  at the call site (plan Design decision 2).
- **HP-2:** a Med-high-band session (`max_total_turns=8`) receives a system
  prompt containing `8 turns` — same assertion style as HP-1, proving the
  interpolation is actually band-aware and not a hardcoded number.
- **HP-3:** the rendered prompt contains an explicit cwd/relative-path
  statement that names the worktree as the resolution root and gives a
  concrete negative example of what not to do (an absolute path prefix such
  as `/home/` or `/Users/`) — verified by asserting the exact added sentence
  is present in the rendered `messages[0]["content"]`.

### Edge cases considered

- **EC-1:** a task card with no `band`/`rri` set (falls through
  `_is_med_high`'s `False` branch, `resolve_effective_limits` returns the
  Moderate default) still renders `30 turns`, not a missing/`None`
  interpolation — proving the fallback path is covered, not just the two
  named bands.
- **EC-2:** the existing tool-call JSON contract (the
  `{"tool_calls": [...]}` shape, the five tool names, the "respond with ONLY
  that JSON object" instruction) is byte-identical apart from the two added
  facts — a full-prompt diff against the pre-change constant shows only
  additive text, no reordering or rewording of the existing tool
  descriptions, so this task does not risk regressing the already-tuned
  tool-calling reliability (the plan's own docstring at `:151-161` notes
  small/medium local models are sensitive to prompt wording for this exact
  contract).
- **EC-3:** a session whose `card.spec` itself happens to contain the
  substring `turns` (coincidental collision) does not break the
  interpolation or the test's assertion — the test asserts on
  `messages[0]["content"]` before `card.spec` is appended, or asserts
  presence via a substring check scoped to the system-prompt portion only,
  not a brittle whole-message equality that a real task spec could
  invalidate.

### Acceptance criteria

- `TOOL_CALLING_SYSTEM_PROMPT` gains a `{max_total_turns}` placeholder (or
  equivalent) consumed via `.format()`/an f-string at the `messages = [...]`
  construction inside `run_loop` (`:468-473`), using
  `limits.max_total_turns` from the `resolve_effective_limits(card)` call
  already at `:460` — no new parameter threaded into `run_loop`'s own
  signature, `build_live_chat_fn`, or any other function.
- The added cwd rule explicitly states: all `read_file`/`write_file`/
  `apply_patch` paths and `run_command`'s `argv` are relative to the
  worktree root the model is already in; never emit an absolute host path.
- No other sentence in `TOOL_CALLING_SYSTEM_PROMPT` is reworded, reordered,
  or removed (EC-2).
- `MAX_TOTAL_TURNS`, `MED_HIGH_MAX_TOTAL_TURNS`,
  `resolve_effective_limits`, `EffectiveLimits`, and `_is_med_high` are
  unchanged (plan Objective boundary).

### Evidence to emit

- A rendered-prompt diff (pre-change constant vs. post-change
  `messages[0]["content"]` for one Moderate and one Med-high card) showing
  only the two additive facts.
- Unit test output for HP-1/HP-2/HP-3/EC-1/EC-2/EC-3.

### Status artifacts affected

- `docs/tasks/antares-security-specialist-advisor.md` § T2a — add a forward
  reference noting the blind-prompt defect T2a hit is now closed by this
  task (informational only; does not reopen T2a, whose escalation-to-cloud
  outcome under ADR-038 was itself correct given the defect existed at the
  time).

### Files affected

- `scripts/local-agent/run_local_task.py`
- `scripts/local-agent/run_local_task_test.py`

---

### RRI

```
python3 scripts/rri.py \
  --touches scripts/local-agent/run_local_task.py \
  --touches scripts/local-agent/run_local_task_test.py \
  --touches docs/plan/med-high-turn-budget-blind-prompt.md \
  --touches docs/tasks/med-high-turn-budget-blind-prompt.md \
  --cc 3 --D 4 --K 3 --P 2 --T 1 --A 0 --X 2
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 3 (one `.format()` call site, no new branches) -> score 0 | High |
| F files | 2 | 4 touched paths | High |
| D domain | 4 | anchor rubric `RRI_POLICY.md:89` — "agent orchestration" scores 4; this is the shared prompt driving the ADR-038/local-first orchestration loop, not ordinary application code | High |
| T coverage | 1 | agent-supplied — `run_local_task_test.py` already has extensive existing coverage of `run_loop`'s message construction to extend, but no test yet asserts prompt *content* specifically | High |
| A ambiguity | 0 | agent-supplied — exact placeholder text and cwd wording are the only open choices, both bounded by the acceptance criteria and negative example from live evidence (T2a) | High |
| K coupling | 3 | agent-supplied — same rationale as the `med-high-escalation-bundle-crash` precedent: integrates into the already-CI-facing local/Med-high pipeline (`run_med_high_task.py` invokes `run_local_task.py` as a subprocess; a prompt regression here silently degrades every future local session in both bands) | High |
| P impact | 2 | agent-supplied — changes local-implementer behavior only; no public API, auth, schema, or persisted user data | High |
| X context | 2 | agent-supplied — must hold `run_loop`, `resolve_effective_limits`, `TOOL_CALLING_SYSTEM_PROMPT`, and the two live-incident evidence sources in mind at once | High |

**Base value:** 100 x (weighted / 5) = 33
**Penalties applied:** none
**Final RRI:** 33 -> band **Moderate (26-40)** -> Effort M
**Gates for this band:** Plan + explicit acceptance criteria required before
approval (satisfied above); 2 Reflection passes; band-routed peer review
(phases 1 and 2, `qwen3.6:27b-q4_K_M` -> Gemma -> D14).
**Decomposition:** not triggered.
**Implementation route:** primary agent (Claude Code) implements directly,
not via `scripts/local-agent/run_local_task.py` — see plan Design decision 5
(self-repair circularity; precedent in `med-high-escalation-bundle-crash.md`).

---

## Closure record

### Implementation route taken

Primary agent (Claude Code) implemented directly, per plan Design decision 5
(self-repair circularity) — not via `scripts/local-agent/run_local_task.py`.

Actual mechanism: swapped a `.format()`-based interpolation attempt for
`.replace("{MAX_TOTAL_TURNS}", str(max_total_turns))` after discovering
`.format()` collides with the pre-existing literal JSON braces already in
`TOOL_CALLING_SYSTEM_PROMPT` (`{"tool_calls": [...]}`, `{"path": ...}`,
etc.) — using `.format()` as originally planned would have required escaping
every one of those braces, risking exactly the EC-2 regression the plan
warns against. `.replace()` needs no escaping and cannot collide with the
JSON examples.

### Peer Reviewer evidence — Phase 1 (task-analysis)

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: manual `curl` to `http://127.0.0.1:11434/api/chat`, `think: false`
- Artifact: `.agent/peer-task-review-TB1-phase1.json`
- Verdict: `PASS`
- Findings: 3 informational (defect source-verifiable, HP/EC cases concrete/testable, scope boundary coherent)
- Gemma fallback: not triggered — reason: first attempt returned `done_reason: "stop"`
- D14 fallback: not triggered — reason: n/a
- disposition_divergence: none

### Peer Reviewer evidence — Phase 2 (code-solution)

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: manual `curl` to `http://127.0.0.1:11434/api/chat`, `think: false`, diff + acceptance criteria embedded
- Artifact: `.agent/peer-task-review-TB1-phase2.json`
- Verdict: `PASS`
- Findings: none
- Gemma fallback: not triggered — reason: first attempt returned `done_reason: "stop"`
- D14 fallback: not triggered — reason: n/a
- disposition_divergence: none
- Primary-agent disposition: accepted (no findings to disposition)

```
Task-analysis review: qwen3.6:27b-q4_K_M .agent/peer-task-review-TB1-phase1.json - PASS
Code-solution review: qwen3.6:27b-q4_K_M .agent/peer-task-review-TB1-phase2.json - PASS
```

### Reflection log

Required passes: 2 (`33` → `Moderate`)

#### Pass 1

- **Draft verdict:** initial `.format()`-based interpolation broke on the
  prompt's pre-existing literal JSON example braces.
- **Critique findings:**
  - `.format()` is incompatible with the existing prompt content without
    escaping dozens of braces — high risk of accidentally rewriting the
    existing JSON contract text (would violate EC-2).
  - No automated check yet confirmed the interpolation actually worked for
    both bands before touching the test suite.
- **Revisions applied:**
  - Switched the placeholder to `{MAX_TOTAL_TURNS}` consumed via
    `.replace()` instead of `.format()`.
  - Verified with a standalone AST-based extractor that both bands (8 and
    30) interpolate correctly with no residual placeholder.

#### Pass 2

- **Draft verdict:** after the `.replace()` fix, wrote 6 new tests
  (`SystemPromptTurnBudgetInterpolation`) plus manual AST checks, but had
  not yet run the full existing suite to confirm no regression against
  tests asserting directly on `TOOL_CALLING_SYSTEM_PROMPT` substrings.
- **Critique findings:**
  - Real risk: if the added text had interfered with already-asserted
    substrings (`"disposable"`, `"no fixed command allowlist"`, etc.), those
    tests would fail silently unless the full suite ran.
  - EC-3 (spec collision with the word "turns") needed an explicit test
    rather than relying on accidental non-collision.
- **Revisions applied:**
  - Ran the full suite (`pytest run_local_task_test.py`): 82/82 passed,
    including preexisting contract tests and the 6 new ones.
  - No further changes needed — the Pass 1 fix already guaranteed isolation
    from the preexisting text.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Moderate band renders `30 turns` | `scripts/local-agent/run_local_task_test.py::SystemPromptTurnBudgetInterpolation::test_hp1_moderate_band_prompt_states_thirty_turns` | passed |
| HP-2 | Happy path | Med-high band renders `8 turns`, not `30 turns` | `scripts/local-agent/run_local_task_test.py::SystemPromptTurnBudgetInterpolation::test_hp2_med_high_band_prompt_states_eight_turns` | passed |
| HP-3 | Happy path | cwd rule present with negative example (`/home/`, `/Users/`) | `scripts/local-agent/run_local_task_test.py::SystemPromptTurnBudgetInterpolation::test_hp3_prompt_states_cwd_rule_with_negative_example` | passed |
| EC-1 | Edge case | card without band/rri falls back to `30 turns`, no residual placeholder | `scripts/local-agent/run_local_task_test.py::SystemPromptTurnBudgetInterpolation::test_ec1_card_without_band_or_rri_falls_back_to_thirty_turns` | passed |
| EC-2 | Edge case | existing tool-contract text unchanged | `scripts/local-agent/run_local_task_test.py::SystemPromptTurnBudgetInterpolation::test_ec2_existing_tool_contract_text_is_unchanged` (plus preexisting `SystemPromptIncludesToolContract`, `SystemPromptCopyTest`) | passed |
| EC-3 | Edge case | `card.spec` containing the word "turns" does not break render or assertions | `scripts/local-agent/run_local_task_test.py::SystemPromptTurnBudgetInterpolation::test_ec3_task_spec_containing_the_word_turns_does_not_break_render` | passed |

Full suite: `cd scripts/local-agent && python3 -m pytest run_local_task_test.py -q` → 82 passed.

### Owner final verification

- Owner: `Matias Kruk`
- Date: `2026-08-07`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior, and that both band-routed peer review phases passed without findings.
- Commands run: `cd scripts/local-agent && python3 -m pytest run_local_task_test.py -q`

## Agent handoff prompt (for delegation)

```
Task: TB1 — docs/tasks/med-high-turn-budget-blind-prompt.md
Plan: docs/plan/med-high-turn-budget-blind-prompt.md

File + line range: scripts/local-agent/run_local_task.py:111-149
(TOOL_CALLING_SYSTEM_PROMPT), :468-473 (messages construction in run_loop).

Acceptance criteria: see "Acceptance criteria" above (bullets only).

Stop condition: after unit tests for HP-1/HP-2/HP-3/EC-1/EC-2/EC-3 pass,
stop and report. Do not touch MAX_TOTAL_TURNS, MED_HIGH_MAX_TOTAL_TURNS,
resolve_effective_limits, EffectiveLimits, _is_med_high, build_live_chat_fn,
or run_med_high_task.py.
```
