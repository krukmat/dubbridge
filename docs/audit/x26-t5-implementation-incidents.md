---
type: AuditNote
title: "X26-T5 implementation and CI incidents"
status: recorded
slice: tiger-style-adaptation
related_task: X26-T5
---

# X26-T5 implementation and CI incidents

## Implementation

X26-T5 was implemented directly on `main` in commit
`45e94631631f2971c7fc63fd36effac4b82792af`
(`feat(X26-T5): make S3 integration mandatory in CI`). No local stack was
used.

The implementation:

- adds fail-closed `make qa-test-s3`, requiring all four
  `DUBBRIDGE_STORAGE_TEST_*` variables before the ignored real-S3 test can run;
- adds a dedicated `s3-integration` GitHub Actions job;
- provisions an official MinIO service container with a health check;
- creates the `dubbridge-ci` bucket before the test;
- runs
  `s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint`
  against the CI MinIO endpoint;
- also provisions Postgres in the ordinary `test` job, satisfying T5's explicit
  acceptance option that `make qa-test` execute with the DB-backed path enabled.

## Remote evidence

GitHub Actions push run: `33430140286` for commit
`45e94631631f2971c7fc63fd36effac4b82792af`.

The T5-specific `s3-integration` job (`99613240012`) completed successfully:

1. MinIO service initialization: **PASS**.
2. MinIO health check: **PASS** (the job proceeded past container readiness).
3. S3 integration-test bucket creation: **PASS**.
4. `make qa-test-s3`: **PASS**.
5. Service cleanup: **PASS**.

This closes the MinIO/S3 portion of R12/D3: the previously ignored real S3
adapter path is now exercised unconditionally on normal push/PR CI runs.

## Deferred control incidents

Per the owner's instruction for X26-T5, implementation is not blocked by
unrelated control failures. Failures observed on the same CI run are recorded
here for later review rather than repaired as part of this task.

### INC-T5-01 — `qa-docs` historical S-150 evidence references

**Status:** deferred; unrelated to T5 implementation.

`qa-docs` job `99613240321` failed in
`scripts/check-task-unit-coverage.sh`. The preceding documentation consistency
check and its Python test suite passed. The failing gate reported historical
review `commit_sha` values in `docs/tasks/s-150-translation-dubbing.md` that no
longer resolve as valid commit objects. The reported tasks include S-150-T1a,
T1c-ii, T2a, T2b-ii-a/b/c, T2c-i/ii/iii/iv-a0/iv-b/iv-c/v/vi-a/vi-b, T3a,
T3b, and T3c.

No T5 file is identified by that failure. This is the same pre-existing
S-150 documentation-evidence class already observed during X26-T4.

### INC-T5-02 — DB-enabled `qa-test` exposes auth fixture isolation defect

**Status:** deferred; surfaced by satisfying T5's Postgres-in-`test` acceptance
option, but not an S3/MinIO defect.

After T5 added Postgres to the regular `test` job, DB-aware tests that formerly
self-skipped executed concurrently. Job `99613240152` failed four
`apps/api/src/routes/auth.rs` tests:

- `login_handler_maps_wrong_password_and_unknown_email_to_same_unauthorized`;
- `login_handler_returns_ok_and_emits_success_audit`;
- `register_handler_maps_duplicate_email_to_conflict`;
- `register_handler_returns_created_and_emits_audit`.

The Postgres log shows concurrent inserts colliding on
`user_account_email_key` for `owner@example.com`. The failures therefore expose
shared-fixture/test-isolation behavior when the full workspace test suite runs
with Postgres available; they do not originate from the MinIO service or
`qa-test-s3`.

The Redis probe/integration steps were skipped only because `make qa-test`
failed first.

## T5 closure disposition

T5's implementation objective is complete: MinIO is provisioned in CI and the
real S3 integration test is mandatory and has passed remotely. The two control
incidents above remain explicit follow-up items and are intentionally not folded
into T5 implementation work.
