---
type: Plan
title: "Plan: S-215 — Mobile Streaming-Style Organization & Continuity Pass"
status: complete
slice: S-215
governed_by: [ADR-029]
---
# Plan: S-215 — Mobile Streaming-Style Organization & Continuity Pass

> **Status:** Complete (2026-06-29). P0 ✅ P1 ✅ P2 ✅ P3 ✅ — T1–T8 all Done.
> **Roadmap phase:** `S-215`, a non-blocking mobile product follow-up on top of S-210 that focuses on organization, continuity, and media-first wayfinding.
> **Tasks ledger:** `docs/tasks/s-215-mobile-streaming-organization-pass.md`

## Purpose

S-210 materially improved the mobile product surface: Home now carries live content,
recent assets are visible, quick actions are clearer, and several screens reclaimed
space and action hierarchy. After recompiling the Android app and re-running the
Maestro screenshot suite on 2026-06-28, however, the app still reads more like a
governed operations console than a streaming-style working surface.

The issue is no longer visual inconsistency. The remaining gap is **organization**:
what the user sees first, how they resume work, how a library grows without becoming
a list of rows, and whether detail/review screens are anchored around media instead of
technical workflow nouns.

S-215 documents and implements that next pass without removing governance,
compliance, upload, review, or publication functionality. The goal is to reorganize
the same product into a more intuitive, content-led flow.

## Evidence: screenshot audit (2026-06-28)

Audit source:

- refreshed screenshots in `mobile/artifacts/screenshots/01_auth_login.png` through
  `16_review_approved.png`
- failure-state capture `mobile/artifacts/screenshots/review_publish_failure.png`

### Current-state findings

| # | Severity | Finding | Screenshot evidence |
|---|---|---|---|
| F1 | 🔴 High | **Home is improved but still not continuity-led.** The dashboard shows recent assets and review count, but there is no "continue reviewing", "resume playback", or "next required action" organization. It still asks the user to choose a section rather than continuing their current workflow. | `02_home` |
| F2 | 🔴 High | **Library organization is only partially solved.** The asset surface now has search, status filters, and an empty-vs-no-results split, but it still lacks sort/grouping and does not yet behave like a shared browsable collection organized by project/language dimensions. | `03_asset_list` |
| F3 | 🔴 High | **Asset detail is not media-first yet.** Playback is present but visually secondary; metadata and compliance still carry more visual weight than the actual media object. A streaming-style surface would anchor the page around preview, duration, language, and next action. | `04_asset_detail` |
| F4 | 🔴 High | **Review surfaces are improved but still not fully editorial-first.** The inbox/detail hierarchy now separates readiness and publication states, but the primary user-facing nouns still lean on `target_language_id`, `project_id`, and other technical identifiers rather than human content context. | `14_review_inbox`, `15_review_detail`, `16_review_approved` |
| F5 | 🔴 High | **Asset detail is still not the strongest media anchor in the product.** The screen now has better action hierarchy, but playback/preview still needs to dominate more clearly than technical metadata and compliance copy. | `04_asset_detail`, `18_asset_detail_playback.png` |
| F6 | 🟡 Medium | **Palette/commercial confidence is still behind the IA work.** The structure improved faster than the visual tone; the app still reads slightly clinical/utility-first and needs the planned palette recalibration pass to feel more productized without leaving ADR-029 restraint. | refreshed screenshots across `02_home`–`16_review_approved` |

## Objective

- Reorganize the mobile product around **continuity first**: what needs attention,
  what is ready, and where the user should resume.
- Turn the asset library into a **browsable collection**, not a flat list.
- Make asset and review detail screens **media-first** while preserving governance
  and audit access.
- Clarify review context with editorial signals before workflow identifiers.
- Finish the remaining media-first asset-detail work, preserve the review/context
  gains already landed, and close the slice with refreshed visual evidence.
- Apply the planned palette recalibration so the improved IA also reads as a more
  commercially confident product surface.

## Scope

### Included

- Home/dashboard information architecture and continuity affordances.
- Library search/filter/sort/grouping and empty/no-results distinction.
- Media-first asset/review cards and detail-surface organization.
- Top-chrome/safe-area corrections on affected mobile screens.
- Publication/playback runtime stabilization on the reviewed mobile paths.
- Screenshot baseline refresh and docs sync.
- Token-level palette recalibration plus synchronized `DESIGN.md` color block.

### Excluded

- New backend write workflows unrelated to playback/publication correctness.
- Replacing the stack navigator with a mandatory bottom-tab shell. This exclusion is
  provisional: a future community/discovery surface may require a dedicated tab, and
  that decision must be reopened before the bottom-tab option is foreclosed by further
  stack-navigation investment.
- New pipeline stages (`S-130+`) or schema unrelated to mobile product organization,
  except additive read-shape support that S-215 explicitly records.
- Removing governance/compliance surfaces or reducing fail-closed behavior.

## Design decisions

### D1 — Preserve the existing product surface; reorganize it

S-215 does not remove upload, compliance, review, or publication. It changes their
presentation order and navigation emphasis so the product feels content-led rather
than action-led.

### D2 — Continuity beats menu navigation

Home should answer "what should I do next?" before "where do you want to go?".
Dashboard content should be organized around resume/review/publish continuity, then
recent assets, then quick actions.

### D3 — Library needs collection mechanics

The asset surface needs a browsing model: search, status filters, sort/grouping, and
a visible distinction between workspace-empty and query-empty. The current empty-state
CTA pattern remains, but collection state becomes first-class.

### D4 — Media object before technical detail

Asset and review detail should lead with preview/player, title, language, duration,
state, and next action. Raw ids stay available in a technical-details group but are
not primary reading material.

### D5 — Editorial context before workflow ids in review

Review inbox and detail must foreground the content being reviewed: title, project,
source/target language, duration, and current blocking reason. Task ids and internal
scope ids are secondary metadata.

### D6 — Reliability gates UX closeout

S-215 does not treat the publication state bug or the playback `401` as visual nits.
The organization pass is not closed until those flows are either repaired or narrowed
to an explicit, visible fail-closed state with updated screenshots and evidence.

### D7 — Streaming-style does not mean consumer-only, but the community path must stay open

This is still a professional governed workspace, not a consumer OTT app. The target
feel is "operator tooling with clear content hierarchy", not "marketing Netflix clone".
The visual language from `DESIGN.md` and ADR-029 remains authoritative.

However, "Published" is not a terminal badge — it is a potential launch point for a
community-facing surface. S-215 IA decisions in T3 (Home) and T4 (Library) must not
foreclose this path:

- **T3 Home:** the dashboard layout must reserve a named composition slot for a future
  community-engagement module (community feedback, shared-stream activity). The slot
  can be empty or hidden at this stage; it must exist in the component structure so it
  can be filled without redesigning the page hierarchy.
- **T4 Library:** the grouping model must treat project / target-language pair as a
  first-class browsing dimension alongside status filters, so a community channel or
  collection layer can be layered on top without replacing the underlying IA.

## Backend/read-shape follow-ups recorded

| Follow-up | Need | Notes |
|---|---|---|
| X-S-215-1 | Asset/review cards need richer read data (`duration_ms`, `source_language`, `target_language`, optional poster/thumbnail key) | Placeholder tiles from S-210 are acceptable interim behavior, but real collection browsing needs these fields. |
| X-S-215-2 | Review rows need stable human-readable context (`title`, project name, target-language label, blocking summary) instead of falling back to ids as the primary surface noun. | Can be derived client-side only if current API shape is sufficient; otherwise record additive read contract. |
| X-S-215-3 | Published content requires a community-accessible read contract: channel/collection grouping keyed by project + target language, public asset metadata (title, target language, duration, thumbnail), and a stable public identifier distinct from internal asset UUIDs. | S-215 does not implement this. It is recorded explicitly so it is not silently assumed at closeout. Required before any community/discovery surface can be built on top of the publication state. |

## Affected components

| Layer | Path | Change |
|---|---|---|
| Mobile screens | `mobile/src/screens/{HomeScreen,AssetListScreen,AssetDetailScreen,ReviewInboxScreen,ReviewDetailScreen}.tsx` | information architecture, continuity, media-first hierarchy |
| Mobile components | `mobile/src/components/{Card,StateView,ScreenHeader,ActionBar,PlaybackStateView}.tsx` | layout/wayfinding/status presentation adjustments |
| Mobile hooks/api | `mobile/src/hooks/*`, `mobile/src/api/{playback,review}.ts` | dashboard aggregation, filter state, publish/playback flow correctness |
| Navigation | `mobile/src/navigation/RootNavigator.tsx` | wayfinding affordances only; stack model retained unless explicitly reopened |
| Backend/runtime if required | `apps/api`, `apps/worker-runner`, `crates/*` on playback/publication path | only if needed to resolve F6 |
| Theme/docs | `mobile/src/theme/tokens.ts`, `DESIGN.md` | commercial palette recalibration and design-doc sync |
| Docs | this plan, task ledger, roadmap, screenshot artifacts | status sync and evidence |

## Phased rollout

| Phase | Theme | Tasks |
|---|---|---|
| P0 | Reliability and chrome | T1 ✅, T2 ✅ |
| P1 | Continuity and library organization | T3 ✅, T4 ✅ |
| P2 | Media-first detail and review context | T5 ✅, T6 ✅ |
| P3 | Evidence and finish | T7 ✅, T8 ✅ |

## Task decomposition

| Task | Title | Effort (provisional) | Depends on |
|---|---|---|---|
| T1 | Safe-area and top-chrome normalization | S | — |
| T2 | Playback/publication reliability gate | M | — |
| T3 | Home continuity dashboard | M | T1 |
| T4 | Library information architecture pass | M | T1 |
| T5 | Media-first asset detail pass | S | T1, T2, T4 |
| T6 | Review inbox/detail editorial context pass | M | T1, T2 |
| T7 | Screenshot, BDD, and docs closeout | S | T1–T8 |
| T8 | Commercial palette recalibration | S | T1 |

RRI must be computed per task before implementation. This plan records the sequence,
scope, and acceptance intent; it does not pre-authorize execution.

## Dependency flow

```mermaid
flowchart TD
  T1["T1 — safe-area and top chrome"] --> T3["T3 — continuity dashboard"]
  T1 --> T4["T4 — library IA"]
  T1 --> T5["T5 — asset detail media-first"]
  T1 --> T6["T6 — review context"]
  T2["T2 — playback/publication reliability"] --> T5
  T2 --> T6
  T4 --> T5
  T3 --> T7["T7 — screenshot + docs closeout"]
  T4 --> T7
  T5 --> T7
  T6 --> T7
  T2 --> T7
  T8["T8 — commercial palette recalibration"] --> T7
```

## Verification expectations

- `cd mobile && npm run typecheck`
- `cd mobile && npm test -- --runInBand`
- Maestro screenshot suite re-run with fresh Android build
- refreshed screenshots must show corrected top chrome and updated organization
- publication path must either render the expected published state or fail with an
  explicit, reviewed product state and synchronized test evidence
- the final screenshot set must reflect the palette recalibration and the
  remaining asset-detail hierarchy changes
- `make qa-docs`

## Related

- `DESIGN.md`
- `docs/plan/s-190-mobile-ux-usability-pass.md`
- `docs/plan/s-210-mobile-product-experience.md`
- `docs/tasks/s-210-mobile-product-experience.md`
- `docs/adr/ADR-029-mobile-as-sole-authenticated-product-surface.md`
- `mobile/artifacts/screenshots/review_publish_failure.png`
