---
type: TaskList
title: "Tasks: Codex Session Model Workflow Helper"
plan: docs/plan/codex-session-model-workflow-helper.md
status: done
rri: 18
band: Low
effort: S
---
# Tasks: Codex Session Model Workflow Helper

## Objective

Save the effective-model lookup snippet as a reusable repository workflow helper
and expose it through a stable repo-local command.

## Governing Documents

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/architecture.md`
- `docs/plan/roadmap.md`
- `docs/plan/codex-session-model-workflow-helper.md`

## Slice RRI

**Score: 18 → Low (0–25) → Effort S.**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | small read-only utility | High |
| F files | 1 | script + test + Makefile | High |
| D domain | 1 | local workflow helper only | High |
| T coverage | 1 | focused unit test sufficient | High |
| A ambiguity | 0 | user supplied exact behavior | High |
| K coupling | 1 | Makefile convenience only | High |
| P impact | 1 | operator visibility only | High |
| X context | 1 | isolated workflow utility | High |

Task-analysis review: n/a (Low-band direct workflow utility; no separate advisory artifact recorded)

## T1 — Add reusable model-inspection helper

- **Status:** [x] Done
- **Effort:** S
- **RRI:** 18 → Low
- **Depends on:** none

### Happy paths considered

- HP-1: newest rollout file contains a `turn_context` and the helper prints the
  session path, effective model, and reasoning effort.
- HP-2: the repo user can run the helper through a stable `Makefile` target
  without reconstructing the inline Python snippet.

### Edge cases considered

- EC-1: no rollout files exist and the helper exits fail-closed with the same
  explicit message used by the ad hoc snippet.
- EC-2: rollout files exist but contain no parseable `turn_context`, and the
  helper prints `No se encontró turn_context.` instead of crashing.

### Inputs

- User-provided inline Python snippet from chat.
- `Makefile`
- `scripts/`

### Outputs

- `scripts/show-codex-session-model.py`
- `scripts/show_codex_session_model_test.py`
- `Makefile` target for invoking the helper

### Acceptance Criteria

1. The script finds the newest `rollout-*.jsonl` file under the Codex sessions directory.
2. The script prints the session path plus the effective model and reasoning effort when `turn_context` exists.
3. The script preserves the explicit no-sessions and no-`turn_context` behaviors from the supplied snippet.
4. A repo-local command invokes the helper without retyping the Python heredoc.
5. Focused unit tests cover HP-1 and EC-1 at minimum.

### Agent handoff prompt

Add a versioned workflow helper that encapsulates the effective Codex model
lookup snippet, wire it into `Makefile`, and add focused tests so the behavior
stays stable.

### Completion evidence

- Added `scripts/show-codex-session-model.py` as a read-only workflow helper for
  Codex rollout inspection.
- Added `make show-codex-session-model` in `Makefile`.
- Added `scripts/show_codex_session_model_test.py` with focused unit coverage for
  the success path and fail-closed edge cases.

Code-solution review: n/a (direct Low-band workflow utility; no separate review artifact recorded)

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | newest rollout with `turn_context` prints session path, model, and reasoning | `scripts/show_codex_session_model_test.py::ShowCodexSessionModelTest::test_hp1_reads_newest_rollout_and_extracts_last_turn_context` | passed |
| HP-2 | Happy path | repo-local command runs the helper without inline heredoc reconstruction | `scripts/show-codex-session-model.py::main` exercised by `make show-codex-session-model` | passed |
| EC-1 | Edge case | no rollout files exits fail-closed with the explicit no-session message | `scripts/show_codex_session_model_test.py::ShowCodexSessionModelTest::test_ec1_fails_closed_when_no_rollouts_exist` | passed |
| EC-2 | Edge case | rollout without parseable `turn_context` prints the explicit fallback instead of crashing | `scripts/show_codex_session_model_test.py::ShowCodexSessionModelTest::test_ec2_reports_missing_turn_context_without_crashing` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-19`
- Statement: I verified the workflow helper preserves the requested lookup behavior, exposes a stable repo-local command, and covers the approved happy-path and fail-closed edge cases with executable evidence.
- Commands run: `python3 scripts/show_codex_session_model_test.py`; `make show-codex-session-model`
