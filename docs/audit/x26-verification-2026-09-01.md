---
type: Audit
title: "X26 independent verification (2026-09-01)"
status: recorded
related:
  - docs/tasks/tiger-style-adaptation.md
  - docs/plan/tiger-style-adaptation.md
  - docs/audit/x26-t3c-correlation-contract.md
  - docs/audit/x26-t4-implementation-incidents.md
  - docs/audit/x26-t5-implementation-incidents.md
  - docs/audit/x26-t6-implementation.md
  - docs/audit/x26-t7-implementation.md
  - docs/audit/x26-t7-implementation-incidents.md
  - docs/audit/x26-t8-implementation.md
  - docs/audit/x26-t9-implementation.md
  - docs/audit/x26-t10-implementation.md
  - docs/audit/x26-t11-implementation.md
  - docs/audit/x26-t12-forward-pointer-closure.md
---

# X26 independent verification (2026-09-01)

Owner requested an independent verification pass over the X26 batch
(`X26-T3c-d`, `T4`, `T6`–`T12`) that was implemented directly on `main` on
2026-08-31 under explicit owner instruction to bypass the normal per-task
presentation/approval and band-routed review workflow. This note records
what was independently checked, what was found, and what remains open.

## Scope

- Rust changes (`X26-T3c-d`, `X26-T4`): reviewed by a context-isolated
  subagent against `docs/audit/x26-t3c-correlation-contract.md` and
  `x26-t4-implementation-incidents.md`.
- Python ASR worker hardening (`X26-T6`–`T11`): reviewed directly, with real
  local execution (Python 3.11, `jsonschema==4.26.0`, `ruff==0.16.5`).
- CI state on the latest `main` commit (`a3aa481e5b2d78e7c56884e419871f2c9ed3603b`):
  investigated the two failing gates (`deny`, `coverage`) that no X26 incident
  note mentions.
- Documentation sync: cross-checked `docs/tasks/tiger-style-adaptation.md`,
  `docs/plan/tiger-style-adaptation.md`, and `docs/plan/roadmap.md` against
  the actual merged commits.

## Findings

### 1. Rust — correct, one minor/theoretical gap

Verified: the family-specific audit-correlation assertion in
`crates/audit/src/lib.rs` covers all 31 `AuditEventKind` variants; both
`crates/db/src/audit_repo.rs` insert paths and row-mapping bind/rehydrate
`platform_ingest_session_id`; migrations `0030`/`0031` are safe; the
translation-dispatch retry-cap state machine
(`crates/db/src/translation_delivery_repo.rs`,
`apps/worker-runner/src/translation_fanout.rs`) enforces exactly 3 attempts
with no lost-counter or cross-worker race path (Postgres row-level locking on
the claiming `UPDATE` prevents it); `TranslationDispatchDisposition::Retryable`
is confirmed still a unit variant as the T4 incident note claims.

**Minor/theoretical gap:** `has_valid_consent_correlation`,
`has_valid_review_correlation`, and `has_valid_playback_correlation` in
`crates/audit/src/lib.rs` check only the three correlation-ID fields, not
`asset_id`, even though the contract matrix documents `asset_id=Some` as part
of those families' expected shape. `AuditEvent.asset_id` is a public field;
nothing currently overwrites it after construction, so this is not reachable
through any live call site today — flagged for awareness, not a defect
requiring immediate action.

### 2. Python ASR worker — correct, one regression-coverage gap

Re-ran independently (not merely re-read): `pytest tests/test_worker.py`
passes **16/16**; `ruff check workers` (pinned 0.16.5) reports **no
findings**. Confirmed in `main.py` that T11's fix is present: `_transcribe_
with_timeout` materializes `list(segments)` before cancelling the `SIGALRM`
deadline, so the timeout covers faster-whisper's actual (lazy-generator)
transcription work, not just the initial `model.transcribe()` call.

**Gap:** `test_transcription_timeout_uses_distinct_error` (the only test
exercising the timeout path) simulates a slow **synchronous** `transcribe()`
call (`time.sleep()` inside the mocked call), not a fast-returning call with
a slow-to-iterate generator — the exact shape of the bug T11 discovered and
fixed. A future regression of that fix (moving `list(segments)` back after
the alarm cancel) would not be caught by the default, CI-executed mocked
suite — only by the opt-in `test_real_model_smoke.py`
(`DUBBRIDGE_ASR_REAL_MODEL_SMOKE=1`), which does not run in normal CI/PR
flow. Not a current defect; a missing regression guard.

### 3. `deny` CI gate — pre-existing, unrelated to X26

`Cargo.lock` was not touched by any X26-T3c-d–T12 commit (empty diff
80cef28..a3aa481). Confirmed by checking CI run `33415876641` on commit
`80cef28` (immediately before the X26 batch started): `deny` was **already
failing** there, on the same two findings present today —
`RUSTSEC-2026-0258` (h2 0.4.14, unbounded empty DATA frames, low severity,
transitively via axum/hyper/object_store) and yanked crate `chacha20 0.10.1`
(via `rand 0.10.2` → `object_store`). This is a pre-existing, unrelated
dependency-policy gap, not introduced by this batch.

### 4. `coverage` CI gate — new regression, root cause is pre-existing code

On commit `80cef28` (before the X26 batch), `coverage` **passed**. On the
current `main` tip it fails with `relation "audit_events" does not exist`,
`column "platform_ingest_session_id" of relation "audit_events" does not
exist`, plus assorted unique/FK collisions across several tables.

Root cause reproduced locally: `apps/api/tests/workspace_test.rs`'s
`migrate_and_reset()` helper (lines 641–660, introduced in `6eb6cf9`
`fix(ci): recover audit_events after fail-closed drop in workspace_test`,
predating X26 by months) assumes `_sqlx_migrations` already exists. It always
runs `SELECT 1 FROM pg_tables WHERE tablename='audit_events'` and, if absent,
unconditionally runs `DELETE FROM _sqlx_migrations WHERE version IN (4, 9)`
to force a re-migrate. On a **truly fresh** database (no `_sqlx_migrations`
table at all — exactly what CI's `coverage` job provisions every run via an
ephemeral Postgres service container), that `DELETE` itself fails with
`42P01 relation "_sqlx_migrations" does not exist`, and every test in the
file (`workspace_test.rs`) fails its setup. Reproduced locally: against a
freshly created database, `cargo test -p dubbridge-api --test workspace_test
-- --test-threads=1` fails all 14 tests; the isolated single-test run shows
the exact `42P01` error.

This bug is **pre-existing** (not introduced by X26), but its CI-visible
symptom (`coverage` gate failing) is **new** as of this batch — most likely
because the batch's changes (new migrations, new test files) shifted
workspace test-binary execution order enough that `workspace_test.rs` now
sometimes runs before any other binary has completed a full
`sqlx::migrate!()` pass against the fresh CI database. It is timing/ordering
dependent, so it may not reproduce on every run.

**Not yet fixed** — this note only records root cause. A fix (e.g., making
`migrate_and_reset` tolerate a missing `_sqlx_migrations` table, or running
an explicit `sqlx::migrate!().run(&pool)` once before checking for
`audit_events`) needs its own scoped task.

### 5. Documentation-sync gap across three governing docs

Before this note, `docs/tasks/tiger-style-adaptation.md` showed 8 of the 9
tasks implemented in this batch (`T3c-d`, `T4`, `T6`–`T12`) as `[ ] Planned`
(only `T5` was `[x] Done`); `docs/plan/tiger-style-adaptation.md` had
"Implementation note" sections only for `T4`/`T5`/`T6` (none for `T3c-d` or
`T7`–`T12`); and `docs/plan/roadmap.md`'s X26 row said only `T3c-d`'s
audit-boundary integration remained and that `X26-T12` stays open until
S-150 `T4` resumes — both contradicted by the actual merged state and by
`T12`'s own closure note. All three have been corrected in the same pass as
this note (see the diff introducing this file).

### 6. OKF frontmatter — 9 of the batch's own audit notes fail `make qa-okf-frontmatter`

`make qa-okf-frontmatter` (`scripts/check_okf_frontmatter.py`) fails on 9
files, all part of this same X26-T6–T12 batch:

- `docs/audit/x26-t10-implementation.md`, `x26-t11-implementation.md`,
  `x26-t12-forward-pointer-closure.md`, `x26-t7-implementation.md`,
  `x26-t7-implementation-incidents.md`, `x26-t8-implementation.md`,
  `x26-t9-implementation.md` — **missing or malformed frontmatter block**
  entirely (no `---`-delimited YAML header at all).
- `docs/audit/x26-t4-implementation-incidents.md`,
  `x26-t5-implementation-incidents.md` — `type: AuditNote`, which is **not**
  in `docs/knowledge/README.md`'s closed 10-value vocabulary (the correct
  value for `docs/audit/*.md` is `type: Audit`). This note's own file
  originally used the same invalid value and was corrected to `Audit` before
  this note was finalized.

Since the batch was pushed directly to `main` bypassing the normal
pre-commit/`qa-docs` gate (by explicit owner instruction), this defect was
never caught at commit time. It does not affect the correctness of the
underlying implementation, but it means `make qa-docs`/`qa-okf-frontmatter`
and the `pre-commit` hook are currently **red on `main`** for reasons
entirely attributable to this batch's own audit trail. Left unfixed pending
owner disposition — these are pre-existing audit records, not something to
edit without direction.

## Open items requiring owner disposition

All five items below were dispositioned and closed on 2026-09-01, the same
day this note was authored, as five separately presented RRI-scored tasks
(`X26-Frontmatter-Fix`, `X26-Deny-Bump`, `X26-Coverage-Fix`,
`X26-ASR-Regression-Test`, `X26-T4-Audit-Row`).

1. **Closed — `X26-T4-Audit-Row`.** Owner selected option 1 (accept the
   `translation_dispatch_outbox` terminal row as evidence; decline a new
   `AuditEventKind`). `docs/tasks/tiger-style-adaptation.md`'s X26-T4 EC-1
   and acceptance criteria amended accordingly;
   `docs/audit/x26-t4-implementation-incidents.md` records the resolution.
2. **Closed — `X26-Deny-Bump`.** `cargo update -p h2` (0.4.14→0.4.19) and
   `cargo update -p chacha20` (0.10.1→0.10.2) resolved both findings without
   touching `object_store`'s pinned git `rev`. Verified: `cargo deny check`
   (advisories/bans/licenses/sources all ok), `cargo check --workspace
   --all-features` clean, `cargo test --workspace --all-features` 861
   passed / 0 failed.
3. **Closed — `X26-Coverage-Fix`.** Verification against real Postgres
   surfaced a second, related root cause beyond the reordering fix:
   migration `0030` (added the same day as this batch) was never in the
   `migrate_and_reset()` `version IN (4, 9)` repair list, so the repair path
   recreated `audit_events` without `platform_ingest_session_id`. Fixed as
   `version IN (4, 9, 30)` plus the migrate-before-DELETE reorder. Verified
   against a genuinely fresh Postgres database (14/14 `workspace_test.rs`
   pass, vs. 14/14 failing with `42P01` before) and the full
   `dubbridge-api` suite (290 passed, 0 failed).
4. **Closed — `X26-ASR-Regression-Test`.** Added
   `test_transcription_timeout_covers_slow_generator_iteration`, using a
   real Python generator (not `MagicMock`) that sleeps per `yield` paired
   with an instantly-returning `transcribe()` mock. Confirmed as a genuine
   regression guard by manually reverting the T11 fix in `main.py` and
   observing the new test fail (`DID NOT RAISE TranscriptionTimeoutError`);
   `main.py` was then restored to its original state (zero `git diff`).
5. **Closed — `X26-Frontmatter-Fix`.** All 9 files corrected via real Qwen
   local delegation (`qwen3.8:27b-mlx`, `before-after` mode), one attempt
   each, no repairs needed. `make qa-okf-frontmatter` and `make qa-docs`
   both pass clean.

Every task above ran through the RRI 0-25 Low-band local review chain
(`muse-glimmer:30b-q4_K_M` phase 1 and phase 2, both `PASS` with 0
findings on every task) before being applied.
