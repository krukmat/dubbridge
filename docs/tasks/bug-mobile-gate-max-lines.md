---
type: TaskList
title: "Bug: Mobile gate max-lines refactor"
description: "Restore the GitHub Actions mobile gate by decomposing AssetListScreen and ReviewDetailScreen under the existing max-lines-per-function rule without changing behavior."
status: closed
plan: docs/plan/mobile-gate-max-lines-refactor.md
rri: 35
band: Moderate
effort: M
---

# BUG-MOBILE-01 — Mobile gate `max-lines-per-function` regression

> Surfaced in GitHub Actions on 2026-07-01 and reproduced locally with `make qa-mobile`.
> Current failures:
> - Run `28514576103` (`ed262c1`) → `mobile` job → `Run mobile gate`
> - Run `28500512994` (`b226931`) → same failure point

- **Task ID:** BUG-MOBILE-01
- **Status:** Active — analyzed and planned; implementation pending approval
- **Effort:** M
- **Complexity:** Moderate
- **RRI:** 35 → Moderate (26–40)
- **Recommended model:** Codex Balanced (`GPT-5.2-Codex`) / Claude Code Balanced (`Claude Sonnet 4`)

## Objective

Restore the `mobile` CI gate by reducing `AssetListScreen` and
`ReviewDetailScreen` below the enforced 60-line function limit while preserving
their current behavior, tests, and mobile design semantics.

## Context

This bug is operationally blocking because `ci.yml` is red on `main`. The
failing pushes were not mobile-feature pushes, which means the mobile gate is now
catching pre-existing screen bloat that must be paid down before unrelated work
can merge cleanly.

The issue sits outside a single roadmap slice but directly affects the mobile
delivery path. It also blocks clean CI for any subsequent work that touches
`main`, including planned S-130 work.

## Related documents

- `docs/plan/mobile-gate-max-lines-refactor.md`
- `docs/daily/2026-07-01.md`
- `DESIGN.md`
- `mobile/eslint.config.js`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

## T1 — Refactor `AssetListScreen` composition

**Effort:** M (RRI inherited: 35 — Moderate)  
**Depends on:** none  
**Status:** [x] Done

**Happy paths considered:**
- HP-1: A ready asset list still renders cards, status badges, search, and status
  chips, and opening an asset still calls `onOpenAsset`.
- HP-2: An empty workspace still shows the upload CTA when `onOpenUpload` is
  available.
- HP-3: Pull-to-refresh still reloads the list and preserves session rotation
  handling.

**Edge cases considered:**
- EC-1: Search + status filter with no matches still shows the dedicated "No
  results" state instead of the generic empty-library state.
- EC-2: `session_expired` still logs the user out instead of showing an inline
  error.
- EC-3: Non-session API failures still render the retryable error state.

**Inputs:**
- `mobile/src/screens/AssetListScreen.tsx`
- `mobile/__tests__/asset.screens.test.tsx`
- GH / local lint evidence pointing at `AssetListScreen` line-count overflow

**Outputs:**
- `AssetListScreen` reduced below 60 lines
- extracted helper(s) or section component(s) for derived list/filter/render logic
- focused tests updated only if necessary to reflect structural refactor

**Acceptance criteria:**
- ESLint no longer reports `max-lines-per-function` for `AssetListScreen`
- existing list/filter/empty/error flows remain covered
- no existing `testID` used by tests or Maestro flows is removed or renamed

**Files expected to change:**
- `mobile/src/screens/AssetListScreen.tsx`
- `mobile/__tests__/asset.screens.test.tsx`

**Agent handoff prompt:** Decompose `AssetListScreen` so the exported screen
function drops below the 60-line lint threshold without weakening behavior.
Prefer extracting derived-state helpers or narrow child sections over changing the
screen contract.

### Reflection log

Required passes: 2 (`28` → `Moderate`)

#### Pass 1

- **Draft verdict:** `useAssetListFilter` extraído; función principal en 58 líneas; JSX compactado.
- **Critique findings:** Lógica HP/EC de search/filter/empty intacta; todos los testIDs preservados; hook llamado incondicionalmente.
- **Revisions applied:** Ninguna.

#### Pass 2

- **Draft verdict:** Tests 61/61 verdes post-implementación.
- **Critique findings:** EC-2 (`session_expired`) no tocado — `useAssetListState` intacto; EC-1 (`isNoResults`) idéntico. Sin side effects en tests adyacentes.
- **Revisions applied:** Ninguna.

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat`
- Command: `python3 scripts/gemma-code-review.py phase2-review-packet.md --passes 1`
- Passes run / usable: `1/1`
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `1` | Disagreement: `0`
- Artifacts: `scratchpad/phase2-result.json`
- Isolated adjudicator: `not triggered` — trigger: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: Pass-specific minor finding (`a.title?.toLowerCase()`) rechazado como falso positivo — `AssetSummary.title` es `string` no-nullable por TypeScript; comportamiento pre-existente en el archivo original, no introducido por este refactor.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | asset list renders cards + opening calls `onOpenAsset` | `mobile/__tests__/asset.screens.test.tsx::SC-LIST-1` | passed |
| HP-2 | Happy path | empty workspace shows upload CTA | `mobile/__tests__/asset.screens.test.tsx::SC-EMPTY-1 HP-1` | passed |
| HP-3 | Happy path | pull-to-refresh reloads list | `mobile/__tests__/asset.screens.test.tsx::EC: gateway or network failure / retries on tap` | passed |
| EC-1 | Edge case | search+filter no matches → "No results" state | `mobile/__tests__/asset.screens.test.tsx::T4 EC-1` | passed |
| EC-2 | Edge case | `session_expired` → logout | `mobile/__tests__/asset.screens.test.tsx::EC: session_expired triggers logout` | passed |
| EC-3 | Edge case | non-session API failure → retryable error state | `mobile/__tests__/asset.screens.test.tsx::EC: gateway or network failure` | passed |

---

## T2 — Refactor `ReviewDetailScreen` composition

**Effort:** M (RRI inherited: 35 — Moderate)  
**Depends on:** T1  
**Status:** [x] Done

**Happy paths considered:**
- HP-1: Pending review tasks still show playback, decision input, and approve /
  reject action bars.
- HP-2: Approved tasks still expose publish state correctly before and after
  publication.
- HP-3: Playback retry and the back-navigation button still behave the same.

**Edge cases considered:**
- EC-1: Pending tasks still hide the publish action.
- EC-2: Rejected tasks still show the rejected publication panel.
- EC-3: Mutation errors still surface through the inline alert text.

**Inputs:**
- `mobile/src/screens/ReviewDetailScreen.tsx`
- `mobile/__tests__/ReviewDetailScreen.test.tsx`
- GH / local lint evidence pointing at `ReviewDetailScreen` line-count overflow

**Outputs:**
- `ReviewDetailScreen` reduced below 60 lines
- extracted summary / playback / publication section(s) where helpful
- focused tests updated only if structural extraction requires it

**Acceptance criteria:**
- ESLint no longer reports `max-lines-per-function` for `ReviewDetailScreen`
- approve / reject / publish behavior remains intact
- playback and publication panels keep current `testID` coverage

**Files expected to change:**
- `mobile/src/screens/ReviewDetailScreen.tsx`
- `mobile/__tests__/ReviewDetailScreen.test.tsx`

**Agent handoff prompt:** Decompose `ReviewDetailScreen` under the 60-line gate
while preserving review-state behavior, playback loading, publication sections,
and button/testID coverage.

### Reflection log

Required passes: 2 (`28` → `Moderate`)

#### Pass 1

- **Draft verdict:** `ReviewPublicationSection` extraído (3 paneles de publicación → 1 línea); función principal en 47 líneas.
- **Critique findings:** Lógica `approved && publishedAt` / `approved && !publishedAt` / `rejected` equivalente — nuevo componente usa early-returns en mismo orden; todos los testIDs preservados (`review-publish-pending-panel`, `review-publish-pending-reason`, `review-rejected-panel`, `review-rejected-reason`).
- **Revisions applied:** Ninguna.

#### Pass 2

- **Draft verdict:** Tests 61/61 verdes; lint limpio.
- **Critique findings:** EC-4 (pending oculta publish action): `ReviewPublicationSection` solo retorna panel publish cuando `taskState === "approved"`, nunca para `"pending"`. EC-5/6 con testIDs intactos verificados en tests.
- **Revisions applied:** Ninguna.

### Gemma Reviewer evidence

(compartido con T1 — mismo packet de Phase 2, diff cubre ambas pantallas)

- Model: `gemma4:26b-a4b-it-qat`
- Command: `python3 scripts/gemma-code-review.py phase2-review-packet.md --passes 1`
- Passes run / usable: `1/1`
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` (para ReviewDetailScreen)
- Isolated adjudicator: `not triggered` — trigger: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: Sin hallazgos para este componente.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-4 | Happy path | pending review shows playback, decision input, approve/reject bars | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-1 pending task` | passed |
| HP-5 | Happy path | approved review exposes publish state before/after publication | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-2 approved task` | passed |
| HP-6 | Happy path | playback retry and back-navigation intact | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-3 playback retry` | passed |
| EC-4 | Edge case | pending tasks hide publish action | `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-1 pending hides publish` | passed |
| EC-5 | Edge case | rejected tasks show rejected publication panel | `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-2 rejected panel` | passed |
| EC-6 | Edge case | mutation errors surface through inline alert | `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-3 mutation error` | passed |

---

## T3 — Verification and status sync

**Effort:** S (docs + verification)  
**Depends on:** T1, T2  
**Status:** [x] Done

**Happy paths considered:**
- HP-1: Focused mobile tests pass and `make qa-mobile` returns green.
- HP-2: Daily and bug artifacts reflect the repaired status.

**Edge cases considered:**
- EC-1: If `qa-mobile` still fails, the remaining failing file/rule is recorded
  explicitly instead of marking the bug closed.

**Inputs:**
- T1/T2 code changes
- `docs/daily/2026-07-01.md`

**Outputs:**
- green local verification evidence
- task record updated to closed with evidence
- daily blocker updated from open issue to resolved or narrowed follow-up

**Acceptance criteria:**
- `cd mobile && npx eslint src/screens/AssetListScreen.tsx src/screens/ReviewDetailScreen.tsx` passes
- focused screen tests pass
- `make qa-mobile` passes
- `make qa-docs` passes

**Files expected to change:**
- `docs/tasks/bug-mobile-gate-max-lines.md`
- `docs/daily/2026-07-01.md`

**Agent handoff prompt:** Verify the refactor with focused mobile tests plus the
full `qa-mobile` gate, then sync the bug record and daily issue state before
reporting completion.
