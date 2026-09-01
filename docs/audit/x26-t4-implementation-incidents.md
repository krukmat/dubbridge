---
type: Audit
title: "X26-T4 implementation incidents and deferred controls"
status: recorded
task: X26-T4
related:
  - docs/tasks/tiger-style-adaptation.md
  - docs/proposals/tiger-style-adaptation-evaluation.md
  - apps/worker-runner/src/translation_fanout.rs
  - crates/db/src/translation_delivery_repo.rs
  - infra/migrations/0031_bound_translation_dispatch_attempts.sql
---

# X26-T4 implementation incidents and deferred controls

This note records the implementation-time incidents observed while closing
`X26-T4` (explicit retry/attempt bounds) on 2026-08-31. It is intentionally
separate from CI success claims: the owner explicitly requested the final
commit/push without waiting for the remaining controls and will review them
later.

## Final implementation state

The final consolidated task commit is
`610d70240026dfe1b481c1a2bab7db01fa8de4b5`
(`feat(X26-T4): bound translation dispatch retries`), with
`1fa4f9b42796ac1975b1f4bf8062641553f5d34a` (`X26-T3c-d`) as its direct
parent.

The implementation adds a durable `attempt_count` to
`translation_dispatch_outbox`, defines
`MAX_TRANSLATION_DISPATCH_ATTEMPTS = 3`, prevents scheduling after the cap,
and persists a terminal `failed` delivery state when retries are exhausted.
The final commit also contains the small inherited audit-test lint cleanup
needed by the current Rust toolchain baseline.

## Incidents observed

### 1. First retry-cap shape broke an existing public enum expectation

An intermediate implementation changed
`TranslationDispatchDisposition::Retryable` from a unit variant to a struct
variant carrying `{ attempt }`. GitHub Actions `cargo-check` rejected an
existing API test in `apps/api/tests/translation_delivery_repo_test.rs`
because it still compared against the unit variant.

**Disposition:** corrected. `Retryable` keeps its previous unit-variant API;
the attempt count is carried separately in the persistence result. The worker
uses that durable count to decide whether another queue delivery is due.

### 2. Formatting failures were partly inherited from X26-T3c-d

The first CI pass reported rustfmt diffs in the new retry-cap test and in
`crates/audit/src/lib.rs`. The audit-file formatting was not introduced by
the retry-bound logic; it was inherited from the preceding `X26-T3c-d`
state.

**Disposition:** both formatting issues were normalized before the final
consolidation.

### 3. Clippy exposed an inherited `useless_vec` finding in the audit test

After the retry-cap compatibility fix, `cargo-check` and `fmt` passed, but
Clippy rejected `crates/audit/src/lib.rs` because the constructor-family test
used `vec![]` where an array was sufficient. This was an inherited T3c-d test
finding, not part of the T4 retry semantics.

**Disposition:** changed the local test collection from `vec![]` to an array.
This cleanup is included in the final T4 commit only because it was required
to restore the current lint baseline during the same direct-main work session.

### 4. `qa-docs` failed on historical S-150 review commit references

The docs consistency job reached `scripts/check-task-unit-coverage.sh` and
failed because multiple review artifacts recorded in
`docs/tasks/s-150-translation-dubbing.md` reference commit SHAs that GitHub no
longer resolves as valid commit objects. The reported entries include
historical S-150 T1/T2/T3 review records and are unrelated to the X26-T4
retry-cap implementation.

**Disposition:** not repaired as part of X26-T4. The owner explicitly directed
that controls be ignored for this push and will review the outstanding
repository hygiene separately. This must not be represented as a T4 code
failure.

### 5. Final commit was consolidated after incremental staging writes

The GitHub contents API produced several intermediate staging commits while
the migration, DB adapter, worker logic, tests, and follow-up fixes were being
assembled. This conflicted with the owner's standing preference of one commit
per completed task.

**Disposition:** the final tree was re-parented directly to the X26-T3c-d
commit and `main` was moved to the single consolidated T4 commit
`610d70240026dfe1b481c1a2bab7db01fa8de4b5`. The intermediate staging commits
are no longer on the `main` ancestry.

## Acceptance-criteria deviation to review

The original T4 task text says that exceeding the retry cap is "durably
audited" and describes an ADR-018 audit row. The implemented terminal state is
durably persisted in `translation_dispatch_outbox` (`delivery_state =
'failed'`, with attempt/error state), but T4 did **not** add a new
`AuditEventKind` or an `audit_events` row for this operational queue failure.

This is an intentional scope decision made during implementation: ADR-018 is
reserved for governance-significant audit events, and introducing a new audit
event family solely for an operational retry ceiling would broaden the audit
contract established in X26-T3c. However, the literal T4 acceptance wording is
therefore not fully satisfied. Owner review should either:

1. accept durable outbox terminal-state persistence as the intended evidence
   for this operational failure, and amend the T4 wording; or
2. create a separately scored task to add a governance/audit event if this
   retry exhaustion is deemed governance-significant.

**Resolved 2026-09-01:** owner selected option 1. `docs/tasks/tiger-style-
adaptation.md`'s X26-T4 EC-1 and acceptance criteria were amended to name
the `translation_dispatch_outbox` terminal row as the accepted evidence;
option 2 (a new `AuditEventKind`) was explicitly declined. See
`docs/audit/x26-verification-2026-09-01.md` open item 1 for the disposition
trail.

## Deferred verification

At the owner's explicit instruction, the final push was not blocked on the
remaining CI/review controls. The following should therefore be treated as
**deferred**, not as passed evidence for the final consolidated commit:

- complete `make qa-test` / GitHub Actions test result;
- coverage gate;
- `qa-docs` repository-history repair;
- any local-model/peer-review workflow required by the normal task discipline.

The code can be reviewed from the final consolidated commit without replaying
the intermediate staging history.
