---
type: TaskList
title: "Tasks: Agent Session Preflight Gate"
plan: docs/plan/agent-session-preflight-gate.md
status: closed
---
# Tasks: Agent Session Preflight Gate

## Objective

Implement a small startup preflight and write-time gate so fresh Codex and
Claude Code sessions load the DubBridge workflow contract before file edits.

## Governing Documents

- `docs/plan/agent-session-preflight-gate.md`
- `README_AGENT_ORDER.md`
- `AGENTS.md`
- `CLAUDE.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`

## Task order

```
T0 -> T1 -> T2 -> T3
```

## T0 — Plan and task ledger

- **Status:** [x] Done
- **Type:** planning
- **Effort:** S
- **RRI:** n/a

### Goal

Create the plan and task ledger for the agent-session preflight work.

### Acceptance Criteria

- Plan and task ledger exist with OKF frontmatter.
- The implementation task scope is explicit.

## T1 — Shared preflight script and tests

- **Status:** [x] Done
- **Type:** development
- **Effort:** L
- **RRI:** 41 -> Med-high
- **Depends on:** T0

### Goal

Add `scripts/agent-preflight.py`, with unit tests, to print the compact workflow
summary and maintain a session-local sentinel under `.agent/`.

### Acceptance Criteria

- `scripts/agent-preflight.py --print-summary` emits the workflow startup summary.
- `scripts/agent-preflight.py --mark` writes `.agent/session-preflight.json`.
- `scripts/agent-preflight.py --check` exits non-zero when the sentinel is absent
  or stale and exits zero after `--mark`.
- Tests cover the missing-sentinel and marked-sentinel paths.
- No task-specific approval decision is encoded in the script.

### Happy path examples

- `HP-1`: Fresh session runs `--mark` -> sentinel exists -> `--check` passes.
- `HP-2`: `--print-summary` -> output names workflow authority, RRI gate,
  approval threshold, mobile `DESIGN.md`, and Gemma/D14 closure review.

### Edge case examples

- `EC-1`: No sentinel exists -> `--check` fails with actionable instructions.
- `EC-2`: Sentinel belongs to a different repository root -> `--check` fails.

### RRI

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | raw CC 12 -> score 2 (policy CC table) | High |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 3 | agent-supplied: agent workflow/tooling script | High |
| T coverage | 2 | agent-supplied: focused unit tests | High |
| A ambiguity | 1 | task has acceptance criteria and examples | High |
| K coupling | 3 | script feeds future session/hook workflow | High |
| P impact | 2 | developer workflow preflight only; no runtime product path | High |
| X context | 2 | workflow docs plus script/test context | High |

**Final RRI:** 41 -> band Med-high (41-55) -> Effort L.

### Implementation summary

Added `scripts/agent-preflight.py` with:

- compact workflow summary output;
- `.agent/session-preflight.json` sentinel path;
- `--mark` to write the sentinel atomically;
- `--check` to fail when the sentinel is missing, invalid, stale-versioned, or
  marked for another repository root;
- `--repo-root` override for tests and future hook wrappers.

Added `scripts/agent_preflight_test.py` covering the approved happy paths and
edge cases.

### Gemma Reviewer evidence

- Command: `python3 scripts/gemma-code-review.py /tmp/agent-preflight-t1.diff --out /tmp/agent-preflight-t1-review.json --passes 3 --task-id agent-session-preflight-T1`
- Passes run / succeeded: 3 / 3
- Quorum: met
- Aggregate status: `FINDINGS`
- Findings: one minor consensus finding; disposition rejected as non-blocking
  because it explicitly said no immediate action was required and the current
  implementation is sufficient for session setup. The earlier atomic-write
  robustness suggestion was accepted and repaired before this final review.
- Primary-agent disposition: no further code changes required.

### Reflection log

| Pass | Focus | Result |
|---|---|---|
| 1 | API/CLI behavior and sentinel semantics | `--print-summary`, `--mark`, `--check`, and `--repo-root` stay narrow and task-scoped. |
| 2 | Failure modes and repo-root validation | Missing, invalid JSON, wrong root, and version mismatch fail closed with actionable messages. |
| 3 | Test coverage and no hidden approval logic | Tests cover HP/EC cases; script records preflight only and does not encode task-specific RRI approval. |

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Fresh session runs `--mark` -> sentinel exists -> `--check` passes | `scripts/agent_preflight_test.py::AgentPreflightTest.test_hp1_mark_then_check_passes` | passed |
| HP-2 | Happy path | `--print-summary` names workflow authority, RRI gate, approval threshold, mobile `DESIGN.md`, and Gemma/D14 closure review | `scripts/agent_preflight_test.py::AgentPreflightTest.test_hp2_summary_names_required_workflow_rules` | passed |
| EC-1 | Edge case | No sentinel exists -> `--check` fails with actionable instructions | `scripts/agent_preflight_test.py::AgentPreflightTest.test_ec1_check_fails_when_sentinel_missing` | passed |
| EC-2 | Edge case | Sentinel belongs to another repository root -> `--check` fails | `scripts/agent_preflight_test.py::AgentPreflightTest.test_ec2_check_fails_for_different_repo_root` | passed |

### Owner final verification

- Owner: Codex
- Date: 2026-06-28
- Commands run:
  - `python3 -m py_compile scripts/agent-preflight.py scripts/agent_preflight_test.py`
  - `python3 -m unittest scripts/agent_preflight_test.py -v`
  - `python3 scripts/agent-preflight.py --print-summary`
  - `python3 scripts/gemma-code-review.py /tmp/agent-preflight-t1.diff --out /tmp/agent-preflight-t1-review.json --passes 3 --task-id agent-session-preflight-T1`
- Result: all direct verification commands passed; Gemma Reviewer quorum met with
  only a non-blocking minor finding.

## T2 — Claude and Codex hook wiring

- **Status:** [x] Done
- **Type:** configuration
- **Effort:** L
- **RRI:** 51 -> Med-high
- **Depends on:** T1

### Goal

Wire the shared preflight into Claude and Codex startup/edit hooks.

### Acceptance Criteria

- Claude `SessionStart` prints and marks the preflight.
- Claude `PreToolUse` for edit/write actions calls the preflight check.
- Codex project config has equivalent session-start and pre-tool-use hooks where
  supported by the installed Codex configuration.
- Existing user-local permission entries are preserved.

### RRI

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 4 -> score 0 (policy CC table) | High |
| F files | 1 | `--touches` -> 2 files at presentation; `.gitignore` added during implementation to ignore generated sentinel | High |
| D domain | 3 | agent workflow configuration | High |
| T coverage | 2 | manual hook-command verification plus config parsing | High |
| A ambiguity | 1 | task has acceptance criteria | High |
| K coupling | 4 | session/edit hooks affect agent edit flow | High |
| P impact | 3 | developer workflow gate; no product runtime path | High |
| X context | 3 | Claude/Codex config syntax plus T1 script behavior | High |

**Final RRI:** 51 -> band Med-high (41-55) -> Effort L.

### Implementation summary

- Added Claude `SessionStart` hook in `.claude/settings.json` to run
  `scripts/agent-preflight.py --print-summary --mark`.
- Added Claude `PreToolUse` hook for `Write|Edit` to emit a `deny` decision when
  `scripts/agent-preflight.py --check` fails.
- Added Codex inline TOML hooks in `/Users/matias/.codex/config.toml` for
  `SessionStart` and `PreToolUse`, guarded so they execute only when the current
  git root is `/Users/matias/dubbridge`.
- Added `.agent/` to `.gitignore` so the generated sentinel is never tracked.
- Preserved existing local `.claude/settings.json` permission entries.

### Verification evidence

- `python3 -m json.tool .claude/settings.json` — passed.
- `python3 -c 'import tomli; ... tomli.load(...)'` against `/Users/matias/.codex/config.toml` — passed.
- Claude `PreToolUse` command without sentinel — emitted valid JSON deny decision.
- Codex `PreToolUse` command without sentinel — emitted valid JSON deny decision.
- Codex `SessionStart` command — printed preflight summary and marked `.agent/session-preflight.json`.
- Claude and Codex `PreToolUse` commands after sentinel exists — returned 0 with no deny output.
- `git check-ignore -v .agent/session-preflight.json` — matched `.gitignore:18:.agent/`.

## T3 — Verification and close

- **Status:** [x] Done
- **Type:** docs/config verification
- **Effort:** S
- **RRI:** n/a (closure verification)
- **Depends on:** T1, T2

### Goal

Verify the script, docs frontmatter, and hook configuration, then update this
ledger with evidence.

### Acceptance Criteria

- Unit tests pass.
- OKF frontmatter check passes for the new docs.
- Task ledger records verification commands and any skipped external hook check.

### Completion evidence

- `python3 -m py_compile scripts/agent-preflight.py scripts/agent_preflight_test.py` — passed.
- `python3 -m unittest scripts/agent_preflight_test.py -v` — 6 tests passed.
- `python3 scripts/check_okf_frontmatter.py docs/plan/agent-session-preflight-gate.md docs/tasks/agent-session-preflight-gate.md` — passed.
- `bash scripts/check-task-unit-coverage.sh` — passed.
- External hook behavior was tested by executing the configured commands directly;
  no full new-window Claude/Codex restart was performed in this session.

## Closure

All tasks in this ledger are complete. No commit has been made.
