---
type: TaskList
title: "Tasks: Low-RRI Local Delegation Redesign"
status: closed
plan: docs/plan/low-rri-local-delegation-redesign.md
---
# Tasks: Low-RRI Local Delegation Redesign

**Plan:** `docs/plan/low-rri-local-delegation-redesign.md`
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `AGENTS.md`
**Related policies:** `docs/policies/RRI_POLICY.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`

## Status legend

- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 -> T1 -> T2 -> T3
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| T0 | Create the plan and task ledger | — | 14 | Low | S |
| T1 | Align the Low-RRI documentation contract | T0 | 21 | Low | S |
| T2 | Replace JSON parsing with tagged-block parsing in the wrapper | T1 | 42 | Med-high | L |
| T3 | Refresh wrapper tests and final doc references | T2 | 38 | Moderate | M |

## T0 — Create the plan and task ledger

- **Status:** [x] Done — 2026-06-13
- **Type:** Planning / docs
- **Objective:** Create the workflow-mandated plan and ledger for this redesign.
- **Inputs:**
  - approved implementation plan from the conversation;
  - existing plan/task patterns in `docs/plan/*` and `docs/tasks/*`.
- **Outputs:**
  - `docs/plan/low-rri-local-delegation-redesign.md`
  - `docs/tasks/low-rri-local-delegation-redesign.md`
- **Acceptance criteria:**
  - both files exist and describe T0–T3 with dependencies and acceptance
    criteria;
  - the plan records the protocol redesign, scope boundaries, and affected files;
  - the ledger distinguishes documentation work from wrapper/test work.
- **Completion record (2026-06-13):**
  - Created `docs/plan/low-rri-local-delegation-redesign.md` with scope,
    affected files, design decisions, and dependency direction.
  - Created `docs/tasks/low-rri-local-delegation-redesign.md` with T0–T3,
    dependency order, RRI-backed effort bands, and behavioral examples for the
    development tasks.
  - Verification: RRI calculated with `python3 scripts/rri.py` for T0–T3.

## T1 — Align the Low-RRI documentation contract

- **Status:** [x] Done — 2026-06-13
- **Type:** Docs
- **Objective:** Replace the stale JSON / unified-diff guidance with one coherent
  tagged-block contract across the policy, playbook, workflow references, and
  local guidance.
- **Inputs:**
  - `docs/gemma-local-improve.md`
  - `docs/policies/RRI_POLICY.md`
  - `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`
  - `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
  - `docs/policies/HITL_AUTONOMY_POLICY.md`
- **Outputs:**
  - updated Low-RRI documentation set with one active contract.
- **Acceptance criteria:**
  - no active guidance instructs the model to return JSON or a unified diff;
  - the tagged-block format is documented consistently where the protocol is
    specified;
  - the wrapper remains clearly identified as the owner of diff construction and
    application.
- **Completion record (2026-06-13):**
  - Replaced the stale local guidance in `docs/gemma-local-improve.md` with the
    active tagged-block contract and repository source-of-truth references.
  - Updated the Low-RRI delegation sections in `docs/policies/RRI_POLICY.md`,
    `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`,
    `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, and
    `docs/policies/HITL_AUTONOMY_POLICY.md` so they all describe tagged text with
    complete file contents, wrapper-built diffs, and fail-closed review.
  - Orchestrator note: the first Low-RRI preview delegation for T1 was rejected
    because it attempted destructive whole-document rewrites; the final edits were
    applied manually and kept narrow to the Low-RRI sections.
  - Verification:
    - `make qa-docs` — passed.

## T2 — Replace JSON parsing with tagged-block parsing in the wrapper

- **Status:** [x] Done — 2026-06-13
- **Type:** Development
- **Objective:** Update `scripts/delegate-low-rri.py` so it prompts for tagged
  blocks, parses them deterministically, preserves the result JSON shape for the
  orchestrator, and fails closed on malformed or policy-violating responses.
- **Inputs:**
  - `scripts/delegate-low-rri.py`
  - the tagged-block contract from T1
- **Outputs:**
  - updated wrapper implementation
- **Acceptance criteria:**
  - the wrapper sends a short imperative system prompt for the tagged format;
  - streaming assembly still works unchanged at the transport level;
  - parser rejects missing markers, duplicate paths, extra text, invalid actions,
    and non-empty `delete` contents;
  - file-action validation rejects `modify` on missing files and `create` on
    existing files;
  - successful parsing still produces result JSON with `status`, `summary`,
    `files`, `test_commands`, and `risk_notes`.
- **Happy path examples:**
  - `HP-1`: a single tagged `modify` block for an allowed existing file parses
    into one file entry and applies cleanly.
  - `HP-2`: two tagged file blocks for allowed paths produce a deterministic
    multi-file diff and apply cleanly.
- **Edge case examples:**
  - `EC-1`: response text before the first permitted header is rejected before any
    diff is built.
  - `EC-2`: duplicate `PATH` blocks are rejected as malformed.
  - `EC-3`: `ACTION: delete` with non-empty content is rejected.
  - `EC-4`: `ACTION: modify` for a missing file or `ACTION: create` for an
    existing file is rejected.
- **Completion record (2026-06-13):**
  - Replaced the schema-constrained JSON response contract in
    `scripts/delegate-low-rri.py` with a strict tagged-block contract.
  - Added deterministic tagged parsing, duplicate-path rejection, invalid-action
    rejection, empty-content enforcement for `delete`, and file-action validation
    for impossible `create` / `modify` / `delete` targets.
  - Preserved the orchestrator result JSON shape and the existing streaming,
    scope-enforcement, git-diff, and git-apply architecture.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | single tagged `modify` block parses and remains compatible with apply flow | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_single_file_tagged_response`, `scripts/delegate_low_rri_test.py::BuildAndApplyDiff.test_modify_existing_file` | passed |
| HP-2 | Happy path | tagged multi-file response yields deterministic multi-file diff | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_multiple_file_blocks_parse`, `scripts/delegate_low_rri_test.py::BuildAndApplyDiff.test_create_then_apply_multifile` | passed |
| EC-1 | Edge case | unexpected text outside the contract is rejected before diff build | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_unexpected_text_raises` | passed |
| EC-2 | Edge case | duplicate `PATH` blocks are rejected | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_duplicate_path_raises` | passed |
| EC-3 | Edge case | `delete` with non-empty content is rejected | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_delete_requires_empty_content` | passed |
| EC-4 | Edge case | impossible file actions fail closed before apply | `scripts/delegate_low_rri_test.py::ValidateFileActions.test_modify_rejects_missing_target`, `scripts/delegate_low_rri_test.py::ValidateFileActions.test_create_rejects_existing_target` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-13`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m unittest scripts/delegate_low_rri_test.py`; `python3 -m py_compile scripts/delegate-low-rri.py scripts/delegate_low_rri_test.py`; `make qa-docs`

## T3 — Refresh wrapper tests and final doc references

- **Status:** [x] Done — 2026-06-13
- **Type:** Development
- **Objective:** Update unit tests to cover the new parser and hardening rules,
  then sync the remaining documentation references and completion records.
- **Inputs:**
  - `scripts/delegate_low_rri_test.py`
  - outputs from T1 and T2
- **Outputs:**
  - updated unit tests
  - synchronized task ledger completion notes
- **Acceptance criteria:**
  - unit tests cover the tagged parser success and failure modes described in the
    implementation plan;
  - diff/application tests still pass for create, modify, and delete flows;
  - final documentation references match the implemented wrapper behavior.
- **Happy path examples:**
  - `HP-1`: a valid tagged response with one file survives parsing and yields the
    expected result structure.
  - `HP-2`: a valid tagged multi-file response still produces an applicable diff.
- **Edge case examples:**
  - `EC-1`: truncated or malformed tagged output raises a parser failure.
  - `EC-2`: out-of-scope or path-escaping entries are rejected before apply.
- **Completion record (2026-06-13):**
  - Reworked `scripts/delegate_low_rri_test.py` around tagged-block parsing while
    keeping the wrapper-result JSON expectations for downstream orchestration.
  - Added parser coverage for `NO_PATCH`, `BLOCKED`, missing markers, invalid
    actions, duplicate paths, malformed deletes, and file-action guards.
  - Added delete-path diff coverage and kept the existing stream, scope, atomic
    result-write, and CLI dry-run checks green.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid tagged response with one file survives parsing and yields the expected result structure | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_single_file_tagged_response` | passed |
| HP-2 | Happy path | valid tagged multi-file response still produces an applicable diff | `scripts/delegate_low_rri_test.py::BuildAndApplyDiff.test_create_then_apply_multifile` | passed |
| EC-1 | Edge case | truncated or malformed tagged output raises a parser failure | `scripts/delegate_low_rri_test.py::ParseStreamContent.test_missing_end_marker_raises`, `scripts/delegate_low_rri_test.py::ParseStreamContent.test_missing_path_raises` | passed |
| EC-2 | Edge case | out-of-scope or path-escaping entries are rejected before apply | `scripts/delegate_low_rri_test.py::EnforceScope.test_out_of_scope_rejected`, `scripts/delegate_low_rri_test.py::EnforceScope.test_parent_escape_rejected` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-13`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m unittest scripts/delegate_low_rri_test.py`; `python3 -m py_compile scripts/delegate-low-rri.py scripts/delegate_low_rri_test.py`; `make qa-docs`

## Agent handoff prompt (delegation-ready)

> Implement T1 -> T3 from `docs/tasks/low-rri-local-delegation-redesign.md`.
> Governing docs: `docs/plan/low-rri-local-delegation-redesign.md`,
> `docs/policies/RRI_POLICY.md`, `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`.
> Main logic lives in `scripts/delegate-low-rri.py`; tests live in
> `scripts/delegate_low_rri_test.py`.
> Acceptance criteria:
> - replace JSON response handling with tagged-block parsing
> - keep complete-file contents + git-built diff architecture
> - reject malformed, duplicate, and invalid file-action responses
> - keep result JSON fields for the orchestrator
> - align docs with the implemented contract
> Stop condition: finish T3, run the relevant verification, update this ledger,
> and do not start unrelated Low-RRI workflow changes.
