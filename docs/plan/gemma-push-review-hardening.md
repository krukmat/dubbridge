---
type: Plan
title: "Plan: Gemma Push Review Hardening"
status: completed
supersedes: ""
---
# Plan: Gemma Push Review Hardening

> **Status:** Completed - T1-T4 delivered and validated locally on 2026-06-25
> **Tasks ledger:** `docs/tasks/gemma-push-review-hardening.md`
> **Related issue cluster:** `docs/daily/2026-06-25.md` O-06 + live audit regressions observed on 2026-06-25

## Objective

Harden the GitHub `push-review` runtime so each advisory run audits the correct
completed CI event, emits only run-local artifacts, and fails closed when the
audit script produces blocked or degraded output.

## Why This Slice Exists

Live runs on 2026-06-25 showed that the current advisory workflow is not
trustworthy for post-push review:

- a `push-review` run for `main` completed successfully while auditing the wrong
  `workflow_run.head_sha` (`50ba1b6`, not the pushed `acdbeeb`);
- another run uploaded historical Markdown summaries plus a new
  `2026-06-25-unknown.md` blocked report instead of a report keyed to the actual
  push SHA;
- the workflow stayed green because `continue-on-error: true` masks audit
  failures even when `make qa-gemma-push-review` exits non-zero;
- blocked artifacts showed parser failures (`LINE must be >= 1`) and token-limit
  failures, but the workflow summary still required manual log inspection to see
  that the audit was unusable.

The repository now has enough evidence to treat this as a runtime-governance
bug, not just a model-quality issue.

## Scope

### Included

- `.github/workflows/push-review.yml`
- `scripts/gemma-push-review.py`
- `scripts/gemma_push_review_test.py`
- `scripts/gemma_push_ops_test.py`
- task/plan/daily status sync for the hardening work

### Excluded

- changing the core Gemma Push Reviewer product scope or RRI routing model
- redesigning the push-audit prompt beyond what is required for deterministic
  fail-closed handling
- changing the primary `ci` workflow itself

## Design Decisions

### D0 - The `workflow_run` event is the only runtime source of truth

The advisory workflow must bind all runtime identity from the triggering
`workflow_run` event: run id, head SHA, branch, status, and conclusion. The
uploaded artifact name, output directory, packet, and Markdown summary must all
derive from that same event identity.

### D1 - Run-local outputs only

The workflow may upload only artifacts created during the current advisory run.
Historical `docs/reports/push-review/*.md` files checked into the repository or
left on disk from prior runs must not be swept into the uploaded artifact.

### D2 - Advisory does not mean silently green

The push-review remains post-pipeline and non-authoritative, but blocked or
degraded execution must be visible in the workflow result. Artifact upload and
summary steps should still run with `if: always()`, while the job conclusion must
reflect whether the audit actually succeeded.

### D3 - Blocked reports must stay key-addressable

When the audit blocks before a valid SHA-scoped summary can be rendered, the
blocked report must still preserve run identity from the triggering event rather
than collapsing into `unknown`.

## Verification Plan

- unit tests for workflow env wiring and run-local artifact expectations
- unit tests for blocked-report identity when parser/token failures occur
- `python3 -m unittest scripts/gemma_push_review_test.py`
- `python3 scripts/gemma_push_ops_test.py`
- `python3 -m py_compile scripts/gemma-push-review.py scripts/gemma_push_review_test.py`
- one local replay of a known completed run using `--run-id`
- `make qa-docs`
