---
type: TaskList
title: "Tasks: README Mobile Experience Refresh"
status: done
plan: docs/plan/readme-mobile-experience-refresh.md
---
# Tasks: README Mobile Experience Refresh

Plan: `docs/plan/readme-mobile-experience-refresh.md`.
RRI computed on 2026-06-25 with:
`python3 scripts/rri.py --platform rn --touches README.md --touches docs/plan/readme-mobile-experience-refresh.md --touches docs/tasks/readme-mobile-experience-refresh.md --touches mobile/artifacts/screenshots/playback_asset_detail.png --touches mobile/artifacts/screenshots/playback_review.png --cc 1 --T 0 --A 1 --X 1 --D 0 --K 0 --P 0`

## Task summary

| ID | Title | RRI → band | Effort | Status | Depends on |
|---|---|---|---|---|---|
| README-MOBILE-T1 | Refresh README mobile journey with embedded screenshots | 8 → Low | S | ✅ Done | — |

---

## README-MOBILE-T1 — Refresh README mobile journey with embedded screenshots

- **Status:** ✅ Done
- **Effort:** S
- **RRI:** 8 → band Low (0–25) → Effort S · thinking Off
- **Depends on:** —
- **Affected:** `README.md`, `mobile/artifacts/screenshots/`

### Objective

Replace the current linked screenshot references in the mobile README section with
an embedded, visually curated walkthrough that includes the latest playback
screens and keeps image weight under control.

### Inputs

- Existing mobile screenshots in `mobile/artifacts/screenshots/`
- New screenshots in `/tmp/dubbridge-maestro-playback/screenshots/`
- Current `README.md` mobile section

### Outputs

- New playback screenshots moved into the repository
- Updated embedded-image mobile section in `README.md`

### Acceptance criteria

- The README mobile section embeds screenshots inline instead of linking to them.
- The section reads as a coherent mobile user journey with short explanatory copy.
- The latest playback screens are represented in the asset-detail and review
  stages.
- Embedded screenshots render at a restrained size suitable for GitHub README
  viewing.
- New screenshots are optimized to reduce page weight while remaining legible.

### Handoff prompt

Refresh the `README.md` mobile section so it tells the current first-party mobile
story with embedded screenshots, including inline playback for asset detail and
review detail. Move the two new screenshots from `/tmp/dubbridge-maestro-playback/screenshots/`
into `mobile/artifacts/screenshots/`, optimize them for README use, and verify the
relative paths render correctly.

### Completion notes

- Moved the new playback screenshots into `mobile/artifacts/screenshots/` as
  `18_asset_detail_playback.png` and `19_review_detail_playback.png`.
- Resized both screenshots from `1080 x 2400` to `720 x 1600` for lighter README
  usage while preserving legibility.
- Rewrote the `README.md` mobile section as an embedded end-to-end walkthrough.
- Verification passed with `make qa-docs`.
