---
type: TaskList
title: "Foundational docs vs. P2P/S-230 consistency fixes"
status: in-progress
---

# Foundational docs vs. P2P/S-230 consistency fixes

## Purpose

Remediate the critical findings from
`docs/audit/foundational-docs-p2p-consistency-2026-09-06.md` (F1, F2, F4, F5).
The audit was read-only; this ledger is the single task that carries out the
owner-approved fixes as ordered subtasks. Minor findings (F3, F6, F7, F8, F9)
stay deferred per the audit's triage table and are out of scope here.

Owner decision on F5 (2026-09-06): permit a new stack. Node.js/TypeScript is
now an allowed backend stack alongside Rust and the existing Python ML-worker
exception, scoped to the Availability Node (ADR-044). `docs/architecture.md`
and `CLAUDE.md` are amended accordingly in T2.

## Dependencies and sequencing

T1, T2, T3 touch disjoint files and have no ordering dependency on each other.
T4 (status sync) runs last because it depends on T1-T3's outcomes.

## Task list

### T1 — Fix migration-count assertion drift (F1)

- **RRI:** 3 -> Low. Effort: S.
- **Type:** development (test fix), one-line assertion.
- **Objective:** stop `apps/cli/tests/migrate_test.rs` from going red in CI
  the moment a DB is available, and make the fix resilient to the next
  migration (ADR-044 P2.T2e is already frozen to add `0033_*`).
- **Allowed paths:** `apps/cli/tests/migrate_test.rs`.
- **Root cause:** hardcoded `assert_eq!(count, 31, ...)`; disk now has 32
  migrations after P2.T1b's `0032_create_p2p_publications_and_outbox.sql`.
- **Acceptance criteria (HP/EC):**
  - HP-1: with a real migrated local database, the test asserts against
    `ls infra/migrations/*.sql | wc -l` (or an equivalent derived count) and
    passes with the current migration set.
  - EC-1: a future migration file added under `infra/migrations/` does not
    require another hand-edit of this test.
- **Evidence to emit:** local `cargo test -p dubbridge-cli` run against a
  migrated DB (this repo's sandbox has no DB access — run via
  `docker compose -f infra/local/docker-compose.yml up -d postgres` first, or
  record as a documented gap if unavailable).
- **Reviewer:** Muse Glimmer -> Gemma -> D14 (RRI 0-25 chain).
- **Evidence recorded:** `cargo check -p dubbridge-cli --tests` (clean),
  `cargo fmt -- --check apps/cli/tests/migrate_test.rs` (clean),
  `cargo clippy -p dubbridge-cli --tests -- -D warnings` (clean, 0 warnings).
  No live-DB run was possible in this session's sandbox (Colima/Docker daemon
  unreachable) — **documented gap**: run
  `docker compose -f infra/local/docker-compose.yml up -d postgres` +
  `cargo test -p dubbridge-cli` in an environment with Docker access to
  confirm HP-1/HP-2 against a real migrated database before the next CI run
  is treated as full confirmation of this fix.
- **Status:** [x] Done (code + static verification; live-DB run pending, see gap above)

### T2 — Amend the language principle for the Availability Node stack (F5 + F2 partial)

- **RRI:** 13 -> Low. Effort: S.
- **Type:** docs-only (policy/architecture amendment). Exempt from Gemma
  review per workflow guide (docs-only).
- **Objective:** reconcile the stated Rust+Python-only principle with the
  Node.js/TypeScript Availability Node already frozen by ADR-044 P2.T0/C0 and
  landed under `apps/availability-node/` (P2.T1-T3 scope).
- **Allowed paths:** `docs/architecture.md`, `CLAUDE.md`,
  `docs/node-exceptions.md` (new file, mirrors `docs/python-exceptions.md`).
- **Acceptance criteria:**
  - HP-1: `docs/architecture.md` § Core principles states the Node.js/TypeScript
    exception (scope: Availability Node only, ciphertext-only, no DB/business-
    authorization/plaintext-key/backend-signing authority per ADR-044) with a
    citation to `docs/node-exceptions.md` and ADR-044, mirroring the existing
    Python-exception sentence structure.
  - HP-2: `CLAUDE.md` architecture summary reflects the same exception (does
    not need the full boundary prose — one line + pointer is enough since
    CLAUDE.md is a summary per its own precedence rule).
  - HP-3: `docs/node-exceptions.md` exists with the same shape as
    `docs/python-exceptions.md` (allowed categories, constraints, operational
    model) scoped to the Availability Node only.
  - EC-1: the amendment does not relax the principle for any other
    `apps/*` or `crates/*` surface — Rust remains the default and the
    exception is narrowly scoped to the one frozen ADR-044 service.
- **Status:** [x] Done

### T3 — Correct `design-inputs.md` P2 presentability claim (F4)

- **RRI:** 3 -> Low. Effort: S.
- **Type:** docs-only. Exempt from Gemma review (docs-only).
- **Objective:** stop `docs/plan/mvp0-p2p-design-inputs.md` from telling the
  next P3-P7 planner that ADR-044 is Proposed and P2 is unpresentable, when
  ADR-044 is Accepted (D1-D4 all resolved) and P2.T0/T1/C0 are PASS/Done.
- **Allowed paths:** `docs/plan/mvp0-p2p-design-inputs.md`.
- **Acceptance criteria:**
  - HP-1: line ~328-329 no longer states ADR-044 is "Proposed — not accepted"
    or that "P2 remains unpresentable"; reflects `Accepted`, D1-D4 resolved,
    and current P2.T0/T1/C0 status.
  - HP-2: line ~375's "Proposed; blocks P2" is corrected the same way.
  - HP-3: the "Open decisions blocking P2" section (or equivalent) removes
    D1/D2 as open (both resolved: `O3 parallel`, `K1`) and cites the correct
    current guardrail number (8, not the stale 9) per
    `docs/plan/mvp0-p2p-first.md:43`.
  - EC-1: any decision genuinely still open (if any survive after D1-D4) stays
    listed as open — this is a correction, not a blanket "everything resolved"
    rewrite.
- **Status:** [x] Done

### T4 — Status artifact sync

- **RRI:** n/a (status-sync, docs-only).
- **Objective:** per `AGENT_WORKFLOW_GUIDE.md § Sync status artifacts before
  reporting completion`, reflect T1-T3 closure in the audit report and roadmap
  where they cite these findings.
- **Allowed paths:** `docs/audit/foundational-docs-p2p-consistency-2026-09-06.md`.
- **Acceptance criteria:** the audit's remediation table marks F1/F2/F4/F5 as
  resolved with a pointer to this ledger; F3/F6/F7/F8/F9 remain open/deferred.
- **Status:** [x] Done

## Related

- `docs/audit/foundational-docs-p2p-consistency-2026-09-06.md` — source audit
- `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`
- `docs/python-exceptions.md` — sibling exception-boundary doc for T2's shape
- `docs/plan/mvp0-p2p-first.md`, `docs/plan/mvp0-p2p-design-inputs.md`
