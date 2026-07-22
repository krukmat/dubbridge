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
version. S-140-T5a enqueuing a review task on `SubtitleStatus::Ready` gives
X-S-160-3 a real subtitle artifact to point *at*, but the review task itself
still cannot carry that artifact's identity without a schema change. Closing
X-S-160-3 requires that schema change as an explicit, separately-scoped
follow-up (S-160 or S-140-T5b, to be decided at T5a/T5b presentation) — it is
not a side effect of T5a as currently decomposed.

## Objective

Deliver a fail-closed subtitle-generation stage that:

- defines a canonical internal subtitle artifact schema (segments with
  timing + text, derived from `WordAlignment`);
- extends `ArtifactKind` with a `Subtitle` variant, following the
  `TranscriptText`/`WordAlignment` precedent (`crates/domain/src/artifact.rs`);
- enqueues a `SubtitleJob` when `TranscriptionStatus` transitions to `Ready`;
- dispatches to a Rust-only segmentation provider (D1a, ratified) behind a
  typed trait, following the `AsrWorkerClient` pattern in `crates/providers`;
- persists the subtitle artifact in object storage with canonical
  storage-owned keys and correct lineage (`parent_artifact_id` → the single
  immediate source artifact, `WordAlignment`), per ADR-006;
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
- `crates/providers`: subtitle generation client trait (Rust segmentation
  provider, D1a ratified).
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
  single immediate source artifact, `WordAlignment` — see T1c's acceptance
  criteria.
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

### D1 — Segmentation source (RATIFIED 2026-07-21 — D1a)

**Ratified: D1a.** Owner confirmed the plan's default via T0 closure on
2026-07-21. Two options were considered:

- **D1a — Reuse `WordAlignment` directly**: derive subtitle segment
  boundaries from existing word-level timestamps with a deterministic
  Rust grouping heuristic (max chars/line, max duration/segment). No new
  ML model, no new Python worker. Lower effort, no new worker contract to
  maintain.
- **D1b — New segmentation model/heuristic**: a dedicated Python worker
  (`workers/subtitle-worker-py`, mirroring `asr-worker-py`) that takes
  transcript + alignment and produces higher-quality segment boundaries
  (e.g. sentence-aware, reading-speed-aware).

D1a was chosen for v1: lower effort, no new worker surface, consistent with
the "Rust-first orchestration" default. T4 (the D1b Python worker task) is
removed from the task ledger rather than marked done — see task
decomposition below. D1b (sentence-aware / reading-speed-aware Python
worker) remains available as a future follow-up if quality requirements
change; re-ratify D1 first if that follow-up is ever proposed.

### D2 — Canonical subtitle artifact schema (RATIFIED 2026-07-21)

**Ratified:** the proposed internal JSON schema. No SRT/VTT/internal-JSON
schema existed anywhere in the repo before this decision. Canonical schema:
`subtitle.json` with `{segments: [{start_ms, end_ms, text}], source_language}`,
analogous to `alignment.json`'s shape, with SRT/VTT export deferred to S-160/
S-170/S-180 presentation concerns rather than baked into the stored artifact.
This keeps the stored artifact format-agnostic and matches ADR-006's
"the database/object-store row is the authority" posture. Task T1c's
acceptance criteria implement this schema; any future field change requires
re-ratifying D2, not a silent schema drift.

### D3 — Worker boundary (resolved by D1a)

No new worker; grouping logic lives in `crates/providers` or
`apps/worker-runner` directly (pure Rust). The D1b alternative (a new
`workers/subtitle-worker-py` following the `asr-worker-py` contract pattern)
was not chosen and has no task in this ledger.

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
| Providers | `crates/providers/src/lib.rs` | Rust segmentation client trait per D1a/D3 |
| Worker-runner | `apps/worker-runner/src/main.rs` | transcription-ready enqueue hook; `process_subtitle_job` handler; S-160 review-task enqueue call |
| BDD | `docs/bdd/s-140-subtitle-generation.feature` | new feature file |
| Docs | `docs/plan/roadmap.md`, this plan, task ledger | sync to in-progress/closed as work proceeds |

## Task decomposition

| Task | Title | Effort | Provisional RRI | Band |
|------|-------|--------|-----------------|------|
| T0 | Ratify D1/D2 and freeze local-handoff sequence | S | 6 | Low |
| T1a | Domain subtitle kind/status types | S | 24 | Low |
| T1b-i | Subtitle status table migration | L | 52 | Med-high |
| T1b-ii | Artifact-kind check extension for subtitle | L | 45 | Med-high |
| T1c | Subtitle repository and readiness evidence | M | 40 | Moderate |
| T1d | Subtitle storage key helper | M | 26 | Moderate |
| T2a | Subtitle job queue contract | M | 34 | Moderate |
| T2b-i | Worker-runner extraction seam for subtitle enqueue | M | 36 | Moderate |
| T2b-ii | Transcription-ready subtitle enqueue hook | M | 34 | Moderate |
| T3a | D1a Rust segmentation provider | M | 35 | Moderate |
| T3b | Subtitle worker-runner handler and readiness transitions | L | 47 | Med-high |
| T5a | ADR-030 review-task enqueue on subtitle readiness | M | 39 | Moderate |
| T5b | Optional derived-artifact identity schema change for review tasks | L | TBD | Recompute if scoped |
| T6 | BDD feature file + docs sync | S | 6 | Low |

T4 (D1b Python subtitle worker) is removed, not skipped — D1 was ratified as
D1a on 2026-07-21, so no Python worker task exists in this decomposition.

Tasks must run in order: T0 → T1a → (T1b-i, T1b-ii in either order, both
required) → T1c → T1d → T2a → T2b-i → T2b-ii → T3a → T3b → T5a →
(T5b only if explicitly scoped) → T6.
Each task requires its own RRI computation and presentation/approval before
execution, per repository workflow. The RRI values above are planning scores
from 2026-07-21 and must be recomputed at task presentation time.

`T2b` was split on 2026-07-21 into an extraction seam and a wiring step so the
worker-runner change stays reviewable under the local-first workflow instead of
forcing a single large `main.rs` handoff.

Implementation note 2026-07-22: `S-140-T2b-ii` landed with a larger-than-planned
surface because the owner directed an aggressive refactor instead of a minimal
hook. The actual implementation moved transcription runtime logic out of
`apps/worker-runner/src/main.rs` into
`apps/worker-runner/src/transcription_runtime.rs`, moved preparation runtime
and its tests into `apps/worker-runner/src/preparation_runtime.rs`, added
atomic subtitle-pending claim semantics in `crates/db/src/subtitle_repo.rs`,
added deterministic asset-to-first-target routing in
`crates/db/src/workspace_repo.rs`, and reduced `main.rs` to a 44-line startup
entrypoint. The task closed at **RRI 50 (Med-high)** rather than the
provisional planning score 34. The decomposition itself remains valid; only
the realized change surface grew.

Live-pilot note 2026-07-21/22: the Serena-based semantic reruns of `T2b-i`
(`LASE-T6`) never converged on an edit. That path was removed on 2026-07-22 and
replaced with a simple read/write/patch local runner; `T2b-i` is re-run through
it. See `docs/plan/local-agent-simple-editing.md` and the resolution note in
`docs/tasks/s-140-subtitle-generation.md` § `S-140-T2b-i`.

T1b was split into T1b-i and T1b-ii on 2026-07-21: both touch
`infra/migrations/**`, whose anchor-rubric floor (ADR-008/ADR-018) plus the
automatic `auth_security` penalty keeps every migration-path task at RRI 41+
regardless of diff size, so splitting narrows each task's change surface
without changing either one's band.

### Coordination-mode adjustment required before implementation

The current decomposition was authored before the owner request to make local
roles take control of S-140 implementation while the primary Codex role stays
focused on the harder coordination bands. It is therefore a planning surface,
not an execution-ready handoff surface.

Before any S-140 implementation starts, the task ledger must be recalibrated
under the current RRI script output and the active local-role boundaries:

- Low tasks (`RRI 0-25`) are local-owned under the existing Low-band path:
  primary-agent direct execution only when the workflow requires it, or Gemma
  Developer for eligible simple code patches.
- Moderate tasks (`RRI 26-40`) are local-owned through the local-first
  implementer route (`scripts/local-agent/run_local_task.py`, default
  `qwen3.6:35b-a3b`) after explicit approval.
- The Codex coordination role is present only for Med-high and Complex tasks
  (`RRI 41-70`): compute/present RRI, route cross-vendor/D14 review, decide
  whether the task must be decomposed again, assemble any local handoff packet
  that remains allowed, enforce scope, and close status artifacts.
- High or above (`RRI 71+`) must not be treated as routine S-140 local-control
  work. Decompose or escalate under the RRI policy before assigning execution.

Owner clarification on 2026-07-21: "you only in Med-high and Complex." For this
S-140 goal, that means Codex does not coordinate Low/Moderate implementation
beyond the minimum workflow gates needed to let local roles proceed.

Preliminary reruns of `scripts/rri.py` on 2026-07-21 showed that broad T1/T5
bundles can score as Complex once migration paths are included. This makes the
current table unsafe as an implementation handoff table. The next S-140 planning
step must split those bundles into narrower execution tasks before any local
handoff packet is prepared.

The execution-card backlog above is the current recalibration target. Do not use
the former broad T1/T2/T3/T5 cards as implementation handoffs.

## Pipeline context

```
S-130 TranscriptionStatus::Ready → [T2 enqueue] → SubtitleJob
    → [T3 worker-runner] → segmentation (D1a Rust heuristic)
    → subtitle.json artifact
    → [T3 persist] → Subtitle artifact (lineage → WordAlignment)
    → SubtitleStatus::Ready
    → [T5] → S-160 review task enqueued via existing ADR-030 publication gate
    → (unblocks S-170 planning; X-S-160-3 remains open — see Open questions/risks)
```

## Open questions / risks

| Risk | Disposition |
|------|-------------|
| D1 segmentation source | **Resolved** — ratified D1a on 2026-07-21; T4 (D1b worker) removed from the ledger |
| D2 canonical schema | **Resolved** — ratified the proposed internal JSON schema on 2026-07-21; implemented in T1c's acceptance criteria |
| Multilingual subtitles | **Not a risk** — explicitly out of scope, owned by S-150 per roadmap pipeline |
| S-160/S-170 dependency framing | S-160 is done; only X-S-160-3 and full S-170 depend on S-140 — do not reintroduce the corrected "S-160 blocked" framing in task text |
| ADR-030 compliance | Must reuse the existing gate; any new/parallel review path is a fail-closed violation, not a design option |
| X-S-160-3 closure requires a `review_tasks` schema change | `review_tasks` has no artifact-identity column today (`infra/migrations/0014_create_review_tasks.sql`); T5 must decide and scope that change explicitly, not assume the existing gate already carries derived-artifact identity |
