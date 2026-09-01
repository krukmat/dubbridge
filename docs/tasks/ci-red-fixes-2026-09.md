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
single-file edits). T4 scored Med-high (41-55) and is **parked, not
implemented under this ledger** — see its entry for why decomposition isn't
sound yet.

## Task summary

| ID | Title | RRI -> band | Effort | Status | Depends on |
|---|---|---:|---|---|---|
| CIRF-T1 | Fix stale migration count in `migrate_test.rs` | 3 -> Low | S | Implemented, verified — pending owner sign-off | - |
| CIRF-T2 | Add `fetch-depth: 0` to `qa-docs` job checkout | 5 -> Low | S | Implemented, verified — pending owner sign-off | - |
| CIRF-T3 | Serialize `qa-test` to eliminate shared-DB race (fast unblock) | 8 -> Low | S | Implemented, verified — pending owner sign-off | - |
| CIRF-T4 | (Parked) Durable per-test DB isolation redesign | 43 -> Med-high | L | Parked | T3 |

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

## CIRF-T4 - (Parked) Durable per-test DB isolation redesign

- **Status:** Parked — not implemented under this ledger
- **Effort:** L
- **RRI:** 43 -> Med-high (41-55)
- **Depends on:** CIRF-T3 (only in the sense that T3 already resolves the
  CI-blocking symptom, so T4 is optional hardening, not a blocker for
  anything)
- **Affected (if pursued):** `apps/api/src/routes/auth.rs`,
  `apps/api/tests/workspace_test.rs`, and potentially other test files
  sharing the same truncate-and-reseed pattern (not yet enumerated)

### Why this is not decomposed into Low subtasks

Per `docs/policies/RRI_POLICY.md`'s anchor rubric, this task carries the
`arch_decision` penalty (+12): the actual fix requires **choosing an
isolation strategy** (per-test transaction with rollback, unique
per-test seed data instead of shared truncate, or per-test schema) before
any file can be edited. That choice is the substance of the task, not
boilerplate around it — decomposing an unmade design decision into
local-dev-executable Low-band subtasks would just relocate the same
decision into whichever subtask happens to run first, not actually reduce
its reasoning burden. Per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Post-repair-budget Low-band
decomposition, decomposition is a routing mechanism for **already-scoped**
implementation work, not a substitute for a pending design decision.

RRI computation (recorded for audit):

```
python3 scripts/rri.py --touches apps/api/src/routes/auth.rs \
  --touches apps/api/tests/workspace_test.rs \
  --cc 3 --D 2 --K 3 --P 0 --T 2 --A 3 --X 2 \
  --penalty arch_decision
# Final RRI: 43 -> band Med-high (41-55)
```

### What would unblock decomposition

1. Owner or primary-agent decision on isolation strategy (recommend:
   unique-per-test seed data — email/workspace name derived from a
   per-test UUID or the test's own name — over transaction-rollback, since
   several of these tests assert on committed rows visible to a second
   connection/handler call, which a wrapping transaction would complicate).
2. A repo-wide grep for the same `TRUNCATE ... RESTART IDENTITY CASCADE`
   shared-truncate pattern across `apps/api/tests/*.rs` and
   `apps/**/src/**/tests` modules, to scope how many files actually need
   the fix (not yet done — out of scope for this ledger).
3. Once (1) and (2) are done, the actual per-file edits are very likely
   each individually Low-band (single test-module edit, no cross-file
   coupling) and decomposable via `scripts/delegate-low-rri.py` at that
   point.

### Recommendation

Do not implement now. CIRF-T3 already removes the CI-blocking symptom.
Revisit this as a separate, explicitly-scoped follow-up task once an owner
decision on isolation strategy is available — present as its own Med-high
card (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Med-high Architect-refined
single-attempt gate) rather than folding it into this Low-band ledger.

## Related

- `docs/plan/ci-red-fixes-2026-09.md`
- `docs/audit/ci-red-findings-2026-09-01.md`
- `docs/plan/roadmap.md` § Cross-cutting obligations, `X28`
