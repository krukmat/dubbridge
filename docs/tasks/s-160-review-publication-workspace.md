# Tasks: S-160 — Human Review & Publication Workspace

**Plan:** `docs/plan/s-160-review-publication-workspace.md`
**Roadmap phase:** `S-160` (depends on `S-105`).
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-008, ADR-018, ADR-023, ADR-024, ADR-006, ADR-027, ADR-030.

> **Namespace.** This phase uses the **`S-160-T`** prefix (`S-160-T0`–`S-160-T7`). Always fully
> qualify cross-slice references (`S-160-T2`, `S-100-T1`), never bare `T2`.

> **RRI provenance.** Every RRI below was computed with `python3 scripts/rri.py`
> (platform `dubbridge`) at planning time, not by hand. Final RRI is recomputed at
> presentation. `S-160-T1` was decomposed 2026-06-13 into `T1a`/`T1b`/`T1c` after
> recomputing the real implementation surface (`RRI 77`, `F >= 4 && K >= 3` trigger).
> `S-160-T2` and `S-160-T4` remain in **Complex (56–70)** and therefore require a
> reviewed plan before implementation — this ledger + the plan provide it.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
S-160-T0 (BDD) ─▶ S-160-T0b (ADR) ─▶ S-160-T1a (migration SQL)
  ─▶ S-160-T1b (domain entity) ─▶ S-160-T1c (DB repo)
  ─▶ S-160-T2 (state machine + publication gate) ─▶ S-160-T3 (API)
  ─▶ S-160-T4 (notifications) ─▶ S-160-T6 (complete mobile reviewer)
  ─▶ S-160-T7 (mock fixtures + Maestro + docs)

S-160-T5 (web review console) = cancelled / superseded by S-160-T6
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| S-160-T0 | BDD `.feature` specs + mapping | — | 11 | Low | S |
| S-160-T0b | ADR authoring: review/decision/publication gate model (X23 → X-S-160-1) | S-160-T0 | 18 | Low | S | ✅ done 2026-06-13 |
| S-160-T1 | ~~Schema + domain + repos (review/decisions/publications)~~ decomposed → T1a + T1b + T1c | S-160-T0b | 77 | High | XL | decomposed 2026-06-13 |
| S-160-T1a | Migration SQL: `0014`/`0015`/`0016` review schema | S-160-T0b | 54 | Med-high | L | ✅ done 2026-06-13 |
| S-160-T1b | Domain entity: `review.rs` — task, verdict, publication-state derivation | S-160-T1a | 29 | Moderate | M | ✅ done 2026-06-13 |
| S-160-T1c | DB repo: `review_repo.rs` — append decision, latest state, queue queries | S-160-T1b | 40 | Moderate | M |
| S-160-T2 | Review state machine + publication gate + audit | S-160-T1c | 66 | Complex | L |
| S-160-T3 | Review/publication API | S-160-T2 | 44 | Med-high | L |
| S-160-T4 | Notifications mechanism (table + emit + push) | S-160-T3 | 66 | Complex | L |
| S-160-T5 | Web review console — cancelled / superseded | — | 33 | Moderate | M |
| S-160-T6 | Complete mobile reviewer surface + push | S-160-T3, S-160-T4, S-105 | Recompute | — | L |
| S-160-T7 | Mock fixtures + Maestro + docs/roadmap sync | S-160-T6 | Recompute | — | M |

## Model resolution (capability → current vendor model)

| Band | Codex | Claude Code | Thinking |
|---|---|---|---|
| Low (0–25) | `GPT-5.2-Codex` | `Claude Haiku 4.5` | Off |
| Moderate (26–40) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` | Off |
| Med-high (41–55) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` (escalate to `Claude Opus 4.8` if it stalls) | On |
| Complex (56–70) | `GPT-5.2-Codex` | `Claude Opus 4.8` | On |
| High (71–85) | `GPT-5.2-Codex` | `Claude Opus 4.8` | On |

---

## S-160-T0 — BDD `.feature` specs + mapping

- **Status:** [x] Done — 2026-06-13
- **Type:** Planning / docs (BDD authoring) · **Effort:** S
- **RRI:** 11 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** — (BDD-first)
- **Objective:** Author the Gherkin specs defining acceptance for the review/publication
  slice and the mapping convention (scenario ID ⇄ executable evidence ⇄ `HP-#`/`EC-#`).
- **Inputs:** plan §D1–§D6; S-100 role model; S-010 artifact lineage; ADR-008.
- **Outputs:** `docs/bdd/s-160-review.feature`; mapping rows appended to `docs/bdd/README.md`.
- **Acceptance criteria:**
  - Each scenario has a stable ID and maps to executable evidence and ≥1 `HP-#`/`EC-#`.
  - Scenarios are behavioral; `make qa-docs` passes.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 1 → 0 | High |
  | F | 1 | 2 files | High |
  | D | 0 | docs/BDD authoring | High |
  | T | 2 | qa-docs validates references | High |
  | A | 0 | criteria + examples present | High |
  | K | 0 | no code coupling | High |
  | P | 0 | no API/security impact | High |
  | X | 2 | a few files | High |

  **Base 11 · penalties none · Final 11 → Low → auto-execute.**

- **BDD scenarios to author (Gherkin):**

  ```gherkin
  Feature: Human review and publication

    Scenario: SC-REVIEW-1 Reviewer sees their queue
      Given I am an org member with the reviewer role
      When I open the review queue
      Then I see the review tasks assigned to my org's projects

    Scenario: SC-REVIEW-2 Approve a derived output
      Given I am reviewing a pending review task
      When I approve it with a comment
      Then the task becomes approved and the decision is recorded immutably

    Scenario: SC-REVIEW-3 Reject a derived output
      Given I am reviewing a pending review task
      When I reject it with a comment
      Then the task becomes rejected and cannot be published

    Scenario: SC-PUBLISH-1 Publish a reviewed asset
      Given a review task is approved
      When I publish its asset/target
      Then a publication record is created and audited

    Scenario: SC-PUBLISH-2 Publication blocked without approval
      Given a review task is pending or rejected
      When I attempt to publish
      Then publication is refused with a clear review-required error

    Scenario: SC-NOTIFY-1 Reviewer notified of assignment
      Given a review task is assigned to me
      When the assignment happens
      Then I receive a notification referencing the task
  ```

- **Completion summary:**
  - Authored `docs/bdd/s-160-review.feature` with six stable scenarios:
    `SC-REVIEW-1/2/3`, `SC-PUBLISH-1/2`, and `SC-NOTIFY-1`.
  - Added the `S-160` mapping table to `docs/bdd/README.md` using the
    mobile-only and backend-evidence convention.
  - Verified documentation consistency with `make qa-docs`.

- **Owner final verification:**
  - Owner: `Codex`
  - Date: `2026-06-13`
  - Statement: I verified the `S-160` scenarios exist with stable IDs in
    `docs/bdd/s-160-review.feature`, the mapping rows were added to
    `docs/bdd/README.md`, and the documentation checks pass.
  - Commands run: `make qa-docs`

---

## S-160-T0b — ADR authoring: review/decision/publication gate model (X23 → X-S-160-1)

- **Status:** [x] Done — 2026-06-13
- **Type:** Architecture decision · **Effort:** S
- **RRI:** 18 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-160-T0, S-100 (org/role model)
- **Blocks:** S-160-T1a, S-160-T2 — **neither may start until this ADR is merged**
- **Objective:** Author and merge the ADR that defines the review/decision/publication gate
  model: how review tasks are created and assigned, how append-only decisions derive
  task state, and how the publication gate enforces approval as a fail-closed precondition
  (ADR-008 spirit). Closes X23 / X-S-160-1.
- **Inputs:**
  - ADR-008 — fail-closed precondition posture
  - ADR-018 — durable audit obligation
  - `infra/migrations/0007` — append-only governance posture (rights_records)
  - `docs/plan/s-160-review-publication-workspace.md` §D1–§D3
  - S-100-T0b ADR (role model — reviewer role gate)
- **Outputs:**
  - `docs/adr/ADR-030-review-decision-ledger-and-fail-closed-publication-gate.md` — decision record covering:
    - Review task lifecycle: Pending → Approved | Rejected (state = latest decision row)
    - `review_decisions` is append-only; no UPDATE/DELETE paths
    - Publication gate: `publications` row only created when latest decision = Approved
    - Unknown verdict rejected at decode boundary (fail-closed)
    - Reviewer role required (from S-100 role model) — org-scoped
    - Every decision + publication emits an `audit_events` row (ADR-018)
    - Forward dependency: operates on fixtures until S-140/S-150 artifacts land
  - ADR index entry added to `docs/adr/README.md`
- **Acceptance criteria:**
  - ADR file present in `docs/adr/` with a real sequential number.
  - `docs/adr/README.md` index updated.
  - ADR text covers: task lifecycle, append-only decision invariant, publication gate,
    fail-closed posture, role gate, audit obligation, and S-140/S-150 forward dependency.
  - `make qa-docs` passes.
- **Completion summary:**
  - Authored `ADR-030` to define the review-task lifecycle, append-only decision
    ledger, org-scoped reviewer authorization, and fail-closed publication gate.
  - Updated `docs/adr/README.md` and the S-160 plan/roadmap references to mark
    `X23 / X-S-160-1` closed.
  - Verified ADR/document consistency with `make qa-docs`.

- **Owner final verification:**
  - Owner: `Codex`
  - Date: `2026-06-13`
  - Statement: I verified `ADR-030` exists, the ADR index and canonical S-160
    references were synchronized, and the documentation checks pass.
  - Commands run: `make qa-docs`

---

## S-160-T1 — Schema + domain + repos (review tasks, decisions, publications)

- **Status:** [~] Decomposed into `S-160-T1a`/`T1b`/`T1c` — 2026-06-13
- **Type:** Historical parent task (do not implement directly) · **Effort:** XL
- **RRI:** 77 → band **High (71–85)** → **Characterization/evidence + diff review + mandatory decomposition.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-160-T0b
- **Objective:** Historical aggregate only. The original schema/domain/repo scope exceeded the
  decomposition gate and was split into `S-160-T1a`, `S-160-T1b`, and `S-160-T1c`.
- **Decomposition trigger:** realistic implementation footprint raised the task to `RRI 77`,
  activating both `RRI > 70` and `F >= 4 && K >= 3` from `docs/policies/RRI_POLICY.md`.
- **Disposition:** implement the child tasks only; `S-160-T2` depends on `S-160-T1c`.

---

## S-160-T1a — Migration SQL: `0014`/`0015`/`0016` review schema

- **Status:** [x] Done — 2026-06-13
- **Type:** Development (SQL migrations) · **Effort:** L
- **RRI:** 54 → band **Med-high (41–55)** → **Plan + explicit acceptance criteria before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking On
- **Depends on:** S-160-T0b
- **Objective:** Create the persisted review/publication schema as three migrations:
  `review_tasks`, append-only `review_decisions`, and `publications`.
- **Inputs:** `infra/migrations/0013_create_voice_consents.sql`, `infra/migrations/0007_harden_governance_invariants.sql`, ADR-008, ADR-018, ADR-030.
- **Outputs:**
  - `infra/migrations/0014_create_review_tasks.sql`
  - `infra/migrations/0015_create_review_decisions.sql`
  - `infra/migrations/0016_create_publications.sql`
  - `apps/api/tests/review_schema_test.rs`
- **Acceptance criteria:**
  - `review_tasks` stores the reviewable unit (`asset_id`, target/scope reference, assignee, timestamps) with FK integrity.
  - `review_decisions` is append-only via RULES / equivalent DB-level protection; verdict constrained to supported values only.
  - `publications` references the governing review task and asset/target identity with uniqueness that prevents duplicate publication rows for the same governed unit.
  - Migrations apply cleanly on a fresh DB and after `0013_create_voice_consents.sql`.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 4 → 0 | High |
  | F | 2 | 3 files | High |
  | D | 4 | anchor: `infra/migrations` floor 4 | High |
  | T | 2 | migration checks exist in CI | High |
  | A | 0 | criteria present | High |
  | K | 4 | anchor: `infra/migrations` floor 4 | High |
  | P | 5 | schema/data impact floor 5 | High |
  | X | 1 | migration-only scope | High |

  **Base 44 · penalties auth_security (+10, P floor ≥ 4) · Final 54 → Med-high.**

- **Happy paths considered:**
  - HP-1: valid project/asset/target scope creates a review task row. (SC-REVIEW-1)
  - HP-2: append an `approved` decision row and preserve it as immutable history. (SC-REVIEW-2)
  - HP-3: create one publication row for the governed review task. (SC-PUBLISH-1)
- **Edge cases considered:**
  - EC-1: `UPDATE`/`DELETE` against `review_decisions` is blocked at the DB layer.
  - EC-2: unknown `verdict` storage value is rejected by constraint. (SC-PUBLISH-2)
  - EC-3: duplicate publication for the same governed review task is rejected.
  - EC-4: malformed relational scope (wrong project target or unlinked asset) is rejected.
- **Handoff prompt:**
  > S-160-T1a — create `0014_create_review_tasks.sql`, `0015_create_review_decisions.sql`,
  > and `0016_create_publications.sql`. Use the append-only governance posture from `0007` and
  > ADR-030. AC: constrained verdicts, append-only decisions, publication FK/uniqueness, clean
  > migration order after `0013`. Stop after SQL + verification; do not start `S-160-T1b`.

- **Completion summary:**
  - Added `review_tasks` as the governed review-unit anchor with composite FKs that enforce
    org/project scope, asset membership through `project_assets`, target-language scope, and
    assignee membership when present.
  - Added append-only `review_decisions` with strict `verdict` storage constraints and a
    latest-decision lookup index for the downstream derived-state repo/gate work.
  - Added `publications` anchored to `review_task_id`, with one-publication-per-governed-unit
    uniqueness.
  - Added `apps/api/tests/review_schema_test.rs` to verify valid scope insertion, append-only
    decision behavior, strict verdict rejection, cross-project target rejection, project-asset
    FK rejection, and duplicate publication rejection.

### Reflection log

Required passes: 3 (`54` → `Med-high`)

#### Pass 1

- **Draft verdict:** Initial migrations created the three tables with baseline FKs and append-only rules.
- **Critique findings:** Project/org and target/project scope were not enforced tightly enough for `review_tasks`, which would make `T1c` responsible for rejecting malformed cross-project review units.
- **Revisions applied:** Added composite uniqueness on `projects (id, org_id)` and `target_languages (id, project_id)`, then used composite FKs in `review_tasks` to bind org/project scope, project-assets membership, and target-language scope at the DB layer.

#### Pass 2

- **Draft verdict:** Scope FKs were correct; append-only and publication shape still needed fail-closed review.
- **Critique findings:** The decision ledger needed storage-level protection against silent mutation, and publication needed a single governed anchor rather than duplicated asset/target columns that could drift.
- **Revisions applied:** Added `review_decisions` RULES to block `UPDATE`/`DELETE`, strict `verdict`/`state` `CHECK`s, a latest-decision index, and a unique `review_task_id` anchor in `publications`.

#### Pass 3

- **Draft verdict:** Schema shape was stable and ready for verification.
- **Critique findings:** The approved `EC-4` required explicit evidence for malformed relational scope, not just a happy-path migration run.
- **Revisions applied:** Added `apps/api/tests/review_schema_test.rs` with six DB-backed schema tests covering valid insertion, append-only no-op mutation attempts, unknown verdict rejection, target-language cross-project rejection, asset/project FK rejection, and duplicate publication rejection.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid project/asset/target scope creates one review task | `apps/api/tests/review_schema_test.rs::review_task_accepts_valid_project_asset_target_scope` | passed |
| HP-2 | Happy path | approved review decision row is inserted and preserved as immutable history | `apps/api/tests/review_schema_test.rs::review_decisions_are_append_only_noops_for_update_and_delete` | passed |
| HP-3 | Happy path | first publication row for a governed review task is accepted | `apps/api/tests/review_schema_test.rs::publications_reject_duplicate_review_task_rows` | passed |
| EC-1 | Edge case | `UPDATE`/`DELETE` against `review_decisions` becomes a no-op; prior row remains intact | `apps/api/tests/review_schema_test.rs::review_decisions_are_append_only_noops_for_update_and_delete` | passed |
| EC-2 | Edge case | unknown stored verdict is rejected by DB constraint | `apps/api/tests/review_schema_test.rs::review_decision_rejects_unknown_verdict` | passed |
| EC-3 | Edge case | duplicate publication for the same governed review task is rejected | `apps/api/tests/review_schema_test.rs::publications_reject_duplicate_review_task_rows` | passed |
| EC-4 | Edge case | invalid relational scope is rejected: target language from another project or asset not linked to the project | `apps/api/tests/review_schema_test.rs::review_task_rejects_target_language_from_another_project`, `apps/api/tests/review_schema_test.rs::review_task_rejects_asset_not_linked_to_project` | passed |

### Owner final verification

- Owner: `Codex`
- Date: 2026-06-13
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`, `cargo test -p dubbridge-api --test review_schema_test`

---

## S-160-T1b — Domain entity: `review.rs`

- **Status:** [x] Done — 2026-06-13
- **Type:** Development (Rust domain) · **Effort:** M
- **RRI:** 29 → band **Moderate (26–40)** → **Confirm tests exist in the affected area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-160-T1a
- **Objective:** Add the review domain model and strict decode boundary used by the repo and
  gate layers: task identity, verdict, derived review state, and publication status.
- **Inputs:** ADR-030, `crates/domain/src/consent.rs`, `crates/domain/src/rights.rs`.
- **Outputs:**
  - `crates/domain/src/review.rs`
  - `crates/domain/src/lib.rs`
  - Unit tests for strict verdict/state decoding and latest-decision derivation
- **Acceptance criteria:**
  - `review.rs` defines typed verdict/state/publication primitives with strict parsing.
  - Latest decision derives the current task state deterministically.
  - Unknown persisted verdict/state values fail closed at decode time.
  - Unit tests cover the derivation and strict-decode paths.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 8 → 1 | High |
  | F | 1 | 2 files | High |
  | D | 2 | anchor: `crates/domain` floor 2 | High |
  | T | 2 | partial domain tests | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | internal domain module | High |
  | P | 2 | typed internal contract | High |
  | X | 2 | one module + crate export | High |

  **Base 29 · penalties none · Final 29 → Moderate.**

- **Happy paths considered:**
  - HP-1: latest persisted decision `approved` derives `ReviewTaskState::Approved`. (SC-REVIEW-2)
  - HP-2: absence of decisions derives `ReviewTaskState::Pending`. (SC-REVIEW-1)
- **Edge cases considered:**
  - EC-1: unknown verdict string from storage fails decode instead of defaulting. (SC-PUBLISH-2)
  - EC-2: rejected latest decision derives `ReviewTaskState::Rejected`, never publishable. (SC-REVIEW-3)
  - EC-3: unknown persisted publication state fails decode.
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> Pending
    Pending --> Approved: latest decision = approve
    Pending --> Rejected: latest decision = reject
  ```

- **Handoff prompt:**
  > S-160-T1b — add `crates/domain/src/review.rs` and export it from `lib.rs`. Implement typed
  > verdict/state/publication primitives, latest-decision derivation, and strict decode. AC:
  > no silent fallback on unknown values; unit tests cover derive/decode behavior. Stop after
  > tests; do not start `S-160-T1c`.

- **Completion summary:**
  - Added `crates/domain/src/review.rs` with `ReviewTaskId`, `ReviewVerdict`,
    `ReviewTaskState`, `PublicationStatus`, `ReviewTask`, `ReviewDecisionRow`,
    `PublicationRow`, and `derive_review_state`.
  - Exported the module from `crates/domain/src/lib.rs`.
  - Kept all decode boundaries fail-closed via explicit `FromStr` implementations and
    added `is_publishable()` as the pure gate helper the next repo/service tasks can consume.
  - Verified the package test suite and coverage, with `review.rs` at 96.15% line coverage
    and `dubbridge-domain` overall at 93.17% line coverage.

### Reflection log

Required passes: 2 (`29` → `Moderate`)

#### Pass 1

- **Draft verdict:** Initial domain model covered verdicts, task state derivation, and publication status parse/display paths.
- **Critique findings:** `T1c` and `T2` would need stable typed anchors, not just enums, so leaving out `ReviewTaskId`, `ReviewDecisionRow`, and `PublicationRow` would force downstream tasks to recreate domain semantics around raw UUIDs and strings.
- **Revisions applied:** Added the missing typed identifiers and row structs, and kept their fields aligned with the `S-160-T1a` schema so the repo layer can map directly without inventing a second model.

#### Pass 2

- **Draft verdict:** Typed model was complete and tests existed for derive/decode behavior.
- **Critique findings:** The fail-closed posture needed to cover more than `ReviewVerdict`; unknown derived-state or publication-state strings also needed explicit rejection, and the publish gate needed a pure boolean helper instead of duplicating `Approved` checks later.
- **Revisions applied:** Added `FromStr` for `ReviewTaskState` and `PublicationStatus`, plus `ReviewTaskState::is_publishable()`, and extended tests to cover unknown task/publication values and pending/rejected behavior.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | latest persisted decision `approved` derives `ReviewTaskState::Approved` | `crates/domain/src/review.rs::tests::hp1_latest_approved_derives_approved_state` | passed |
| HP-2 | Happy path | absence of decisions derives `ReviewTaskState::Pending` | `crates/domain/src/review.rs::tests::hp2_no_decisions_derives_pending_state` | passed |
| EC-1 | Edge case | unknown verdict string fails decode instead of defaulting | `crates/domain/src/review.rs::tests::ec1_unknown_verdict_fails_closed` | passed |
| EC-2 | Edge case | latest rejected decision derives `ReviewTaskState::Rejected` and is not publishable | `crates/domain/src/review.rs::tests::ec2_latest_rejected_is_not_publishable` | passed |
| EC-3 | Edge case | unknown persisted publication state fails decode | `crates/domain/src/review.rs::tests::ec3_unknown_publication_state_fails_closed` | passed |

### Owner final verification

- Owner: `Codex`
- Date: 2026-06-13
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cargo fmt --all`, `cargo test -p dubbridge-domain`, `cargo llvm-cov --package dubbridge-domain --summary-only`

---

## S-160-T1c — DB repo: `review_repo.rs`

- **Status:** [ ] Not started
- **Type:** Development (Rust DB) · **Effort:** M
- **RRI:** 40 → band **Moderate (26–40)** → **Confirm tests exist in the affected area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-160-T1b
- **Objective:** Implement persistence and query operations for review tasks/decisions/publications,
  using the new schema and domain types.
- **Inputs:** `crates/db/src/consent_repo.rs`, `crates/db/src/workspace_repo.rs`, ADR-030.
- **Outputs:**
  - `crates/db/src/review_repo.rs`
  - `crates/db/src/lib.rs`
  - Repo-focused tests covering append, latest-state derivation, and queue queries
- **Acceptance criteria:**
  - Repo can insert review tasks and append approve/reject decisions without any mutation path for existing decision rows.
  - Queue/list queries are org/project scoped and return derived state from the latest decision row.
  - Publication persistence reads the same governed identity expected by the gate layer.
  - Tests prove append-only behavior and latest-state derivation.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 10 → 1 | High |
  | F | 2 | 3 files | High |
  | D | 3 | anchor: `crates/db` floor 3 | High |
  | T | 2 | partial repo coverage | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | DB/repo coupling | High |
  | P | 3 | persisted internal contract | High |
  | X | 2 | repo + export + tests | High |

  **Base 40 · penalties none · Final 40 → Moderate.**

- **Happy paths considered:**
  - `HP-1`: append approve decision through repo → queue/read model reflects approved. (SC-REVIEW-2)
- **Edge cases considered:**
  - `EC-1`: second decision supersedes current derived state without mutating prior history. (SC-REVIEW-3)
  - `EC-2`: query outside org/project scope returns no review tasks. (SC-REVIEW-1)
- **Diagram:**

  ```mermaid
  flowchart LR
    RT[(review_tasks)] --> RR[review_repo]
    RD[(review_decisions)] --> RR
    RR --> DS[derived current state]
    RR --> Q[queue query]
  ```

- **Handoff prompt:**
  > S-160-T1c — implement `crates/db/src/review_repo.rs` and export it from `crates/db/src/lib.rs`.
  > Cover task insert, append-only decisions, derived latest state, and queue reads. AC: no
  > in-place mutation of decision history, tests prove derived-state behavior. Stop after tests;
  > do not start `S-160-T2`.

---

## S-160-T2 — Review state machine + publication gate + audit

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 66 → band **Complex (56–70)** → **Plan first; human reviews the plan; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-160-T1c
- **Objective:** Implement the review transition rules and the **fail-closed publication
  gate**: a publication cannot be created unless its governing review task is `approved`.
  Emit durable audit on decisions and publish attempts. (Plan §D2.)
- **Inputs:** `review_repo` (S-160-T1c), `crates/audit` emission, ADR-008 (rights gate as the
  template), `finalize_ingestion_core` (reusable-gate pattern).
- **Outputs:**
  - `apps/api/src/services/review_gate.rs` (transition rules + `require_approved_for_publish`).
  - Audit rows for every decision and every publish attempt (allowed and refused).
  - Tests: approve→publish allowed; pending/rejected→publish refused + audited.
- **Acceptance criteria:**
  - Publish against a non-approved task is refused; the refusal is audited. (SC-PUBLISH-2)
  - Publish against an approved task succeeds and is audited. (SC-PUBLISH-1)
  - The gate is a reusable service (S-180 can call it directly).
  - ≥90% coverage; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 18 → 2 | High |
  | F | 2 | 4 files | High |
  | D | 4 | anchor: `crates/audit` (ADR-008, ADR-018) floor 4 | High |
  | T | 2 | audit/domain area has tests | High |
  | A | 1 | transition matrix minor ambiguity | High |
  | K | 4 | anchor: `crates/audit` floor 4 | High |
  | P | 5 | anchor: `crates/audit` floor 5 (governance/audit) | High |
  | X | 3 | service + domain + audit + repo | High |

  **Base 56 · penalties auth_security (+10, P floor ≥ 4) · Final 66 → Complex → plan-first.**

- **Happy paths considered:**
  - `HP-1`: approved task → publish → `publications` row + audit. (SC-PUBLISH-1)
- **Edge cases considered:**
  - `EC-1`: pending task → publish refused + audit, no publication row. (SC-PUBLISH-2)
  - `EC-2`: rejected task → publish refused + audit. (SC-REVIEW-3 + SC-PUBLISH-2)
  - `EC-3`: re-publish of an already-published task → idempotent / refused, no duplicate.
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> pending
    pending --> approved: approve decision
    pending --> rejected: reject decision
    approved --> published: publish (gate passes) + audit
    pending --> blocked: publish attempt -> refused + audit
    rejected --> blocked: publish attempt -> refused + audit
  ```

- **Handoff prompt:**
  > S-160-T2 — review transitions + fail-closed publication gate + audit. Docs: this ledger +
  > plan §D2, ADR-008/018. Add `apps/api/src/services/review_gate.rs` with
  > `require_approved_for_publish`; audit every decision + publish attempt. AC: SC-PUBLISH-1/2,
  > reusable gate, ≥90% cov. Stop after tests; do not start S-160-T3.

---

## S-160-T3 — Review/publication API

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 44 → band **Med-high (41–55)** → **Plan + explicit acceptance criteria before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
  (escalate to `Claude Opus 4.8` if it stalls) · thinking On
- **Depends on:** S-160-T2
- **Objective:** Expose the review queue, decide (approve/reject), and publish endpoints,
  org/role-guarded (S-100-T2), calling the S-160-T2 gate.
- **Inputs:** `review_gate` (S-160-T2), org guard (S-100-T2), `apps/api` route patterns.
- **Outputs:** `apps/api/src/routes/review.rs` + `dto/review.rs`; endpoints
  (`GET queue`, `POST decision`, `POST publish`); route/integration tests.
- **Acceptance criteria:**
  - Queue is scoped to the reviewer's org/projects (S-100 role). (SC-REVIEW-1)
  - Decide appends a decision; publish goes through the gate (refused if not approved).
  - ≥90% coverage; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 14 → 2 | High |
  | F | 2 | 3 files | High |
  | D | 3 | anchor: `crates/db` (ADR-006, ADR-018) floor 3 | High |
  | T | 2 | route/repo tests exist | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | anchor: `crates/db` floor 3 | High |
  | P | 3 | new write endpoints (gated) | High |
  | X | 3 | routes + dto + repo | High |

  **Base 44 · penalties none · Final 44 → Med-high → plan+AC before approval.**

- **Happy paths considered:**
  - `HP-1`: reviewer approves via API → decision recorded; queue reflects approved. (SC-REVIEW-2)
- **Edge cases considered:**
  - `EC-1`: non-reviewer decides → 403 (role-guarded), no decision row.
  - `EC-2`: publish a non-approved task via API → refused + audited. (SC-PUBLISH-2)
- **Diagram:**

  ```mermaid
  flowchart LR
    C[mobile] -->|POST /api/review/{id}/decision| G[gateway] --> A[apps/api review routes]
    A --> M[org_scope guard] --> GT[review_gate] --> DB[(review_repo)]
    GT --> AU[(audit_events)]
  ```

- **Handoff prompt:**
  > S-160-T3 — review/publication API. Docs: this ledger + plan §D2–§D3. Add `routes/review.rs`
  > + dto; queue/decide/publish, role-guarded, calling the S-160-T2 gate. AC: SC-REVIEW-1/2 +
  > SC-PUBLISH-2, ≥90% cov, tests green. Stop after tests; do not start S-160-T4.

---

## S-160-T4 — Notifications mechanism (table + emit + push)

- **Status:** [ ] Not started
- **Type:** Development (Rust + SQL + RN) · **Effort:** L
- **RRI:** 66 → band **Complex (56–70)** → **Plan first; human reviews the plan; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-160-T3
- **Objective:** Add a `notifications` table, emission on assignment/decision/publish, and
  mobile push-token registration. Payloads carry references only (no PII). (Plan §D5.)
- **Inputs:** `infra/migrations/` (next free index 0017), `review_gate` emit points,
  ADR-018 (redaction), Expo push.
- **Outputs:**
  - `0017_create_notifications.sql` (recipient, kind, ref, read_at).
  - `crates/db/src/notification_repo.rs`; emit hooks at decision/publish/assignment.
  - `apps/api/src/routes/notifications.rs` (list/mark-read).
  - `mobile/src/push/registerPush.ts` (Expo push token registration).
  - Tests for emission + no-PII payload shape.
- **Acceptance criteria:**
  - A notification row is written on assignment/decision/publish; payload has no PII. (SC-NOTIFY-1)
  - Mobile registers a push token; list/mark-read endpoints work, owner-scoped.
  - ≥90% coverage on repo + emit; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 12 → 2 | High |
  | F | 2 | 4 files | High |
  | D | 4 | anchor: `infra/migrations` (ADR-008, ADR-018) floor 4 | High |
  | T | 2 | db area has tests | High |
  | A | 1 | push-delivery provider open (X-S-160-3) | High |
  | K | 4 | anchor: `infra/migrations` floor 4 | High |
  | P | 5 | anchor: `infra/migrations` floor 5 (schema/data) | High |
  | X | 3 | migration + repo + route + push | High |

  **Base 56 · penalties auth_security (+10, P floor ≥ 4) · Final 66 → Complex → plan-first.**

- **Happy paths considered:**
  - `HP-1`: assign a review task → notification row for the assignee; mobile receives push. (SC-NOTIFY-1)
- **Edge cases considered:**
  - `EC-1`: payload must not contain asset title/PII → asserted (reference only).
  - `EC-2`: list notifications for another user → empty/denied (owner-scoped).
- **Diagram:**

  ```mermaid
  flowchart LR
    GT[review_gate emit] --> NR[(notifications)]
    NR --> API[/api/notifications]
    GT --> PUSH[Expo push -> device]
  ```

- **Handoff prompt:**
  > S-160-T4 — notifications. Docs: this ledger + plan §D5, ADR-018. Add migration 0017,
  > `notification_repo`, emit hooks, `routes/notifications.rs`, `mobile/src/push/registerPush.ts`.
  > AC: rows on assign/decide/publish, no-PII payload, owner-scoped list, ≥90% cov. Stop after tests;
  > do not start S-160-T6.

---

## S-160-T5 — Web review console

- **Status:** [-] Cancelled / superseded by S-160-T6 — 2026-06-13
- **Type:** Development (TS/web) · **Effort:** M
- **RRI:** 33 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-160-T3, S-160-T4
- **Objective:** Historical proposal only. S-105 removed the authenticated web-console
  product surface before implementation.
- **Inputs:** None; do not implement.
- **Outputs:** `ReviewQueueScreen.tsx`, `ReviewDetailScreen.tsx`, `SideBySidePreview.tsx`;
  `data-testid`s (`review-queue-screen`, `review-detail-screen`, `review-approve`,
  `review-reject`, `publish-action`); component tests.
- **Acceptance criteria:**
  - Queue lists assigned tasks; detail shows preview + decision controls. (SC-REVIEW-1)
  - Approve/reject post decisions; publish disabled unless approved. (SC-REVIEW-2/3, SC-PUBLISH-2)
  - `data-testid`s present; `npm test` + typecheck green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 12 → 2 | High |
  | F | 2 | 4 files | High |
  | D | 2 | web UI + API integration | High |
  | T | 1 | web harness exists (S-100-T4) | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | API coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 3 | screens + component + test | High |

  **Base 33 · penalties none · Final 33 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: approve in detail → state approved; publish enabled. (SC-REVIEW-2, SC-PUBLISH-1)
- **Edge cases considered:**
  - `EC-1`: pending/rejected task → publish control disabled. (SC-PUBLISH-2)
  - `EC-2`: empty queue → empty-state, no error.
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> queue
    queue --> detail: open task
    detail --> decided: approve/reject
    decided --> published: publish (if approved)
  ```

- **Disposition:** All user-facing behavior is owned by S-160-T6.

---

## S-160-T6 — Complete mobile reviewer surface + push

- **Status:** [ ] Not started
- **Type:** Development (TS/RN) · **Effort:** L
- **RRI:** Recompute from the expanded mobile-only scope before presentation.
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-160-T3, S-160-T4, S-105
- **Objective:** Add the complete mobile review experience: inbox, original/derived
  comparison through an alternable or stacked view, approve/reject with comments,
  publish visible only when approved, notifications, and deep links.
- **Inputs:** `mobile/src/api/client.ts`, nav, S-160-T3 endpoints, `registerPush.ts` (S-160-T4).
- **Outputs:** `ReviewInboxScreen.tsx`, `ReviewDetailScreen.tsx`, comparison and
  decision/publish controls, nav/deep links, testIDs, and component tests.
- **Acceptance criteria:**
  - Inbox lists assigned tasks; detail compares original and derived content and
    posts approve/reject with a comment. (SC-REVIEW-1/2/3)
  - Publish action is visible only for approved state; backend remains authoritative.
  - A push notification deep-links to the relevant task. (SC-NOTIFY-1)
  - testIDs present; `npm test` + typecheck green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 10 → 1 | High |
  | F | 2 | 4 files | High |
  | D | 2 | mobile UI + API integration | High |
  | T | 1 | mobile harness exists | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | network/push coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 2 | screens + nav + test | High |

  **Base 31 · penalties none · Final 31 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: inbox → open task → approve → decision posted. (SC-REVIEW-2)
- **Edge cases considered:**
  - `EC-1`: push tapped → app deep-links to the task. (SC-NOTIFY-1)
  - `EC-2`: 401 → `auth.logout()` (transport contract preserved).
- **Diagram:**

  ```mermaid
  flowchart LR
    PUSH[push notification] --> INBOX[review-inbox-screen]
    INBOX --> DEC[review-detail-screen]
    DEC --> CMP[original / derived comparison]
    DEC --> API[/api/review/{id}/decision or publish]
  ```

- **Handoff prompt:**
  > S-160-T6 — complete mobile reviewer + push. Build inbox/detail, alternable or
  > stacked comparison, commented decisions, approved-only publish visibility, nav,
  > notifications and deep links. Keep publication fail-closed in backend.

---

## S-160-T7 — E2E fixtures + docs/roadmap sync

- **Status:** [ ] Not started
- **Type:** Development (Node fixture) / ops / docs · **Effort:** S
- **RRI:** 24 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-160-T6
- **Objective:** Extend the mock-gateway with review/publication/notification fixtures,
  author the mobile Maestro review flow, and sync status docs.
- **Inputs:** `mock-gateway-server.mjs`, S-160-T3/T4 contracts, S-055 env, `docs/plan/roadmap.md`.
- **Outputs:** `/api/*` review fixtures + `node --test`;
  `mobile/maestro/review.yaml`; roadmap row updated; X-S-160-1/2/3 recorded; BDD mapping closed.
- **Acceptance criteria:**
  - The mobile review flow passes against the deterministic mock-gateway, including the
    publish-blocked-without-approval narrative. (SC-PUBLISH-2)
  - `make qa-docs` green; status docs consistent; follow-ups recorded.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 4 → 0 | High |
  | F | 2 | 4 files | High |
  | D | 1 | fixtures + orchestration | High |
  | T | 2 | mock-gateway has `node --test` | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | process/fixture coupling | High |
  | P | 1 | dev/test + docs only | High |
  | X | 3 | fixtures + flows + docs | High |

  **Base 24 · penalties none · Final 24 → Low → auto-execute.**

- **Happy paths considered:**
  - `HP-1`: approve→publish flow passes; publish-blocked flow asserts the refusal. (SC-PUBLISH-1/2)
- **Edge cases considered:**
  - `EC-1`: `/api/*` review route without session → 401, no data.
  - `EC-2`: non-reviewer fixture session → decide denied in the flow.
- **Handoff prompt:**
  > S-160-T7 — E2E fixtures + docs sync. Docs: this ledger + plan + roadmap. Add mock-gateway
  > review/publication/notification `/api/*` + `node --test`,
  > `mobile/maestro/review.yaml`, sync roadmap + X-S-160-1/2/3. AC: flow passes, qa-docs green.
  > Stop after sync.

---

## Coverage contract

This ledger does **not** declare `Behavioral coverage contract: unit-v1`. Development
tasks (S-160-T1a…S-160-T6) still require the standard `Unit coverage certification` + `Owner
final verification` completion record per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
before being marked `[x] Done`. The BDD `.feature` scenarios (S-160-T0) are the behavioral
source of truth from which each task's `HP-#`/`EC-#` cases are derived.
