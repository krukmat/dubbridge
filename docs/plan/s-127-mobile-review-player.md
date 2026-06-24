---
type: Plan
title: "Plan: S-127 — Mobile Review Player Surface"
status: done
slice: S-127
---
# Plan: S-127 — Mobile Review Player Surface

> **Status:** Done — T0/T0b/T1/T2b/T2a/T3/T4/T5 complete as of 2026-06-24. 159 tests green; typecheck clean; Maestro playback.yaml authored and mock gateway extended; BDD mapping synchronized. Maestro runtime execution (screenshot capture) remains pending a Java-capable environment.
> **Roadmap phase:** `S-127` — mobile playback consumer for the S-125 HLS boundary.
> **Tasks ledger:** `docs/tasks/s-127-mobile-review-player.md`.
> **Governing ADRs:** ADR-032 (HLS playback delivery boundary), ADR-029 (mobile sole authenticated surface).

## Purpose

S-125 delivered a complete, tested HLS playback-delivery boundary (`POST /assets/{id}/playback-grants` → `GET .../playback/{grant_id}/manifest` → `GET .../playback/segments/{filename}?token=`). That boundary has no first-party consumer in the mobile app: reviewers approve dubs they cannot watch, and asset detail shows title + two UUIDs with no media.

S-127 closes this gap by adding a **playback client** and a **review-player screen** that consumes the S-125 boundary through the existing bearer-auth mobile gateway pattern. The focus is the review flow first: a reviewer opens a task, watches the original HLS stream, and submits a verdict — all in one screen. Asset browsing gains a secondary entry to the same player.

S-190 ("Mobile UX Usability Pass") polished the existing form-based screens and is a foundation this slice builds on; it explicitly left new screens and the API contract out of scope.

## Objective

- **T0b** — Add `docs/bdd/s-127-mobile-review-player.feature` plus `docs/bdd/README.md` mapping rows for the mobile playback scenarios this slice owns.
- **T1** — Add `mobile/src/api/playback.ts`: typed wrapper for the S-125 grant-issue flow plus a pure manifest-URL builder.
- **T2** — Add `<VideoPlayer>` primitive (`mobile/src/components/VideoPlayer.tsx`) using `expo-video` with load/buffer/error/end states, backed by a pure state module and design-token styling.
- **T3** — Replace `ReviewDetailScreen` with a v2 that embeds the player above the Approve/Reject controls so the reviewer watches before deciding.
- **T4** — Add an inline "Play" entry in `AssetDetailScreen` for playback-eligible assets, using the grant endpoint as the source of truth for readiness (reuses the T1/T2 stack).
- **T5** — QA pass: tests green, Maestro `playback.yaml` flow, BDD/docs sync (roadmap S-127 row, this plan, the ledger, and BDD mapping).

Execution order: T0b → T1 → T2b → T2a → T3 → T4 → T5.

## Design decisions

### D1 — expo-video, not expo-av
SDK is Expo ~56 (`package.json`). `expo-av` is deprecated; `expo-video` is the SDK-56 successor with native ExoPlayer (Android) / AVPlayer (iOS) HLS support.

### D2 — Auth handoff for HLS segments
The S-125 manifest contains rewritten segment references of the form `.../playback/segments/{filename}?token=<short-lived>`. These are absolute URLs through the gateway. The grant is issued with the current mobile bearer token (still named `sessionRef` in some app types); segment fetches carry the short-lived token in the query string — not the bearer token — so the player native HTTP stack fetches them without custom headers. This must be confirmed in T1 (verify segment refs in the rewritten manifest are absolute + token-scoped, not raw storage keys).

### D3 — Grant issued at screen open for review playback
The playback grant is requested when the review detail screen loads, not when the user taps Play. This avoids delay between tap and first frame. Review detail does not have a separate playback-readiness flag in its DTO, so the grant endpoint itself is the source of truth for whether playback is currently available. If the grant expires mid-session the player surfaces an error with a retry.

### D4 — Fail-closed on unavailable playback
If playback is not currently available for an asset, the API returns a denial and the UI shows a `StateView kind="empty"` ("Media not ready yet") rather than a broken player. Transport and permission failures are shown as `StateView kind="error"` so the app does not mislabel a network/auth issue as a readiness issue.

### D5 — Derived track placeholder
S-140/S-150 (subtitles/dubbing) are not built. The review player shows the original HLS stream only. A "Dubbed track" tab renders as a `StateView kind="empty"` placeholder — the seam for those slices to fill.

### D6 — No new design tokens
Follows S-115/S-190 restraint. VideoPlayer is a new primitive; no new color tokens beyond `tokens.ts`.

### D7 — testID continuity
`ReviewDetailScreen` keeps all existing testIDs (`review-detail-screen`, `review-approve`, `review-reject`, `review-comment-input`, `publish-action`). Player adds `review-player`. Asset detail adds `asset-play-button`. No existing Maestro flows break.

### D8 — Asset detail uses an optimistic playback gate
The current mobile asset DTO exposes ingestion status (`finalized`) but not S-125 preparation readiness (`PreparationStatus::Ready`). `AssetDetailScreen` therefore must not pretend it can know playback readiness locally. The v1 rule is:

- show the Play entry only for assets that are plausibly playback-eligible from the current DTO (`status === "finalized"`);
- issue the grant only on tap;
- treat grant success as permission to render the inline player;
- treat playback denial (`409`/`422` or equivalent) as `StateView kind="empty"` with "Media not ready yet".

This keeps S-127 within the current mobile/API contract instead of widening the asset DTO just to precompute a button gate.

### D9 — Asset detail stays inline, not a new navigation route
`AssetDetailScreen` opens playback inline in the same screen, not via a new push route. This keeps S-127 inside the declared file boundary, avoids `RootNavigator` churn, and preserves the slice as a surface enhancement rather than a navigation redesign.

### D10 — BDD is explicit, not implied by Maestro
This slice owns new user-visible playback behavior and therefore should record it in the repository's canonical BDD home, not only in a Maestro flow. `docs/bdd/s-127-mobile-review-player.feature` defines the stable behavioral scenarios; `mobile/maestro/playback.yaml` is one executable evidence artifact for the scenarios that require UI flow coverage; Jest and screen tests provide the rest.

## Affected files

| Layer | Path | Change |
|---|---|---|
| API client | `mobile/src/api/playback.ts` (new) | Grant issue + manifest URL helpers |
| Primitive | `mobile/src/components/VideoPlayer.tsx` (new) | `expo-video` wrapper |
| Primitive state | `mobile/src/components/video-player-state.ts` (new) | Pure playback state machine for the player shell |
| Primitive barrel | `mobile/src/components/index.ts` | Export VideoPlayer |
| Screen | `mobile/src/screens/ReviewDetailScreen.tsx` | Embed player; keep all existing testIDs |
| Screen | `mobile/src/screens/AssetDetailScreen.tsx` | Add inline Play button for finalized assets; readiness resolved by grant outcome |
| Tests | `mobile/__tests__/playback.api.test.ts` (new) | T1 unit tests |
| Tests | `mobile/__tests__/video-player-state.test.ts` (new) | T2 pure state tests |
| Tests | `mobile/__tests__/ReviewDetailScreen.test.tsx` (existing) | Extend for player states |
| Tests | `mobile/__tests__/asset.screens.test.tsx` (existing) | Extend for inline Play entry states |
| BDD | `docs/bdd/s-127-mobile-review-player.feature` (new) | Canonical playback scenarios for this slice |
| BDD mapping | `docs/bdd/README.md` | Map scenario IDs to Jest/Maestro evidence |
| Maestro | `mobile/maestro/playback.yaml` (new) | T5 E2E flow |
| Docs | `docs/plan/roadmap.md` | Add S-127 row |

## Verification

- `cd mobile && npm test -- --runInBand` and `npm run typecheck` green.
- All existing `testID`s preserved; existing Maestro flows unaffected.
- `review-player` testID present in rendered review-detail tree.
- `docs/bdd/s-127-mobile-review-player.feature` and `docs/bdd/README.md` are synchronized with the delivered Jest/Maestro evidence.
- `make qa-docs` passes.

## Relationship to adjacent slices

- **Consumes:** S-125 HLS playback boundary (ADR-032); S-115/S-190 design-system foundation.
- **Does not implement:** S-170 review runtime, S-180 publication runtime, S-130/S-140/S-150. The derived-track placeholder (D5) is the seam for those slices.
- **ADR-029:** mobile remains the sole authenticated UI; this is not a public player.
