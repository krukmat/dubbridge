---
type: Plan
title: "CI Red Fixes — 2026-09"
status: active
description: "Plan for the three concrete, low-risk fixes to main's red CI jobs (coverage, qa-docs, test), diagnosed in docs/audit/ci-red-findings-2026-09-01.md."
---

# CI Red Fixes — 2026-09

## Objective

Turn `main`'s three failing CI jobs (`test`, `coverage`, `qa-docs`) green with
the smallest correct change per finding, without weakening any gate's actual
check.

## Context

`docs/audit/ci-red-findings-2026-09-01.md` diagnosed three independent, unrelated
root causes as of commit `f3adf34` (post-merge of PR#6). None was introduced by
that PR; all three are confirmed pre-existing on `main`. `docs/plan/roadmap.md`
§ Cross-cutting obligations `X28` tracks this at the roadmap level.

## Affected files

- `apps/cli/tests/migrate_test.rs` (T1 — coverage)
- `.github/workflows/ci.yml` (T2 — qa-docs)
- `Makefile` (T3 — test, fast unblock)

## Design decisions

- **T1/T2 are pure corrections of stale/incomplete state** — no design choice
  involved. T1 updates a literal count to match reality; T2 mirrors a
  `fetch-depth: 0` fix two sibling jobs (`maintainability`,
  `peer-workflow-review`) already carry for the identical reason.
- **T3 is the fast-unblock option for the `test` job's shared-DB race**, not
  the durable fix. It mirrors `qa-coverage`'s existing `-- --test-threads=1`
  workaround for the same bug class, applied to `qa-test`. This is a
  deliberate scope choice: it eliminates the race (tests stop running
  concurrently against the shared Postgres test database) without touching
  any test's logic, at the cost of a slower `test` job.
- **Explicitly out of scope for this plan:** the durable per-test database
  isolation redesign (transaction-per-test, unique-per-test seed data, or
  schema-per-test) that would let `qa-test` return to parallel execution.
  Scored separately at RRI 43 (Med-high) — driven by a genuine
  architecture-decision penalty, since the isolation *strategy* itself is
  undecided and the shared-truncate anti-pattern likely recurs in test files
  beyond `apps/api/src/routes/auth.rs` and `apps/api/tests/workspace_test.rs`.
  Decomposing an unmade design decision into local-dev-executable Low
  subtasks isn't sound; this is recorded as a parked Med-high follow-up
  (see `docs/tasks/ci-red-fixes-2026-09.md` § T4) requiring an owner decision
  on strategy before it can be scoped and decomposed.

## Task sequence

T1, T2, T3 are fully independent — no ordering dependency, may be delegated
and merged in any order or in parallel.

## Related

- `docs/audit/ci-red-findings-2026-09-01.md` — root-cause evidence for all
  three findings.
- `docs/plan/roadmap.md` § Cross-cutting obligations, `X28`.
- `docs/tasks/ci-red-fixes-2026-09.md` — task ledger.
