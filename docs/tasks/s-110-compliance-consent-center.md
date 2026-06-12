# Tasks: S-110 — Compliance & Consent Center

**Plan:** `docs/plan/s-110-compliance-consent-center.md`
**Roadmap phase:** `S-110` (depends on `S-100`). Closes X11.
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-008, ADR-018, ADR-025, ADR-023, ADR-024, ADR-006.

> **Namespace.** This phase uses the **`S-110-T`** prefix (`S-110-T0`–`S-110-T6`). Always fully
> qualify cross-slice references (`S-110-T2`, `S-100-T1`), never bare `T2`.

> **RRI provenance.** Every RRI below was computed with `python3 scripts/rri.py`
> (platform `dubbridge`) at planning time, not by hand. Final RRI is recomputed at
> presentation. `S-110-T1` was decomposed 2026-06-12 into T1a/T1b/T1c to lower
> complexity; `S-110-T2` lands in **Complex (56–70)** and requires a reviewed plan
> before implementation — this ledger + the plan provide it.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
S-110-T0 (BDD) ─▶ S-110-T0b (ADR X-S-110-1) ─▶ S-110-T1a (migration SQL)
                                                         │
                                              ┌──────────┴──────────┐
                                    S-110-T1b (domain entity)   S-110-T1c (DB repo)
                                              └──────────┬──────────┘
                                                         ▼
                                              S-110-T2 (consent gate + audit, X11)
                                                         ▼
                                              S-110-T3 (compliance read API)
                                              ┌──────────┴──────────┐
                                    S-110-T4 (web dashboard)   S-110-T5 (mobile consent)
                                              └──────────┬──────────┘
                                                         ▼
                                              S-110-T6 (E2E + docs sync)
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| S-110-T0 | BDD `.feature` specs + mapping | — | 11 | Low | S | ✅ done 2026-06-12 |
| S-110-T0b | ADR authoring: voice-consent ledger + TTS precondition (X24 → X-S-110-1) | S-110-T0 | 18 | Low | S | ✅ done 2026-06-12 |
| S-110-T1a | Migration SQL: `0013_create_voice_consents.sql` + RULES + CHECK constraints | S-110-T0b | 51 | Med-high | M |
| S-110-T1b | Domain entity: `consent.rs` — types, status derivation, grant validation | S-110-T1a | 27 | Moderate | M |
| S-110-T1c | DB repo: `consent_repo.rs` — append, latest_status, list | S-110-T1a | 31 | Moderate | M |
| S-110-T2 | Consent gate + TTS precondition + audit (X11) | S-110-T1b, S-110-T1c | 66 | Complex | L |
| S-110-T3 | Compliance read API (audit/rights viewer) | S-110-T2 | 44 | Med-high | L |
| S-110-T4 | Web compliance dashboard | S-110-T3 | 30 | Moderate | M |
| S-110-T5 | Mobile consent + compliance surfaces | S-110-T3 | 31 | Moderate | M |
| S-110-T6 | E2E fixtures + docs/roadmap sync | S-110-T4, S-110-T5 | 24 | Low | S |

## Model resolution (capability → current vendor model)

| Band | Codex | Claude Code | Thinking |
|---|---|---|---|
| Low (0–25) | `GPT-5.2-Codex` | `Claude Haiku 4.5` | Off |
| Moderate (26–40) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` | Off |
| Med-high (41–55) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` (escalate to `Claude Opus 4.8` if it stalls) | On |
| Complex (56–70) | `GPT-5.2-Codex` | `Claude Opus 4.8` | On |

---

## S-110-T0 — BDD `.feature` specs + BDD⇄web⇄mobile⇄unit mapping

- **Status:** [x] Done — 2026-06-12
- **Type:** Planning / docs (BDD authoring) · **Effort:** S
- **RRI:** 11 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** — (BDD-first)
- **Objective:** Author the Gherkin specs defining acceptance for the compliance/consent
  slice and the mapping convention (scenario ID ⇄ web/mobile flow ⇄ `HP-#`/`EC-#`).
- **Inputs:** plan §D1–§D4; S-010 `audit_events`/`rights_records`; X11; ADR-008/018/025.
- **Outputs:** `docs/bdd/p6-compliance.feature`; mapping rows appended to `docs/bdd/README.md`.
- **Acceptance criteria:**
  - Each scenario has a stable ID and maps to one web/mobile flow and ≥1 `HP-#`/`EC-#`.
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
  Feature: Compliance and consent center

    Scenario: SC-AUDIT-1 View an asset's audit timeline
      Given I own an asset with recorded governance events
      When I open its compliance view
      Then I see its audit events in chronological order

    Scenario: SC-AUDIT-2 Audit view is ownership-scoped
      Given an asset I do not own
      When I request its audit timeline
      Then I am denied and see no governance data

    Scenario: SC-RIGHTS-1 View the rights ledger for an asset
      Given I own an asset with a rights record
      When I open its rights view
      Then I see its rights ledger entries

    Scenario: SC-CONSENT-1 Grant voice consent
      Given I own an asset
      When I grant voice-cloning consent with an evidence reference
      Then the consent is recorded as active

    Scenario: SC-CONSENT-2 Revoke voice consent
      Given an active voice consent exists
      When I revoke it
      Then the consent becomes inactive and the history is preserved

    Scenario: SC-CONSENT-3 Synthesis blocked without consent
      Given no active voice consent exists for an asset
      When a TTS/voice-cloning derivative is requested
      Then it is refused with a clear consent-required error
  ```

- **Handoff prompt:**
  > S-110-T0 — author BDD specs. Docs: this ledger + plan §D1–§D4. Create
  > `docs/bdd/p6-compliance.feature` (SC-AUDIT-1/2, SC-RIGHTS-1, SC-CONSENT-1/2/3) and append
  > mapping rows to `docs/bdd/README.md`. AC: stable IDs mapped to web/mobile + HP/EC, qa-docs
  > green. Stop after docs; do not start S-110-T0b.

---

## S-110-T0b — ADR authoring: voice-consent ledger + TTS precondition (X24 → X-S-110-1)

- **Status:** [x] Done — 2026-06-12
- **Type:** Architecture decision · **Effort:** S
- **RRI:** 18 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-110-T0, S-100 (org/asset ownership model)
- **Blocks:** S-110-T1, S-110-T2 — **neither may start until this ADR is merged**
- **Objective:** Author and merge the ADR that defines the voice-consent ledger and the
  fail-closed TTS precondition. Closes X24 / X-S-110-1 and closes **X11** at the
  architecture-contract level before S-150 (TTS/dubbing) is built.
- **Inputs:**
  - ADR-008 — fail-closed precondition posture
  - ADR-018 — durable audit obligation
  - ADR-025 — owner credentials/evidence stored by reference, redacted from logs
  - `infra/migrations/0002` (rights_records shape), `0007` (append-only governance)
  - `docs/plan/s-110-compliance-consent-center.md` §D1–§D3
  - X11 obligation in roadmap
- **Outputs:**
  - `docs/adr/ADR-NNN-voice-consent-ledger.md` — decision record covering:
    - `voice_consents` is append-only; current status = latest row (grant / revoke)
    - Consent scope: asset-level; granted by the asset owner or authorized delegate
    - Evidence stored as an opaque reference (URI/ID), never inline (ADR-025 spirit)
    - Evidence bytes/secrets are not stored in the DB and are redacted from logs
    - TTS precondition: `consent_gate.rs` checks latest consent status fail-closed
      before any TTS derivative is created; absent or revoked consent → hard reject
    - Every consent mutation emits an `audit_events` row (ADR-018)
    - Revocation does not delete history; prior consent rows are immutable
    - Open follow-up: evidence-store tie to X20 (S-090 owner-credential secret-store)
  - ADR index entry added to `docs/adr/README.md`
- **Acceptance criteria:**
  - ADR file present in `docs/adr/` with a real sequential number.
  - `docs/adr/README.md` index updated.
  - ADR text covers: append-only ledger, evidence-by-reference, TTS fail-closed gate,
    revocation immutability, audit obligation, and evidence-store open follow-up.
  - `make qa-docs` passes.
- **Handoff prompt:**
  > S-110-T0b — author ADR for voice-consent ledger + TTS precondition (X24, closes X11).
  > Inputs: ADR-008, ADR-018, ADR-025, migrations 0002/0007, plan §D1–§D3. Create
  > `docs/adr/ADR-NNN-voice-consent-ledger.md` (append-only ledger, evidence by ref,
  > TTS gate fail-closed, revocation immutability, audit obligation, evidence-store
  > follow-up) and update `docs/adr/README.md` index. AC: real ADR number, index
  > updated, qa-docs green. Stop after docs; do not start S-110-T1.

---

## S-110-T1a — Migration SQL: `0013_create_voice_consents.sql`

- **Status:** [ ] Not started
- **Type:** Development (SQL) · **Effort:** M
- **RRI:** 51 → band **Med-high (41–55)** → **Plan + explicit AC before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking On
- **Depends on:** S-110-T0b
- **Objective:** Create the `voice_consents` table with append-only RULES and CHECK constraints
  on `status` and `scope`. Establishes the schema foundation for T1b (domain) and T1c (repo).
- **Inputs:** `infra/migrations/0002_create_rights_records.sql` (table shape), `0007_harden_governance_invariants.sql` (RULES + CHECK pattern).
- **Outputs:** `infra/migrations/0013_create_voice_consents.sql`.
- **Acceptance criteria:**
  - Table created with columns: `id UUID PK`, `asset_id UUID FK→assets`, `scope TEXT NOT NULL`, `status TEXT NOT NULL`, `evidence_ref TEXT`, `granted_by UUID NOT NULL`, `happened_at TIMESTAMPTZ NOT NULL`.
  - `status` CHECK constraint: `grant`, `revoke` only.
  - `scope` CHECK constraint: `voice_clone`, `tts_synthesis` only.
  - RULES block UPDATE and DELETE (append-only, ADR-028).
  - Migration applies cleanly against a fresh DB; FK to `assets` enforced.
- **RRI variable table:**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 3 → 0 | High |
  | F | 1 | 1 file | High |
  | D | 4 | anchor: `infra/migrations` floor 4 | High |
  | T | 2 | migration tests exist in CI | High |
  | A | 0 | criteria present | High |
  | K | 4 | anchor: `infra/migrations` floor 4 | High |
  | P | 5 | schema/data impact floor 5 | High |
  | X | 1 | single migration file | High |

  **Base 41 · penalty auth_security +10 · Final 51 → Med-high.**

- **Handoff prompt:**
  > S-110-T1a — migration SQL for voice_consents. Docs: ledger §T1a, ADR-028, migrations 0002/0007.
  > Create `infra/migrations/0013_create_voice_consents.sql`: table + FK→assets + CHECK(status IN
  > ('grant','revoke')) + CHECK(scope IN ('voice_clone','tts_synthesis')) + RULES no-update/no-delete.
  > AC: migration applies cleanly. Stop; do not touch Rust files.

---

## S-110-T1b — Domain entity: `crates/domain/src/consent.rs`

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** M
- **RRI:** 27 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-110-T1a
- **Objective:** Define `ConsentScope`, `ConsentStatus`, `ConsentRow`, grant/revoke constructors,
  and `derive_status` / `is_active` helpers. No DB access — pure domain logic.
- **Inputs:** `crates/domain/src/rights.rs` (pattern), `crates/domain/src/lib.rs` (module registry), ADR-028.
- **Outputs:** `crates/domain/src/consent.rs`; `pub mod consent;` in `lib.rs`.
- **Acceptance criteria:**
  - `ConsentScope` and `ConsentStatus` decode fail-closed: unknown string → `Err`.
  - `new_grant` rejects empty `evidence_ref` → `Err(ConsentError::MissingEvidenceRef)`.
  - `derive_status` returns `Some(Grant)` when latest row is grant; `Some(Revoke)` when revoked; `None` when no rows.
  - `is_active` true iff `derive_status` = `Some(Grant)`.
  - Unit tests cover HP-1, HP-2, EC-1, EC-2; ≥90% coverage; `cargo test -p dubbridge-domain` green.
- **RRI variable table:**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 6 → 1 | High |
  | F | 1 | 1 file + lib.rs | High |
  | D | 2 | pure domain, no migrations | High |
  | T | 2 | domain area has tests | High |
  | A | 0 | criteria present | High |
  | K | 2 | no external coupling | High |
  | P | 1 | domain-internal only | High |
  | X | 2 | entity + tests | High |

  **Base 27 · no penalties · Final 27 → Moderate.**

- **Happy paths / Edge cases:**
  - `HP-1`: `new_grant` + `derive_status` → `Some(Grant)`. (SC-CONSENT-1)
  - `HP-2`: grant + revoke rows → `derive_status` → `Some(Revoke)`; both rows preserved. (SC-CONSENT-2)
  - `EC-1`: empty `evidence_ref` on grant → `Err`. (ADR-028)
  - `EC-2`: unknown scope string decode → `Err`.
- **Handoff prompt:**
  > S-110-T1b — domain entity consent.rs. Docs: ledger §T1b, ADR-028, rights.rs pattern.
  > Add `crates/domain/src/consent.rs` (ConsentScope, ConsentStatus, ConsentRow, new_grant,
  > new_revoke, derive_status, is_active) + `pub mod consent` in lib.rs. AC: fail-closed decode,
  > missing evidence_ref rejected, HP-1/HP-2/EC-1/EC-2 unit tests, ≥90% cov. Stop; do not touch db crate.

---

## S-110-T1c — DB repo: `crates/db/src/consent_repo.rs`

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** M
- **RRI:** 31 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-110-T1a (schema), S-110-T1b (types) — can run parallel to T1b once T1a is done
- **Objective:** Implement the three consent repo functions over the `voice_consents` table.
- **Inputs:** `crates/db/src/rights_repo.rs` (append pattern), `crates/domain/src/consent.rs` (T1b types), `crates/db/src/lib.rs` (module registry).
- **Outputs:** `crates/db/src/consent_repo.rs`; `pub mod consent_repo;` in `lib.rs`.
- **Acceptance criteria:**
  - `append_consent(pool, row)` — INSERT only; no upsert.
  - `latest_consent_status(pool, asset_id, scope)` — returns `Option<ConsentStatus>` from latest row by `happened_at DESC LIMIT 1`.
  - `list_consents_for_asset(pool, asset_id)` — returns all rows ordered `happened_at ASC`.
  - Integration tests (sqlx test-db) cover HP-1, HP-2; ≥90% coverage; `cargo test -p dubbridge-db` green.
- **RRI variable table:**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 5 → 0 | High |
  | F | 1 | 1 file + lib.rs | High |
  | D | 3 | DB layer, references migrations | High |
  | T | 2 | db area has integration tests | High |
  | A | 0 | criteria present | High |
  | K | 3 | sqlx + domain types coupling | High |
  | P | 2 | read/write single new table | High |
  | X | 2 | repo + tests | High |

  **Base 31 · no penalties · Final 31 → Moderate.**

- **Happy paths / Edge cases:**
  - `HP-1`: append grant row → `latest_consent_status` returns `Grant`. (SC-CONSENT-1)
  - `HP-2`: append grant + revoke → `latest_consent_status` returns `Revoke`; `list` returns both rows. (SC-CONSENT-2)
- **Handoff prompt:**
  > S-110-T1c — DB repo consent_repo.rs. Docs: ledger §T1c, ADR-028, rights_repo.rs pattern.
  > Add `crates/db/src/consent_repo.rs` (append_consent, latest_consent_status, list_consents_for_asset)
  > + `pub mod consent_repo` in lib.rs. AC: append-only INSERT, latest-row query, HP-1/HP-2 integration
  > tests, ≥90% cov. Stop; do not start S-110-T2.

---

## S-110-T2 — Consent ledger + TTS precondition + audit (closes X11)

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 66 → band **Complex (56–70)** → **Plan first; human reviews the plan; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-110-T1b, S-110-T1c
- **Objective:** Implement the consent grant/revoke transitions and the **fail-closed TTS
  precondition** (X11): no TTS/voice-cloning derivative may proceed without an active,
  unrevoked consent for the target scope. Emit durable audit on grant/revoke and on every
  precondition check. (Plan §D2.)
- **Inputs:** `consent_repo` (S-110-T1), `crates/audit` emission, ADR-008 (rights gate as the
  template), roadmap X11.
- **Outputs:**
  - `apps/api/src/services/consent_gate.rs` (`require_active_consent(asset, scope)` + audit).
  - Audit rows on grant, revoke, and every precondition check (allowed and refused).
  - Tests: active consent → allowed; missing/revoked → refused + audited.
- **Acceptance criteria:**
  - A synthesis request without active consent is refused and audited. (SC-CONSENT-3)
  - The gate is a reusable service S-150 (TTS/dubbing) calls directly when built.
  - ≥90% coverage; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 16 → 2 | High |
  | F | 2 | 4 files | High |
  | D | 4 | anchor: `crates/audit` (ADR-008, ADR-018) floor 4 | High |
  | T | 2 | audit/domain area has tests | High |
  | A | 1 | scope-match rule minor ambiguity | High |
  | K | 4 | anchor: `crates/audit` floor 4 | High |
  | P | 5 | anchor: `crates/audit` floor 5 (governance/audit) | High |
  | X | 3 | service + domain + audit + repo | High |

  **Base 56 · penalties auth_security (+10, P floor ≥ 4) · Final 66 → Complex → plan-first.**

- **Happy paths considered:**
  - `HP-1`: active consent → `require_active_consent` passes; check audited. 
- **Edge cases considered:**
  - `EC-1`: no consent → refused + audit, no derivative proceeds. (SC-CONSENT-3)
  - `EC-2`: revoked consent → refused + audit (latest row is revoke). (SC-CONSENT-2 + SC-CONSENT-3)
  - `EC-3`: scope mismatch (consent for a different scope) → refused, fail-closed.
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> none
    none --> active: grant + audit
    active --> revoked: revoke + audit
    active --> allowed: synthesis check passes + audit
    none --> refused: synthesis check -> refused + audit
    revoked --> refused: synthesis check -> refused + audit
  ```

- **Handoff prompt:**
  > S-110-T2 — consent transitions + fail-closed TTS precondition + audit (X11). Docs: this ledger
  > + plan §D2, ADR-008/018. Add `apps/api/src/services/consent_gate.rs` with
  > `require_active_consent`; audit grant/revoke + every check. AC: SC-CONSENT-3, reusable gate,
  > ≥90% cov. Stop after tests; do not start S-110-T3.

---

## S-110-T3 — Compliance read API (audit/rights viewer + consent grant/revoke)

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 44 → band **Med-high (41–55)** → **Plan + explicit acceptance criteria before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
  (escalate to `Claude Opus 4.8` if it stalls) · thinking On
- **Depends on:** S-110-T2
- **Objective:** Expose ownership/org-scoped **read** endpoints over `audit_events` and
  `rights_records`, plus consent grant/revoke endpoints (calling S-110-T2). Reads never mutate
  governance rows. (Plan §D3.)
- **Inputs:** `consent_gate` (S-110-T2), `audit_repo`/`rights_repo` (add scoped read queries),
  org guard (S-100-T2), `apps/api` route patterns.
- **Outputs:** `apps/api/src/routes/compliance.rs` + `dto/compliance.rs`; scoped read queries
  in `audit_repo.rs`/`rights_repo.rs`; route/integration tests.
- **Acceptance criteria:**
  - Audit/rights reads return only the caller's owned/org assets; cross-owner denied. (SC-AUDIT-2)
  - Reads are side-effect-free (no governance mutation).
  - Consent grant/revoke endpoints write append-only rows via S-110-T2 + audit. (SC-CONSENT-1/2)
  - ≥90% coverage; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 12 → 2 | High |
  | F | 2 | 4 files | High |
  | D | 3 | anchor: `crates/db` (ADR-006, ADR-018) floor 3 | High |
  | T | 2 | route/repo tests exist | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | anchor: `crates/db` floor 3 | High |
  | P | 3 | new read + consent-write endpoints | High |
  | X | 3 | routes + dto + 2 repos | High |

  **Base 44 · penalties none · Final 44 → Med-high → plan+AC before approval.**

- **Happy paths considered:**
  - `HP-1`: owner reads audit timeline → chronological events for their asset. (SC-AUDIT-1)
  - `HP-2`: owner reads rights ledger → rights entries returned. (SC-RIGHTS-1)
- **Edge cases considered:**
  - `EC-1`: read audit for a non-owned asset → denied, no data. (SC-AUDIT-2)
  - `EC-2`: grant consent without evidence reference → rejected (fail-closed).
- **Diagram:**

  ```mermaid
  flowchart LR
    C[web/mobile] -->|GET /api/assets/{id}/audit| G[gateway] --> A[apps/api compliance routes]
    A --> M[ownership/org guard] --> RD[(audit_repo / rights_repo read-only)]
    C -->|POST /api/consents| A --> CG[consent_gate] --> CR[(consent_repo)]
  ```

- **Handoff prompt:**
  > S-110-T3 — compliance read API + consent endpoints. Docs: this ledger + plan §D3, ADR-018.
  > Add `routes/compliance.rs` + dto; ownership-scoped audit/rights reads (no mutation) +
  > consent grant/revoke via S-110-T2. AC: SC-AUDIT-1/2 + SC-RIGHTS-1 + SC-CONSENT-1/2, ≥90% cov.
  > Stop after tests; do not start S-110-T4.

---

## S-110-T4 — Web compliance dashboard

- **Status:** [ ] Not started
- **Type:** Development (TS/web) · **Effort:** M
- **RRI:** 30 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-110-T3
- **Objective:** Build the web compliance dashboard: an audit timeline per asset, the rights
  ledger view, and consent management (grant/revoke).
- **Inputs:** S-100-T4 web shell/client, S-110-T3 endpoints, BDD scenarios, `data-testid` convention.
- **Outputs:** `ComplianceScreen.tsx`, `ConsentScreen.tsx`, `AuditTimeline.tsx`;
  `data-testid`s (`compliance-screen`, `consent-screen`, `audit-timeline`,
  `consent-grant`, `consent-revoke`); component tests.
- **Acceptance criteria:**
  - Audit timeline renders chronological events; rights ledger renders entries. (SC-AUDIT-1, SC-RIGHTS-1)
  - Consent grant/revoke update the displayed status. (SC-CONSENT-1/2)
  - `data-testid`s present; `npm test` + typecheck green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 10 → 1 | High |
  | F | 2 | 4 files | High |
  | D | 2 | web UI + API integration | High |
  | T | 1 | web harness exists (S-100-T4) | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | API coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 3 | screens + component + test | High |

  **Base 30 · penalties none · Final 30 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: open compliance → audit timeline + rights entries shown. (SC-AUDIT-1, SC-RIGHTS-1)
- **Edge cases considered:**
  - `EC-1`: asset with no events → empty timeline, no error.
  - `EC-2`: revoke active consent → status flips to inactive in the UI. (SC-CONSENT-2)
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> compliance
    compliance --> timeline: view audit
    compliance --> rights: view rights ledger
    compliance --> consent: grant/revoke
  ```

- **Handoff prompt:**
  > S-110-T4 — web compliance dashboard. Docs: this ledger + plan §D4. Build Compliance/Consent/
  > AuditTimeline against S-110-T3, data-testids, component tests. AC: SC-AUDIT-1 + SC-RIGHTS-1 +
  > SC-CONSENT-1/2, tests+typecheck green. Stop after tests; do not start S-110-T6.

---

## S-110-T5 — Mobile consent + compliance surfaces

- **Status:** [ ] Not started
- **Type:** Development (TS/RN) · **Effort:** M
- **RRI:** 31 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-110-T3
- **Objective:** Add mobile consent capture (grant/revoke from the device) and a compliance
  summary view, reachable from the authed tree.
- **Inputs:** `mobile/src/api/client.ts`, nav, S-110-T3 endpoints, S-055 `testID` convention.
- **Outputs:** `ConsentScreen.tsx`, `ComplianceScreen.tsx`, nav route,
  `consent-screen`/`compliance-screen` testIDs, component tests.
- **Acceptance criteria:**
  - Consent screen grants/revokes; compliance summary shows recent audit/rights state. (SC-CONSENT-1/2, SC-AUDIT-1)
  - `session_expired` triggers logout (transport contract preserved).
  - testIDs present; `npm test` + typecheck green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 8 → 1 | High |
  | F | 2 | 4 files | High |
  | D | 2 | mobile UI + API integration | High |
  | T | 1 | mobile harness exists | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | network/API coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 2 | screens + nav + test | High |

  **Base 31 · penalties none · Final 31 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: grant consent from device → status active; reflected on refresh. (SC-CONSENT-1)
- **Edge cases considered:**
  - `EC-1`: revoke → status inactive, history preserved. (SC-CONSENT-2)
  - `EC-2`: 401 → `auth.logout()` (contract preserved).
- **Diagram:**

  ```mermaid
  flowchart LR
    H[home-screen] --> CMP[compliance-screen]
    CMP --> CON[consent-screen]
    CON --> API[/api/consents grant/revoke]
  ```

- **Handoff prompt:**
  > S-110-T5 — mobile consent + compliance. Docs: this ledger + plan §D4. Add Consent/Compliance
  > screens + nav + testIDs against S-110-T3. AC: SC-CONSENT-1/2 + SC-AUDIT-1, 401→logout,
  > tests+typecheck green. Stop after tests; do not start S-110-T6.

---

## S-110-T6 — E2E fixtures + docs/roadmap sync

- **Status:** [ ] Not started
- **Type:** Development (Node fixture) / ops / docs · **Effort:** S
- **RRI:** 24 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-110-T4, S-110-T5
- **Objective:** Extend the mock-gateway with compliance/consent fixtures, author web
  (Playwright) + mobile (Maestro) compliance flows, and sync status docs (including marking
  X11 addressed at the contract level).
- **Inputs:** `mock-gateway-server.mjs`, S-110-T3 contracts, S-055 env, `docs/plan/roadmap.md`
  (X11 row).
- **Outputs:** `/api/*` compliance/consent fixtures + `node --test`; `web/e2e/compliance.spec.ts`;
  `mobile/maestro/compliance.yaml`; roadmap row updated (X11 contract-level closure noted);
  X-S-110-1/2/3 recorded; BDD mapping closed.
- **Acceptance criteria:**
  - Web + mobile compliance flows pass against the deterministic mock-gateway, including the
    synthesis-blocked-without-consent narrative. (SC-CONSENT-3)
  - `make qa-docs` green; status docs consistent; X11 status reconciled; follow-ups recorded.
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
  - `HP-1`: audit/rights/consent flows pass; consent-blocked-synthesis flow asserts refusal. (SC-CONSENT-3)
- **Edge cases considered:**
  - `EC-1`: `/api/*` compliance route without session → 401, no data.
  - `EC-2`: cross-owner audit read in the flow → denied. (SC-AUDIT-2)
- **Handoff prompt:**
  > S-110-T6 — E2E fixtures + docs sync. Docs: this ledger + plan + roadmap (X11). Add mock-gateway
  > compliance/consent `/api/*` + `node --test`, `web/e2e/compliance.spec.ts`,
  > `mobile/maestro/compliance.yaml`, sync roadmap + X-S-110-1/2/3 + X11 contract-level note. AC:
  > flows pass, qa-docs green. Stop after sync.

---

## Coverage contract

This ledger does **not** declare `Behavioral coverage contract: unit-v1`. Development
tasks (S-110-T1…S-110-T5) still require the standard `Unit coverage certification` + `Owner
final verification` completion record per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
before being marked `[x] Done`. The BDD `.feature` scenarios (S-110-T0) are the behavioral
source of truth from which each task's `HP-#`/`EC-#` cases are derived.
