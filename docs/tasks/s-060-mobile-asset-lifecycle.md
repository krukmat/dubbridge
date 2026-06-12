# Tasks: S-060 - First-party Mobile Asset Lifecycle (Functional Surface + BDD/Maestro)

**Plan:** `docs/plan/s-060-mobile-asset-lifecycle.md`
**Roadmap phase:** `S-060` (mobile functional surface; sibling of `S-055`).
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-024, ADR-023, ADR-008, ADR-026.

> **Namespace.** This slice uses the **`T`** prefix (`T0`–`T6`). It is a different
> namespace from `S-055` (`V1`–`V8`). When referencing the screenshot suite, always
> fully qualify (`S-055 V7a`), never bare `V7a`.

> **RRI provenance.** Every RRI below was computed with `python3 scripts/rri.py`
> (platform `dubbridge`) at planning time, not by hand. Final RRI is recomputed at
> each task's presentation per the workflow. All tasks scored ≤ 55 → no further
> decomposition required (T3 is already split into T3a/T3b).

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 (BDD specs) ─┬─▶ T1 (backend GET /assets) ─┬─▶ T2 (mobile list) ──┐
                │                              └─▶ T4 (mock /api/*) ──┤
                └─▶ T3a (multipart + upload) ─▶ T3b (rights+finalize)─┤
                                                          │           │
                                                          └─▶ T4 ─────┤
                                                                      ▼
                                                          T5 (Maestro flows) ─▶ T6 (runner + docs sync)
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| T0 | BDD `.feature` specs + mapping | — | 13 | Low | S |
| T1 | Backend `GET /assets` list endpoint | T0 | 41 | Med-high | L |
| T2 | Mobile asset list rewired to real endpoint | T0, T1 | 29 | Moderate | M |
| T3a | Mobile multipart client + upload screen | T0 | 38 | Moderate | M |
| T3b | Mobile rights + finalize state machine | T3a | 42 | Med-high | L |
| T4 | Mock-gateway `/api/*` fixtures | T1, T3b | 31 | Moderate | M |
| T5 | Maestro flows + testIDs + screenshots | T2, T3b, T4 | 27 | Moderate | M |
| T6 | Runner integration + `npm run screenshots` + docs/roadmap sync | T5 | 24 | Low | S |

## Model resolution (capability → current vendor model)

Resolved against the active environment at planning time; reconfirm at presentation.

| Band | Codex | Claude Code | Thinking |
|---|---|---|---|
| Low (0–25) | `GPT-5.2-Codex` | `Claude Haiku 4.5` | Off |
| Moderate (26–40) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` | Off |
| Med-high (41–55) | `GPT-5.2-Codex` | `Claude Sonnet 4.6` (escalate to `Claude Opus 4.8` if it stalls) | On |

---

## T0 — BDD `.feature` specs + BDD⇄Maestro⇄unit mapping

- **Status:** [ ] Not started
- **Type:** Planning / docs (BDD authoring) · **Effort:** S
- **RRI:** 13 → band **Low (0–25)** → **auto-execute** (present RRI + one-line summary, then proceed)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** — (foundation; BDD-first)
- **Objective:** Author the Gherkin specs that define acceptance for the whole slice
  and the mapping convention (scenario ID ⇄ Maestro flow ⇄ `HP-#`/`EC-#`).
- **Inputs:** plan §D5; current screens; backend endpoint contracts.
- **Outputs:**
  - `mobile/bdd/asset-lifecycle.feature` (scenarios below).
  - `mobile/bdd/README.md` — the mapping table + convention.
- **Acceptance criteria:**
  - Each scenario has a stable ID and maps to one Maestro flow and ≥1 `HP-#`/`EC-#`.
  - Scenarios are written in behavioral terms (no implementation calls).
  - `make qa-docs` passes (no dangling references introduced).
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 1 → 0 | High |
  | F | 2 | 3 files | High |
  | D | 0 | docs/BDD authoring | High |
  | T | 2 | qa-docs validates references | High |
  | A | 0 | criteria + examples present | High |
  | K | 0 | no code coupling | High |
  | P | 0 | no API/security impact | High |
  | X | 2 | a few files | High |

  **Base 13 · penalties none · Final 13 → Low → auto-execute.**

- **BDD scenarios to author (Gherkin):**

  ```gherkin
  Feature: Mobile asset lifecycle

    Scenario: SC-LIST-1 Browse my assets
      Given I am an authenticated mobile user with at least one owned asset
      When I open the asset list
      Then I see each of my assets with its title and status

    Scenario: SC-LIST-2 Empty asset list
      Given I am an authenticated mobile user with no owned assets
      When I open the asset list
      Then I see a clear empty state and no error

    Scenario: SC-DETAIL-1 Open an asset from the list
      Given I am viewing my populated asset list
      When I tap an asset
      Then I see its detail with title, status, asset id, and uploader id

    Scenario: SC-INGEST-1 Upload a new asset
      Given I am an authenticated mobile user
      When I pick a file, submit valid rights, and finalize
      Then the asset is created and appears in my asset list

    Scenario: SC-INGEST-2 Upload rejected without rights
      Given I have uploaded a file but not submitted rights
      When I attempt to finalize
      Then finalization is rejected and I see a clear rights-required error
  ```

- **Handoff prompt:**
  > T0 — author BDD specs. Docs: this ledger + plan §D5. Create
  > `mobile/bdd/asset-lifecycle.feature` (SC-LIST-1/2, SC-DETAIL-1, SC-INGEST-1/2)
  > and `mobile/bdd/README.md` mapping table. AC: stable scenario IDs, mapped to
  > Maestro + HP/EC, qa-docs green. Stop after docs; do not start T1.

---

## T1 — Backend `GET /assets` list endpoint (S-010 read-surface extension)

- **Status:** [ ] Not started
- **Type:** Development (Rust) · **Effort:** L
- **RRI:** 41 → band **Med-high (41–55)** → **Plan + explicit acceptance criteria
  required before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
  (escalate to `Claude Opus 4.8` if it stalls) · thinking On
- **Depends on:** T0
- **Objective:** Add an ownership-scoped, ordered, paginated `GET /assets` endpoint to
  `apps/api` under the existing `assets:read` scope, backed by a new
  `crates/db` query. (Plan §D1, §D2.)
- **Inputs:** `apps/api/src/routes/ingestion.rs` (`read_routes`, lines 52-61),
  `apps/api/src/dto/ingestion.rs` (`AssetSummaryResponse`), `crates/db/src/asset_repo.rs`.
- **Outputs:**
  - `asset_repo::list_assets(pool, uploader_id, limit, offset)` returning ordered assets.
  - `GET /assets` route mounted in `read_routes` (`assets:read`, authenticated principal).
  - List response (array of `AssetSummaryResponse`, ordered `created_at DESC`).
  - Unit/integration tests for ownership scoping, ordering, and pagination bounds.
- **Acceptance criteria:**
  - `GET /assets` returns only the authenticated principal's assets, `created_at DESC`.
  - Page size is bounded (default 50, hard cap); out-of-range params fail safe.
  - Unauthenticated / wrong-scope requests are rejected by the existing middleware.
  - ≥90% line coverage on the new query + handler; all tests green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 6 → 1 | High |
  | F | 2 | 3 files | High |
  | D | 3 | anchor: `crates/db` (ADR-006/018) floor 3 | High |
  | T | 2 | area has route/repo tests | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | anchor: `crates/db` floor 3 (DB/HTTP) | High |
  | P | 4 | new public read endpoint + data visibility | High |
  | X | 2 | route + dto + repo | High |

  **Base 41 · penalties none · Final 41 → Med-high → plan+AC before approval.**

- **Happy paths considered:**
  - `HP-1`: authenticated `assets:read` caller with 3 owned assets → `200` with 3
    summaries ordered `created_at DESC`. (BDD SC-LIST-1)
  - `HP-2`: caller with 0 owned assets → `200` with `[]`. (BDD SC-LIST-2)
- **Edge cases considered:**
  - `EC-1`: caller without `assets:read` scope → `403` (middleware), no query runs.
  - `EC-2`: `limit` above the hard cap → clamped to the cap, not an unbounded scan.
  - `EC-3`: assets owned by a different principal are never returned (ownership scope).
- **Diagram:**

  ```mermaid
  flowchart LR
    C[mobile] -->|GET /api/assets| G[gateway proxy]
    G -->|Bearer + GET /assets| A[apps/api read_routes]
    A -->|authenticate_bearer + require_scope assets:read| H[list handler]
    H -->|list_assets uploader_id| DB[(crates/db asset_repo)]
    DB --> H --> A --> G --> C
  ```

- **Handoff prompt:**
  > T1 — add ownership-scoped `GET /assets`. Docs: this ledger + plan §D1/§D2,
  > ADR-023. Add `asset_repo::list_assets` (uploader-scoped, created_at DESC,
  > bounded limit/offset) and mount `GET /assets` in `read_routes` under
  > `assets:read`. AC: own-assets only, bounded page, ≥90% cov, tests green. Stop
  > after tests pass; do not start T2.

---

## T2 — Mobile asset list rewired to the real endpoint

- **Status:** [ ] Not started
- **Type:** Development (TS) · **Effort:** M
- **RRI:** 29 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** T0, T1
- **Objective:** Point `AssetListScreen` at the now-real `GET /api/assets`, retire the
  permanent `not_available` (404) branch, and add refresh + error-retry. Add the
  `asset-list-screen` testID.
- **Inputs:** `mobile/src/screens/AssetListScreen.tsx`, `mobile/__tests__/asset.screens.test.tsx`.
- **Outputs:** rewired list screen; refresh/retry affordance; updated component tests.
- **Acceptance criteria:**
  - A populated list renders one card per asset (title + status). (BDD SC-LIST-1)
  - An empty result renders the empty state (not the 404 "not available" copy). (SC-LIST-2)
  - `session_expired` still triggers logout; network error shows retry.
  - `asset-list-screen` testID present; `npm test` + `npm run typecheck` green.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 6 → 1 | High |
  | F | 1 | 2 files | High |
  | D | 2 | mobile UI + API integration | High |
  | T | 1 | `asset.screens.test.tsx` exists | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | network/API coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 2 | screen + test | High |

  **Base 29 · penalties none · Final 29 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: gateway returns 3 assets → 3 cards rendered, tappable to detail. (SC-LIST-1)
- **Edge cases considered:**
  - `EC-1`: empty array → empty state, no error copy. (SC-LIST-2)
  - `EC-2`: `401` → `auth.logout()` (unchanged contract).
  - `EC-3`: network error → retry affordance, no crash.
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> loading
    loading --> ready: assets.length > 0
    loading --> empty: assets.length == 0
    loading --> error: network/http
    error --> loading: retry
    ready --> [*]: tap -> AssetDetail
  ```

- **Handoff prompt:**
  > T2 — rewire `AssetListScreen` to real `GET /api/assets`. Docs: this ledger +
  > plan §D1. Remove permanent 404 `not_available` path, keep empty/error states,
  > add refresh/retry + `asset-list-screen` testID. AC: SC-LIST-1/2, 401→logout,
  > tests+typecheck green. Stop after tests; do not start T3.

---

## T3a — Mobile multipart client method + upload (ingest-create) screen

- **Status:** [ ] Not started
- **Type:** Development (TS) · **Effort:** M
- **RRI:** 38 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** T0
- **Objective:** Add a multipart-capable client method and a first upload screen that
  picks a file and creates an ingestion session (`POST /ingest`), returning an
  `ingest_token`. (Plan §D3.)
- **Inputs:** `mobile/src/api/client.ts`, `mobile/src/navigation/RootNavigator.tsx`,
  `expo-document-picker`, backend `POST /ingest` contract (`title` + `file`).
- **Outputs:**
  - `client.postMultipart<T>(path, sessionRef, formData)` (no JSON content-type;
    session header carried; same error mapping as `request`).
  - `UploadScreen.tsx` step 1: pick file → `POST /ingest` → hold `ingest_token`.
  - Route registered; entry point from Home/List; `upload-screen` + `upload-pick-file` testIDs.
  - Client unit tests for the multipart path (header omission, session header, errors).
- **Acceptance criteria:**
  - `postMultipart` sends `FormData` without a JSON content-type and with
    `X-Dubbridge-Session`; maps `401/403/network` like the JSON client.
  - Picking a file and submitting yields a valid `ingest_token` held in screen state.
  - `npm test` + `npm run typecheck` green; ≥90% coverage on the new client method.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 1 | raw CC 8 → 1 | High |
  | F | 2 | 4 files | High |
  | D | 3 | multipart + upload state | High |
  | T | 2 | `api.client.test.ts` exists | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | network/IO coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 3 | client + screen + nav + test | High |

  **Base 38 · penalties none · Final 38 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: pick file + title → `POST /ingest` → `201` with `ingest_token` retained.
- **Edge cases considered:**
  - `EC-1`: user cancels the file picker → no request, screen stays idle.
  - `EC-2`: `413` (oversized) → clear "file too large" message, no crash.
  - `EC-3`: `401` during create → `auth.logout()` (transport contract preserved).
- **Diagram:**

  ```mermaid
  sequenceDiagram
    participant U as UploadScreen
    participant C as gateway client
    participant G as gateway/api
    U->>C: postMultipart(/api/ingest, FormData{title,file})
    C->>G: POST /ingest (multipart, X-Dubbridge-Session)
    G-->>C: 201 { ingest_token }
    C-->>U: ingest_token (-> T3b)
  ```

- **Handoff prompt:**
  > T3a — multipart client + upload step 1. Docs: this ledger + plan §D3. Add
  > `client.postMultipart`, `UploadScreen` file-pick → `POST /ingest`, register
  > route + testIDs. AC: multipart header handling, ingest_token retained, errors
  > mapped, ≥90% cov, tests+typecheck green. Stop after step-1 works; do not start T3b.

---

## T3b — Mobile rights + finalize 3-step state machine

- **Status:** [ ] Not started
- **Type:** Development (TS) · **Effort:** L
- **RRI:** 42 → band **Med-high (41–55)** → **Plan + explicit acceptance criteria
  required before approval; thinking On.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
  (escalate to `Claude Opus 4.8` if it stalls) · thinking On
- **Depends on:** T3a
- **Objective:** Complete the ingestion flow: submit rights (`POST /ingest/{token}/rights`)
  and finalize (`POST /ingest/{token}/finalize`), with a 3-step state machine
  (uploaded → rights-recorded → finalized), per-step error handling, session-rotation
  handling, and success → navigate to the new asset's detail/list.
- **Inputs:** `UploadScreen.tsx` (from T3a), `client.ts`, backend rights/finalize contracts,
  `SubmitRightsRequest` shape (`owner`, `license_type`, `source_type`, `proof_reference`).
- **Outputs:** rights form + finalize step; state machine; success navigation;
  `upload-submit-rights` + `upload-finalize` testIDs; component tests for all transitions.
- **Acceptance criteria:**
  - Full flow: pick → `POST /ingest` → rights → finalize → `201` asset → navigate so
    the asset is visible in the list. (BDD SC-INGEST-1)
  - Finalize without rights → backend `422` surfaced as a clear rights-required error;
    flow does not advance. (BDD SC-INGEST-2)
  - `X-Dubbridge-Session` rotation from any step is persisted via `onSessionRotation`.
  - `npm test` + `npm run typecheck` green; ≥90% coverage on the flow logic.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 14 → 2 | High |
  | F | 2 | 3 files | High |
  | D | 3 | 3-step orchestration + errors | High |
  | T | 2 | `asset.screens.test.tsx` exists | High |
  | A | 0 | criteria + examples present | High |
  | K | 3 | network/transaction coupling | High |
  | P | 2 | client-internal behavior | High |
  | X | 3 | screen + client + tests | High |

  **Base 42 · penalties none · Final 42 → Med-high → plan+AC before approval.**

- **Happy paths considered:**
  - `HP-1`: uploaded → valid rights → finalize → `201` asset → list shows it. (SC-INGEST-1)
- **Edge cases considered:**
  - `EC-1`: finalize before rights → `422` rights-required, flow blocked. (SC-INGEST-2)
  - `EC-2`: expired ingest session (`410 Gone`) → clear "session expired, restart" message.
  - `EC-3`: session rotation header mid-flow → rotated `session_ref` persisted, flow continues.
- **Diagram:**

  ```mermaid
  stateDiagram-v2
    [*] --> uploaded: POST /ingest -> ingest_token
    uploaded --> rights_recorded: POST /ingest/{token}/rights (200)
    uploaded --> rights_error: 4xx
    rights_recorded --> finalized: POST /ingest/{token}/finalize (201)
    rights_recorded --> finalize_blocked: 422 rights-required
    rights_recorded --> expired: 410 gone
    finalized --> [*]: navigate to AssetDetail/List
  ```

- **Handoff prompt:**
  > T3b — rights + finalize state machine. Docs: this ledger + plan §D3, ADR-008.
  > Add rights form + finalize to `UploadScreen`, 3-step state machine with per-step
  > errors + session-rotation persistence, success→detail/list nav, testIDs. AC:
  > SC-INGEST-1/2, 410 handling, rotation persisted, ≥90% cov, tests+typecheck green.
  > Stop after tests; do not start T4.

---

## T4 — Mock-gateway `/api/*` fixtures (Maestro E2E backend)

- **Status:** [ ] Not started
- **Type:** Development (Node fixture) · **Effort:** M
- **RRI:** 31 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** T1, T3b
- **Objective:** Extend `mock-gateway-server.mjs` with an in-memory asset store and
  `/api/*` routes so the authenticated data screens and the full ingestion narrative
  are exercisable in Maestro without Postgres. (Plan §D4.)
- **Inputs:** `scripts/e2e-seed/mock-gateway-server.mjs`, T1 list contract, T3b ingest usage.
- **Outputs:**
  - `GET /api/assets`, `GET /api/assets/{id}` served from an in-memory store (session-gated).
  - `POST /api/ingest`, `/rights`, `/finalize` driving a fixture asset into the store.
  - `scripts/e2e-seed/mock-gateway-server.test.mjs` covering list, detail, and the
    upload→list visibility narrative.
- **Acceptance criteria:**
  - `/api/*` routes require a resolved `X-Dubbridge-Session`; missing → `401`.
  - After the ingest sequence, the new asset appears in `GET /api/assets`.
  - Shapes match the real DTO (`id`, `title`, `uploader_id`, `status`, timestamps).
  - `node --test` for the mock-gateway test passes.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 2 | raw CC 12 → 2 | High |
  | F | 1 | 2 files | High |
  | D | 2 | in-memory store + routing | High |
  | T | 2 | sibling fixtures have tests | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | contained side effects | High |
  | P | 1 | dev/test fixture only | High |
  | X | 2 | server + test | High |

  **Base 31 · penalties none · Final 31 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: seeded store + valid session → `GET /api/assets` returns the fixtures.
  - `HP-2`: ingest sequence → asset becomes visible in the subsequent list.
- **Edge cases considered:**
  - `EC-1`: `/api/*` without session header → `401`, no data leaked.
  - `EC-2`: `GET /api/assets/{unknown}` → `404`.
- **Handoff prompt:**
  > T4 — mock-gateway `/api/*` fixtures. Docs: this ledger + plan §D4. Add in-memory
  > asset store + `GET /api/assets`, `GET /api/assets/{id}`, ingest routes (session-
  > gated), and a `node --test`. AC: 401 without session, upload→list visibility,
  > DTO-shaped, test green. Stop after tests; do not start T5.

---

## T5 — Maestro flows + testIDs + screenshots

- **Status:** [ ] Not started
- **Type:** Development (Maestro/config) · **Effort:** M
- **RRI:** 27 → band **Moderate (26–40)** → **Confirm tests exist in the area.**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6` · thinking Off
- **Depends on:** T2, T3b, T4
- **Objective:** Author Maestro flows for the list, detail, and ingestion BDD
  scenarios, each capturing a screenshot and asserting the T2/T3 testIDs, running
  against the T4 extended mock-gateway. (Plan §D5/§D6.)
- **Inputs:** T0 `.feature` scenarios, testIDs from T2/T3, T4 mock-gateway,
  S-055 env (`screenshot-env.sh`, ports, ANR guard pattern).
- **Outputs:**
  - `mobile/maestro/asset-list.yaml`, `asset-detail.yaml`, `asset-ingestion.yaml`.
  - Screenshots: `03_asset_list`, `04_asset_detail`, `05_upload`.
  - README mapping each flow back to its BDD scenario ID.
- **Acceptance criteria:**
  - Each flow reaches its target screen via testID and captures its screenshot.
  - Flows reuse the S-055 ANR guard + `--env` seed convention; no port collisions.
  - Each flow file names the BDD scenario it satisfies (SC-LIST-1/2, SC-DETAIL-1, SC-INGEST-1/2).
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 2 → 0 | High |
  | F | 3 | 6 files | High |
  | D | 1 | declarative flows | High |
  | T | 2 | E2E config, no unit | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | emulator/process coupling | High |
  | P | 1 | test artifacts only | High |
  | X | 3 | flows + screens + README | High |

  **Base 27 · penalties none · Final 27 → Moderate → confirm tests exist.**

- **Happy paths considered:**
  - `HP-1`: authed bootstrap → list flow reaches `asset-list-screen`, captures `03_asset_list`.
  - `HP-2`: ingestion flow drives pick→rights→finalize and lands on the new asset. (SC-INGEST-1)
- **Edge cases considered:**
  - `EC-1`: empty-list run asserts the empty state, not a false-positive card. (SC-LIST-2)
  - `EC-2`: ANR dialog mid-flow → guarded by the repeated `isn't responding` poll.
- **Diagram:**

  ```mermaid
  flowchart LR
    seed[issue-handoff] --> deeplink[openLink dubbridge://auth/callback]
    deeplink --> home[home-screen]
    home --> list[asset-list-screen + 03_asset_list]
    list --> detail[asset-detail-screen + 04_asset_detail]
    home --> upload[upload-screen + 05_upload]
  ```

- **Handoff prompt:**
  > T5 — Maestro flows for list/detail/ingestion. Docs: this ledger + plan §D5/§D6,
  > S-055 README. Author asset-list/detail/ingestion `.yaml`, assert testIDs, capture
  > `03/04/05` screenshots against T4 mock-gateway, reuse ANR guard + `--env` seed.
  > AC: each flow reaches its screen + names its BDD scenario. Stop after capture;
  > do not start T6.

---

## T6 — Runner integration + `npm run screenshots` + docs/roadmap sync

- **Status:** [ ] Not started
- **Type:** Ops / docs · **Effort:** S
- **RRI:** 24 → band **Low (0–25)** → **auto-execute** (present RRI + one-line summary, then proceed)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** T5
- **Objective:** Fold the new flows into the S-055 runner, expose a one-command
  `npm run screenshots`, and sync status documents (this ledger, roadmap, plan,
  S-055 cross-references, open follow-ups X-P3F-1/X-P3F-2).
- **Inputs:** S-055 V7/V8 runner contract, `mobile/package.json`, `mobile/maestro/README.md`,
  `docs/plan/roadmap.md`.
- **Outputs:** runner that starts mock-oauth + mock-gateway + Metro + adb reverse and
  runs the full flow set; `npm run screenshots`; updated README; roadmap slice row set
  to its delivered status; open follow-ups recorded.
- **Acceptance criteria:**
  - One command brings up the stack and runs all five flows, archiving screenshots.
  - Status docs are internally consistent (no stale S-060 state) before reporting done.
  - X-P3F-1 / X-P3F-2 are recorded where future readers will find them.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 4 → 0 | High |
  | F | 2 | 4 files | High |
  | D | 1 | orchestration script | High |
  | T | 2 | runner smoke only | High |
  | A | 0 | criteria + examples present | High |
  | K | 2 | process orchestration | High |
  | P | 1 | tooling only | High |
  | X | 3 | runner + manifest + docs | High |

  **Base 24 · penalties none · Final 24 → Low → auto-execute.**

- **Handoff prompt:**
  > T6 — runner + npm script + docs sync. Docs: this ledger + plan + S-055 V7/V8.
  > Integrate the new flows into the runner, add `npm run screenshots`, sync roadmap +
  > README + follow-ups. AC: one-command full run, status docs consistent. Stop after
  > docs are synced.

---

## Coverage contract

This ledger does **not** declare `Behavioral coverage contract: unit-v1` (the
automated `make qa-docs` gate). Development tasks (T1, T2, T3a, T3b, T4) still require
the standard `Unit coverage certification` + `Owner final verification` completion
record per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` before being marked `[x] Done`.
The BDD `.feature` scenarios (T0) are the behavioral source of truth from which each
task's `HP-#`/`EC-#` cases are derived.
