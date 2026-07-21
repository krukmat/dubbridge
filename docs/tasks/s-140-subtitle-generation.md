---
type: TaskList
title: "S-140 Subtitle Generation"
status: proposed
slice: S-140
plan: docs/plan/s-140-subtitle-generation.md
Behavioral coverage contract: unit-v1
---
# S-140 Subtitle Generation

> **Status:** Proposed 2026-07-20. Authored via ADR-037 T5 (Local Architect
> advisory verified against repository evidence — see
> `docs/evaluations/adr037-direct-project-report.md` reconciliation section).
> T0 ratified Design decisions D1 (segmentation source → D1a) and D2 (subtitle
> schema → the proposed internal JSON schema) on 2026-07-21 — see
> `docs/plan/s-140-subtitle-generation.md`. No task below has started
> execution. Each development task still requires its own RRI computation and
> presentation/approval — this ledger does not inherit ADR-037 T5's approval.
> Owner coordination-mode adjustment 2026-07-21: S-140 implementation must not
> start from the former broad provisional task cards. The primary agent's role
> is limited to Med-high/Complex coordination for this goal: recompute/present
> RRI, route peer/D14 review, decide whether to decompose again, assemble any
> allowed handoff packet, enforce scope, and close status artifacts. Low and
> Moderate cards are local-owned under their normal local routes. High+ cards
> must be decomposed or escalated under the RRI policy before execution.
> **Plan:** `docs/plan/s-140-subtitle-generation.md`.
> **Behavioral coverage contract:** unit-v1.

## Coordination-mode planning note (2026-07-21)

This ledger is currently a planning ledger. It is not yet a local-handoff ledger.
Before preparing any implementation handoff packet:

1. ~~Ratify D1/D2 in `docs/plan/s-140-subtitle-generation.md`.~~ Done via T0,
   2026-07-21 (D1a; proposed D2 schema).
2. Recompute RRI with `scripts/rri.py` for the exact affected paths.
3. Split any broad task that scores above Moderate when local implementation is
   expected.
4. Fill every development task with final HP/EC examples, acceptance criteria,
   evidence to emit, status artifacts affected, allowed paths, verification
   commands, and a stop condition.
5. Preserve the band boundary: Low/Moderate cards are local-owned; Codex
   coordination appears only for Med-high/Complex cards; High+ cards are
   decomposed or escalated before execution.

Preliminary RRI checks on 2026-07-21 show why recalibration is required:

| Provisional task | Conservative rerun | Planning consequence |
|---|---:|---|
| T1 broad domain + migration + repository bundle | RRI 68, Complex | Split before handoff; migration raises D/K/P under the DubBridge rubric. |
| T2 job contract + enqueue | RRI 40, Moderate | Local-first candidate after approval and final HP/EC criteria. |
| T3 segmentation + runner + readiness | RRI 55, Med-high | Codex-coordinated; split or route through normal RRI 41+ gates before any local execution. |
| T5 review enqueue plus possible artifact-identity schema work | RRI 69, Complex | Decide schema scope first; split migration from enqueue wiring. |

The execution-card backlog below is the current local-control plan. Do not use
the former broad T1/T2/T3/T5 cards as implementation handoffs.

## S-140-T0: Ratify D1/D2 and freeze local-handoff sequence

**Effort:** S (planning RRI 2 — Low; docs/task-ledger-only, exempt from
Gemma Reviewer/D14 per `CLAUDE.md`)
**Depends on:** S-130 (closed)
**Status:** Done — planning-only; no product code changes

**Objective:** Convert the two open S-140 design decisions into an execution
contract before any local implementation role receives a handoff.

**Acceptance criteria:**
- [x] D1 is ratified as either D1a Rust `WordAlignment` segmentation or D1b
  Python subtitle worker. — **Ratified D1a**, owner decision 2026-07-21.
- [x] D2 is ratified as the canonical stored subtitle schema. — **Ratified**
  the proposed internal JSON schema (`subtitle.json`), owner decision
  2026-07-21.
- [x] The task sequence below is updated if D1b or a review-task
  artifact-identity schema change changes the implementation graph. — T4
  (D1b Python worker) removed from this ledger; no artifact-identity schema
  change was in scope for T0.
- [x] The Low/Moderate local-ownership boundary and Med-high/Complex Codex
  coordination boundary are preserved.

**Files expected to change:**
- `docs/plan/s-140-subtitle-generation.md`
- `docs/tasks/s-140-subtitle-generation.md`

**Evidence to emit:** RRI output, D1/D2 decision note, updated task sequence.

**Evidence emitted:**
- RRI: `python3 scripts/rri.py --C 0 --T 0 --A 0 --X 0 --D 0 --K 0 --P 0
  --touches docs/plan/s-140-subtitle-generation.md --touches
  docs/tasks/s-140-subtitle-generation.md --platform dubbridge` → Final RRI 2,
  Low.
- D1/D2 decision note: owner ratified D1a and the proposed D2 schema on
  2026-07-21 (see `docs/plan/s-140-subtitle-generation.md`, D1/D2 sections).
- Updated task sequence: T4 removed below; T3a/T3b dependency wording
  simplified to D1a-only.

**Status artifacts affected:** This ledger and the S-140 plan.

**Stop condition:** Stop after the plan/ledger reflect ratified D1/D2. Do not
start T1a implementation.

**Agent handoff prompt:** Planning-only. Ratify D1/D2, update the S-140 plan and
task ledger, emit the RRI/decision evidence, then stop before implementation.

**Status: [x] Done 2026-07-21 — D1a and D2 ratified; T4 removed; T1a unblocked**

---

## S-140-T1a: Domain subtitle kind/status types

**Effort:** S (RRI 21 — Low, recomputed at presentation time)
**Depends on:** S-140-T0
**Status:** Done — domain types added; T1b unblocked

**Happy paths considered:**
- HP-1: `ArtifactKind::Subtitle` round-trips through display/parse helpers.
- HP-2: `SubtitleStatus` round-trips all valid states: Pending, InProgress,
  Ready, Failed.

**Edge cases considered:**
- EC-1: Unknown artifact kind text still fails closed as `UnknownStoredValue`.
- EC-2: Unknown subtitle status text fails closed as `UnknownStoredValue`.

**Inputs:** `crates/domain/src/artifact.rs` and the existing
`TranscriptionStatus` pattern.

**Outputs:** `ArtifactKind::Subtitle`, `SubtitleStatus`, and
`SubtitleStatusRecord` domain types.

**Acceptance criteria:**
- Subtitle kind and statuses use the existing domain parsing/display style.
- Unknown stored values fail closed with the existing error shape.
- All HP/EC cases above have unit coverage.

**Files expected to change:**
- `crates/domain/src/artifact.rs`

**Evidence to emit:** RRI output, unit test names, local implementation artifact
if delegated.

**Evidence emitted:**
- RRI: `python3 scripts/rri.py --C 1 --F 0 --D 2 --T 1 --A 0 --K 2 --P 2 --X 0
  --touches crates/domain/src/artifact.rs --platform rust` → Final RRI 21,
  Low.
- Delegated to local Gemma (`gemma4:26b-a4b-it-qat`) via
  `scripts/delegate-low-rri.py`, `--allow-path crates/domain/src/artifact.rs`.
  Attempt 1 introduced an unrelated syntax defect (corrupted an existing line
  in `derived_artifact_new_sets_fields`); caught in orchestrator review before
  apply, not applied. Attempt 2 (bounded repair cycle) fixed it cleanly; diff
  validated with `git apply --check` and applied.
- Unit tests added and passing (`cargo test -p dubbridge-domain --lib
  artifact`, 18/18 passed): `parse_subtitle` (HP-1),
  `subtitle_status_display_all_variants` (HP-2). EC-1/EC-2 (unknown-value
  fail-closed via `UnknownStoredValue`) are DB-layer (`crates/db`) concerns
  out of this file's scope by the stop condition below; deferred to T1b,
  which already extends the DB-layer `parse_kind`/check constraints.
- `cargo fmt -p dubbridge-domain -- --check` clean; `cargo clippy -p
  dubbridge-domain --all-features -- -D warnings` clean.
- Gemma Reviewer phase-2 diff review (`scripts/gemma-code-review.py`, 3
  passes): `status: findings`, 1 minor non-blocking finding (confirmed
  `snake_case` serde convention is correct, no action needed), 3/3 pass
  consensus.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after domain tests pass. Do not create migrations or DB
repository code.

**Agent handoff prompt:** Add only the subtitle domain kind/status types in
`crates/domain/src/artifact.rs`, cover HP/EC with unit tests, and stop before any
migration or repository work.

**Status: [x] Done 2026-07-21 — Subtitle domain types added via Low-RRI local
delegation (Gemma), Gemma Reviewer phase-2 passed with 1 non-blocking minor
finding; T1b unblocked**

---

## S-140-T1b-i: Subtitle status table migration

**Effort:** L (planning RRI 52 — Med-high; recompute at presentation time)
**Depends on:** S-140-T1a
**Status:** Done — migration merged; T1c unblocked

> Split from the original T1b ("Subtitle status migration and artifact-kind
> check extension") into T1b-i and [[S-140-T1b-ii]]. Both halves touch
> `infra/migrations/**`, whose anchor-rubric floor (D=4, P=5, K=4;
> ADR-008/ADR-018) plus the automatic `auth_security` penalty puts every
> migration-path task at RRI 41+ regardless of diff size — confirmed via
> `scripts/rri.py` on the minimal-possible check-only change (still 45). The
> split does not lower either task's band; it narrows each task's own change
> surface and lets the simpler half (T1b-ii) be approved and merged
> independently.
>
> **Implementation route:** Med-high (41–55) now routes local-first per the
> 2026-07-21 owner override to `docs/policies/RRI_POLICY.md` §Med-high
> local-first handling — `scripts/local-agent/run_local_task.py` +
> `DUBBRIDGE_LOCAL_AGENT_MODEL` (default `qwen3.6:35b-a3b`), 1 repair attempt
> max before escalating to cloud. Cross-vendor peer review (phases 1 and 2)
> and 3 Reflection passes still apply unchanged; only the code-authoring
> surface moved local.

**Happy paths considered:**
- HP-1: Migration creates `asset_subtitle_status` with one row per asset and
  status/error/update fields mirroring `asset_transcription_status`.

**Edge cases considered:**
- EC-1: Duplicate subtitle status rows for one asset are rejected by the primary
  key.
- EC-2: Invalid subtitle status text is rejected by the status check.

**Inputs:** Current migration for `asset_transcription_status`.

**Outputs:** New subtitle status table migration.

**Acceptance criteria:**
- Migration is reversible or follows the repository's forward-only migration
  convention if no down migrations exist.
- Table shape mirrors the transcription status table unless a documented reason
  is added.

**Files expected to change:**
- `infra/migrations/00XX_create_subtitle_status.sql` (new; exact number chosen
  at task time)

**Evidence to emit:** RRI output, migration test/check command output,
cross-vendor/D14 phase-1 artifact if required.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after migration validation. Do not implement repository
methods. Do not touch the artifact-kind check (that is [[S-140-T1b-ii]]).

**Agent handoff prompt:** Add only the subtitle status table migration,
validate it, and stop before repository code or the artifact-kind check.

### Execution record (2026-07-21)

- **RRI at implementation time:** 52 (Med-high) — recomputed via
  `scripts/rri.py --C 0 --T 3 --A 0 --X 1 --D 4 --K 4 --P 5 --touches infra/migrations/0024_create_subtitle_status.sql --penalty auth_security --platform dubbridge`;
  anchor-rubric floor (D=4, K=4, P=5; ADR-008/ADR-018) plus `auth_security`
  +10 penalty, base 42 + 10 = 52.
- **Implementation route:** local-first per Med-high routing —
  `scripts/local-agent/run_local_task.py`, implementer `qwen3.6:35b-a3b`
  (`DUBBRIDGE_LOCAL_AGENT_MODEL` default), disposable git worktree
  (`local/s-140-t1b-i` branch). An initial harness attempt exposed an
  environment issue (`createdb`/`psql` absent on the host), so verification
  was moved into the existing `local-postgres-1` PostgreSQL 16 container and
  the local run was repeated against the same worktree. **1 repair attempt
  used** in the successful rerun (the first test failure required tightening
  the migration comment so it explicitly states that `0022` had no status
  constraint).
- **Verification:** custom `test_runner` applied all 24 migrations in order
  (0001–0024) against a fresh PostgreSQL 16 database inside
  `local-postgres-1` and confirmed via `information_schema.columns` and
  `pg_get_constraintdef` that `asset_subtitle_status` contains the expected
  columns/defaults, `PRIMARY KEY (asset_id)`, `FOREIGN KEY (asset_id)
  REFERENCES assets(id)`, and a closed `subtitle_status_check` over
  `pending`, `in_progress`, `ready`, `failed`. Scope check: in-scope, no
  boundary violations, no files touched outside `allowed_paths`.
- **Code-solution review (phase 2):** `qwen3.6:27b-q4_K_M` via Ollama
  (`http://localhost:11434`), per the 2026-07-21 owner directive replacing
  the cross-vendor peer as the default Med-high reviewer (see
  `docs/policies/RRI_POLICY.md` §Local pipeline phase-2 reviewer override).
  Phase-1 task-analysis review: **n/a** (migration-only task, exempt per
  policy). Artifact: `.agent/peer-code-review-S-140-T1b-i-qwen.json` with
  packet `.agent/peer-review-packet-S-140-T1b-i.md`. Verdict: **PASS**, no
  findings — "Migration satisfies all acceptance criteria with correct
  schema, constraints, and forward-only scope." Gemma fallback: not
  triggered. D14 fallback: not triggered. `disposition_divergence: none`.

### Reflection log

Required passes: 3 (`RRI 52` → `Med-high`)

#### Pass 1
- **Draft verdict:** Local-model-authored migration created
  `asset_subtitle_status` with the correct table shape and a status check over
  the four `SubtitleStatus` literals.
- **Critique findings:** The explanatory SQL comment justified the deviation
  imprecisely; the verifier required it to state explicitly that
  `0022_create_transcription.sql` had no status constraint.
- **Revisions applied:** Tightened the migration comment to name the exact
  precedent and exact reason for the deviation.

#### Pass 2
- **Draft verdict:** Revised migration comment matched the task's documented
  deviation requirement.
- **Critique findings:** Verified migration numbering and scope discipline:
  `0024` is the next available migration number after committed `0023`, and
  the diff remained limited to `infra/migrations/0024_create_subtitle_status.sql`.
- **Revisions applied:** none.

#### Pass 3
- **Draft verdict:** Final migration ready for acceptance.
- **Critique findings:** Re-checked the status literals against
  `crates/domain/src/artifact.rs` and confirmed the SQL check uses the exact
  wire forms `pending`, `in_progress`, `ready`, `failed`; re-checked that the
  table still mirrors `asset_transcription_status` except for the documented
  status constraint required by EC-2.
- **Revisions applied:** none.

### Unit coverage certification

Not applicable — this is a migration-only change (no Rust source touched,
no new code path to cover). `crates/domain/src/artifact.rs` test coverage
for `SubtitleStatus` display literals was already certified under S-140-T1a.

### Owner verification

Pending — reported to owner below; not yet independently re-verified by a
human.

**Status: [x] Done — 2026-07-21, migration merged to `infra/migrations/0024_create_subtitle_status.sql`; T1c unblocked**

---

## S-140-T1b-ii: Artifact-kind check extension for subtitle

**Effort:** L (recomputed RRI 55 — Med-high, at implementation time 2026-07-21)
**Depends on:** S-140-T1a
**Status:** Done — migration merged; T1b-i still required before T1c

> Split from the original T1b; see [[S-140-T1b-i]] for the rationale. This
> half only extends an existing check constraint — no new table — but still
> touches `infra/migrations/**`, so it carries the same anchor-rubric floor
> and band. Independent of T1b-i; either may be implemented first, but both
> must land before [[S-140-T1c]] starts.
>
> **Implementation route:** same Med-high local-first routing as [[S-140-T1b-i]]
> — `scripts/local-agent/run_local_task.py` + `DUBBRIDGE_LOCAL_AGENT_MODEL`
> (default `qwen3.6:35b-a3b`), 1 repair attempt max; cross-vendor peer review
> and 3 Reflection passes unchanged.

**Happy paths considered:**
- HP-1: Migration accepts `Subtitle` in the artifact kind check after T1a
  introduces the domain kind.

**Edge cases considered:**
- EC-1: Artifact kind check still rejects values outside the known kind set
  (no unrelated kind added alongside `subtitle`).

**Inputs:** Current artifact-kind check migration.

**Outputs:** New migration extending the artifact-kind check.

**Acceptance criteria:**
- Migration is reversible or follows the repository's forward-only migration
  convention if no down migrations exist.
- Artifact kind check includes the new `Subtitle` kind and no unrelated kind.

**Files expected to change:**
- `infra/migrations/00XX_extend_artifact_kind_check.sql` (new; exact number
  chosen at task time)

**Evidence to emit:** RRI output, migration test/check command output,
cross-vendor/D14 phase-1 artifact if required.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after migration validation. Do not implement
repository methods. Do not touch the subtitle status table (that is
[[S-140-T1b-i]]).

**Agent handoff prompt:** Add only the artifact-kind check extension for
`subtitle`, validate it, and stop before repository code or the status table
migration.

### Execution record (2026-07-21)

- **RRI at implementation time:** 55 (Med-high) — recomputed via
  `scripts/rri.py` with `--touches infra/migrations/0023_extend_artifact_kind_check_subtitle.sql`;
  anchor-rubric floor (D=4, K=4, P=5; ADR-008/ADR-018) plus `auth_security`
  +10 penalty, base 45 + 10 = 55.
- **Implementation route:** local-first per Med-high routing —
  `scripts/local-agent/run_local_task.py`, implementer `qwen3.6:35b-a3b`
  (`DUBBRIDGE_LOCAL_AGENT_MODEL` default), disposable git worktree
  (`local/s-140-t1b-ii` branch). **0 repair attempts used** (succeeded on
  first draft; 1-attempt Med-high budget was not exhausted).
- **Verification:** custom `test_runner` applied all 23 migrations in order
  (0001–0023) against a fresh PostgreSQL 16 instance (`local-postgres-1`
  container) and confirmed via `pg_get_constraintdef` that the resulting
  `artifact_kind_check` constraint contains all 6 pre-existing kinds plus
  `subtitle`, in order, with no other kind added. Scope check: in-scope, no
  boundary violations, no files touched outside `allowed_paths`.
- **Code-solution review (phase 2):** `qwen3.6:27b-q4_K_M` via Ollama
  (`http://localhost:11434`), per the 2026-07-21 owner directive replacing
  the cross-vendor peer as the default Med-high reviewer (see
  `docs/policies/RRI_POLICY.md` §Local pipeline phase-2 reviewer override).
  Phase-1 task-analysis review: **n/a** (migration-only task, exempt per
  policy). Verdict: **PASS**, no findings — confirmed SQL correctness/lock
  behavior matches precedent 0020/0022, no structural deviation, all
  acceptance criteria satisfied. `disposition_divergence: none`.

  **Correction (2026-07-21):** this review was originally run against a D14
  context-isolated subagent after `which codex`/`codex --version` reported
  no binary. That conclusion was wrong — `codex` is installed but not on
  `$PATH` (it ships inside the OpenAI ChatGPT VS Code extension bundle,
  e.g. `~/.vscode/extensions/openai.chatgpt-*/bin/macos-aarch64/codex`);
  resolving the binary directly confirms it works
  (`codex login status` → `Logged in using ChatGPT`). Separately, the owner
  clarified the same day that Med-high phase-2 review should route to
  `qwen3.6:27b-q4_K_M`, not the cross-vendor peer, once the local pipeline
  is in play — see `[[feedback_local_pipeline_roles]]` (memory) and the
  policy sections cited above. The review was re-run for real against
  `qwen3.6:27b-q4_K_M` (see verdict above); the original D14 finding (file
  mode `100755` vs. repo convention `100644`) was independently corroborated
  and the fix (`chmod 644`) stands unchanged.

### Reflection log

Required passes: 3 (`RRI 55` → `Med-high`)

#### Pass 1
- **Draft verdict:** Local-model-authored migration; DROP+ADD CONSTRAINT
  pattern matching precedent 0020/0022, all 6 prior kinds preserved,
  `subtitle` appended.
- **Critique findings:** File mode `100755` (executable) deviates from
  repo's `100644` convention for `.sql` migrations (also flagged
  independently by phase-2 review).
- **Revisions applied:** `chmod 644` on the migration file.

#### Pass 2
- **Draft verdict:** Revised file (644, unchanged content).
- **Critique findings:** Checked migration-number collision risk — `0022`
  is HEAD's latest migration, no concurrent branch claims `0023`; numbering
  is safe.
- **Revisions applied:** none.

#### Pass 3
- **Draft verdict:** Same file, final state.
- **Critique findings:** Verified the literal `'subtitle'` string is an
  exact match to `ArtifactKind::Subtitle`'s wire format in
  `crates/domain/src/artifact.rs:45,65` (not a near-miss like `'subtitles'`
  or `'Subtitle'`); verified EC-1 (rejection of unknown kinds) is
  structurally guaranteed by the closed `CHECK (kind IN (...))` enumeration
  form, consistent with every prior migration touching this constraint.
- **Revisions applied:** none.

### Unit coverage certification

Not applicable — this is a migration-only change (no Rust source touched,
no new code path to cover). `crates/domain/src/artifact.rs` test coverage
for `ArtifactKind::Subtitle`/`parse_artifact_kind("subtitle")` was already
certified under S-140-T1a.

### Owner verification

Pending — reported to owner below; not yet independently re-verified by a
human.

**Status: [x] Done — 2026-07-21, migration merged to `infra/migrations/0023_extend_artifact_kind_check_subtitle.sql`**

---

## S-140-T1c-i: Subtitle artifact uniqueness constraint migration

> Split from the original T1c 2026-07-21 after phase-1 review (qwen, via
> `scripts/peer-workflow-review.py`) found that EC-4 (duplicate `Subtitle`
> artifact insertion must be rejected) cannot be enforced atomically without
> a DB-level `UNIQUE` constraint — a repository-layer check-then-insert has a
> TOCTOU race. The existing `artifact_records` table (migrations
> 0003/0019/0022/0023) has no uniqueness on `(asset_id, kind,
> parent_artifact_id)`. Bundling the constraint migration into the
> repository task pushed its RRI to 66 (Complex), the same anti-pattern that
> caused the original broad T1 to be split into T1a/T1b-i/T1b-ii. Splitting
> the migration into its own task keeps both halves at Med-high and
> locally routable. Owner decision 2026-07-21: add the constraint via a new
> migration rather than accept either a TOCTOU-prone check or a
> best-effort/non-atomic EC-4.
>
> **Implementation route:** same Med-high local-first routing as
> [[S-140-T1b-i]]/[[S-140-T1b-ii]] —
> `scripts/local-agent/run_local_task.py` + `DUBBRIDGE_LOCAL_AGENT_MODEL`
> (default `qwen3.6:35b-a3b`), 1 repair attempt max; `qwen3.6:27b-q4_K_M`
> phase-2 review and 3 Reflection passes apply.

**Effort:** L (recomputed RRI 52 — Med-high; `python3 scripts/rri.py --C 0
--T 3 --A 0 --X 1 --D 4 --K 4 --P 5 --touches
infra/migrations/0025_add_subtitle_artifact_unique_constraint.sql --penalty
auth_security --platform dubbridge`)
**Depends on:** S-140-T1b-i, S-140-T1b-ii
**Status:** Not started — unblocked; T1b complete

**Happy paths considered:**
- HP-1: Migration adds a partial `UNIQUE` index on
  `artifact_records(asset_id, parent_artifact_id) WHERE kind = 'subtitle'` so
  a second `Subtitle` insert for the same asset/parent pair is rejected by
  Postgres itself, not by application code. The partial-index form is
  deliberate: it avoids PostgreSQL's multiple-NULLs-are-distinct `UNIQUE`
  behavior by only ever indexing `subtitle` rows, which structurally always
  have non-NULL `asset_id`/`parent_artifact_id` (see acceptance criteria).

**Edge cases considered:**
- EC-1: The constraint does not affect existing rows for other kinds
  (`TranscriptText`, `WordAlignment`, etc.), which may legitimately share
  `parent_artifact_id` values with each other under the current schema.
- EC-2: Applying the migration against the existing seeded/test data (all
  prior migrations 0001–0024) succeeds without a constraint violation on
  current rows — guaranteed structurally because no repository code writes
  `Subtitle` artifact rows until [[S-140-T1c-ii]] (which depends on this
  task) ships.

**Inputs:** `infra/migrations/0019_create_preparation.sql` (introduces
`parent_artifact_id`), `infra/migrations/0023_extend_artifact_kind_check_subtitle.sql`,
`infra/migrations/0024_create_subtitle_status.sql`.

**Outputs:** New migration adding the uniqueness constraint.

**Acceptance criteria:**
- Migration is forward-only, consistent with repository convention (no down
  migrations exist); no down-migration/rollback tooling exists anywhere in
  `infra/migrations/`, and this task does not introduce one.
- Constraint scope is limited to preventing duplicate `Subtitle` artifacts
  per `(asset_id, parent_artifact_id)`; it must not restrict other artifact
  kinds sharing a `parent_artifact_id` (EC-1).
- No existing data can violate this constraint at migration time. Verified
  ground truth: this repository has no seed-data mechanism, and `'subtitle'`
  appears in exactly one prior migration
  (`0023_extend_artifact_kind_check_subtitle.sql`), which only extends a
  `CHECK` constraint and inserts no rows. No repository code writes
  `ArtifactKind::Subtitle` rows until [[S-140-T1c-ii]] (which depends on this
  task). This is a verified structural guarantee, not an assumption; no
  pre-migration `COUNT(*)` check is needed, though the migration comment
  should state this guarantee explicitly for future readers.
- The constraint targets only non-NULL `(asset_id, parent_artifact_id)`
  pairs: `asset_id` is `NOT NULL` on `artifact_records` (migration 0003);
  `parent_artifact_id` is nullable in general, but the
  `artifact_source_or_derived` `CHECK` (migration 0019) already guarantees
  `parent_artifact_id IS NOT NULL` whenever a row is derived (not an
  original) — every `Subtitle` row is derived, so both columns are always
  set for the rows this constraint targets. The partial-index form (`WHERE
  kind = 'subtitle'`) sidesteps PostgreSQL's multiple-NULLs-are-distinct
  `UNIQUE` behavior entirely.
- Migration applies cleanly on top of the full existing migration chain
  (0001–0024) with no constraint violation on current data (EC-2), verified
  against the same `local-postgres-1` container setup used for
  [[S-140-T1b-i]]/[[S-140-T1b-ii]].
- Consistent with repository convention: no down-migration/rollback tooling
  exists anywhere in `infra/migrations/` (forward-only, per T1b-i
  precedent); this task does not introduce one. If reversal is ever needed,
  the repository pattern is a new forward migration that drops the
  constraint, not a down-migration file. The migration's SQL comment must
  state the exact reversal statement (`DROP INDEX <index_name>`) so a future
  forward migration can copy it verbatim.
- Migration numbering verified: `ls infra/migrations/ | sort -V | tail -5`
  confirms `0024_create_subtitle_status.sql` (S-140-T1b-i) is the current
  highest migration on this branch; `0025` is the next available number and
  does not collide with T1b-i/T1b-ii, both already merged.
- The `CREATE UNIQUE INDEX` statement does not use `IF NOT EXISTS`:
  idempotency is provided by the `sqlx` migration runner's tracking table
  (each migration applies exactly once), consistent with every prior
  migration in `infra/migrations/`, none of which use `IF NOT EXISTS`.

**Files expected to change:**
- `infra/migrations/0025_add_subtitle_artifact_unique_constraint.sql` (new;
  verified as the next available number per the numbering check above)

**Evidence to emit:** RRI output, migration test/check command output (SQL
apply-and-inspect via `sqlx::migrate!`/the custom `test_runner` used for
T1b-i/T1b-ii against a fresh PostgreSQL 16 `local-postgres-1` container,
confirmed via `pg_get_indexdef`/`psql \d artifact_records` — not the repo's
Rust test suite, since no repository code exists yet to run those tests
against),
qwen phase-2 review artifact.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after migration validation. Do not implement
repository methods that consume the constraint (that is [[S-140-T1c-ii]]).

**Agent handoff prompt:** Add only the subtitle artifact uniqueness
constraint migration, validate it against the full migration chain, and stop
before repository code.

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M` (phase-1 task-analysis and phase-2
  code-solution, per Med-high band routing)
- Phase-1 command: `scripts/peer-workflow-review.py --phase task --rri 52`
- Phase-1 artifact: `.agent/peer-task-review-S-140-T1c-i.md` /
  `.agent/peer-task-review-S-140-T1c-i-v4.json` — Verdict: `PASS`
  (1 INFO finding, no action needed; 4 iterative rounds)
- Phase-2 command: `scripts/peer-workflow-review.py --phase code --rri 52
  --caller claude-code`
- Phase-2 artifact (round 1): `.agent/peer-code-review-S-140-T1c-i.json` —
  Verdict: `FINDINGS` (1 LOW finding: partial-index predicate should restate
  the NOT NULL guarantee explicitly rather than relying solely on the
  `artifact_source_or_derived` CHECK, to stay fail-closed against future
  schema drift)
- Disposition: accepted and repaired — added
  `AND asset_id IS NOT NULL AND parent_artifact_id IS NOT NULL` to the index
  predicate
- Phase-2 artifact (round 2, final): `.agent/peer-code-review-S-140-T1c-i-v2.json`
  — Verdict: `PASS` (1 INFO note on seed-data/deployment-order assumption;
  already covered by the task card's verified ground truth that no
  seed-data mechanism exists in this repository)
- Gemma fallback: not triggered — `qwen3.6:27b-q4_K_M` available throughout
- D14 fallback: not triggered
- disposition_divergence: none

### Implementation evidence

- Route: local-first (`scripts/local-agent/run_local_task.py`,
  `qwen3.6:35b-a3b`), disposable worktree
  `/private/tmp/dubbridge-s140-t1c-i` (branch `agent/s-140-t1c-i`), 0 repair
  attempts needed (first draft matched spec; a scope-check "out_of_scope"
  flag on `0024_create_subtitle_status.sql` was a false positive from the
  operator pre-seeding that file into the worktree for migration-chain
  context — diffed byte-identical to the source, confirmed untouched by the
  model)
- Output: `infra/migrations/0025_add_subtitle_artifact_unique_constraint.sql`
- Migration-chain validation: `sqlx::migrate!` (via the existing
  `user_account::tests` harness pointed at `DUBBRIDGE_DATABASE_URL`) applied
  cleanly against a fresh, disposable Postgres 16 container (0001–0025); the
  shared `local-postgres-1` dev database was not touched
- Index shape verified via `psql \d artifact_records` / `pg_indexes`:
  `CREATE UNIQUE INDEX artifact_records_subtitle_unique_asset_parent ON
  artifact_records USING btree (asset_id, parent_artifact_id) WHERE (kind =
  'subtitle' AND asset_id IS NOT NULL AND parent_artifact_id IS NOT NULL)`
- EC-1 verified: two `word_alignment` rows sharing the same
  `parent_artifact_id` insert successfully (index does not restrict other
  kinds)
- HP-1 verified: first `Subtitle` row for a given
  `(asset_id, parent_artifact_id)` inserts successfully
- Duplicate-rejection verified: a second `Subtitle` row for the same
  `(asset_id, parent_artifact_id)` fails with `duplicate key value violates
  unique constraint "artifact_records_subtitle_unique_asset_parent"`

**Status: [x] Done — migration validated and merged into working tree;
awaiting owner's next-step decision (T1c-ii implementation)**

---

## S-140-T1c-ii: Subtitle repository and readiness evidence

> Split from the original T1c; see [[S-140-T1c-i]] for the rationale. This
> half is the repository/readiness code and depends on T1c-i's constraint
> for atomic EC-4 enforcement via `ON CONFLICT`, matching the existing
> `upsert_transcription_status` idiom in `transcription_repo.rs`.
>
> **Exact constraint from T1c-i** (restated here so this card is
> self-contained): a partial `UNIQUE` index `CREATE UNIQUE INDEX ... ON
> artifact_records (asset_id, parent_artifact_id) WHERE kind = 'subtitle'`,
> in `infra/migrations/0025_add_subtitle_artifact_unique_constraint.sql`.

**Effort:** L (recomputed RRI 47 — Med-high; `python3 scripts/rri.py --C 2 --T 3
--A 1 --X 1 --D 3 --K 3 --P 3 --touches crates/db/src/subtitle_repo.rs
--touches crates/db/src/lib.rs --touches apps/api/tests/subtitle_repo_test.rs
--platform rust`)
**Depends on:** S-140-T1c-i
**Status:** Done — implementation complete, phase-2 review clean, Reflection
passes 1-3 done, committed as `d1cb62f`; T1d unblocked.

**Happy paths considered:**
- HP-1: Insert a `Subtitle` derived artifact and list it with correct
  `parent_artifact_id` lineage to `WordAlignment` under D1a.
- HP-2: Subtitle status transitions Pending -> InProgress -> Ready round-trip
  through the repository.
- HP-3: `get_subtitle_readiness_evidence` returns `true` only when both the
  subtitle artifact row exists AND the persisted status is Ready (artifact
  existence alone is not sufficient).

**Edge cases considered:**
- EC-1: Failed status persists `error_detail` and remains queryable.
- EC-2: `get_subtitle_status` returns `None` for an asset with no row.
- EC-3: Readiness evidence returns `false` when no subtitle artifact row
  exists for the asset, regardless of status.
- EC-4: Attempting to insert a duplicate `Subtitle` artifact for the same
  asset/parent (`WordAlignment`) pair returns `DbError::Conflict` (existing
  variant, `crates/db/src/error.rs`), rejected not silently ignored.
  Enforced atomically by the `UNIQUE` constraint added in [[S-140-T1c-i]].
  The SQLSTATE `23505` constraint-violation error from Postgres maps to
  `DbError::Conflict` using the same pattern as the existing
  `is_unique_violation(error: &sqlx::Error) -> bool` helper in
  `crates/db/src/user_account.rs:116-126`, not a repository-layer
  check-then-insert (which would have a TOCTOU race).
- EC-5: Inserting a `Subtitle` artifact with a `parent_artifact_id` that does
  not reference an existing `artifact_records` row returns
  `DbError::QueryFailed` (the existing general sqlx-error-carrying variant —
  there is no dedicated foreign-key-violation variant in this codebase
  today, unlike EC-4's `DbError::Conflict`; the underlying `sqlx::Error`
  already carries the SQLSTATE `23503` detail for callers who need it). The
  FK violation is on the existing
  `parent_artifact_id REFERENCES artifact_records(id)` constraint from
  migration `0019_create_preparation.sql` — this constraint already exists;
  no new migration is needed in this task (the new constraint from T1c-i
  covers EC-4 uniqueness, not this FK).

**Inputs:** `crates/db/src/transcription_repo.rs`, `crates/db/src/artifact_repo.rs`,
`crates/db/src/error.rs` (`DbError::Conflict` already exists; verify in
Reflection pass 1), `crates/db/src/user_account.rs` (line 116: existing
`is_unique_violation` fn checking SQLSTATE `23505` — the exact check-logic
precedent to mirror for EC-4's error mapping; it is private/module-scoped,
so `subtitle_repo.rs` duplicates the same check rather than importing it),
`apps/api/tests/transcription_repo_test.rs`, the ratified D1 lineage rule,
and the `UNIQUE` constraint from [[S-140-T1c-i]] — **before writing any code
against it, Reflection pass 1 must verify the actual merged
`infra/migrations/0025_add_subtitle_artifact_unique_constraint.sql` defines
exactly the expected index** (via `psql \d artifact_records` or
`pg_get_indexdef`), not just trust the restated expectation in this card.

**Outputs:** `crates/db/src/subtitle_repo.rs` and module export wiring.

**Acceptance criteria:**
- Repository API follows the transcription repository pattern: subtitle
  status lives in the mutable `asset_subtitle_status` row
  (`upsert_subtitle_status`, mirroring `upsert_transcription_status`), while
  the `Subtitle` derived-artifact row is a separate one-shot insert
  (mirroring `insert_transcript_artifacts`) — HP-2's Pending -> InProgress ->
  Ready transitions apply only to the status row, never to the artifact row.
- Derived artifact lineage uses one immediate parent: `WordAlignment` (D1a,
  ratified), enforced at the DB layer by the existing `parent_artifact_id`
  foreign-key constraint from migration `0019_create_preparation.sql`
  (EC-5) — verified present in Reflection pass 1 via `psql \d
  artifact_records` against a live migrated database, not assumed from
  reading the migration file alone.
- Subtitle artifact insertion sets `storage_key` and `checksum` per ADR-006
  (immutable derived-artifact row); the artifact row itself is never updated
  in place after insertion (status transitions happen only in the separate
  status row, not the artifact row).
- `get_subtitle_readiness_evidence` is fail-closed per ADR-018: it returns
  `Result<bool, DbError>` (or an equivalent explicit error-carrying type),
  any DB query failure propagates as `Err`, and it returns `true` only when
  both the artifact row exists and status is Ready.
- Duplicate `Subtitle` artifact insertion for the same asset/parent pair
  (EC-4) returns `DbError::Conflict`, enforced atomically by the T1c-i
  `UNIQUE` constraint (no repository-layer check-then-insert race), using
  the same SQLSTATE `23505` check-logic pattern as
  `is_unique_violation` in `crates/db/src/user_account.rs:116`. That helper
  is private/module-scoped (`fn`, no `pub`), so the check is duplicated
  locally in `subtitle_repo.rs` rather than imported.
- `subtitle_repo_test.rs` tests are independent of `transcription_repo_test.rs`:
  each test inserts its own asset/artifact fixture rows (matching the
  existing per-test `setup_pool`/`insert_asset` pattern already used in
  `transcription_repo_test.rs`) rather than relying on shared mutable state,
  so the two test files are safe to run in parallel or in either order.
- Existing transcription repository tests
  (`apps/api/tests/transcription_repo_test.rs`) show no regression versus
  their pre-task baseline — the new subtitle repo/tests must not couple with
  or regress transcription coverage; a pre-existing unrelated flake
  (confirmed by reproducing it on the pre-task commit) does not block
  closure.
- All HP/EC cases above have integration-test coverage in
  `apps/api/tests/subtitle_repo_test.rs`.

**Files expected to change:**
- `crates/db/src/subtitle_repo.rs` (new)
- `crates/db/src/lib.rs`
- `apps/api/tests/subtitle_repo_test.rs` (new)

No migration is in scope for this task: `asset_subtitle_status`
(`infra/migrations/0024_create_subtitle_status.sql`, S-140-T1b-i), the
artifact-kind check extension
(`infra/migrations/0023_extend_artifact_kind_check_subtitle.sql`,
S-140-T1b-ii), and the uniqueness constraint
(`infra/migrations/0025_add_subtitle_artifact_unique_constraint.sql`,
S-140-T1c-i) are all merged before this task starts; this task only writes
repository code that reads/writes those existing tables/constraints.

**Evidence to emit:** RRI output, local-run artifact, exact test commands,
unit/integration test names, and the disposition of any phase-2 review
findings that remain open at handoff/closure time.

**Closure evidence (2026-07-21):**
- RRI 47, Med-high band (`python3 scripts/rri.py --C 2 --T 3 --A 1 --X 1 --D 3
  --K 3 --P 3 --touches crates/db/src/subtitle_repo.rs --touches
  crates/db/src/lib.rs --touches apps/api/tests/subtitle_repo_test.rs
  --platform rust`).
- Phase-2 review (mandatory gate, `qwen3.6:27b-q4_K_M` via
  `scripts/peer-workflow-review.py --phase code --rri 47`) run against the
  complete final diff (tracked + untracked files):
  `.agent/peer-code-review-S-140-T1c-ii-closure.json`, verdict `findings`,
  no action-required findings: one MEDIUM ("tests lack cleanup/transactions")
  re-confirmed as a false positive — every test generates its own
  `AssetId::new()`/`Uuid::new_v4()` row, matching the already-merged pattern
  in `transcription_repo_test.rs`/`preparation_repo_test.rs`, verified by
  grep, not asserted from memory; two LOW notes, one explicitly "no change
  needed", the other a CI-robustness note on the concurrency test, not a
  defect. Full 8-round history (v1-v7) and the superseded
  `-rerun-full.json` LOW finding (missing `Failed`-status readiness test) are
  documented in `docs/tasks/handoff-s140-t1c-ii-2026-07-21.md`; that finding
  is closed — the test
  (`subtitle_readiness_evidence_false_when_artifact_exists_and_status_failed`)
  is present in `apps/api/tests/subtitle_repo_test.rs:433`.
- Reflection pass 1 (schema verification against a live migrated database,
  not the migration files alone): `psql \d artifact_records` confirms
  `artifact_records_subtitle_unique_asset_parent` UNIQUE btree on
  `(asset_id, parent_artifact_id) WHERE kind = 'subtitle'` (T1c-i, migration
  0025); `psql \d asset_subtitle_status` confirms the table (migration 0024)
  with `PRIMARY KEY (asset_id)` and `subtitle_status_check` CHECK constraint
  matching `SubtitleStatus`'s four variants.
- Reflection pass 2 (FK constraint for EC-5): `psql \d artifact_records`
  confirms `artifact_records_parent_artifact_id_fkey` FOREIGN KEY
  (`parent_artifact_id`) REFERENCES `artifact_records(id)` (migration 0019,
  pre-existing).
- Reflection pass 3 (error-mapping EC-4/EC-5 verified by test, not by
  inspection alone): `insert_subtitle_artifact_rejects_duplicate_for_same_parent`
  exercises SQLSTATE 23505 -> `DbError::Conflict`;
  `insert_subtitle_artifact_rejects_missing_parent` exercises SQLSTATE 23503
  -> `DbError::QueryFailed`, both green.
- Test commands run against a disposable `postgres:16` container (port
  5433, not the shared `local-postgres-1`):
  - `cargo test -p dubbridge-api --test subtitle_repo_test -- --nocapture`
    -> 13 passed, 0 failed (HP-1..HP-3, EC-1..EC-5, plus EC-3b/EC-4b
    supplementary cases).
  - `cargo test -p dubbridge-api --test transcription_repo_test` -> 8
    passed, 0 failed (no regression vs. pre-task baseline).
  - `cargo test -p dubbridge-db subtitle_repo::tests::parse_subtitle_status_unknown_fails_closed -- --exact`
    -> 1 passed.
  - `cargo fmt --check -p dubbridge-db -p dubbridge-api` -> clean.
  - `cargo clippy -p dubbridge-db -p dubbridge-api --tests -- -D warnings`
    -> clean, 0 warnings.
- Files committed as `d1cb62f` (2026-07-21): `crates/db/src/subtitle_repo.rs`
  (new), `apps/api/tests/subtitle_repo_test.rs` (new), `crates/db/src/lib.rs`
  (adds `pub mod subtitle_repo;`). `.githooks/pre-push` was a pre-existing,
  unrelated modification and was excluded from this commit (see
  `docs/tasks/handoff-s140-t1c-ii-2026-07-21.md`).
- Documented tech debt (not a closure blocker, out of scope for T1c-ii):
  `get_subtitle_readiness_evidence` cannot distinguish per-language
  readiness when multiple `Subtitle` artifacts exist for one asset, since
  `asset_subtitle_status` is one row per `asset_id`. Covered by test
  `subtitle_readiness_evidence_true_with_multiple_artifacts_and_asset_level_ready`,
  which documents the limitation rather than fixing it; a timestamp-ordering
  fix was attempted and discarded as flaky (two independent clock sources).

**Status artifacts affected:** This ledger (`docs/tasks/s-140-subtitle-generation.md`)
must be synced with the execution record, reflection log, and closure status
before this task is reported done.

**Stop condition:** Stop after `cargo test -p dubbridge-api --test
subtitle_repo_test` (new) passes with each of HP-1..HP-3 and EC-1..EC-5
covered by a named test, and `transcription_repo_test.rs` shows no new
failures versus its pre-task baseline (recorded before implementation
starts; a single re-run on the pre-task commit is sufficient to classify
any failure as pre-existing, not a full flake-reproduction study). A full
workspace regression run is not required for closure of this task. Do not
add storage helpers, job queues, or worker-runner hooks.

**Agent handoff prompt:** Implement only the subtitle repository/readiness seam
following the transcription repo pattern, rely on the T1c-i `UNIQUE`
constraint (not a repository-layer check) for EC-4, keep readiness evidence
fail-closed per ADR-018, cover HP-1..HP-3/EC-1..EC-5 in integration tests,
execute and document all 3 required Reflection passes (RRI 47, Med-high)
before closure, and stop before storage or job work.

**Status: [x] Done — 2026-07-21, committed as `d1cb62f`; T1d unblocked**

---

## S-140-T1d: Subtitle storage key helper

**Effort:** M (planning RRI 26 — Moderate) → **recomputed RRI 14, Low**
(`python3 scripts/rri.py --C 1 --T 1 --A 0 --X 0 --D 1 --K 1 --P 1 --touches
crates/storage/src/lib.rs --platform rust`)
**Depends on:** S-140-T1c-ii
**Status:** Done — 2026-07-21

**Happy paths considered:**
- HP-1: `subtitle_key(asset_id)` returns the canonical storage-owned subtitle
  key for an asset.

**Edge cases considered:**
- EC-1: The helper does not accept caller-provided path fragments or extensions.

**Inputs:** Existing storage key helper patterns in `crates/storage/src/lib.rs`.

**Outputs:** `subtitle_key(asset_id)` helper.

**Acceptance criteria:**
- Helper follows the existing storage key style.
- Tests cover the canonical key format and absence of caller path injection.

**Files expected to change:**
- `crates/storage/src/lib.rs`

**Evidence to emit:** RRI output and exact storage test command.

**Closure evidence (2026-07-21):**
- RRI 14, Low band — routed to Gemma Developer (`gemma4:26b-a4b-it-qat`) per
  Low-band policy, via `scripts/delegate-low-rri.py --mode before-after
  --allow-path crates/storage/src/lib.rs --apply`.
- Attempt 1 applied but produced an invalid file: Gemma emitted a duplicate,
  unclosed `#[cfg(test)] mod tests { ... }` block containing literal `...`
  placeholder text copied from the packet's illustrative example, breaking
  compilation (`error: this file contains an unclosed delimiter`). Caught by
  `cargo build -p dubbridge-storage` immediately after apply, before any
  further step. Repaired directly (not re-delegated): removed the malformed
  duplicate block, added `subtitle_key` once after `alignment_key` (matching
  precedent), and added the two required tests
  (`subtitle_key_format`, `subtitle_and_transcript_keys_differ`) inside the
  existing single `mod tests` block next to `transcript_and_alignment_keys_differ`.
- `cargo build -p dubbridge-storage` clean after repair.
- `cargo test -p dubbridge-storage --lib` → 49/49 passed, including the two
  new tests.
- `cargo fmt -p dubbridge-storage -- --check` clean.
- `cargo clippy -p dubbridge-storage --all-features -- -D warnings` clean.
- Gemma Reviewer (`scripts/gemma-code-review.py`, 3 passes) against the final
  repaired diff: `status: findings`, 1 consensus minor note (confirms the
  implementation matches `transcript_key`/`alignment_key` convention, no
  action) + 1 pass-specific minor note (no `sanitize_filename` call on
  `asset_id` — consistent with existing precedent, not a new defect, no
  action). No blocking/major findings; D14 not triggered.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after storage helper tests pass. Do not modify DB/jobs.

**Agent handoff prompt:** Add only the subtitle storage key helper and focused
tests, then stop.

**Status: [x] Done — 2026-07-21, `subtitle_key` added to
`crates/storage/src/lib.rs` with tests; T2a unblocked**

---

## S-140-T2a: Subtitle job queue contract

**Effort:** M (planning RRI 34 — Moderate; recompute at presentation time)
**Depends on:** S-140-T1d
**Status:** Not started — unblocked, T1d done

**Happy paths considered:**
- HP-1: A `SubtitleJob` carries the asset/project/target-language context needed
  by the worker-runner.
- HP-2: The in-memory subtitle queue enqueues and drains jobs in FIFO order,
  matching existing queue-test style.

**Edge cases considered:**
- EC-1: The queue returns `None` when empty.
- EC-2: Job payload construction does not accept missing asset identity.

**Inputs:** Existing `TranscriptionJob` queue contract in `crates/jobs/src/lib.rs`.

**Outputs:** `SubtitleJob`, `SubtitleJobQueue`, and in-memory implementation.

**Acceptance criteria:**
- Queue contract mirrors transcription queue naming and semantics.
- Tests cover enqueue/dequeue and empty-queue behavior.
- No worker-runner enqueue hook is added in this task.

**Files expected to change:**
- `crates/jobs/src/lib.rs`

**Evidence to emit:** RRI output, local-run artifact, exact jobs test command.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after job queue tests pass. Do not edit
`apps/worker-runner`.

**Agent handoff prompt:** Add only the subtitle job queue contract in
`crates/jobs/src/lib.rs`, test enqueue/dequeue/empty behavior, and stop before
worker-runner changes.

**Status: [ ] Not started — blocked on T1d**

---

## S-140-T2b: Transcription-ready enqueue hook

**Effort:** M (planning RRI 35 — Moderate; recompute at presentation time)
**Depends on:** S-140-T2a
**Status:** Not started — blocked on T2a

**Happy paths considered:**
- HP-1: `process_transcription_job` enqueues exactly one `SubtitleJob` after
  writing `TranscriptionStatus::Ready`.

**Edge cases considered:**
- EC-1: Failed transcription does not enqueue a subtitle job.
- EC-2: Transcription jobs that fail before readiness do not leave a queued
  subtitle job behind.

**Inputs:** `apps/worker-runner/src/main.rs` transcription processing flow and
S-130 enqueue tests.

**Outputs:** Subtitle enqueue hook from transcription readiness.

**Acceptance criteria:**
- Enqueue happens only after Ready is persisted.
- Failure paths do not enqueue.
- Tests preserve the existing S-130 transcription behavior.

**Files expected to change:**
- `apps/worker-runner/src/main.rs`

**Evidence to emit:** RRI output, local-run artifact, exact worker-runner test
command.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after enqueue-hook tests pass. Do not implement subtitle
job processing.

**Agent handoff prompt:** Add only the transcription-ready subtitle enqueue hook
in `apps/worker-runner`, test ready/failure behavior, and stop before
`process_subtitle_job`.

**Status: [ ] Not started — blocked on T2a**

---

## S-140-T3a: D1a Rust segmentation provider

**Effort:** M (planning RRI 35 — Moderate; recompute at presentation time)
**Depends on:** S-140-T2b (D1a ratified 2026-07-21)
**Status:** Not started — blocked on T2b

**Happy paths considered:**
- HP-1: Word alignments are grouped into ordered subtitle segments with
  non-overlapping `start_ms`/`end_ms` and joined text.
- HP-2: Segment grouping respects the ratified max-duration/max-character
  constraints from D2/T0.

**Edge cases considered:**
- EC-1: Empty word-alignment input returns no segments without panicking.
- EC-2: Malformed timing input fails closed instead of producing overlapping
  segments.

**Inputs:** Ratified D1a/D2 constraints and existing provider trait style in
`crates/providers/src/lib.rs`.

**Outputs:** Rust segmentation provider/trait for subtitle generation.

**Acceptance criteria:**
- The algorithm is deterministic and unit-testable.
- It does not call an external ML worker under D1a.
- All HP/EC cases above have unit coverage.

**Files expected to change:**
- `crates/providers/src/lib.rs`

**Evidence to emit:** RRI output, local-run artifact, exact provider test command.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after provider tests pass. Do not wire the provider into
worker-runner.

**Agent handoff prompt:** Implement only the D1a Rust segmentation provider and
unit tests, then stop before worker-runner integration.

**Status: [ ] Not started — blocked on T2b**

---

## S-140-T3b: Subtitle worker-runner handler and readiness transitions

**Effort:** L (planning RRI 47 — Med-high; recompute at presentation time)
**Depends on:** S-140-T3a
**Status:** Not started — blocked on segmentation implementation and RRI 41+ gate

**Happy paths considered:**
- HP-1: `process_subtitle_job` loads upstream alignment, generates subtitle
  segments, stores `subtitle.json`, inserts the `Subtitle` artifact, and writes
  `SubtitleStatus::Ready`.
- HP-2: Readiness requires both a Ready status and the persisted subtitle
  artifact evidence.

**Edge cases considered:**
- EC-1: Missing upstream alignment marks subtitle status Failed with
  `error_detail`.
- EC-2: Storage write failure marks status Failed and does not report readiness.
- EC-3: Invalid segmentation output fails closed before artifact insertion.

**Inputs:** Subtitle repo/storage/provider/job contracts from T1/T2/T3a.

**Outputs:** `process_subtitle_job` and readiness transition wiring.

**Acceptance criteria:**
- Pending -> InProgress -> Ready/Failed transitions are durable.
- Subtitle artifact checksum/key/lineage follow ADR-006.
- Failure paths record durable observability per ADR-018.
- All HP/EC cases above have focused tests.

**Files expected to change:**
- `apps/worker-runner/src/main.rs`
- `crates/db/src/subtitle_repo.rs` (only if final API adjustment is required)

**Evidence to emit:** RRI output, cross-vendor/D14 phase-1 artifact, local-run
or escalation artifact, exact worker-runner test command.

**Status artifacts affected:** This ledger and S-140 plan if RRI/decomposition
changes.

**Stop condition:** Stop after subtitle handler tests pass. Do not enqueue S-160
review tasks.

**Agent handoff prompt:** Implement only `process_subtitle_job` readiness and
artifact persistence, cover success/failure tests, and stop before S-160 review
enqueue.

**Status: [ ] Not started — blocked on segmentation implementation and RRI 41+ gate**

---

> **S-140-T4 removed 2026-07-21:** the D1b Python subtitle worker task was
> removed, not skipped, after T0 ratified D1a. Its own stop condition required
> exactly this: "If D1a ... is ratified instead, this task is removed from the
> ledger rather than marked done." See T0 and
> `docs/plan/s-140-subtitle-generation.md` D1/D3 for the ratification record.

---

## S-140-T5a: ADR-030 review-task enqueue on subtitle readiness

**Effort:** M (planning RRI 39 — Moderate; recompute at presentation time)
**Depends on:** S-140-T3b
**Status:** Not started — blocked on T3b

**Happy paths considered:**
- HP-1: When `SubtitleStatus::Ready` is written and artifact evidence exists,
  the worker enqueues a review task through the existing ADR-030/S-160 path.

**Edge cases considered:**
- EC-1: Subtitle Failed status does not enqueue a review task.
- EC-2: Ready status without artifact evidence does not enqueue a review task.
- EC-3: The implementation does not introduce a parallel review state machine or
  bypass the existing publication gate.

**Inputs:** S-160 review-task creation API and ADR-030 publication-gate contract.

**Outputs:** Review-task enqueue wiring from subtitle readiness.

**Acceptance criteria:**
- Uses the existing S-160/ADR-030 path only.
- Enqueue is gated by subtitle readiness evidence.
- Tests cover ready, failed, and missing-artifact cases.

**Files expected to change:**
- `apps/worker-runner/src/main.rs`

**Evidence to emit:** RRI output, local-run artifact, exact test command.

**Status artifacts affected:** This ledger and any S-160/X-S-160-3 blocker note
only if the task changes blocker state.

**Stop condition:** Stop after review enqueue tests pass. Do not change
`review_tasks` schema in this task.

**Agent handoff prompt:** Add only ADR-030/S-160 review enqueue on subtitle
readiness, prove failed/missing-artifact paths do not enqueue, and stop before
any schema change.

**Status: [ ] Not started — blocked on T3b**

---

## S-140-T5b: Optional derived-artifact identity schema change for review tasks

**Effort:** L (RRI TBD — recompute only if scoped)
**Depends on:** S-140-T5a and explicit owner decision to carry artifact identity
in `review_tasks`
**Status:** Not started — optional; not authorized by S-140-T5a

This task exists only to prevent accidental scope creep. The S-140 plan records
that X-S-160-3 cannot fully close unless review tasks can carry a derived-artifact
identity/version. If the owner decides to close that gap here, this task must be
expanded into a full development card with HP/EC, migration acceptance criteria,
RRI output, phase-1 review, and approval before implementation.

**Files expected to change if scoped:**
- `infra/migrations/00XX_update_review_tasks_artifact_identity.sql` (new)
- `crates/domain/src/review.rs`
- S-160/S-140 repository or worker-runner files identified at presentation time

**Stop condition:** Stop after scoping/presentation. Do not implement from this
placeholder.

**Status: [ ] Not started — optional and unscoped**

---

## S-140-T6: BDD feature file + docs sync

**Effort:** S (planning RRI 6 — Low; recompute at presentation time)
**Depends on:** S-140-T5a, and S-140-T5b if T5b is explicitly scoped
**Status:** Not started — blocked on T5a and any scoped T5b

Mirrors S-130-T5: `docs/bdd/s-140-subtitle-generation.feature`, roadmap
status sync (`docs/plan/roadmap.md:134`), plan/task ledger closure.

**Acceptance criteria:**
- BDD feature records the delivered S-140 happy/failure paths.
- Roadmap, S-140 plan, and this ledger reflect the final delivered state.
- Any remaining X-S-160-3 blocker is described accurately instead of being
  closed by implication.

**Files expected to change:**
- `docs/bdd/s-140-subtitle-generation.feature`
- `docs/plan/roadmap.md`
- `docs/plan/s-140-subtitle-generation.md`
- `docs/tasks/s-140-subtitle-generation.md`

**Evidence to emit:** RRI output, `make qa-docs` output, final status-sync note.

**Status artifacts affected:** Roadmap, S-140 plan, this ledger, and BDD index if
the feature is indexed there.

**Stop condition:** Stop after docs checks pass and status artifacts are synced.
Do not start S-150 or S-170 planning.

**Status: [ ] Not started — blocked on T5a and any scoped T5b**
