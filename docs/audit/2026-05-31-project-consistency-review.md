# Project Consistency Review — 2026-05-31

Scope: full-repository review requested before approving the Stream Recording
Ingest plan. Goal: verify the new plan is well integrated, and fix or record any
inconsistencies found across documentation, configuration, and code — even those
unrelated to the recording objective.

Constraint honored: the Rust toolchain (`rustc`/`cargo`) is **not installed** in
this environment, so no Rust source was edited (changes could not be compiled or
tested). Code-level issues are recorded here and folded into the recording plan as
explicit tasks instead of being patched blind.

## Findings

| ID | Area | Severity | Status |
|----|------|----------|--------|
| F1 | Duplicate/conflicting `AuditEvent` types | High | **Closed by T1/T5**; H1 owns centralized emission |
| F2 | `audit_events.ingest_token NOT NULL` blocks recording lifecycle events | High | Recorded → folded into plan (T1/T2) |
| F3 | Dead branch in `find_original_by_ingest_token` (artifact kind) | Medium | Recorded → folded into plan (T1/T5), surfaced in ADR-021 |
| F4 | `.gitignore` ignores `Cargo.lock` in a binary workspace | Medium | **Fixed** |
| F5 | `AGENTS.md` references 3 non-existent governance docs | Medium | **Fixed** (scaffolds created) |
| F6 | `README.md` stale environment-specific "Validation note" | Low | **Fixed** |
| F7 | Toolchain pin drift (`docker-compose` rust:1.88 vs `stable`) | Low | **Closed by P0 Task 6** |
| F8 | S1 plan vs code drift around `crates/audit` | Medium | **Closed by T1/T5** |

---

### F1 — Two conflicting `AuditEvent` definitions (High)

- `crates/domain/src/audit.rs` defines the real `AuditEvent`
  (`id`, `asset_id: Option<AssetId>`, `event_kind: AuditEventKind`, `ingest_token`,
  `detail`, `happened_at`), persisted by `crates/db/src/audit_repo.rs`.
- `crates/audit/src/lib.rs` defines a **different, unused** `AuditEvent`
  (`event_type: String`, `happened_at`). It has no dependency on `dubbridge-domain`
  or `dubbridge-db`, and does not implement the `AuditLogger` that the S1 plan
  promised (`crates/audit/src/lib.rs — AuditLogger wrapping audit_repo + tracing`).

**Impact on recording plan:** the first draft said recording would "reuse
`crates/audit`". That is wrong — recording audit must use `domain::audit::AuditEvent`
+ `db::audit_repo`, identical to S1. Plan corrected.

**Resolution:** T1-T5 removed the placeholder type and kept `crates/audit` as a
reserved empty namespace. H1 now owns the shared durable-emission boundary required
before recording lifecycle events are added.

### F2 — `audit_events.ingest_token` is `NOT NULL` (High)

`infra/migrations/0004_create_audit_events.sql` declares `ingest_token UUID NOT NULL`
and `AuditEventKind` only has ingestion variants. Recording lifecycle events
(session created, rejected, capturing, failed) occur **before** any ingestion token
exists, so they cannot be persisted under the current schema/type.

**Resolution (folded into plan):**
- T1 extends `AuditEventKind` with recording variants and makes
  `AuditEvent.ingest_token: Option<Uuid>` plus a new `recording_session_id: Option<Uuid>`.
- S3 T2 adds the next available `alter_audit_events_for_recording` migration after
  H1 migrations: relax
  `ingest_token` to nullable and add nullable `recording_session_id` FK.
- This matches ADR-018, which already anticipates `recording_session_id` as a
  correlation identifier.

### F3 — Dead branch in artifact-kind mapping (Medium)

`crates/db/src/artifact_repo.rs::find_original_by_ingest_token` maps the stored
`kind` with `if r.kind == "original_media" { OriginalMedia } else { OriginalMedia }`
— both branches are identical. Once `ArtifactKind::RecordedStreamMedia` exists, this
silently mislabels recorded artifacts as `OriginalMedia`, corrupting lineage.

**Resolution:** plan T1 introduces a single `parse_artifact_kind(&str)` helper used
by the repository; T5 verifies recorded artifacts round-trip with the correct kind.
Surfaced as a consequence in ADR-021.

### F4 — `.gitignore` ignores `Cargo.lock` (Medium) — FIXED

This is a workspace with binaries (`apps/api`, `apps/cli`, `apps/worker-runner`).
Cargo guidance is to **commit** `Cargo.lock` for applications to get reproducible
builds; the file already exists on disk (~80 KB) but `git check-ignore` confirmed it
was ignored, so the first commit would have dropped it. Removed the `Cargo.lock`
entry from `.gitignore`.

### F5 — `AGENTS.md` references missing governance docs (Medium) — FIXED

`AGENTS.md` says it complements `README_AGENT_ORDER.md`,
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, and `docs/policies/HITL_AUTONOMY_POLICY.md`
— none existed. Created honest scaffolds that **consolidate only the rules already
written** in `AGENTS.md` and the project/global `CLAUDE.md` (no invented policy),
each marked as a living document. This removes the dangling references.

### F6 — `README.md` stale "Validation note" (Low) — FIXED

The README baked in an environment-specific note ("On May 25, 2026, both `rustc` and
`cargo` were unavailable on the host shell"). Replaced with a durable statement: CI
runs `cargo check --workspace`; local validation requires installing the toolchain
via `rustup`.

### F7 — Toolchain pin drift (Low) — CLOSED 2026-06-03

The local Compose file was pinning `image: rust:1.88` for the `api`/`worker-runner`
services, while `rust-toolchain.toml` and CI used `stable`. That drift was closed on
2026-06-03 by P0 Task 6; the local Compose file now tracks `rust:stable`.

### F8 — S1 plan vs code drift around `crates/audit` (Medium) — CLOSED

`docs/plan/s1-asset-ingestion-rights-ledger.md` lists `crates/audit` as hosting an
`AuditLogger` with deps on `domain` and `db`. The implemented `crates/audit` does
neither (see F1). ADR-018's "Implemented by" line was corrected to point at the real
implementation (`domain::audit` + `db::audit_repo`). T1-T5 later removed the
placeholder type and kept `crates/audit` as a reserved empty namespace. H1 owns the
future shared durable-emission boundary.

## Actions taken in this review

- Fixed: `.gitignore` (F4), `README.md` (F6).
- Created: `README_AGENT_ORDER.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `docs/policies/HITL_AUTONOMY_POLICY.md` (F5).
- Corrected: `docs/adr/ADR-018-...md` "Implemented by" line (F8).
- Updated: recording `docs/plan/stream-recording-ingest.md` and
  `docs/tasks/stream-recording-ingest.md` to integrate F1, F2, F3.

## Consolidated follow-ups (now tracked in the recording plan)

- Reconcile or remove `crates/audit` (F1/F8) → **closed by T1-T5**. H1 owns the
  centralized durable-emission boundary.
- Align docker-compose Rust image with the toolchain policy (F7) →
  `docs/tasks/stream-recording-ingest.md` **Task 9** (low-priority housekeeping).

## Still open

- Backfill the remaining open ADR numbers if/when their decisions are identified.

## Post-review consolidation update — 2026-05-31

Later work closed F1/F8 through T1-T5 by removing the duplicate
`crates/audit::AuditEvent` placeholder. The reserved namespace remains empty; H1 now
owns centralized durable-audit emission semantics.

Migrations `0005` and `0006` were subsequently allocated to pending-ingestion
hardening. Future recording migrations must use the next free sequence after H1
migrations. The broader roadmap/ADR review is recorded in
`docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md`.
