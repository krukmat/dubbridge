---
type: TaskList
title: "Tasks: S-105 - Mobile Workspace Parity and Web Console Retirement"
status: closed
slice: S-105
plan: docs/plan/s-105-mobile-workspace-parity.md
---
# Tasks: S-105 - Mobile Workspace Parity and Web Console Retirement

**Plan:** `docs/plan/s-105-mobile-workspace-parity.md`
**Roadmap phase:** `S-105` (depends on completed `S-100`; precedes mobile client
surfaces in `S-110` and `S-160`).
**Governing ADRs:** ADR-024, ADR-029.
**Behavioral coverage contract:** unit-v1

## Status legend

- [ ] Not started
- [~] In progress
- [x] Done
- [-] Cancelled / superseded

## Dependency order

```text
S-105-T0 -> S-105-T1 -> S-105-T2 -> S-110-T5 -> S-105-T3
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| S-105-T0 | Architecture, roadmap, and BDD contract | S-100 | 44 | Med-high | L |
| S-105-T1 | Mobile organization context | S-105-T0 | 41 | Med-high | L |
| S-105-T2 | Mobile workspace parity | S-105-T1 | 41 | Med-high | L |
| S-105-T3 | Retire authenticated web console | S-105-T2, S-110-T5 | 52 | Med-high | L |

## S-105-T0 - Architecture, roadmap, and BDD contract

- **Status:** [x] Done — 2026-06-13
- **Type:** Architecture / planning
- **RRI:** 44 - Med-high
- **Objective:** Declare mobile as the canonical authenticated UI, insert S-105 in
  the roadmap, and change BDD mappings from web-first to executable-surface evidence.
- **Outputs:** this plan and ledger; updated architecture, roadmap, BDD conventions,
  and S-110/S-160 task dependencies.
- **Acceptance criteria:**
  - The retirement boundary excludes future public web/player surfaces.
  - Web remains present until S-105-T3's evidence gate passes.
  - S-110 and S-160 no longer require authenticated web tasks.
  - `make qa-docs` passes after final synchronization.

## S-105-T1 - Mobile organization context

- **Status:** [x] Done — 2026-06-13
- **Type:** Development (TS/RN)
- **RRI:** 41 - Med-high
- **Objective:** Add organization list/create/select and make Projects reachable
  from Home with an API-derived organization ID.
- **Inputs:** `GET/POST /api/orgs`, mobile gateway client, authenticated navigator.
- **Outputs:** `OrganizationListScreen`, Home/navigation wiring, component tests.
- **Acceptance criteria:**
  - Organizations returned by the gateway render and selection opens Projects.
  - Creating a non-empty organization posts it and opens its Projects.
  - Loading, empty, forbidden/network failure, and session expiry are handled.
  - Session rotation is persisted on successful requests.
- **Happy paths considered:**
  - `HP-1`: list organizations -> select one -> ProjectList receives its real ID.
  - `HP-2`: create organization -> returned organization becomes the selected context.
- **Edge cases considered:**
  - `EC-1`: no organizations -> explicit empty state with create action available.
  - `EC-2`: blank organization name -> no request and inline validation error.
  - `EC-3`: session expired -> logout; no authenticated content remains.
- **Unit coverage certification:** `organization.screens.test.tsx` covers list,
  create, blank input, failures, selection, session expiry, and session rotation.
- **Owner final verification:** mobile tests/typecheck passed; workspace API tests
  passed with the new `viewer_role` response contract.

## S-105-T2 - Mobile workspace parity

- **Status:** [x] Done — 2026-06-13
- **Type:** Development (TS/RN)
- **RRI:** 41 - Med-high
- **Objective:** Add organization member viewing/creation and show target languages
  in mobile project detail.
- **Inputs:** `GET/POST /api/orgs/{id}/members`; project detail response.
- **Outputs:** `OrganizationMembersScreen`, navigation wiring, target-language section,
  component tests.
- **Acceptance criteria:**
  - Members render for the selected organization.
  - Owner/Admin can add a member; Editor/Reviewer/Viewer do not see mutation controls.
  - Project detail displays target languages and appropriate empty state.
  - Session expiry logs out and failed requests never show stale success UI.
- **Happy paths considered:**
  - `HP-1`: owner adds a member -> returned member appears in the list.
  - `HP-2`: project detail shows linked assets and target-language pairs.
- **Edge cases considered:**
  - `EC-1`: Viewer/Editor/Reviewer -> add-member controls absent.
  - `EC-2`: project has no target languages -> explicit empty state.
  - `EC-3`: member request fails -> error state and no optimistic row.
- **Unit coverage certification:** organization/member tests cover owner/admin
  controls and viewer suppression; project tests cover target languages and empty state.
- **Owner final verification:** mobile tests/typecheck passed.

## S-105-T3 - Retire authenticated web console

- **Status:** [x] Done — 2026-06-13
- **Type:** Development / removal / docs
- **RRI:** 52 - Med-high
- **Objective:** Remove the dormant authenticated web console and duplicate frontend
  tooling after mobile parity and compliance are certified.
- **Inputs:** completed S-105-T2 and S-110-T5 evidence.
- **Outputs:** removal of `web/`; updated CI/docs/BDD references; backend or Maestro
  evidence replacing web-labelled coverage.
- **Acceptance criteria:**
  - Mobile tests and typecheck pass before deletion.
  - Workspace and compliance UI flows are mapped to Maestro/mobile unit evidence.
  - No canonical document describes `web/` as an operational authenticated surface.
  - No build or QA command references the removed console.
  - `make qa-docs` passes.
- **Completion evidence:** `web/` and its Vitest/Playwright tooling were removed;
  workspace and compliance UI evidence now maps to mobile tests/Maestro, while
  authorization and fail-closed gates remain backend-certified.
- **Owner final verification:** mobile tests/typecheck, mock-gateway tests, Rust
  workspace tests, YAML parsing, shell syntax, stale-reference scan, and `make qa-docs`.

## Reflection strategy

Each development/removal task is Med-high and requires three Draft -> Critique ->
Revise passes. Pass 1 checks API contracts and navigation; Pass 2 checks fail-closed
session/error behavior and role visibility; Pass 3 checks coverage, stale references,
and maintenance reduction before certification.

All three passes completed for T1-T3. No web component was ported mechanically;
behavior was rebuilt on the existing React Native session/navigation patterns.

## Agent handoff prompt

> Implement S-105 in dependency order. Preserve existing user changes. Reuse the
> mobile gateway/session patterns, do not move authorization into UI, and do not
> delete `web/` until mobile workspace plus S-110 compliance evidence is green.
> Recompute RRI from actual touched files, record reflection and unit certification,
> synchronize roadmap/BDD/S-110/S-160, and stop after S-105-T3 verification.
