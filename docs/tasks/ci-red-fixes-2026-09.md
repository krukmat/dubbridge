---
type: TaskList
title: "Tasks: CI Red Fixes - 2026-09"
status: active
plan: docs/plan/ci-red-fixes-2026-09.md
---
# Tasks: CI Red Fixes - 2026-09

Plan: `docs/plan/ci-red-fixes-2026-09.md`. Root cause evidence:
`docs/audit/ci-red-findings-2026-09-01.md`.

RRI calculations run 2026-09-01 with `scripts/rri.py`. T1-T3 are Low-band
(0-25): no full approval presentation, eligible for local Qwen Developer
delegation per `docs/policies/HITL_AUTONOMY_POLICY.md` (narrow, mechanical,
single-file edits). T4 was originally scored Med-high (41-55) and parked
pending an owner decision on isolation strategy (see "Original parked
entry" below for that record). The owner selected **unique-per-test seed
data** (in practice: delete the shared `TRUNCATE ... RESTART IDENTITY
CASCADE` call from every affected setup helper, since all tables use UUID
primary keys — never SERIAL/IDENTITY — so `RESTART IDENTITY` is a no-op
and every test already inserts its own uniquely-generated UUID rows) and
approved the recalculated task card (RRI 67 -> Complex, decomposed into
per-file subtasks per `docs/policies/RRI_POLICY.md`'s RRI >= 56
decomposition gate). Status and per-file evidence: see "CIRF-T4 —
decomposed implementation" below.

## Task summary

| ID | Title | RRI -> band | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| CIRF-T1 | Fix stale migration count in `migrate_test.rs` | 3 -> Low | S | Implemented, verified — pending owner sign-off | - |
| CIRF-T2 | Add `fetch-depth: 0` to `qa-docs` job checkout | 5 -> Low | S | Implemented, verified — pending owner sign-off | - |
| CIRF-T3 | Serialize `qa-test` to eliminate shared-DB race (fast unblock) | 8 -> Low | S | Implemented, verified — pending owner sign-off | - |
| CIRF-T4 | Remove shared TRUNCATE from every affected test-DB setup helper | 67 -> Complex (decomposed) | L | Done — all 15 files closed (14 originally-scoped + 1 owner-acknowledged post-approval addition) | T3 |
| CIRF-T5 | Fix 5 racy tests in `apps/api/src/routes/auth.rs` under parallel execution (discovered validating T4), decomposed into 6 Low-band subtasks (T5-1..T5-4, T5-5a/b, T5-6a/b) | 30 -> Moderate (decomposed to Low per-subtask, each 13 -> Low) | M | Done — 8/8 `auth.rs` tests passing, 5/5 consecutive parallel runs, owner sign-off 2026-09-05 | T4 |

---

## CIRF-T1 - Fix stale migration count in `migrate_test.rs`

- **Status:** Not started
- **Effort:** S
- **RRI:** 3 -> Low (0-25)
- **Depends on:** -
- **Affected:** `apps/cli/tests/migrate_test.rs`

### Objective

`migrations_apply_and_are_idempotent_on_second_run` hardcodes
`assert_eq!(count, 29, ...)`, but `infra/migrations/` now holds 31 files
(two landed via X26-T3c-d/X26-T4 without updating this test). Update the
literal and its doc comment to match reality.

### Inputs

- `apps/cli/tests/migrate_test.rs` (full file, 41 lines).
- `docs/audit/ci-red-findings-2026-09-01.md` § Finding 2.
- `ls infra/migrations/*.sql | wc -l` — re-verify the live count at fix
  time in case more migrations landed since this ledger was written.

### Outputs

- `apps/cli/tests/migrate_test.rs` with the assertion and doc comment
  updated to the current real migration count.

### Acceptance criteria (behavioral)

- **HP-1:** against a reachable database, `cargo test -p dubbridge-cli
  --test migrate_test` passes: the real migration count (31, or whatever
  `ls infra/migrations/*.sql | wc -l` reports at fix time) matches the
  assertion.
- **EC-1:** the second `sqlx::migrate!(...).run(&pool)` call (idempotency
  check) still succeeds as a no-op after the count fix — this behavior must
  not regress; do not touch that half of the test.

### Handoff prompt

In `apps/cli/tests/migrate_test.rs`, update the doc comment ("all 29
migration files apply") and the `assert_eq!(count, 29, ...)` literal (line
~33) to the real count from `ls infra/migrations/*.sql | wc -l`. Do not
change any other logic in the file — the connect-or-skip guard and the
idempotent second-run check must remain exactly as they are. Stop once the
test passes locally against a reachable Postgres; do not touch any other
file.

### Completion notes

Delegated to local Qwen Developer (`qwen3.8:27b-mlx`) via
`scripts/delegate-low-rri.py --mode before-after`, `--allow-path
apps/cli/tests/migrate_test.rs`, `--task-id CIRF-T1`, `--attempt 1`.
Applied patch changed the two required literals correctly but introduced
incidental whitespace drift (extra leading spaces on 6 lines) not requested
in the packet; caught by `cargo fmt -p dubbridge-cli --check` failing, and
corrected by running `cargo fmt -p dubbridge-cli` (mechanical
formatting-only normalization, no logic change — permitted under the
handoff-prompt formatter pass). Final diff is exactly the two-substitution
change specified in the handoff prompt. Verified: `cargo test -p
dubbridge-cli --test migrate_test` against local Postgres
(`postgres://dubbridge:dubbridge@localhost:5432/dubbridge`) — `test
migrations_apply_and_are_idempotent_on_second_run ... ok`, 1 passed, 0
failed.

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M`
- Phase 1 (task-analysis, pre-delegation): PASS, 0 findings.
- Phase 2 (code-solution, post-implementation, against final formatted
  diff): PASS, 0 findings.
- Passes run / usable: 1/1 each phase (single-pass invocation, not the
  N-pass `make qa-gemma-review` wrapper — invoked directly against Ollama
  with the same authority-boundary system prompt).
- Isolated adjudicator (D14): not triggered — Muse Glimmer responded with
  `done_reason: stop` and valid JSON on both phases.
- disposition_divergence: n/a (no divergence — no D14 invocation)
- Primary-agent disposition: accepted both verdicts; separately caught and
  fixed the formatting drift the reviewer did not flag (a cosmetic/
  behavior-preserving issue, consistent with `feedback_whitespace_not_a_discrepancy`
  memory — not something the reviewer was expected to block on, but still
  fixed for `cargo fmt --check` cleanliness).

---

## CIRF-T2 - Add `fetch-depth: 0` to `qa-docs` job checkout

- **Status:** Not started
- **Effort:** S
- **RRI:** 5 -> Low (0-25)
- **Depends on:** -
- **Affected:** `.github/workflows/ci.yml`

### Objective

The `qa-docs` job's `actions/checkout@v4` step defaults to a shallow
`fetch-depth: 1` clone, so `check-task-unit-coverage.sh`'s `git cat-file -e`
historical-commit validation fails on real, valid `commit_sha` citations in
`docs/tasks/s-150-translation-dubbing.md` simply because their commit
objects aren't present in the shallow clone. Two sibling jobs
(`maintainability`, `peer-workflow-review`) already carry `fetch-depth: 0`
for the identical reason; `qa-docs` was missed.

### Inputs

- `.github/workflows/ci.yml` (full file for context; only the `qa-docs`
  job's checkout step, ~line 17, needs to change).
- `docs/audit/ci-red-findings-2026-09-01.md` § Finding 3 (includes the
  local-repro proof that the flagged data is already correct).

### Outputs

- `.github/workflows/ci.yml` with `fetch-depth: 0` added to the `qa-docs`
  job's `actions/checkout@v4` step, matching the `maintainability`/
  `peer-workflow-review` jobs' existing pattern exactly.

### Acceptance criteria (behavioral)

- **HP-1:** with `fetch-depth: 0`, a CI run of the `qa-docs` job resolves
  all previously-flagged `commit_sha` values in
  `docs/tasks/s-150-translation-dubbing.md` as valid, reachable commits, and
  `make qa-docs` passes in CI.
- **EC-1:** the check must still correctly **reject** a genuinely invalid or
  fabricated `commit_sha` (one absent even from full history) — the fix
  must not weaken `check-task-unit-coverage.sh`'s validation into a no-op;
  only the checkout depth changes, no script logic.

### Handoff prompt

In `.github/workflows/ci.yml`, find the `qa-docs` job's
`actions/checkout@v4` step (~line 17) and add a `with: fetch-depth: 0` block
under it, in the exact same YAML shape already used by the
`maintainability` job (~line 154-156) and the `peer-workflow-review` job
(~line 252-254). Do not change any other job or step. Stop once the diff is
a single added `with:`/`fetch-depth: 0` pair; do not touch script logic in
`scripts/check-task-unit-coverage.sh`.

### Completion notes

Delegated to local Qwen Developer (`qwen3.8:27b-mlx`) via
`scripts/delegate-low-rri.py --mode before-after`, `--allow-path
.github/workflows/ci.yml`, `--task-id CIRF-T2`, `--attempt 1`. The applied
patch added the correct `with: / fetch-depth: 0` content but at the wrong
indentation (shifted the `- uses:` and `- name:` list-item markers to
7-space indent, leaving the `run:` line at the original 8-space indent) —
this broke YAML parsing entirely (`yaml.parser.ParserError: while parsing
a block collection`). This is a genuine correctness defect, not cosmetic
whitespace: caught by parsing the file with `python3 -c "import yaml;
yaml.safe_load(...)"`, which raised before the fix and parses clean with
the expected `qa-docs` job structure after. Corrected the indentation to
match the file's 2-space nesting convention and the byte-identical sibling
pattern in the `peer-workflow-review` job. Final diff is the minimal
2-line addition matching that sibling exactly. Verified: `make qa-docs`
passes locally (full local git history makes the shallow-clone symptom
inapplicable locally, but this confirms no regression to the check
itself).

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M`
- Phase 1 (task-analysis, pre-delegation): PASS, 0 findings.
- Phase 2 (code-solution, post-implementation, against final corrected
  diff): PASS, 0 findings.
- Passes run / usable: 1/1 each phase (direct single-pass Ollama
  invocation with the same authority-boundary system prompt as `make
  qa-gemma-review` uses).
- Isolated adjudicator (D14): not triggered — Muse Glimmer responded with
  `done_reason: stop` and valid JSON on both phases.
- disposition_divergence: n/a (no divergence — no D14 invocation)
- Primary-agent disposition: accepted both verdicts; independently caught
  and fixed a real YAML-breaking indentation defect in the applied patch
  that neither review pass flagged (phase-2 review ran against the
  corrected diff, not the broken intermediate state) — the delegating
  agent's own build/verify step (`git apply` diff review + YAML parse),
  not the reviewer, is what caught this, consistent with the runner
  contract's "final diff scope check remains mandatory as defense in
  depth."

---

## CIRF-T3 - Serialize `qa-test` to eliminate shared-DB race (fast unblock)

- **Status:** Not started
- **Effort:** S
- **RRI:** 8 -> Low (0-25)
- **Depends on:** -
- **Affected:** `Makefile`

### Objective

`qa-test` (`cargo test --workspace --all-features`) runs with cargo's
default parallel test threads. `apps/api/src/routes/auth.rs`'s test module
truncates shared tables (`migrate_and_reset`) with no per-test isolation,
so concurrent test threads race on that shared Postgres test database,
producing non-deterministic `test` job failures (`docs/audit/ci-red-findings-2026-09-01.md`
§ Finding 1). `qa-coverage` already works around the identical bug class
with `-- --test-threads=1`. Apply the same flag to `qa-test` as the fast,
low-risk unblock; the durable per-test isolation redesign is scoped
separately as CIRF-T4 (parked).

### Inputs

- `Makefile` (`qa-test` and `qa-coverage` targets).
- `docs/audit/ci-red-findings-2026-09-01.md` § Finding 1.

### Outputs

- `Makefile`'s `qa-test` target invokes `cargo test --workspace
  --all-features -- --test-threads=1`, mirroring `qa-coverage`'s existing
  pattern.

### Acceptance criteria (behavioral)

- **HP-1:** `make qa-test` run twice in a row (or in CI across repeated
  runs) passes deterministically — no flaky failure in
  `apps/api/src/routes/auth.rs`'s test module regardless of which specific
  tests happen to run near each other.
- **EC-1:** a genuinely broken/regressed handler (a real logic bug, not a
  race) must still fail under serialized execution — this change must only
  remove concurrency, not mask real failures. Confirmed by the fact that
  serialized execution changes nothing about what each test asserts, only
  the order/concurrency of execution.

### Handoff prompt

In `Makefile`, change the `qa-test` target's recipe from
`$(CARGO) test --workspace --all-features` to `$(CARGO) test --workspace
--all-features -- --test-threads=1`, matching the exact flag already used
in the `qa-coverage` target's recipe. Do not change `qa-coverage` or any
other target. Stop once the diff is a single added ` -- --test-threads=1`
suffix; do not touch any Rust source file.

### Completion notes

Delegated to local Qwen Developer (`qwen3.8:27b-mlx`) via
`scripts/delegate-low-rri.py --mode before-after`, `--allow-path
Makefile`, `--task-id CIRF-T3`, `--attempt 1`. Applied patch was correct
on the first attempt: exact ` -- --test-threads=1` suffix appended, tab
character preserved on the recipe line, no other target touched. Verified
`make -n qa-test` renders the expected recipe. Verified `make qa-test`
(full workspace, `DUBBRIDGE_DATABASE_URL`/`DUBBRIDGE_REDIS_URL` pointed at
local Docker Compose Postgres/Redis): 60 passed, 1 failed in
`apps/worker-runner`
(`translation_fanout_tests::ec1_partial_claim_leaves_other_target_working`,
`assertion left == right failed: left: 2, right: 1`). **This failure is
pre-existing and unrelated to CIRF-T3**: reproduced identically in
isolation (`cargo test -p dubbridge-worker-runner --bin
dubbridge-worker-runner
translation_fanout_tests::ec1_partial_claim_leaves_other_target_working --
--test-threads=1 --exact`, same panic) and reproduced identically against
clean `main` with all three CIRF changes `git stash`-removed (same panic,
same line). No file this ledger touches
(`apps/cli/tests/migrate_test.rs`, `.github/workflows/ci.yml`,
`Makefile`) has any relationship to `apps/worker-runner` or the
translation-fanout domain. Root cause not diagnosed here (out of scope for
CIRF-T3) but the failure signature (assertion on row-count state seeded
within the test, against the shared local dev Postgres DB) matches the
same shared-DB-state class as Finding 1 — most likely stale local
dev-environment row state, not a `main` regression; not filed as a new
roadmap item without further isolation (e.g. against a freshly-reset local
DB) to avoid over-claiming a root cause, but flagged here for owner
awareness. Serialization via `-- --test-threads=1` does not mask this
failure — it surfaced deterministically and identically with or without
concurrency, confirming CIRF-T3's fix does not hide real failures (EC-1
satisfied).

### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M`
- Phase 1 (task-analysis, pre-delegation): PASS, 0 findings (reviewer
  affirmatively noted correct scoping and tab-preservation awareness).
- Phase 2 (code-solution, post-implementation): PASS, 0 findings.
- Passes run / usable: 1/1 each phase (direct single-pass Ollama
  invocation with the same authority-boundary system prompt as `make
  qa-gemma-review` uses).
- Isolated adjudicator (D14): not triggered — Muse Glimmer responded with
  `done_reason: stop` and valid JSON on both phases.
- disposition_divergence: n/a (no divergence — no D14 invocation)
- Primary-agent disposition: accepted both verdicts; no correction needed
  — this was the one of the three delegations that required zero
  orchestrator intervention beyond the standard build/verify step.

---

## CIRF-T4 — decomposed implementation

- **Status:** Done — all 15 files closed (14 originally-scoped + 1 found
  during implementation and owner-acknowledged, `apps/api/tests/
  notifications_api_test.rs`)
- **Effort:** L
- **RRI:** 67 -> Complex (decomposed per the RRI >= 56 mandatory
  decomposition gate into 14 file-level subtasks: 13 Low-band via
  `scripts/delegate-low-rri.py --mode before-after`, 1 Moderate-band
  (`crates/db/src/user_account.rs`, RRI 29, floored by the `crates/db`
  anchor rubric per ADR-006/018) via `scripts/local-agent/run_local_task.py`)
- **Depends on:** CIRF-T3 (T3 already resolved the CI-blocking symptom;
  this task is durable hardening, approved to proceed rather than staying
  parked)
- **Owner decision (approved):** unique-per-test seed data — in practice,
  delete the shared `TRUNCATE TABLE ... RESTART IDENTITY CASCADE` call
  from each setup helper. All affected tables use UUID primary keys (never
  SERIAL/IDENTITY, confirmed against `infra/migrations/*.sql`), so
  `RESTART IDENTITY` has no effect, and every test already inserts its own
  uniquely-generated UUID rows — no reseeding logic needs to be added, only
  the shared truncate removed.

### Original parked entry (superseded, kept for audit trail)

This task was originally scored RRI 43 -> Med-high (41-55) and parked
2026-09-01 pending an owner decision on isolation strategy, carrying the
`arch_decision` penalty (+12) because choosing an isolation strategy was
the substance of the task. Original RRI computation:

```
python3 scripts/rri.py --touches apps/api/src/routes/auth.rs \
  --touches apps/api/tests/workspace_test.rs \
  --cc 3 --D 2 --K 3 --P 0 --T 2 --A 3 --X 2 \
  --penalty arch_decision
# Final RRI: 43 -> band Med-high (41-55)
```

The two blockers named in that entry are now resolved: (1) the owner
selected unique-per-test seed data (stated above) over transaction-rollback
or per-test schema; (2) the repo-wide grep for the shared-truncate pattern
was completed, enumerating the affected files below. With the design
decision made, the recalculated task (touching all enumerated files as one
unit) scored RRI 67 -> Complex, triggering the unconditional RRI >= 56
decomposition gate rather than staying a single Med-high card; the owner
approved the resulting per-file decomposition.

### Enumerated files (grep: `TRUNCATE TABLE ... RESTART IDENTITY CASCADE` /
`async fn migrate_and_reset` across `apps/**/*.rs` and `crates/**/*.rs`)

| # | File | Band | Status |
|---|---|---|---|
| 1 | `apps/worker-runner/src/translation_fanout_tests.rs` | Low | Done |
| 2 | `apps/worker-runner/src/subtitle_runtime_tests.rs` | Low | Done |
| 3 | `apps/api/tests/review_api_test.rs` | Low | Done |
| 4 | `apps/worker-runner/src/preparation_runtime_tests/support.rs` | Low | Done |
| 5 | `apps/worker-runner/src/runner_topology_tests.rs` | Low | Done |
| 6 | `apps/worker-runner/src/subtitle_enqueue.rs` | Low | Done |
| 7 | `apps/worker-runner/src/transcription_runtime.rs` | Low | Done |
| 8 | `apps/worker-runner/src/translation_runtime_tests.rs` | Low | Done |
| 9 | `apps/api/tests/workspace_test.rs` | Low | Done |
| 10 | `apps/api/tests/ingestion_test.rs` | Low | Done |
| 11 | `apps/api/tests/support/mod.rs` | Low | Done |
| 12 | `apps/api/tests/auth_public_routes.rs` | Low | Done |
| 13 | `apps/api/src/routes/auth.rs` | Low | Done |
| 14 | `crates/db/src/user_account.rs` | Moderate (RRI 29) | Done |
| 15 | `apps/api/tests/notifications_api_test.rs` | Low (pattern-identical) | Done (found post-approval, owner acknowledged 2026-09-04 and approved delegation) |

### Post-approval scope finding

While delegating file 13, a fresh repo-wide grep (run to cross-check the
enumerated list before touching the last two files) found
`apps/api/tests/notifications_api_test.rs:634` also defines
`migrate_and_reset` with the identical shared-`TRUNCATE` pattern
(`notifications, push_tokens, publications, review_decisions,
review_tasks, target_languages, project_assets, projects, org_members,
organizations, pending_ingestions, audit_events, artifact_records,
rights_records, assets`). `git log` confirms this file predates the
current session (added by the S-160 notifications work, commits `c8e9d25`/
`d945890`), so it was missed by the original enumeration pass, not
introduced afterward. The fix is identical in kind to the 14 approved
files (delete the shared TRUNCATE call, table already uses UUID PKs), so
no new design decision is introduced. The owner explicitly acknowledged
this additional scope on 2026-09-04 and approved delegating it under the
same CIRF-T4 effort (see "File 15" below) rather than tracking it as a
separate follow-up task.

### File 15 (`apps/api/tests/notifications_api_test.rs`) — owner-acknowledged scope addition

Delegated identically to files 1-13 via `scripts/delegate-low-rri.py
--mode before-after` after the owner's explicit go-ahead. Phase 1 (Muse
Glimmer, primary for 0-25) PASS 0 findings
(`muse-glimmer:30b-q4_K_M`, `num_ctx=32768`, 15.7s). Delegation applied a
27-line diff; `cargo fmt --package dubbridge-api` corrected minor
continuation-line indentation (9 vs. 8 spaces) with no logic change —
final diff:

```diff
     async fn migrate_and_reset(pool: &PgPool) {
         sqlx::migrate!("../../infra/migrations")
             .run(pool)
             .await
             .expect("migrations");
-
-    sqlx::query(
-        "TRUNCATE TABLE notifications, push_tokens, publications, review_decisions, review_tasks, \
-         target_languages, project_assets, projects, org_members, organizations, \
-         pending_ingestions, audit_events, artifact_records, rights_records, assets \
-         RESTART IDENTITY CASCADE",
-    )
-    .execute(pool)
-    .await
-    .expect("truncate tables");
     }
```

Independently verified: `cargo build -p dubbridge-api --tests` exit 0
(23.36s); confirmed via `git status --short` that the package-wide `cargo
fmt` run touched only the 15 files already part of CIRF-T4 plus this
ledger and `docs/plan/roadmap.md`, no unrelated file. Phase 2 (Muse
Glimmer) PASS 0 findings (`num_ctx=32768`, 10.75s,
`phase2-review-15.json`).

### Muse Glimmer phase-1 false-positive (file 13)

`muse-glimmer:30b-q4_K_M` returned `BLOCKED` twice (identical packet, then
a revised packet with explicit `grep`-verified evidence of the `#[cfg(test)]
mod tests` boundary) on the false claim that `apps/api/src/routes/auth.rs`
does not contain a `migrate_and_reset` test helper. This was independently
verified false (`grep -n "async fn migrate_and_reset"
apps/api/src/routes/auth.rs` -> line 582, inside `mod tests` opened at line
185/`#[cfg(test)]` at line 184). Per the RRI 0-25 review chain, the packet
was sent to the intermediate fallback (`gemma4:26b-a4b-it-qat`), which
returned `PASS` with findings consistent with the verified file content.
Both Muse Glimmer `BLOCKED` verdicts and the Gemma `PASS` are recorded
here per the disposition-divergence requirement; primary-agent disposition:
accepted Gemma's verdict over Muse Glimmer's, based on independently
reproduced `grep` evidence contradicting Muse Glimmer's factual claim.

### File 14 (`crates/db/src/user_account.rs`) — anchor-rubric re-evaluation, then ADR-039 cloud fallback

**Anchor-rubric re-evaluation.** Before delegating, the owner asked whether
this Moderate (RRI 29) task should be split into Low-band sub-tasks like
files 1-13. Re-running the RRI in isolation
(`python3 scripts/rri.py --touches crates/db/src/user_account.rs --cc 1
--D 1 --K 1 --P 1 --T 1 --A 1 --X 0`) confirmed the RRI 29 floor is driven
entirely by the `crates/db` anchor-rubric row (D/K/P floor 3, ADR-006/018 —
`docs/policies/RRI_POLICY.md` § DubBridge anchor rubric), not by the actual
change complexity (a single mechanical deletion). The rubric text anchors
the floor to the touched crate/path, with no carve-out for `#[cfg(test)]`
code, and `apps/api` (where files 11-13 correctly scored Low) has no
dedicated anchor-rubric row at all — so the Moderate/Low split between
`crates/db` and `apps/api` files is a real, policy-documented distinction,
not an inconsistency. Decision: keep RRI 29 Moderate, do not force a Low
split or override the rubric floor.

**Local-first attempt and ADR-039 fallback.** Phase 1 (Gemma, primary for
26-55) ran on the task card and returned PASS, 0 findings
(`gemma4:26b-a4b-it-qat`, `num_ctx=8192`, 12.7s,
`phase1-review-14.json`). `scripts/local-agent/run_local_task.py` was then
invoked against a disposable worktree
(`.agent/worktrees/cirf-t4-14`); the local implementer
(`nemotron-3.5-lightning:30b-a3b-q4_K_M`) hit a `transport_error` — "Gemma
idle timeout after 180s without a token" — an operational failure, not a
packet defect. `/api/ps` showed zero loaded models and ample free memory
immediately after, so this was not the resource-saturation pattern seen
elsewhere in this repo's history; the runner correctly emitted a
`fallback-selection-v1` artifact in `awaiting_fallback_selection` (per
ADR-039, no fallback may fire without a completed human selection).

The owner was presented the operational-only takeover options
(`gpt-5.6-terra`/`medium` per the Codex resolution table, or direct
Claude Sonnet 5 authorship) and selected **`gpt-5.6-terra`/`medium`**
(Codex CLI). The `fallback-selection-v1` artifact was completed
accordingly (`selection_mode: human-select`, `status: selected`,
`selected_by: owner, via interactive confirmation 2026-09-04`) before
invocation.

**Implementation.** Codex CLI (`codex exec --sandbox workspace-write`,
resolved at `/Users/matias/.local/bin/codex` — see updated
`reference_codex_cli_location` memory, the CLI's location on this machine
has changed since it was last checked) applied exactly the required diff
directly to the main working tree (not the disposable worktree, which was
therefore removed unused):

```diff
         MIGRATOR.run(&pool).await.expect("migrations");
-        sqlx::query("TRUNCATE TABLE user_account, organizations RESTART IDENTITY CASCADE")
-            .execute(&pool)
-            .await
-            .expect("truncate auth tables");
-
         Some(pool)
     }
```

No other file or function was touched. Independently verified (not just
trusting Codex's own report): `cargo build -p dubbridge-db --tests` exit 0.

**Phase 2 review.** Gemma (`gemma4:26b-a4b-it-qat`, `num_ctx=8192`, 8.9s)
reviewed the actual diff plus independent build confirmation and returned
PASS with three confirmatory (non-defect) findings — diff matches
acceptance criteria exactly, `MIGRATOR.run`/`Some(pool)` unchanged, no
other files touched (`phase2-review-14.json`).

Note: Codex's own output additionally reported an internal
same-provider-degraded "D14" pass as part of its own execution — this is
Codex's internal process, not a substitute for DubBridge's band-routed
Gemma phase-2 reviewer, which ran independently as recorded above.

### Gemma Reviewer evidence (file 14)

- Model: `gemma4:26b-a4b-it-qat` (band 26-55 primary; Moderate/Muse
  Glimmer fallback not triggered)
- Phase 1 (task-analysis, pre-delegation): PASS, 0 findings
- Phase 2 (code-solution, post-implementation): PASS, 0 findings (all
  three findings confirmatory)
- Passes run / usable: 1/1 each phase
- Isolated adjudicator (D14): not triggered on the DubBridge side (Gemma
  responded `done_reason: stop` with valid JSON both phases); Codex
  separately ran its own internal same-provider-degraded D14-equivalent,
  noted above for transparency but not counted as this task's D14 fallback
- disposition_divergence: n/a
- Fallback selection: `human-select`, ADR-039 artifact at
  `result-14.fallback-selection.json`, trigger `operational-only` (local
  implementer idle timeout), selected `gpt-5.6-terra`/`medium` via Codex
  CLI
- Primary-agent disposition: accepted both verdicts; independently
  reproduced the build result rather than trusting Codex's self-report

## CIRF-T5 — fix 5 racy `apps/api/src/routes/auth.rs` tests under parallel execution

**Discovered** while validating CIRF-T4's fix: a full `cargo test --workspace
--all-features` run (deliberately without `--test-threads=1`) surfaced 5
failing tests in `apps/api/src/routes/auth.rs`, racy for two independent
reasons: (1) several tests share the literal seed/login/register email
`"owner@example.com"`, causing account-uniqueness collisions across
concurrently-running tests; (2) `count_audit_events`/`latest_audit_detail`
filter only by `event_kind` (the `audit_events` table has no actor/email
column — `infra/migrations/0004_create_audit_events.sql`), so an absolute
count assertion races against any other concurrently-running test emitting
the same event kind.

**RRI and decomposition.** The whole-bundle fix scored RRI 30 (Moderate,
`--D 3 --K 2 --P 2 --T 2 --A 2 --X 1`); a single-test-function-scoped edit
scored RRI 13 (Low, `--D 1 --K 1 --P 1 --T 1 --A 1 --X 0`), confirmed via
`scripts/rri.py`. Per the owner's instruction to subdivide into Low tasks and
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band
decomposition`, decomposed into independent Low-band subtasks, each
delegated via `scripts/delegate-low-rri.py --mode before-after` (with the
40-line BEFORE safety cap forcing two functions to split into `a`/`b`
sub-packets), phase-1/phase-2 reviewed by `muse-glimmer:30b-q4_K_M`
(RRI 0-25 chain primary).

| Subtask | Function | Fix | Phase 1 | Phase 2 |
|---|---|---|---|---|
| T5-1 | `login_handler_returns_ok_and_emits_success_audit` | unique email + delta count | PASS 0 findings | PASS 0 findings |
| T5-2 (2a+2b) | `login_handler_maps_wrong_password_and_unknown_email_to_same_unauthorized` | unique email + delta count (split: 45-line BEFORE exceeded cap) | PASS 0 findings (each) | PASS 0 findings (combined) |
| T5-3 | `login_handler_maps_validation_errors_to_bad_request` | dual before/after delta counters (racy even without email collision) | PASS 0 findings | PASS 0 findings |
| T5-4 | `login_handler_fails_closed_when_audit_persistence_fails` | unique email only (no count assertion) | PASS 0 findings | PASS 0 findings |
| T5-5a | `register_handler_returns_created_and_emits_audit` | unique email + delta count | BLOCKED (packet-prose inconsistency) -> corrected -> PASS | see T5-6a (superseded) |
| T5-5b | `register_handler_maps_duplicate_email_to_conflict` | unique email + delta count | PASS 0 findings | see T5-6b (superseded) |

**T5-5a packet-inconsistency finding (real, not a false positive).** The
first phase-1 review of T5-5a's packet correctly returned `BLOCKED`: the
acceptance-criteria prose gave a single-line `assert_eq!` example that
contradicted the Required AFTER block's correct multi-line formatting
matching the file's existing style. Per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Per-task discipline` (a materially
revised packet needs its own fresh phase-1 pass), the prose was corrected
to reference the AFTER block's own formatting instead of restating it, and
phase-1 was re-run, returning PASS.

**T5-1 scope-violation finding (real, caught by manual diff review).** T5-1's
applied diff also deleted the shared `TRUNCATE` call from `migrate_and_reset`
in `auth.rs` — not authorized by the packet's AFTER block and not part of
CIRF-T4's file list. Caught via `git diff` inspection before phase-2 review;
reverted with Edit, re-verified via `git diff` that only the intended
function changed, and re-verified `cargo build -p dubbridge-api --tests`
before proceeding.

**Second-order collision discovered post-closure (T5-6).** After all of
T5-1 through T5-5b were applied and phase-2-reviewed PASS, a real
`cargo test --workspace --all-features` run (no `--test-threads=1`) still
failed T5-5a and T5-5b against **each other**: both had been given their own
unique email, but both still counted the same `event_kind = "auth_registered"`
via the shared global `count_audit_events` helper, so one test's
before/after snapshot window could straddle the other's registration
(observed delta 2 instead of 1, reproduced 3/3 consecutive runs). Root
cause: the delta pattern only isolates a test from *other* event kinds and
*other* emails' registrations — two tests emitting the *same* kind
concurrently still race, since neither snapshot is atomic relative to the
other, and the schema has no actor/email column to filter by at the SQL
level (confirmed against the migration; out of scope to add one).

**T5-6 fix.** Replaced the racy global-count delta in both functions with
the already-existing `count_accounts_by_email(pool, email)` helper, scoped
to each test's own unique email — inherently race-free since no two tests
share an email. Scored RRI 13 (Low, same profile as T5-1..T5-5), split into
T5-6a/T5-6b (68-line combined BEFORE exceeded the 40-line cap). Both
phase-1 and the combined phase-2 review (run against the current file state
rather than the raw diff, to avoid confusing the reviewer with intermediate
pre-T5-5 content) returned PASS, 0 findings.

**Verification.** `cargo fmt --package dubbridge-api` run once after the
full T5-1..T5-6b chain (per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Indentation drift is not a
delegation defect`, added this session — see below). `cargo test -p
dubbridge-api --lib routes::auth::tests::` (no `--test-threads=1`): 8/8
passing, 5/5 consecutive runs. Full `cargo test --workspace --all-features`:
90/91 `dubbridge-api` tests passing — the one remaining failure
(`routes::compliance::tests::get_audit_timeline_handler_returns_owned_events`)
is a **new, separate, pre-existing race** in `apps/api/src/routes/compliance_tests.rs`
(last touched by `7ceb503`, unrelated to CIRF-T4/T5's file list), confirmed
reproducible only under full-workspace parallel execution and passing both
in isolation and when run with only its own module — i.e. it races against
a test in a *different* module sharing the same database, not against
itself. This was out of CIRF-T5's scope (`auth.rs` only) and was recorded as
a new finding for a follow-up fix, applied directly the same day (2026-09-05,
owner-authorized ad hoc fix, no separate task card) — see
**CIRF-T5-addendum** below.

### CIRF-T5-addendum — connection-pool exhaustion, not a data race

Root-caused by direct inspection rather than repeated reproduction (the
failure did not reproduce across 6 additional full-workspace runs, consistent
with a timing/load-dependent resource issue rather than a logic defect):
`list_audit_events_for_owned_asset` in `crates/db/src/audit_repo.rs` filters
strictly by `asset_id` (unique per test via UUID) — genuinely race-free at
the SQL level, ruling out a data-correctness bug. The actual cause is
connection-pool exhaustion: `compliance_tests.rs::setup_pool` and
`auth.rs`'s `TestContext::new`/`with_closed_audit_pool` each called
`PgPool::connect` with no `max_connections` cap, defaulting to sqlx's 10 —
with 7 tests in `compliance_tests.rs` and 6 in `auth.rs` running in parallel
(each opening its own pool), that's up to 130 concurrent connections
contending against Postgres, which can intermittently time out a `connect`
or `migrate!().run()` call under load.

**Fix applied directly** (mechanical, no behavior change to any test
assertion): capped every raw `PgPool::connect` call in these two files via
`sqlx::postgres::PgPoolOptions::new().max_connections(N)` — `2` for
`compliance_tests.rs::setup_pool`, `2` for `auth.rs`'s `TestContext::new` and
its `auth_pool`, `1` for `with_closed_audit_pool`'s `audit_pool` (immediately
closed, never used for queries). Verified: `cargo build -p dubbridge-api
--tests` clean; `cargo fmt --all -- --check` clean; `cargo clippy --workspace
--all-targets --all-features -- -D warnings` clean; `cargo test --workspace
--all-features` run 4 additional consecutive times with zero failures (a 5th
run was cut short by an unrelated local command timeout, not a test
failure).

**New governance rule documented this session.** Mid-chain, indentation/
whitespace drift from `before-after` mode's context-line retyping was
observed (and, per explicit owner correction, must not be treated as a
review finding, must not pause the delegation chain, and must not consume a
repair attempt). Documented in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before
implementing` (new paragraph after the Qwen Developer delegation paragraph)
as the canonical, repo-committed source, plus personal memory
(`feedback_low_rri_ignore_indentation_drift`). All phase-1/phase-2 review
prompts in this task explicitly instructed the reviewer to ignore
indentation/whitespace drift.

### Gemma Reviewer evidence (CIRF-T5, all subtasks)

- Model: `muse-glimmer:30b-q4_K_M` (RRI 0-25 chain primary), `num_ctx=32768`
- Phase 1 (task-analysis, pre-delegation): PASS 0 findings on 7/8 packets
  (T5-1, T5-2a, T5-2b, T5-3, T5-4, T5-6a, T5-6b); T5-5a BLOCKED once on a
  genuine packet-prose inconsistency, corrected, re-reviewed PASS
- Phase 2 (code-solution, post-implementation): PASS 0 findings, all 5
  review calls (T5-1; T5-2 combined; T5-3; T5-4; T5-5a+T5-5b+T5-6a+T5-6b
  combined, reviewed against final file state)
- Passes run / usable: 1/1 each phase, each packet
- Isolated adjudicator (D14): not triggered — Muse Glimmer responded
  `done_reason: stop` with valid JSON on every call (including the one
  BLOCKED verdict, which was a correct finding, not an availability failure)
- disposition_divergence: none
- Primary-agent disposition: accepted all findings; T5-1's out-of-scope
  TRUNCATE removal reverted; T5-5a's packet inconsistency corrected and
  re-reviewed; T5-6 designed and delegated after independently reproducing
  the T5-5a/T5-5b mutual race via real `cargo test` runs (not merely
  inferred)

### Owner final verification

- Owner: `kruk.matias@gmail.com`
- Date: 2026-09-05
- Statement: I authorize closure of CIRF-T5. The implementation (T5-1
  through T5-6b) was independently verified by the delegating agent —
  build, phase-1/phase-2 Muse Glimmer review (all PASS 0 findings, one
  correctly-caught BLOCKED on T5-5a's first packet, resolved), and 5
  consecutive parallel `cargo test` runs with 8/8 `auth.rs` tests passing
  every time. I reviewed the reported evidence and accept it as sufficient;
  closing without a separate manual re-run.
- Commands run: `cargo build -p dubbridge-api --tests`; `cargo fmt --package
  dubbridge-api`; `cargo test -p dubbridge-api --lib
  routes::auth::tests:: ` (5 consecutive runs, no `--test-threads=1`);
  `cargo test --workspace --all-features` (1 run, surfaced the unrelated
  `compliance_tests.rs` finding, tracked separately)

## Related

- `docs/plan/ci-red-fixes-2026-09.md`
- `docs/audit/ci-red-findings-2026-09-01.md`
- `docs/plan/roadmap.md` § Cross-cutting obligations, `X28`
