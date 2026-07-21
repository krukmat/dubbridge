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
**Status:** Not started — blocked on T1a and RRI 41+ gate

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

**Status: [ ] Not started — blocked on T1a and RRI 41+ gate**

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

## S-140-T1c: Subtitle repository and readiness evidence

**Effort:** M (planning RRI 40 — Moderate; recompute at presentation time)
**Depends on:** S-140-T1b-i, S-140-T1b-ii
**Status:** Not started — blocked on T1b-i and T1b-ii

**Happy paths considered:**
- HP-1: Insert a `Subtitle` derived artifact and list it with correct
  `parent_artifact_id` lineage to `WordAlignment` under D1a.
- HP-2: Subtitle status transitions Pending -> InProgress -> Ready round-trip
  through the repository.
- HP-3: `get_subtitle_readiness_evidence` returns `true` once the subtitle
  artifact exists for the asset.

**Edge cases considered:**
- EC-1: Failed status persists `error_detail` and remains queryable.
- EC-2: `get_subtitle_status` returns `None` for an asset with no row.
- EC-3: Readiness evidence returns `false` when status is Ready but no artifact
  row exists.

**Inputs:** `crates/db/src/transcription_repo.rs`,
`apps/api/tests/transcription_repo_test.rs`, and the ratified D1 lineage rule.

**Outputs:** `crates/db/src/subtitle_repo.rs` and module export wiring.

**Acceptance criteria:**
- Repository API follows the transcription repository pattern.
- Derived artifact lineage uses one immediate parent: `WordAlignment` (D1a,
  ratified).
- All HP/EC cases above have integration-test coverage.

**Files expected to change:**
- `crates/db/src/subtitle_repo.rs` (new)
- `crates/db/src/lib.rs`
- `apps/api/tests/subtitle_repo_test.rs` (new)

**Evidence to emit:** RRI output, local-run artifact, exact test commands,
unit/integration test names.

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after repository tests pass. Do not add storage helpers,
job queues, or worker-runner hooks.

**Agent handoff prompt:** Implement only the subtitle repository/readiness seam
following the transcription repo pattern, cover HP/EC in integration tests, and
stop before storage or job work.

**Status: [ ] Not started — blocked on T1b**

---

## S-140-T1d: Subtitle storage key helper

**Effort:** M (planning RRI 26 — Moderate; recompute at presentation time)
**Depends on:** S-140-T1c
**Status:** Not started — blocked on T1c

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

**Status artifacts affected:** This ledger.

**Stop condition:** Stop after storage helper tests pass. Do not modify DB/jobs.

**Agent handoff prompt:** Add only the subtitle storage key helper and focused
tests, then stop.

**Status: [ ] Not started — blocked on T1c**

---

## S-140-T2a: Subtitle job queue contract

**Effort:** M (planning RRI 34 — Moderate; recompute at presentation time)
**Depends on:** S-140-T1d
**Status:** Not started — blocked on T1d

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
