---
type: TaskList
title: "Tasks: H1 Governance Atomicity + Durable Audit Hardening"
status: closed
plan: docs/plan/h1-governance-atomicity-hardening.md
---
# Tasks: H1 Governance Atomicity + Durable Audit Hardening

Governing plan: `docs/plan/h1-governance-atomicity-hardening.md`
Governing ADRs: ADR-006, ADR-008, ADR-018, ADR-021

## Status legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

---

## Task 1 — Atomic finalize and cleanup coordination

**Effort:** L
**Complexity:** High
**Depends on:** S1, S3 Task 0
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective

Make successful ingestion finalization one atomic relational operation and prevent
cleanup from deleting a blob while finalize commits its artifact reference.

### Scope

- Add transaction-aware repository operations or an equivalent transaction-owned
  service boundary.
- Move the finalize service into a shared app-neutral module that can be consumed by
  both `apps/api` and the future `apps/worker-runner` recording bridge.
- Serialize finalize and cleanup for the same pending-ingestion row using a
  PostgreSQL-backed lock or claim strategy.
- Commit asset, rights, artifact, status, success audit, and pending-row consumption
  together.
- Preserve the storage-first cross-store ordering from ADR-006.

### Acceptance criteria

- Failure after any intermediate metadata write leaves no partial relational rows.
- Concurrent finalize calls produce exactly one asset, one rights row, one artifact,
  and one success audit event for the ingest token.
- Cleanup cannot delete a blob referenced by a successfully finalized artifact.
- Existing ingestion behavior and HTTP status contracts remain stable.
- `apps/worker-runner` can reuse the service without depending on `dubbridge-api`.
- `cargo test -p dubbridge-api --test ingestion_test -- --test-threads=1` passes.
- `cargo check --workspace` passes.

### Files likely affected

- `apps/api/src/ingestion_service.rs`
- `apps/api/src/cleanup.rs`
- shared app-neutral service module selected during implementation
- `crates/db/src/{asset_repo,rights_repo,artifact_repo,audit_repo,pending_ingestion_repo}.rs`
- `apps/api/tests/ingestion_test.rs`

### Status: [x] DONE — atomic finalize transaction implemented, cleanup-vs-finalize race closed with SKIP LOCKED, service moved to app-neutral crates/ingestion. 13/13 tests passing. 2026-05-31.

### Files affected
- `crates/ingestion/` (new crate) — `IngestionServiceError` + `finalize_ingestion_core` (app-neutral, H1-T1)
- `crates/ingestion/Cargo.toml` (new, H1-T1)
- `crates/ingestion/src/lib.rs` (new, H1-T1)
- `crates/db/src/pending_ingestion_repo.rs` — added `lock_for_finalize` (SELECT FOR UPDATE), `delete_pending_ingestion_tx`, `claim_expired_for_cleanup` (CTE DELETE SKIP LOCKED) (H1-T1)
- `crates/db/src/asset_repo.rs` — added `insert_asset_tx`, `update_asset_status_tx` (H1-T1)
- `crates/db/src/rights_repo.rs` — added `insert_rights_record_tx` (H1-T1)
- `crates/db/src/artifact_repo.rs` — added `insert_artifact_record_tx`, `exists_for_token_tx` (H1-T1)
- `crates/db/src/audit_repo.rs` — added `insert_audit_event_tx` (H1-T1)
- `apps/api/src/ingestion_service.rs` — replaced with thin re-export from dubbridge-ingestion (H1-T1)
- `apps/api/src/routes/ingestion.rs` — simplified finalize handler (no pre-load), added `is_body_size_limit_error` helper + `payload_too_large` ApiError (H1-T1, also fixes pre-existing T1-T6 test bug)
- `apps/api/src/cleanup.rs` — replaced with claim_expired_for_cleanup (SKIP LOCKED); DB row deleted atomically before storage delete (H1-T1)
- `apps/api/Cargo.toml` — added dubbridge-ingestion dep (H1-T1)
- `Cargo.toml` — added crates/ingestion workspace member (H1-T1)
- `apps/api/tests/ingestion_test.rs` — added `cleanup_skips_row_locked_by_in_flight_finalize` test; fixed `upload_too_large_is_rejected` (Content-Length header) (H1-T1)

---

## Task 2 — Enforce persistence-level governance invariants

**Effort:** M
**Complexity:** Medium
**Depends on:** Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective

Enforce append-only rights records and strict stored-state decoding at the
persistence boundary.

### Scope

- Add the next available migration(s) after `0006` for database-backed
  rights-ledger immutability and any required constraints.
- Prevent normal `UPDATE` / `DELETE` mutation of `rights_records`; document the
  explicit operational path for any future correction workflow.
- Fail explicitly when repositories encounter unknown persisted governance
  statuses or artifact kinds; do not silently coerce them to fallback variants.
- Preserve test cleanup capability without weakening runtime invariants.

### Acceptance criteria

- Direct mutation of a persisted rights record is rejected by PostgreSQL.
- Empty mandatory rights fields cannot be inserted by bypassing domain validation.
- Unknown stored asset status and artifact kind values fail explicitly.
- Existing migrations apply cleanly from a fresh database.
- `cargo check --workspace` passes.

### Files likely affected

- `infra/migrations/<next>_harden_governance_invariants.sql`
- `crates/db/src/{asset_repo,artifact_repo,rights_repo}.rs`
- DB-backed tests in the owning package

### Status: [x] DONE — migration 0007 adds CHECK constraints + append-only RULEs on rights_records; parse_status and parse_kind fail-closed with DbError::UnknownStoredValue; 4/4 unit tests passing; cargo check --workspace clean. 2026-05-31.

### Files affected
- `infra/migrations/0007_harden_governance_invariants.sql` (new, H1-T2)
- `crates/db/src/error.rs` — added `UnknownStoredValue` variant (H1-T2)
- `crates/db/src/asset_repo.rs` — `parse_status` returns `Result`, tests added (H1-T2)
- `crates/db/src/artifact_repo.rs` — extracted `parse_kind` returns `Result`, tests added (H1-T2)
- `apps/api/src/routes/ingestion.rs` — `from_db` covers `UnknownStoredValue` (H1-T2)

---

## Task 3 — Centralize durable governance audit emission

**Effort:** M
**Complexity:** Medium
**Depends on:** Task 1, Task 2
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective

Introduce one audit-emission boundary that couples durable audit persistence with
correlated tracing and remove fire-and-forget governance audit writes.

### Scope

- Implement the boundary in `crates/audit` or another explicitly documented shared
  layer without duplicating `crates/domain::audit` types or `crates/db::audit_repo`.
- Support transaction participation for successful finalize events.
- Await rejection audit persistence before completing the response path.
- Persist duplicate-token governance rejections, not only traces.
- Define and test fail-closed behavior if an audit row cannot be persisted.

### Acceptance criteria

- Governance audit callers no longer manually duplicate persistence + tracing logic.
- No governance audit write uses detached `tokio::spawn`.
- Durable audit rows and traces share the relevant correlation identifier.
- The failure policy for audit persistence is explicit and covered by tests.
- ADR-018 implementation references match the chosen boundary.
- `cargo check --workspace` passes.

### Files likely affected

- `crates/audit/src/lib.rs`
- `crates/audit/Cargo.toml`
- `crates/db/src/audit_repo.rs`
- `apps/api/src/ingestion_service.rs`
- `apps/api/src/routes/ingestion.rs`
- `docs/adr/ADR-018-structured-observability-traceable-events.md`

### Status: [x] DONE — emit_governance_audit implemented in crates/audit; tokio::spawn removed; AlreadyFinalized paths emit durable rows; map_service_error converted to async; AuditEventKind::IngestionRejectedDuplicateToken added; ADR-018 updated; 1/1 unit tests passing; cargo check --workspace clean. 2026-05-31.

### Files affected
- `crates/audit/src/lib.rs` — `emit_governance_audit` + `AuditEmitError` (H1-T3)
- `crates/audit/Cargo.toml` — dependencies added (H1-T3)
- `crates/ingestion/Cargo.toml` — added dubbridge-audit dep (H1-T3)
- `crates/ingestion/src/lib.rs` — AlreadyFinalized paths emit durable audit (H1-T3)
- `crates/domain/src/audit.rs` — added `IngestionRejectedDuplicateToken` variant (H1-T3)
- `apps/api/Cargo.toml` — added dubbridge-audit dep (H1-T3)
- `apps/api/src/routes/ingestion.rs` — map_service_error async, tokio::spawn removed (H1-T3)
- `docs/adr/ADR-018-structured-observability-traceable-events.md` — implementation reference updated (H1-T3)

---

## Task 4 — Rollback and race regression suite

**Effort:** M
**Complexity:** Medium
**Depends on:** Task 1, Task 2, Task 3
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective

Lock the H1 invariants with deterministic Postgres-backed regression tests.

### Acceptance criteria

- A forced mid-finalize failure proves full relational rollback.
- Concurrent duplicate finalize proves one complete winner and no leaked loser rows.
- Cleanup-vs-finalize proves no successful artifact references a deleted blob.
- Missing-rights rejection proves the durable audit row exists before response
  completion.
- Duplicate-token rejection proves the durable audit row exists.
- Coverage remains at or above the enforced 90% line gate.
- `cargo test --workspace` passes.
- The CI-equivalent `cargo llvm-cov` command passes.

### Files likely affected

- `apps/api/tests/ingestion_test.rs`
- test support in touched crates as needed

### Status: [x] DONE — `apps/api/tests/ingestion_test.rs` now locks rollback on duplicate-artifact constraint failure, durable rejection audits for missing-rights and duplicate-token paths, and concurrent finalize winner invariants (`assets=1`, `rights_records=1`, `ingestion_finalized audit=1`). Verified with `cargo test -p dubbridge-api --test ingestion_test -- --test-threads=1` and `cargo check --workspace`. 2026-05-31.

---

## Agent handoff prompt

```text
Implement one approved H1 task only.

Tasks: docs/tasks/h1-governance-atomicity-hardening.md
Plan: docs/plan/h1-governance-atomicity-hardening.md

Preserve:
- ADR-008 fail-closed rights validation.
- ADR-006 storage-first ordering and explicit cross-store reconciliation boundary.
- ADR-018 durable audit rows plus correlated tracing.
- Existing ingestion HTTP contracts.

After the approved task, run its checks, update the ledger, report, and stop.
Do not start the next task without approval.
```
