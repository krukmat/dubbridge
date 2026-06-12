# Tasks: S-160 — Human Review & Publication Workspace

**Plan:** `docs/plan/s-160-review-publication-workspace.md`
**Roadmap phase:** `S-160` (depends on `S-100`).
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-008, ADR-018, ADR-023, ADR-024, ADR-006.

> **Namespace.** This phase uses the **`S-160-T`** prefix (`S-160-T0`–`S-160-T7`). Always fully
> qualify cross-slice references (`S-160-T2`, `S-100-T1`), never bare `T2`.

> **RRI provenance.** Every RRI below was computed with `python3 scripts/rri.py`
> (platform `dubbridge`) at planning time, not by hand. Final RRI is recomputed at
> presentation. All tasks scored ≤ 70 → no mandatory decomposition; `S-160-T1`, `S-160-T2`,
> and `S-160-T4` land in **Complex (56–70)** and therefore require a reviewed plan before
> implementation — this ledger + the plan provide it.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
S-160-T0 (BDD) ─▶ S-160-T1 (schema+domain+repos) ─▶ S-160-T2 (review state machine + publication gate + audit) ─▶ S-160-T3 (review/publication API) ─┬─▶ S-160-T4 (notifications) ─┬─▶ S-160-T5 (web console) ─┐
                                                                                                                                          │                          ├─▶ S-160-T6 (mobile + push) ┤
                                                                                                                                          └──────────────────────────┴─▶ S-160-T7 (E2E + docs) ◀───┘
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| S-160-T0 | BDD `.feature` specs + mapping | — | 11 | Low | S |
| S-160-T1 | Schema + domain + repos (review/decisions/publications) | S-160-T0 | 63 | Complex | L |
| S-160-T2 | Review state machine + publication gate + audit | S-160-T1 | 66 | Complex | L |
| S-160-T3 | Review/publication API | S-160-T2 | 44 | Med-high | L |
| S-160-T4 | Notifications mechanism (table + emit + push) | S-160-T3 | 66 | Complex | L |
| S-160-T5 | Web review console | S-160-T3, S-160-T4 | 33 | Moderate | M |
| S-160-T6 | Mobile reviewer surfaces + push | S-160-T3, S-160-T4 | 31 | Moderate | M |
| S-160-T7 | E2E fixtures + docs/roadmap sync | S-160-T5, S-160-T6 | 24 | Low | S |

## Model resolution (capability → current vendor model)

| Band | Codex | Claude Code | Thinking |
|---|---|---|---|
| Low (0–25) | `GPT-5.2-Codex` | `Claude Haiku 4.5` | Off |
| Moderate (26–40) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` | Off |
| Med-high (41–55) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` (escalate to `Claude Opus 4.8` if it stalls) | On |
| Complex (56–70) | `GPT-5.2-Codex` | `Claude Opus 4.8` | On |

---

## S-160-T0 — BDD `.feature` specs + BDD⇄web⇄mobile⇄unit mapping

- **Status:** [ ] Not started
- **Type:** Planning / docs (BDD authoring) · **Effort:** S
- **RRI:** 11 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** — (BDD-first)
- **Objective:** Author the Gherkin specs defining acceptance for the review/publication
  slice and the mapping convention (scenario ID ⇄ web/mobile flow ⇄ `HP-#`/`EC-#`).
- **Inputs:** plan §D1–§D6; S-100 role model; S-010 artifact lineage; ADR-008.
- **Outputs:** `docs/bdd/p5-review.feature`; mapping rows appended to `docs/bdd/README.md`.
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

- **Handoff prompt:**
  > S-160-T0 — author BDD specs. Docs: this ledger + plan §D1–§D6. Create
  > `docs/bdd/p5-review.feature` (SC-REVIEW-1/2/3, SC-PUBLISH-1/2, SC-NOTIFY-1) and append
  > mapping rows to `docs/bdd/README.md`. AC: stable IDs mapped to web/mobile + HP/EC,
  > qa-docs green. Stop after docs; do not start S-160-T1.

---

## S-160-T1 — Schema + domain + repos (review tasks, decisions, publications)

- **Status:** [ ] Not started
- **Type:** Development (Rust + SQL) · **Effort:** L
- **RRI:** 63 → band **Complex (56–70)** → **Plan first; human reviews the plan; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-160-T0
- **Objective:** Introduce the review/publication data model — `review_tasks`,
  append-only `review_decisions`, `publications` — plus domain entities and repos. (Plan §D1.)
- **Inputs:** `infra/migrations/` (next free index 0013), `migration 0007` (append-only
  governance posture), `crates/db/src/rights_repo.rs` (append-only patterns).
- **Outputs:**
  - `0013_create_review_tasks.sql`, `0014_create_review_decisions.sql` (append-only),
    `0015_create_publications.sql`.
  - `crates/domain/src/review.rs` (review task / verdict / publication state derivation).
  - `crates/db/src/review_repo.rs` (insert decision, derive latest state, list queue).
  - Unit/integration tests; ≥90% coverage.
- **Acceptance criteria:**
  - `review_decisions` is append-only (no UPDATE/DELETE path); current state = latest row.
  - Task state decodes strictly; unknown verdict rejected (fail-closed).
  - Migrations apply cleanly, FK-constrained to assets/projects.
  - ≥90% coverage; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 12 → 2 | High |
  | F | 2 | 5 files | High |
  | D | 4 | anchor: `infra/migrations` (ADR-008, ADR-018) floor 4 | High |
  | T | 2 | db/domain area has tests | High |
  | A | 0 | criteria + examples present | High |
  | K | 4 | anchor: `infra/migrations` floor 4 | High |
  | P | 5 | anchor: `infra/migrations` floor 5 (schema/data) | High |
  | X | 3 | migrations + domain + repo | High |

  **Base 53 · penalties auth_security (+10, P floor ≥ 4) · Final 63 → Complex → plan-first.**

- **Happy paths considered:**
  - `HP-1`: insert approve decision → task state derives `approved`. (SC-REVIEW-2)
- **Edge cases considered:**
  - `EC-1`: attempt to UPDATE a decision row → not supported; supersede via new row only.
  - `EC-2`: unknown verdict string → decode error, no row.
- **Diagram:**

  ```mermaid
  erDiagram
    assets ||--o{ review_tasks : reviewed_by
    review_tasks ||--o{ review_decisions : append_only
    review_tasks ||--o| publications : gates
  ```

- **Handoff prompt:**
  > S-160-T1 — review/publication schema + domain + repos. Docs: this ledger + plan §D1, ADR-008/018.
  > Add migrations 0013–0015, `crates/domain/src/review.rs`, `crates/db/src/review_repo.rs`.
  > AC: append-only decisions, latest-state derivation, strict decode, ≥90% cov. Stop after tests;
  > do not start S-160-T2.

---

## S-160-T2 — Review state machine + publication gate + audit

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 66 → band **Complex (56–70)** → **Plan first; human reviews the plan; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-160-T1
- **Objective:** Implement the review transition rules and the **fail-closed publication
  gate**: a publication cannot be created unless its governing review task is `approved`.
  Emit durable audit on decisions and publish attempts. (Plan §D2.)
- **Inputs:** `review_repo` (S-160-T1), `crates/audit` emission, ADR-008 (rights gate as the
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
    C[web/mobile] -->|POST /api/review/{id}/decision| G[gateway] --> A[apps/api review routes]
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
- **Inputs:** `infra/migrations/` (next free index 0016), `review_gate` emit points,
  ADR-018 (redaction), Expo push.
- **Outputs:**
  - `0016_create_notifications.sql` (recipient, kind, ref, read_at).
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
  > S-160-T4 — notifications. Docs: this ledger + plan §D5, ADR-018. Add migration 0016,
  > `notification_repo`, emit hooks, `routes/notifications.rs`, `mobile/src/push/registerPush.ts`.
  > AC: rows on assign/decide/publish, no-PII payload, owner-scoped list, ≥90% cov. Stop after tests;
  > do not start S-160-T5.

---

## S-160-T5 — Web review console

- **Status:** [ ] Not started
- **Type:** Development (TS/web) · **Effort:** M
- **RRI:** 33 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-160-T3, S-160-T4
- **Objective:** Build the web review console: queue, side-by-side preview (original vs
  derived output), approve/reject with comment, and a publish action gated on state.
- **Inputs:** S-100-T4 web shell/client, S-160-T3 endpoints, S-160-T4 notifications, BDD scenarios.
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

- **Handoff prompt:**
  > S-160-T5 — web review console. Docs: this ledger + plan §D6. Build ReviewQueue/ReviewDetail/
  > SideBySidePreview against S-160-T3/S-160-T4, gated publish, data-testids, component tests.
  > AC: SC-REVIEW-1/2/3 + SC-PUBLISH-2, tests+typecheck green. Stop after tests; do not start S-160-T7.

---

## S-160-T6 — Mobile reviewer surfaces + push

- **Status:** [ ] Not started
- **Type:** Development (TS/RN) · **Effort:** M
- **RRI:** 31 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-160-T3, S-160-T4
- **Objective:** Add a mobile reviewer inbox and a decide screen (approve/reject on the go),
  wired to push notifications from S-160-T4.
- **Inputs:** `mobile/src/api/client.ts`, nav, S-160-T3 endpoints, `registerPush.ts` (S-160-T4).
- **Outputs:** `ReviewInboxScreen.tsx`, `ReviewDecisionScreen.tsx`, nav route,
  `review-inbox-screen`/`review-decision-screen` testIDs, component tests.
- **Acceptance criteria:**
  - Inbox lists assigned tasks; decision screen posts approve/reject. (SC-REVIEW-1/2/3)
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
    INBOX --> DEC[review-decision-screen]
    DEC --> API[/api/review/{id}/decision]
  ```

- **Handoff prompt:**
  > S-160-T6 — mobile reviewer + push. Docs: this ledger + plan §D5–§D6. Add ReviewInbox/
  > ReviewDecision + nav + testIDs, push deep-link via registerPush. AC: SC-REVIEW-1/2/3 +
  > SC-NOTIFY-1, 401→logout, tests+typecheck green. Stop after tests; do not start S-160-T7.

---

## S-160-T7 — E2E fixtures + docs/roadmap sync

- **Status:** [ ] Not started
- **Type:** Development (Node fixture) / ops / docs · **Effort:** S
- **RRI:** 24 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-160-T5, S-160-T6
- **Objective:** Extend the mock-gateway with review/publication/notification fixtures,
  author web (Playwright) + mobile (Maestro) review flows, and sync status docs.
- **Inputs:** `mock-gateway-server.mjs`, S-160-T3/T4 contracts, S-055 env, `docs/plan/roadmap.md`.
- **Outputs:** `/api/*` review fixtures + `node --test`; `web/e2e/review.spec.ts`;
  `mobile/maestro/review.yaml`; roadmap row updated; X-S-160-1/2/3 recorded; BDD mapping closed.
- **Acceptance criteria:**
  - Web + mobile review flows pass against the deterministic mock-gateway, including the
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
  > review/publication/notification `/api/*` + `node --test`, `web/e2e/review.spec.ts`,
  > `mobile/maestro/review.yaml`, sync roadmap + X-S-160-1/2/3. AC: flows pass, qa-docs green.
  > Stop after sync.

---

## Coverage contract

This ledger does **not** declare `Behavioral coverage contract: unit-v1`. Development
tasks (S-160-T1…S-160-T6) still require the standard `Unit coverage certification` + `Owner
final verification` completion record per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
before being marked `[x] Done`. The BDD `.feature` scenarios (S-160-T0) are the behavioral
source of truth from which each task's `HP-#`/`EC-#` cases are derived.
