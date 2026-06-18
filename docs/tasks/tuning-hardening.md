---
type: TaskList
title: "Tasks: T1 Tuning / Hardening"
status: closed
plan: docs/plan/tuning-hardening.md
---
# Tasks: T1 Tuning / Hardening

Governing plan: `docs/plan/tuning-hardening.md`

## Status Legend
- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked

---

## Task 1 — Persist pending upload-ingestion sessions

**Effort:** M
**Complexity:** Medium
**Depends on:** S1 Task 5
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Replace the in-memory pending-ingestion session state introduced in S1 T5 with a
durable mechanism that survives API restarts and preserves the existing
`/ingest -> /ingest/{token}/rights -> /ingest/{token}/finalize` contract.

### Context
- At task start, S1 T5 stored pending upload-ingestion sessions in process memory.
- A restart between upload and finalize loses the session and orphaned stored
  objects may remain.
- This does not block S1 acceptance, but it is a real reliability risk before
  production use.

### Acceptance criteria
- Pending ingestion state survives API restart.
- `POST /ingest/{token}/rights` and `POST /ingest/{token}/finalize` work after a
  restart if the upload already succeeded.
- The solution preserves fail-closed rights validation and authenticated uploader
  derivation from S0.
- Orphan-risk behavior is documented or reduced.

### Candidate approaches
- Add a durable `pending_ingestions` table in PostgreSQL.
- Reuse an existing durable store with equivalent guarantees.

### Files likely affected
- `apps/api/src/state.rs`
- `apps/api/src/routes/ingestion.rs`
- `crates/db/src/*`
- `infra/migrations/*`

### Status: [x] DONE — durable pending sessions implemented, restart-survival test passing. 2026-05-31.

### Evidence
- Added durable `pending_ingestions` persistence through:
  - `infra/migrations/0005_create_pending_ingestions.sql`
  - `crates/db/src/pending_ingestion_repo.rs`
- Removed in-memory pending-ingestion tracking from `apps/api/src/state.rs`.
- Updated `apps/api/src/routes/ingestion.rs` to:
  - persist pending upload state after `POST /ingest`
  - update rights state durably on `POST /ingest/{token}/rights`
  - load and consume pending state durably on `POST /ingest/{token}/finalize`
- Reduced orphan risk by attempting storage cleanup if the upload is written but
  pending-ingestion persistence fails.
- Added restart-survival coverage in `apps/api/tests/ingestion_test.rs` proving
  `rights` + `finalize` still succeed after rebuilding the app over the same DB.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo test -p dubbridge-api --test ingestion_test`
  - `~/.cargo/bin/cargo check --workspace`

---

## Task 2 — Expire and clean up abandoned pending ingestions

**Effort:** M
**Complexity:** Medium
**Depends on:** T1 Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Add TTL/expiration semantics plus cleanup for pending ingestions so uploads that
never reach `finalize` do not remain in PostgreSQL and storage forever.

### Context
- `T1-T1` made pending ingestions durable, which removed restart-loss risk.
- A new residual risk remains: abandoned sessions can now persist indefinitely.
- The system needs explicit lifecycle management, not just durable state.

### Acceptance criteria
- Pending ingestions have an explicit expiration policy.
- Expired sessions cannot be advanced through `rights` or `finalize`.
- A cleanup path exists for expired rows and their stored blobs.
- Cleanup behavior is either automated or clearly schedulable.
- Retry/idempotency behavior is documented for partial cleanup failure.

### Files likely affected
- `infra/migrations/*`
- `crates/db/src/*`
- `apps/api/src/routes/ingestion.rs`
- `apps/api/tests/*`
- possibly `apps/worker-runner` or a scheduled cleanup entrypoint

### Status: [x] DONE — expiration enforced on rights/finalize, cleanup module + background task added, 3 new integration tests passing. 2026-05-31.

### Evidence
- Migration `infra/migrations/0006_add_expires_at_to_pending_ingestions.sql`: added `expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '24 hours')`.
- `crates/db/src/pending_ingestion_repo.rs`: added `expires_at` field to struct and all queries; added `list_expired_pending_ingestions`; exported `PENDING_INGESTION_TTL_HOURS = 24`.
- `apps/api/src/routes/ingestion.rs`: `create_ingestion` sets `expires_at = now + 24h`; `submit_rights` loads record and returns 410 Gone if expired; `finalize_ingestion` returns 410 Gone if expired.
- `apps/api/src/cleanup.rs`: new module `cleanup_expired_ingestions(pool, storage)` — storage-first delete, retry-safe on partial failure.
- `apps/api/src/lib.rs`: exposes `pub mod cleanup`.
- `apps/api/src/main.rs`: spawns background tokio task running cleanup on a 1-hour interval.
- `apps/api/tests/ingestion_test.rs`: three new tests (`expired_session_is_rejected_on_rights`, `expired_session_is_rejected_on_finalize`, `cleanup_removes_expired_sessions`), all passing.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo fmt --all`
  - `DATABASE_URL=... ~/.cargo/bin/cargo test -p dubbridge-api --test ingestion_test` → 9/9 passed
  - `~/.cargo/bin/cargo check --workspace` → clean

---

## Task 3 — Raise measured coverage toward the 90% QA gate

**Effort:** M
**Complexity:** Medium
**Depends on:** T1-T1, T1-T2
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Reach the 90% line-coverage gate with an honest strategy: add unit tests for
code that has real logic and is currently uncovered, then adjust the CI gate scope
to exclude only binaries and structural init files that are not meaningfully
unit-testable.

### Context
- CI enforces `cargo llvm-cov --workspace --fail-under-lines 90`.
- Measured baseline after T1-T1 and T1-T2: **81.88% lines** (2229 total, 404
  uncovered). Gap: 8.12 pp.
- Analysis identified two categories of uncovered files:
  1. Real logic without tests — `ffprobe_command`, storage path helpers, config
     defaults. Silent failures in production if wrong.
  2. Structurally non-testable — binary `main.rs` entry points, tracing init,
     pool factory, trivial constants. Honest to exclude from the gate.
- Chosen strategy: add tests for category 1 first, then exclude category 2 from
  the gate.
- Unlocks T1-T4 (concurrency hardening).

### Approved strategy

#### Step 1 — Add unit tests for real logic

| File | Function | Why it matters |
|---|---|---|
| `crates/media/src/lib.rs` | `ffprobe_command` | Defines ffprobe argv contract; wrong flags = silent media probe failure |
| `crates/storage/src/lib.rs` | `asset_prefix`, `recording_prefix` | ADR-006 canonical storage key layout; wrong prefix = silent storage misses |
| `crates/storage/src/config.rs` | `StorageConfig::from_env` | Default `base_path` matters if env var absent in production |

#### Step 2 — Narrow CI gate scope (exclude non-testable files)

| File excluded | Justification |
|---|---|
| `apps/api/src/main.rs` | Binary — startup wiring, fails loudly at boot |
| `apps/cli/src/main.rs` | Skeleton binary — no logic |
| `apps/worker-runner/src/main.rs` | Skeleton binary — no logic |
| `crates/db/src/lib.rs` | Thin sqlx factory, requires live DB |
| `crates/jobs/src/lib.rs` | `default_queue()` returns a string literal |
| `crates/observability/src/lib.rs` | `init_tracing()` with `let _ = try_init()` |

### Files affected
- `crates/media/src/lib.rs`
- `crates/storage/src/lib.rs`
- `crates/storage/src/config.rs`
- `.github/workflows/ci.yml`
- `docs/tasks/tuning-hardening.md`
- `docs/plan/tuning-hardening.md`

### Acceptance criteria
- [ ] Unit tests added for `ffprobe_command`, `asset_prefix`, `recording_prefix`,
      `StorageConfig::from_env`.
- [ ] `cargo llvm-cov` with adjusted scope passes `--fail-under-lines 90`.
- [ ] Each `--exclude-files` entry in `ci.yml` has an inline comment with its
      justification.
- [ ] Any high-risk uncovered code that remains is explicitly tracked.
- [ ] `cargo test --workspace` passes (no DB required for the new unit tests).

### Status: [x] DONE — 91.08% line coverage achieved (gate: ≥ 90%), all tests passing. 2026-05-31.

### Evidence
- Added unit tests in:
  - `crates/media/src/lib.rs` — 3 tests for `ffprobe_command` argv contract
  - `crates/storage/src/lib.rs` — 4 tests for `asset_prefix` and `recording_prefix`
  - `crates/storage/src/config.rs` — 5 tests for `StorageConfig::from_env` (env var isolation via `temp-env`)
  - `crates/config/src/lib.rs` — 8 tests for `AppConfig::from_env` and `AuthSettings::from_env`
  - `crates/domain/src/rights.rs` — 3 tests for `SourceType::Display`, `LicenseType::Display`, `RightsRecord::new`
  - `crates/domain/src/asset.rs` — 3 tests for `IngestionStatus::Display`, `AssetId::Display`, `Asset::new_pending`
- Added `temp-env = "0.3"` dev-dependency to `crates/storage/Cargo.toml` and `crates/config/Cargo.toml` for thread-safe env var isolation.
- Narrowed CI gate in `.github/workflows/ci.yml` using `--ignore-filename-regex` to exclude 6 non-testable files (binary entry points, sqlx factory, string literal, tracing init). Each exclusion is justified in inline comments.
- Post-consolidation fix: aligned `.githooks/pre-push` with the same
  `--ignore-filename-regex` so the local and CI coverage scopes match.
- Final measurement: **91.08% lines** (1783 total, 159 uncovered) with exclusions applied.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo test -p dubbridge-config -p dubbridge-domain -p dubbridge-media -p dubbridge-storage` → 36/36 passed
  - `DATABASE_URL=... ~/.cargo/bin/cargo llvm-cov --workspace --summary-only --fail-under-lines 90 --ignore-filename-regex '...'` → exit 0, 91.08%

---

## Task 4 — Concurrency and race-condition hardening for ingestion

**Effort:** M
**Complexity:** Medium
**Depends on:** S1, T1 Task 1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Verify and harden ingestion behavior under overlapping requests such as duplicate
finalize, concurrent rights/finalize, and cleanup-vs-finalize races.

### Acceptance criteria
- Concurrency-sensitive ingestion scenarios are covered by deterministic tests.
- Observed race windows are either closed or documented with explicit invariants.
- Duplicate or overlapping requests remain fail-closed and idempotent where intended.

### Status: [x] DONE — 2 deterministic concurrency tests added, cleanup-vs-finalize race documented with invariant. 2026-05-31.

### Evidence
- Added to `apps/api/tests/ingestion_test.rs`:
  - `concurrent_duplicate_finalize_one_wins`: proves exactly one finalize wins (201) and the other is rejected (409 or 404 depending on scheduler); artifact count = 1. Documents that the DB unique constraint on `artifact_records.ingest_token` is the real guard.
  - `concurrent_rights_and_finalize_is_consistent`: proves no silent data corruption when rights and finalize race; finalize gets 201 (rights loaded) or 422 (rights not visible yet); artifact count ≤ 1.
  - Helper functions `make_finalize_request` and `make_rights_request` added for direct `oneshot` composition.
- `apps/api/src/cleanup.rs`: race window documented as explicit invariant comment (T1-T4 block). Explains the boundary condition and the risk (orphaned blob reference on artifact record). Full finalize/cleanup coordination moved to blocking gate H1 before S3 expansion.
- Verified: `cargo test -p dubbridge-api --test ingestion_test -- --test-threads=1` → 11/11 passed.
- Verified: `cargo check --workspace` → clean.
- Integration tests require `--test-threads=1` with a live DB (all tests share `migrate_and_reset` which truncates tables; documented in test comment).

---

## Task 5 — Reconcile `crates/audit` with the intended architecture

**Effort:** M
**Complexity:** Medium
**Depends on:** S1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Align `crates/audit` with the intended boundary described in plans and ADR-linked
architecture instead of leaving it as a lightweight placeholder.

### Acceptance criteria
- The crate either becomes the intended audit wrapper boundary or the plans are
  corrected to match the implemented architecture.
- Audit writes and tracing responsibilities are not ambiguously split.

### Status: [x] DONE — stub replaced with architecture comment, stale dependencies removed, cargo check clean. 2026-05-31.

### Evidence
- `crates/audit/src/lib.rs`: replaced 8-line stub `AuditEvent` with a module-level
  comment that documents the real architecture (domain types in
  `crates/domain/src/audit.rs`, DB writes in `crates/db/src/audit_repo.rs`) and
  the S2+ intent for this crate.
- `crates/audit/Cargo.toml`: removed unused `serde` and `time` workspace
  dependencies; replaced with a comment explaining why the block is empty.
- No other crate imports `dubbridge-audit` (confirmed by grep across workspace).
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo check --workspace` → clean (0.69 s).

---

## Task 6 — Operational safeguards for upload ingestion

**Effort:** M
**Complexity:** Medium
**Depends on:** S1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Add operational guardrails around upload ingestion such as size ceilings,
multipart/resource safety, and abuse-resistant behavior.

### Acceptance criteria
- Upload-size limits are explicit and enforced.
- Resource usage is bounded more deliberately than the MVP path.
- Failure behavior is clear for oversized or malformed uploads.

### Status: [x] DONE — MAX_UPLOAD_BYTES constant defined, DefaultBodyLimit layer applied, 413 test passing (12/12). 2026-05-31.

### Evidence
- `apps/api/src/routes/ingestion.rs`: added `pub const MAX_UPLOAD_BYTES: usize = 500 * 1024 * 1024` with justification comment; added `DefaultBodyLimit` import; applied `.layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))` on `mutation_routes` inside `router()` before auth layers.
- `apps/api/tests/ingestion_test.rs`: added `upload_too_large_is_rejected` — builds a multipart body of `MAX_UPLOAD_BYTES + 1` bytes inline and asserts 413 Payload Too Large.
- Limit applies only to `mutation_routes` (POST /ingest, /rights, /finalize); JSON-only read routes are unaffected.
- Verification run on 2026-05-31:
  - `~/.cargo/bin/cargo check --workspace` → clean (1.72 s).
  - `~/.cargo/bin/cargo test -p dubbridge-api --test ingestion_test -- --test-threads=1` → 12/12 passed.
