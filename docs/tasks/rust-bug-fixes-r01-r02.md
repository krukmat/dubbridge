---
type: TaskList
title: "Tasks: Rust Bug Fixes — R-01 / R-02"
plan: ""
status: active
rri: 18
band: Low
effort: S
---
# Tasks: Rust Bug Fixes — R-01 / R-02

## Objective

Fix two confirmed Low-RRI bugs found in the Rust DB layer via codebase
exploration on 2026-06-25. Both are delegated to Gemma Developer (before-after
mode) followed by mandatory Gemma Reviewer triple-pass per ADR-034.

## Governing Documents

- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`

## Ground Truth

| ID | File | Line(s) | Description | RRI |
|----|------|---------|-------------|-----|
| R-01 | `crates/db/src/audit_repo.rs` | 86, 105 | `recording_session_id` column omitted from both INSERT queries — exists in DB since migration 0009, silently lost on every recording lifecycle audit event | 18 |
| R-02 | `crates/db/src/pending_ingestion_repo.rs` | 298, 311 | `parse_license_type` and `parse_source_type` return `DbError::QueryFailed(Protocol(...))` for unknown values — should be `DbError::UnknownStoredValue` per repo contract | 8 |

---

## T1 — R-01: Persist `recording_session_id` in audit INSERT queries

**Status:** done ✅
**Effort:** S · **RRI:** 18 · **Band:** Low
**Commit:** `556cd40`

### Context

`AuditEvent::new_recording()` sets `recording_session_id: Some(uuid)`. Both
`insert_audit_event_tx` and `insert_audit_event` omitted the column — value was
silently dropped on every INSERT. The SELECT in
`list_audit_events_for_owned_asset` and `row_to_event` already handled the
column correctly. Fix: add column + `$7` + `.bind(event.recording_session_id)`
to both functions.

### Inputs

- `crates/db/src/audit_repo.rs` lines 84–120 (both INSERT functions)
- `infra/migrations/0009_alter_audit_events_for_recording.sql` — confirms column exists

### Outputs

- Both INSERT queries include `recording_session_id` as `$7`
- `.bind(event.recording_session_id)` added to both bind chains

### Acceptance criteria

1. Both INSERT functions include `recording_session_id` in column list and VALUES
2. Both bind `event.recording_session_id` as the 7th parameter
3. `cargo check -p dubbridge-db` passes
4. `cargo test -p dubbridge-db --lib` — 67 tests green

### Arbiter verdict

- **Developer:** Gemma produced correct patch on first attempt (502 tokens, 14s,
  before-after mode). Scope clean — no changes outside the two INSERT functions.
- **Reviewer:** STATUS PASS · 3/3 passes · 0 findings · degraded: false.
  Format artifact: stray `=== FINDING END ===` on 2/3 passes — logged as PG-09,
  parser fix committed alongside patch.
- **Verification:** `cargo check` ✅ · 67 unit tests ✅

---

## T2 — R-02: Use `UnknownStoredValue` in `parse_license_type` / `parse_source_type`

**Status:** done ✅
**Effort:** S · **RRI:** 8 · **Band:** Low
**Commit:** `841a564`

### Context

`parse_license_type()` and `parse_source_type()` in `pending_ingestion_repo.rs`
return `DbError::QueryFailed(sqlx::Error::Protocol(format!(...)))` for unknown
stored values. Every other DB repo uses `DbError::UnknownStoredValue { field,
value }` for this case (see `rights_repo.rs` lines 31 and 45). The mismatch
breaks the fail-closed pattern from ADR-008: callers that `match` on
`DbError::UnknownStoredValue` to detect data integrity issues will miss these
two parse paths.

No callers currently pattern-match on `UnknownStoredValue` from this repo, so
the change is safe and non-breaking. The file was written before the pattern
was established.

### Inputs

```rust
// crates/db/src/pending_ingestion_repo.rs lines 291–315 (verbatim BEFORE):
fn parse_license_type(value: &str) -> Result<LicenseType, DbError> {
    match value {
        "all_rights_reserved" => Ok(LicenseType::AllRightsReserved),
        "creative_commons" => Ok(LicenseType::CreativeCommons),
        "public_domain" => Ok(LicenseType::PublicDomain),
        "licensed_distribution" => Ok(LicenseType::LicensedDistribution),
        "internal_only" => Ok(LicenseType::InternalOnly),
        _ => Err(DbError::QueryFailed(sqlx::Error::Protocol(format!(
            "unknown license_type '{value}'"
        )))),
    }
}

fn parse_source_type(value: &str) -> Result<SourceType, DbError> {
    match value {
        "direct_upload" => Ok(SourceType::DirectUpload),
        "authorized_s3" => Ok(SourceType::AuthorizedS3),
        "internal_feed" => Ok(SourceType::InternalFeed),
        "licensed_source" => Ok(SourceType::LicensedSource),
        "public_domain_with_proof" => Ok(SourceType::PublicDomainWithProof),
        _ => Err(DbError::QueryFailed(sqlx::Error::Protocol(format!(
            "unknown source_type '{value}'"
        )))),
    }
}
```

### Outputs

```rust
// AFTER — both wildcard arms:
_ => Err(DbError::UnknownStoredValue {
    field: "pending_ingestions.license_type",
    value: value.to_owned(),
})

_ => Err(DbError::UnknownStoredValue {
    field: "pending_ingestions.source_type",
    value: value.to_owned(),
})
```

### Acceptance criteria

1. `parse_license_type` wildcard arm returns `DbError::UnknownStoredValue { field: "pending_ingestions.license_type", value: value.to_owned() }`
2. `parse_source_type` wildcard arm returns `DbError::UnknownStoredValue { field: "pending_ingestions.source_type", value: value.to_owned() }`
3. No other lines modified
4. `cargo check -p dubbridge-db` passes
5. `cargo test -p dubbridge-db --lib` green

### Delegation spec

- Mode: `before-after`
- Target: `crates/db/src/pending_ingestion_repo.rs`
- Before file: exact text of lines 291–315 shown above
- Allow path: `crates/db/src/pending_ingestion_repo.rs`
- Constraint: only touch the two `_` wildcard arms; do not add imports (
  `DbError::UnknownStoredValue` is already in scope via the existing `use
  crate::error::DbError` import)

### Execution log

- Developer patch landed in `841a564` (`fix(db+tooling): R-02 UnknownStoredValue + reviewer think=true default`).
- Follow-up formatting landed in `9400b79` (`style(db): cargo fmt R-02 wildcard arms`).
- Verified current file state matches the intended two-arm replacement exactly.
- Verification rerun on 2026-07-16:
  - `cargo check -p dubbridge-db`
  - `cargo test -p dubbridge-db --lib`

### Arbiter verdict

- **Developer:** Gemma-produced patch is present in the repo and matches the task contract: both wildcard arms now return `DbError::UnknownStoredValue` with the correct `field` names and `value.to_owned()`.
- **Scope:** Clean. The requested behavior change is confined to `crates/db/src/pending_ingestion_repo.rs`; no extra code changes were needed for the fix itself beyond the later formatting commit.
- **Verification:** `cargo check -p dubbridge-db` ✅ · `cargo test -p dubbridge-db --lib` ✅ (`69` tests passed on 2026-07-16).
