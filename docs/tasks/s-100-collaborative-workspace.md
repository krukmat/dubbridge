# Tasks: S-100 — Collaborative Localization Workspace

**Plan:** `docs/plan/s-100-collaborative-workspace.md`
**Roadmap phase:** `S-100` (foundation for `S-110`, `S-160`).
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-023, ADR-024, ADR-008, ADR-018, ADR-006.
**Prior-phase reuse:** S-040 gateway session/proxy, S-050 mobile client/session/navigation,
S-055 testID + mock-gateway conventions, and S-060's planned asset lifecycle boundary.

> **Namespace.** This phase uses the **`S-100-T`** prefix (`S-100-T0`–`S-100-T7`). Always fully
> qualify cross-slice references (`S-100-T2`, `S-160-T1`), never bare `T2`.

> **RRI provenance.** Every RRI below was computed with `python3 scripts/rri.py`
> (platform `dubbridge`) at planning time, not by hand. Final RRI is recomputed at
> each task's presentation per the workflow. All tasks scored ≤ 70 → no mandatory
> decomposition; `S-100-T1` and `S-100-T2` land in **Complex (56–70)** and therefore require
> a reviewed plan before implementation — this ledger + the plan provide it.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
S-100-T0 (BDD) ─▶ S-100-T0b (ADR X-S-100-1) ─▶ S-100-T1 (schema+domain+repos) ─┬─▶ S-100-T2 (org-aware authz) ─┐
                                                                          └────────────────────────────┼─▶ S-100-T3 (workspace API) ─┬─▶ S-100-T4 (web skeleton) ─▶ S-100-T5 (web screens) ─┐
                                                                                                        │                          ├─▶ S-100-T6 (mobile projects) ─────────────────────┤
                                                                                                        │                          └─▶ S-100-T7 (E2E + docs sync) ◀────────────────────┘
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| S-100-T0 | BDD `.feature` specs + mapping | — | 11 | Low | S |
| S-100-T0b | ADR authoring: org/membership/role authorization model (X22 → X-S-100-1) | S-100-T0 | 18 | Low | S |
| S-100-T1 | Schema + domain + repos (orgs/projects/target-langs) | S-100-T0b | 60 | Complex | L |
| S-100-T2 | Org-aware authorization (membership + role) | S-100-T1 | 64 | Complex | L |
| S-100-T3 | Workspace API + audit | S-100-T1, S-100-T2 | 44 | Med-high | L |
| S-100-T4 | Web console skeleton (Vite/React + gateway auth) | S-100-T3 | 35 | Moderate | M |
| S-100-T5 | Web org/project screens | S-100-T4 | 29 | Moderate | M |
| S-100-T6 | Mobile project surfaces | S-100-T3 | 31 | Moderate | M |
| S-100-T7 | E2E fixtures + `npm run` wiring + docs/roadmap sync | S-100-T3, S-100-T5, S-100-T6 | 24 | Low | S |

## Model resolution (capability → current vendor model)

Resolved against the active environment at planning time; reconfirm at presentation.

| Band | Codex | Claude Code | Thinking |
|---|---|---|---|
| Low (0–25) | `GPT-5.2-Codex` | `Claude Haiku 4.5` | Off |
| Moderate (26–40) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` | Off |
| Med-high (41–55) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` (escalate to `Claude Opus 4.8` if it stalls) | On |
| Complex (56–70) | `GPT-5.2-Codex` | `Claude Opus 4.8` | On |

## Prior-slice reuse checkpoint

Before presenting any S-100 implementation task, re-check the current state of:

- `docs/plan/s-055-maestro-screenshot-suite.md` and `docs/tasks/s-055-maestro-screenshot-suite.md`
  for available testID, Maestro, and mock-gateway runner conventions.
- `docs/plan/s-060-mobile-asset-lifecycle.md` and `docs/tasks/s-060-mobile-asset-lifecycle.md`
  for any delivered `GET /assets`, mobile upload, and `/api/*` fixture work.
- `mobile/src/api/client.ts`, `mobile/src/api/types.ts`, and
  `mobile/src/navigation/RootNavigator.tsx` for the live mobile session/error/nav
  contracts.

S-100 must consume delivered prior work rather than fork it. In particular, S-100 does not
own a generic asset list endpoint, mobile upload flow, or standalone asset fixture
store; those remain S-060 responsibilities unless already built and available for
S-100 to reuse.

---

## S-100-T0 — BDD `.feature` specs + BDD⇄web⇄mobile⇄unit mapping

- **Status:** [ ] Not started
- **Type:** Planning / docs (BDD authoring) · **Effort:** S
- **RRI:** 11 → band **Low (0–25)** → **auto-execute** (present RRI + one-line summary, then proceed)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** — (BDD-first)
- **Objective:** Author the Gherkin specs defining acceptance for the workspace slice and
  the mapping convention (scenario ID ⇄ web/mobile flow ⇄ `HP-#`/`EC-#`).
- **Inputs:** plan §D1–§D7; current principal/auth model; S-040 gateway session contract.
- **Outputs:** `docs/bdd/p4-workspace.feature`; `docs/bdd/README.md` (mapping table + convention).
- **Acceptance criteria:**
  - Each scenario has a stable ID and maps to one web/mobile flow and ≥1 `HP-#`/`EC-#`.
  - Scenarios are behavioral (no implementation calls).
  - `make qa-docs` passes (no dangling references introduced).
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
  Feature: Collaborative localization workspace

    Scenario: SC-ORG-1 Create an organization and become its owner
      Given I am an authenticated user with no organization
      When I create an organization
      Then I am its owner and can see it in my organization list

    Scenario: SC-MEMBER-1 Invite a member with a role
      Given I am an org owner or admin
      When I add a member with the "reviewer" role
      Then that member can access the org with reviewer permissions

    Scenario: SC-MEMBER-2 Non-member is denied org access
      Given I am authenticated but not a member of an organization
      When I request that organization's projects
      Then I am denied access and no project data is returned

    Scenario: SC-PROJECT-1 Create a project and link assets
      Given I am an org owner or admin and own some assets
      When I create a project and link my assets to it
      Then the project lists those assets

    Scenario: SC-LANG-1 Declare target languages for a project
      Given I am viewing a project I can edit
      When I set a source language and one or more target languages
      Then the project records the localization intent
  ```

- **Handoff prompt:**
  > S-100-T0 — author BDD specs. Docs: this ledger + plan §D1–§D7. Create
  > `docs/bdd/p4-workspace.feature` (SC-ORG-1, SC-MEMBER-1/2, SC-PROJECT-1, SC-LANG-1)
  > and `docs/bdd/README.md` mapping table. AC: stable IDs mapped to web/mobile + HP/EC,
  > qa-docs green. Stop after docs; do not start S-100-T0b.

---

## S-100-T0b — ADR authoring: org/membership/role authorization model (X22 → X-S-100-1)

- **Status:** [ ] Not started
- **Type:** Architecture decision · **Effort:** S
- **RRI:** 18 → band **Low (0–25)** → **auto-execute** (present RRI + one-line summary, then proceed)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-100-T0
- **Blocks:** S-100-T1, S-100-T2 — **neither may start until this ADR is merged**
- **Objective:** Author and merge the ADR that defines the multi-tenant authorization model
  for DubBridge: how organizations form tenancy boundaries, how `org_members` role-enum
  maps to API scopes, and how the Axum middleware enforces org membership as a
  fail-closed precondition on every org-scoped route. Closes X22 / X-S-100-1 and
  satisfies the "plan-first" gate for S-100-T1/T2.
- **Inputs:**
  - `crates/auth/src/principal.rs` — current flat `AuthenticatedPrincipal`
  - ADR-023 — JWT resource server, API caller identity at Axum boundary
  - ADR-008 — fail-closed precondition posture
  - `docs/plan/s-100-collaborative-workspace.md` §D2 (tenancy model) and §D3 (auth design)
- **Outputs:**
  - `docs/adr/ADR-NNN-org-membership-authorization.md` — decision record covering:
    - Org as tenancy boundary; assets stay uploader-owned; projects *link* assets
    - `OrgMemberPrincipal` wraps `AuthenticatedPrincipal` with resolved `org_id` + `role`
    - Role enum: `Owner | Admin | Editor | Reviewer | Viewer` — strict decode, fail-closed
    - Axum middleware `require_org_member(min_role)` as precondition extractor
    - No org-scoped JWT claim; membership resolved from DB at request time
    - Audit obligation: membership changes emit `audit_events` rows (ADR-018)
  - ADR index entry added to `docs/adr/README.md`
- **Acceptance criteria:**
  - ADR file present in `docs/adr/` with a real sequential number.
  - `docs/adr/README.md` index updated with the new entry.
  - ADR text covers: tenancy model, role enum, middleware contract, fail-closed posture,
    audit obligation, and open follow-ups.
  - `make qa-docs` passes (ADR index consistent, no dangling refs).
- **Handoff prompt:**
  > S-100-T0b — author ADR for org/membership/role authorization (X22). Inputs:
  > `crates/auth/src/principal.rs`, ADR-023, ADR-008, plan §D2/§D3. Create
  > `docs/adr/ADR-NNN-org-membership-authorization.md` (tenancy boundary, role enum,
  > Axum middleware contract, fail-closed posture, audit obligation) and update
  > `docs/adr/README.md` index. AC: real ADR number, index updated, qa-docs green.
  > Stop after docs; do not start S-100-T1.

---

## S-100-T1 — Schema + domain + repos (organizations, projects, target languages)

- **Status:** [ ] Not started
- **Type:** Development (Rust + SQL) · **Effort:** L
- **RRI:** 60 → band **Complex (56–70)** → **Plan first; human reviews the plan before
  implementation; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-100-T0b
- **Objective:** Introduce the tenancy data model — `organizations` + `org_members`
  (role enum), `projects` + `project_assets`, `target_languages` — plus the domain
  entities and DB repos. (Plan §D1–§D3.)
- **Inputs:** `infra/migrations/` (next free index 0010), `crates/domain/src/asset.rs`
  (uploader-owned invariant), `crates/db/src/asset_repo.rs` (repo patterns and any
  S-060 list/ownership query if it has landed).
- **Outputs:**
  - `0010_create_organizations.sql` (`organizations`, `org_members` with role enum, FKs).
  - `0011_create_projects.sql` (`projects` → org FK; `project_assets` M:N → `assets`).
  - `0012_create_target_languages.sql` (`target_languages` → project FK; BCP-47 codes).
  - `crates/domain/src/workspace.rs` (Org/Project/Membership/Role/TargetLanguage + invariants).
  - `crates/db/src/workspace_repo.rs` (insert/list/link, ordered, ownership-aware).
  - Unit/integration tests; ≥90% coverage on new domain + repo code.
- **Acceptance criteria:**
  - Migrations apply cleanly and are reversible-safe (forward-only, FK-constrained).
  - `project_assets` references existing `assets` without reassigning `uploader_id`.
  - Project linking validates asset existence + caller ownership without introducing
    a duplicate generic asset list endpoint or mobile upload surface.
  - Role enum decodes strictly (unknown role rejected, fail-closed).
  - ≥90% line coverage on the new domain + repo; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 10 → 1 | High |
  | F | 2 | 5 files | High |
  | D | 4 | anchor: `infra/migrations` (ADR-008, ADR-018) floor 4 | High |
  | T | 2 | db/domain area has tests | High |
  | A | 0 | criteria + examples present | High |
  | K | 4 | anchor: `infra/migrations` floor 4 | High |
  | P | 5 | anchor: `infra/migrations` floor 5 (schema/data) | High |
  | X | 3 | migrations + domain + repo | High |

  **Base 50 · penalties auth_security (+10, P floor ≥ 4) · Final 60 → Complex → plan-first.**

- **Happy paths considered:**
  - `HP-1`: create org → owner membership row written; org listed for owner. (SC-ORG-1)
  - `HP-2`: create project in org + link 2 owned assets → `project_assets` has 2 rows. (SC-PROJECT-1)
- **Edge cases considered:**
  - `EC-1`: link an asset not owned by the caller → rejected (ownership preserved, ADR-023).
  - `EC-2`: unknown role string on insert → decode error, row not written (fail-closed).
  - `EC-3`: duplicate `(project_id, asset_id)` link → idempotent / unique-constrained, no dup.
- **Diagram:**

  ```mermaid
  erDiagram
    organizations ||--o{ org_members : has
    organizations ||--o{ projects : owns
    projects ||--o{ project_assets : groups
    assets ||--o{ project_assets : linked_by
    projects ||--o{ target_languages : declares
  ```

- **Handoff prompt:**
  > S-100-T1 — tenancy schema + domain + repos. Docs: this ledger + plan §D1–§D3, ADR-023/008/018.
  > Add migrations 0010–0012, `crates/domain/src/workspace.rs`, `crates/db/src/workspace_repo.rs`.
  > AC: assets not reassigned, strict role decode, ≥90% cov, tests green. Stop after tests;
  > do not start S-100-T2.

---

## S-100-T2 — Org-aware authorization (membership + role)

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 64 → band **Complex (56–70)** → **Plan first; human reviews the plan before
  implementation; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.8` · thinking On
- **Depends on:** S-100-T1
- **Objective:** Add an org-scope authorization layer on top of the ADR-023 scope check:
  resolve the principal's role in the target org and enforce role→action
  (`owner/admin` write, `reviewer/viewer` read for S-100), fail-closed.
  (Plan §D2.)
- **Inputs:** `crates/auth/src/principal.rs`, `crates/auth/src/axum.rs`, `workspace_repo`
  (membership lookup), ADR-023, ADR-008.
- **Outputs:**
  - `crates/auth/src/membership.rs` (role model; `require_org_role(principal, org, role)`).
  - `apps/api/src/middleware/org_scope.rs` (extractor/guard resolving membership per request).
  - Tests for: member with sufficient role, insufficient role, non-member, missing org.
- **Acceptance criteria:**
  - A non-member request to an org resource is rejected (403), no data leaked. (SC-MEMBER-2)
  - Insufficient role (e.g. `viewer` attempting a write) is rejected; sufficient role passes.
  - The existing OAuth scope check is preserved (org guard is additive, not a replacement).
  - ≥90% line coverage on the guard + role logic; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 14 → 2 | High |
  | F | 2 | 3 files | High |
  | D | 4 | anchor: `crates/auth` (ADR-023) floor 4 | High |
  | T | 2 | auth crate has tests | High |
  | A | 1 | role→action matrix has minor ambiguity | High |
  | K | 4 | anchor: `crates/auth` floor 4 | High |
  | P | 4 | anchor: `crates/auth` floor 4 (authz/data visibility) | High |
  | X | 3 | auth module + middleware + routes | High |

  **Base 54 · penalties auth_security (+10, P floor ≥ 4) · Final 64 → Complex → plan-first.**

- **Happy paths considered:**
  - `HP-1`: `admin` member adds a member → allowed; audit row written. (SC-MEMBER-1)
- **Edge cases considered:**
  - `EC-1`: non-member requests org projects → 403, empty body. (SC-MEMBER-2)
  - `EC-2`: `viewer` attempts a project write → 403 (role insufficient).
  - `EC-3`: valid scope but unknown org id → 404/403 fail-closed, no enumeration leak.
- **Diagram:**

  ```mermaid
  flowchart LR
    R[org-scoped request] --> S[authenticate_bearer + require_scope]
    S --> M[org_scope guard: resolve membership+role]
    M -->|member & role ok| H[handler]
    M -->|non-member / insufficient| D[403 fail-closed]
  ```

- **Handoff prompt:**
  > S-100-T2 — org-aware authorization. Docs: this ledger + plan §D2, ADR-023/008. Add
  > `crates/auth/src/membership.rs` (role model + `require_org_role`) and
  > `apps/api/src/middleware/org_scope.rs`. AC: non-member/insufficient-role rejected,
  > scope check preserved, ≥90% cov, tests green. Stop after tests; do not start S-100-T3.

---

## S-100-T3 — Workspace API + audit

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 44 → band **Med-high (41–55)** → **Plan + explicit acceptance criteria
  required before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
  (escalate to `Claude Opus 4.8` if it stalls) · thinking On
- **Depends on:** S-100-T1, S-100-T2
- **Objective:** Expose org/project/member/target-language endpoints in `apps/api`,
  guarded by the org-scope layer, emitting durable audit on governance events, and
  returning linked asset summaries through project endpoints without taking over
  S-060's generic asset lifecycle. (Plan §D2.)
- **Inputs:** `workspace_repo` (S-100-T1), org guard (S-100-T2), existing audit emission,
  `apps/api/src/routes/ingestion.rs` (route patterns), `AssetSummaryResponse` and
  any S-060-delivered ownership/list helper.
- **Outputs:**
  - `apps/api/src/routes/workspace.rs` + `apps/api/src/dto/workspace.rs`.
  - Endpoints: create/list orgs; add/list members; create/list projects; link assets;
    set/list target languages — each role-guarded.
  - Project detail/list responses include linked asset summaries needed by web/mobile
    project screens; no new standalone `GET /assets` or upload endpoint is created here.
  - Audit rows on org/member/project governance events (ADR-018).
  - Route + integration tests for happy + denied paths.
- **Acceptance criteria:**
  - Each mutation emits a durable audit row; reads are ownership/role-scoped.
  - Denied (non-member / insufficient role) paths return fail-closed without side effects.
  - ≥90% line coverage on new routes/dtos; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 16 → 2 | High |
  | F | 2 | 3 files | High |
  | D | 3 | anchor: `crates/db` (ADR-006, ADR-018) floor 3 | High |
  | T | 2 | route/repo tests exist | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | anchor: `crates/db` floor 3 (DB/HTTP) | High |
  | P | 3 | new write endpoints (org-scoped) | High |
  | X | 3 | routes + dto + repo | High |

  **Base 44 · penalties none · Final 44 → Med-high → plan+AC before approval.**

- **Happy paths considered:**
  - `HP-1`: owner creates project + links assets → 201; audit row written; project lists assets. (SC-PROJECT-1)
  - `HP-2`: admin sets target languages → 200; intent persisted. (SC-LANG-1)
- **Edge cases considered:**
  - `EC-1`: viewer attempts create-project → 403, no row, no audit side effect.
  - `EC-2`: link asset owned by another principal → rejected (ADR-023).
- **Diagram:**

  ```mermaid
  flowchart LR
    C[web/mobile] -->|POST /api/orgs/{id}/projects| G[gateway proxy]
    G --> A[apps/api workspace routes]
    A --> M[org_scope guard]
    M --> H[handler] --> DB[(workspace_repo)]
    H --> AU[(audit_events)]
  ```

- **Handoff prompt:**
  > S-100-T3 — workspace API + audit. Docs: this ledger + plan §D2, ADR-018. Add
  > `apps/api/src/routes/workspace.rs` + dto; org/project/member/target-language
  > endpoints, role-guarded, audited, with linked asset summaries but no generic asset
  > lifecycle duplication. AC: audit on mutations, denied paths side-effect-free,
  > ≥90% cov, tests green. Stop after tests; do not start S-100-T4.

---

## S-100-T4 — Web console skeleton (Vite/React + gateway session auth)

- **Status:** [ ] Not started
- **Type:** Development (TS/web) · **Effort:** M
- **RRI:** 35 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-100-T3
- **Objective:** Stand up the first web console in `web/`: Vite + React app authenticating
  through the S-040 gateway (no token in browser), with a session-backed API client. (Plan §D4.)
- **Inputs:** `web/README.md` (reserved React line), S-040 gateway routes (`/auth/login`,
  cookie session, `/api/*`), mobile `client.ts` / `types.ts` as result/error
  contract references, ADR-024.
- **Outputs:**
  - `web/package.json`, `web/vite.config.ts`, `web/src/main.tsx` (app shell).
  - `web/src/api/gatewayClient.ts` (session-cookie fetch; no JWT handling; mirrors
    S-050 mobile error/result vocabulary where practical).
  - `web/src/auth/SessionProvider.tsx` (login redirect to gateway; session state).
  - Test setup (Vitest/RTL) + one smoke test; `npm run build` + `npm test` green.
- **Acceptance criteria:**
  - The app builds and renders an authenticated shell when a gateway session exists.
  - No access/refresh token is stored in the browser (ADR-024); auth is via gateway session.
  - Smoke test renders the shell; typecheck + build green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 6 → 1 | High |
  | F | 2 | 5 files | High |
  | D | 2 | new web app infra | High |
  | T | 2 | establishes web test harness | High |
  | A | 1 | minor stack/config choices | High |
  | K | 2 | gateway/session coupling | High |
  | P | 2 | client-internal (no new server surface) | High |
  | X | 3 | shell + client + auth + config | High |

  **Base 35 · penalties none · Final 35 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: existing gateway session → shell renders authenticated; `/api/*` calls succeed.
- **Edge cases considered:**
  - `EC-1`: no session → redirect to gateway `/auth/login` (no token handling in browser).
  - `EC-2`: session expired (401 from `/api/*`) → re-login, no crash.
- **Diagram:**

  ```mermaid
  flowchart LR
    B[browser web app] -->|/auth/login| G[S-040 gateway]
    G -->|sets session cookie| B
    B -->|/api/* + cookie| G --> A[apps/api]
  ```

- **Handoff prompt:**
  > S-100-T4 — web console skeleton. Docs: this ledger + plan §D4, ADR-024. Scaffold Vite+React
  > in `web/`, gateway-session client (no JWT), SessionProvider login redirect, Vitest smoke.
  > AC: builds, no token in browser, smoke green. Stop after build/test; do not start S-100-T5.

---

## S-100-T5 — Web org/project screens

- **Status:** [ ] Not started
- **Type:** Development (TS/web) · **Effort:** M
- **RRI:** 29 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-100-T4
- **Objective:** Build the org switcher, project list/detail (with linked assets and target
  languages), and member management screens against the S-100-T3 API.
- **Inputs:** S-100-T4 client/shell, S-100-T3 endpoints, BDD scenarios, S-055 `data-testid`
  convention.
- **Outputs:** `ProjectsScreen.tsx`, `ProjectDetailScreen.tsx`, `MembersScreen.tsx`;
  `data-testid`s (`projects-screen`, `project-detail-screen`, `members-screen`);
  component tests.
- **Acceptance criteria:**
  - Project list renders one row per project; detail shows linked assets + target languages. (SC-PROJECT-1, SC-LANG-1)
  - Member management reflects role-guarded actions (owner/admin can add; viewer cannot). (SC-MEMBER-1)
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
  | X | 2 | screens + test | High |

  **Base 29 · penalties none · Final 29 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: org with 2 projects → 2 rows; open detail → assets + languages shown. (SC-PROJECT-1)
- **Edge cases considered:**
  - `EC-1`: empty org → empty-state, no error. 
  - `EC-2`: viewer role → add-member control hidden/disabled (role-guarded UI). (SC-MEMBER-1)
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> projects
    projects --> detail: open project
    detail --> members: manage (if role permits)
    detail --> languages: edit target languages (if role permits)
  ```

- **Handoff prompt:**
  > S-100-T5 — web org/project screens. Docs: this ledger + plan §D4–§D6. Build Projects/
  > ProjectDetail/Members screens against S-100-T3 API, add data-testids, component tests.
  > AC: SC-PROJECT-1/SC-LANG-1/SC-MEMBER-1, role-guarded UI, tests+typecheck green.
  > Stop after tests; do not start S-100-T7.

---

## S-100-T6 — Mobile project surfaces

- **Status:** [ ] Not started
- **Type:** Development (TS/RN) · **Effort:** M
- **RRI:** 31 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** S-100-T3
- **Objective:** Add mobile project browsing: a project list and a project detail showing
  the project's linked assets, reachable from the authed tree.
- **Inputs:** `mobile/src/api/client.ts`, `mobile/src/api/types.ts`,
  `mobile/src/navigation/RootNavigator.tsx`, existing `AssetDetail` route, S-100-T3
  endpoints, S-055 `testID` convention, and any S-060-delivered asset list/upload
  surface.
- **Outputs:** `ProjectListScreen.tsx`, `ProjectDetailScreen.tsx`, nav route,
  `project-list-screen`/`project-detail-screen` testIDs, component tests; reuse
  `createGatewayClient`, `GatewayResult`, `auth.onSessionRotation`, and the existing
  `AssetDetail` navigation target.
- **Acceptance criteria:**
  - Project list renders the caller's org projects; detail lists linked assets.
  - Linked asset taps navigate to the existing asset detail route when an asset id is
    available; generic asset browsing/upload remains S-060-owned.
  - `session_expired` triggers logout and session rotation is persisted (unchanged
    transport contract).
  - testIDs present; `npm test` + typecheck green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 8 → 1 | High |
  | F | 2 | 4 files | High |
  | D | 2 | mobile UI + API integration | High |
  | T | 1 | mobile test harness exists | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | network/API coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 2 | screens + nav + test | High |

  **Base 31 · penalties none · Final 31 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: 2 projects → 2 cards; tap → detail lists linked assets.
- **Edge cases considered:**
  - `EC-1`: empty project list → empty-state, no error.
  - `EC-2`: 401 → `auth.logout()` (contract preserved).
- **Diagram:**

  ```mermaid
  flowchart LR
    H[home-screen] --> PL[project-list-screen]
    PL --> PD[project-detail-screen]
    PD --> AD[asset-detail-screen]
  ```

- **Handoff prompt:**
  > S-100-T6 — mobile project surfaces. Docs: this ledger + plan §D4/§D7. Add ProjectList/
  > ProjectDetail screens + nav + testIDs against S-100-T3 API, reusing the S-050 gateway
  > client/session rotation and existing AssetDetail route. AC: list+detail render,
  > 401→logout, tests+typecheck green. Stop after tests; do not start S-100-T7.

---

## S-100-T7 — E2E fixtures + runner wiring + docs/roadmap sync

- **Status:** [ ] Not started
- **Type:** Development (Node fixture) / ops / docs · **Effort:** S
- **RRI:** 24 → band **Low (0–25)** → **auto-execute** (present RRI + one-line summary, then proceed)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** S-100-T3, S-100-T5, S-100-T6
- **Objective:** Extend the existing S-055 mock-gateway with in-memory workspace
  fixtures, preserving health/handoff bootstrap routes and reusing S-060 asset
  fixtures if present, then author the web (Playwright) + mobile (Maestro) workspace
  flows and sync status docs.
- **Inputs:** `scripts/e2e-seed/mock-gateway-server.mjs`, S-100-T3 contracts, S-055 env,
  S-060 fixture state, `docs/plan/roadmap.md`.
- **Outputs:**
  - `/api/*` workspace routes in the mock-gateway (in-memory orgs/projects/members)
    that preserve existing health/handoff routes and share the S-060 asset store if
    available; add `node --test` coverage for the new workspace routes.
  - `web/e2e/projects.spec.ts` (Playwright) + `mobile/maestro/projects.yaml`.
  - Roadmap phase row set to its delivered status; follow-ups X-S-100-1/2/3 recorded; BDD mapping closed.
- **Acceptance criteria:**
  - The web + mobile workspace flows pass against the deterministic mock-gateway.
  - Existing S-055 handoff bootstrap behavior and any S-060 asset fixture behavior keep
    passing after workspace routes are added.
  - Status docs are internally consistent (no stale S-100 state); `make qa-docs` green.
  - X-S-100-1/2/3 recorded where future readers will find them.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 4 → 0 | High |
  | F | 2 | 4 files | High |
  | D | 1 | fixtures + orchestration | High |
  | T | 2 | seed fixtures have `node --test`; workspace route tests added here | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | process/fixture coupling | High |
  | P | 1 | dev/test + docs only | High |
  | X | 3 | fixtures + flows + docs | High |

  **Base 24 · penalties none · Final 24 → Low → auto-execute.**

- **Happy paths considered:**
  - `HP-1`: seeded org/project → web + mobile flows render and assert the BDD scenarios.
- **Edge cases considered:**
  - `EC-1`: `/api/*` workspace route without session → 401, no data leaked.
  - `EC-2`: non-member fixture session → org access denied in the flow. (SC-MEMBER-2)
- **Handoff prompt:**
  > S-100-T7 — E2E fixtures + docs sync. Docs: this ledger + plan + roadmap. Extend the
  > existing S-055 mock-gateway with workspace `/api/*` + `node --test`, preserving
  > handoff routes and reusing S-060 asset fixtures if present; add `web/e2e/projects.spec.ts`,
  > `mobile/maestro/projects.yaml`, sync roadmap + follow-ups. AC: flows pass, qa-docs
  > green, docs consistent. Stop after sync.

---

## Coverage contract

This ledger does **not** declare `Behavioral coverage contract: unit-v1` (the
automated `make qa-docs` per-task gate). Development tasks (S-100-T1…S-100-T6) still require
the standard `Unit coverage certification` + `Owner final verification` completion
record per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` before being marked `[x] Done`.
The BDD `.feature` scenarios (S-100-T0) are the behavioral source of truth from which each
task's `HP-#`/`EC-#` cases are derived.
