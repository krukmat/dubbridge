---
type: TaskList
title: "Tasks: Mobile Playback Overlay Contrast"
status: done
plan: docs/plan/mobile-playback-overlay-contrast.md
---
# Tasks: Mobile Playback Overlay Contrast

Plan: `docs/plan/mobile-playback-overlay-contrast.md`.

RRI computed on 2026-06-25 with:
`python3 scripts/rri.py --platform rn --touches mobile/src/components/StateView.tsx --touches mobile/src/components/VideoPlayer.tsx --touches mobile/__tests__/components.test.tsx --cc 8 --T 1 --A 0 --X 2 --D 2 --K 2 --P 0 --json`

## Task summary

| ID | Title | RRI -> band | Effort | Status | Depends on |
|---|---|---|---|---|---|
| MPC-T1 | Dark playback overlay contrast fix | 25 -> Low | S | Done | - |

---

## MPC-T1 - Dark playback overlay contrast fix

- **Status:** Done
- **Effort:** S
- **RRI:** 25 -> Low (0-25)
- **Depends on:** -
- **Affected:** `mobile/src/components/StateView.tsx`,
  `mobile/src/components/VideoPlayer.tsx`,
  `mobile/__tests__/components.test.tsx`

### Objective

Make playback loading/error/end overlays legible on the dark `VideoPlayer` shell
without changing playback behavior or any non-player screen surface.

### Inputs

- `docs/audit/mobile-design-md-playback-audit.md`
- `DESIGN.md`
- `mobile/src/components/StateView.tsx`
- `mobile/src/components/VideoPlayer.tsx`
- `mobile/__tests__/components.test.tsx`

### Outputs

- Additive `StateView` appearance prop.
- `VideoPlayer` overlay wired to the dark appearance.
- Updated component tests covering the new contrast treatment.

### Acceptance criteria

- Playback overlays render with light-on-dark foreground colors.
- Default `StateView` appearance remains unchanged for the rest of the app.
- Existing `testID`s and retry wiring remain unchanged.
- Focused component tests and typecheck pass.

### Happy paths considered

- **HP-1:** Loading playback in `VideoPlayer` shows the existing overlay copy with
  a dark-surface-safe foreground treatment.
- **HP-2:** Error playback in `VideoPlayer` still shows retry controls and uses the
  same dark-surface-safe foreground treatment.

### Edge cases considered

- **EC-1:** Screens that use `StateView` outside the player keep the existing light
  appearance by default.
- **EC-2:** Overlay state transitions (`loading` -> `ready` -> `end` / `error`)
  remain unchanged; only the appearance changes.

### Handoff prompt

Implement the narrow playback polish follow-up from the S-205 audit. Add an
additive appearance prop to `StateView`, use it only in `VideoPlayer` for the dark
overlay shell, and update component tests to prove default `StateView` styling is
unchanged while playback overlays use light-on-dark text.

### Completion notes

- Added `appearance?: "default" | "inverse"` to `StateView`, defaulting to the
  existing light-surface behavior.
- Updated `VideoPlayer` to render overlay `StateView`s with `appearance="inverse"`.
- Added focused component tests proving:
  - inverse `StateView` uses light-on-dark text colors;
  - `VideoPlayer` loading overlays now inherit the inverse foreground treatment.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | loading playback overlay shows dark-surface-safe foreground treatment | `mobile/__tests__/components.test.tsx::HP-5: renders the expo-video shell inside the tokenized player container` | passed |
| HP-2 | Happy path | error playback overlay keeps retry behavior and dark-surface-safe foreground treatment | `mobile/__tests__/components.test.tsx::HP-6: binds an error overlay through StateView without embedding reducer logic` | passed |
| EC-1 | Edge case | non-player `StateView` usage keeps the default light appearance | `mobile/__tests__/components.test.tsx::loading renders the provided message` | passed |
| EC-2 | Edge case | overlay state transitions remain behaviorally unchanged while only appearance changes | `mobile/__tests__/components.test.tsx::HP-7: statusChange ready hides the overlay and loading shows it again`; `mobile/__tests__/components.test.tsx::HP-8: playToEnd shows the end overlay after playback is ready` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-25`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cd mobile && npm test -- --runInBand __tests__/components.test.tsx`; `cd mobile && npm run typecheck`
