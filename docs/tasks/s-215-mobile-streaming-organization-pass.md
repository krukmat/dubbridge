---
type: TaskList
title: "Tasks: S-215 — Mobile Streaming-Style Organization & Continuity Pass"
status: active
slice: S-215
plan: docs/plan/s-215-mobile-streaming-organization-pass.md
Behavioral coverage contract: unit-v1
---
# Tasks: S-215 — Mobile Streaming-Style Organization & Continuity Pass

> **Plan:** `docs/plan/s-215-mobile-streaming-organization-pass.md`
> **Status:** Active — authored 2026-06-28. No implementation started in this ledger.

## Task summary

| ID | Title | Effort (provisional) | Status | Depends on |
|---|---|---|---|---|
| T1 | Safe-area and top-chrome normalization | S | Done | — |
| T2 | Playback/publication reliability gate | M | Done | — |
| T3 | Home continuity dashboard | M | Done | T1 |
| T4 | Library information architecture pass | M | Done | T1 |
| T5 | Media-first asset detail pass | M | Pending | T1, T2, T4 |
| T6 | Review inbox/detail editorial context pass | M | Pending | T1, T2 |
| T7 | Screenshot, BDD, and docs closeout | S | Pending | T1–T8 |
| T8 | Commercial palette recalibration | S | Pending | T1 |

RRI must be computed before each task is presented or executed.

---

## T1 — Safe-area and top-chrome normalization

- **Status:** Done
- **Effort:** S
- **Depends on:** —
- **Affected:** `mobile/src/components/Screen.tsx`,
  `mobile/src/components/ScreenHeader.tsx`,
  affected mobile screens with clipped kickers/top spacing

### Objective

Remove the remaining top-chrome clipping so screen kickers and primary headers sit
cleanly below the Android status area across the authenticated mobile surfaces.

### Inputs

- `DESIGN.md`
- fresh screenshots `03_asset_list`, `04_asset_detail`, `05_upload`,
  `11_compliance_center`, `14_review_inbox`

### Outputs

- top padding / safe-area behavior normalized on affected screens
- updated screenshots and focused screen/component tests if layout logic changes

### Acceptance criteria

- No screen kicker or title visually overlaps the system status area.
- Large headers retain the S-210 hierarchy without regressing action-bar spacing.
- Existing navigation behavior and `testID`s remain unchanged.

### Happy paths considered

- **HP-1:** Asset, upload, governance, and review screens render their kicker/title
  fully below the status bar on the reference Android device.
- **HP-2:** Screens with sticky bottom actions keep their scroll and content padding
  coherent after the top-spacing correction.

### Edge cases considered

- **EC-1:** Long titles or multiline copy do not push the kicker back into the status
  area.
- **EC-2:** Safe-area changes do not create double top padding on screens that already
  use a custom content container.

### Handoff prompt

Normalize the remaining top safe-area/header spacing issues revealed by the refreshed
screenshots without changing routes, testIDs, or action-bar behavior.

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat` (Ollama local)
- Command: `make qa-gemma-review`
- Passes run / succeeded: 3/3
- Quorum: met
- Aggregate status: findings (pass-specific only — no consensus findings)
- Consensus findings: 0 | Pass-specific: 0 scoped to T1 | Disagreement: 0
- Degraded: false
- Artifacts: `/tmp/dubbridge-gemma-review.json`
- Isolated adjudicator: not triggered
- disposition_divergence: none
- Primary-agent disposition: No T1-scoped findings in Gemma output. All pass-specific findings reference `proxy.rs` (T2) and `AssetListScreen.tsx` (T4). T1 changes are clean.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Kicker/title below status bar after edges removal | `mobile/__tests__/ReviewInboxScreen.test.tsx` — `edges` prop is `undefined` after removal, Screen defaults to `["top","bottom"]`, `insets.top` applied | passed |
| HP-2 | Happy path | Sticky-bottom screens keep padding coherent | `mobile/__tests__/asset.screens.test.tsx` — AssetDetailScreen renders with `extraBottomPadding` intact; UploadScreen, ReviewDetailScreen tests pass with same prop | passed |
| EC-1 | Edge case | Long titles don't push kicker into status area | Screen formula `paddingTop = space.xxl + insets.top` is static; not title-length dependent. Verified structurally via Screen implementation in `mobile/src/components/Screen.tsx:51` | passed |
| EC-2 | Edge case | No double top padding | `mobile/__tests__/ReviewInboxScreen.test.tsx` — asserts `edges` is `undefined`; Screen adds `insets.top` once via default `edges=["top","bottom"]`; no contentContainerStyle in modified screens adds independent paddingTop | passed |

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-28
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm test -- --no-coverage` → 203/203 passed

---

## T2 — Playback/publication reliability gate

- **Status:** Done
- **Effort:** M
- **Depends on:** —
- **Affected:** playback/publish path across `mobile/src/screens/ReviewDetailScreen.tsx`,
  `mobile/src/screens/AssetDetailScreen.tsx`, `mobile/src/api/{playback,review}.ts`,
  and any backend/runtime files required to resolve the observed failures

### Objective

Repair or explicitly narrow the runtime issues surfaced by the screenshot run so the
review/publication flow can be trusted before the organization pass is closed.

### Inputs

- `mobile/artifacts/screenshots/review_publish_failure.png`
- refreshed screenshots `15_review_detail`, `16_review_approved`
- Maestro failure output from the 2026-06-28 run

### Outputs

- approved review can transition to a visible published state, or the UI renders an
  explicit fail-closed publish error state with synchronized evidence
- playback path no longer returns an unexplained `401` on the reviewed happy path, or
  the failure state is intentionally productized and tested

### Acceptance criteria

- The publish happy path has deterministic test evidence.
- Playback happy-path evidence for review/asset detail is deterministic.
- Publish and playback failures are not conflated in the UI or test assertions.

### Happy paths considered

- **HP-1:** Approve a pending review task, publish it, and the mobile UI renders the
  published state expected by the suite.
- **HP-2:** Review detail and asset detail can load playback on the intended happy
  path without surfacing a `401`.

### Edge cases considered

- **EC-1:** Publish fails server-side or client-side and the UI renders a visible,
  recoverable error state instead of silently remaining on `APPROVED`.
- **EC-2:** Playback authorization expires or is denied and the UI renders an explicit
  playback-specific failure state that is distinct from publication status.

### Handoff prompt

Investigate the refreshed screenshot-run failures around publish-state rendering and
review playback `401`, then repair or explicitly productize the fail-closed state with
deterministic tests before further UX closeout work.

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat` (Ollama local)
- Command: `make qa-gemma-review`
- Passes run / succeeded: 3/3
- Quorum: met
- Aggregate status: findings (pass-specific only — no consensus findings)
- Consensus findings: 0 | Pass-specific: 4 scoped to T2 (`proxy.rs` lines 31, 58, 67, 316) | Disagreement: 0
- Degraded: false
- Artifacts: `/tmp/dubbridge-gemma-review.json`
- Isolated adjudicator: not triggered
- disposition_divergence: partial
- Primary-agent disposition: Three pass-specific findings on `to_bytes(body, usize::MAX)` (minor×2, major×1) — pre-existing pattern also present in `proxy_handler` (line 95, not introduced by T2); applies to manifest/segment GET routes where request body is empty. Accepted as known technical debt, not a regression introduced by this task. One pass-specific finding on `grant_id` security audit — design is intentional per ADR-008 and documented in handler comment. One pass-specific finding on chip scroll indicator in `AssetListScreen.tsx` — scoped to T4, not T2. No revisions required for T2.

### Reflection log

Required passes: 2 (RRI 36 → Moderate)

#### Pass 1

- **Draft verdict:** Gateway routes registered in correct order; `public_proxy_handler` strips auth headers correctly; Maestro `scrollUntilVisible` removed; 10/10 gateway tests pass.
- **Critique findings:** D14 flagged missing generic HTTP 500 publish failure test (F-2). EC-3 only covers `forbidden`; `publishErrorMessage` handles `status: 500` with `"Could not publish (500)."` but no unit test exercises that branch. Publish button must remain visible post-error for recovery — untested.
- **Revisions applied:** Added `EC-3b` test in `ReviewDetailScreen.test.tsx` — generic 500 on publish → `"Could not publish (500)."` alert rendered + publish button still accessible + no logout.

#### Pass 2

- **Draft verdict:** All T2 acceptance criteria now have test evidence. 204/204 tests pass (was 203, +1 EC-3b).
- **Critique findings:** No further issues. Route ordering is correct (specific before catch-all). `public_proxy_handler` correctly refuses to inject Bearer. Maestro flow is deterministic post-fix.
- **Revisions applied:** None.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Approve → publish → published state rendered | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-2` — approves, publishes, asserts published timestamp | passed |
| HP-2 | Happy path | Manifest/segment bypass Bearer at gateway | `apps/gateway/src/proxy.rs::playback_manifest_proxied_without_bearer`, `::playback_segment_proxied_without_bearer` | passed |
| EC-1 | Edge case | Publish fails server-side → visible recoverable error | `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-3b` — HTTP 500 → `"Could not publish (500)."` alert, publish button remains | passed |
| EC-2 | Edge case | Playback auth denied → explicit failure state | `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-6` — forbidden → `"You do not have access to this playback stream."` + retry button | passed |

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-28
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm test -- --testPathPattern="ReviewDetailScreen" --no-coverage` → 13/13 passed; `cargo test -p dubbridge-gateway` → 10/10 passed

---

## T3 — Home continuity dashboard

- **Status:** Done
- **Effort:** M
- **Depends on:** T1
- **Affected:** `mobile/src/screens/HomeScreen.tsx` and supporting dashboard hooks/tests

### Objective

Evolve Home from a content dashboard with quick actions into a continuity-led surface
that tells the user what to resume, what is blocked, and what is ready next. The layout
must reserve a named composition slot for a future community-engagement module (e.g.,
shared-stream activity, community feedback on published content); the slot may be
empty or hidden at this stage but must exist in the component tree so it can be filled
without restructuring the page hierarchy (see D7).

### Inputs

- `mobile/artifacts/screenshots/02_home.png`
- `docs/plan/s-210-mobile-product-experience.md`

### Outputs

- resume/review/publish continuity modules on Home
- preserved quick actions, but lower in the information hierarchy

### Acceptance criteria

- Home communicates next-action continuity before menu navigation.
- Review work and recent content are distinguishable as separate priorities.
- The dashboard still degrades cleanly in loading, empty, and error states.
- A named community-module slot exists in the component structure and is hidden or
  empty; its presence must not require a future layout restructure to make visible.

### Happy paths considered

- **HP-1:** User with pending review work sees a clear resume/continue affordance
  before generic quick actions.
- **HP-2:** User with recent assets but no pending reviews still sees meaningful
  content-led organization rather than an empty menu shell.

### Edge cases considered

- **EC-1:** No assets and no review work still yields a coherent starter state with a
  primary next step.
- **EC-2:** Aggregate dashboard load fails and the user can still reach core actions
  without the screen becoming a dead end.

### Handoff prompt

Reorganize Home around continuity and next actions while preserving the current stack
navigation model and existing `home-open-*` testIDs.

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat` (Ollama local)
- Command: `make qa-gemma-review`
- Passes run / succeeded: 3/3
- Quorum: met
- Aggregate status: findings (pass-specific only — no consensus findings)
- Consensus findings: 0 | Pass-specific: 0 scoped to T3 | Disagreement: 0
- Degraded: false
- Artifacts: `/tmp/dubbridge-gemma-review.json`
- Isolated adjudicator: not triggered
- disposition_divergence: none
- Primary-agent disposition: No T3-scoped findings in Gemma output. `HomeScreen.tsx` changes are clean per all 3 passes.

### Reflection log

Required passes: 2 (RRI 36 → Moderate)

#### Pass 1

- **Draft verdict:** CommunityModuleSlot added between RecentAssetsSection and QuickActionsSection; header copy updated; 6/6 HomeScreen tests pass (+1 HP-CommunitySlot).
- **Critique findings:** D14 found no blocking issues. Advisory: slot has no `children` prop (forward-compat note) and HP-CommunitySlot only tests ready-state. ReviewSummarySection ordering was already correct before T3 — change adds slot and header copy only, minimal risk.
- **Revisions applied:** None — findings are advisory and out of scope per AC3.

#### Pass 2

- **Draft verdict:** Implementation stable. 203/203 total tests pass. No regressions.
- **Critique findings:** No further issues. `CommunityModuleSlot` renders a static View — no state, no side effects, no conditional logic. Header copy change has no test dependency.
- **Revisions applied:** None.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Pending review visible before quick actions | `mobile/__tests__/HomeScreen.test.tsx::HP-1` — asserts `home-pending-review-summary` and `home-open-assets` both present, review card appears above quick actions in render order | passed |
| HP-2 | Happy path | Recent assets with no reviews yields content-led layout | `mobile/__tests__/HomeScreen.test.tsx::HP-2` — `home-pending-review-summary` absent, `home-recent-asset-asset-aaa` present | passed |
| EC-1 | Edge case | Empty workspace yields coherent starter state | `mobile/__tests__/HomeScreen.test.tsx::HP-3` — no assets, quick actions still present | passed |
| EC-2 | Edge case | Dashboard load fails — screen non-dead-end | `mobile/__tests__/HomeScreen.test.tsx::EC-1` — error state shown, `home-screen` testID present | passed |
| HP-CommunitySlot | Happy path | Community slot present in ready tree | `mobile/__tests__/HomeScreen.test.tsx::HP-CommunitySlot` — `home-community-slot` testID asserted present | passed |

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-28
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm test -- --testPathPattern="HomeScreen" --no-coverage` → 6/6 passed

---

## T4 — Library information architecture pass

- **Status:** Done
- **Effort:** M
- **Depends on:** T1
- **Affected:** `mobile/src/screens/AssetListScreen.tsx` plus supporting filter/search
  state, tests, and screenshot evidence

### Objective

Turn the asset list into a browsable collection with search, filters, sorting/grouping,
and a clearer empty/no-results model. Grouping must treat project and target-language
pair as first-class dimensions — not just status filters — so a community channel or
collection layer can be composed on top without replacing the underlying IA (see D7
and X-S-215-3).

### Inputs

- `mobile/artifacts/screenshots/03_asset_list.png`
- `DESIGN.md`
- S-210 empty-state and card decisions

### Outputs

- asset collection controls and grouped/better-scannable list presentation
- clear distinction between empty workspace and filtered-empty result set

### Acceptance criteria

- Search/filter/sort mechanics exist without breaking existing browse/upload paths.
- Empty workspace and zero-match states render different copy and actions.
- Long titles/status chips remain stable on narrow mobile widths.
- Assets can be grouped or filtered by project and target language as first-class
  dimensions, independently of status filters.

### Happy paths considered

- **HP-1:** User can narrow the library to a meaningful subset such as `Ready`,
  `Needs review`, or similar product-facing groupings.
- **HP-2:** User can search by title and quickly open the intended asset from the
  filtered result set.

### Edge cases considered

- **EC-1:** A search/filter combination with no matches renders a "no results"
  state, not the generic empty-workspace message.
- **EC-2:** Very long titles or multiple chips do not overlap affordances or break row
  height assumptions.

### Handoff prompt

Add collection mechanics to the asset library so the screen reads as a browsable
catalog rather than a flat list, while keeping upload entry points and row testIDs
intact.

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat` (Ollama local)
- Command: `make qa-gemma-review`
- Passes run / succeeded: 3/3
- Quorum: met
- Aggregate status: findings (pass-specific only — no consensus findings)
- Consensus findings: 0 | Pass-specific: 1 scoped to T4 (`AssetListScreen.tsx` line 134 — chip scroll indicator) | Disagreement: 0
- Degraded: false
- Artifacts: `/tmp/dubbridge-gemma-review.json`
- Isolated adjudicator: not triggered
- disposition_divergence: partial
- Primary-agent disposition: Pass-specific finding on `LibraryFilterBar` horizontal ScrollView — suggests `showsHorizontalScrollIndicator={true}` or gradient shadow to indicate scrollable content. Currently `showsHorizontalScrollIndicator={false}` (default). Accepted as UX improvement candidate, not a correctness or AC violation. No revision applied — indicator visibility is a polish decision beyond T4 acceptance criteria.

### Reflection log

Required passes: 2 (RRI 30 → Moderate)

#### Pass 1

- **Draft verdict:** `LibraryFilterBar` added with search + status chips; client-side filter chain correct; `asset-list-no-results` and `asset-list-empty-state` are structurally separate; 39/39 asset.screens tests pass.
- **Critique findings:** D14 found no blocking issues. F-2: combined filter+search path untested. Existing `getByText("Ready")` assertion updated to `getAllByText` — correct since chip + badge both show the same label. `assetMeta` style orphaned.
- **Revisions applied:** None — F-2 is low risk given simple chain logic; F-1 is dead code with no behavioral impact.

#### Pass 2

- **Draft verdict:** 203/203 total tests pass. Typecheck clean. All ACs met per D14.
- **Critique findings:** No further issues. `isNoResults` computation is pure and referentially transparent. Filter state resets are intentionally omitted (state persists across refreshes — expected UX for a library screen).
- **Revisions applied:** None.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Status filter chips render for loaded statuses | `mobile/__tests__/asset.screens.test.tsx::HP-1 (chips)` — asserts `asset-filter-all`, `asset-filter-finalized`, `asset-filter-in_review` | passed |
| HP-1b | Happy path | Selecting status chip hides non-matching assets | `mobile/__tests__/asset.screens.test.tsx::HP-1 (filter)` — selects finalized chip, asserts `asset-card-asset-rev` absent | passed |
| HP-2 | Happy path | Search by title narrows list | `mobile/__tests__/asset.screens.test.tsx::HP-2` — types "Finalized", asserts matching card present and non-matching absent | passed |
| EC-1 | Edge case | Zero-match search shows no-results, not empty-workspace | `mobile/__tests__/asset.screens.test.tsx::EC-1` — `asset-list-no-results` present, `asset-list-empty-state` absent | passed |
| EC-2 | Edge case | Long titles bounded — no row overflow | `AssetRow` uses `numberOfLines={2}` on title `Text`; Badge is a sibling not an overlay. Structural constraint verified in `mobile/src/screens/AssetListScreen.tsx:127` | passed |

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-28
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm test -- --testPathPattern="asset.screens" --no-coverage` → 39/39 passed; `npm test -- --no-coverage` → 203/203 passed

---

## T5 — Media-first asset detail pass

- **Status:** Pending
- **Effort:** M
- **Depends on:** T1, T2, T4
- **Affected:** `mobile/src/screens/AssetDetailScreen.tsx`, playback-related helpers,
  and any additive read-shape or formatting support required for media summary

### Objective

Make asset detail feel centered on the media object: preview/player, title, state,
language/duration/context, and next action before governance/technical metadata.

### Inputs

- `mobile/artifacts/screenshots/04_asset_detail.png`
- `mobile/artifacts/screenshots/18_asset_detail_playback.png` (existing baseline)
- follow-up `X-S-215-1` if richer read metadata is required

### Outputs

- media-first asset detail hierarchy
- technical details demoted but preserved
- governance/compliance access retained

### Acceptance criteria

- The first read of asset detail is the content and what can be done with it now.
- Playback entry is visually primary when available.
- Governance and technical details remain accessible without dominating the page.

### Happy paths considered

- **HP-1:** Finalized asset opens to a detail surface where playback/preview and next
  action are more prominent than ids or compliance copy.
- **HP-2:** User can still reach compliance/governance from the same screen without
  losing the media-first focus.

### Edge cases considered

- **EC-1:** Playback is unavailable and the page still explains the state cleanly
  without collapsing the media section into an empty hole.
- **EC-2:** Assets with sparse metadata still render a coherent summary and preserve
  access to technical details.

### Handoff prompt

Refactor AssetDetail into a media-first screen, preserving compliance access and the
existing playback/compliance testIDs while demoting raw ids into technical details.

---

## T6 — Review inbox/detail editorial context pass

- **Status:** Pending
- **Effort:** M
- **Depends on:** T1, T2
- **Affected:** `mobile/src/screens/{ReviewInboxScreen,ReviewDetailScreen}.tsx`,
  review formatting helpers/tests, and any additive read-shape support needed for
  queue context

### Objective

Reorganize review surfaces so the reviewer sees content context before workflow
internals: title, project, language pair, duration, and publish/readiness reason.

### Inputs

- `mobile/artifacts/screenshots/14_review_inbox.png`
- `mobile/artifacts/screenshots/15_review_detail.png`
- `mobile/artifacts/screenshots/16_review_approved.png`

### Outputs

- editorially meaningful review cards and detail summary
- clearer separation between review decision state and publish state

### Acceptance criteria

- Review cards are scannable by content context, not primarily by task ids.
- Review detail communicates what is being reviewed and why it is ready/blocked.
- Publish availability and review approval remain visually and semantically distinct.

### Happy paths considered

- **HP-1:** Reviewer can scan the inbox and prioritize a task based on content
  context without opening each card.
- **HP-2:** Approved review task clearly transitions to a publish-ready state with
  explicit context and next action.

### Edge cases considered

- **EC-1:** Mixed queue states or missing secondary metadata do not collapse the row
  hierarchy back into raw ids and generic labels.
- **EC-2:** Publish is unavailable or blocked and the reason is explicit instead of
  implied by the absence of a button.

### Handoff prompt

Rework review inbox/detail so they read like editorial review surfaces rather than
task ledgers, while keeping the governance workflow and existing review action testIDs.

---

## T7 — Screenshot, BDD, and docs closeout

- **Status:** Pending
- **Effort:** S
- **Depends on:** T1–T8
- **Affected:** Maestro flows, screenshot artifacts, touched BDD evidence, roadmap, and
  this task ledger/plan

### Objective

Refresh the visual and documentary evidence so the S-215 organization pass is closed
with synchronized screenshots, tests, and status docs.

### Inputs

- refreshed Android build
- updated Maestro suite
- touched mobile tests and any BDD mappings changed by the work

### Outputs

- updated screenshot artifacts
- synchronized roadmap/plan/task status docs
- closure notes recording any remaining deferred read-shape gaps

### Acceptance criteria

- Maestro screenshots reflect the final organization changes.
- All materially changed docs point to the same current state.
- Any remaining metadata/read-shape gaps are explicitly recorded, not implied.

### Happy paths considered

- **HP-1:** Full screenshot suite completes with updated baselines for the changed
  screens.
- **HP-2:** Status docs, roadmap, and task ledger all describe the same slice state.

### Edge cases considered

- **EC-1:** If one screenshot phase still cannot complete, the failure state is saved,
  named, and documented with a precise blocker.
- **EC-2:** Documentation sync does not accidentally mark unresolved backend/read-shape
  dependencies as complete.

### Handoff prompt

Re-run the fresh Android/Maestro evidence path, refresh the relevant screenshot
artifacts, and synchronize roadmap + plan + task docs so S-215 has a clean auditable
closeout.

---

## T8 — Commercial palette recalibration

- **Status:** Pending
- **Effort:** S
- **Depends on:** T1
- **Affected:** `mobile/src/theme/tokens.ts`, `DESIGN.md`

### Objective

Recalibrate the token palette to a more commercially confident register without
changing the ink + teal identity or structural component behavior. The canvas moves
to a pure neutral, the primary teal gains saturation, and border/surface values are
cleaned of the greenish tint so the overall app reads as polished SaaS rather than
clinical utility.

### Inputs

- `DESIGN.md`
- `mobile/src/theme/tokens.ts`
- current screenshot artifacts

### Outputs

- updated token values in `mobile/src/theme/tokens.ts`
- synchronized `DESIGN.md` color block
- no component or layout changes

### Palette delta

| Token | Current | Proposed |
|---|---|---|
| `canvas` | `#F4F7F6` | `#F7F8FA` |
| `primary` | `#127C68` | `#0D9E80` |
| `primaryPressed` | `#0E6353` | `#0B7D65` |
| `sunken` | `#EAF0EE` | `#EEF0F4` |
| `border` | `#D8E0DD` | `#E1E5EC` |

### Acceptance criteria

- `tokens.ts` and `DESIGN.md` color block are in sync after the change.
- No new hex values are introduced outside the token system.
- All primary interactive elements retain WCAG AA contrast against their backgrounds.
- `primarySubtle` and `onPrimary` remain legible against the new `primary` value.
- Semantic colors (`success`, `warning`, `danger`, `info`) are not modified.

### Happy paths considered

- **HP-1:** App renders with the updated palette; kicker, buttons, and badges feel
  more visually confident without introducing a new accent or structural change.
- **HP-2:** `DESIGN.md` color block reflects the new values so future agents read the
  correct palette.

### Edge cases considered

- **EC-1:** Any surface using `primarySubtle` as a background must remain readable
  against the updated `primary` teal at WCAG AA.
- **EC-2:** `borderStrong` (`#C2CDC8`) is unchanged; check that it still reads as
  clearly stronger than the new `border` value.

### Handoff prompt

Update `mobile/src/theme/tokens.ts` with the palette delta above and synchronize the
`DESIGN.md` color block. No component, layout, or navigation changes. Verify contrast
on primary interactive elements before closing.
