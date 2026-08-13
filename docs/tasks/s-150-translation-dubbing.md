---
type: TaskList
title: "S-150 Translation and Dubbing"
status: planned
slice: S-150
plan: docs/plan/s-150-translation-dubbing.md
Behavioral coverage contract: unit-v1
---
# S-150 Translation and Dubbing

> **Status:** Planned 2026-08-02. S-150-T0, S-150-T1a, S-150-T1b, and
> S-150-T1c-i are
> complete; the slice now has ratified artifact boundaries, product-code domain
> kinds/status types, and the matching per-target migration layer for
> translation/dubbing status storage, exact current-generation pointer/claim
> storage, plus the full S-150 artifact-kind set. The
> exact rerun of the original `S-150-T1c` on 2026-08-02 returned `RRI 56` and
> exposed a schema gap against D1/D5, so `T1c` is now decomposed into `T1c-i`
> (generation-claim/current-pointer schema, now complete) and `T1c-ii`
> (repositories, now complete). The former T2 fan-out parent is decomposed into
> T2b-i/T2b-ii/T2c. T2b-i is complete; T2b-ii was decomposed on 2026-08-12
> into T2b-ii-a/T2b-ii-b/T2b-ii-c after an exact `RRI 57` result. All three
> children are complete as of 2026-08-13. Complex parent T2c was decomposed on
> 2026-08-13 into T2c-i through T2c-v; T2c-i is the next executable task after
> its own phase-1 review and approval. The
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

> Exact rerun on 2026-08-02 over the original repository surface
> (`crates/db/src/translation_repo.rs`, `crates/db/src/dubbing_repo.rs`,
> `crates/db/src/artifact_repo.rs`, `crates/db/src/lib.rs`) returned `RRI 56`
> (`Complex`) and the phase-1 review packet confirmed the current
> `0027_create_translation_dubbing_status_and_extend_artifact_kind_check.sql`
> schema stops short of D1/D5: it has no generation-claim storage and no exact
> current source/output pointers on the localization status rows. This parent is
> therefore decomposed into `S-150-T1c-i` and `S-150-T1c-ii`. Do not implement
> the parent directly.

**Type:** development parent (not executable as written)
**Effort:** L (rerun RRI 56 — Complex)
**Depends on:** S-150-T1b
**Status:** [ ] Decomposed 2026-08-02 into S-150-T1c-i + S-150-T1c-ii

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

**Acceptance criteria:** Close this parent only through `S-150-T1c-i` and
`S-150-T1c-ii`. The first child must add the exact current-pointer/generation-claim
schema needed by D1/D5; the second must implement the fail-closed repositories and
readiness evidence over that schema, including the HP/EC cases above.

**Evidence to emit:** Exact RRI rerun, phase-1 review artifact, and the full
child-task-specific review/Reflection/coverage/verification evidence.

**Status artifacts affected:** This ledger, `docs/plan/s-150-translation-dubbing.md`,
and `docs/plan/roadmap.md`.

**Agent handoff prompt:** Do not implement this parent. Execute only the approved
child task and stop at its boundary.

**Stop condition:** Parent cannot be marked Done; start with `S-150-T1c-i`, then
stop before `T2`.

- Task-analysis review: `d14` `.agent/peer-task-review-s-150-t1c.json` - `BLOCKED`

---

## S-150-T1c-i: Generation-claim and exact-pointer schema migration

**Type:** migration-only
**Effort:** L (recomputed RRI 52 — Med-high)
**Depends on:** S-150-T1b
**Status:** [x] Done 2026-08-02

**Behavioral examples:**

- **HP-1:** `asset_translation_status` can persist one localization unit's exact
  current subtitle source artifact, translated-subtitle output artifact, and the
  current `generation_request_id`.
- **HP-2:** `asset_dubbing_status` can persist one localization unit's exact
  translated-subtitle source artifact plus its current manifest/final-audio
  artifacts and `generation_request_id`.
- **EC-1:** A duplicate generation claim for the same
  `(operation, project_id, asset_id, target_language_id, generation_request_id)`
  is rejected by PostgreSQL.
- **EC-2:** Pointer/claim columns remain nullable until a generation is actually
  claimed or promoted current; the migration does not invent a backfill.
- **EC-3:** Pointer or claim rows cannot reference a missing `artifact_records.id`.

**Acceptance criteria:** Add one new forward-only migration after `0027` that
extends `asset_translation_status` and `asset_dubbing_status` with exact current
artifact pointers and `current_generation_request_id`, and adds a normalized
generation-claim table that stores `operation`, the exact localization unit, the
claimed `generation_request_id`, and its exact `source_artifact_id`. Keep the
schema provider-neutral, preserve the existing localization-unit primary keys, and
validate the full migration chain against a fresh PostgreSQL database. This task
introduces storage/constraints only; it does not add repository code.

**Files expected to change:** One new next-numbered file under `infra/migrations/`
after exact decomposition.

**Evidence to emit:** Exact RRI report, phase-1/phase-2 exemption lines for
migration-only work, schema inspection, and fresh-Postgres verification output.

**Status artifacts affected:** This ledger, `docs/plan/s-150-translation-dubbing.md`,
and migration inventory references if the final task creates them.

**Agent handoff prompt:** Add only the generation-claim/current-pointer migration
and live-Postgres verification; stop before repository code.

**Stop condition:** Stop after migration verification. Do not start `S-150-T1c-ii`.

### RRI evidence

Artifact: `docs/audit/s-150-t1c-i-rri.md`

- Final RRI: `52`
- Band: `Med-high (41-55)`
- Effort: `L`
- Decomposition: `not triggered`

- Task-analysis review: `n/a` - migration-only exemption

### Closure note

- Added [infra/migrations/0028_add_localization_generation_claims_and_exact_pointers.sql](/Users/matias/dubbridge/infra/migrations/0028_add_localization_generation_claims_and_exact_pointers.sql:1)
  to extend both localization status tables with exact current-generation pointer
  columns and to add the normalized `localization_generation_claims` table.
- Kept all new current-pointer columns nullable, so pre-existing and newly inserted
  rows can remain unclaimed/current-less without backfill until a generation is
  actually promoted.
- Added fail-closed checks so a current generation always carries a source
  artifact, and a current dubbed-audio pointer cannot exist without a current
  manifest pointer.
- Executed on the user-requested cloud path: `CLOUD_REQUIRED` for this task's
  implementation route; the ADR-038 local authoring path was intentionally not
  used.

### Migration verification

- Fresh database: `s150_t1c_i_verify` on PostgreSQL 16 inside `local-postgres-1`
- Migration apply: all migrations `0001` through `0028` applied in order with
  `psql -v ON_ERROR_STOP=1`
- Schema inspection:
  - `information_schema.columns` confirmed nullable `current_*` pointer columns on
    `asset_translation_status` and `asset_dubbing_status`, plus the full
    `localization_generation_claims` table shape
  - `pg_get_constraintdef` confirmed:
    - status-table FKs from current pointers to `artifact_records(id)` with
      `ON DELETE RESTRICT`
    - fail-closed current-pointer checks on both status tables
    - primary-key uniqueness on
      `(operation, project_id, asset_id, target_language_id, generation_request_id)`
      for `localization_generation_claims`
    - claim-table FKs to `project_assets`, `target_languages`, and
      `artifact_records`
- Behavioral verification:
  - `HP-1`: inserting a translation status row without any new pointer columns left
    all `current_*` fields `NULL`; updating that row with
    `current_generation_request_id`, exact subtitle source, and exact
    translated-subtitle output succeeded
  - `HP-2`: inserting a dubbing status row without any new pointer columns left all
    `current_*` fields `NULL`; updating that row with exact translated-subtitle
    source plus exact manifest/final-audio pointers succeeded
  - `EC-1`: inserting a second `localization_generation_claims` row for the same
    `(operation, project_id, asset_id, target_language_id, generation_request_id)`
    failed with `localization_generation_claims_pk`
  - `EC-2`: the initial inserts into both status tables succeeded with every new
    pointer/claim column omitted, proving the migration introduced no backfill
    requirement
  - `EC-3`: updating `asset_translation_status.current_source_artifact_id` to a
    missing UUID failed with
    `asset_translation_status_current_source_artifact_fk`
  - Additional fail-closed check: setting `current_dubbed_audio_artifact_id`
    without `current_manifest_artifact_id` failed with
    `asset_dubbing_status_current_pointer_check`

- Code-solution review: `n/a` - migration-only exemption

### Owner final verification

- Owner: `Codex agent`
- Date: 2026-08-02
- Statement: I verified the migration on a fresh PostgreSQL database and confirmed the task's happy paths and edge cases through schema inspection plus passing/failing row checks.
- Commands run: `docker-compose -f infra/local/docker-compose.yml up -d postgres`; `docker-compose -f infra/local/docker-compose.yml exec -T postgres psql -U dubbridge -d postgres -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS s150_t1c_i_verify;"`; `docker-compose -f infra/local/docker-compose.yml exec -T postgres psql -U dubbridge -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE s150_t1c_i_verify;"`; `for f in infra/migrations/*.sql; do docker-compose -f infra/local/docker-compose.yml exec -T postgres psql -U dubbridge -d s150_t1c_i_verify -v ON_ERROR_STOP=1 < "$f"; done`; schema inspection queries against `s150_t1c_i_verify`; behavioral `INSERT`/`UPDATE` verification and expected-failure checks against `s150_t1c_i_verify`

---

## S-150-T1c-ii: Translation/dubbing repositories and readiness evidence

> This child consumes the schema introduced by `S-150-T1c-i`. No migration is in
> scope here.

**Type:** development
**Effort:** L (recomputed RRI 47 — Med-high)
**Depends on:** S-150-T1c-i
**Status:** [x] Done 2026-08-02
- **RRI:** 47 -> Med-high (41-55)

**Happy paths considered:**

- **HP-1:** A translation claim for one localization unit persists its exact
  source `Subtitle`, transitions the unit state, and promotes the exact
  `TranslatedSubtitle` artifact/current generation only when readiness evidence is
  complete.
- **HP-2:** A dubbing claim for one localization unit persists its exact source
  `TranslatedSubtitle`, transitions the unit state, and promotes the exact
  `DubbingManifest`/`DubbedAudio` artifacts only when readiness evidence is
  complete.
- **HP-3:** Re-delivery with the same `(operation, localization unit,
  generation_request_id)` resolves the same persisted claim instead of creating a
  second mutable generation.

**Edge cases considered:**

- **EC-1:** A pointer to an artifact belonging to another asset or wrong kind is
  rejected.
- **EC-2:** Partial artifact sets cannot transition translation or dubbing to
  Ready.
- **EC-3:** A stale generation cannot overwrite the current generation's pointers.
- **EC-4:** An unknown stored artifact kind fails closed at the repository's strict
  decode boundary even though the legacy domain parser remains lenient.
- **EC-5:** Reusing a `generation_request_id` with different operation/source facts
  fails closed instead of aliasing another generation.
- **EC-6:** An explicit regeneration cannot claim the deterministic request ID
  reserved for the initial translation of its source subtitle.

**Acceptance criteria:** Implement fail-closed translation/dubbing repositories for
per-target state, generation claims, exact artifact pointers, and readiness
evidence over the ratified `T1c-i` schema. Enforce one atomic generation claim per
`(operation, localization unit, generation_request_id)`, persist and re-read its
exact source artifact, reject explicit-regeneration use of the reserved initial ID,
and cover every HP/EC case with unit/integration tests. This task must not add or
change migrations.

**Files expected to change:** `crates/db/src/translation_repo.rs`,
`crates/db/src/dubbing_repo.rs`, `crates/db/src/artifact_repo.rs`,
`crates/db/src/lib.rs`, and scoped repository tests (expected under
`apps/api/tests/` unless implementation proves a narrower in-crate test surface).

**Evidence to emit:** Exact RRI, reviewer artifact, Reflection log, unit coverage
certification, Postgres test output, and owner verification.

**Status artifacts affected:** This ledger.

**Agent handoff prompt:** Implement only localization repositories and readiness
evidence for the `T1c-i` schema; stop before queues and workers.

**Stop condition:** Stop after repository tests and closure gates. Do not start T2.

### RRI evidence

Artifact: `docs/audit/s-150-t1c-ii-rri.md`

- Final RRI: `47`
- Band: `Med-high (41-55)`
- Effort: `L`
- Decomposition: `not triggered`

Task-analysis review: `claude` `.agent/peer-task-review-S-150-T1c-ii-cloud.json` - `PASS`

### Closure note

- Added [crates/db/src/translation_repo.rs](/Users/matias/dubbridge/crates/db/src/translation_repo.rs:1)
  and [crates/db/src/dubbing_repo.rs](/Users/matias/dubbridge/crates/db/src/dubbing_repo.rs:1)
  to own fail-closed generation claims, exact current pointers, and readiness
  evidence for the `0028` localization schema.
- Extended [crates/db/src/artifact_repo.rs](/Users/matias/dubbridge/crates/db/src/artifact_repo.rs:1)
  with strict S-150 artifact-kind decoding plus generic derived-artifact
  insert/lookup helpers, and exported the new repositories from
  [crates/db/src/lib.rs](/Users/matias/dubbridge/crates/db/src/lib.rs:1).
- Added [apps/api/tests/localization_repo_test.rs](/Users/matias/dubbridge/apps/api/tests/localization_repo_test.rs:1)
  to cover claim idempotency, exact ready promotion, partial-readiness negatives,
  stale-generation protection, reused-request fail-closed behavior, reserved
  initial-request rejection, and EC-1 promotion failures for wrong-kind,
  wrong-parent, and other-asset artifacts.
- Executed on the user-directed CLOUD authoring/review route; the primary
  `claude` phase-2 review stalled repeatedly, so the documented isolated `d14`
  fallback reviewer was used and returned `PASS` after the added EC-1 coverage.

### Peer Reviewer evidence

- Reviewer: `d14`
- Command: `multi_agent_v1.spawn_agent` with model override `gpt-5.4` for an
  isolated read-only review over the current workspace files after the primary
  `claude` cloud reviewer stalled
- Artifact: `docs/audit/gemma-evidence/S-150-T1c-ii.json`
- Verdict: `PASS`
- Findings: initial isolated review returned 2 LOW findings about missing EC-1
  promotion-failure coverage in `promote_translation_ready` and
  `promote_dubbing_ready`; both were accepted and repaired by adding
  wrong-kind/wrong-parent/other-asset promotion tests before the rerun PASS
- Gemma fallback: `not triggered` — reason: `user-directed CLOUD override kept the review path off the local qwen/gemma chain`
- D14 fallback: `triggered` — reason: `primary claude cloud review stalled repeatedly with no usable output`
- disposition_divergence: `none`
- Primary-agent disposition: accepted the LOW coverage findings, added the
  missing EC-1 promotion-failure tests, reran `cargo test` / `cargo clippy`,
  and obtained a rerun `PASS`
- Review artifact: `docs/audit/gemma-evidence/S-150-T1c-ii.json`

Code-solution review: `d14` `docs/audit/gemma-evidence/S-150-T1c-ii.json` - `PASS`

### Reflection log

Required passes: 3 (`47` -> `Med-high`)

#### Pass 1

- **Draft verdict:** Translation/dubbing repositories, strict artifact helpers,
  and baseline integration coverage were implemented end-to-end over `0028`.
- **Critique findings:**
  - The first integration-test cut did not yet prove EC-1 at the promotion APIs
    (`promote_translation_ready` / `promote_dubbing_ready`) for wrong-kind,
    wrong-parent, and other-asset artifacts.
  - The test helper carried one stray unused SQL query line that should be
    removed before static-gate validation.
- **Revisions applied:**
  - Removed the stray unused query in `insert_scope`.
  - Added promotion-failure coverage for translation and dubbing EC-1 paths.

#### Pass 2

- **Draft verdict:** Coverage now exercised both claim-time and promotion-time
  fail-closed boundaries across translation and dubbing.
- **Critique findings:**
  - The added coverage pushed two integration tests over the repo's clippy
    `too_many_lines` budget.
  - The updated coverage still needed a clean rerun through `cargo clippy` and
    the neighboring subtitle/transcription repository tests.
- **Revisions applied:**
  - Extracted `claim_dubbing_generation`, `insert_dubbing_outputs`, and
    `assert_dubbing_promote_error` helpers in the integration test file.
  - Reran `cargo test -p dubbridge-db`,
    `cargo test -p dubbridge-api --test subtitle_repo_test --test transcription_repo_test --test localization_repo_test`,
    and `cargo clippy -p dubbridge-db -p dubbridge-api --tests -- -D warnings`.

#### Pass 3

- **Draft verdict:** Runtime tests and static gates were green on the updated
  repo/test surface.
- **Critique findings:**
  - The primary `claude` phase-2 review stalled, so closure still required an
    explicit fallback review artifact.
  - The first isolated fallback review reported only the two LOW EC-1 coverage
    gaps already addressed above; the rerun found no further issues.
- **Revisions applied:** none after the coverage expansion and test refactor;
  recorded the final fallback `PASS` artifact.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | A translation claim persists its exact `Subtitle` source, transitions the localization unit to `InProgress`, and promotes the exact `TranslatedSubtitle` only when readiness evidence is complete | `apps/api/tests/localization_repo_test.rs::translation_claim_and_promote_ready_persists_exact_current_artifacts` | passed |
| HP-2 | Happy path | A dubbing claim persists its exact `TranslatedSubtitle` source, transitions the localization unit to `InProgress`, and promotes the exact `DubbingManifest`/`DubbedAudio` pair only when readiness evidence is complete | `apps/api/tests/localization_repo_test.rs::dubbing_claim_and_promote_ready_persists_exact_manifest_and_audio` | passed |
| HP-3 | Happy path | Re-delivery with the same `(operation, localization unit, generation_request_id)` resolves the same persisted claim instead of creating a second mutable generation | `apps/api/tests/localization_repo_test.rs::translation_redelivery_same_request_reuses_existing_claim`; `apps/api/tests/localization_repo_test.rs::dubbing_redelivery_same_request_reuses_existing_claim` | passed |
| EC-1 | Edge case | A pointer to an artifact belonging to another asset or wrong kind is rejected, both at claim time and at ready-promotion time | `apps/api/tests/localization_repo_test.rs::translation_claim_rejects_wrong_kind_and_other_asset`; `apps/api/tests/localization_repo_test.rs::translation_promote_ready_rejects_wrong_kind_wrong_parent_and_other_asset_outputs`; `apps/api/tests/localization_repo_test.rs::dubbing_claim_rejects_wrong_kind_and_other_asset`; `apps/api/tests/localization_repo_test.rs::dubbing_promote_ready_rejects_wrong_kind_wrong_parent_and_other_asset_outputs` | passed |
| EC-2 | Edge case | Partial artifact sets cannot transition translation or dubbing to `Ready` | `apps/api/tests/localization_repo_test.rs::translation_claim_and_promote_ready_persists_exact_current_artifacts`; `apps/api/tests/localization_repo_test.rs::dubbing_claim_and_promote_ready_persists_exact_manifest_and_audio` | passed |
| EC-3 | Edge case | A stale generation cannot overwrite the current generation's pointers | `apps/api/tests/localization_repo_test.rs::translation_stale_generation_cannot_overwrite_new_current_output`; `apps/api/tests/localization_repo_test.rs::dubbing_stale_generation_cannot_overwrite_new_current_outputs` | passed |
| EC-4 | Edge case | Unknown stored artifact kinds fail closed at the strict repository decode boundary | `crates/db/src/artifact_repo.rs::parse_kind_unknown_value_fails_closed` | passed |
| EC-5 | Edge case | Reusing a `generation_request_id` with different source facts fails closed instead of aliasing another generation | `apps/api/tests/localization_repo_test.rs::translation_reused_request_id_with_different_source_conflicts`; `apps/api/tests/localization_repo_test.rs::dubbing_reused_request_id_with_different_source_conflicts` | passed |
| EC-6 | Edge case | An explicit regeneration cannot claim the deterministic request ID reserved for the initial translation of its source subtitle | `apps/api/tests/localization_repo_test.rs::translation_explicit_regeneration_cannot_use_reserved_initial_request_id`; `crates/db/src/translation_repo.rs::tests::explicit_regeneration_cannot_use_reserved_initial_request_id` | passed |

### Owner final verification

- Owner: `Codex agent`
- Date: 2026-08-02
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-db`; `cargo test -p dubbridge-api --test subtitle_repo_test --test transcription_repo_test --test localization_repo_test`; `cargo clippy -p dubbridge-db -p dubbridge-api --tests -- -D warnings`

---

## S-150-T2a: Extract oversized seams before T2 fan-out

**Type:** development
**Effort:** M (RRI 26 — Moderate)
**Depends on:** S-150-T1c-ii
**Status:** [x] Done

**Objective:** Extract the two code seams S-150-T2 needs — the post-ready branch
point in `subtitle_runtime.rs` and the target-language lookup in
`workspace_repo.rs` — into focused modules under the 500-line local-delegation
read gate, with zero behavior change.

**Happy paths considered:**

- **HP-1:** Existing subtitle-ready flow behaves identically after extraction
  (all current tests in `subtitle_runtime.rs` and `workspace_repo.rs` pass
  unchanged).

**Edge cases considered:**

- **EC-1:** Both target files (post-extraction, the modules T2 will read/edit)
  are under 500 lines.

**Acceptance criteria:** Extract the post-ready dispatch call inside
`process_subtitle_job_inner` (`apps/worker-runner/src/subtitle_runtime.rs`,
lines 44–117) into its own small module/function so T2's future
`post_ready_route` branch has a narrow, self-contained edit surface. Move
`list_target_languages` (`crates/db/src/workspace_repo.rs`, lines 555–582) into
a new focused module (e.g. `crates/db/src/target_language_repo.rs`), re-exported
from `crates/db/src/lib.rs`. No `post_ready_route` field, no UUIDv5 derivation,
no fan-out logic, no queue/job changes — pure mechanical extraction.

**Files expected to change:** `apps/worker-runner/src/subtitle_runtime.rs`,
`crates/db/src/workspace_repo.rs`, new `crates/db/src/target_language_repo.rs`,
`crates/db/src/lib.rs` (module export).

**Files actually changed (scope expanded during Reflection Pass 2 — see
Reflection log):**

- `apps/worker-runner/src/subtitle_runtime.rs` (520 → 132 lines)
- `apps/worker-runner/src/subtitle_alignment.rs` (new, 22 lines) — `RawAlignmentFile`,
  `RawWord`, `raw_words_to_provider` moved verbatim
- `apps/worker-runner/src/subtitle_runtime_tests.rs` (new, 373 lines) — all 6
  tests moved out of `subtitle_runtime.rs`
- `apps/worker-runner/src/main.rs` — added `mod subtitle_alignment;`,
  `#[cfg(test)] mod subtitle_runtime_tests;`
- `crates/db/src/workspace_repo.rs` (777 → 451 lines)
- `crates/db/src/target_language_repo.rs` (new, grew 44 → 164 lines) — full
  "Target languages" section moved (`list_target_languages`,
  `upsert_target_language`, `upsert_target_language_tx`,
  `delete_target_languages_for_project_tx`, `AssetSubtitleRouteRow`,
  `AssetSubtitleRoute`, `get_source_language_for_asset`,
  `get_asset_subtitle_route`, `TargetLanguageRow`)
- `crates/db/src/workspace_repo_tests.rs` (new, 170 lines) — all 11 tests
  moved out of `workspace_repo.rs`; several previously-private items
  (`require_org_role`, `OrgRow`, `org_from_row`, `MemberRow`, `member_from_row`,
  `ProjectRow`, `project_from_row`, `AssetRow`, `parse_asset_status`,
  `asset_from_row`) made `pub(crate)` to support the split
- `crates/db/src/lib.rs` — added `pub mod target_language_repo;`,
  `#[cfg(test)] mod workspace_repo_tests;`
- Call-site updates (outside the originally-approved scope):
  `apps/worker-runner/src/review_enqueue.rs`,
  `apps/worker-runner/src/subtitle_enqueue.rs`,
  `apps/worker-runner/src/runner_topology_tests.rs`,
  `apps/worker-runner/src/transcription_runtime.rs`,
  `apps/worker-runner/src/preparation_runtime_tests/enqueue_flow.rs`,
  `apps/api/src/workspace_service.rs`, `apps/api/tests/workspace_test.rs`

Scope expansion was authorized by the user after Reflection Pass 2 found the
originally-approved minimal extraction insufficient (EC-1 not met — see
Reflection log below).

**Evidence to emit:** RRI report, phase-1/phase-2 review artifacts, Reflection
log (2 passes), unit coverage certification, owner verification.

**Status artifacts affected:** This ledger's S-150-T2b/T2c scope and the
S-150 plan's execution sequence (paths change post-extraction).

**Agent handoff prompt:** Extract the named seams verbatim into focused
modules; preserve behavior exactly; do not touch job/queue types or branching
logic. Stop after extraction compiles and existing tests pass.

**Stop condition:** Stop once both target files are under 500 lines and all
existing tests pass. Do not start any T2 fan-out logic.

**RRI:** 26 → Moderate (26–40); gates: confirm tests exist in affected area;
penalties: none. `python3 scripts/rri.py --touches
apps/worker-runner/src/subtitle_runtime.rs --touches
crates/db/src/workspace_repo.rs --cc 2 --D 1 --K 1 --P 1 --T 0 --A 0 --X 1
--platform dubbridge`

**Task-analysis review:** `gemma docs/audit/gemma-evidence/S-150-T2.json - BLOCKED→resolved`
(3 findings disposed: 1 HIGH rejected as false positive with compile-level
evidence, 1 MEDIUM accepted and resolved via this T2a/T2 decomposition, 1 LOW
accepted and resolved by direct inspection)

**Approval:** User approved via Compact Approval Task Card v2, 2026-08-09.

### Reflection log

Required passes: 2 (`26` → `Moderate`)

#### Pass 1

- **Draft verdict:** Local implementer (`qwen3.6:35b-a3b`, disposable worktree
  `local/s-150-t2a`) produced a minimal extraction before the run was
  interrupted (session-level kill of the orchestrator process, not the Ollama
  daemon): `dispatch_post_ready` wrapper added in `subtitle_runtime.rs`
  (call-site extracted, nothing removed); `list_target_languages` +
  `TargetLanguageRow` moved out of `workspace_repo.rs` into a new
  `target_language_repo.rs`. Compiles; existing tests pass.
- **Critique findings:** Contract/behavior check only — did not yet verify
  EC-1 (file-size gate) with `wc -l`. Logical correctness against HP-1
  confirmed by passing tests.
- **Revisions applied:** none (contract pass; no defects found against
  HP-1).

#### Pass 2

- **Draft verdict:** Ran `wc -l` on both target files as part of the coverage
  focus. Result: `subtitle_runtime.rs` = 529 lines (grew from 520 — the
  wrapper added code without removing any), `workspace_repo.rs` = 739 lines
  (down from 777, but still far above the 500-line gate). **EC-1 not
  satisfied.**
- **Critique findings:** The approved minimal-extraction scope
  (`dispatch_post_ready` wrapper + `list_target_languages` only) does not
  achieve the task's own acceptance criterion. Both target files remain
  above the 500-line local-delegation read gate that this task exists to
  clear for S-150-T2.
- **Revisions applied:** Reported the gap to the user rather than declaring
  success from passing tests alone. User authorized (via AskUserQuestion)
  expanding T2a's scope now rather than deferring to a separate T2a-ii task,
  then authorized (via a second AskUserQuestion) proceeding with the full
  "Target languages" section move despite it touching more call-site files
  than the originally-approved card. Implemented directly by the primary
  agent (not re-invoking the local runner — this is a scope expansion
  authorized by the user, not a local-agent repair attempt within the
  Moderate-band 2-attempt budget):
  - Moved `RawAlignmentFile`/`RawWord`/`raw_words_to_provider` to new
    `subtitle_alignment.rs`; moved all 6 `subtitle_runtime.rs` tests to new
    `subtitle_runtime_tests.rs`. Result: `subtitle_runtime.rs` → 132 lines.
  - Moved the remaining "Target languages" functions/structs
    (`upsert_target_language`, `upsert_target_language_tx`,
    `delete_target_languages_for_project_tx`, `AssetSubtitleRouteRow`,
    `AssetSubtitleRoute`, `get_source_language_for_asset`,
    `get_asset_subtitle_route`) to `target_language_repo.rs`; moved all 11
    `workspace_repo.rs` tests to new `workspace_repo_tests.rs` (required
    marking several previously-private items `pub(crate)`). Result:
    `workspace_repo.rs` → 451 lines.
  - Updated all 7 affected call-site files; confirmed via repo-wide grep
    that zero unmigrated `workspace_repo::{upsert_target_language,
    delete_target_languages_for_project_tx, upsert_target_language_tx,
    get_source_language_for_asset, get_asset_subtitle_route,
    list_target_languages}` references remain.
  - Re-verified: `cargo check --all-targets`, `cargo fmt --check`, `cargo
    clippy --all-targets -- -D warnings` all clean (only a pre-existing
    unrelated `apalis-redis` future-incompatibility warning). Full test
    suites re-run against a live Postgres backend (`--test-threads=1`, no
    mocks): `dubbridge-db` 77/77, `dubbridge-worker-runner` 52/52,
    `dubbridge-api` full crate (~22 binaries) all "0 failed". EC-1 now
    satisfied: both target files under 500 lines (132, 451).

Review artifact: docs/audit/gemma-evidence/S-150-T2a.json

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: manual Ollama `/api/chat` invocation (`OLLAMA_HOST=http://127.0.0.1:11434`),
  `think: false`, `num_predict: 8192`, `num_ctx: 131072`
- Artifact: `docs/audit/gemma-evidence/S-150-T2a.json`
- Verdict: `PASS`
- Findings: none
- Gemma fallback: not triggered — reason: `qwen3.6:27b-q4_K_M` healthy
  (precheck `done_reason: stop`, non-empty content) and returned a usable
  parseable verdict on first send after one transport-level timeout retry
  (initial 300s curl timed out on the ~66K-character packet; retried at
  540s and completed with `done_reason: stop`)
- D14 fallback: not triggered — reason: n/a
- disposition_divergence: `none`
- Primary-agent disposition: accepted PASS verdict. Independently
  cross-checked the reviewer's four requested focus areas against the
  primary agent's own evidence before accepting: (1) pure-extraction claim —
  confirmed via `git diff` showing deletions in the two target files
  reappearing verbatim in the four new files, no signature/logic changes;
  (2) missed call sites — confirmed via repo-wide grep (zero unmigrated
  references, see Reflection log Pass 2); (3) visibility changes — the
  `pub(crate)` grants on `workspace_repo.rs` items were reviewed individually
  during implementation, scoped to exactly what `workspace_repo_tests.rs`
  needs, nothing broader; (4) test coverage — all 17 moved tests (6 + 11)
  accounted for in the new `_tests.rs` files and passing.

**Task-analysis review:** `qwen3.6:27b-q4_K_M — n/a (user override: Gemma-first,
Codex after 2 failed attempts — not exercised; phase-1 resolved via the
`gemma` line recorded above)`
**Code-solution review:** `qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/S-150-T2a.json - PASS`

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Existing subtitle-ready flow, alignment parsing, and org/member/project/asset row mapping behave identically after extraction | `apps/worker-runner/src/subtitle_runtime_tests.rs::process_subtitle_job_marks_ready_and_stores_artifact_on_success`, `alignment_seconds_to_ms_conversion_is_correct`; `crates/db/src/workspace_repo_tests.rs::org_from_row_maps_fields_correctly`, `member_from_row_maps_known_role`, `project_from_row_maps_fields_correctly`, `asset_from_row_maps_known_status` | passed |
| EC-1 | Edge case | Both target files under 500 lines post-extraction | `apps/worker-runner/src/subtitle_runtime_tests.rs::subtitle_runtime_stays_under_local_delegation_read_gate`, `crates/db/src/workspace_repo_tests.rs::workspace_repo_stays_under_local_delegation_read_gate` (each reads its target file via `CARGO_MANIFEST_DIR` and asserts `line_count < 500`) | passed |
| EC-1 (regression) | Edge case | Failure-path behavior unchanged after extraction (fails closed on invalid input) | `apps/worker-runner/src/subtitle_runtime_tests.rs::process_subtitle_job_fails_when_alignment_missing`, `process_subtitle_job_fails_closed_on_invalid_segmentation_output`, `process_subtitle_envelope_rejects_wrong_job_type`; `crates/db/src/workspace_repo_tests.rs::require_org_role_unknown_value_fails_closed`, `parse_asset_status_unknown_value_fails_closed`, `member_from_row_unknown_role_fails_closed`, `asset_from_row_unknown_status_fails_closed` | passed |

### Owner final verification

- Owner: `Matias Kruk`
- Date: 2026-08-09
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior. I confirm the extraction is a pure mechanical move with zero behavior change, verified independently by the primary agent's compile/lint/test cycle and by the qwen3.6:27b-q4_K_M phase-2 reviewer.
- Commands run: `cargo check --all-targets`; `cargo fmt --all -- --check`; `cargo clippy --all-targets -- -D warnings`; `DUBBRIDGE_DATABASE_URL="postgres://dubbridge:dubbridge@localhost:5432/dubbridge" cargo test -p dubbridge-db -- --test-threads=1`; `DUBBRIDGE_DATABASE_URL="postgres://dubbridge:dubbridge@localhost:5432/dubbridge" cargo test -p dubbridge-worker-runner -- --test-threads=1`; `DUBBRIDGE_DATABASE_URL="postgres://dubbridge:dubbridge@localhost:5432/dubbridge" cargo test -p dubbridge-api -- --test-threads=1`; `git merge --ff-only local/s-150-t2a`

---

## S-150-T2: Translation fan-out delivery parent — decomposed

**Type:** development parent
**Effort:** L (historical parent RRI 50 — Med-high; not an executable handoff)
**Depends on:** S-150-T2a
**Status:** [ ] Decomposed 2026-08-09 into S-150-T2b-i, S-150-T2b-ii, and S-150-T2c

**Objective:** Replace the first-target-only S-140 bridge with a durable,
per-target translation delivery path without treating a Redis message as the
durable source of truth.

**Why decomposed:** The former single card combined a new durable delivery
contract, asset/project/target authorization, serialized-job compatibility,
Redis fan-out, and runtime wiring. A review found that it could not state what
happens after a claim succeeds but enqueue fails, and it did not validate that
the job's project contains the asset. The migration-bearing persistence portion
also exceeds the RRI 56 decomposition threshold when scored together with the
repository changes. The children preserve the two intended phases while making
the migration independently reviewable:

1. **Durable delivery contract:** T2b-i persists the outbox boundary; T2b-ii
   consumes it through a fail-closed repository API and the exact
   asset/project/target lookup.
2. **Job compatibility and fan-out:** T2c adds the versioned job contracts,
   deterministic request ID, replay lookup, Redis enqueue, and worker-runner
   wiring that consume the completed durable contract.

**Shared contract for every child:** “exactly one” means exactly one durable
logical translation generation and one durable dispatch record per localization
unit. Redis remains at-least-once transport; physical message redelivery is
allowed only when it resolves the same claim and dispatch record. A crash between
claim creation and Redis enqueue must remain recoverable from PostgreSQL.

**Status artifacts affected:** This ledger, `docs/plan/s-150-translation-dubbing.md`,
and `docs/plan/roadmap.md`.

**Stop condition:** Do not implement this parent. Present and execute children in
order; do not start T3a before T2c closes.

---

## S-150-T2b-i: Translation dispatch outbox migration

**Type:** migration
**Effort:** L (RRI 55 — Med-high)
**Depends on:** S-150-T2a
**Status:** [x] Done 2026-08-12

### RRI evidence

- Artifact: `docs/audit/s-150-t2b-i-rri.md`
- Final RRI: `55` — Med-high (41–55); Effort `L`; decomposition not triggered.
- Task-analysis review: `n/a` — migration-only exemption.

**Happy paths considered:**

- **HP-1:** Creating an initial translation generation persists one pending
  dispatch row for each valid `(project_id, asset_id, target_language_id,
  generation_request_id)` claim.

**Edge cases considered:**

- **EC-1:** A duplicate delivery or a conflicting source/request tuple cannot
  create a second dispatch row.
- **EC-2:** A dispatch row cannot reference an asset outside its project, a target
  language outside that project, or a source artifact outside the claim.

**Acceptance criteria:** Add one forward-only migration after `0028` that creates
the translation-dispatch outbox. Its primary/unique identity must match the
translation generation claim identity, and its foreign keys must preserve the
existing `(project_id, asset_id)` and `(target_language_id, project_id)` ownership
boundaries from `localization_generation_claims`. Persist an explicit delivery
state sufficient to distinguish pending, acknowledged, and enqueue-failed work,
with error detail and timestamps; constrain all stored state values. The migration
must not add a mutable artifact pointer, change TTS/dubbing behavior, or enqueue
Redis work.

**Files expected to change:** one new `infra/migrations/0029_*.sql` file (the exact
number is revalidated immediately before implementation) and migration-focused
tests only if the repository's migration harness requires them.

**Evidence to emit:** Exact RRI output, migration application/constraint evidence
against fresh PostgreSQL, phase-review artifact, Reflection log, and unit coverage
certification.

**Status artifacts affected:** This ledger and the S-150 plan's task sequence.

**Agent handoff prompt:** Add only the translation-dispatch outbox migration and
its constraints; validate it on fresh PostgreSQL; stop before repository or Redis
code.

**Stop condition:** Stop after migration validation. Do not start T2b-ii.

### ADR-038 route evidence

- Muse Glimmer refinement: `route_recommendation: GO_LOCAL` —
  `.agent/local-architect/med-high-refinement-v1/S150-T2b-i/refinement-artifact.json`
  (model `muse-glimmer:30b-q4_K_M`, digest-verified).
- Primary route receipt: `GO_LOCAL` (no downgrade) —
  `.agent/local-architect/med-high-refinement-v1/S150-T2b-i/primary-receipt.json`.
- Gate decision (`scripts/local-agent/med_high_gate.py`): `GO_LOCAL` — both sides
  independently recommended local implementation.
- Bounded local attempt (`qwen3.6:27b-q4_K_M`, `run_med_high_task.py`, own
  process group, ≤8 turns/≤300s/0 repairs): **failed** — `boundary_violation`,
  the model attempted a denylisted `docker` command while probing available
  tooling before it could verify its own draft. This is the sole real attempt;
  Med-high has zero repair attempts, so it escalated directly.
- Escalation classification: `capability-risk` (not operational-only) — the
  local model produced a draft but could not complete the verification step
  within its allowed command boundary.
- ADR-039 fallback-selection-v1 checkpoint: `fallback_authorized` —
  `selected_model: gpt-5.6-terra`, `selected_reasoning_effort: high`,
  `selected_by: matias`, `selection_mode: preauthorized`. This departs from the
  policy-recommended `gpt-5.6-sol/high` for a capability-risk trigger; the human
  selector explicitly chose Terra instead, which the checkpoint schema permits
  (`selected_model` is not constrained to match `recommended_model`).
  Artifact: `.agent/local-architect/med-high-refinement-v1/S150-T2b-i/fallback-selection.json`.
- Cloud implementation (`gpt-5.6-terra`/`high` via `codex exec`): reviewed the
  local draft against the spec, corrected its stale header comment
  (`S-150-T1d` → `S-150-T2b-i`), confirmed the PK/FK/CHECK shape matched the
  contract, but could not run the required PostgreSQL verification — its
  sandbox had no access to the host's Docker/Colima socket
  (`permission denied`, `operation not permitted`). Codex stopped rather than
  claiming unverified success.
- **Operational note:** an intermediate re-invocation of `run_med_high_task.py`
  (intended only to re-emit the checkpoint) accidentally launched a second real
  local attempt at a 1-second wall clock, which timed out
  (`wall_clock_exceeded`). This was a primary-agent operational error, not a
  second sanctioned repair attempt; it was caught immediately, and the
  checkpoint above was reconstructed from the original `boundary_violation`
  bundle (the true, sole local attempt) rather than the spurious one.

### Independent PostgreSQL verification (primary agent, real database)

Codex's static review was correct but unverified against a real database. The
primary agent (Claude Code) ran the verification Codex's sandbox could not
reach, using the already-running local Postgres (`local-postgres-1`, exact
schema state migrations 0001–0028 applied, confirmed via `_sqlx_migrations`),
cloned into an ephemeral `t2b_i_verify` database (`CREATE DATABASE ...
TEMPLATE dubbridge`), dropped after the run:

```
CREATE TABLE                                              -- migration applies cleanly
INSERT 0 1                                                -- HP-1: valid row, delivery_state='pending'
ERROR: duplicate key value violates unique constraint     -- EC-1: duplicate claim tuple rejected
ERROR: violates foreign key constraint ...project_asset_fk       -- EC-2: asset outside project rejected
ERROR: violates foreign key constraint ...target_language_fk     -- EC-2: target outside project rejected
ERROR: violates check constraint ...delivery_state_check  -- invalid delivery_state rejected
ERROR: violates check constraint ...operation_check       -- invalid operation rejected
count = 1                                                 -- only the one valid row persisted
```

All 7 acceptance tests passed. Fixture and test SQL:
`/private/tmp/claude-501/-Users-matias-dubbridge/44408e28-980e-47b9-86c0-4a3da925fcb3/scratchpad/t2b_i_fixture.sql`,
`t2b_i_tests.sql` (session-scratch, not committed — the migration file and
this evidence record are the durable artifacts).

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat` (resolved `DUBBRIDGE_REVIEW_MODEL`)
- Command: `GEMMA_REVIEW_TASK_ID=s-150-t2b-i REVIEW_PATHS="infra/migrations/0029_create_translation_dispatch_outbox.sql" make qa-gemma-review`
- Passes run / usable: `3/3`
- Aggregate status: `FINDINGS` (2 pass-specific, both `minor`, none blocking)
- Consensus findings: `0` | Pass-specific: `2` | Disagreement: `0`
- Artifacts: `/tmp/dubbridge-gemma-review.json`, `docs/audit/gemma-evidence/s-150-t2b-i.json`
- Isolated adjudicator: `not triggered` — Gemma responded normally, no fallback needed
- D14 provider route: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition:
  - Finding 1 (`updated_at` has no refresh trigger): **rejected as out-of-scope,
    not a defect of this task.** Verified via
    `grep -rln "BEFORE UPDATE\|CREATE.*TRIGGER" infra/migrations/*.sql` → zero
    matches repo-wide; `infra/migrations/0027_create_translation_dubbing_status_and_extend_artifact_kind_check.sql:12`
    has the identical `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()` pattern
    with no trigger. This is pre-existing, repo-wide behavior, not something
    introduced by T2b-i; adding a trigger here alone would make this table
    inconsistent with `asset_translation_status`/`asset_dubbing_status`. Flagged
    as technical debt worth a future cross-cutting task, not fixed inline.
  - Finding 2 (`delivery_state` CHECK may need more lifecycle states): **rejected
    as contradicting the approved spec.** The task card (line 1008 of this
    ledger) specifies the delivery state must distinguish exactly "pending,
    acknowledged, and enqueue-failed" — no more. Additional states belong to
    T2c (fan-out) or T2b-ii (repository), which are explicitly out of scope
    here per the Stop condition ("Stop after migration validation. Do not
    start T2b-ii."). Expanding the CHECK now would violate the approved
    acceptance criteria, not satisfy them.

Code-solution review: gemma docs/audit/gemma-evidence/s-150-t2b-i.json - PASS

### Reflection log

Required passes: 3 (`55` → `Med-high`)

#### Pass 1

- **Draft verdict:** Migration file matches the spec: composite PK identical
  to `localization_generation_claims`, dual FK to `project_assets` and
  `target_languages` with `ON DELETE CASCADE`, `delivery_state` and
  `operation` CHECK constraints, no mutable artifact pointer.
- **Critique findings:** header comment on the original local-model draft
  referenced the wrong task ID (`S-150-T1d` instead of `S-150-T2b-i`); no
  independent PostgreSQL verification had been run yet at this point in the
  cycle (Codex's sandbox could not reach the database).
- **Revisions applied:** header comment corrected to `S-150-T2b-i` (done by
  the cloud implementer); flagged the missing PostgreSQL verification as a
  blocking gap to close before certification.

#### Pass 2

- **Draft verdict:** After running the 7 acceptance tests against a real,
  ephemeral PostgreSQL database cloned from the exact 0001–0028 schema state,
  every constraint behaved as specified — no logic defects found.
- **Critique findings:** Gemma Reviewer's phase-2 pass surfaced 2 pass-specific
  minor findings (missing `updated_at` trigger; narrow `delivery_state`
  enum). Both needed independent verification against the actual spec and
  repo convention before disposition, rather than either blind acceptance or
  blind dismissal.
- **Revisions applied:** none to the migration file — both findings verified
  as out-of-scope for this task (see Gemma Reviewer evidence disposition
  above) rather than defects; no code change was warranted.

#### Pass 3

- **Draft verdict:** Final migration file is unchanged since Pass 1's header
  fix; all 7 acceptance tests pass against real PostgreSQL; Gemma Reviewer
  findings are dispositioned with cited evidence; no mutable artifact
  pointer, Rust code, or Redis wiring was introduced, honoring the
  migration-only scope boundary.
- **Critique findings:** no further issues found. The one process deviation
  worth recording is operational, not a code defect: an accidental second
  local-runner invocation during checkpoint reconstruction (see ADR-038
  route evidence above), which did not affect the final artifact or its
  verification.
- **Revisions applied:** none — the migration file and its verification
  evidence are final.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Valid claim tuple persists one pending dispatch row | Live-PostgreSQL insert against `t2b_i_verify` (ephemeral, schema 0001–0029): `INSERT INTO translation_dispatch_outbox (...) VALUES ('translation', ...) ` → `INSERT 0 1`, `delivery_state='pending'` confirmed by final `count(*) = 1` | passed |
| EC-1 | Edge case | Duplicate `(operation, project_id, asset_id, target_language_id, generation_request_id)` tuple rejected | Live-PostgreSQL insert of the identical tuple a second time → `ERROR: duplicate key value violates unique constraint "translation_dispatch_outbox_pk"` | passed |
| EC-2 | Edge case | Asset outside project, or target language outside project, rejected before any row is written | Live-PostgreSQL insert with `asset_id` unlinked from `project_id` → `ERROR: ... violates foreign key constraint "translation_dispatch_outbox_project_asset_fk"`; insert with `target_language_id` bound to a different project → `ERROR: ... violates foreign key constraint "translation_dispatch_outbox_target_language_fk"` | passed |

This ledger does not carry the `Behavioral coverage contract: unit-v1` marker,
and this is a migration-only task with no Rust/application code — "unit test
evidence" here is the live-PostgreSQL constraint verification above, which is
the applicable acceptance mechanism named in the task's own "Evidence to
emit" line ("migration application/constraint evidence against fresh
PostgreSQL").

### Owner final verification

- Owner: Claude Code (primary agent, orchestrator of record; cloud
  implementation via `gpt-5.6-terra`/`high` under human-authorized ADR-039
  checkpoint, `selected_by: matias`)
- Date: 2026-08-12
- Statement: I verified every happy path and edge case defined for this task
  (HP-1, EC-1, EC-2) against a real, ephemeral PostgreSQL database cloned from
  the exact pre-migration schema state (0001–0028 applied), not only a static
  read of the SQL. I independently verified both Gemma Reviewer findings
  against the approved spec and repo-wide migration conventions before
  rejecting them, rather than accepting or dismissing them without evidence.
- Commands run:
  - `python3 scripts/rri.py --cc 3 --T 3 --A 1 --X 2 --D 4 --K 4 --P 5 --touches infra/migrations/0029_create_translation_dispatch_outbox.sql --penalty auth_security --platform dubbridge`
  - `python3 scripts/local-architect/run_analysis.py --packet .agent/local-architect/med-high-refinement-v1/S150-T2b-i/packet.json --profile med-high-refinement-v1 --expected-packet-sha256 b266707dab7414b19a98f6ce3e242e097395a07ed66b0b75eeb0deb9321d3d37 --output .agent/local-architect/med-high-refinement-v1/S150-T2b-i/refinement-artifact.json --model-tag muse-glimmer:30b-q4_K_M --expected-model-digest de878ce33ad81d060001db1469a02eebe4d86f0ad58cfe52dc062fdcbe4464c1 --timeout-seconds 300`
  - `python3 scripts/local-agent/med_high_gate.py --refinement-artifact .agent/local-architect/med-high-refinement-v1/S150-T2b-i/refinement-artifact.json --primary-receipt .agent/local-architect/med-high-refinement-v1/S150-T2b-i/primary-receipt.json --card-hash b266707dab7414b19a98f6ce3e242e097395a07ed66b0b75eeb0deb9321d3d37 --rri 55`
  - `python3 scripts/local-agent/run_med_high_task.py --card .agent/local-architect/med-high-refinement-v1/S150-T2b-i/runner-card.json --worktree .agent/worktrees/s-150-t2b-i --out .agent/local-architect/med-high-refinement-v1/S150-T2b-i/runner-transcript.json --bundle-out .agent/local-architect/med-high-refinement-v1/S150-T2b-i/escalation-bundle.md --refinement-artifact .agent/local-architect/med-high-refinement-v1/S150-T2b-i/refinement-artifact.json --primary-receipt .agent/local-architect/med-high-refinement-v1/S150-T2b-i/primary-receipt.json --card-hash b266707dab7414b19a98f6ce3e242e097395a07ed66b0b75eeb0deb9321d3d37 --rri 55 --wall-clock-seconds 300 --fallback-mode human-select`
  - `codex exec -C .agent/worktrees/s-150-t2b-i -m gpt-5.6-terra -c model_reasoning_effort=high -s workspace-write --skip-git-repo-check ...`
  - `docker exec local-postgres-1 psql -U dubbridge -d postgres -c "CREATE DATABASE t2b_i_verify TEMPLATE dubbridge;"`
  - `docker exec local-postgres-1 psql -U dubbridge -d t2b_i_verify -f /tmp/0029.sql`
  - `docker exec local-postgres-1 psql -U dubbridge -d t2b_i_verify -f /tmp/fixture.sql`
  - `docker exec local-postgres-1 psql -U dubbridge -d t2b_i_verify -f /tmp/tests.sql`
  - `docker exec local-postgres-1 psql -U dubbridge -d postgres -c "DROP DATABASE t2b_i_verify;"`
  - `GEMMA_REVIEW_TASK_ID=s-150-t2b-i REVIEW_PATHS="infra/migrations/0029_create_translation_dispatch_outbox.sql" make qa-gemma-review`
- Result: all commands passed; migration applies cleanly; all 7 acceptance
  tests passed against real PostgreSQL; Gemma Reviewer returned PASS with 2
  non-blocking findings, both verified and dispositioned `reviewed_no_change`
  with cited evidence.

**S-150-T2b-i status: `[x] Done`**

---

## S-150-T2b-ii: Durable translation delivery repository and exact target binding

**Type:** development parent
**Effort:** L (RRI 57 — Complex; not an executable handoff)
**Depends on:** S-150-T2b-i
**Status:** Closed by decomposition — all children S-150-T2b-ii-a,
S-150-T2b-ii-b, and S-150-T2b-ii-c completed by 2026-08-13; this parent was
never an executable handoff.

**Happy paths considered:**

- **HP-1:** One exact persisted `Subtitle` source creates or reuses one durable
  translation claim and pending dispatch per configured target-language row.
- **HP-2:** Re-delivery of the same source/request reports the existing dispatch
  without creating another logical generation; a failed dispatch becomes eligible
  for bounded re-enqueue using that same identity.

**Edge cases considered:**

- **EC-1:** No target language, an asset not linked to the requested project, or a
  target belonging to another project fails closed before any claim or outbox row
  is written.
- **EC-2:** Marking one dispatch enqueue-failed changes only its localization unit;
  sibling targets and their claims remain intact.
- **EC-3:** A reused request ID with a different source remains a conflict and is
  never treated as a retry.

**Decomposition record:** The exact presentation-time score is `RRI 57`
(`Complex`), including the `auth_security` penalty for the persisted ownership
boundary. Mandatory decomposition was approved by Matias on 2026-08-12.
`docs/audit/s-150-t2b-ii-rri.md` retains the full calculation and D14's passed
task-analysis review. Each child must receive a fresh RRI, its own approval card,
and its own verification before code changes begin.

**Decomposition constraints:** The reusable scope helper may decode/query delivery
scope, but it must not make an authoritative pre-write decision. The persistence
child must invoke that helper inside its single PostgreSQL transaction before any
claim or dispatch write; this prevents a validation/write TOCTOU gap. The failure
child must use full dispatch identity plus a permitted source-state predicate, so a
stale or acknowledged dispatch cannot be changed into `enqueue_failed`.

**Status artifacts affected:** This ledger, the S-150 plan, and the roadmap.

**Stop condition:** This parent is not executable. Do not start T2c until all three
children are complete and their composed PostgreSQL acceptance matrix has passed.

---

## S-150-T2b-ii-a: Candidate delivery-scope query and decoding helpers

**Type:** development
**Effort:** M (RRI 39 — Moderate; replanned 2026-08-12)
- **RRI:** 39 / Moderate (26–40)
**Depends on:** S-150-T2b-i
**Status:** [x] Done — 2026-08-12

**Task-analysis review:** `gemma .agent/peer-task-review-S-150-T2b-ii-a-v2.json - PASS`
The prior RRI 49 review is superseded and cannot authorize this changed scope.

Code-solution review: gemma docs/audit/gemma-evidence/S-150-T2b-ii-a.json - PASS

**Happy paths considered:**

- **HP-1:** The persistence caller can obtain persisted candidate project/target
  rows and an exact source `Subtitle` identity through a reusable delivery-scope
  helper, without asserting any caller-selected project.

**Edge cases considered:**

- **EC-1:** Missing target configuration or a non-`Subtitle`/mismatched source
  yields no candidate scope. Exact requested-project and target membership
  enforcement belongs to T2b-ii-b inside its writer transaction.

**Acceptance criteria:** Extract only read/query and decoding helpers needed to
load persisted candidate delivery scope from an asset and source-artifact ID. The
helper must accept the writer's transaction but must not open or commit it; it may
not accept a caller-selected project, create claims/dispatches, or expose a
standalone authorization decision. T2b-ii-b must select and enforce the requested
project/target from these candidates inside that same transaction before any write.

**Files expected to change:** `crates/db/src/target_language_repo.rs` and new
focused integration test `apps/api/tests/delivery_scope_repo_test.rs`. This keeps
the helper independent of the 542-line `translation_repo.rs` and avoids extending
the 1,194-line legacy localization test. The exact RRI and routing evidence is
`docs/audit/s-150-t2b-ii-a-rri.md`.

**Evidence to emit:** Exact child RRI, phase reviews, unit tests for the helper
contract, and an implementation handoff note for T2b-ii-b.

**Status artifacts affected:** This ledger and the S-150 plan.

**Agent handoff prompt:** Extract only transaction-bound candidate delivery-scope
read helpers; do not accept a requested project or make an authorization decision,
and do not persist claims/dispatches.

**Stop condition:** Stop after the helper contract and tests. Do not start
T2b-ii-b.

### Implementation and route evidence

- Implementer route: `qwen3.6:27b-q4_K_M` authored the production helper in the
  disposable worktree. Nemotron was not invoked.
- Local audit: `.agent/s-150-t2b-ii-a-qwen-helper-result.json` recorded the
  expected in-scope production diff but could not sign success because the
  organization gate measured 52 meaningful lines against its 35-line
  file-growth budget. Matias explicitly accepted this bounded 52/35 exception;
  scope, acceptance, tests, and review were not waived.
- Test repair: the final Qwen run exhausted its six-turn budget and emitted
  `.agent/s-150-t2b-ii-a-qwen-final-test-result.json`. Matias then explicitly
  directed the current Codex orchestrator to correct the focused test and finish
  this child without another local-model attempt. The resulting two-test suite
  passed against live PostgreSQL.
- Antares refinement/post-implementation: typed skip — this task carried no
  task-relevant CWE hypothesis from `scripts/antares/cwe_watchlist.py`; no generic
  security sweep was run.
- Handoff to T2b-ii-b: invoke
  `target_language_repo::list_delivery_scope_candidates_tx` inside the writer's
  existing transaction, select the requested project/target from its returned
  persisted candidates, and enforce that selection before the first claim or
  dispatch write. Do not replace it with the first-target route helper.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Model: `gemma4:26b-a4b-it-qat`
- Command: manual Ollama `/api/chat` phase-2 review with `think=false`,
  `stream=false`, `num_ctx=131072`, and `num_predict=4096`
- Artifact: `.agent/peer-code-review-S-150-T2b-ii-a.json`
- Verdict: `PASS`
- Findings: none
- Retry: first response was semantic PASS but invalid under the raw-JSON schema;
  the mandatory immediate retry returned
  `{"verdict":"PASS","findings":[]}` with `done_reason: stop`
- Muse Glimmer fallback: not triggered — Gemma's retry was usable
- D14 fallback: not triggered — the local reviewer chain remained usable
- D14 provider route: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: no findings to repair or reject
- Review artifact: docs/audit/gemma-evidence/S-150-T2b-ii-a.json

### Reflection log

Required passes: 2 (`39` → `Moderate`)

#### Pass 1

- **Draft verdict:** The transaction-bound helper matched the approved read-only
  contract, but Qwen's generated test fixture was incomplete and did not compile
  or represent the migrated schema correctly.
- **Critique findings:** The fixture omitted its organization row, referenced the
  singular `artifact` table, moved owned `String` fields during SQL binding, and
  did not yet contain HP-1/EC-1 assertions.
- **Revisions applied:** After the final bounded Qwen run exhausted its turn
  budget, the explicitly authorized Codex repair replaced raw artifact SQL with
  existing `artifact_repo`/`subtitle_repo` seams, restored valid tenancy rows,
  and added the two focused PostgreSQL tests.

#### Pass 2

- **Draft verdict:** The revised helper and tests compile and both behavioral
  cases pass against live PostgreSQL; the implemented helper's executable lines
  are 100% covered in the focused `cargo llvm-cov` run.
- **Critique findings:** Rechecked the boundary for caller-selected project input,
  hidden transaction ownership, writes/authorization decisions, nondeterministic
  ordering, and non-Subtitle/mismatched/missing-target behavior. No code defect
  remained. The unsigned local audit and 52/35 organization exception are
  process evidence, not hidden as a signed success.
- **Revisions applied:** none — Gemma's independent phase-2 retry returned PASS
  with no findings.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Decode every persisted candidate project/target for the exact Subtitle identity, without a caller-selected project, in deterministic order | `apps/api/tests/delivery_scope_repo_test.rs::delivery_scope_candidates_decode_all_targets_in_deterministic_order` | passed |
| EC-1 | Edge case | Missing target configuration, non-Subtitle source, or mismatched asset/source yields no candidate | `apps/api/tests/delivery_scope_repo_test.rs::delivery_scope_candidates_fail_closed_for_missing_or_mismatched_scope` | passed |

### Owner final verification

- Owner: Codex (primary agent and orchestrator of record)
- Date: 2026-08-12
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior. I also verified that the helper borrows the caller's transaction, performs no writes, accepts no requested project, and does not expose an authorization decision.
- Commands run: `cargo fmt --all -- --check`; `cargo check -p dubbridge-db`; `cargo check -p dubbridge-api --test delivery_scope_repo_test`; `DUBBRIDGE_DATABASE_URL='postgres://dubbridge:dubbridge@127.0.0.1:5432/dubbridge' cargo test -p dubbridge-api --test delivery_scope_repo_test -- --nocapture`; `DUBBRIDGE_DATABASE_URL='postgres://dubbridge:dubbridge@127.0.0.1:5432/dubbridge' cargo llvm-cov --workspace --test delivery_scope_repo_test --summary-only`; `git diff --check`
  Detailed command record:
  - `cargo fmt --all -- --check`
  - `cargo check -p dubbridge-db`
  - `cargo check -p dubbridge-api --test delivery_scope_repo_test`
  - `DUBBRIDGE_DATABASE_URL='postgres://dubbridge:dubbridge@127.0.0.1:5432/dubbridge' cargo test -p dubbridge-api --test delivery_scope_repo_test -- --nocapture`
  - `DUBBRIDGE_DATABASE_URL='postgres://dubbridge:dubbridge@127.0.0.1:5432/dubbridge' cargo llvm-cov --workspace --test delivery_scope_repo_test --summary-only`
  - `git diff --check`
- Result: all commands passed; PostgreSQL behavioral tests passed `2/2`; LCOV
  execution counts show every executable line in
  `list_delivery_scope_candidates_tx` (lines 155–198) was exercised (`44/44`,
  100% implemented-scope line coverage). The broader pre-existing repository file
  is not claimed as 100% covered by this focused task.

---

## S-150-T2b-ii-b: Atomic delivery claim and dispatch persistence

**Type:** development
**Effort:** L (RRI 52 — Med-high)
**RRI:** 52 / Med-high
**Depends on:** S-150-T2b-ii-a
**Status:** [x] Done — 2026-08-12

Task-analysis review: gemma `.agent/peer-task-review-S-150-T2b-ii-b.json` - PASS

**Task-local phase-2 review override (2026-08-12):** Matias selected
`muse-glimmer:30b-q4_K_M` as the code-solution reviewer for this child. This
overrides the RRI 26–55 default Gemma binding for phase 2 only; phase 1 remains
the recorded Gemma PASS. If Muse Glimmer is unavailable, stalled, invalid, or
`BLOCKED`, apply the canonical fallback protocol and then D14 rather than
self-reviewing.

**Happy paths considered:**

- **HP-1:** One exact persisted `Subtitle` source creates or reuses one durable
  translation claim and pending dispatch per configured target-language row.
- **HP-2:** Re-delivery of the same source/request returns the existing dispatch;
  retryable rows remain distinguishable from active or acknowledged rows.

**Edge cases considered:**

- **EC-3:** A reused request ID with a different source is a conflict and is never
  treated as a retry.
- **EC-1 (writer enforcement):** The transaction fails before its first
  claim/dispatch write when the helper exposes invalid project/asset/target/source
  scope.

**Acceptance criteria:** Add the focused DB persistence API in one PostgreSQL
transaction. It must invoke T2b-ii-a's delivery-scope helper inside that transaction
before any claim or outbox write, then create/reuse the generation claims and
dispatch rows. Its return type must explicitly classify `new`/`retryable` versus
`active`/`acknowledged`; T2c must never infer this from queue state. Roll back the
whole transaction for scope or identity conflict. Do not add job structs, Redis,
provider calls, review rows, or enqueue-failure mutation.

**Files expected to change:** `crates/db/src/translation_repo.rs` (or a focused
delivery module), `crates/db/src/lib.rs` only if a module is introduced, and
`apps/api/tests/localization_repo_test.rs`. Recheck actual full-read paths and the
500-line gate when the child is presented.

**Evidence to emit:** Exact child RRI, route receipt if required, phase reviews,
live-PostgreSQL atomicity and redelivery evidence, Reflection log as required by
the resulting band, unit coverage certification, and owner verification.

**Status artifacts affected:** This ledger and the S-150 plan.

**Agent handoff prompt:** Persist/reuse delivery claims and dispatches atomically;
invoke the scope helper inside the writer transaction before any write; stop before
failure transitions or queue code.

**Stop condition:** Stop after atomic persistence tests. Do not start T2b-ii-c or
T2c.

### Execution evidence

- ADR-038 refinement: Muse Glimmer `GO_LOCAL`, then primary receipt
  `CLOUD_REQUIRED` because ADR-038 Amendment 1 disables Med-high local developer
  execution; both artifacts are hash-bound under
  `.agent/local-architect/med-high-refinement-v1/S-150-T2b-ii-b/`.
- Antares refinement/post-implementation: typed skip — this task carries no
  task-relevant CWE hypothesis from `scripts/antares/cwe_watchlist.py`; no generic
  security sweep was run.
- Focused live-PostgreSQL test: `translation_delivery_repo_test` passed `4/4`.
- Focused implemented-scope coverage: `crates/db/src/translation_delivery_repo.rs`
  reported `96.04%` line coverage under `cargo llvm-cov`.
- Code-solution review: muse-glimmer
  `docs/audit/gemma-evidence/S-150-T2b-ii-b.json` - PASS
- Reflection pass 1 (contract coverage): found and repaired missing direct cases
  for an unknown target and a non-`Subtitle` source; focused tests passed after
  the revision.
- Reflection pass 2 (atomicity/idempotency): verified scope lookup precedes the
  transaction's first write and `ON CONFLICT` preserves one claim/outbox identity;
  source conflict returns before dispatch mutation. No revision required.
- Reflection pass 3 (boundary and status): verified reused rows are classified from
  durable dispatch state without mutation or queue inspection, and no T2b-ii-c/T2c
  behavior entered the diff. No revision required.
- The default parallel `cargo test --workspace` exposes six pre-existing test
  isolation collisions on the fixed `owner@example.com` identity; the same result
  reproduces on a fresh PostgreSQL container, so it is not shared-database residue.
  The isolated verification therefore ran `dubbridge-api` serially against that
  fresh database, then ran the remaining workspace without its database override:
  both portions passed. The temporary container was removed afterward.

### Peer Reviewer evidence

- Reviewer: `muse-glimmer`
- Model: `muse-glimmer:30b-q4_K_M`
- Command: manual Ollama `/api/chat` phase-2 review with `stream=false`,
  `think=false`, `num_ctx=131072`, and `num_predict=4096`
- Artifact: `docs/audit/gemma-evidence/S-150-T2b-ii-b.json`
- Verdict: `PASS`
- Findings: none
- Muse Glimmer fallback: not triggered — task-local Muse binding returned valid PASS
- D14 fallback: not triggered — the local reviewer chain remained usable
- D14 provider route: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: no findings to repair or reject
- Review artifact: docs/audit/gemma-evidence/S-150-T2b-ii-b.json

### Reflection log

Required passes: 3 (`52` → `Med-high`)

#### Pass 1

- **Draft verdict:** The transaction and focused test suite implemented the approved
  single-target persistence boundary.
- **Critique findings:** Direct coverage was missing for unknown target membership
  and a non-`Subtitle` source.
- **Revisions applied:** Added both fail-closed cases to
  `translation_delivery_repo_test`; the focused PostgreSQL suite passed afterward.

#### Pass 2

- **Draft verdict:** Scope validation precedes claim/status/outbox writes, with
  `ON CONFLICT` preserving the one-identity contract.
- **Critique findings:** No defect found in the atomicity, source-conflict, or
  redelivery paths.
- **Revisions applied:** none.

#### Pass 3

- **Draft verdict:** Existing dispatch state is returned as an explicit disposition
  without queue inspection or state mutation.
- **Critique findings:** No queue, enqueue-failure transition, migration, or other
  T2b-ii-c/T2c behavior entered the final diff.
- **Revisions applied:** none.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Each caller-selected configured target creates one claim and one pending dispatch | `apps/api/tests/translation_delivery_repo_test.rs::persistence_creates_one_claim_and_pending_dispatch_for_each_selected_target` | passed |
| HP-2 | Happy path | Redelivery reports durable active, retryable, and acknowledged dispositions | `apps/api/tests/translation_delivery_repo_test.rs::redelivery_classifies_existing_dispatch_state_without_mutation` | passed |
| EC-1 | Edge case | Invalid project, target, or non-Subtitle source writes no claim or dispatch | `apps/api/tests/translation_delivery_repo_test.rs::invalid_requested_scope_fails_before_claim_or_dispatch_write` | passed |
| EC-3 | Edge case | A generation request reused with another source conflicts without a second dispatch | `apps/api/tests/translation_delivery_repo_test.rs::same_generation_with_different_source_rolls_back_without_second_dispatch` | passed |

### Owner final verification

- Owner: Codex (primary agent and orchestrator of record)
- Date: 2026-08-12
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior. The production API validates persisted scope before every write and exposes only durable dispatch dispositions; it does not enter queue or failure-transition scope.
- Commands run: `cargo fmt --all -- --check`; `cargo check -p dubbridge-db`;
  `cargo check -p dubbridge-api --test translation_delivery_repo_test`;
  `DUBBRIDGE_DATABASE_URL='postgres://dubbridge:dubbridge@127.0.0.1:5432/dubbridge' cargo test -p dubbridge-api --test translation_delivery_repo_test -- --nocapture`;
  `DUBBRIDGE_DATABASE_URL='postgres://dubbridge:dubbridge@127.0.0.1:5432/dubbridge' cargo llvm-cov --workspace --test translation_delivery_repo_test --summary-only`;
  `cargo clippy --workspace --all-targets -- -D warnings`; `make qa-docs`; and
  serial full-workspace verification split between fresh PostgreSQL `dubbridge-api`
  and the no-database-override remainder.

---

## S-150-T2b-ii-c: Guarded dispatch enqueue-failure transition

**Type:** development
**Effort:** M (RRI 35 — Moderate)
**RRI:** 35 / Moderate
**Depends on:** S-150-T2b-ii-b
**Status:** [x] Done — 2026-08-13

Task-analysis review: gemma `.agent/peer-task-review-S-150-T2b-ii-c.json` - PASS

**Happy paths considered:**

- **HP-3:** A retryable dispatch can be marked `enqueue_failed` through its exact
  project, asset, target, generation, and dispatch identity without altering a
  sibling localization unit.

**Edge cases considered:**

- **EC-2:** Marking one dispatch enqueue-failed changes only its localization unit;
  sibling targets and their claims remain intact.
- **EC-4:** An active or acknowledged dispatch, a stale generation, or a mismatched
  identity is rejected (or affects zero rows) and cannot be converted to
  `enqueue_failed`.

**Acceptance criteria:** Add a bounded, idempotent failure transition using the
full dispatch identity and a permitted source-state predicate. It must reject
active/acknowledged and mismatched rows, expose an unambiguous affected-row/result
contract, and preserve all sibling targets and claims. After this child, run the
composed live-PostgreSQL matrix across T2b-ii-a/b/c: atomic scope validation plus
persistence, concurrent/redelivery reuse, source-conflict rejection, and
sibling-preserving guarded failure. Do not add Redis, job structs, provider calls,
or a review row.

**Files changed:** `crates/db/src/translation_delivery_repo.rs` and
`apps/api/tests/translation_delivery_repo_test.rs`.

**Evidence to emit:** Exact child RRI, phase reviews, composed live-PostgreSQL
acceptance evidence, Reflection log as required by the resulting band, unit
coverage certification, and owner verification.

**Status artifacts affected:** This ledger, the S-150 plan, and the roadmap if the
parent's execution status changes.

**Agent handoff prompt:** Implement only the full-identity, state-guarded
enqueue-failure transition and its composed PostgreSQL proof; do not start queue or
job work.

**Stop condition:** Stop after the composed repository acceptance matrix. Do not
start T2c.

### Execution evidence

- Local DEV: `qwen3.6:35b-a3b` authored the bounded two-file change in the
  disposable `s-150-t2b-ii-c-qwen35-deterministic` worktree. It required two
  evidence-backed compile repairs, stayed within `allowed_paths`, and finished
  with all three card acceptance commands passing.
- The historical local artifact ended as `organization_violation` only because
  the then-active 35-line growth gate rejected 83 meaningful production lines.
  Matias explicitly removed line-count/organization from the DEV success
  decision; scope plus operator-authored acceptance are the operative functional
  gates. No out-of-scope diff was accepted.
- Integration commit: `f835b01` (`Add guarded translation dispatch failure
  transition`), pushed directly to `main` on 2026-08-13.
- Focused acceptance: `cargo fmt --all -- --check`, `cargo check -p
  dubbridge-db`, and `cargo test -p dubbridge-api --test
  translation_delivery_repo_test -- --test-threads=1` all passed; the focused
  suite passed `8/8`.
- Repository pre-push verification passed `cargo fmt`, workspace Clippy with
  warnings denied, the full workspace test suite, and workspace `cargo check`.
- Antares refinement/post-implementation: typed skip — this task carries no
  task-relevant CWE hypothesis from `scripts/antares/cwe_watchlist.py`; no generic
  security sweep was run.
- Code-solution review: gemma
  `docs/audit/gemma-evidence/S-150-T2b-ii-c.json` - PASS

### Peer Reviewer evidence

- Reviewer: `gemma`
- Model: `gemma4:26b-a4b-it-qat`
- Command: `make qa-peer-workflow-review PEER_REVIEW_PHASE=code
  PEER_REVIEW_RRI=35 PEER_REVIEW_CALLER=codex
  PEER_REVIEW_TASK_ID=S-150-T2b-ii-c
  PEER_REVIEW_ARTIFACT=.agent/peer-code-review-S-150-T2b-ii-c.json
  PEER_REVIEW_BASE=f835b01^ REVIEW_PATHS='crates/db/src/translation_delivery_repo.rs
  apps/api/tests/translation_delivery_repo_test.rs'`
- Artifact: `.agent/peer-code-review-S-150-T2b-ii-c.json`
- Verdict: `PASS`
- Findings: none
- Muse Glimmer fallback: not triggered — Gemma returned a usable PASS
- D14 fallback: not triggered — the local reviewer chain remained usable
- D14 provider route: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: no findings to repair or reject
- Review artifact: docs/audit/gemma-evidence/S-150-T2b-ii-c.json

### Reflection log

Required passes: 2 (`35` -> `Moderate`)

#### Pass 1

- **Draft verdict:** The bounded local implementation introduced the exact
  pending-to-`enqueue_failed` update and all four explicit result variants, but
  its first two drafts did not compile because the optional scalar query was
  typed incorrectly.
- **Critique findings:** The first draft produced nested `Option` handling; the
  first repair still left `delivery_state` optional at the result match.
- **Revisions applied:** Qwen used the two permitted evidence-backed repair
  cycles to switch to a correctly typed `fetch_optional` query and explicit
  absent-row handling. The third acceptance run passed all commands.

#### Pass 2

- **Draft verdict:** The integrated implementation updates only an exact pending
  dispatch identity, returns explicit outcomes for already-failed,
  acknowledged, and absent identities, and passes focused and workspace checks.
- **Critique findings:** Gemma's phase-2 review returned PASS with no findings;
  the primary agent rechecked the exact composite predicate, state guard,
  transaction boundary, result mapping, and lack of queue/provider changes.
- **Revisions applied:** none.

### Happy paths covered

- A persisted `pending` dispatch transitions to `enqueue_failed`, stores the
  supplied failure detail, and returns `Marked` in
  `translation_dispatch_enqueue_failure`; exercised by
  `enqueue_failure_marks_pending_returns_marked_and_sets_error_detail`.

### Edge cases covered

- The production update predicate includes operation, project, asset, target,
  generation request, and `pending` state, so only the exact eligible row can
  mutate; the composed target fixture and exact-row transition tests preserve
  the sibling delivery boundary.
- Already-failed dispatches return `AlreadyFailed`; acknowledged dispatches
  return `Rejected`; absent or mismatched identities return `NotFound`, covered
  by the three corresponding `enqueue_failure_*` tests.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-3 | Happy path | Exact pending dispatch transitions to enqueue_failed and persists error detail | `apps/api/tests/translation_delivery_repo_test.rs::enqueue_failure_marks_pending_returns_marked_and_sets_error_detail` | passed |
| EC-2 | Edge case | Exact target identity is isolated from sibling target deliveries and claims | `apps/api/tests/translation_delivery_repo_test.rs::persistence_creates_one_claim_and_pending_dispatch_for_each_selected_target`; `apps/api/tests/translation_delivery_repo_test.rs::enqueue_failure_marks_pending_returns_marked_and_sets_error_detail` | passed |
| EC-4 | Edge case | Already-failed, acknowledged, and absent/mismatched identities do not transition as pending work | `apps/api/tests/translation_delivery_repo_test.rs::enqueue_failure_returns_already_failed_when_enqueue_failed`; `apps/api/tests/translation_delivery_repo_test.rs::enqueue_failure_returns_rejected_when_acknowledged`; `apps/api/tests/translation_delivery_repo_test.rs::enqueue_failure_returns_not_found_for_absent_identity` | passed |

### Owner final verification

- Owner: Codex (primary agent and orchestrator of record)
- Date: 2026-08-13
- Statement: I verified every happy path and edge case defined for this child has unit test evidence in the focused PostgreSQL matrix; I also verified the exact composite SQL predicate. The implementation contains no Redis, job, provider, migration, review-row, or T2c behavior.
- Commands run: `cargo fmt --all -- --check`; `cargo check -p dubbridge-db`;
  `cargo test -p dubbridge-api --test translation_delivery_repo_test --
  --test-threads=1`; repository pre-push `cargo fmt --all -- --check`, `cargo
  clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test
  --workspace --all-features`, and `cargo check --workspace --all-targets
  --all-features`; phase-2 peer-review command recorded above.
- Result: all verification commands passed; focused PostgreSQL suite `8/8`;
  Gemma phase-2 review PASS with no findings.

---

## S-150-T2c: Versioned localization jobs and outbox-backed fan-out

**Type:** development
**Effort:** L (RRI 65 — Complex; mandatory decomposition before presentation)
**Depends on:** S-150-T2b-ii-c
**Status:** [~] Decomposed 2026-08-13 into S-150-T2c-i through S-150-T2c-v

**RRI evidence:** `docs/audit/s-150-t2c-rri.md`; decomposition evidence:
`docs/audit/s-150-t2c-decomposition-rri.md`

Task-analysis review: `muse-glimmer` `.agent/peer-task-review-S-150-T2c-local-pair.json` - `PASS`

**Review routing exception (2026-08-13):** Matias selected two local analysis
passes (`muse-glimmer:30b-q4_K_M` and `gemma4:26b-a4b-it-qat`) in place of the
normal Complex-band cross-vendor reviewer after that route was unavailable. Muse
returned no findings. Gemma's two findings are retained with the primary-agent
dispositions in `.agent/peer-task-review-S-150-T2c-local-pair.json`; neither
requires a parent-scope change, but the UUIDv5 basis must be explicit in the
decomposed child tests.

**Happy paths considered:**

- **HP-1:** A localization-routed subtitle readiness event resolves its exact
  persisted `Subtitle`, derives the deterministic initial request ID, and creates
  one logical dispatch for every validated target.
- **HP-2:** A serialized pre-S-150 `SubtitleJob` without `post_ready_route`
  decodes to `legacy_subtitle_review_v1` and preserves legacy review enqueue.
- **HP-3:** A newly constructed localization `SubtitleJob` serializes
  `s150_localization_v1`; its emitted `TranslationJob`s carry project, asset,
  target-language ID, exact source artifact ID, and generation-request ID.

**Edge cases considered:**

- **EC-1:** An unknown `post_ready_route` fails deserialization; legacy jobs never
  fan out and localization jobs never call the null-artifact review enqueue.
- **EC-2:** Replay resolves the existing subtitle by the exact
  `(asset_id, word_alignment_parent_artifact_id)` boundary rather than inserting a
  replacement; an explicit regeneration uses caller-minted UUIDv4 rather than the
  initial UUIDv5 derivation.
- **EC-3:** One Redis enqueue failure marks only its outbox/localization unit
  failed and leaves retryable durable work; no sibling claim is corrupted.

**Acceptance criteria:** Add `SubtitleJob.post_ready_route` with exactly
`legacy_subtitle_review_v1` and `s150_localization_v1` wire values. `SubtitleJob::new`
must remain explicitly legacy; an absent field defaults only to the legacy value;
unknown values must fail closed. Add an explicit localization constructor and a
`TranslationJob`/queue contract containing the full durable identity, including
`target_language_id` rather than a free-form target string. At the extracted
post-ready seam, branch exclusively on this route: the legacy branch calls the
existing review enqueue; the localization branch resolves the exact stored subtitle,
derives `UUIDv5(S150_INITIAL_TRANSLATION_NAMESPACE,
"initial-translation-v1:" || canonical_lowercase_subtitle_artifact_uuid)`, asks
T2b-ii for durable dispatches, and enqueues only dispatches its returned state marks
eligible. Persist acknowledgement or enqueue failure through T2b-ii. Wire Redis
translation enqueue into the subtitle worker, but do not register provider execution
or create an S-150 review row before T6.

**Files expected to change:** root `Cargo.toml` if UUIDv5 must be enabled,
`crates/jobs/src/lib.rs`, `crates/db/src/subtitle_repo.rs`,
`apps/worker-runner/src/subtitle_runtime.rs`, new
`apps/worker-runner/src/translation_enqueue.rs`, `apps/worker-runner/src/main.rs`,
their scoped tests, and only the minimum T2b-ii API call sites. Recheck all actual
read paths against the 500-line gate before local routing; the expected job/runtime
surface may require the ADR-038 cloud branch.

**Evidence to emit:** Exact RRI, route receipt, phase reviews, Reflection log,
unit coverage certification, owner verification, legacy-JSON characterization, and
Redis/in-memory queue evidence.

**Status artifacts affected:** This ledger, the S-150 plan, and the S-140 plan/ledger
only if their delivered handoff wording becomes materially stale.

**Agent handoff prompt:** Add only the versioned route, exact-id job contract, and
outbox-backed fan-out; preserve legacy behavior and stop before provider execution.

**Stop condition:** This parent is not executable. Present and execute the children
in order; do not start T3a before T2c-v closes.

---

## S-150-T2c-i: Versioned subtitle and translation job contracts

**Type:** development
**Effort:** L (RRI 41 — Med-high)
**RRI:** 41
**Decomposed from:** S-150-T2c
**Depends on:** S-150-T2b-ii-c
**Status:** [x] Done — 2026-08-13

**RRI evidence:** `docs/audit/s-150-t2c-decomposition-rri.md`

Task-analysis review: `gemma` `.agent/peer-task-review-S-150-T2c-i.json` - `PASS`

**Phase-1 disposition:** Gemma requested a concrete fail-closed deserialization
mechanism. The acceptance test must assert a serde error for an unknown route wire
value; derived enum behavior is sufficient only if that test passes, otherwise the
implementation must supply the minimum custom deserializer. No scope expansion is
authorized.

**Happy paths considered:**

- **HP-1:** Pre-S-150 serialized `SubtitleJob` JSON without `post_ready_route`
  decodes as `legacy_subtitle_review_v1`; `SubtitleJob::new` stays explicit legacy.
- **HP-2:** A localization constructor emits `s150_localization_v1`, and its
  `TranslationJob` carries project, asset, target-language UUID, exact subtitle UUID,
  and generation-request UUID.
- **HP-3:** The same exact subtitle UUID derives the same initial UUIDv5 request ID
  in every replay and target fan-out.

**Edge cases considered:**

- **EC-1:** An unknown route wire value fails deserialization instead of falling
  back to either behavior.
- **EC-2:** The helper uses a canonical lowercase hyphenated UUID input; explicit
  regeneration is not represented by the deterministic initial-ID helper.

**Acceptance criteria:** Add the serde-compatible route enum with an absent-field
legacy default; keep `SubtitleJob::new` legacy; add an explicit localization
constructor; add `TranslationJob`, queue trait, and in-memory queue contracts. Own
the public `S150_INITIAL_TRANSLATION_NAMESPACE` and a pure helper using exactly
`initial-translation-v1:` plus the canonical lowercase subtitle UUID. No database,
Redis, route dispatch, provider work, or review-row creation is allowed.

**Files expected to change:** `Cargo.toml`, `crates/jobs/Cargo.toml`, `Cargo.lock`,
and focused unit tests in `crates/jobs/src/lib.rs`. The manifest/lockfile changes
only enable the existing `uuid` dependency's UUIDv5 support and the existing
workspace `serde_json` crate for JSON characterization tests.

**Evidence to emit:** RRI, phase reviews, ADR-038 route receipt, Reflection log,
unit coverage certification, owner verification, and JSON/UUID characterization
tests.

**Status artifacts affected:** This ledger, `docs/plan/s-150-translation-dubbing.md`,
and `docs/plan/roadmap.md`.

**Agent handoff prompt:** Implement only versioned job serialization and pure
identity contracts in `crates/jobs/src/lib.rs`; prove legacy JSON and UUIDv5
determinism; stop before database or Redis code.

**Stop condition:** Stop after focused jobs tests. Do not start T2c-ii, T2c-iii, or
runtime wiring.

### ADR-038 route evidence

- Muse Glimmer refinement: `GO_LOCAL` —
  `.agent/local-architect/med-high-refinement-v1/S-150-T2c-i/refinement-artifact.json`.
- Primary receipt: `GO_LOCAL` —
  `.agent/local-architect/med-high-refinement-v1/S-150-T2c-i/primary-receipt.json`.
- Gate: `GO_LOCAL`; Med-high local execution is policy-excluded, so the supervisor
  emitted the cloud handoff and ADR-039 preauthorized `gpt-5.6-sol` / `high` for
  the approved capability-risk route —
  `.agent/local-architect/med-high-refinement-v1/S-150-T2c-i/fallback-selection.json`.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: `GEMMA_REVIEW_BASE=HEAD GEMMA_REVIEW_TASK_ID=S-150-T2c-i GEMMA_REVIEW_RESULT=.agent/peer-code-review-S-150-T2c-i.json REVIEW_PATHS='Cargo.toml crates/jobs/Cargo.toml crates/jobs/src/lib.rs' make qa-gemma-review`
- Artifact: `.agent/peer-code-review-S-150-T2c-i.json`
- Verdict: `PASS`
- Findings: one pass-specific `nit` on `#[derive(Default)]`; verified by
  `cargo clippy -p dubbridge-jobs --all-targets -- -D warnings` on the current
  toolchain, which accepts the derive and `#[default]` variant. No change needed.
- Muse Glimmer fallback: not triggered — Gemma returned 3/3 usable passes.
- D14 fallback: not triggered — Gemma aggregate was usable.
- D14 provider route: n/a — reason: n/a.
- disposition_divergence: null
- Primary-agent disposition: reviewed_no_change; the test and Clippy evidence
  resolve the reviewer nit without expanding scope.

Code-solution review: `gemma` `.agent/peer-code-review-S-150-T2c-i.json` - `PASS`

### Reflection log

Required passes: 3 (`41` → `Med-high`)

#### Pass 1

- **Draft verdict:** Added the versioned route, immutable UUID identity contract,
  in-memory translation queue, and focused JSON/UUID tests.
- **Critique findings:** UUIDv5 was unavailable from the workspace feature set;
  JSON characterization tests required the existing workspace `serde_json` crate
  as a direct dev dependency.
- **Revisions applied:** Enabled `uuid` feature `v5`, added the focused
  `serde_json` dev dependency, and recorded the resulting lockfile update.

#### Pass 2

- **Draft verdict:** Focused tests passed; the first Clippy run rejected a manual
  `Default` implementation as derivable.
- **Critique findings:** The route's default must remain explicit, idiomatic, and
  stable under the repository lint policy.
- **Revisions applied:** Replaced the manual implementation with
  `#[derive(Default)]` and a `#[default]` legacy variant; focused tests and
  Clippy then passed.

#### Pass 3

- **Draft verdict:** Gemma completed 3/3 code-review passes and found one
  pass-specific toolchain-compatibility nit.
- **Critique findings:** Confirm whether `#[default]` is accepted by the actual
  repository toolchain rather than relying on inference.
- **Revisions applied:** None; the exact Clippy command passed. The focused
  `cargo llvm-cov` report remains affected by pre-existing Redis code in this
  large module (60.94% aggregate), while every approved HP/EC has direct unit
  evidence below.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Legacy JSON defaults to the legacy route and `SubtitleJob::new` remains legacy | `crates/jobs/src/lib.rs::tests::legacy_subtitle_job_json_defaults_to_legacy_route` | passed |
| HP-2 | Happy path | Localization serialization and translation payload retain versioned route and full UUID identity | `crates/jobs/src/lib.rs::tests::localization_subtitle_job_serializes_its_versioned_route`; `crates/jobs/src/lib.rs::tests::translation_job_serializes_the_full_durable_identity`; `crates/jobs/src/lib.rs::tests::in_memory_translation_queue_records_jobs` | passed |
| HP-3 | Happy path | The same subtitle UUID derives a stable initial translation request ID | `crates/jobs/src/lib.rs::tests::initial_translation_request_id_is_deterministic_and_canonical` | passed |
| EC-1 | Edge case | Unknown route strings fail deserialization rather than selecting a fallback | `crates/jobs/src/lib.rs::tests::unknown_subtitle_post_ready_route_fails_deserialization` | passed |
| EC-2 | Edge case | Initial derivation uses the canonical lowercase hyphenated UUID name | `crates/jobs/src/lib.rs::tests::initial_translation_request_id_is_deterministic_and_canonical` | passed |

### Owner final verification

- Owner: Codex (primary agent and approved cloud-route implementer)
- Date: 2026-08-13
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior; I also confirmed the sole Gemma nit against the actual toolchain and kept the implementation inside the versioned job-contract boundary.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-jobs`;
  `cargo clippy -p dubbridge-jobs --all-targets -- -D warnings`; `cargo fmt --check`;
  `cargo llvm-cov -p dubbridge-jobs --lib --summary-only`; `git diff --check`.

- Review artifact: `docs/audit/gemma-evidence/S-150-T2c-i.json`

**S-150-T2c-i status: `[x] Done`**

---

## S-150-T2c-ii: Exact persisted subtitle resolver

**Type:** development
**Effort:** M (RRI 37 — Moderate)
**Decomposed from:** S-150-T2c
**Depends on:** S-150-T2b-ii-c
**Status:** [ ] Planned — approval pending

**RRI evidence:** `docs/audit/s-150-t2c-decomposition-rri.md`

**Happy paths considered:**

- **HP-1:** A persisted `Subtitle` is recovered by its exact asset and
  word-alignment-parent artifact boundary for deterministic replay identity.

**Edge cases considered:**

- **EC-1:** A missing or wrong-kind parent produces no substitute artifact and
  fails through the repository's typed error path.

**Acceptance criteria:** Add only a fail-closed exact `Subtitle` lookup API and live
PostgreSQL coverage for its composite identity. Do not insert, mutate readiness,
fan out, or inspect queue state.

**Files expected to change:** `crates/db/src/subtitle_repo.rs` and
`apps/api/tests/subtitle_repo_test.rs`.

**Evidence to emit:** RRI, phase reviews, focused PostgreSQL evidence, Reflection
log, unit coverage certification, and owner verification.

**Status artifacts affected:** This ledger and the S-150 plan.

**Stop condition:** Stop after exact-lookup tests. Do not add runtime routing.

---

## S-150-T2c-iii: Translation dispatch acknowledgement transition

**Type:** development
**Effort:** L (RRI 43 — Med-high)
**Decomposed from:** S-150-T2c
**Depends on:** S-150-T2b-ii-c
**Status:** [ ] Planned — approval pending

**RRI evidence:** `docs/audit/s-150-t2c-decomposition-rri.md`

**Happy paths considered:**

- **HP-1:** A successful enqueue records `acknowledged` for exactly one durable
  dispatch identity.

**Edge cases considered:**

- **EC-1:** A duplicate acknowledgement or failure after acknowledgement does not
  reopen, overwrite, or corrupt the outbox row.

**Acceptance criteria:** Add only the guarded acknowledgement repository transition
and its typed result contract; preserve the failure transition and exact composite
identity. No queue connection, worker, route discriminator, or provider execution
is in scope.

**Files expected to change:** `crates/db/src/translation_delivery_repo.rs` and
`apps/api/tests/translation_delivery_repo_test.rs`.

**Evidence to emit:** RRI, phase reviews, ADR-038 receipt, live PostgreSQL state
matrix, Reflection log, unit coverage certification, and owner verification.

**Status artifacts affected:** This ledger and the S-150 plan.

**Stop condition:** Stop after acknowledgement-transition tests. Do not wire Redis.

---

## S-150-T2c-iv: Localization route branch and durable target fan-out

**Type:** development
**Effort:** L (RRI 48 — Med-high)
**Decomposed from:** S-150-T2c
**Depends on:** S-150-T2c-i, S-150-T2c-ii, S-150-T2c-iii
**Status:** [ ] Planned — approval pending

**RRI evidence:** `docs/audit/s-150-t2c-decomposition-rri.md`

**Happy paths considered:**

- **HP-1:** The localization route resolves the exact subtitle and creates one
  durable dispatch intent per eligible target while legacy jobs still enqueue their
  legacy review.

**Edge cases considered:**

- **EC-1:** The localization route never calls null-artifact review enqueue and
  leaves a failed target independent from its siblings.

**Acceptance criteria:** Branch only at the extracted post-ready seam; use T2c-i
contracts, T2c-ii exact resolver, and T2b-ii persistence to build durable per-target
work. Keep queue transport injected and in-memory only; do not connect Redis or run
a translation worker.

**Files expected to change:** `apps/worker-runner/src/subtitle_runtime.rs` and
`apps/worker-runner/src/subtitle_runtime_tests.rs`.

**Evidence to emit:** RRI, phase reviews, ADR-038 receipt, route/fan-out test
evidence, Reflection log, unit coverage certification, and owner verification.

**Status artifacts affected:** This ledger and the S-150 plan.

**Stop condition:** Stop after route/fan-out tests. Do not alter runner topology.

---

## S-150-T2c-v: Redis translation queue and worker topology

**Type:** development
**Effort:** L (RRI 50 — Med-high)
**Decomposed from:** S-150-T2c
**Depends on:** S-150-T2c-i, S-150-T2c-iii, S-150-T2c-iv
**Status:** [ ] Planned — approval pending

**RRI evidence:** `docs/audit/s-150-t2c-decomposition-rri.md`

**Happy paths considered:**

- **HP-1:** A durable eligible dispatch is enqueued to the dedicated translation
  namespace and then acknowledged with its exact identity.

**Edge cases considered:**

- **EC-1:** Redis enqueue failure records only that dispatch's `enqueue_failed`
  state and does not start provider execution.

**Acceptance criteria:** Implement the Redis adapter, worker-runner queue wiring,
and enqueue/failure/acknowledgement seam using T2c-i and T2c-iii contracts. Prove
namespace separation and fail-closed Redis errors. Do not register provider
execution, subprocesses, or S-150 review rows.

**Files expected to change:** `crates/jobs/src/lib.rs`,
`apps/worker-runner/src/translation_enqueue.rs`, `apps/worker-runner/src/main.rs`,
and `apps/worker-runner/src/runner_topology_tests.rs`.

**Evidence to emit:** RRI, phase reviews, ADR-038 receipt, Redis/in-memory queue
evidence, Reflection log, unit coverage certification, and owner verification.

**Status artifacts affected:** This ledger, the S-150 plan, and roadmap.

**Stop condition:** Stop after queue-topology tests. Do not start T3a.

---

## S-150-T3a: Translation provider/subprocess contract

**Type:** development
**Effort:** L (provisional RRI 42 — Med-high; recompute before presentation)
**Depends on:** S-150-T2c
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
