---
type: TaskList
title: "Tasks: Gemma Push Review gh CLI Compatibility"
plan: docs/plan/gemma-push-review-gh-cli-compat.md
status: complete
rri: 18
band: Low
effort: S
---
# Tasks: Gemma Push Review gh CLI Compatibility

## Objective

Fix the local push-review replay path so `scripts/gemma-push-review.py --run-id`
works with GitHub CLI builds that expose the workflow run attempt field as
`attempt` instead of `runAttempt`.

## Governing Documents

- `docs/plan/gemma-push-review-gh-cli-compat.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/daily/2026-06-25.md`

## Ground Truth

| ID | File | Description | RRI |
|---|---|---|---|
| O-05 | `scripts/gemma-push-review.py` | local `gh run view/list --json ... runAttempt ...` fails on gh 2.95.0 because the installed CLI only exposes `attempt` | 18 |

---

## T0 - Create the compatibility fix plan and task ledger

- **Status:** done ✅
- **Type:** documentation / planning
- **Effort:** S
- **RRI:** 7 -> Low
- **Scope:** `docs/plan/gemma-push-review-gh-cli-compat.md`,
  `docs/tasks/gemma-push-review-gh-cli-compat.md`
- **Depends on:** none

### Objective

Create the minimal plan/task artifacts required to execute the compatibility
fix under the repository workflow.

### Acceptance Criteria

1. A dedicated plan exists for the compatibility fix
2. A dedicated task ledger exists with acceptance criteria and HP/EC coverage
3. The task references the open O-05 debt from the daily note

### Completion Evidence

- Created `docs/plan/gemma-push-review-gh-cli-compat.md`
- Created `docs/tasks/gemma-push-review-gh-cli-compat.md`

---

## T1 - Support `gh` run-attempt field-name compatibility

- **Status:** done ✅
- **Type:** development
- **Effort:** S
- **RRI:** 18 -> Low
- **Scope:** `scripts/gemma-push-review.py`,
  `scripts/gemma_push_review_test.py`
- **Depends on:** T0

### Objective

Make local run resolution work when `gh run view` / `gh run list` require
`attempt` instead of `runAttempt`, while preserving existing event-payload
normalization and report behavior.

### Happy Path Examples

- **HP-1:** Local replay with `--run-id` on a `gh` binary that supports only
  `attempt` -> wrapper falls back automatically, resolves the run, and continues
  building the push-review packet.
- **HP-2:** `workflow_run` event payload with `runAttempt` -> wrapper still
  normalizes `run_attempt` correctly without needing the `gh` fallback path.

### Edge Case Examples

- **EC-1:** `gh` supports the legacy `runAttempt` field -> wrapper keeps the
  first request and does not perform unnecessary retries.
- **EC-2:** `gh` rejects one field-name variant as unknown -> wrapper retries
  with the other variant instead of surfacing an operational failure.
- **EC-3:** a run payload contains only `attempt` -> normalization still records
  `run_attempt` with the right value.

### Acceptance Criteria

1. `resolve_run()` succeeds against `gh` variants that expose either
   `runAttempt` or `attempt`
2. internal normalized run metadata continues to use `run_attempt`
3. unit tests cover both field-name variants plus normalization
4. `python3 -m unittest scripts/gemma_push_review_test.py` passes
5. `python3 -m py_compile scripts/gemma-push-review.py scripts/gemma_push_review_test.py` passes

### Execution log

- Confirmed the host `gh` behavior with `gh version` (`2.95.0`) and direct CLI
  probes: `gh run view ... --json runAttempt,...` fails locally because this
  build exposes `attempt`.
- Added runtime `gh` JSON-field negotiation in
  `scripts/gemma-push-review.py::_run_gh_json_with_attempt_compat` so
  `resolve_run()` retries with `attempt` when `runAttempt` is rejected.
- Expanded normalization in `scripts/gemma-push-review.py::_normalize_run` so
  either source field still maps to the internal `run_attempt` key.
- Added unit coverage for both `run view` and `run list` fallback behavior plus
  the alias normalization path in `scripts/gemma_push_review_test.py`.
- Replayed the real run with
  `python3 scripts/gemma-push-review.py --run-id 28157583084 --dry-run --force --out-dir /tmp/dubbridge-push-review-o05`
  and verified it wrote `/tmp/dubbridge-push-review-o05/packet.json` instead of
  failing on the `gh` JSON schema mismatch.

### Arbiter verdict

The compatibility fix is complete and verified. Local replay now passes the
former `runAttempt`/`attempt` boundary on this machine. A separate residual
issue was observed during validation: `--collect-only` with a brand-new
`--out-dir` still assumes the directory already exists before writing the
sentinel artifact. That path is out of scope for O-05 and remains open.

### Gemma Reviewer evidence

- Command: `make qa-gemma-review`
- Result: passed; aggregate written to `/tmp/dubbridge-gemma-review.json`
- Notes: all three passes emitted the known `STATUS PASS` + findings warning and
  were coerced by the reviewer parser; the aggregate artifact recorded `0`
  concrete findings and the command exited successfully

### Happy paths covered

- **HP-1:** `scripts/gemma-push-review.py::_run_gh_json_with_attempt_compat`
  retries `resolve_run()` lookups with `attempt` after an unknown-field failure.
  Test evidence:
  `scripts/gemma_push_review_test.py::ResolveRunById::test_falls_back_to_attempt_field_when_run_attempt_is_unknown`.
- **HP-2:** `scripts/gemma-push-review.py::_normalize_run` still preserves the
  workflow-event path that provides `runAttempt`. Test evidence:
  `scripts/gemma_push_review_test.py::ResolveRunFromEvent::test_workflow_run_event_resolves_without_gh_call`.

### Edge cases covered

- **EC-1:** legacy `runAttempt` support does not trigger a retry. Code evidence:
  `scripts/gemma-push-review.py::_run_gh_json_with_attempt_compat`; test
  evidence:
  `scripts/gemma_push_review_test.py::ResolveRunById::test_legacy_run_attempt_field_does_not_retry`.
- **EC-2:** unknown-field rejection on `runAttempt` retries with `attempt` for
  both `run view` and `run list`. Test evidence:
  `scripts/gemma_push_review_test.py::ResolveRunById::test_falls_back_to_attempt_field_when_run_attempt_is_unknown`,
  `scripts/gemma_push_review_test.py::ResolveRunUnavailable::test_run_list_falls_back_to_attempt_field`.
- **EC-3:** payloads that contain only `attempt` still normalize to
  `run_attempt`. Test evidence:
  `scripts/gemma_push_review_test.py::NormalizeRun::test_normalize_attempt_field_alias`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `--run-id` replay falls back from `runAttempt` to `attempt` and resolves the run | `scripts/gemma_push_review_test.py::ResolveRunById::test_falls_back_to_attempt_field_when_run_attempt_is_unknown` | passed |
| HP-2 | Happy path | `workflow_run` event payload with `runAttempt` still normalizes correctly | `scripts/gemma_push_review_test.py::ResolveRunFromEvent::test_workflow_run_event_resolves_without_gh_call` | passed |
| EC-1 | Edge case | legacy `runAttempt` support succeeds without a retry | `scripts/gemma_push_review_test.py::ResolveRunById::test_legacy_run_attempt_field_does_not_retry` | passed |
| EC-2 | Edge case | unknown `runAttempt` field retries with `attempt` for `run list` too | `scripts/gemma_push_review_test.py::ResolveRunUnavailable::test_run_list_falls_back_to_attempt_field` | passed |
| EC-3 | Edge case | payloads with only `attempt` still normalize to `run_attempt` | `scripts/gemma_push_review_test.py::NormalizeRun::test_normalize_attempt_field_alias` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-25`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior, and I confirmed the real local replay path now clears the `gh` JSON field mismatch on this host.
- Commands run: `python3 -m unittest scripts/gemma_push_review_test.py`; `python3 -m py_compile scripts/gemma-push-review.py scripts/gemma_push_review_test.py`; `python3 scripts/gemma-push-review.py --run-id 28157583084 --dry-run --force --out-dir /tmp/dubbridge-push-review-o05 >/tmp/dubbridge-push-review-o05/payload.json 2>/tmp/dubbridge-push-review-o05/stderr.log`; `make qa-gemma-review`; `make qa-docs`
