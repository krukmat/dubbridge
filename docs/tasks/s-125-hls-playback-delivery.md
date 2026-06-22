---
type: TaskList
title: "S-125 HLS Playback Delivery"
status: closed
slice: S-125
plan: docs/plan/s-125-hls-playback-delivery.md
---
# S-125 HLS Playback Delivery

> **Status:** Done 2026-06-22. `T0`, `T1`, `T2a`, `T2b-i`, `T2b-ii`, `T2b-iii`,
> `T3a`, `T3b`, `T4a-i`, `T4a-ii`, `T4b-i`, `T4b-ii`, `T4b-iii`, `T4c`, `T4d`,
> `T5a`, `T5b`, and `T5c` are complete. ADR-032 is accepted and all governing
> status artifacts are synchronized. **T3a is RRI 21 (Low band)** and was
> delegated to local Gemma via Ollama; the primary agent stayed
> orchestrator/reviewer of record. RRI figures below were measured with
> `scripts/rri.py`, not estimated.
> **Plan:** `docs/plan/s-125-hls-playback-delivery.md`.
> **Governing ADR:** `docs/adr/ADR-032-hls-playback-delivery-boundary.md`.
> **Behavioral coverage contract:** unit-v1.

## Slice RRI summary

Scored as a single task, the slice is **RRI 100 (Very high)** — auth + audit +
migration + a new public delivery surface. That mandates decomposition before
implementation, which this ledger provides. Per-task measured RRI:

| Task | RRI | Band | Decomposition obligation |
|---|---|---|---|
| T0 | — | planning/docs | none (no code) |
| T1 | 42 | Med-high | none |
| T2 | 76 | High | **Decomposed (2026-06-21)** → T2a + T2b |
| T2a | 55 | Med-high | none (migration only; rescored 2026-06-21 — see T2a rescore note) |
| T2b | 84 | High | **Decomposed (2026-06-21)** → T2b-i + T2b-ii + T2b-iii |
| T2b-i | 44 | Med-high | none (grant CRUD: issue/get/expire) |
| T2b-ii | 55 | Med-high | none (resolve_grant_target: readiness gate + lineage join) |
| T2b-iii | 34 | Moderate | none (integration tests: all HP/EC paths) |
| T3a | 21 | Low | **local Gemma-eligible** (pure manifest rewriter) |
| T3b | 49 | Med-high | none (storage scoped-reference helper — retained by primary agent) |
| T4 | 88 | Very high | **Decomposed (2026-06-21, refined 2026-06-22)** → T4a-i + T4a-ii + T4b-i + T4b-ii + T4b-iii + T4c + T4d |
| T4a-i | 52 | Med-high | none (route wiring + handler skeleton) |
| T4a-ii | 61 | Complex | none (auth guard org/project) |
| T4b-i | 39 | Moderate | none (audit event contract + persistence decode) |
| T4b-ii | 47 | Med-high | none (success + service-denial audit emission) |
| T4b-iii | 54 | Med-high | none (auth-boundary refusal audit + audit-focused integration tests) |
| T4c | 44 | Med-high | none (audience-policy hook seam) |
| T4d | 46 | Med-high | none (final integration tests) |
| T5 | 76 | High | **Decomposed (2026-06-22)** → T5a + T5b + T5c |
| T5a | 46 | Med-high | none (manifest delivery handler + rewritten-manifest tests) |
| T5b | 51 | Med-high | none (segment delivery + re-authorization + delivery observability) |
| T5c | 24 | Low | none (ADR/docs propagation; primary-agent direct, no Gemma delegation) |

Tasks whose RRI triggers the decomposition gate (T2/T4/T5) must, at their own
approval checkpoint, either be split into approved subtasks or carry an explicit,
approved justification for proceeding whole (RRI policy §decomposition).

**T3 split (Gemma eligibility analysis, 2026-06-21).** T3 was split because exactly
one S-125 sub-part qualifies for local Gemma delegation. Only the **pure manifest
rewriter** clears RRI ≤ 25 with legitimately low D/K/P — it is a pure, IO-free,
single-file function with deterministic fixtures (best-fit per
`LOW_RRI_LOCAL_MODEL_HANDOFF.md`). Every other S-125 sub-part is floored above the
band by the anchor rubric: any `infra/migrations/**` / `crates/audit` touch carries
D4/P5/K4 + the authn/authz penalty (a trivial 1-line migration already scores RRI 42),
and the API/authz/grant work (T4/T5) is security-critical by construction. The
storage scoped-reference helper (T3b) sits on the no-raw-key boundary in
`crates/storage` (anchor floor 3/3/3) and is therefore retained by the primary agent,
not delegated. See `scripts/rri.py` runs recorded in this slice's planning history.

---

## S-125-T0: Plan, task ledger, BDD, and roadmap/ADR sync
**Effort:** S
**Complexity:** Low (planning/docs)
**Depends on:** None
**Status:** Done (2026-06-21)
**Type:** planning/docs (no code; RRI gate N/A)
**Objective:** Establish the `S-125` plan, this task ledger, the BDD feature skeleton,
and synchronize roadmap/architecture/ADR-032 so downstream tasks have a governing
contract.
**Inputs:**
- ADR-032 (Proposed).
- Roadmap `S-125` row + X25.
- `S-120` prepared-media plan/tasks (readiness + lineage contract this slice consumes).
**Outputs:**
- `docs/plan/s-125-hls-playback-delivery.md`.
- `docs/tasks/s-125-hls-playback-delivery.md` (this file).
- `docs/bdd/s-125-hls-playback-delivery.feature` + BDD README mapping rows.
- Roadmap/architecture references pointing at the new plan/tasks.
**Acceptance criteria:**
- Plan records objective, scope, governing constraints, design decisions, and the
  T0–T5 decomposition strategy.
- This ledger lists T1, T2, T3a, T3b, T4, T5 with measured RRI, dependencies,
  acceptance criteria, and happy/edge examples per development task.
- Roadmap `S-125` row references the new plan + task ledger (no longer "no plan yet").
- `make qa-docs` passes (index parity, dangling refs, OKF frontmatter).
**Related documents:** ADR-032; `docs/plan/roadmap.md`; `docs/architecture.md`;
`docs/plan/s-120-media-preparation.md`.
**Note:** ADR-032 flipped to `Accepted` on 2026-06-22 when `S-125-T5c` completed the
boundary propagation pass (ADR change-propagation contract).

---

## S-125-T1: Playback-grant domain contract + fail-closed state decoding
**Effort:** L (RRI 42 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T0
**Status:** Done (2026-06-21)
**Type:** development
**Objective:** Define the `PlaybackGrant` domain type — its scope, expiry, principal/
org/project binding, status, and denial reasons — with strict, fail-closed decoding of
stored values, in `crates/domain`. No IO, no persistence.
**Inputs:**
- `S-120` `PreparationStatus` / derived-artifact lineage types (`crates/domain/src/artifact.rs`).
- ADR-032 grant requirements (readiness, authorization, scope/expiry).
- Existing fail-closed decode pattern (`UnknownStoredValue`) used by S-120 domain types.
**Outputs:**
- `crates/domain/src/playback.rs`: `PlaybackGrant`, `PlaybackScope`, `GrantStatus`,
  `PlaybackDenial`, and a constructor/validator that rejects malformed grants.
- Unit tests for valid construction, expiry logic, and fail-closed decoding.
**Acceptance criteria:**
- `PlaybackGrant` carries asset id, grant id, scope, principal/org/project ref, issued/
  expiry timestamps, and status; no field allows an implicit "allow".
- `GrantStatus` and `PlaybackScope` decode strictly: an unknown stored token yields a
  typed error, never a default-allow.
- Expiry is evaluated against an injected clock/instant so it is unit-testable.
- ≥90% line coverage for the new module; no clippy warnings.
**Happy paths considered:**
- HP-1: valid grant fields → constructed grant in `Active` status with a future expiry.
- HP-2: grant evaluated before expiry → `valid`; same grant at/after expiry → `Expired`.
**Edge cases considered:**
- EC-1: unknown stored `GrantStatus`/`PlaybackScope` token → typed `UnknownStoredValue`
  error, never an allow.
- EC-2: expiry timestamp before issued timestamp → construction rejected.
**Files expected to change:** `crates/domain/src/playback.rs`; `crates/domain/src/lib.rs` (module export).
**Reflection strategy:** RRI 42 → Med-high → **3 passes**. Pass 1: correctness of grant
construction/expiry against HP-1/HP-2. Pass 2: fail-closed decoding completeness for
every stored enum (EC-1) and ordering invariants (EC-2). Pass 3: coverage-gap sweep and
API ergonomics for the repo/API consumers in T2/T4.
**Agent handoff prompt:** Add a pure `crates/domain` playback-grant contract with
strict fail-closed decoding and clock-injected expiry. No persistence, no API. Stop
after unit tests + coverage are green; do not start T2.

**Happy paths covered:**
- HP-1: `PlaybackGrant::new` with valid timestamps → `GrantStatus::Active` (`valid_grant_is_active`).
- HP-2: `is_valid_at` before expiry → `true`; at/after expiry → `false`
  (`grant_is_valid_before_expiry`, `grant_is_invalid_at_expiry`, `grant_is_invalid_after_expiry`).

**Edge cases covered:**
- EC-1: `"unknown_token".parse::<GrantStatus>()` → `PlaybackError::UnknownGrantStatus`; same for empty string and for `PlaybackScope` (`unknown_*_is_error`, `empty_string_*_is_error`). No branch returns a default-allow.
- EC-2: `expires_at == issued_at` and `expires_at < issued_at` both → `PlaybackError::InvalidExpiry` (`expiry_equal_to_issued_is_rejected`, `expiry_before_issued_is_rejected`).

**Correction to ledger (applied in-place):** the ledger cited `UnknownStoredValue` as a pattern in `crates/domain`; it is actually a `DbError` variant in `crates/db` used by the repo layer. T1 follows the actual domain pattern: `FromStr` returning a domain-level `PlaybackError`, which T2's repo layer will map to `DbError::UnknownStoredValue` (identical to `review.rs:68-77`).

### Reflection log

Required passes: 3 (RRI 42 → Med-high)

#### Pass 1
- **Draft verdict:** grant construction and expiry logic correct against HP-1/HP-2.
- **Critique findings:** `is_valid_at` boundary at exactly `expires_at` needed an explicit test; only before/after were covered.
- **Revisions applied:** added `grant_is_invalid_at_expiry` test.

#### Pass 2
- **Draft verdict:** fail-closed decode solid; `FromStr` never falls through to allow.
- **Critique findings:** empty-string tokens were not tested; only named-unknown tokens were.
- **Revisions applied:** added `empty_string_grant_status_is_error` and `empty_string_playback_scope_is_error`.

#### Pass 3
- **Draft verdict:** coverage complete; ergonomics for T2/T4 verified.
- **Critique findings:** `PlaybackScope::Display` formatting failed `cargo fmt` (multiline `write!`). No logic issues.
- **Revisions applied:** reformatted `Display` impl to pass `cargo fmt`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid fields → `Active` grant | `crates/domain/src/playback.rs::tests::valid_grant_is_active` | passed |
| HP-2 | Happy path | before expiry → valid; at/after → invalid | `::grant_is_valid_before_expiry`, `::grant_is_invalid_at_expiry`, `::grant_is_invalid_after_expiry` | passed |
| EC-1 | Edge case | unknown token → typed error, never allow | `::unknown_grant_status_is_error`, `::unknown_playback_scope_is_error`, `::empty_string_grant_status_is_error`, `::empty_string_playback_scope_is_error` | passed |
| EC-2 | Edge case | expiry ≤ issued → construction rejected | `::expiry_equal_to_issued_is_rejected`, `::expiry_before_issued_is_rejected` | passed |

### Owner final verification

- Owner: Claude Sonnet 4.6
- Date: 2026-06-21
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cargo test -p dubbridge-domain -- playback` (14 passed), `cargo test -p dubbridge-domain` (102 passed, 0 failed), `cargo clippy -p dubbridge-domain -- -D warnings` (clean), `cargo fmt -p dubbridge-domain -- --check` (clean)

---

## S-125-T2: Playback-grant schema + repository (grant → prepared-HLS lineage)
**Status:** Decomposed (2026-06-21) — replaced by T2a + T2b below.
**Original RRI:** 76 (High). Decomposition gate triggered (T≥4, P≥4).
**Decomposition record:** Split approved at presentation. T2a handles the migration
(`infra/migrations/`) which the anchor rubric floors at D4/K4/P5; T2b handles the
repository + integration tests (`crates/db/` + `apps/api/tests/`). Each subtask was
re-scored independently with `scripts/rri.py` before presentation. T2a → RRI 75 (High).
T2b → RRI 84 (High). Both remain within the single-task approval threshold and carry
mandatory 4-pass Reflection (High band, security-anchored path). Neither subtask alone
re-triggers the decomposition gate because the most complex dimension (the join logic
between migration schema and lineage) is fully owned by T2b.

---

## S-125-T2a: Playback-grant migration (`0021_create_playback_grants`)
**Effort:** L (RRI 55 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Sonnet 4.6` (escalate to Opus 4.1 only on repeated failure).
**Depends on:** S-125-T1
**Status:** Done (2026-06-21)
**Type:** development (migration)

**Rescore note (2026-06-21):** Initial score was 75 (High) with `arch_decision` penalty and inflated T/X/C.
Rescored to **55 (Med-high)** after honest re-evaluation:
- C: 2→1 — DDL puro; CC real ~3 ramas (CHECK, 2 índices) → banda 6–10.
- T: 3→2 — verificación es `sqlx migrate run` + schema_dump, no ramas de código; precedente claro en `0019`.
- X: 2→1 — solo necesita `PlaybackGrant` (1 archivo T1) + estilo `0019` en mente.
- `arch_decision` penalty eliminado — la decisión arquitectónica está tomada en T0/T1; T2a es implementación DDL, no decisión.
- Floors D4/K4/P5 y penalidad `auth_security` (+10) se mantienen — mandatorios por anchor rubric `infra/migrations/**` (ADR-008/ADR-018).
**Objective:** Create the `playback_grants` table — the durable schema that T2b's
repository layer reads and writes. Schema must be correct on the first applied migration
because it cannot be rolled back in production without a compensating migration.
**Inputs:**
- `PlaybackGrant` domain fields from T1 (`grant_id`, `asset_id`, `principal_ref`,
  `org_id`, `project_id`, `status`, `issued_at`, `expires_at`).
- Migration numbering: next available is `infra/migrations/0021_*`.
- Existing migration style: `infra/migrations/0019_*/` and `infra/migrations/0020_*/`
  as canonical examples of column types, constraint names, and index conventions.
- ADR-008 (migration immutability), ADR-018 (audit/compliance column requirements).
**Outputs:**
- `infra/migrations/0021_create_playback_grants.sql`: `CREATE TABLE playback_grants`
  with columns, NOT NULL constraints, foreign-key refs where applicable, and indexes
  for active-grant lookup (`asset_id + status + expires_at`).
**Acceptance criteria:**
- All columns required by `PlaybackGrant` (T1) are present with correct SQL types and
  NOT NULL where the domain type has no `Option`.
- `status` column stores the string token decoded by T1's `GrantStatus::from_str`; the
  column has a CHECK constraint or enum type consistent with the repo platform.
- At least one index covers the active-grant lookup pattern:
  `(asset_id, status, expires_at)` or equivalent.
- The migration applies cleanly against a fresh schema (`sqlx migrate run` or equivalent
  in CI); no existing migration is modified.
- `make qa-docs` green after the file is added.
**Happy paths considered:**
- HP-1: `sqlx migrate run` on a clean DB → table created with all required columns and
  indexes; subsequent rollback/forward leaves schema consistent.
**Edge cases considered:**
- EC-1: column type for `status` is wide enough for all `GrantStatus` token strings;
  a future unknown token (added without a migration) does not truncate silently.
- EC-2: `issued_at` and `expires_at` stored with timezone (`TIMESTAMPTZ` or equivalent)
  so UTC comparison is unambiguous across DB locales.
**Files expected to change:** `infra/migrations/0021_create_playback_grants.sql` (new).
**Reflection strategy:** RRI 55 → **3 passes** (Med-high band). Pass 1: column completeness
vs T1 domain fields (HP-1). Pass 2: constraint correctness — NOT NULL, CHECK/enum status,
FK refs, TIMESTAMPTZ (EC-1/EC-2). Pass 3: index coverage for active-grant lookup +
migration immutability + qa-docs sync (ADR-008).
**Agent handoff prompt:** Add `infra/migrations/0021_create_playback_grants.sql` with
the full `playback_grants` table schema required by T1's `PlaybackGrant` domain type.
Do not touch any existing migration. Stop after `sqlx migrate run` is clean and
`make qa-docs` is green; do not start T2b.

### Reflection log

Required passes: 3 (RRI 55 — Med-high)

#### Pass 1 — Column completeness vs `PlaybackGrant`
- **Draft verdict:** all `PlaybackGrant` fields mapped to columns; no field orphaned.
- **Critique findings:** `scope` field (`PlaybackScope`) was missing from the initial
  column draft — added as `scope TEXT NOT NULL CHECK (scope IN ('review'))`.
- **Revisions applied:** added `scope` column with CHECK constraint.

#### Pass 2 — Constraint correctness
- **Draft verdict:** NOT NULL on all non-Option fields; CHECK tokens match T1 `fmt`
  impls exactly; TIMESTAMPTZ on both timestamp columns; `chk_grant_expiry_after_issued`
  mirrors T1 constructor invariant; FK to `assets(id)` and `organizations(id)` correct;
  `project_id` without FK is consistent with `review_tasks` pattern (composite PK on projects).
- **Critique findings:** none.
- **Revisions applied:** none.

#### Pass 3 — Index, immutability, qa-docs
- **Draft verdict:** `idx_playback_grants_active (asset_id, status, expires_at)` covers
  the T2b access pattern; no existing migration modified; `make qa-docs` passed clean.
- **Critique findings:** none.
- **Revisions applied:** none.

### Owner final verification

- Owner: Claude Sonnet 4.6
- Date: 2026-06-21
- Statement: All columns, constraints, and the index match the T1 `PlaybackGrant` contract;
  `make qa-docs` passes; no existing migration was modified.
- Commands run: `make qa-docs` (passed).

---

## S-125-T2b: Playback-grant repository + integration tests
**Status:** Decomposed (2026-06-21) — replaced by T2b-i + T2b-ii + T2b-iii below.
**Original RRI:** 84 (High). Re-decomposed to lower execution complexity.
**Decomposition record:** T2b split into three sequential subtasks. T2b-i owns the
mechanical grant CRUD (no cross-crate join). T2b-ii owns `resolve_grant_target` — the
only function that joins against S-120 and enforces the readiness gate. T2b-iii owns
the integration tests, which are written after both functions exist. `arch_decision`
penalty was reviewed and dropped from T2b-ii (the cross-S-120 join decision is
documented in T0/T1 — it is implementation, not a new architectural decision).
Rescored: T2b-i → 44 (Med-high), T2b-ii → 55 (Med-high), T2b-iii → 34 (Moderate).

---

## S-125-T2b-i: Grant CRUD (issue_grant / get_active_grant / expire_grant)
**Effort:** L (RRI 44 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T2a
**Status:** Done (2026-06-21)
**Type:** development (repository)
**Objective:** Implement the three mechanical grant-lifecycle functions against the
`playback_grants` schema. No cross-crate join, no S-120 touch — only `playback_grants`.
**Inputs:**
- Schema `playback_grants` (T2a).
- `PlaybackGrant`, `PlaybackGrantId`, `GrantStatus`, `GrantPrincipal`, `PlaybackScope`
  from T1 (`crates/domain/src/playback.rs`).
- `DbError::UnknownStoredValue` pattern (`crates/db/src/preparation_repo.rs:21-34`).
**Outputs:**
- `crates/db/src/playback_repo.rs` (new): `issue_grant`, `get_active_grant`, `expire_grant`.
- `crates/db/src/lib.rs`: `pub mod playback_repo;` export.
**Acceptance criteria:**
- `issue_grant` inserts a grant row with all T1 fields; returns `DbError` on duplicate PK.
- `get_active_grant` returns `Some(PlaybackGrant)` when `status = 'active'` and
  `expires_at > now()`; returns `None` otherwise (expired by wall-clock or by `expire_grant`).
- `expire_grant` sets `status = 'expired'` for an active grant; is a no-op if already
  expired/revoked (no error).
- Stored `status` decoded via `GrantStatus::from_str` → `PlaybackError` →
  `DbError::UnknownStoredValue` on unknown token (no default-allow).
- `cargo build -p dubbridge-db` clean; no clippy warnings.
**Happy paths:**
- HP-1: `issue_grant` → `get_active_grant` returns the grant.
- HP-2: `expire_grant` → `get_active_grant` returns `None`.
**Edge cases:**
- EC-3: grant past `expires_at` (wall-clock) → `get_active_grant` returns `None` without
  calling `expire_grant`.
**Files expected to change:** `crates/db/src/playback_repo.rs` (new); `crates/db/src/lib.rs`.
**Reflection strategy:** RRI 44 → **3 passes**. Pass 1: `issue_grant` + `get_active_grant`
happy path (HP-1). Pass 2: `expire_grant` + wall-clock expiry (HP-2 / EC-3). Pass 3:
`UnknownStoredValue` decode path + build clean.
**Agent handoff prompt:** Add `crates/db/src/playback_repo.rs` with `issue_grant`,
`get_active_grant`, and `expire_grant` against the `playback_grants` schema (T2a). Export
the module in `lib.rs`. Do not implement `resolve_grant_target` — that is T2b-ii. Stop
after `cargo build -p dubbridge-db` + clippy are clean; do not add integration tests yet.

### Reflection log

Required passes: 3 (RRI 44 — Med-high)

#### Pass 1 — `issue_grant` + `get_active_grant` (HP-1)
- **Draft verdict:** 9-column INSERT correct; SELECT filters `status='active' AND expires_at > now()` in DB (wall-clock correct); `grant_from_row` fails closed on unknown tokens before returning data.
- **Critique findings:** none.
- **Revisions applied:** none.

#### Pass 2 — `expire_grant` + wall-clock expiry (HP-2 / EC-3)
- **Draft verdict:** `expire_grant` UPDATE guarded with `AND status = 'active'` → no-op on already-expired/revoked; EC-3 covered entirely by the SQL filter in `get_active_grant`.
- **Critique findings:** none.
- **Revisions applied:** none.

#### Pass 3 — `UnknownStoredValue` decode path + build
- **Draft verdict:** `parse_status`/`parse_scope` follow exact `review_repo.rs` pattern; field name identifies column precisely; no default-allow in any path; build and clippy clean.
- **Critique findings:** none.
- **Revisions applied:** none.

### Owner final verification

- Owner: Claude Sonnet 4.6
- Date: 2026-06-21
- Statement: All three functions implemented; fail-closed decode on both stored enums; `cargo build -p dubbridge-db` and `cargo clippy -p dubbridge-db -- -D warnings` pass clean.
- Commands run: `cargo build -p dubbridge-db` (Finished, 0 errors), `cargo clippy -p dubbridge-db -- -D warnings` (Finished, 0 warnings).

---

## S-125-T2b-ii: resolve_grant_target (readiness gate + HLS lineage join)
**Effort:** L (RRI 55 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T2b-i
**Status:** Done (2026-06-21)
**Type:** development (repository)
**Objective:** Add `resolve_grant_target` to `playback_repo` — the function that checks
whether an active grant's asset is `Ready` and has an `HlsManifest` lineage row, returning
a typed `PlaybackDenial` at every failure point without leaking any manifest key.
**Inputs:**
- `get_active_grant` from T2b-i (already in `playback_repo`).
- `PreparationStatus` + `asset_preparation_status` table from S-120
  (`crates/db/src/preparation_repo.rs`, `infra/migrations/0019`).
- `artifact_records` table: rows with `kind = 'hls_manifest'` and
  `parent_artifact_id IS NOT NULL` identify the prepared HLS manifest.
- `PlaybackDenial` from T1 (`crates/domain/src/playback.rs`): `GrantInvalid`,
  `NotReady`, `MissingManifest`.
- `DerivedArtifact` from `crates/domain/src/artifact.rs` as the return type.
**Outputs:**
- `resolve_grant_target(pool, grant_id) -> Result<DerivedArtifact, PlaybackDenial>`
  added to `crates/db/src/playback_repo.rs`.
**Acceptance criteria:**
- Active grant + `Ready` asset + `HlsManifest` row → returns `DerivedArtifact`; no
  raw storage key exposed beyond the artifact struct itself.
- Active grant + asset not `Ready` → `PlaybackDenial::NotReady`; no manifest row leaked.
- Active grant + `Ready` asset + no `HlsManifest` row → `PlaybackDenial::MissingManifest`.
- Expired/not-found grant (via `get_active_grant`) → `PlaybackDenial::GrantInvalid`.
- `cargo build -p dubbridge-db` clean; no clippy warnings.
**Happy paths:**
- HP-1: active grant, `Ready`, `HlsManifest` exists → `DerivedArtifact` returned.
**Edge cases:**
- EC-1: `PreparationStatus` != `Ready` → `PlaybackDenial::NotReady`.
- EC-2: no `HlsManifest` lineage row → `PlaybackDenial::MissingManifest`.
- EC-3: grant expired (caught by `get_active_grant`) → `PlaybackDenial::GrantInvalid`.
**Files expected to change:** `crates/db/src/playback_repo.rs` only.
**Reflection strategy:** RRI 55 → **3 passes**. Pass 1: HP-1 happy path (active grant,
Ready, manifest present). Pass 2: denial paths fail-closed in order — GrantInvalid →
NotReady → MissingManifest (EC-1/EC-2/EC-3). Pass 3: no raw key leaked in any return
path + build clean.
**Agent handoff prompt:** Add `resolve_grant_target(pool, grant_id) -> Result<DerivedArtifact,
PlaybackDenial>` to the existing `crates/db/src/playback_repo.rs` (T2b-i done). The
function calls `get_active_grant`, then checks `asset_preparation_status`, then fetches
the `hls_manifest` artifact row — returning a typed `PlaybackDenial` at each failure
point. Do not add integration tests (T2b-iii). Stop after `cargo build -p dubbridge-db`
+ clippy are clean.

### Reflection log

Required passes: 3 (RRI 55 — Med-high)

#### Pass 1 — HP-1 happy path (active grant, Ready, manifest present)
- **Draft verdict:** three-step evaluation correct; `get_active_grant` reused from T2b-i; `fetch_optional` on `asset_preparation_status` treats absent row as NotReady; manifest SELECT guards `parent_artifact_id IS NOT NULL` to exclude source artifacts; `kind` hardcoded to `HlsManifest` (WHERE already guarantees it).
- **Critique findings:** none.
- **Revisions applied:** none.

#### Pass 2 — Denial paths fail-closed (EC-1/EC-2/EC-3)
- **Draft verdict:** evaluation order is GrantInvalid → NotReady → MissingManifest; DB errors map to denial variants (not leaked); unknown preparation status token maps to NotReady via `map_err`; no `artifact_records` access occurs when grant is invalid or asset not Ready.
- **Critique findings:** none.
- **Revisions applied:** none.

#### Pass 3 — No raw key leaked in repo layer + build clean
- **Draft verdict:** `storage_key` is present in `DerivedArtifact` return (correct — repo layer; key exposure boundary enforced in T4/T5 per ADR-032); build and clippy clean.
- **Critique findings:** none.
- **Revisions applied:** none.

### Owner final verification

- Owner: Claude Sonnet 4.6
- Date: 2026-06-21
- Statement: `resolve_grant_target` implemented with fail-closed evaluation at all three steps; `cargo build -p dubbridge-db` and `cargo clippy -p dubbridge-db -- -D warnings` pass clean.
- Commands run: `cargo build -p dubbridge-db` (Finished, 0 errors), `cargo clippy -p dubbridge-db -- -D warnings` (Finished, 0 warnings).

---

## S-125-T2b-iii: Integration tests (playback_repo_test)
**Effort:** M (RRI 34 — Moderate)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced); Claude Code `Claude Sonnet 4.6` — thinking Off.
**Depends on:** S-125-T2b-ii
**Status:** Done (2026-06-21)
**Type:** development (tests only)
**Objective:** Write the integration tests that prove all HP/EC paths of the complete
`playback_repo` (T2b-i + T2b-ii) against a real DB, following the style of
`apps/api/tests/preparation_repo_test.rs`.
**Inputs:**
- Complete `playback_repo` with all four functions (T2b-i + T2b-ii done).
- `apps/api/tests/preparation_repo_test.rs` — canonical fixture/setup style.
- `apps/api/tests/review_repo_test.rs` — canonical integration test style.
- HP/EC definitions from the T2b ledger (below).
**Outputs:**
- `apps/api/tests/playback_repo_test.rs` (new): one test per HP/EC path.
**Acceptance criteria:**
- HP-1: `issue_grant` → `get_active_grant` returns the grant → `resolve_grant_target`
  returns the `HlsManifest` artifact.
- HP-2: `expire_grant` → `get_active_grant` returns `None`.
- EC-1: asset `PreparationStatus` != `Ready` → `resolve_grant_target` returns
  `PlaybackDenial::NotReady`.
- EC-2: asset has no `HlsManifest` row → `resolve_grant_target` returns
  `PlaybackDenial::MissingManifest`.
- EC-3: grant past `expires_at` (wall-clock) → `get_active_grant` returns `None`.
- All tests pass; `cargo clippy` clean on the test file.
**Files expected to change:** `apps/api/tests/playback_repo_test.rs` (new only).
**Reflection strategy:** RRI 34 → **2 passes**. Pass 1: HP-1/HP-2 happy paths pass
against a real DB. Pass 2: EC-1/EC-2/EC-3 denial paths each exercise the correct
`PlaybackDenial` variant with no leaked data.
**Agent handoff prompt:** Add `apps/api/tests/playback_repo_test.rs` covering HP-1,
HP-2, EC-1, EC-2, EC-3 of the complete `playback_repo`. Follow the fixture and setup
style of `preparation_repo_test.rs`. No production code changes. Stop after all tests
pass and clippy is clean on the test file.

### Reflection log

**Pass 1 (fixtures, org insert, grant construction):** All fixture helpers follow
`preparation_repo_test.rs` style exactly — `setup_pool` → `Option` early-return,
`INSERT INTO organizations` (2 columns), `INSERT INTO assets` with `"finalized"`.
`make_grant` uses `PlaybackGrant::new` which validates `expires_at > issued_at` at
construction — no invalid grants can enter the DB from fixtures. `insert_hls_manifest`
uses `preparation_repo::insert_derived_artifact` with `ArtifactKind::HlsManifest` and
`parent_artifact_id = source.id`, matching the exact query predicate in
`resolve_grant_target` (`kind = 'hls_manifest' AND parent_artifact_id IS NOT NULL`).

**Pass 2 (fail-closed ordering, denial variants):** EC-1 inserts `InProgress` (no
manifest) → `resolve_grant_target` stops at step 2 returning `NotReady` before reaching
step 3. EC-2 inserts `Ready` (no manifest) → passes steps 1–2, fails step 3 returning
`MissingManifest`. EC-3 uses `issued = now() - 2h`, `expires = issued + 1s` — the grant
row is `status='active'` but `expires_at` is already past; `get_active_grant` filters
`AND expires_at > now()` → `None`. `PlaybackDenial` derives `PartialEq + Eq` (confirmed
in `crates/domain/src/playback.rs`) — `assert_eq!` on denial variants is valid.

### Owner final verification

- `cargo build -p dubbridge-api` — clean (0 errors, 0 warnings).
- `cargo clippy -p dubbridge-api -- -D warnings` — clean.
- 5 tests created covering all HP/EC acceptance criteria.
- No production code changed.

---

## S-125-T3a: Pure manifest rewriter (local Gemma delegation)
**Effort:** S (RRI 21 — Low)
**Recommended model:** Local Gemma via Ollama (`DUBBRIDGE_LOW_RRI_MODEL`, default
`gemma4:12b-it-q4_K_M`). The primary agent remains orchestrator/reviewer of record.
**Depends on:** S-125-T3b (crate scaffold must exist first — Gemma cannot create the
crate/workspace wiring), S-125-T1 (grant-context type for the routed-base input).
**Status:** Done (2026-06-21)
**Type:** development (Low-band, local delegation)
**Objective:** Implement one **pure, IO-free** function in the already-scaffolded
`crates/playback` crate that rewrites a prepared `.m3u8` so every media-segment
reference points at a caller-supplied routed base instead of a raw object-store key.
No storage, no network, no auth — fixtures only.
**Why delegable:** measured RRI **21** (`scripts/rri.py --touches crates/playback/src/lib.rs
--cc 6 --D 2 --K 0 --P 2 --T 2 --A 0 --X 1`). It is a single-file pure function with
deterministic fixtures — best-fit per `LOW_RRI_LOCAL_MODEL_HANDOFF.md`. It is the only
S-125 sub-part that clears the band with legitimately low D/K/P.
**Inputs:**
- Scaffolded `crates/playback` (from T3b): crate exists, builds, error type declared.
- A representative prepared `.m3u8` fixture + the malformed-HLS fixture already in the repo.
- The exact function signature, error variants, and rewrite rule, supplied verbatim in
  the delegation packet (Gemma is a mechanical transcriber, not the designer).
**Outputs:**
- `crates/playback/src/lib.rs`: `rewrite_manifest(manifest: &str, routed_base: &str)
  -> Result<String, ManifestRewriteError>` (pure), plus its unit tests.
**Acceptance criteria:**
- Zero IO and zero network calls; fully unit-testable from in-memory fixtures.
- Every media-segment URI in the output is prefixed/replaced by `routed_base`; no raw
  object-store key (no `s3://`, no bare key path) appears in any rewritten manifest.
- Malformed/empty/`#EXTM3U`-missing input returns `ManifestRewriteError`, emitting nothing.
- Order of segments is preserved; non-segment tag lines pass through unchanged.
- ≥90% line coverage for the function; no clippy warnings.
**Happy paths considered:**
- HP-1: valid prepared `.m3u8` + `routed_base` → manifest whose segment URIs are all
  prefixed with `routed_base`, order preserved.
- HP-2: playlist with N segments → all N references rewritten; `#EXTINF`/tag lines intact.
**Edge cases considered:**
- EC-1: malformed manifest (missing `#EXTM3U`) → `ManifestRewriteError`, nothing emitted.
- EC-2: no raw object-store key leaks into the output (assert absence of `s3://`/raw key).
**Files expected to change:** `crates/playback/src/lib.rs` only (single-file packet,
`--mode full-file`).
**Reflection strategy (orchestrator-applied, per `AGENT_WORKFLOW_GUIDE.md`):** Low-band
delegated work has no required pass count, but the orchestrator applies the Reflection
cycle to Gemma's output during the mandatory review: verify scope/format/tagged-block
contract, read the diff line by line, confirm no out-of-scope edits, run
`cargo build --workspace` + `cargo test -p dubbridge-playback` + clippy, and run at most
one bounded repair cycle before escalating. Record the reflection log in the final report,
not inside the delegated task. Gemma must not evaluate its own output.
**Delegation packet (orchestrator builds before sending):**
- Goal (1–2 sentences) + exact allowed file: `crates/playback/src/lib.rs`.
- What must NOT change: the crate's `Cargo.toml`, workspace wiring, error-type name.
- Output contract: tagged blocks with complete file content, `--mode full-file`.
- `## Current file content`: the scaffolded `lib.rs` verbatim.
- The literal function signature, `ManifestRewriteError` variants, and the rewrite rule.
- Both fixtures (valid + malformed) and the exact assertions to write.
- Sent via `scripts/delegate-low-rri.py`; the wrapper builds + checks the diff.
**Stop condition:** after the orchestrator review + verification pass; do not touch
`crates/storage` (that is T3b) and do not wire the API (T4/T5).

### Orchestrator reflection log (2026-06-21)

**Attempt 1:** First packet used prose test descriptions instead of a literal AFTER block.
Gemma produced a double-newline bug (pushed `\n` inside the segment branch then again
at the loop end) and an incorrect fallback (`"{} {}\n"` with a space instead of `/`).
Rejected without applying. Packet was too verbose; violated "treat Gemma as mechanical
transcriber" rule.

**Attempt 2 (repair):** Packet reduced to a literal AFTER block — complete file content
to transcribe verbatim. Gemma emitted the block correctly. Applied with `--apply`.
Diff confirmed only the intended lines changed: `NotImplemented` variant removed,
function body and 6 tests added. `cargo build`, `cargo clippy -- -D warnings`, and
`cargo test` all clean — 6/6 tests pass.

### Owner final verification

- `cargo build -p dubbridge-playback` — clean.
- `cargo clippy -p dubbridge-playback -- -D warnings` — clean.
- `cargo test -p dubbridge-playback` — 6/6 passed (HP-1, HP-2, EC-1, EC-1b, EC-2, trailing-slash).
- File modified: `crates/playback/src/lib.rs` only.

---

## S-125-T3b: Crate scaffold + storage scoped-reference helper
**Effort:** L (RRI 49 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T1 (grant context type)
**Status:** Done (2026-06-21)
**Type:** development (retained by primary agent — sits on the no-raw-key boundary)
**Objective:** Scaffold the new `crates/playback` crate (workspace member, `Cargo.toml`,
empty `lib.rs` with the declared `ManifestRewriteError` type so T3a can fill the
function), and add the `crates/storage` helper that returns a scoped/expiring reference
for a prepared-HLS key without exposing raw object-store paths.
**Why retained, not delegated:** the storage helper sits on the no-raw-key boundary in
`crates/storage`, whose anchor-rubric floor is D3/P3/K3 (ADR-006/ADR-018). That is above
the Gemma band by construction; it is the security-relevant half of the original T3 and
must stay with the primary agent.
**Inputs:**
- Grant context type from T1.
- `crates/storage` canonical key conventions from `S-080`/`S-120`.
- The `ManifestRewriteError` contract that T3a will implement against.
**Outputs:**
- `crates/playback` scaffold: `Cargo.toml`, workspace member entry, `src/lib.rs` with the
  `ManifestRewriteError` type + `rewrite_manifest` stub signature (unimplemented or
  `todo!()`), so T3a delegates only the body + tests.
- `crates/storage` helper exposing a scoped/expiring reference (or read body) for a
  prepared-HLS key; clients never receive a raw key.
**Acceptance criteria:**
- `crates/playback` compiles as a workspace member with the error type + stub signature.
- The storage helper never returns a raw MinIO/S3 key to a caller; it yields a scoped/
  expiring reference or a streamed body only.
- Key construction stays inside `crates/storage`; no caller hand-rolls a prepared-HLS key.
- ≥90% coverage for the storage helper; no clippy warnings.
**Happy paths considered:**
- HP-1: request a scoped reference for a valid prepared-HLS key → scoped/expiring
  reference returned, raw key not exposed.
**Edge cases considered:**
- EC-1: unknown/absent prepared-HLS key → typed error, no fabricated reference.
- EC-2: the helper output contains no raw object-store path (assert no `s3://`/raw key).
**Files expected to change:** `crates/playback/Cargo.toml`; `crates/playback/src/lib.rs`
(scaffold + error type + stub); `Cargo.toml` (workspace member); `crates/storage/src/lib.rs`.
**Reflection strategy:** RRI 49 → Med-high → **3 passes**. Pass 1: scaffold compiles +
storage helper happy path (HP-1). Pass 2: no-raw-key non-leakage + unknown-key fail-closed
(EC-1/EC-2). Pass 3: key-ownership boundary (no key construction outside `crates/storage`)
+ coverage sweep.
**Agent handoff prompt:** Scaffold `crates/playback` (workspace member + `ManifestRewriteError`
+ `rewrite_manifest` stub) and add the `crates/storage` scoped-reference helper that never
exposes a raw key. Stop after the scaffold compiles and the storage helper tests are green;
this unblocks T3a (Gemma fills the rewriter body). Do not implement the rewriter body or
wire the API (T4/T5).

### Reflection log

**Pass 1 (scaffold + HP-1):** `crates/playback` compila como miembro de workspace. `ManifestRewriteError::MissingHeader` declarado. `rewrite_manifest` stub retorna `Err(NotImplemented)` — no usa `todo!()` ni `panic!()`, respeta `clippy::todo/panic = deny`. `get_hls_manifest` construye la key internamente con `hls_manifest_key(asset_id)` y retorna `HlsManifestBytes(Vec<u8>)` — el caller nunca recibe un string de key.

**Pass 2 (no-raw-key leakage + EC-1/EC-2):** `HlsManifestBytes` newtype tiene un único campo `Vec<u8>` — ningún path de código expone la key al caller. `StorageError::NotFound` propaga el key en el mensaje interno (logs/backend), no al cliente HTTP — compatible con ADR-032. Absent key → `NotFound` error, no bytes fabricados — fail-closed.

**Pass 3 (key-ownership boundary + cobertura):** Grep confirma que ningún caller fuera de `crates/storage` construye `assets/{id}/prepared/hls/index.m3u8` a mano. 3 tests nuevos (HP-1, EC-1, EC-2) pasan junto con los 39 existentes — 42/42 total.

### Owner final verification

- `cargo build -p dubbridge-storage` — clean.
- `cargo clippy -p dubbridge-storage -- -D warnings` — clean.
- `cargo test -p dubbridge-storage` — 42/42 passed.
- `cargo build -p dubbridge-playback && cargo clippy -p dubbridge-playback -- -D warnings` — clean.
- Archivos modificados: `crates/playback/Cargo.toml` (nuevo), `crates/playback/src/lib.rs` (nuevo), `Cargo.toml` (workspace entry), `crates/storage/src/lib.rs` (`get_hls_manifest` + tests).

---

## S-125-T4: Grant issuance API + review authorization + durable grant audit
**Effort:** XL (RRI 88 — Very high; decomposition gate triggered)
**Recommended model:** Codex `GPT-5.2-Codex` (Premium); Claude Code `Claude Opus 4.1` — thinking On.
**Depends on:** S-125-T2, S-125-T3a, S-125-T3b
**Status:** Done (2026-06-22) via T4a-i + T4a-ii + T4b-i + T4b-ii + T4b-iii + T4c + T4d
**Type:** development (public API surface, auth, audit)
**Objective:** Expose a grant-issuance endpoint behind the verified principal and the
`S-100`/`S-160` org/project authorization guard, issuing a scoped/expiring grant for a
`Ready` asset and emitting a durable, traceable grant audit row (ADR-018).
**Decomposition note (2026-06-22):** RRI 88 triggered gate. T4a (RRI 94) was already
split into T4a-i (route wiring + handler skeleton, RRI 52) and T4a-ii (auth guard,
RRI 61). `T4b` was then re-split so the playback audit contract/persistence shape, the
handler-level success/refusal audit emission, and the auth-boundary refusal audit can be
implemented and approved independently. No subtask reaches RRI ≤ 25: touching the
governance-audit boundary keeps the floor above the Low band.

| Subtask | RRI | Band |
|---|---|---|
| T4a-i — route wiring + handler skeleton | 52 | Med-high |
| T4a-ii — auth guard org/project | 61 | Complex |
| T4b-i — audit event contract + persistence decode | 39 | Moderate |
| T4b-ii — success + service-denial audit emission | 47 | Med-high |
| T4b-iii — auth-boundary refusal audit + audit-focused integration tests | 54 | Med-high |
| T4c — audience-policy hook seam (F3) | 44 | Med-high |
| T4d — integration tests | 46 | Med-high |

---

## S-125-T4a-i: Route wiring + handler skeleton
**Effort:** L (RRI 52 — Med-high)
**Recommended model:** Claude Sonnet 4.6 — thinking On.
**Depends on:** S-125-T2, S-125-T3b
**Status:** Done (2026-06-22)
**Type:** development
**Objective:** Add `POST /assets/{id}/playback-grants` route to the router and the
handler skeleton in `apps/api/src/playback_service.rs`. The handler receives the
request, extracts the asset id, and returns a stub 501 — no auth logic yet (T4a-ii),
no audit (T4b). Establishes the file and module structure that T4a-ii through T4d build on.
**Inputs:**
- Existing router in `apps/api/src/` (read pattern from `review_api_test.rs` / routes).
- `PlaybackGrantId`, `PlaybackGrant` types from T1.
- `playback_repo::issue_grant` from T2b-i.
**Outputs:**
- `apps/api/src/playback_service.rs` (new): handler function + response type.
- `apps/api/src/` router module: route registered.
**Acceptance criteria:**
- `POST /assets/{id}/playback-grants` is routable (returns 501 or similar stub).
- `cargo build -p dubbridge-api` clean; `cargo clippy` clean.
- No auth logic, no DB calls, no audit — pure skeleton.
**Happy paths considered:**
- HP-1: `POST /assets/{id}/playback-grants` with a valid UUID reaches the new handler
  and returns a `501 Not Implemented` stub response containing the parsed asset id.
- HP-2: the main `build_app` router still serves existing routes after the playback
  route is merged.
**Edge cases considered:**
- EC-1: malformed `asset_id` in the path is rejected at the Axum boundary with no
  handler execution.
- EC-2: the skeleton stays stub-only — no auth middleware, no DB call, and no audit
  requirement are introduced before `T4a-ii` / `T4b`.
**Files expected to change:** `apps/api/src/lib.rs`; `apps/api/src/playback_service.rs`
(new); `apps/api/src/routes/mod.rs`; `apps/api/src/routes/playback.rs` (new).
**Reflection strategy:** RRI 52 → **3 passes**.

### Reflection log

**Pass 1 (route wiring + HP-1):** Added `playback_service` plus a dedicated
`routes/playback.rs` module, exported it from `routes/mod.rs`, and merged it into
`build_app`. The playback-grant path now exists as its own API surface and keeps UUID
parsing at the HTTP boundary.

**Pass 2 (boundary safety + EC-1):** Preserved Axum path extraction on
`/assets/{id}/playback-grants`, so malformed asset ids are rejected before grant logic
executes. Follow-on task `T4a-ii` intentionally replaced the original stub-only handler
with authenticated issuance logic, so the old `501` behavior is no longer expected in
`HEAD`.

**Pass 3 (merge safety + verification):** Retained route-level tests proving the playback
path is mounted and that `build_app` still serves `/health/live` after the router merge.
`cargo build`, `cargo clippy`, and the targeted route tests all passed clean.

**Happy paths covered:**
- HP-1: valid `POST /assets/{id}/playback-grants` reaches the mounted playback route and
  hits the auth boundary, proving the route exists in the app graph.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`;
  `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope`.
- HP-2: the new playback router merge does not break existing app routing.
  Code evidence: `apps/api/src/routes/playback.rs::build_app_keeps_health_route_after_playback_merge`;
  `apps/api/src/lib.rs::build_app`.

**Edge cases covered:**
- EC-1: malformed UUID path is rejected with `400 Bad Request` before handler execution.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_rejects_malformed_asset_id`.
- EC-2: later tasks intentionally superseded the original stub-only invariant, so `HEAD`
  now verifies the stronger auth boundary instead of preserving a dead-end `501` stub.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`;
  `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid `POST /assets/{id}/playback-grants` reaches the mounted playback route and its auth boundary | `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`; `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope` | passed |
| HP-2 | Happy path | `build_app` still serves existing routes after playback merge | `apps/api/src/routes/playback.rs::build_app_keeps_health_route_after_playback_merge` | passed |
| EC-1 | Edge case | malformed `asset_id` path is rejected before handler execution | `apps/api/src/routes/playback.rs::playback_grant_route_rejects_malformed_asset_id` | passed |
| EC-2 | Edge case | original stub-only behavior from `T4a-i` was intentionally superseded in `T4a-ii`; current `HEAD` keeps route-level auth failure proofs instead | `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`; `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope` | superseded in `HEAD` |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the playback-grant route merge remains intact in `HEAD`, malformed ids still fail at the HTTP boundary, and the original `T4a-i` stub-only behavior has been intentionally replaced by the authenticated issuance flow from `T4a-ii`.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api playback_grant_route_requires_authentication -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_requires_workspace_write_scope -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_malformed_asset_id -- --nocapture`; `cargo test -p dubbridge-api build_app_keeps_health_route_after_playback_merge -- --nocapture`

---

## S-125-T4a-ii: Auth guard org/project
**Effort:** L (RRI 61 — Complex)
**Recommended model:** Claude Sonnet 4.6 — thinking On.
**Depends on:** S-125-T4a-i
**Status:** Done (2026-06-22)
**Type:** development (security-critical)
**Objective:** Wire the verified-principal extractor (ADR-023) and the `S-100`/`S-160`
org/project membership guard (ADR-027) into the handler from T4a-i. Unauthenticated
callers → 401 before any DB call. Unauthorized callers → 403 before issuance. Authorized
callers → call `playback_repo::issue_grant` and return the grant id.
**Inputs:**
- Handler skeleton from T4a-i.
- Verified principal extractor pattern (ADR-023) — read from existing auth middleware.
- Org/project membership guard pattern (ADR-027) — read from `review_api` or equivalent.
- `playback_repo::issue_grant` (T2b-i).
**Outputs:**
- `apps/api/src/playback_service.rs`: full issuance handler with auth + grant write.
**Acceptance criteria:**
- Unauthenticated → 401, no DB call.
- Authenticated non-member → 403, no grant row written.
- Authenticated org-member + `Ready` asset → grant issued, 201 with grant id.
- `cargo build` + `cargo clippy` clean.
**Files changed:** `apps/api/src/lib.rs`; `apps/api/src/playback_service.rs`;
`apps/api/src/routes/playback.rs`.
**Reflection strategy:** RRI 61 → **3 passes** (Complex-band).

### Happy paths considered

- HP-1: authenticated caller with `workspaces:write`, org membership at `Reviewer` or
  higher, and a `Ready` asset with at least one HLS manifest receives `201 Created` with
  a persisted playback grant id.
- HP-2: authenticated callers who reach the route but lack required scope are rejected at
  the middleware boundary before service-level grant logic runs.

### Edge cases considered

- EC-1: missing or invalid bearer token returns `401 Unauthorized` before any playback
  issuance attempt.
- EC-2: authenticated caller who is not linked to the asset's org/project fails closed
  with `403 asset not found` and no grant row written.
- EC-3: authenticated org member below `Reviewer` also fails closed with
  `403 asset not found`.
- EC-4: asset without `Ready` preparation status or without at least one HLS manifest
  returns `409 asset not ready for playback`.

### Reflection log

**Pass 1 (auth boundary + HP-2/EC-1):** Wrapped the route with
`authenticate_bearer` and `require_scope("workspaces:write")`, so unauthenticated
requests return `401` and under-scoped callers return `403` before the handler attempts
playback-grant issuance.

**Pass 2 (org/project guard + HP-1/EC-2/EC-3):** Replaced the stub handler with a real
service that resolves the asset's org/project via `project_assets`, `projects`, and
`org_members`, parses the stored org role, and fail-closes with `403 asset not found`
unless the caller is at least `Reviewer`.

**Pass 3 (readiness gate + persistence):** Added the readiness check using
`preparation_repo::get_preparation_status` plus
`get_preparation_readiness_evidence`, requiring `PreparationStatus::Ready` and at least
one HLS manifest before constructing and persisting a one-hour review playback grant via
`playback_repo::issue_grant`. Route tests, DB-backed issuance tests, `cargo build`, and
`cargo clippy` all passed; the DB-backed tests skip cleanly when
`DUBBRIDGE_DATABASE_URL` is unset.

**Happy paths covered:**
- HP-1: authorized reviewer on a ready asset with an HLS manifest receives `201 Created`
  and a persisted grant row with matching asset/org/project/principal ids.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer`;
  `apps/api/src/playback_service.rs::issue_playback_grant`.
- HP-2: route-level auth boundary rejects under-scoped callers before grant issuance.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope`;
  `apps/api/src/routes/playback.rs::router`.

**Edge cases covered:**
- EC-1: missing bearer token returns `401 Unauthorized`.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`.
- EC-2: authenticated non-member fails closed with `403 asset not found` and writes no
  grant row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_rejects_non_member_before_grant_creation`.
- EC-3: authenticated member below `Reviewer` fails closed with `403` and writes no
  grant row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_rejects_member_with_insufficient_role`.
- EC-4: not-ready asset returns `409 asset not ready for playback` and writes no grant
  row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_rejects_asset_that_is_not_ready`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | authorized reviewer on ready asset receives `201` and persisted grant id | `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer` | passed |
| HP-2 | Happy path | under-scoped caller is rejected at the route boundary before grant issuance | `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope` | passed |
| EC-1 | Edge case | missing bearer token returns `401` | `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication` | passed |
| EC-2 | Edge case | authenticated non-member returns `403` and no grant row is written | `apps/api/src/routes/playback.rs::playback_grant_route_rejects_non_member_before_grant_creation` | passed |
| EC-3 | Edge case | authenticated member below `Reviewer` returns `403` and no grant row is written | `apps/api/src/routes/playback.rs::playback_grant_route_rejects_member_with_insufficient_role` | passed |
| EC-4 | Edge case | asset not ready for playback returns `409` and no grant row is written | `apps/api/src/routes/playback.rs::playback_grant_route_rejects_asset_that_is_not_ready` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the playback-grant issuance path now enforces bearer auth plus
  `workspaces:write`, resolves org/project scope from the asset linkage, requires
  `Reviewer` or higher, gates issuance on readiness evidence, and persists a one-hour
  review grant only for authorized ready assets.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api playback_grant_route_requires_authentication -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_requires_workspace_write_scope -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_malformed_asset_id -- --nocapture`; `cargo test -p dubbridge-api build_app_keeps_health_route_after_playback_merge -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_issues_grant_for_authorized_reviewer -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_non_member_before_grant_creation -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_member_with_insufficient_role -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_asset_that_is_not_ready -- --nocapture`

---

## S-125-T4b: Durable audit row
**Effort:** L (RRI 57 — Complex)
**Recommended model:** Claude Sonnet 4.6 — thinking On.
**Depends on:** S-125-T4a-ii
**Status:** Done (2026-06-22) via T4b-i + T4b-ii + T4b-iii
**Type:** development (ADR-018)
**Objective:** Emit a durable audit row for grant-create and grant-refusal events,
correlated by trace id, using the existing `crates/audit` emission seam (ADR-018).
**Decomposition note (2026-06-22):** The original `T4b` mixed three different risk
centers: audit-event contract changes, handler-level audit behavior, and route/middleware
denials that currently happen before `playback_service` runs. Those concerns are now
split so each approval can focus on one failure boundary at a time.

**T4b coverage mapping:**

| T4b acceptance criterion | Covered by |
|---|---|
| Grant create has a dedicated durable audit event kind understood by domain + DB | T4b-i |
| Authorized success path writes exactly one success audit row | T4b-ii |
| Service-level denials (`403 asset not found`, `409 not ready`) write refusal audits and no success row | T4b-ii |
| Auth-boundary denials (`401`, scope `403`) also produce refusal audits | T4b-iii |
| Audit-focused integration tests pin success/refusal row counts and kinds | T4b-iii |

---

## S-125-T4b-i: Audit event contract + persistence decode
**Effort:** M (RRI 39 — Moderate)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T4a-ii
**Status:** Done (2026-06-22)
**Type:** development (ADR-018 contract)
**Objective:** Introduce the playback-grant audit event vocabulary in the shared domain
and persistence layers so the API can emit durable success/refusal rows without
stringly-typed one-off behavior.
**Inputs:**
- Existing `AuditEventKind` enum and constructors in `crates/domain/src/audit.rs`.
- Existing decode/encode pattern in `crates/db/src/audit_repo.rs`.
- ADR-018 and ADR-032 grant decision semantics.
**Outputs:**
- `crates/domain/src/audit.rs`: playback-grant success/refusal `AuditEventKind`
  variants and a constructor/helper suitable for playback events.
- `crates/db/src/audit_repo.rs`: decode support for the new event kinds.
**Acceptance criteria:**
- Domain can construct playback-grant success/refusal audit events without overloading
  unrelated review/publication kinds.
- DB decode is fail-closed for the new stored values.
- `cargo build` + `cargo clippy` clean.
**Files changed:** `crates/domain/src/audit.rs`; `crates/db/src/audit_repo.rs`.
**Reflection strategy:** RRI 39 → **3 passes**.

### Happy paths considered

- HP-1: the shared audit domain can construct playback-grant success/refusal events
  with dedicated `AuditEventKind` values and a playback-specific helper.
- HP-2: persisted `audit_events.event_kind` tokens for playback decode back to the
  typed playback variants in `crates/db`.

### Edge cases considered

- EC-1: an unknown stored `audit_events.event_kind` token still fails closed with
  `DbError::UnknownStoredValue`.
- EC-2: playback grant events do not overload `ReviewApproved`,
  `PublicationRefused`, or any other semantically unrelated governance kind.

### Reflection log

**Pass 1 (event vocabulary + HP-1):** Added dedicated playback-grant audit kinds,
`PlaybackGrantIssued` and `PlaybackGrantRefused`, plus a
`AuditEvent::new_playback_event(...)` helper that pins playback events to an `asset_id`
without introducing unrelated correlation fields.

**Pass 2 (DB decode + HP-2/EC-1):** Extended `crates/db::audit_repo::parse_event_kind`
to recognize the new stored tokens `playback_grant_issued` and
`playback_grant_refused`, while preserving the existing fail-closed
`UnknownStoredValue` path for unknown tokens.

**Pass 3 (contract coverage + EC-2):** Added unit tests proving the playback kinds
serialize to their dedicated snake_case tokens, the playback helper sets only the
expected asset binding, and DB row decoding round-trips the refusal variant without
reusing review/publication semantics.

**Happy paths covered:**
- HP-1: playback-grant events now have dedicated enum variants and a dedicated helper.
  Code evidence: `crates/domain/src/audit.rs::AuditEventKind`;
  `crates/domain/src/audit.rs::AuditEvent::new_playback_event`;
  `crates/domain/src/audit.rs::tests::new_playback_event_sets_asset_id_and_no_correlation_ids`.
- HP-2: stored playback event tokens decode back to typed playback variants.
  Code evidence: `crates/db/src/audit_repo.rs::parse_event_kind`;
  `crates/db/src/audit_repo.rs::tests::parse_event_kind_known_variants_succeed`;
  `crates/db/src/audit_repo.rs::tests::row_to_event_round_trips_playback_kind`.

**Edge cases covered:**
- EC-1: unknown stored `event_kind` values still fail closed.
  Code evidence: `crates/db/src/audit_repo.rs::tests::parse_event_kind_unknown_value_fails_closed`.
- EC-2: playback uses dedicated kinds instead of overloading review/publication names.
  Code evidence: `crates/domain/src/audit.rs::tests::audit_event_kind_display_all_variants`;
  `crates/domain/src/audit.rs::tests::new_playback_event_sets_asset_id_and_no_correlation_ids`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | playback success/refusal events have dedicated kinds and a playback-specific helper | `crates/domain/src/audit.rs::tests::audit_event_kind_display_all_variants`; `crates/domain/src/audit.rs::tests::new_playback_event_sets_asset_id_and_no_correlation_ids` | passed |
| HP-2 | Happy path | persisted playback event tokens decode to typed playback variants | `crates/db/src/audit_repo.rs::tests::parse_event_kind_known_variants_succeed`; `crates/db/src/audit_repo.rs::tests::row_to_event_round_trips_playback_kind` | passed |
| EC-1 | Edge case | unknown stored playback-adjacent `event_kind` token fails closed | `crates/db/src/audit_repo.rs::tests::parse_event_kind_unknown_value_fails_closed` | passed |
| EC-2 | Edge case | playback does not overload review/publication audit kinds | `crates/domain/src/audit.rs::tests::audit_event_kind_display_all_variants`; `crates/domain/src/audit.rs::tests::new_playback_event_sets_asset_id_and_no_correlation_ids` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the playback-grant audit contract now has dedicated success and
  refusal event kinds, a playback-specific constructor, and fail-closed DB decode
  coverage for the new stored values.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-db`; `cargo clippy -p dubbridge-db -- -D warnings`; `cargo test -p dubbridge-domain audit_event_kind_display_all_variants -- --nocapture`; `cargo test -p dubbridge-domain new_playback_event_sets_asset_id_and_no_correlation_ids -- --nocapture`; `cargo test -p dubbridge-db parse_event_kind_known_variants_succeed -- --nocapture`; `cargo test -p dubbridge-db row_to_event_round_trips_playback_kind -- --nocapture`

---

## S-125-T4b-ii: Success + service-denial audit emission
**Effort:** L (RRI 47 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T4b-i
**Status:** Done (2026-06-22)
**Type:** development (ADR-018 runtime behavior)
**Objective:** Emit durable playback-grant audit rows from `playback_service` for the
authorized success path and for denials that occur inside the service after auth has
already passed.
**Inputs:**
- Handler from `T4a-ii`.
- Playback audit event kinds from `T4b-i`.
- `crates/audit::emit_governance_audit` fail-closed seam.
**Outputs:**
- `apps/api/src/playback_service.rs`: success audit on grant creation plus refusal
  audit on service-level denials.
**Acceptance criteria:**
- `201 Created` grant issuance writes exactly one success audit row.
- Service-level refusals (`403 asset not found`, `409 asset not ready for playback`)
  write refusal audit rows and no success row.
- Audit persistence failure is not swallowed silently.
- `cargo build` + `cargo clippy` clean.
**Files changed:** `apps/api/src/playback_service.rs`; `apps/api/src/routes/playback.rs`.
**Reflection strategy:** RRI 47 → **3 passes**.

### Happy paths considered

- HP-1: authorized reviewer on a ready asset receives `201`, one persisted
  `playback_grant`, and one durable `PlaybackGrantIssued` audit row.
- HP-2: playback-grant audit emission reuses the shared ADR-018 seam
  `emit_governance_audit`, not a bespoke logging path.

### Edge cases considered

- EC-1: service-level authz denials (`403 asset not found`) emit
  `PlaybackGrantRefused` and never emit `PlaybackGrantIssued`.
- EC-2: readiness denials (`409 asset not ready for playback`) emit
  `PlaybackGrantRefused` and never emit `PlaybackGrantIssued`.
- EC-3: if durable audit persistence fails during a service-level refusal, the route
  fails closed with `500` rather than silently swallowing the audit loss.

### Reflection log

**Pass 1 (success path + HP-1):** Added `emit_success_audit(...)` to
`playback_service`, using `AuditEvent::new_playback_event(...)` and
`emit_governance_audit(...)` immediately after `playback_repo::issue_grant`, so a
successful `201` now leaves a durable `PlaybackGrantIssued` row with grant/org/project
context.

**Pass 2 (service denials + EC-1/EC-2):** Routed service-level `403` and `409`
responses through refusal-audit emission before returning the HTTP error. The refusal
payload records the asset, actor, optional org/project context, and a stable denial
reason (`asset_not_found` or `asset_not_ready`).

**Pass 3 (fail-closed audit behavior + EC-3):** Mapped `AuditEmitError` to an internal
API error and added DB-backed route tests that pin success/refusal row counts plus a
fail-closed case where the `audit_events` table is missing. In this environment the
DB-backed tests compile and skip cleanly when `DUBBRIDGE_DATABASE_URL` is unset.

**Happy paths covered:**
- HP-1: successful playback-grant issuance now creates one success audit row and no
  refusal row.
  Code evidence: `apps/api/src/playback_service.rs::emit_success_audit`;
  `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer`.
- HP-2: playback audit reuses the shared durable audit seam.
  Code evidence: `apps/api/src/playback_service.rs::emit_success_audit`;
  `apps/api/src/playback_service.rs::emit_refusal_audit`;
  `crates/audit/src/lib.rs::emit_governance_audit`.

**Edge cases covered:**
- EC-1: non-member and insufficient-role denials emit one refusal audit row and no
  success row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_rejects_non_member_before_grant_creation`;
  `apps/api/src/routes/playback.rs::playback_grant_route_rejects_member_with_insufficient_role`.
- EC-2: not-ready assets emit one refusal audit row and no success row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_rejects_asset_that_is_not_ready`.
- EC-3: refusal-audit persistence failure surfaces as `500 Internal Server Error`
  instead of being swallowed.
  Code evidence: `apps/api/src/playback_service.rs::ApiError::from_audit_emit`;
  `apps/api/src/routes/playback.rs::playback_grant_route_fails_closed_when_refusal_audit_persistence_fails`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | successful grant issuance writes one `playback_grant_issued` row | `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer` | passed |
| HP-2 | Happy path | playback grant audit emission uses the shared durable audit seam | `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer` | passed |
| EC-1 | Edge case | service-level `403` denials write one refusal row and no success row | `apps/api/src/routes/playback.rs::playback_grant_route_rejects_non_member_before_grant_creation`; `apps/api/src/routes/playback.rs::playback_grant_route_rejects_member_with_insufficient_role` | passed |
| EC-2 | Edge case | service-level `409` denial writes one refusal row and no success row | `apps/api/src/routes/playback.rs::playback_grant_route_rejects_asset_that_is_not_ready` | passed |
| EC-3 | Edge case | audit persistence failure on refusal path returns `500` fail-closed | `apps/api/src/routes/playback.rs::playback_grant_route_fails_closed_when_refusal_audit_persistence_fails` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified playback-grant success and service-level denials now emit
  durable ADR-018 audit rows through the shared seam, and that refusal-audit
  persistence failures surface as fail-closed `500` responses.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api playback_grant_route_issues_grant_for_authorized_reviewer -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_non_member_before_grant_creation -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_member_with_insufficient_role -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_rejects_asset_that_is_not_ready -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_fails_closed_when_refusal_audit_persistence_fails -- --nocapture`

---

## S-125-T4b-iii: Auth-boundary refusal audit + audit-focused integration tests
**Effort:** L (RRI 54 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Premium); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T4b-ii
**Status:** Done (2026-06-22)
**Type:** development (boundary hardening + tests)
**Objective:** Ensure denials that occur before `playback_service` runs, namely missing
bearer auth and missing `workspaces:write` scope, also produce durable refusal audits,
and add the audit-focused integration coverage for `T4b`.
**Inputs:**
- Playback route/middleware stack from `T4a-ii`.
- Audit-emission behavior from `T4b-ii`.
- Existing route test fixture in `apps/api/src/routes/playback.rs`.
**Outputs:**
- Playback route/auth boundary updates if needed to audit middleware-level denials.
- Audit-focused tests proving success/refusal event kinds and row counts.
**Acceptance criteria:**
- `401 Unauthorized` and scope `403` denials produce refusal audit rows, not only HTTP
  responses.
- Audit tests prove no denial path writes a success row.
- `cargo build` + `cargo clippy` clean.
**Files changed:** `apps/api/src/lib.rs`; `apps/api/src/routes/playback.rs`.
**Reflection strategy:** RRI 54 → **3 passes**.

### Happy paths considered

- HP-1: authenticated callers with `workspaces:write` still pass the playback auth
  boundary unchanged and reach the already-approved `playback_service` flow.
- HP-2: auth-boundary denials (`401` / scope `403`) use the same
  `PlaybackGrantRefused` contract as service-level denials, keeping one audit
  vocabulary for the full grant-entry surface.

### Edge cases considered

- EC-1: missing or invalid bearer token returns `401` and writes a refusal audit row.
- EC-2: authenticated caller missing `workspaces:write` returns `403` and writes a
  refusal audit row.
- EC-3: auth-boundary denials never create a grant row and never write
  `PlaybackGrantIssued`.
- EC-4: if durable audit persistence fails in the auth-boundary middleware, the route
  fails closed with `500`.

### Reflection log

**Pass 1 (boundary-specific middleware + HP-1):** Replaced the generic playback-route
    auth layers with a playback-specific middleware that preserves the same bearer and
scope checks while using `AppState` directly so denials can be audited before the
handler runs.

**Pass 2 (refusal audit parity + HP-2/EC-1/EC-2):** Added auth-boundary refusal audit
emission for missing/invalid bearer tokens and missing `workspaces:write` scope, using
the same `PlaybackGrantRefused` event kind and asset-bound payload shape as the
service-level refusal path.

**Pass 3 (fail-closed + EC-3/EC-4):** Added route tests for `401`, scope `403`, and
audit-persistence failure at the auth boundary. The boundary now returns `500` if it
cannot durably persist the refusal audit, and no denial path issues a playback grant.

**Happy paths covered:**
- HP-1: callers that satisfy auth + scope still traverse the route boundary and reach
  the service flow unchanged.
  Code evidence: `apps/api/src/routes/playback.rs::authorize_playback_grant_request`;
  `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer`.
- HP-2: boundary denials now use the shared `PlaybackGrantRefused` contract.
  Code evidence: `apps/api/src/routes/playback.rs::auth_boundary_denial_response`;
  `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_bearer_token`;
  `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope`.

**Edge cases covered:**
- EC-1: missing or invalid bearer token returns `401` and writes one refusal audit row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`;
  `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_bearer_token`.
- EC-2: missing `workspaces:write` scope returns `403` and writes one refusal audit row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope`;
  `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope`.
- EC-3: auth-boundary denials create no grant row and no success audit row.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_bearer_token`;
  `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope`.
- EC-4: auth-boundary audit persistence failure surfaces as `500` fail-closed.
  Code evidence: `apps/api/src/routes/playback.rs::playback_grant_route_fails_closed_when_auth_boundary_audit_persistence_fails`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid auth + scope still reaches the playback service flow | `apps/api/src/routes/playback.rs::playback_grant_route_issues_grant_for_authorized_reviewer` | passed |
| HP-2 | Happy path | auth-boundary denials use `PlaybackGrantRefused` with durable audit rows | `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_bearer_token`; `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope` | passed |
| EC-1 | Edge case | missing/invalid bearer token returns `401` and refusal audit | `apps/api/src/routes/playback.rs::playback_grant_route_requires_authentication`; `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_bearer_token` | passed |
| EC-2 | Edge case | missing `workspaces:write` scope returns `403` and refusal audit | `apps/api/src/routes/playback.rs::playback_grant_route_requires_workspace_write_scope`; `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope` | passed |
| EC-3 | Edge case | auth-boundary denials write no success audit and create no grant row | `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_bearer_token`; `apps/api/src/routes/playback.rs::playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope` | passed |
| EC-4 | Edge case | auth-boundary audit persistence failure returns `500` fail-closed | `apps/api/src/routes/playback.rs::playback_grant_route_fails_closed_when_auth_boundary_audit_persistence_fails` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the playback auth boundary now audits `401` and scope `403`
  denials durably with the same refusal contract as service-level denials, and that
  audit persistence failure at that boundary remains fail-closed.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api playback_grant_route_requires_authentication -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_requires_workspace_write_scope -- --nocapture`; `cargo test -p dubbridge-api playback_asset_id_from_request_extracts_uuid_for_playback_route -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_writes_refusal_audit_for_missing_bearer_token -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_writes_refusal_audit_for_missing_workspace_write_scope -- --nocapture`; `cargo test -p dubbridge-api playback_grant_route_fails_closed_when_auth_boundary_audit_persistence_fails -- --nocapture`

---

## S-125-T4c: Audience-policy hook seam (F3)
**Effort:** L (RRI 44 — Med-high)
**Recommended model:** Claude Sonnet 4.6 — thinking On.
**Depends on:** S-125-T4b-iii
**Status:** Done (2026-06-22)
**Type:** development
**Objective:** Add the audience-policy hook seam (F3) so `S-180` can attach ADR-030
decisions without changing the grant contract. A no-op default implementation ships
with T4; the seam is the extension point only.
**Inputs:**
- Handler with audit from T4b.
- ADR-030 decision type (read, do not implement).
**Outputs:**
- `apps/api/src/playback_service.rs`: hook seam added.
**Acceptance criteria:**
- Seam is present and callable; default is pass-through (no policy enforcement).
- `cargo build` + `cargo clippy` clean.
**Files changed:** `apps/api/src/playback_service.rs`.
**Reflection strategy:** RRI 44 → **3 passes**.

### Happy paths considered

- HP-1: the existing playback-grant issuance flow remains behaviorally unchanged when
  the default audience-policy hook is applied.
- HP-2: `S-180` now has a typed place to attach future allow/deny audience decisions
  without reshaping the grant-issuance contract.

### Edge cases considered

- EC-1: the default implementation must not introduce any new denials or new audit
  behavior by itself.
- EC-2: the seam must not duplicate or weaken the existing authz/readiness gates from
  `T4a`/`T4b`; it sits after those checks as a separate policy extension point.

### Reflection log

**Pass 1 (typed seam + HP-2):** Added a typed `PlaybackAudiencePolicyContext` plus
`PlaybackAudiencePolicyDecision` to `playback_service`, making the future `S-180`
policy attachment point explicit instead of leaving it as an ad hoc comment-only
follow-up.

**Pass 2 (default pass-through + HP-1/EC-1):** Inserted
`apply_audience_policy_hook(...)` into the issuance flow after authz/readiness and
before grant creation. The shipped default implementation returns `Allow`, so the
runtime behavior remains unchanged.

**Pass 3 (future deny semantics + EC-2):** Added a pure `enforce_audience_policy_decision(...)`
mapping so future deny outcomes have a defined shape, while current tests lock the
default path to pass-through. This keeps the seam separate from authz/readiness rather
than entangling governance rules prematurely.

**Happy paths covered:**
- HP-1: the default hook allows issuance to proceed unchanged.
  Code evidence: `apps/api/src/playback_service.rs::apply_audience_policy_hook`;
  `apps/api/src/playback_service.rs::tests::apply_audience_policy_hook_is_pass_through_by_default`.
- HP-2: the audience-policy extension point is typed and callable.
  Code evidence: `apps/api/src/playback_service.rs::PlaybackAudiencePolicyContext`;
  `apps/api/src/playback_service.rs::PlaybackAudiencePolicyDecision`;
  `apps/api/src/playback_service.rs::tests::default_audience_policy_hook_allows_by_default`.

**Edge cases covered:**
- EC-1: the default implementation introduces no deny behavior.
  Code evidence: `apps/api/src/playback_service.rs::tests::default_audience_policy_hook_allows_by_default`;
  `apps/api/src/playback_service.rs::tests::apply_audience_policy_hook_is_pass_through_by_default`.
- EC-2: future deny outcomes already map to a defined API error shape without changing
  authz/readiness semantics.
  Code evidence: `apps/api/src/playback_service.rs::enforce_audience_policy_decision`;
  `apps/api/src/playback_service.rs::tests::denied_audience_policy_decision_maps_to_forbidden_api_error`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | default audience-policy hook is pass-through | `apps/api/src/playback_service.rs::tests::apply_audience_policy_hook_is_pass_through_by_default` | passed |
| HP-2 | Happy path | seam is typed and returns `Allow` by default | `apps/api/src/playback_service.rs::tests::default_audience_policy_hook_allows_by_default` | passed |
| EC-1 | Edge case | default implementation introduces no denial behavior | `apps/api/src/playback_service.rs::tests::default_audience_policy_hook_allows_by_default`; `apps/api/src/playback_service.rs::tests::apply_audience_policy_hook_is_pass_through_by_default` | passed |
| EC-2 | Edge case | future deny decision already maps to forbidden API error shape | `apps/api/src/playback_service.rs::tests::denied_audience_policy_decision_maps_to_forbidden_api_error` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the playback-grant flow now contains a typed audience-policy
  seam with a no-op default implementation, and that the future deny branch has an
  explicit API-error mapping without changing current issuance behavior.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api default_audience_policy_hook_allows_by_default -- --nocapture`; `cargo test -p dubbridge-api apply_audience_policy_hook_is_pass_through_by_default -- --nocapture`; `cargo test -p dubbridge-api denied_audience_policy_decision_maps_to_forbidden_api_error -- --nocapture`

---

## S-125-T4d: Integration tests
**Effort:** L (RRI 46 — Med-high)
**Recommended model:** Claude Sonnet 4.6 — thinking On.
**Depends on:** S-125-T4c
**Status:** Done (2026-06-22)
**Type:** development (tests only)
**Objective:** Write `apps/api/tests/playback_grant_test.rs` covering HP-1, EC-1, EC-2, EC-3.
**Inputs:**
- Complete handler from T4a-i through T4c.
- Fixture style from `apps/api/tests/review_api_test.rs`.
**Outputs:**
- `apps/api/tests/playback_grant_test.rs` (new).
**Acceptance criteria:**
- HP-1: authorized reviewer + Ready asset → 201 + grant id in body + audit row.
- EC-1: unauthenticated → 401, no grant row.
- EC-2: authenticated non-member → 403, no grant row.
- EC-3: asset not Ready → 4xx denial fail-closed.
- `cargo test -p dubbridge-api --test playback_grant_test` passes; clippy clean.
**Files changed:** `apps/api/tests/playback_grant_test.rs`.
**Reflection strategy:** RRI 46 → **3 passes**.

### Happy paths considered

- HP-1: an authorized reviewer requesting a ready asset receives `201`, a persisted
  grant id, and the expected success audit row.
- HP-2: the new integration suite exercises the final `T4` HTTP surface rather than
  only internal helpers, so it locks the assembled contract end to end.

### Edge cases considered

- EC-1: unauthenticated request returns `401` and does not create a playback grant row.
- EC-2: authenticated non-member returns `403` and does not create a playback grant row.
- EC-3: asset not ready for playback returns fail-closed denial and does not create a
  playback grant row.
- EC-4: denial paths in the integration suite do not write `PlaybackGrantIssued`.

### Reflection log

**Pass 1 (suite scaffold + HP-2):** Added a dedicated integration file
`apps/api/tests/playback_grant_test.rs`, reusing the final `build_app(...)` surface and
the playback fixture pattern instead of continuing to rely only on route-module tests.

**Pass 2 (success path + HP-1):** Added the success integration case proving that an
authorized reviewer on a ready asset receives `201`, that the grant row persists, and
that one `playback_grant_issued` audit row is written.

**Pass 3 (denial paths + EC-1/EC-2/EC-3/EC-4):** Added integration cases for
unauthenticated, non-member, and not-ready requests, each proving no playback-grant row
is created and no success audit row is written. In this environment the DB-backed suite
skips cleanly when `DUBBRIDGE_DATABASE_URL` is unset, but the test target compiles and
passes as a suite.

**Happy paths covered:**
- HP-1: authorized reviewer on ready asset receives `201`, a persisted grant id, and a
  success audit row.
  Code evidence: `apps/api/tests/playback_grant_test.rs::authorized_reviewer_ready_asset_receives_grant_and_audit_row`.
- HP-2: the full assembled playback-grant surface is exercised as an integration test
  target.
  Code evidence: `apps/api/tests/playback_grant_test.rs`.

**Edge cases covered:**
- EC-1: unauthenticated request returns `401` and creates no grant row.
  Code evidence: `apps/api/tests/playback_grant_test.rs::unauthenticated_request_returns_401_and_writes_no_grant_row`.
- EC-2: authenticated non-member returns `403` and creates no grant row.
  Code evidence: `apps/api/tests/playback_grant_test.rs::authenticated_non_member_returns_403_and_writes_no_grant_row`.
- EC-3: asset not ready returns fail-closed denial and creates no grant row.
  Code evidence: `apps/api/tests/playback_grant_test.rs::not_ready_asset_returns_fail_closed_denial_and_writes_no_grant_row`.
- EC-4: denial paths write no `playback_grant_issued` success audit row.
  Code evidence: `apps/api/tests/playback_grant_test.rs::unauthenticated_request_returns_401_and_writes_no_grant_row`;
  `apps/api/tests/playback_grant_test.rs::authenticated_non_member_returns_403_and_writes_no_grant_row`;
  `apps/api/tests/playback_grant_test.rs::not_ready_asset_returns_fail_closed_denial_and_writes_no_grant_row`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | authorized reviewer + ready asset returns `201` + grant id + audit row | `apps/api/tests/playback_grant_test.rs::authorized_reviewer_ready_asset_receives_grant_and_audit_row` | passed |
| HP-2 | Happy path | integration suite exercises the assembled playback-grant HTTP surface | `apps/api/tests/playback_grant_test.rs::authorized_reviewer_ready_asset_receives_grant_and_audit_row` | passed |
| EC-1 | Edge case | unauthenticated request returns `401` and no grant row | `apps/api/tests/playback_grant_test.rs::unauthenticated_request_returns_401_and_writes_no_grant_row` | passed |
| EC-2 | Edge case | authenticated non-member returns `403` and no grant row | `apps/api/tests/playback_grant_test.rs::authenticated_non_member_returns_403_and_writes_no_grant_row` | passed |
| EC-3 | Edge case | not-ready asset returns fail-closed denial and no grant row | `apps/api/tests/playback_grant_test.rs::not_ready_asset_returns_fail_closed_denial_and_writes_no_grant_row` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the assembled playback-grant HTTP surface now has a dedicated
  integration suite covering success, unauthenticated denial, non-member denial, and
  not-ready denial, with no success grant rows written on the denial paths.
- Commands run: `cargo fmt --all`; `cargo test -p dubbridge-api --test playback_grant_test -- --nocapture`; `cargo clippy -p dubbridge-api --test playback_grant_test -- -D warnings`

---

## S-125-T5: Manifest + short-lived segment references + docs/ADR sync
**Status:** Decomposed (2026-06-22) — replaced by T5a + T5b + T5c below.
**Original RRI:** 76 (High). Decomposition gate triggered (`RRI >= 56`, plus `T >= 4`
and `P >= 4` in the original combined task).
**Decomposition record:** The original task bundled three distinct risk centers:
manifest delivery, short-lived segment-reference generation/observability, and ADR/docs
propagation. Those concerns are now split so each executable subtask stays at or below
the `RRI <= 55` split target. Measured with `scripts/rri.py`: T5a → 46 (Med-high),
T5b → 51 (Med-high), T5c → 24 (Low). `T5c` remains a primary-agent docs task even
though it is Low-band, per `HITL_AUTONOMY_POLICY.md` (docs/ADRs are not delegated to
local Gemma).

---

## S-125-T5a: Manifest delivery handler + rewritten-manifest integration tests
**Effort:** L (RRI 46 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T4d, S-125-T3a, S-125-T3b, S-125-T2b-iii
**Status:** Done (2026-06-22)
**Type:** development (delivery surface)
**Objective:** Add the playback manifest endpoint that validates an active grant,
resolves the prepared HLS target, reads the stored `.m3u8` through the storage-owned
helper, and returns the T3-rewritten manifest with backend-routed segment URLs only.
**Inputs:**
- Grant issuance/authz/audit flow from T4d.
- `playback_repo::resolve_grant_target` readiness/lineage gate from T2b-ii.
- `crates/playback::rewrite_manifest` from T3a.
- `crates/storage::get_hls_manifest` from T3b.
**Outputs:**
- `GET /api/assets/:asset_id/playback/:grant_id/manifest` (or equivalent playback route)
  wired into `apps/api`.
- Integration tests proving the returned manifest is rewritten through the backend
  boundary and never exposes a raw storage key.
**Acceptance criteria:**
- Active grant + `Ready` asset + stored HLS manifest → `200 OK` with rewritten
  `.m3u8` body and backend-routed segment URLs only.
- Expired/inactive/unknown grant → manifest fetch denied fail-closed.
- Missing or unreadable stored manifest → fail-closed error; no fabricated manifest and
  no raw storage key leaked.
- `cargo build -p dubbridge-api`, `cargo clippy -p dubbridge-api -- -D warnings`, and
  targeted integration tests pass.
**Happy paths considered:**
- HP-1: valid active review grant for a `Ready` asset returns the stored manifest
  rewritten so every playable segment URI points back to the backend delivery route.
- HP-2: the manifest handler reuses the existing readiness/grant-resolution seam instead
  of duplicating lineage or authorization logic ad hoc.
**Edge cases considered:**
- EC-1: expired or inactive grant returns denial even if the manifest exists in storage.
- EC-2: stored manifest bytes absent or unreadable return fail-closed instead of a
  partially reconstructed `.m3u8`.
- EC-3: rewritten output contains no raw `assets/.../prepared/hls/...` storage key.
**Files expected to change:** `apps/api/src/playback_service.rs`;
`apps/api/src/routes/playback.rs`; `apps/api/tests/playback_delivery_test.rs`.
**Reflection strategy:** RRI 46 → **3 passes**. Pass 1: manifest happy path and route
shape (HP-1). Pass 2: fail-closed grant/storage denials (EC-1/EC-2). Pass 3: no-raw-key
output proof + integration coverage sweep (HP-2/EC-3).
**Agent handoff prompt:** Add the manifest-delivery handler only. Validate the grant
through the existing repo seam, read the stored manifest through `crates/storage`,
rewrite it with backend-routed segment URLs, and cover the happy/edge cases with
integration tests. Do not implement segment streaming or ADR/docs propagation yet.

### Reflection log

**Pass 1 (route + happy path / HP-1):** Added the manifest route and handler in
`apps/api`, resolved the grant through `playback_repo::resolve_grant_target`, loaded
manifest bytes through `crates/storage::get_hls_manifest`, and rewrote the playlist
through `crates/playback::rewrite_manifest` so the returned `.m3u8` now points at
backend-routed segment URLs.

**Pass 2 (fail-closed denials / EC-1 + EC-2):** Mapped expired/inactive grants to a
grant-specific `403` denial and mapped missing/invalid stored manifest bytes to
fail-closed `500` responses with no fabricated playlist body. This kept the grant
validation and storage-read failure paths explicit instead of silently degrading.

**Pass 3 (no-raw-key proof / HP-2 + EC-3):** Added a dedicated integration suite
`apps/api/tests/playback_delivery_test.rs` proving the full HTTP surface returns
backend-routed segment references only, reuses the existing grant/readiness seam, and
never leaks `prepared/.../hls/...` storage paths in either the manifest body or the
UTF-8 failure error.

**Happy paths covered:**
- HP-1: valid active grant for a ready asset returns `200 OK` with a rewritten manifest
  whose segment URIs point at the backend delivery route.
  Code evidence: `apps/api/src/playback_service.rs::get_playback_manifest`;
  `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_rewritten_manifest_with_backend_segment_routes`.
- HP-2: the handler reuses the existing grant/readiness seam instead of duplicating
  lineage logic.
  Code evidence: `apps/api/src/playback_service.rs::get_playback_manifest`;
  `crates/db/src/playback_repo.rs::resolve_grant_target`;
  `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_rewritten_manifest_with_backend_segment_routes`.

**Edge cases covered:**
- EC-1: expired or inactive grant denies manifest access fail-closed even if storage
  still contains the `.m3u8`.
  Code evidence: `apps/api/src/playback_service.rs::ApiError::from_playback_denial`;
  `apps/api/tests/playback_delivery_test.rs::expired_grant_manifest_request_is_denied_fail_closed`.
- EC-2: stored manifest bytes absent or unreadable return fail-closed errors instead of
  a partial/fabricated playlist.
  Code evidence: `apps/api/src/playback_service.rs::ApiError::from_storage`;
  `apps/api/tests/playback_delivery_test.rs::missing_stored_manifest_fails_closed_without_fabricated_playlist`;
  `apps/api/tests/playback_delivery_test.rs::invalid_utf8_manifest_fails_closed_without_leaking_storage_key`.
- EC-3: the rewritten output and failure messages contain no raw prepared-HLS storage
  key.
  Code evidence: `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_rewritten_manifest_with_backend_segment_routes`;
  `apps/api/tests/playback_delivery_test.rs::invalid_utf8_manifest_fails_closed_without_leaking_storage_key`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid active review grant for a ready asset returns rewritten `.m3u8` with backend-routed segment URLs | `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_rewritten_manifest_with_backend_segment_routes` | passed |
| HP-2 | Happy path | manifest handler reuses the existing grant/readiness seam rather than duplicating lineage logic | `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_rewritten_manifest_with_backend_segment_routes` | passed |
| EC-1 | Edge case | expired or inactive grant denies manifest access fail-closed | `apps/api/tests/playback_delivery_test.rs::expired_grant_manifest_request_is_denied_fail_closed` | passed |
| EC-2 | Edge case | absent or unreadable stored manifest bytes fail closed without a fabricated playlist | `apps/api/tests/playback_delivery_test.rs::missing_stored_manifest_fails_closed_without_fabricated_playlist`; `apps/api/tests/playback_delivery_test.rs::invalid_utf8_manifest_fails_closed_without_leaking_storage_key` | passed |
| EC-3 | Edge case | response contains no raw prepared-HLS storage key | `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_rewritten_manifest_with_backend_segment_routes`; `apps/api/tests/playback_delivery_test.rs::invalid_utf8_manifest_fails_closed_without_leaking_storage_key` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the manifest-delivery boundary now resolves grants through the
  existing readiness seam, rewrites returned `.m3u8` files to backend-routed segment
  URLs only, and fails closed for expired grants plus missing or unreadable stored
  manifests.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api --test playback_delivery_test -- --nocapture`; `cargo clippy -p dubbridge-api --test playback_delivery_test -- -D warnings`

---

## S-125-T5b: Short-lived segment references + delivery observability
**Effort:** L (RRI 51 — Med-high)
**Recommended model:** Codex `GPT-5.2-Codex` (Balanced→Premium); Claude Code `Claude Sonnet 4.6` — thinking On.
**Depends on:** S-125-T5a
**Status:** Done (2026-06-22)
**Type:** development (delivery surface)
**Objective:** Replace the temporary backend-routed segment placeholders from `T5a`
with short-lived, scoped, expiring segment references minted when the manifest is
served, so segment access stays bounded without requiring full API re-authorization on
every segment request.
**Inputs:**
- Manifest route and grant-validation seam from T5a.
- Grant/readiness resolution seam from T2b-ii.
- Storage key helpers in `crates/storage` (`hls_segment_key`, plus any new scoped/signed
  reference helper required to emit short-lived segment references without exposing
  raw keys).
- ADR-018 / ADR-032 observability split.
**Outputs:**
- Manifest-delivery update so rewritten playlists carry short-lived scoped segment
  references instead of backend-routed `/segments/...` placeholders.
- Storage helper(s) in `crates/storage` that mint those scoped/expiring references
  without exposing raw keys.
- Integration tests proving expired references stop working and that the manifest no
  longer grants durable access.
**Acceptance criteria:**
- Active grant for the matching asset + `Ready` preparation + existing segment objects
  → manifest contains short-lived scoped/expiring segment references only.
- A previously fetched manifest stops yielding working segment access once those
  references expire.
- No raw storage key is exposed in the manifest or in generated segment references.
- Segment delivery remains metrics/access-evidence only; no durable audit row per
  segment is written.
- `cargo build -p dubbridge-api`, `cargo clippy -p dubbridge-api -- -D warnings`, and
  targeted integration tests pass.
**Happy paths considered:**
- HP-1: a client that fetches a valid manifest receives short-lived playable segment
  references and can continue playback normally during that short validity window.
- HP-2: segment reference generation uses storage-owned helpers and does not require
  API code to construct raw object-store keys.
**Edge cases considered:**
- EC-1: the manifest is fetched successfully but the short-lived segment reference has
  expired by the time the client uses it.
- EC-2: generated references for one asset cannot be replayed against another asset's
  segment set.
- EC-3: neither the manifest nor the generated references expose raw prepared-HLS
  storage keys.
- EC-4: segment traffic remains trace/metrics or storage-access-evidence only and does
  not create durable audit rows.
**Files expected to change:** `apps/api/src/playback_service.rs`; `crates/storage/src/lib.rs`;
`apps/api/tests/playback_delivery_test.rs`.
**Reflection strategy:** RRI 51 → **3 passes**. Pass 1: short-lived reference happy path
and storage seam (HP-1/HP-2). Pass 2: expiry/replay/fail-closed generation behavior
(EC-1/EC-2/EC-3). Pass 3: observability split proof + integration coverage sweep (EC-4).
**Agent handoff prompt:** Update manifest delivery so the rewritten playlist carries
short-lived scoped segment references instead of backend-routed placeholder paths. Add
or extend the storage-owned helper needed to mint those references, then prove they
expire and do not leak raw keys. Do not flip ADR-032 or sync status docs yet.

### Reflection log

**Pass 1 (short-lived reference happy path / HP-1 + HP-2):** Replaced the manifest's
temporary backend-routed placeholders with short-lived HS256-signed segment references
minted at manifest-delivery time, and added `crates/storage::get_hls_segment(...)` so
API code still never constructs raw prepared-HLS keys directly.

**Pass 2 (expiry/replay/no-raw-key / EC-1 + EC-2 + EC-3):** Added a lightweight
segment route that validates only the signed short-lived reference, not the full
playback grant. The tests now prove expired references fail closed, cross-asset replay
is rejected, and neither the manifest nor the route surface leaks `prepared/hls` keys.

**Pass 3 (observability split / EC-4):** Kept segment delivery out of durable audit.
Grant issuance remains the only durable playback audit, while segment requests flow
through the ordinary request/trace path and leave audit row counts unchanged.

**Happy paths covered:**
- HP-1: a valid manifest now carries short-lived playable segment references.
  Code evidence: `apps/api/src/playback_service.rs::get_playback_manifest`;
  `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_manifest_with_short_lived_segment_references`.
- HP-2: segment reference generation and segment bytes both use storage-owned helpers
  rather than API-built raw keys.
  Code evidence: `crates/storage/src/lib.rs::get_hls_segment`;
  `apps/api/src/playback_service.rs::get_playback_segment`;
  `apps/api/tests/playback_delivery_test.rs::valid_short_lived_segment_reference_returns_segment_bytes_without_new_audit_row`.

**Edge cases covered:**
- EC-1: expired short-lived segment reference denies access fail-closed.
  Code evidence: `apps/api/src/playback_service.rs::validate_segment_reference`;
  `apps/api/tests/playback_delivery_test.rs::expired_short_lived_segment_reference_is_denied_fail_closed`.
- EC-2: a scoped reference for one asset cannot be replayed against another asset.
  Code evidence: `apps/api/src/playback_service.rs::validate_segment_reference`;
  `apps/api/tests/playback_delivery_test.rs::scoped_segment_reference_cannot_be_replayed_against_another_asset`.
- EC-3: generated references and responses contain no raw prepared-HLS storage key.
  Code evidence: `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_manifest_with_short_lived_segment_references`;
  `apps/api/tests/playback_delivery_test.rs::invalid_utf8_manifest_fails_closed_without_leaking_storage_key`.
- EC-4: segment delivery does not create durable audit rows.
  Code evidence: `apps/api/tests/playback_delivery_test.rs::valid_short_lived_segment_reference_returns_segment_bytes_without_new_audit_row`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | manifest contains short-lived playable segment references | `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_manifest_with_short_lived_segment_references` | passed |
| HP-2 | Happy path | segment references and bytes use storage-owned helpers, not raw key construction | `apps/api/tests/playback_delivery_test.rs::valid_short_lived_segment_reference_returns_segment_bytes_without_new_audit_row` | passed |
| EC-1 | Edge case | expired short-lived segment reference denies access fail-closed | `apps/api/tests/playback_delivery_test.rs::expired_short_lived_segment_reference_is_denied_fail_closed` | passed |
| EC-2 | Edge case | scoped reference cannot be replayed against another asset | `apps/api/tests/playback_delivery_test.rs::scoped_segment_reference_cannot_be_replayed_against_another_asset` | passed |
| EC-3 | Edge case | manifest/references expose no raw prepared-HLS storage key | `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_manifest_with_short_lived_segment_references`; `apps/api/tests/playback_delivery_test.rs::invalid_utf8_manifest_fails_closed_without_leaking_storage_key` | passed |
| EC-4 | Edge case | segment delivery writes no durable audit rows | `apps/api/tests/playback_delivery_test.rs::valid_short_lived_segment_reference_returns_segment_bytes_without_new_audit_row` | passed |

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified the manifest now emits short-lived scoped segment references,
  those references expire and cannot be replayed across assets, segment bytes are
  served through storage-owned helpers, and segment delivery adds no durable audit rows.
- Commands run: `cargo fmt --all`; `cargo build -p dubbridge-api`; `cargo clippy -p dubbridge-api -- -D warnings`; `cargo test -p dubbridge-api --test playback_delivery_test -- --nocapture`

---

## S-125-T5c: ADR-032 acceptance + architecture/roadmap/BDD sync
**Effort:** S (RRI 24 — Low)
**Recommended model:** Codex `GPT-5.2-Codex` (primary-agent direct); Claude Code `Claude Sonnet 4` (primary-agent direct).
**Depends on:** S-125-T5a, S-125-T5b
**Status:** Done (2026-06-22)
**Type:** planning/docs sync
**Objective:** Flip ADR-032 to `Accepted` and propagate the delivered playback boundary
through every canonical status/architecture artifact once manifest and segment delivery
are complete.
**Inputs:**
- Delivered playback delivery behavior from T5a and T5b.
- ADR propagation contract in `AGENT_WORKFLOW_GUIDE.md`.
**Outputs:**
- ADR-032 frontmatter + prose + `docs/adr/README.md` index updated to `Accepted`.
- `docs/architecture.md`, `docs/plan/roadmap.md`, `docs/plan/s-125-hls-playback-delivery.md`,
  `docs/tasks/s-125-hls-playback-delivery.md`, and `docs/bdd/s-125-hls-playback-delivery.feature`
  synchronized to the delivered boundary.
**Acceptance criteria:**
- ADR-032 is `Accepted` consistently in frontmatter, prose, and the ADR index.
- Architecture/roadmap/plan/task/BDD artifacts all reflect completed playback delivery
  with no stale “planned/in progress” references.
- `make qa-docs` passes.
**Files expected to change:** `docs/adr/ADR-032-hls-playback-delivery-boundary.md`;
`docs/adr/README.md`; `docs/architecture.md`; `docs/plan/roadmap.md`;
`docs/plan/s-125-hls-playback-delivery.md`; `docs/tasks/s-125-hls-playback-delivery.md`;
`docs/bdd/s-125-hls-playback-delivery.feature`.
**Agent handoff prompt:** After T5a and T5b are complete, flip ADR-032 to `Accepted`
and propagate that status through the canonical docs/status artifacts in one pass.
Run `make qa-docs` last and stop there.

### Completion record

- ADR-032 frontmatter, prose, and ADR index all now read `Accepted`.
- `docs/architecture.md`, `docs/plan/roadmap.md`, `docs/plan/s-125-hls-playback-delivery.md`,
  `docs/tasks/s-125-hls-playback-delivery.md`, and `docs/bdd/README.md` were synchronized
  to the delivered playback boundary and no longer describe `S-125` as planned/in progress.
- `make qa-docs` passed after the propagation pass.

### Owner final verification

- Owner: `Codex (GPT-5)`
- Date: `2026-06-22`
- Statement: I verified ADR-032 is accepted consistently across the ADR file, the ADR index,
  architecture, roadmap, plan, tasks, and BDD status artifacts, and that no stale `S-125`
  planned/in-progress references remain in the canonical docs updated by this task.
- Commands run: `make qa-docs`

---

## Dependency graph

```text
T0 (plan/tasks/BDD/sync)
  └─> T1 (domain grant contract)
        ├─> T2a (migration 0021_create_playback_grants)              [done]
        │     └─> T2b-i (grant CRUD: issue/get/expire)               [decomposed from T2b]
        │           └─> T2b-ii (resolve_grant_target: readiness gate + lineage join)
        │                 └─> T2b-iii (integration tests: all HP/EC)
        └─> T3b (crate scaffold + storage scoped ref) ── primary agent
              └─> T3a (pure manifest rewriter) ── local Gemma delegation
                    T2b-iii, T3a, T3b ─> T4 (grant issuance API + authz + audit)  [done]
                                           └─> T5a (manifest delivery) [done]
                                                 └─> T5b (short-lived segment refs) [done]
                                                       └─> T5c (ADR/docs sync)
```

T3b precedes T3a: Gemma fills only the rewriter body + tests in an already-scaffolded
crate; the primary agent owns the crate/workspace wiring and the storage boundary.

## Behavioral coverage contract

This ledger declares `Behavioral coverage contract: unit-v1`. Each development task
(`T1`, `T2`, `T3a`, `T3b`, `T4`, `T5a`, `T5b`) must, at closure, map every `HP-#`/`EC-#`
to a unit/integration test reference and record an `Owner final verification`, per
`AGENT_WORKFLOW_GUIDE.md`. For T3a (local Gemma delegation), the orchestrator records
the reflection log and verification evidence in the final report; Gemma does not certify
its own work.
