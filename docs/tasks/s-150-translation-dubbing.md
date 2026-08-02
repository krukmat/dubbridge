---
type: TaskList
title: "S-150 Translation and Dubbing"
status: planned
slice: S-150
plan: docs/plan/s-150-translation-dubbing.md
Behavioral coverage contract: unit-v1
---
# S-150 Translation and Dubbing

> **Status:** Planned 2026-08-02. S-150-T0, S-150-T1a, and S-150-T1b are
> complete; the slice now has ratified artifact boundaries, product-code domain
> kinds/status types, and the matching per-target migration layer for
> translation/dubbing status storage plus the full S-150 artifact-kind set. The
> plan-review conditions recorded for this slice remain in force, especially the
> durable S-140/S-150 route marker, deterministic initial generation-request
> derivation, and deferred ADR-028 ownership seam for TTS. Tasks T5, T6, and
> future follow-up T8 remain non-executable parent requirements whose
> provisional RRI requires decomposition before implementation.
> **Plan:** `docs/plan/s-150-translation-dubbing.md`.
> **Behavioral coverage contract:** unit-v1.

## Slice execution contract

- Recompute RRI with `scripts/rri.py` for the exact paths before presenting or
  delegating every task. Provisional scores below are not reusable approvals.
- Development tasks must map every approved `HP-#` and `EC-#` to passing unit
  tests before closure.
- Python worker URIs are temporary transport. Only Rust orchestration owns
  storage keys, checksums, PostgreSQL writes, readiness, and governance gates.
- Generation redelivery preserves the initiating `generation_request_id`; an
  intentional regeneration creates a new ID. Reuse with different operation/source
  facts fails closed.
- Initial translation fan-out derives its request ID from the exact persisted
  subtitle artifact under the fixed `S150_INITIAL_TRANSLATION_NAMESPACE`; explicit
  regeneration never uses that derivation path.
- The serialized `SubtitleJob.post_ready_route` is the sole S-140/S-150 branch
  discriminator. Missing fields decode as legacy for queued-job compatibility;
  unknown values fail closed.
- TTS implementation cannot start before T4 resolves the app-neutral ADR-028
  consent seam and decomposes the Complex parent.
- T2 suppresses legacy null-bound review enqueue only for the S-150 route. T6 owns
  the generation-aware cutover/backfill and retirement of the compatibility path.
- T6 cannot execute as a single task: RRI 71 requires decomposition and human
  review of the eventual diff.
- Stop after each task and synchronize this ledger plus every named status
  artifact before reporting completion.

---

## S-150-T0: Open slice plan + task ledger and ratify artifact boundaries

**Type:** planning/task-ledger-only
**Effort:** S (RRI 23 — Low)
**Depends on:** S-140 (closed), S-110 consent precondition (closed)
**Status:** [x] Done 2026-08-01

**Objective:** Create the canonical S-150 plan/ledger and freeze the artifact,
localization-unit, lineage, versioning, transport, consent, and review boundaries
before any implementation task is presented.

**Acceptance criteria:**

- [x] The plan and task ledger exist and link to each other.
- [x] The localization unit is ratified as
  `(project_id, asset_id, target_language_id)`.
- [x] Persisted artifact kinds and the immediate-parent chain are named.
- [x] Worker URIs are classified as temporary transport rather than storage or DB
  identity.
- [x] The current S-140 bare-array payload drift is recorded and normalized without
  rewriting immutable source artifacts.
- [x] Regeneration and idempotency preserve immutable prior generations.
- [x] Consent enforcement remains in Rust with enqueue-time and dispatch-time
  checks; the current app-layer ownership mismatch is an explicit T4 gate.
- [x] Review tasks are required to bind exact artifact versions; S-150 owns the
  remaining `X-S-160-3` follow-up.
- [x] The roadmap links the new plan/ledger and accurately says implementation has
  not started.

**Files changed:**

- `docs/plan/s-150-translation-dubbing.md`
- `docs/tasks/s-150-translation-dubbing.md`
- `docs/plan/roadmap.md`

**Evidence to emit:** Full RRI output, ratified D1–D7 decisions, task ordering,
and docs QA output.

**Status artifacts affected:** This ledger, the S-150 plan, and
`docs/plan/roadmap.md`.

**Review exemptions:**

- Task-analysis review: n/a — planning/task-ledger-only exemption.
- Code-solution review: n/a — no product code, config, schema, or migration changed.
- Reflection/unit coverage/owner final verification: n/a — not a development task.

### RRI evidence

Command:

```bash
python3 scripts/rri.py --C 0 --T 0 --A 1 --X 3 --D 0 --K 0 --P 0 \
  --touches docs/plan/s-150-translation-dubbing.md \
  --touches docs/tasks/s-150-translation-dubbing.md \
  --touches docs/plan/roadmap.md \
  --penalty arch_decision --platform dubbridge
```

```text
Platform: dubbridge
C=0, F=2, D=0, T=0, A=1, K=0, P=0, X=3
Base value: 11
Penalties applied: arch_decision (+12)
Final RRI: 23 -> Low (0-25) -> Effort S
Decomposition: not triggered
```

**Decision evidence:** Plan decisions D1–D7 ratify four new artifact kinds,
single-immediate-parent lineage through an ordered dubbing manifest, per-target
status identity, immutable generations, Rust-owned persistence/governance, two
consent checks with allowed/denied audit evidence, deterministic normalization of
legacy S-140 segment arrays, deterministic initial request identity, a durable
versioned S-140/S-150 route discriminator, and exact review-version binding. The
2026-08-02 second audit reported no High findings and these two resolved Medium
conditions.

**Agent handoff prompt:** Planning-only. Open the S-150 plan and ledger, ratify
artifact boundaries, sync the roadmap, run docs QA, and stop before T1a.

**Stop condition:** Stop after docs QA and status synchronization. Do not start
S-150-T1a or edit product code.

---

## S-150-T1a: Domain artifact kinds and localization status types

**Type:** development
**Effort:** S (provisional RRI 23 — Low; recompute before execution)
**Depends on:** S-150-T0
**Status:** [x] Done 2026-08-01

**Happy paths considered:**

- **HP-1:** All four ratified S-150 artifact kinds round-trip through domain
  display/parse boundaries using the exact stored strings from D2.
- **HP-2:** Translation and dubbing statuses represent Pending, InProgress, Ready,
  and Failed for one exact localization unit.

**Edge cases considered:**

- **EC-1:** An unknown localization status fails closed at the new status decode
  boundary; the existing lenient `parse_artifact_kind` fallback remains unchanged.
- **EC-2:** A status record cannot omit its `project_id`, `asset_id`, or
  `target_language_id` identity.

**Acceptance criteria:** Implement only domain types; add the four exact known-kind
round trips without changing the legacy unknown-kind fallback; add strict status
decoding; cover HP-1/HP-2/EC-1/EC-2 with unit tests. Strict decoding of unknown
stored artifact kinds belongs to T1c's repository boundary, not this task.

**Files expected to change:** `crates/domain/src/artifact.rs` (split into a new
module first if the exact task hits the local target-file size gate).

**Evidence to emit:** Exact RRI output, implementation/review receipt, named unit
tests, and verification output.

**Status artifacts affected:** This ledger.

**Agent handoff prompt:** Add only the ratified S-150 artifact/status domain types
and unit tests; stop before migrations, repositories, queues, or workers.

**Stop condition:** Stop after domain tests pass. Do not start T1b.

### RRI evidence

Command:

```bash
python3 scripts/rri.py --cc 4 --D 0 --K 0 --P 0 --T 1 --A 1 --X 1 \
  --touches crates/domain/src/artifact.rs \
  --touches crates/domain/Cargo.toml \
  --platform dubbridge
```

```text
Platform: dubbridge
C=0, F=1, D=2, T=1, A=1, K=2, P=2, X=1
Base value: 24
Penalties applied: none
Final RRI: 24 -> Low (0-25) -> Effort S
Decomposition: not triggered
```

### Closure note

- Added the four ratified S-150 artifact kinds to `ArtifactKind` and preserved
  the existing lenient `parse_artifact_kind` fallback for legacy callers.
- Added strict `TranslationStatus` / `DubbingStatus` decode boundaries plus
  per-localization status records keyed by `project_id`, `asset_id`, and
  `target_language_id`.
- Added unit coverage for round-trips, fail-closed unknown statuses, and
  missing-identity deserialization failures at the status-record boundary.

### Gemma Reviewer evidence

- Model: local Gemma reviewer via `make qa-gemma-review`
- Command: `GEMMA_REVIEW_BASE=HEAD REVIEW_PATHS='crates/domain/src/artifact.rs crates/domain/Cargo.toml' GEMMA_REVIEW_TASK_ID=S-150-T1a make qa-gemma-review`
- Passes run / usable: `3/3`
- Aggregate status: `FINDINGS`
- Consensus findings: `0` | Pass-specific: `2` | Disagreement: `0`
- Artifacts: `/tmp/dubbridge-gemma-review.json`, `/tmp/dubbridge-gemma-review.pass1.json`, `/tmp/dubbridge-gemma-review.pass2.json`, `/tmp/dubbridge-gemma-review.pass3.json`
- Isolated adjudicator: `not triggered` — trigger: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: accepted both minor findings as non-blocking. The
  `parse_artifact_kind` fallback is intentionally preserved by T1a contract and
  strict artifact-kind rejection is deferred to T1c; the `now_utc()` constructors
  are acceptable for current scope because no time-based branching or assertions
  depend on injectable clocks here.
- Review artifact: `docs/audit/gemma-evidence/S-150-T1a.json`

### Reflection log

Required passes: 0 (`RRI 24` -> `Low`)

- **Draft verdict:** Domain-only implementation complete and scoped correctly to
  `ArtifactKind`, localization statuses, and status-record identity.
- **Critique findings:**
  - Gemma flagged the lenient `parse_artifact_kind` fallback, but T1a
    explicitly requires preserving that legacy behavior while introducing strict
    fail-closed status parsing only.
  - Gemma noted `now_utc()` constructors may limit future deterministic clock
    assertions; current tests assert identity and decode boundaries only, so no
    change is needed in this task.
- **Revisions applied:** none. The critique confirmed the implementation matches
  the T1a boundary and defers strict artifact-kind rejection to T1c as planned.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | All four ratified S-150 artifact kinds round-trip through domain display/parse boundaries using the exact stored strings from D2 | `crates/domain/src/artifact.rs::parse_translated_subtitle`; `crates/domain/src/artifact.rs::parse_dubbed_audio_segment`; `crates/domain/src/artifact.rs::parse_dubbing_manifest`; `crates/domain/src/artifact.rs::parse_dubbed_audio`; `crates/domain/src/artifact.rs::artifact_kind_display_translation_variants` | passed |
| HP-2 | Happy path | Translation and dubbing statuses represent Pending, InProgress, Ready, and Failed for one exact localization unit | `crates/domain/src/artifact.rs::translation_status_parse_known_variants_succeeds`; `crates/domain/src/artifact.rs::dubbing_status_parse_known_variants_succeeds`; `crates/domain/src/artifact.rs::translation_status_record_new_carries_localization_unit_identity`; `crates/domain/src/artifact.rs::dubbing_status_record_new_carries_localization_unit_identity` | passed |
| EC-1 | Edge case | Unknown localization status fails closed at the new status decode boundary; the existing lenient `parse_artifact_kind` fallback remains unchanged | `crates/domain/src/artifact.rs::translation_status_unknown_value_fails_closed`; `crates/domain/src/artifact.rs::dubbing_status_unknown_value_fails_closed`; `crates/domain/src/artifact.rs::parse_unknown_falls_back_to_original_media` | passed |
| EC-2 | Edge case | A status record cannot omit its `project_id`, `asset_id`, or `target_language_id` identity | `crates/domain/src/artifact.rs::translation_status_record_new_carries_localization_unit_identity`; `crates/domain/src/artifact.rs::dubbing_status_record_new_carries_localization_unit_identity`; `crates/domain/src/artifact.rs::translation_status_record_missing_identity_fields_fail_to_deserialize` | passed |

### Owner final verification

- Owner: `Codex agent`
- Date: 2026-08-01
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-domain artifact -- --nocapture`; `GEMMA_REVIEW_BASE=HEAD REVIEW_PATHS='crates/domain/src/artifact.rs crates/domain/Cargo.toml' GEMMA_REVIEW_TASK_ID=S-150-T1a make qa-gemma-review`

---

## S-150-T1b: Per-target status and artifact-kind migration

**Type:** migration-only
**Effort:** L (RRI 52 — Med-high; recomputed 2026-08-02)
**Depends on:** S-150-T1a
**Status:** [x] Done 2026-08-02
**Scheduled work slot:** next 07:00 Europe/Madrid, by operator request. This is
an execution-planning note only; no OS, calendar, or CI automation is attached.

**Behavioral examples:**

- **HP-1:** Two target languages for one asset can hold independent translation
  and dubbing status rows.
- **EC-1:** Duplicate rows for the same localization unit and unknown status text
  are rejected by PostgreSQL.
- **EC-2:** Artifact kinds outside the complete known set remain rejected.

**Acceptance criteria:** Add per-target translation/dubbing status storage and the
four D2 artifact-kind literals without weakening existing checks. Rebuild the
`artifact_kind_check` constraint from the exhaustive current domain literal set —
including existing `recorded_stream_media` and `downloaded_platform_media` — rather
than copying migration 0023's incomplete list; validate against a fresh migrated
PostgreSQL database.

**Files expected to change:** One or more next-numbered files under
`infra/migrations/` after exact decomposition.

**Evidence to emit:** Exact RRI report, phase-1/phase-2 exemption lines for
migration-only work, schema inspection, and migration verification output.

**Status artifacts affected:** This ledger and migration inventory references if
the final task creates them.

**Agent handoff prompt:** Add only the per-target localization status/artifact-kind
migration and live-Postgres verification; stop before repository code.

**Stop condition:** Stop after migration verification. Do not start T1c.

### RRI evidence

Artifact: `docs/audit/s-150-t1b-rri.md`

- Final RRI: `52`
- Band: `Med-high (41-55)`
- Effort: `L`
- Decomposition: `not triggered`

### Closure note

- Added `infra/migrations/0027_create_translation_dubbing_status_and_extend_artifact_kind_check.sql`
  with two per-target status tables: `asset_translation_status` and
  `asset_dubbing_status`.
- Keyed both tables by the exact localization unit
  `(project_id, asset_id, target_language_id)` and enforced relational scope via
  composite foreign keys to `project_assets` and `target_languages`.
- Rebuilt `artifact_kind_check` from the exhaustive current domain literal set,
  preserving existing `recorded_stream_media` and `downloaded_platform_media`
  while adding `translated_subtitle`, `dubbed_audio_segment`,
  `dubbing_manifest`, and `dubbed_audio`.

### Migration verification

- Fresh database: `s150_t1b_verify` on PostgreSQL 16 inside `local-postgres-1`
- Migration apply: all migrations `0001` through `0027` applied in order with
  `psql -v ON_ERROR_STOP=1`
- Schema inspection:
  - `information_schema.columns` confirmed both new tables expose
    `project_id`, `asset_id`, `target_language_id`, `status`, `error_detail`,
    and `updated_at DEFAULT now()`
  - `pg_get_constraintdef` confirmed:
    - primary key on `(project_id, asset_id, target_language_id)` for both tables
    - closed status checks over `pending`, `in_progress`, `ready`, `failed`
    - composite foreign keys to `project_assets(project_id, asset_id)` and
      `target_languages(id, project_id)`
    - `artifact_kind_check` closed over the full 13-kind set
- Behavioral verification:
  - `HP-1`: inserting two target-language rows for one asset succeeded in both
    status tables (`2` translation rows, `2` dubbing rows)
  - `EC-1`: duplicate localization-unit insert failed with
    `asset_translation_status_pk`; unknown status text failed with
    `asset_dubbing_status_check`
  - `EC-2`: inserting `artifact_records.kind = 'bogus_kind'` failed with
    `artifact_kind_check`

- Task-analysis review: `n/a` - migration-only exemption
- Code-solution review: `n/a` - migration-only exemption

### Owner final verification

- Owner: `Codex agent`
- Date: 2026-08-02
- Statement: I verified the migration on a fresh PostgreSQL database and confirmed the task's happy path and edge cases through schema inspection and failing-row checks.
- Commands run: `docker-compose -f infra/local/docker-compose.yml up -d postgres`; `docker-compose -f infra/local/docker-compose.yml exec -T postgres psql -U dubbridge -d postgres -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS s150_t1b_verify;"`; `docker-compose -f infra/local/docker-compose.yml exec -T postgres psql -U dubbridge -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE s150_t1b_verify;"`; `for f in infra/migrations/*.sql; do docker-compose -f infra/local/docker-compose.yml exec -T postgres psql -U dubbridge -d s150_t1b_verify -v ON_ERROR_STOP=1 < "$f"; done`; schema inspection and behavioral `psql` inserts against `s150_t1b_verify`

---

## S-150-T1c: Translation/dubbing repositories and immutable generation pointers

**Type:** development
**Effort:** L (provisional RRI 47 — Med-high; recompute and split if necessary)
**Depends on:** S-150-T1b
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** Each localization unit independently transitions status and resolves
  its exact current source/output artifact IDs.
- **HP-2:** A completed regeneration atomically advances current pointers while
  older immutable artifacts remain queryable.

**Edge cases considered:**

- **EC-1:** A pointer to an artifact belonging to another asset or wrong kind is
  rejected.
- **EC-2:** Partial artifact sets cannot transition translation or dubbing to
  Ready.
- **EC-3:** A stale generation cannot overwrite the current generation's pointers.
- **EC-4:** An unknown stored artifact kind fails closed at the repository's strict
  decode boundary even though the legacy domain parser remains lenient.
- **EC-5:** Reusing a `generation_request_id` with a different operation or source
  artifact is rejected rather than claiming or mutating the existing generation.
- **EC-6:** An explicit regeneration cannot claim the deterministic request ID
  reserved for the initial translation of its source subtitle.

**Acceptance criteria:** Implement fail-closed repositories for per-target state,
generation claims, exact artifact pointers, and readiness evidence. Enforce one
atomic generation claim per `(operation, localization unit, generation_request_id)`
and persist its exact source artifact; reject explicit-regeneration use of the
reserved initial ID; cover every case with unit/integration tests.

**Files expected to change:** `crates/db/src/translation_repo.rs`,
`crates/db/src/dubbing_repo.rs`, `crates/db/src/artifact_repo.rs`,
`crates/db/src/lib.rs`, and scoped tests.

**Evidence to emit:** Exact RRI, reviewer artifact, Reflection log, unit coverage
certification, Postgres test output, and owner verification.

**Status artifacts affected:** This ledger.

**Agent handoff prompt:** Implement only localization repositories and readiness
evidence for the ratified schema; stop before queues and workers.

**Stop condition:** Stop after repository tests and closure gates. Do not start T2.

---

## S-150-T2: Translation fan-out job contract and S-140 handoff

**Type:** development
**Effort:** L (provisional RRI 50 — Med-high; recompute before presentation)
**Depends on:** S-150-T1c
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** One source `Subtitle` readiness event enqueues exactly one translation
  job for every configured `target_language_id`.
- **HP-2:** Re-delivery of the same readiness event does not duplicate an active
  generation: it resolves the same persisted subtitle artifact and derives the
  same `generation_request_id`.
- **HP-3:** A pre-existing serialized S-140 `SubtitleJob` without
  `post_ready_route` decodes as `legacy_subtitle_review_v1` and preserves the
  existing subtitle-only review enqueue.
- **HP-4:** A newly adopted S-150 subtitle job carries
  `s150_localization_v1`; all target fan-out jobs derive the same initial request
  ID from the exact persisted subtitle artifact while retaining independent
  localization-unit claims.

**Edge cases considered:**

- **EC-1:** No configured target languages fails observably without inventing a
  default route.
- **EC-2:** One target's queue failure marks only that localization unit Failed and
  does not corrupt successful sibling targets.
- **EC-3:** The legacy first-target-only selection and premature null-artifact
  review enqueue are not used for the full localization route.
- **EC-4:** An explicit regeneration uses a new `generation_request_id`; a
  redelivery that changes the source under an existing request ID fails closed.
- **EC-5:** An unknown `post_ready_route` value fails job deserialization instead
  of guessing a route.
- **EC-6:** An S-150 route never invokes the legacy null-artifact review enqueue;
  a legacy route never silently fans out translation work.

**Acceptance criteria:** Add the translation job/queue and deterministic all-target
fan-out from the exact S-140 subtitle artifact. Add the versioned
`SubtitleJob.post_ready_route` enum with exact wire values
`legacy_subtitle_review_v1` and `s150_localization_v1`; keep
`SubtitleJob::new` explicitly legacy, default an absent serialized field to legacy,
and reject unknown values. Add an explicit localization constructor and use it at
the T2 cutover seam. Branch only on that field after subtitle readiness: legacy
calls the existing review enqueue, while localization resolves every target,
derives `UUIDv5(S150_INITIAL_TRANSLATION_NAMESPACE,
"initial-translation-v1:" || canonical_lowercase_subtitle_artifact_uuid)`,
propagates it to every target job, and never calls the legacy review enqueue. On
post-ready replay, resolve the existing subtitle by its exact asset/word-alignment
parent uniqueness boundary and reuse its artifact ID rather than creating a
replacement. Explicit regeneration uses a caller-minted UUIDv4 and cannot invoke
the initial derivation. Preserve upstream subtitle readiness and cover
HP-1–HP-4/EC-1–EC-6, including characterization of old JSON. Do not delete the
subtitle-only compatibility path or create an S-150 review row before T6.

**Files expected to change:** workspace UUID feature configuration if UUIDv5 is not
already enabled, `crates/jobs`, scoped `apps/worker-runner` enqueue/runtime
modules/tests, `crates/db/src/subtitle_repo.rs` for exact replay resolution, and
only the minimum S-140 seam required. Because `subtitle_runtime.rs` and
`workspace_repo.rs` currently exceed the 500-line local-read gate, the exact T2
presentation must either split/extract the required seams into focused files first
or route implementation to cloud with the required evidence; it must not delegate
those files as-is to the local implementer.

**Evidence to emit:** Exact RRI, route receipt, phase reviews, Reflection log, unit
coverage certification, and Redis/in-memory queue evidence.

**Status artifacts affected:** This ledger and the S-140 plan/ledger only if their
delivered handoff wording becomes materially stale.

**Agent handoff prompt:** Add the versioned post-ready discriminator, deterministic
initial request ID, and per-target translation fan-out while preserving explicit
legacy behavior; stop before provider execution.

**Stop condition:** Stop after enqueue/idempotency tests. Do not start T3a.

---

## S-150-T3a: Translation provider/subprocess contract

**Type:** development
**Effort:** L (provisional RRI 42 — Med-high; recompute before presentation)
**Depends on:** S-150-T2
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** A typed Rust client sends a versioned source subtitle and receives a
  translated payload preserving every segment ID and timing boundary.
- **HP-2:** The JSON schemas reject extra fields and express the D3 canonical
  payload contract.
- **HP-3:** A current S-140 bare segment array is normalized with deterministic
  IDs derived from `(subtitle_artifact_id, ordinal)` and the relational source
  language, without rewriting the source artifact.

**Edge cases considered:**

- **EC-1:** Missing, duplicated, reordered-without-identity, or timing-mutated
  segments are rejected before persistence.
- **EC-2:** Non-zero exit, malformed JSON, missing output file, or non-file URI
  returns a typed provider error.
- **EC-3:** A legacy S-140 array with invalid/overlapping timing or an unresolved
  source language is rejected before worker dispatch.

**Acceptance criteria:** Extract a focused provider module if required by the
500-line local read gate; update schemas before runtime code depends on them; test
the contract without a real translation model.

**Files expected to change:** `crates/providers/src/translation.rs`, provider module
wiring, and `workers/translation-worker-py/*schema.json`/README.

**Evidence to emit:** Exact RRI, schema validation, phase reviews, Reflection log,
unit coverage certification, and owner verification.

**Status artifacts affected:** This ledger and worker contract README.

**Agent handoff prompt:** Implement only the typed translation client and versioned
schema contract with deterministic stubs; stop before the real Python worker/runtime.

**Stop condition:** Stop after provider/schema tests. Do not start T3b.

---

## S-150-T3b: Functional translation worker

**Type:** development
**Effort:** L (provisional RRI 44 — Med-high; recompute before presentation)
**Depends on:** S-150-T3a
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** Valid versioned input produces one translated segment for each source
  segment with identity/timing preserved.

**Edge cases considered:**

- **EC-1:** Provider failure emits the error-schema payload and exits non-zero.
- **EC-2:** Unsupported language or invalid source payload produces no success
  artifact.

**Acceptance criteria:** Implement the Python stdin/stdout worker behind a
configurable provider; keep model credentials in injected environment only; use a
deterministic fake in tests.

**Files expected to change:** `workers/translation-worker-py/main.py`, dependency
metadata, Dockerfile, README, and tests.

**Evidence to emit:** Exact RRI, phase reviews, Reflection log, unit coverage,
Python tests, and container/contract check.

**Status artifacts affected:** This ledger and worker README.

**Agent handoff prompt:** Implement only the translation worker/provider adapter and
deterministic tests; stop before Rust persistence.

**Stop condition:** Stop after Python contract tests. Do not start T3c.

---

## S-150-T3c: Translation runtime persistence and readiness

**Type:** development
**Effort:** L (provisional RRI 53 — Med-high; recompute before presentation)
**Depends on:** S-150-T3b
**Status:** [ ] Planned

**Happy paths considered:**

- **HP-1:** A valid worker result is uploaded under a generation-scoped key,
  checksummed, inserted as `TranslatedSubtitle`, and marks only its localization
  unit Ready.

**Edge cases considered:**

- **EC-1:** Worker, validation, storage, or DB failure leaves the unit non-Ready and
  records observable failure detail.
- **EC-2:** A stale/replayed job cannot overwrite the current generation.
- **EC-3:** Worker-local URI text is never persisted as the canonical storage key.

**Acceptance criteria:** Implement the Rust translation consumer, D4 normalization,
immutable artifact persistence, lineage, and evidence-backed Ready transition.

**Files expected to change:** Scoped worker-runner translation modules, translation
repository/storage helpers, and tests.

**Evidence to emit:** Exact RRI, signed implementation audit, phase reviews,
Reflection log, unit coverage, real storage/Postgres tests, and owner verification.

**Status artifacts affected:** This ledger.

**Agent handoff prompt:** Implement only translation consumption, persistence,
lineage, and readiness; stop before consent/TTS work.

**Stop condition:** Stop after translation runtime closure. Do not start T4.

---

## S-150-T4: Amend ADR-028 ownership seam and decompose TTS

**Type:** ADR/planning-only
**Effort:** M (provisional RRI 26 — Moderate; recompute before presentation)
**Depends on:** S-150-T3c
**Status:** [ ] Planned

**Objective:** Resolve the current mismatch between ADR-028's reusable consent rule
and its app-owned implementation, then create executable child tasks for T5 whose
exact RRI is below the decomposition target.

**Acceptance criteria:**

- ADR-028 names an app-neutral Rust owner callable by API and worker-runner without
  a dependency between binaries.
- ADR-028 corrects its stale implementation reference from
  `apps/api/src/services/consent_gate.rs` to the current
  `apps/api/src/consent_gate.rs` before documenting the app-neutral destination.
- The decision preserves durable denial audit behavior and both enqueue/dispatch
  checks from D6, and reconciles ADR-028's requirement to audit allowed as well as
  denied checks.
- ADR frontmatter/index/canonical prose are propagated under the ADR change contract.
- T5 is replaced or expanded with ordered child tasks, exact paths, HP/EC cases,
  verification, evidence/status artifacts, and fresh scripted RRI output.

**Evidence to emit:** ADR propagation checklist, `make qa-docs`, exact RRI outputs,
and decomposed child-task cards.

**Status artifacts affected:** ADR-028, `docs/adr/README.md`, architecture/roadmap if
the ownership boundary changes their prose, this plan, and this ledger.

**Agent handoff prompt:** Decide and propagate the app-neutral consent seam, then
decompose T5; stop before any consent or TTS code edit.

**Stop condition:** Stop after ADR/docs QA and executable T5 child definitions. Do
not implement them.

---

## S-150-T5: TTS/dubbing implementation parent — decomposition required

**Type:** development parent (not executable as written)
**Effort:** L (provisional RRI 68–70 — Complex)
**Depends on:** S-150-T4
**Status:** [ ] Blocked on mandatory decomposition by T4

**Happy paths considered:**

- **HP-1:** Active exact-scope consent at enqueue and dispatch produces immutable
  segment artifacts, an ordered manifest, and a merged dubbed-audio artifact.
- **HP-2:** Complete evidence advances only the exact localization generation to
  Ready.
- **HP-3:** Both successful consent checks emit durable, correlated allowed-check
  audit evidence before synthesis proceeds.
- **HP-4:** Re-delivery with the same `generation_request_id` resumes the same TTS
  generation instead of creating duplicate segment or merged-audio artifacts.

**Edge cases considered:**

- **EC-1:** Missing/revoked/mismatched/unreadable consent produces no synthesis
  bytes and emits durable denial evidence.
- **EC-2:** Revocation after enqueue but before dispatch is caught by the second
  check.
- **EC-3:** Partial segment, manifest, merge, storage, or DB failure never marks
  Ready and never overwrites a prior generation.
- **EC-4:** Malformed segment ordering/timing or a manifest referencing the wrong
  generation is rejected.
- **EC-5:** Reusing a request ID with different operation/source facts fails closed.

**Acceptance criteria:** T4 must replace this parent with independently executable
children covering job/queue, consent enforcement, provider contract, Python worker,
segment persistence, manifest/merge persistence, generation-request propagation,
readiness, and Redis topology.

**Evidence to emit:** Child-task-specific RRI/review/Reflection/coverage/owner
verification plus durable consent and artifact-lineage evidence.

**Status artifacts affected:** This plan/ledger, ADR-028 implementation references,
and X11/X24 language when runtime enforcement is genuinely delivered.

**Agent handoff prompt:** Do not implement this parent. Execute only the approved
child task produced by T4 and stop at its boundary.

**Stop condition:** Parent cannot be marked Done; close only after every decomposed
child passes its own gates.

---

## S-150-T6: Exact review artifact/version binding — decomposition required

**Type:** development parent (not executable as written)
**Effort:** XL (provisional RRI 71 — High)
**Depends on:** all decomposed S-150-T5 children
**Status:** [ ] Blocked on mandatory decomposition and human diff review

**Happy paths considered:**

- **HP-1:** A completed localization generation creates one ADR-030 review unit
  bound by role to its exact `TranslatedSubtitle` and `DubbedAudio` IDs.
- **HP-2:** Regeneration creates a distinct review unit; prior decisions remain
  attached only to the old artifact set.
- **HP-3:** A complete S-150 generation accumulated before T6 cutover receives one
  exact-bound review unit during the controlled backfill/enqueue pass.

**Edge cases considered:**

- **EC-1:** Missing, cross-asset, cross-target, wrong-kind, or incomplete artifact
  bindings fail closed.
- **EC-2:** Legacy S-160/S-140 rows remain readable and cannot accidentally approve
  a new generation.
- **EC-3:** Review enqueue failure does not fabricate readiness or bypass ADR-030;
  it remains observable and retryable.

**Acceptance criteria:** Decompose schema/version identity, domain/repository/API
compatibility, enqueue wiring, and migration/backfill behavior into separately
approved tasks with characterization tests. Human reviews the resulting diffs per
the High-band gate. The decomposition must preserve legacy rows, introduce
generation-aware uniqueness and exact bindings atomically, backfill/enqueue complete
S-150 generations accumulated before cutover, and retire the compatibility path only
after the new route is live. Close `X-S-160-3` only after exact version behavior is
proven.

**Evidence to emit:** Decomposition/RRI artifacts, migration and compatibility
evidence, phase reviews, Reflection logs, unit coverage, human diff approval, and
the synchronized X-S-160-3 closure record.

**Status artifacts affected:** This plan/ledger, `docs/plan/roadmap.md`, S-140 and
S-160 plan/task prose that names the deferred seam, and ADR-030 implementation
references if materially changed.

**Agent handoff prompt:** Do not implement this parent. First create and present the
required child tasks; execute only one approved child at a time.

**Stop condition:** Do not close X-S-160-3 or start T7 until all child tasks and
compatibility evidence pass.

---

## S-150-T7: BDD and canonical docs closeout

**Type:** docs-only
**Effort:** S (provisional RRI 11 — Low; recompute before execution)
**Depends on:** all T5/T6 child tasks
**Status:** [ ] Planned

**Acceptance criteria:**

- Add `docs/bdd/s-150-translation-dubbing.feature` covering per-target translation,
  consent allow/deny/revoke race, exact artifact lineage, and version-bound review.
- Map scenarios in `docs/bdd/README.md` to real executable evidence.
- Synchronize plan, ledger, roadmap, affected upstream/downstream plans, and cross-
  cutting obligations without claiming deferred work complete.
- Run `make qa-docs` and all relevant scoped/full QA commands.

**Evidence to emit:** BDD mapping, exact command output, final status diff, and fresh
RRI.

**Status artifacts affected:** This plan/ledger, roadmap, BDD index/feature, S-140,
S-160, and any downstream blocker text materially changed by completion.

**Agent handoff prompt:** Add only evidence-backed S-150 BDD and canonical status
synchronization; stop before S-170 or S-180 planning.

**Stop condition:** Stop after docs QA and truthful slice status sync. Do not start
S-170/S-180.

---

## S-150-T8: Future voice-consent hardening and evidence lifecycle

**Type:** future ADR/planning parent (not executable as implementation)
**Effort:** XL (provisional RRI 71 — High; recompute and decompose when activated)
**Depends on:** S-150-T7 for scheduling only; coordinate with X20 and S-180
**Status:** [ ] Future follow-up — non-blocking for S-150-T1 through T7

**Objective:** Turn the voice-consent topics intentionally left outside the S-150
delivery boundary into an explicit future governance program without weakening or
reopening ADR-028's current fail-closed TTS precondition.

**Decision scenarios to preserve:**

- **HP-1:** A consent grant references durable, access-controlled proof whose
  integrity, retention, redaction, and authorized retrieval policy is explicit.
- **HP-2:** Automated real-stack tests prove allowed and denied checks at enqueue
  and dispatch against live `voice_consents` and `audit_events` storage.
- **EC-1:** Revocation after an artifact was synthesized has an explicit policy for
  review, publication, download, regeneration, and provider-side/voice-profile
  disablement; it is not silently treated as only a future-request concern.
- **EC-2:** Multi-speaker assets, speaker identity, voice-profile identity, scope
  evolution, delegated grant authority, and consent expiry cannot alias an
  asset-level consent accidentally.
- **EC-3:** Missing/unreadable proof, audit-write failure, evidence-store outage, or
  provider deletion uncertainty remains fail-closed and observable.

**Acceptance criteria:**

- Resolve `X-S-110-2` with the X20 owner-credential/secrets-store decision while
  keeping evidence bytes and secrets out of PostgreSQL and logs.
- Reconcile the current `X-S-110-3` wording drift (ADR-028 names live
  `voice_consents`; the S-110 plan names live compliance reads), then automate
  live-Postgres coverage for the consent/compliance paths and both S-150 gate
  positions, including durable allow/deny audit evidence.
- Decide whether ADR-028's `(asset, scope)` identity remains sufficient for
  multi-speaker/voice-profile use cases; any semantic change must amend or supersede
  the ADR and propagate every canonical reference.
- Define the effect of revocation/expiry on already-produced dubbed artifacts and
  the S-180 publication/download gate, plus retention/deletion responsibilities for
  external proof and provider-side voice material.
- Produce a threat/privacy analysis, then decompose all implementation into
  separately scored tasks with HP/EC cases, evidence, status artifacts, and explicit
  production-readiness gates.
- Do not make this future hardening a retroactive blocker for T1–T7 unless the owner
  explicitly promotes one resolved risk into the S-150 critical path.

**Files expected to change when activated:** ADR-028 (or a successor), ADR index,
S-110/S-150 plans and ledgers, roadmap, S-180 plan/tasks, and narrowly scoped future
implementation ledgers. No product-code path is authorized by this parent.

**Evidence to emit:** Threat/privacy analysis, decision matrix, ADR propagation
checklist, automated real-stack verification design, exact child-task RRI reports,
and explicit disposition of `X-S-110-2`/`X-S-110-3`/X20 dependencies.

**Status artifacts affected:** This plan/ledger, ADR-028 and index, S-110 plan/task
follow-ups, roadmap X20/X-S-150-1, and future S-180 plan/task status.

### Provisional RRI evidence

```text
Platform: dubbridge
C=0, F=3, D=5, T=0, A=2, K=3, P=5, X=4
Base value: 49
Penalties applied: arch_decision (+12); auth_security (+10)
Final RRI: 71 -> High (71-85) -> Effort XL
Decomposition: triggered by RRI >= 56
```

**Agent handoff prompt:** Recompute RRI, prepare the High-band planning/ADR card,
decompose the governance decisions before any implementation, and stop after the
approved decision artifacts and child ledgers.

**Stop condition:** Do not execute this parent, edit product code, or reopen
ADR-028/X11 by implication. Stop after approved decomposition and canonical status
synchronization.
