---
type: TaskList
title: "Tasks: Mobile Playback Summary ID Polish"
status: done
plan: docs/plan/mobile-playback-summary-id-polish.md
---
# Tasks: Mobile Playback Summary ID Polish

Plan: `docs/plan/mobile-playback-summary-id-polish.md`.

RRI computed on 2026-06-25 with:
`python3 scripts/rri.py --platform rn --touches mobile/src/screens/ReviewDetailScreen.tsx --touches mobile/src/screens/AssetDetailScreen.tsx --touches mobile/__tests__/ReviewDetailScreen.test.tsx --touches mobile/__tests__/asset.screens.test.tsx --cc 8 --T 1 --A 0 --X 3 --D 1 --K 2 --P 0 --json`

## Task summary

| ID | Title | RRI -> band | Effort | Status | Depends on |
|---|---|---|---|---|---|
| MPS-T1 | Playback-adjacent summary ID polish | 23 -> Low | S | Done | - |

---

## MPS-T1 - Playback-adjacent summary ID polish

- **Status:** Done
- **Effort:** S
- **RRI:** 23 -> Low (0-25)
- **Depends on:** -
- **Affected:** `mobile/src/screens/ReviewDetailScreen.tsx`,
  `mobile/src/screens/AssetDetailScreen.tsx`,
  `mobile/__tests__/ReviewDetailScreen.test.tsx`,
  `mobile/__tests__/asset.screens.test.tsx`

### Objective

Make the playback-adjacent summary rows resilient to long identifiers without
changing the underlying values or the surrounding playback flow.

### Inputs

- `docs/audit/mobile-design-md-playback-audit.md`
- `DESIGN.md`
- `mobile/src/format/index.ts`
- Review/detail screen source and tests

### Outputs

- Summary rows in the two playback-adjacent screens render full identifiers with
  single-line tail ellipsis when space is tight.
- Screen tests cover the long-ID presentation contract.

### Acceptance criteria

- Review detail summary rows remain readable with long task/asset IDs.
- Asset detail metadata rows remain readable with long asset/uploader IDs.
- Full identifier strings remain the underlying rendered values.
- Existing behavior and test IDs remain unchanged.

### Happy paths considered

- **HP-1:** Review detail renders long task/asset IDs without overlapping badges or
  decision controls.
- **HP-2:** Asset detail renders long asset/uploader IDs without pushing adjacent
  metadata into an incoherent layout.

### Edge cases considered

- **EC-1:** Full identifier strings remain present in the rendered tree; the UI
  relies on native tail ellipsis rather than manual token cutting.
- **EC-2:** Existing short IDs continue to render normally with no layout change to
  the surrounding playback panels.

### Handoff prompt

Implement the narrow playback-adjacent ID polish follow-up from the S-205 audit.
Update only `ReviewDetailScreen` and `AssetDetailScreen` so long summary-row IDs
use mobile-friendly single-line tail ellipsis while keeping the full identifier
string intact. Update the focused screen tests accordingly.

### Completion notes

- Updated `ReviewDetailScreen` summary values to render `formatId(...)` with
  `numberOfLines={1}` and `ellipsizeMode="tail"`.
- Updated `AssetDetailScreen` metadata values with the same single-line tail
  ellipsis treatment.
- Added focused tests proving the full identifier string remains present while the
  summary rows opt into native tail ellipsis behavior.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | review detail renders long task/asset IDs without overlapping surrounding playback-adjacent UI | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-1b: summary-row ids use single-line tail ellipsis without dropping the full value` | passed |
| HP-2 | Happy path | asset detail renders long asset/uploader IDs without incoherent metadata layout treatment | `mobile/__tests__/asset.screens.test.tsx::HP-2b: long metadata ids use single-line tail ellipsis without dropping the full value` | passed |
| EC-1 | Edge case | full identifier strings remain in the rendered tree while the UI relies on native tail ellipsis | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-1b: summary-row ids use single-line tail ellipsis without dropping the full value`; `mobile/__tests__/asset.screens.test.tsx::HP-2b: long metadata ids use single-line tail ellipsis without dropping the full value` | passed |
| EC-2 | Edge case | existing short-ID playback-adjacent screens remain behaviorally unchanged | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-1: approve posts a scoped decision, rotates session, and reveals publish`; `mobile/__tests__/asset.screens.test.tsx::loads asset detail and shows the available S1 summary` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-25`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `cd mobile && npm test -- --runInBand __tests__/ReviewDetailScreen.test.tsx __tests__/asset.screens.test.tsx`; `cd mobile && npm run typecheck`
