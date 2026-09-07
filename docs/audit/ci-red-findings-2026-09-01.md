---
type: Audit
title: "CI Red Findings — main, 2026-09-01"
status: active
description: "Root-cause diagnosis of the three failing main CI jobs (test, coverage, qa-docs) as of commit f3adf34, for follow-up fix work."
---

# CI Red Findings — `main`, 2026-09-01

Investigated HEAD: `f3adf34b5722890356d359892334812d66f9f453` (post-merge of
PR#6, `feat/ckg-context-provider`). Confirmed pre-existing at the PR's base
commit `c65b4583ab86adcbe1805b6a6288603d13cef54a` as well — none of the three
findings below were introduced by PR#6; that PR's diff does not touch any of
the files implicated.

CI runs referenced:
- `main` @ `f3adf34`: run `33520862123`
- `main` @ `c65b458` (pre-merge baseline): run `33512265188`
- PR#6 head @ `585db00`: run `33519062816`

Failing jobs, all three runs: `test`, `coverage`, `qa-docs`. 27+ other jobs
(clippy, fmt, deny, mobile, s3-integration, release-build, cargo-check,
config-secrets, maintainability, python-complexity, roadmap-drift,
peer-workflow-review, local-agent-tests) pass clean on all three runs.

## Finding 1 — `test` job: flaky auth-route tests (shared-DB race)

**Severity:** Medium (non-deterministic, blocks the `test` CI gate; no
production-code defect).

**Symptom:** `cargo test --workspace --all-features` fails inside
`apps/api/src/routes/auth.rs`'s test module, but the *specific* failing
tests differ between runs:

- Run `33512265188` / `33519062816` (identical failing set):
  `login_handler_returns_ok_and_emits_success_audit`,
  `login_handler_fails_closed_when_audit_persistence_fails`,
  `register_handler_maps_validation_errors_to_bad_request`,
  `register_handler_fails_closed_when_audit_persistence_fails` (this last one
  actually passed in the `qa-docs`-log excerpt captured — see raw logs for
  exact per-run set).
- Run `33520862123` (post-merge, otherwise same code path):
  `login_handler_returns_ok_and_emits_success_audit`,
  `login_handler_fails_closed_when_audit_persistence_fails`,
  `register_handler_maps_duplicate_email_to_conflict`.

A different subset failing on effectively the same code across runs is
itself the signature of a race condition, not a deterministic logic bug.

**Root cause:** `apps/api/src/routes/auth.rs`'s test module
(`migrate_and_reset`, ~line 582) does:

```rust
async fn migrate_and_reset(pool: &PgPool) {
    sqlx::migrate!("../../infra/migrations").run(pool).await.expect("migrations");
    sqlx::query(
        "TRUNCATE TABLE user_account, organizations, audit_events RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate auth tables");
}
```

This truncates **shared, global tables** against the one Postgres test
database every test in the binary connects to
(`DUBBRIDGE_DATABASE_URL`), with no per-test isolation (no transaction
rollback, no per-test schema, no locking). `make qa-test` runs
`cargo test --workspace --all-features` with **no `--test-threads=1`** (see
`Makefile:qa-test`), so cargo's default parallel-thread test execution lets
multiple `#[tokio::test]` functions in this module run concurrently against
the same DB. Two failure shapes result:

1. **`seed_account: Conflict`** (the panics at `auth.rs:626:14`): one test's
   `seed_account()` helper races another concurrently-running test's
   `TRUNCATE` + re-seed, or two tests briefly hold conflicting rows against
   `user_account_email_key`, so `auth_service.register(...)` returns a
   duplicate-email `Conflict` where the test expected success.
2. **Wrong status code** (`register_handler_maps_duplicate_email_to_conflict`
   asserting `left: 409, right: 201`, i.e. expected `201 Created` and got
   `409 Conflict`): the email this test expects to be fresh already exists
   because a concurrently-running test seeded it first.

`qa-coverage` (the `coverage` job) does **not** exhibit this specific failure
because its Makefile recipe explicitly passes `-- --test-threads=1`
(serializing all tests in the binary) — confirming the fix is already known
and applied in one place but not the other.

**Proposed fix (not yet applied):** either (a) run `qa-test` with
`--test-threads=1` to match `qa-coverage`'s existing workaround (simple,
but slows the `test` job and only papers over the underlying isolation gap
for every future test file that follows this same shared-truncate pattern),
or (b) give each test file/module a real isolation boundary (per-test
transaction with rollback, or a uniquely-named seed email/workspace per
test instead of a shared truncate-and-reseed pattern). (b) is the more
durable fix; (a) is the fast unblock. Related precedent: `fb0b92f fix(ci):
make workspace_test migrate_and_reset survive a fresh database` fixed a
same-class issue in `apps/api/tests/workspace_test.rs` — this finding is the
same bug pattern recurring in `auth.rs`'s independent copy of
`migrate_and_reset`, not a regression of that fix.

## Finding 2 — `coverage` job: stale hardcoded migration count

**Severity:** Low (single hardcoded constant, purely mechanical fix).

**Symptom:** `apps/cli/tests/migrate_test.rs::migrations_apply_and_are_idempotent_on_second_run`
panics:

```
assertion `left == right` failed: expected exactly 29 applied migrations, found 31
  left: 31
 right: 29
```

**Root cause:** `apps/cli/tests/migrate_test.rs:31-34` hardcodes
`assert_eq!(count, 29, ...)`, and the doc comment above the test still says
"all 29 migration files apply." `infra/migrations/` currently contains
**31** `.sql` files. `git log` shows the two newest
(`0030_add_platform_ingest_correlation_to_audit_events.sql`,
`0031_bound_translation_dispatch_attempts.sql`) landed via
`1fa4f9b feat(X26-T3c-d)` and `610d702 feat(X26-T4)`, both after
`migrate_test.rs` was last touched (`a0d2371 S-230-T1b/T2`) — the test was
simply never updated when those two migrations were added.

This is the failure that gates the `coverage` job overall (it runs before
llvm-cov can produce a percentage; the job never reaches the 90% threshold
check).

**Proposed fix (not yet applied):** update `apps/cli/tests/migrate_test.rs:33`
to `assert_eq!(count, 31, ...)` and the preceding doc comment
("all 31 migration files"). Trivial one-line/two-line change; re-verify the
literal count against `ls infra/migrations/*.sql | wc -l` at fix time in
case more migrations have landed by then.

## Finding 3 — `qa-docs` job: shallow-clone breaks historical commit_sha validation

**Severity:** Low (CI configuration gap, not a content defect — the flagged
data is correct).

**Symptom:** `bash scripts/check-task-unit-coverage.sh` (part of `make
qa-docs`) reports 17 entries in `docs/tasks/s-150-translation-dubbing.md`
whose `Review artifact commit_sha` is "not a valid commit object", e.g.:

```
docs/tasks/s-150-translation-dubbing.md: S-150-T1a: ... commit_sha
'42fd1123c876f9e0278d9605749033f173df56ab' is not a valid commit object
```

**Root cause — confirmed NOT a content defect.** Running the identical
script locally (full git history) passes clean:

```
$ bash scripts/check-task-unit-coverage.sh
Task completion evidence check passed.
$ git cat-file -e 42fd1123c876f9e0278d9605749033f173df56ab   # exit 0, exists
```

`scripts/check-task-unit-coverage.sh:257` validates each citation with
`git cat-file -e "${receipt_commit_sha}^{commit}"`, which requires the
commit object to be present in the local repository — impossible on a
shallow clone that only fetched the tip commit. `.github/workflows/ci.yml`'s
`qa-docs` job checkout step (line 17) has **no `fetch-depth: 0`**, so
`actions/checkout@v4` defaults to `fetch-depth: 1` (tip commit only). Two
other jobs in the same workflow already carry the `fetch-depth: 0` fix for
this exact reason — `maintainability` (line 154-156) and
`peer-workflow-review` (line 252-254) — `qa-docs` was simply missed when
that fix was applied elsewhere, despite needing full history for the same
class of check.

**Proposed fix (not yet applied):** add `fetch-depth: 0` to the `qa-docs`
job's `actions/checkout@v4` step in `.github/workflows/ci.yml` (mirroring
the `maintainability`/`peer-workflow-review` jobs exactly). One-line
workflow change; no script or doc content needs to change.

## Summary

| Job | Root cause | Fix size | Blocks other jobs? |
|---|---|---|---|
| `test` | Shared-DB race in `auth.rs` test truncate/reseed under parallel `cargo test` | Medium (isolation redesign) or Low (add `--test-threads=1`, same as `qa-coverage`) | No |
| `coverage` | Stale hardcoded `29` in `migrate_test.rs`, now 31 real migrations | Low (1-2 line edit) | Gates the 90% coverage check itself |
| `qa-docs` | `qa-docs` job checkout missing `fetch-depth: 0` (shallow clone) | Low (1-line workflow edit) | No — content is already correct |

None of the three interact with each other; they can be fixed independently
and in any order. `qa-docs` and `coverage` are both single-line-class fixes
with no design ambiguity. `test` has a fast unblock (mirror `qa-coverage`'s
`--test-threads=1`) and a more durable follow-up (real per-test DB
isolation) — recommend applying the fast unblock first and tracking the
isolation redesign separately, since the same shared-truncate pattern likely
recurs in other test files beyond `auth.rs` and `workspace_test.rs`.

## Related

- `docs/audit/ckg-m1-m2-local-merge-audit-result.md` — the audit that
  surfaced this investigation while checking PR#6's merge readiness.
- `docs/plan/roadmap.md` § Cross-cutting obligations, `X26` — records the
  same-class `workspace_test.rs` migration-reset race that was already fixed
  (`fb0b92f`); this doc's Finding 1 is the same bug pattern's independent
  recurrence in `auth.rs`, not a regression of that fix.
