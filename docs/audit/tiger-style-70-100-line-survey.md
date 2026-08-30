---
type: Audit
title: "Tiger Style adoption — 70-100 line function survey (X26-T0)"
status: active
---

# Tiger Style adoption — 70-100 line function survey (X26-T0)

## Purpose

Enumerates every function in the Rust workspace whose body clippy's
`too_many_lines` lint measures at 70 lines or more (its default ceiling is
100; the Cargo-workspace lint is `deny`d at the 100-line default today —
`Cargo.toml:65`), so `X26-T1`'s decomposition work and `X26-T2`'s threshold
flip have a known, bounded blast radius. No source file was modified to
produce this survey.

## Method

Reused the workspace's own `clippy::too_many_lines` lint instead of a
bespoke script. Ran with the threshold temporarily lowered to 70 via
`clippy.toml`, and the lint level forced to `warn` on the command line
(the Cargo-workspace `deny` would otherwise hard-error out of the first
offending crate and skip every crate depending on it):

```bash
# clippy.toml temporarily carried:
#   too-many-lines-threshold = 70
cargo clippy --workspace --all-targets --all-features -- \
  -A clippy::all -W clippy::too_many_lines
```

`clippy.toml` was reverted to its committed state immediately after this run
captured its output; the change was never committed. `X26-T2` owns making the
70-line threshold permanent in `Cargo.toml`/`clippy.toml`.

Clippy's line count excludes the function signature line, blank lines, and
comment-only lines within the body — it is **not** the same as a raw
`start..end` file-span count. `finalize_ingestion_core` is called out below
with both measures since the plan/task ledger cites the raw span (`crates/
ingestion/src/lib.rs:48-145`, 97 lines).

## Survey table

| # | File | Function | Lines (clippy count / 70) | Kind | Status |
|---|------|----------|---------------------------|------|--------|
| 1 | `crates/config/src/lib.rs:341` | `GatewaySettings::validate` | 71 | production | **Resolved by X26-T1** — decomposed into `validate` (6 lines) + `validate_required_fields` (61) + `validate_production_constraints` (~26) |
| 2 | `crates/ingestion/src/lib.rs:48` | `finalize_ingestion_core` (raw span 48-144, 97 lines — see Method) | 77 | production | **Resolved by X26-T1** — decomposed into `finalize_ingestion_core` (41) + `lock_pending_or_reject` (25) + `build_finalize_command` (19) + `persist_finalization_writes` (46), threading the same `sqlx::Transaction` throughout (ADR-006/008/021 atomicity preserved) |
| 3 | `apps/api/src/routes/workspace.rs:39` | `router` | 76 | production | **Resolved by X26-T1** — decomposed into `router` (7) + `global_write_routes` (12) + `global_read_routes` (12) + `org_write_routes` (29) + `org_read_routes` (29) |
| 4 | `apps/api/tests/delivery_scope_repo_test.rs:31` | `seed_scope` | 78 | test helper | **Resolved by X26-T2** — decomposed into `seed_scope_project_and_asset` + `seed_scope_targets` (mechanical, order-preserving extraction) |
| 5 | `apps/api/tests/review_repo_test.rs:37` | `insert_review_scope` | 84 | test helper | **Resolved by X26-T2** — decomposed into `insert_review_org_and_projects` + `insert_review_assets_and_language` (mechanical, order-preserving extraction) |
| 6 | `apps/api/tests/workspace_test.rs:66` | `TestContext::new` | 74 | test helper | **Resolved by X26-T2** — stub-token-verifier construction extracted into `build_stub_verifier` (pure builder, no state) |
| 7 | `apps/api/tests/localization_repo_test.rs:267` | `translation_claim_and_promote_ready_persists_exact_current_artifacts` | 88 | test | **Resolved by X26-T2** — kept as one scenario, justified `#[allow(clippy::too_many_lines)]` (single linear claim→assert→promote→assert narrative, CC 0) |
| 8 | `apps/api/tests/localization_repo_test.rs:368` | `translation_redelivery_same_request_reuses_existing_claim` | 83 | test | **Resolved by X26-T2** — justified `#[allow(clippy::too_many_lines)]` (same rationale as row 7) |
| 9 | `apps/api/tests/localization_repo_test.rs:512` | `translation_promote_ready_rejects_wrong_kind_wrong_parent_and_other_asset_outputs` | 87 | test | **Resolved by X26-T2** — justified `#[allow(clippy::too_many_lines)]`; phase-2 D14 review flagged this one as a plausible future candidate for splitting into 2-3 `#[tokio::test]` cases (independent negative-case matrix) instead of suppression — non-blocking stylistic note, not acted on in this task |
| 10 | `apps/api/tests/localization_repo_test.rs:683` | `translation_stale_generation_cannot_overwrite_new_current_output` | 84 | test | **Resolved by X26-T2** — justified `#[allow(clippy::too_many_lines)]` (same rationale as row 7) |
| 11 | `apps/api/tests/localization_repo_test.rs:779` | `dubbing_claim_and_promote_ready_persists_exact_manifest_and_audio` | 98 | test | **Resolved by X26-T2** — justified `#[allow(clippy::too_many_lines)]` (same rationale as row 7) |
| 12 | `apps/api/tests/localization_repo_test.rs:892` | `dubbing_redelivery_same_request_reuses_existing_claim` | 89 | test | **Resolved by X26-T2** — justified `#[allow(clippy::too_many_lines)]` (same rationale as row 7) |

12 functions total: **3 production**, **9 test** (1 test helper counted
separately from the 8 `#[tokio::test]` integration tests above it — see row
5/6 vs. rows 7-12).

## Resolution (X26-T1 + X26-T2, 2026-08-30)

Rows 1-3 (production) closed by `X26-T1` — see
`docs/tasks/tiger-style-adaptation.md` § `X26-T1` for the full closure record
(diff, Reflection log, phase-1/phase-2 D14 review artifacts, unit coverage
certification, owner verification).

Rows 4-12 (test code) closed by `X26-T2` — see
`docs/tasks/tiger-style-adaptation.md` § `X26-T2` for the full closure record
(RRI 24 recompute, D14 phase-1/phase-2 review artifacts, `make qa-lint`
before/after, unit coverage certification, owner verification). All 12 rows
are now resolved; `clippy.toml`'s `too-many-lines-threshold` is set to 70
workspace-wide with zero unjustified `#[allow(clippy::too_many_lines)]`
attributes.

**Open follow-up (not blocking, carried in `X26-T2`'s closure record):** the
3 decomposed test-fixture functions (rows 4-6) were verified by
compilation, `cargo fmt`, `cargo clippy`, and manual line-by-line diff
review only — this execution environment has no reachable Postgres
(`DUBBRIDGE_DATABASE_URL` unset, Docker image pulls blocked by the outbound
network allowlist), so every touched integration test hit its
`setup_pool()`/`TestContext::new()` early-return path and never exercised
the decomposed DB-writing logic at runtime. Genuine behavioral verification
of insert order, bind-parameter order, and token-mapping fidelity for
`seed_scope_project_and_asset`/`seed_scope_targets`,
`insert_review_org_and_projects`/`insert_review_assets_and_language`, and
`build_stub_verifier` is still owed at the next CI run or session with live
Postgres access.

## Notes for X26-T1

- Only rows 1-3 (production code) are in `X26-T1`'s stated scope
  (`finalize_ingestion_core` explicitly named, plus any other flagged
  function). The 9 test-code rows are not production logic; `X26-T1`'s
  acceptance criteria ("existing tests... pass unmodified") implies tests are
  not what's being decomposed — this needs an explicit scope decision before
  `X26-T1` starts (see Open question below).
- `finalize_ingestion_core`'s own citation in `docs/plan/tiger-style-
  adaptation.md` already notes it is "already partially decomposed into five
  helpers under 20 lines each" — confirmed by direct inspection this
  session; the remaining 77-line (clippy count) / 97-line (raw span) body is
  the orchestration wrapper around those helpers.

## Open question (for X26-T1 scoping, not resolved by this survey)

`X26-T0`'s objective and acceptance criteria say "every function... across
the Rust workspace" with no test/production distinction, but `X26-T1`'s
acceptance criteria only discuss production behavior preservation. Whether
the 9 test-code rows are in `X26-T1`'s scope or a separate/exempt bucket is
not decided by this survey — it is a scoping question for whoever presents
`X26-T1` for approval (already presented once this session at RRI 43 using
only the 3 production rows; if test rows are added to scope, C/F inputs and
the RRI recompute would need to account for the larger touched-file set).

## Reproduction

```bash
cargo clippy --workspace --all-targets --all-features -- \
  -A clippy::all -W clippy::too_many_lines
# with clippy.toml carrying: too-many-lines-threshold = 70
```

Raw output captured at survey time: 12 `too_many_lines` warnings, 0 build
errors, workspace `Finished` cleanly.
