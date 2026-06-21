---
type: TaskList
title: "Mobile org members double-submit guard"
status: closed
owner: "Codex"
date: "2026-06-21"
linked_plan_file: "docs/plan/mobile-org-members-double-submit-guard.md"
---

# Tasks

- [x] `MOG-DSG-1` Guard member creation against repeated taps in `OrganizationMembersScreen`
  - Status: Complete
  - Effort: S
  - Acceptance criteria:
    - The add-member action ignores repeated presses while the first request is still pending.
    - The add-member control reflects the pending state so the user cannot intentionally trigger duplicates.
    - Existing add-member behavior still succeeds on a normal single submit.
    - A regression test proves only one POST is sent when the button is pressed multiple times before the request resolves.
  - Happy path examples:
    - `HP-1`: owner enters a valid subject id, taps add once, and the returned member row appears.
  - Edge case examples:
    - `EC-1`: owner taps add repeatedly before the first response resolves and only one network request is sent.
    - `EC-2`: blank subject id still fails closed without sending a request.
  - Agent handoff prompt: Add a submit guard and regression test for repeated taps in the mobile organization members screen without changing backend contracts.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | owner enters a valid subject id, taps add once, and the returned member row appears | `mobile/__tests__/organization.screens.test.tsx::HP-1 members: owner adds a member and the returned row appears` | passed |
| EC-1 | Edge case | owner taps add repeatedly before the first response resolves and only one network request is sent | `mobile/__tests__/organization.screens.test.tsx::regression: multiple rapid presses of add button only trigger one request` | passed |
| EC-2 | Edge case | blank subject id still fails closed without sending a request | `mobile/__tests__/organization.screens.test.tsx::EC-2 members: blank subject id is rejected without sending a request` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-21`
- Statement: I verified every happy path and edge case defined for this task has test evidence that replicates the expected behavior for the mobile add-member flow.
- Commands run: `cd mobile && npm test -- --runTestsByPath __tests__/organization.screens.test.tsx --runInBand`
