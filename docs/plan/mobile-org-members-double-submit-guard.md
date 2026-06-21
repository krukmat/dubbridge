---
type: Plan
title: "Mobile org members double-submit guard"
status: closed
owner: "Codex"
date: "2026-06-21"
linked_task_file: "docs/tasks/mobile-org-members-double-submit-guard.md"
---

# Objective

Prevent duplicate member-add submissions in the mobile organization members screen so a rapid double tap cannot emit duplicate write requests or duplicate audit events.

# Scope

- `mobile/src/screens/OrganizationMembersScreen.tsx`
- `mobile/__tests__/organization.screens.test.tsx`

# Design decisions

- Add a narrow client-side submit guard local to the member-add action.
- Keep the fix in the mobile screen layer; do not change backend behavior in this task.
- Add a regression test that proves repeated presses while the first request is in flight still produce a single POST.

# Dependencies

- Existing `Button` disabled behavior in mobile tests.
- Existing org-member POST contract in the mobile gateway client.

# Outcome

- Added a client-side in-flight guard using a ref plus visible loading/disabled state on the add-member button.
- Added a regression test that proves two rapid presses before request resolution still emit exactly one POST.
- Verified with `cd mobile && npm test -- --runTestsByPath __tests__/organization.screens.test.tsx --runInBand`.
