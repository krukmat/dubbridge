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
> No task below has started execution. T1 requires ratification of Design
> decisions D1 (segmentation source) and D2 (subtitle schema) from
> `docs/plan/s-140-subtitle-generation.md` before implementation, plus its
> own RRI computation and presentation/approval — this ledger does not
> inherit T5's approval.
> **Plan:** `docs/plan/s-140-subtitle-generation.md`.
> **Behavioral coverage contract:** unit-v1.

## S-140-T1: Domain types + migration + repository + D1/D2 ratification

**Effort:** M (provisional RRI 38 — Moderate; recompute at presentation time)
**Depends on:** S-130 (closed)
**Status:** Not started — blocked on D1/D2 ratification and presentation/approval

**Happy paths considered (provisional — confirm against ratified D1/D2 at presentation):**
- HP-1: Insert a `Subtitle` derived artifact and list it back with correct
  `parent_artifact_id` lineage to its single immediate source artifact
  (`WordAlignment` under default D1a; see acceptance criteria for the D1b
  conditional case).
- HP-2: `SubtitleStatus` transitions Pending → InProgress → Ready round-trip
  through the repository, mirroring `TranscriptionStatus`.
- HP-3: `get_subtitle_readiness_evidence` returns `true` once the `Subtitle`
  artifact exists for the asset.

**Edge cases considered (provisional):**
- EC-1: `Failed` status persists `error_detail` and is queryable.
- EC-2: Unknown `ArtifactKind::Subtitle` or `SubtitleStatus` values fail
  closed (`UnknownStoredValue`), matching S-130-T1-EC-3.
- EC-3: `get_subtitle_status` returns `None` for an asset with no status row.

**Inputs:**
- `crates/domain/src/artifact.rs` — existing `ArtifactKind`,
  `TranscriptionStatus` patterns (direct precedent).
- `crates/db/src/transcription_repo.rs` — repository seam pattern to follow.
- `docs/plan/s-140-subtitle-generation.md` D1/D2 — **must be ratified by the
  approver before this task's acceptance criteria are finalized**; the
  canonical schema (D2) and segmentation source (D1) are not yet decided.

**Outputs:**
- `ArtifactKind::Subtitle` in domain.
- `SubtitleStatus` enum and `SubtitleStatusRecord` in domain.
- Migration: `asset_subtitle_status` table + extended `artifact_kind_check`.
- `crates/db/src/subtitle_repo.rs`: status CRUD, artifact insertion,
  readiness evidence query.
- `crates/storage/src/lib.rs`: `subtitle_key(asset_id)` helper.
- Ratified subtitle JSON schema (D2), documented in this task's acceptance
  criteria once decided — not before.

**Acceptance criteria (placeholder — finalize after D1/D2 ratification):**
- `ArtifactKind::Subtitle` round-trips through `to_string` / `parse_artifact_kind`.
- Migration adds `asset_subtitle_status` mirroring `asset_transcription_status`
  shape (`asset_id` PK, `status TEXT NOT NULL`, `error_detail TEXT`, `updated_at`).
- `insert_subtitle_artifact` creates one `Subtitle` derived artifact row
  linked via `parent_artifact_id` to its single immediate source artifact.
  `parent_artifact_id` (`crates/domain/src/artifact.rs`) is a single UUID
  column — it cannot reference two parents. Under D1a (default), the parent
  is `WordAlignment` only, since alignment already encodes transcript timing
  and no dual-parent case arises. **If D1b is ratified and the segmentation
  worker's design genuinely requires lineage to both `TranscriptText` and
  `WordAlignment`**, this acceptance criterion does not hold as written and
  T1 must instead either (a) narrow lineage to `WordAlignment` only, treating
  `TranscriptText` as input-not-lineage, or (b) scope a `parent_artifact_id`
  schema change (e.g. a join table) as part of T1's own deliverables. This
  must be resolved at D1/D1b ratification, before T1's acceptance criteria
  are finalized — not assumed here.
- `get_subtitle_readiness_evidence` returns `true` only when the artifact
  is present for the asset.
- All HP and EC cases above are unit-tested.

**Files expected to change:**
- `crates/domain/src/artifact.rs`
- `infra/migrations/00XX_create_subtitle.sql` (new)
- `crates/db/src/subtitle_repo.rs` (new)
- `crates/db/src/lib.rs`
- `crates/storage/src/lib.rs`
- `apps/api/tests/subtitle_repo_test.rs` (new integration test)

**Agent handoff prompt:** Do not start until D1 (segmentation source) and D2
(subtitle schema) in `docs/plan/s-140-subtitle-generation.md` are ratified
by the task approver. Once ratified, add `ArtifactKind::Subtitle` and
`SubtitleStatus`/`SubtitleStatusRecord` to the domain, extend the
`artifact_kind_check` migration, create `asset_subtitle_status`, implement
`subtitle_repo` with status CRUD and readiness evidence, add a
`subtitle_key` storage helper, and cover all HP/EC cases with integration
tests following the `transcription_repo_test` pattern.

**Status: [ ] Not started — blocked on D1/D2 ratification**

---

## S-140-T2: Job contract + enqueue from transcription-ready

**Effort:** M (provisional RRI 36 — Moderate)
**Depends on:** S-140-T1
**Status:** Not started — blocked on T1

Mirrors S-130-T2 exactly (`SubtitleJob` payload + queue trait, enqueue hook
inside `process_transcription_job` on `TranscriptionStatus::Ready`). Full
HP/EC breakdown, inputs/outputs, and acceptance criteria to be authored at
T1 closure, once the ratified domain types from T1 are known. See
`docs/plan/s-140-subtitle-generation.md` Design decision D4.

**Status: [ ] Not started — blocked on T1**

---

## S-140-T3: Segmentation client trait + worker-runner handler + readiness gating

**Effort:** L (provisional RRI 44 — Med-high)
**Depends on:** S-140-T2
**Status:** Not started — blocked on T2 and D1 ratification

Shape depends entirely on D1 (Rust-only heuristic vs. new Python worker) —
see `docs/plan/s-140-subtitle-generation.md` Design decisions D1/D3. Full
task detail to be authored once D1 is ratified; authoring it earlier would
risk presenting acceptance criteria for a design that isn't chosen.

**Status: [ ] Not started — blocked on T2 and D1 ratification**

---

## S-140-T4: Python subtitle worker implementation (only if D1b selected)

**Effort:** M (provisional RRI 37 — Moderate)
**Depends on:** S-140-T3, D1 ratified as D1b
**Status:** Not started — conditional; skipped entirely if D1a is ratified

If D1a (reuse `WordAlignment` via Rust heuristic) is ratified instead, this
task is removed from the ledger rather than marked done, and task numbering
in the plan's decomposition table is adjusted accordingly at T1 closure.

**Status: [ ] Not started — conditional on D1b**

---

## S-140-T5: S-160 review-task enqueue integration (ADR-030 gate)

**Effort:** M (provisional RRI 39 — Moderate)
**Depends on:** S-140-T3 (and T4 if applicable)
**Status:** Not started — blocked on T3

Enqueues a review task through the **existing** ADR-030 publication gate on
`SubtitleStatus::Ready` — this is satisfying a ratified ADR-030 obligation
(`docs/adr/ADR-030-*.md:104`), not introducing new review policy. No new
review state machine, no parallel path. Full HP/EC breakdown to be authored
at T3 closure against the real S-160 review-task creation API.

**Status: [ ] Not started — blocked on T3**

---

## S-140-T6: BDD feature file + docs sync

**Effort:** S
**Depends on:** S-140-T5
**Status:** Not started — blocked on T5

Mirrors S-130-T5: `docs/bdd/s-140-subtitle-generation.feature`, roadmap
status sync (`docs/plan/roadmap.md:134`), plan/task ledger closure.

**Status: [ ] Not started — blocked on T5**
