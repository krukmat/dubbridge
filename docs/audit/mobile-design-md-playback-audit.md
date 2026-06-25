---
type: Audit
title: "Audit: S-205 playback surfaces against DESIGN.md"
status: open
---
# Audit: S-205 playback surfaces against DESIGN.md

Date: `2026-06-25`
Scope: `ReviewDetailScreen`, `AssetDetailScreen`, `PlaybackStateView`,
`VideoPlayer`, and screenshots `18_asset_detail_playback.png` /
`19_review_detail_playback.png`.

## Verdict

The playback surfaces are broadly aligned with `DESIGN.md` and the underlying
S-115/S-190 mobile system. The overall hierarchy is correct:

- playback sits inside existing `Panel` surfaces rather than inventing a second UI
  language;
- the `VideoPlayer` shell uses the same radius/border vocabulary as the rest of
  the app;
- the playback entry point remains operationally clear;
- playback failure states preserve the surrounding decision/compliance actions.

No redesign is needed.

Two low-risk follow-up patches are recommended before treating the playback
surfaces as fully polished.

## Findings

### F1 - Dark playback overlays inherit light-surface text colors

Severity: Medium

`VideoPlayer` renders `StateView` inside a dark media overlay, but `StateView`
always uses light-surface typography colors (`ink900` / `ink500`). In the current
screenshots, the loading message inside the player is visibly muted against the
dark background rather than crisp and intentionally readable.

Evidence:

- Code: `mobile/src/components/VideoPlayer.tsx` renders `StateView` inside a dark
  overlay.
- Code: `mobile/src/components/StateView.tsx` hardcodes `ink900` and `ink500`.
- Screenshot: `mobile/artifacts/screenshots/18_asset_detail_playback.png`
- Screenshot: `mobile/artifacts/screenshots/19_review_detail_playback.png`

Impact:

- The playback shell breaks the "operational clarity first" rule in `DESIGN.md`.
- Loading/error/ended overlays are less legible than the surrounding mobile UI.

Recommended follow-up:

- Add an additive tone/appearance prop to `StateView` so dark media overlays can
  use light foreground colors without forking the component.
- Keep the change testID-preserving and scoped to playback surfaces.

### F2 - Detail metadata panels still expose raw IDs without mobile-friendly formatting

Severity: Low

The playback-specific parts of the screens are clean, but the summary metadata
immediately above or around them still show raw IDs. That is serviceable in seeded
fixtures, yet it weakens the overall playback-screen polish and increases long-ID
overflow risk.

Evidence:

- Code: `mobile/src/screens/ReviewDetailScreen.tsx` top summary panel uses raw
  `task.id` and `task.asset_id`.
- Code: `mobile/src/screens/AssetDetailScreen.tsx` uses raw `asset.id` and
  `uploader_id`.
- Screenshot: `mobile/artifacts/screenshots/19_review_detail_playback.png`
- Screenshot: `mobile/artifacts/screenshots/18_asset_detail_playback.png`

Impact:

- The playback surfaces inherit adjacent metadata that is less refined than the
  later S-190 formatting patterns.
- Longer production identifiers may crowd the right-aligned summary rows.

Recommended follow-up:

- Decide whether these rows should use `formatId`, native ellipsis, or a compact
  stacked layout.
- Keep this as a separate polish patch, not a player-shell redesign.

## Passes

These areas already match the intended system and should remain stable:

- `PlaybackStateView` keeps denial/error/loading inside the same panel rhythm as
  the rest of the mobile surface.
- `VideoPlayer` uses the same border-radius and border treatment as cards/panels,
  so playback feels native to the product.
- `AssetDetailScreen` uses an explicit Play action instead of auto-opening media.
- `ReviewDetailScreen` keeps playback subordinate to the decision flow rather than
  letting the media surface dominate the screen.

## Proposed next task

If we want to act on this audit, open one narrow development task for playback
surface polish with the following scope:

- fix dark-overlay `StateView` contrast in `VideoPlayer`;
- optionally normalize summary-row identifier formatting in the two playback
  screens;
- preserve existing behavior and `testID`s;
- verify with the focused mobile screen/component suites and refreshed screenshots.
