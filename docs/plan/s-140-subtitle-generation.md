---
type: Plan
title: "Plan: S-140 — Subtitle Generation"
status: proposed
slice: S-140
---
# Plan: S-140 — Subtitle Generation

> **Status:** Proposed 2026-07-20. Authored via ADR-037 T5 (Local Architect
> advisory verified against repository evidence; see reconciliation in
> `docs/evaluations/adr037-direct-project-report.md`). Not yet approved —
> requires cross-vendor peer/D14 review per RRI 53 (Med-high) before task
> execution begins.
> **Roadmap phase:** `S-140` — Processing / Subtitle generation.
> **Tasks ledger:** `docs/tasks/s-140-subtitle-generation.md`.

## Purpose

S-130 delivers `TranscriptText` and `WordAlignment` derived artifacts and a
`TranscriptionStatus::Ready` gate per asset. The next processing stage —
turning the timed transcript into a canonical subtitle artifact — has no
plan, no task ledger, and no worker contract today
(`docs/plan/roadmap.md:134`: "⬜ no plan yet").

S-140 closes that gap: a canonical internal subtitle artifact schema,
Rust orchestration analogous to S-130's `TranscriptionJob` pattern, a
`SubtitleReady` readiness gate, and integration with the existing S-160
review/publication contract defined by ADR-030 — not a new gate, the
**existing** one.

Without this slice, S-170 (human review runtime) has no plan and cannot
proceed (`docs/plan/roadmap.md:137`), and roadmap item `X-S-160-3` (review
tasks against real derived-asset identities) stays open
(`docs/plan/roadmap.md:384`).

**Correction to prior advisory framing:** S-160 itself is not blocked on
S-140 — S-160 shipped complete (`✅ done 2026-06-13`,
`docs/plan/roadmap.md:136`), built against fixtures per its own ADR-030
design ("The contract operates ahead of real subtitle/dub producers",
`docs/adr/ADR-030-review-decision-ledger-and-fail-closed-publication-gate.md:104`).
Only the narrow sub-item X-S-160-3 and the full S-170 slice depend on S-140.
**X-S-160-3 is not fully closed by S-140 as scoped below**: `review_tasks`
(`infra/migrations/0014_create_review_tasks.sql`,
`crates/domain/src/review.rs`) stores only `(project_id, asset_id,
target_language_id)` — it has no column for derived-artifact identity or
version. S-140-T5 enqueuing a review task on `SubtitleStatus::Ready` gives
X-S-160-3 a real subtitle artifact to point *at*, but the review task itself
still cannot carry that artifact's identity without a schema change. Closing
X-S-160-3 requires that schema change as an explicit, separately-scoped
follow-up (S-160 or S-140-T5, to be decided at T5 presentation) — it is not
a side effect of T5 as currently decomposed.

## Objective

Deliver a fail-closed subtitle-generation stage that:

- defines a canonical internal subtitle artifact schema (segments with
  timing + text, derived from `WordAlignment`);
- extends `ArtifactKind` with a `Subtitle` variant, following the
  `TranscriptText`/`WordAlignment` precedent (`crates/domain/src/artifact.rs`);
- enqueues a `SubtitleJob` when `TranscriptionStatus` transitions to `Ready`;
- dispatches to a worker (Rust-only or Rust+Python, per Design decision D1)
  behind a typed trait, following the `AsrWorkerClient` pattern in
  `crates/providers`;
- persists the subtitle artifact in object storage with canonical
  storage-owned keys and correct lineage (`parent_artifact_id` → a single
  immediate source artifact — `WordAlignment` under default D1a; see T1
  acceptance criteria for the D1b conditional case), per ADR-006;
- gates subtitle readiness (`SubtitleStatus::Ready`) fail-closed on artifact
  presence;
- enqueues S-160 review tasks through ADR-030's existing publication gate —
  **no parallel review path**, per the ADR-030 obligation this slice must
  satisfy, not merely consider; and
- produces durable observability on success and failure (ADR-018).

## Scope

### Included

- New `ArtifactKind::Subtitle` variant.
- New `SubtitleStatus` domain type (Pending / InProgress / Ready / Failed),
  mirroring `TranscriptionStatus`.
- Canonical internal subtitle JSON schema (segments: start_ms, end_ms, text),
  versioned, decided in Design decision D2 below.
- Migration: `asset_subtitle_status` table + `artifact_kind_check` extension.
- `crates/db`: `subtitle_repo.rs` (status CRUD, artifact insertion, readiness
  evidence query) — same shape as `transcription_repo.rs`.
- `crates/storage`: canonical key helpers for `subtitles/<asset_id>/`.
- `crates/jobs`: `SubtitleJob` payload + queue trait + in-memory impl.
- `crates/providers`: subtitle generation client trait (segmentation source
  decided in D1).
- `apps/worker-runner`: transcription-ready hook that enqueues `SubtitleJob`;
  `process_subtitle_job(...)` handler; readiness transition; S-160 review-task
  enqueue call against the existing ADR-030 publication gate.
- BDD feature file and docs sync.

### Excluded

- Translation / multilingual subtitles — explicitly owned by **S-150**, not
  S-140, per the roadmap pipeline (`docs/plan/roadmap.md:82`:
  `S-140 subtitles -> S-150 translation + dubbing`). This resolves the open
  question the T4 advisory raised; it is not actually open.
- Dubbing / TTS / voice cloning (S-150).
- Human review runtime execution (S-170) — S-140 only produces the artifact
  and enqueues the review task; it does not implement review UI or decisions.
- CDN delivery of subtitle artifacts; consumption is internal pipeline +
  S-160/S-170 review surfaces only, until S-180 publication.

## Governing constraints

- ADR-006: subtitle artifacts are immutable object-store records referenced
  by key and SHA-256 checksum, with lineage (`parent_artifact_id`) to a
  single immediate source artifact — see T1 acceptance criteria for the
  D1a/D1b lineage cases.
- ADR-018: subtitle-generation failures and transitions must produce durable
  traceable observability.
- ADR-021: transcription completion (`TranscriptionStatus::Ready`) is the
  upstream gate; subtitle generation is a downstream derived stage, not a
  parallel ingest path.
- **ADR-030 (binding, not advisory):** S-140 must enqueue review tasks and
  call the existing publication gate rather than introduce a parallel path
  — this is a ratified obligation the ADR already places on this slice
  (`docs/adr/ADR-030-*.md:104`), not a new recommendation to weigh.
- ADR-026: no environment-specific defaults compiled in.
- Roadmap X23: S-140/S-150 artifact contract with the review/publication gate
  is closed by ADR-030; S-140 consumes it, does not redefine it.

## Design decisions

### D1 — Segmentation source (OPEN — requires decision before task execution)

**Not resolved by repository evidence.** Two options:

- **D1a — Reuse `WordAlignment` directly**: derive subtitle segment
  boundaries from existing word-level timestamps with a deterministic
  Rust grouping heuristic (max chars/line, max duration/segment). No new
  ML model, no new Python worker. Lower effort, no new worker contract to
  maintain.
- **D1b — New segmentation model/heuristic**: a dedicated Python worker
  (`workers/subtitle-worker-py`, mirroring `asr-worker-py`) that takes
  transcript + alignment and produces higher-quality segment boundaries
  (e.g. sentence-aware, reading-speed-aware).

This plan defaults to **D1a** for v1 (lower effort, no new worker surface,
consistent with "Rust-first orchestration" default) but flags this as a
reviewer decision point — D1b should be chosen instead if reading-speed /
sentence-boundary quality is a hard product requirement for v1. Whoever
approves the canonical task must confirm D1a vs. D1b before T1 starts.

### D2 — Canonical subtitle artifact schema (OPEN — requires decision before task execution)

**Not resolved by repository evidence.** No SRT/VTT/internal-JSON schema
exists anywhere in the repo today. Proposed default: an internal JSON schema
(`subtitle.json`) with `{segments: [{start_ms, end_ms, text}], source_language}`,
analogous to `alignment.json`'s shape, with SRT/VTT export deferred to S-160/
S-170/S-180 presentation concerns rather than baked into the stored artifact.
This keeps the stored artifact format-agnostic and matches ADR-006's
"the database/object-store row is the authority" posture. Final schema must
be ratified in task T1's acceptance criteria, not assumed here.

### D3 — Worker boundary (depends on D1)

If D1a is chosen: no new worker; grouping logic lives in
`crates/providers` or `apps/worker-runner` directly (pure Rust). If D1b is
chosen: new `workers/subtitle-worker-py` following the `asr-worker-py`
input/output/error JSON-schema contract pattern exactly.

### D4 — Enqueue trigger location

Mirrors S-130-D5: the `SubtitleJob` enqueue happens inside
`process_transcription_job` in `apps/worker-runner`, immediately after it
writes `TranscriptionStatus::Ready`.

## Affected components

| Layer | Path | Change |
|---|---|---|
| Domain | `crates/domain/src/artifact.rs` | `+Subtitle` kind; `+SubtitleStatus`, `+SubtitleStatusRecord` |
| Migration | `infra/migrations/00XX_create_subtitle.sql` | `asset_subtitle_status` table + `artifact_kind_check` extension |
| DB | `crates/db/src/subtitle_repo.rs` (new) | status CRUD, artifact insertion, readiness evidence |
| Storage | `crates/storage/src/lib.rs` | `+subtitle_key` helper |
| Jobs | `crates/jobs/src/lib.rs` | `+SubtitleJob`, `+SubtitleJobQueue` trait, `+InMemorySubtitleJobQueue` |
| Providers | `crates/providers/src/lib.rs` | segmentation client trait per D1/D3 |
| Worker-runner | `apps/worker-runner/src/main.rs` | transcription-ready enqueue hook; `process_subtitle_job` handler; S-160 review-task enqueue call |
| Python worker (if D1b) | `workers/subtitle-worker-py/` (new) | only if D1b is selected |
| BDD | `docs/bdd/s-140-subtitle-generation.feature` | new feature file |
| Docs | `docs/plan/roadmap.md`, this plan, task ledger | sync to in-progress/closed as work proceeds |

## Task decomposition

| Task | Title | Effort | Provisional RRI | Band |
|------|-------|--------|-----------------|------|
| T1 | Domain types + migration + repository + schema/segmentation decision (D1/D2) | M | 38 | Moderate |
| T2 | Job contract + enqueue from transcription-ready | M | 36 | Moderate |
| T3 | Segmentation client trait + worker-runner handler + readiness gating | L | 44 | Med-high |
| T4 | (only if D1b) Python subtitle worker implementation | M | 37 | Moderate |
| T5 | S-160 review-task enqueue integration (ADR-030 gate) | M | 39 | Moderate |
| T6 | BDD feature file + docs sync | S | — | Done (docs) |

Tasks must run in order: T1 → T2 → T3 → (T4 if D1b) → T5 → T6.
Each task requires its own RRI computation and presentation/approval before
execution, per repository workflow — the table above is provisional only.

## Pipeline context

```
S-130 TranscriptionStatus::Ready → [T2 enqueue] → SubtitleJob
    → [T3 worker-runner] → segmentation (D1a Rust heuristic | D1b Python worker)
    → subtitle.json artifact
    → [T3 persist] → Subtitle artifact (lineage → WordAlignment under D1a;
      see T1 acceptance criteria for D1b conditional case)
    → SubtitleStatus::Ready
    → [T5] → S-160 review task enqueued via existing ADR-030 publication gate
    → (unblocks S-170 planning; X-S-160-3 remains open — see Open questions/risks)
```

## Open questions / risks

| Risk | Disposition |
|------|-------------|
| D1 segmentation source undecided | Must be ratified by reviewer/approver before T1 starts; default D1a proposed above |
| D2 canonical schema undecided | Must be ratified in T1 acceptance criteria; default internal JSON proposed above |
| Multilingual subtitles | **Not a risk** — explicitly out of scope, owned by S-150 per roadmap pipeline |
| S-160/S-170 dependency framing | S-160 is done; only X-S-160-3 and full S-170 depend on S-140 — do not reintroduce the corrected "S-160 blocked" framing in task text |
| ADR-030 compliance | Must reuse the existing gate; any new/parallel review path is a fail-closed violation, not a design option |
| X-S-160-3 closure requires a `review_tasks` schema change | `review_tasks` has no artifact-identity column today (`infra/migrations/0014_create_review_tasks.sql`); T5 must decide and scope that change explicitly, not assume the existing gate already carries derived-artifact identity |
