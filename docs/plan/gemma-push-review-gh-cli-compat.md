---
type: Plan
title: "Plan: Gemma Push Review gh CLI Compatibility"
status: complete
supersedes: ""
---
# Plan: Gemma Push Review gh CLI Compatibility

> **Status:** Complete
> **Tasks ledger:** `docs/tasks/gemma-push-review-gh-cli-compat.md`
> **Related issue:** `docs/daily/2026-06-25.md` O-05

## Objective

Restore local `scripts/gemma-push-review.py --run-id` replay on environments
where the installed GitHub CLI exposes the workflow run attempt field as
`attempt` instead of `runAttempt`.

## Scope

### Included

- `scripts/gemma-push-review.py`
- `scripts/gemma_push_review_test.py`
- task/plan/daily status synchronization for the fix

### Excluded

- changes to the GitHub Actions `workflow_run` payload contract
- changes to push-review routing, parsing, or audit authority
- broader `gh` schema discovery beyond the run-attempt compatibility gap

## Design Decisions

### D0 - Negotiate the gh JSON field name at runtime

The wrapper should try the legacy `runAttempt` field first and fall back to
`attempt` when the local `gh` binary rejects the request as an unknown field.

### D1 - Keep normalized runtime shape stable

Internal run metadata stays normalized as `run_attempt` regardless of whether
the source payload used `runAttempt` or `attempt`.

### D2 - Preserve workflow event compatibility

GitHub `workflow_run` event payloads already use `runAttempt`; the local replay
fix must not regress that path.

## Verification Plan

- unit tests for both `gh` field-name variants
- `python3 -m unittest scripts/gemma_push_review_test.py`
- `python3 -m py_compile scripts/gemma-push-review.py scripts/gemma_push_review_test.py`
- `make qa-docs`
