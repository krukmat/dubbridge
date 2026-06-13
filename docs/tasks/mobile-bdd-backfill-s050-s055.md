# Tasks: Mobile BDD Backfill for S-050 and S-055

**Plan:** `docs/plan/mobile-bdd-backfill-s050-s055.md`
**Slices covered:** `S-050`, `S-055`
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
MBF-T0 (mobile BDD convention + index) ─┬─▶ MBF-T1 (S-050 feature + mapping) ─┐
                                         └─▶ MBF-T2 (S-055 feature + mapping) ─┼─▶ MBF-T3 (ledger sync)
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| MBF-T0 | Mobile BDD convention + index normalization | — | 13 | Low | S | ✅ done 2026-06-12 |
| MBF-T1 | S-050 retrospective BDD `.feature` + mapping | MBF-T0 | 13 | Low | S | ✅ done 2026-06-12 |
| MBF-T2 | S-055 retrospective BDD `.feature` + mapping | MBF-T0 | 13 | Low | S | ✅ done 2026-06-12 |
| MBF-T3 | Ledger cross-references + consistency sync | MBF-T1, MBF-T2 | 13 | Low | S | ✅ done 2026-06-12 |

## Model resolution

| Band | Codex | Claude Code | Thinking |
|---|---|---|---|
| Low (0–25) | `GPT-5.2-Codex` | `Claude Haiku 4.5` | Off |

## Contract correction note

The original backfill intent was correct, but the artifact contract needs to be
read precisely:

- mobile-only slices live in `mobile/bdd/`;
- cross-surface slices live in `docs/bdd/`;
- retrospective slices may map to shipped unit/integration evidence or runner
  artifacts when no standalone Maestro flow exists;
- mobile-first executable slices still map to Maestro flows when they own that
  executable surface.

Under that contract, `S-050` is retrospective and test-evidence-backed, `S-055`
is retrospective and Maestro/runner-backed, `S-060` remains the mobile-first
Maestro precedent in `mobile/bdd/`, and `S-100` remains cross-surface in
`docs/bdd/`.

---

## MBF-T0 — Mobile BDD convention + index normalization

- **Status:** [x] Done — 2026-06-12
- **Type:** Planning / docs · **Effort:** S
- **RRI:** 13 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** —
- **Objective:** Normalize the mobile BDD home so `S-050`, `S-055`, and `S-060`
  can coexist under `mobile/bdd/` with one index and clear per-slice spec files.
- **Inputs:** `mobile/bdd/README.md`, `docs/tasks/s-050-mobile-client.md`,
  `docs/tasks/s-055-maestro-screenshot-suite.md`, `docs/tasks/s-060-mobile-asset-lifecycle.md`.
- **Outputs:**
  - updated `mobile/bdd/README.md` as a multi-slice mobile BDD index;
  - naming convention for new per-slice `.feature` files.
- **Acceptance criteria:**
  - The index no longer reads as S-060-only.
  - The file-home decision for `S-050` and `S-055` is explicit.
  - Existing `S-060` mappings remain intact and correct.
  - The convention distinguishes mobile-first Maestro-backed slices from
    retrospective slices that map to existing evidence instead of standalone
    Maestro flows.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 1 → 0 | High |
  | F | 2 | 3 docs/files | High |
  | D | 0 | docs convention only | High |
  | T | 2 | qa-docs / mapping review | High |
  | A | 0 | criteria explicit | High |
  | K | 0 | no code coupling | High |
  | P | 0 | no runtime impact | High |
  | X | 2 | few docs | High |

  **Base 13 · penalties none · Final 13 → Low → auto-execute.**

### Completion record (2026-06-12)

- Reworked `mobile/bdd/README.md` from an `S-060`-only note into a multi-slice
  mobile BDD index.
- Recorded the file-home decision that mobile-only retrospective specs for `S-050`
  and `S-055` live under `mobile/bdd/` as one file per slice.
- Reserved explicit index entries for `S-050` and `S-055` backfill without
  inventing scenario IDs before the actual `.feature` work in `MBF-T1` / `MBF-T2`.
- Preserved the existing `S-060` mapping table unchanged at the scenario level.

---

## MBF-T1 — S-050 retrospective BDD `.feature` + mapping

- **Status:** [x] Done — 2026-06-12
- **Type:** Planning / docs · **Effort:** S
- **RRI:** 13 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** MBF-T0
- **Objective:** Author a behavioral `.feature` file for the shipped `S-050` mobile
  client and map each scenario to existing unit/integration evidence.
- **Inputs:** `docs/tasks/s-050-mobile-client.md`, `mobile/__tests__/auth.provider.test.tsx`,
  `mobile/__tests__/asset.screens.test.tsx`, `mobile/__tests__/mobile.auth-flow.test.tsx`.
- **Outputs:**
  - `mobile/bdd/s-050-mobile-client.feature`
  - `mobile/bdd/README.md` rows for `S-050`
- **Acceptance criteria:**
  - Scenarios cover the delivered surface: auth entry, authenticated home, asset list,
    asset detail, and fail-closed token/session handling.
  - Each scenario maps only to evidence that exists today.
  - Scenario language is behavioral, not implementation-specific.
  - The mapping remains retrospective: `S-050` does not promise a standalone
    Maestro flow where none exists.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 1 → 0 | High |
  | F | 2 | 3 docs/files | High |
  | D | 0 | docs authoring | High |
  | T | 2 | mapping to existing tests | High |
  | A | 0 | criteria explicit | High |
  | K | 0 | no code coupling | High |
  | P | 0 | no runtime impact | High |
  | X | 2 | few docs | High |

  **Base 13 · penalties none · Final 13 → Low → auto-execute.**

### Completion record (2026-06-12)

- Added `mobile/bdd/s-050-mobile-client.feature` with six retrospective
  behavioral scenarios covering the shipped mobile surface:
  `SC-AUTH-1`, `SC-AUTH-2`, `SC-AUTH-3`, `SC-NAV-1`, `SC-ASSET-1`, `SC-ASSET-2`.
- Updated `mobile/bdd/README.md` with the `S-050` mapping rows, pointing only to
  evidence that already exists in the `S-050` test suite.
- Kept the backfill strictly retrospective: no new runtime flows were invented, no
  standalone Maestro entry was promised where none exists today, and the mapping
  stays anchored to shipped test evidence.

---

## MBF-T2 — S-055 retrospective BDD `.feature` + mapping

- **Status:** [x] Done — 2026-06-12
- **Type:** Planning / docs · **Effort:** S
- **RRI:** 13 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** MBF-T0
- **Objective:** Author a behavioral `.feature` file for the shipped `S-055` two-phase
  Maestro suite and map each scenario to the existing flows, runner, and sanitizer.
- **Inputs:** `docs/tasks/s-055-maestro-screenshot-suite.md`, `mobile/maestro/auth-surface.yaml`,
  `mobile/maestro/authenticated-audit.yaml`, `mobile/maestro/seed-and-run.sh`,
  `mobile/maestro/README.md`.
- **Outputs:**
  - `mobile/bdd/s-055-maestro-suite.feature`
  - `mobile/bdd/README.md` rows for `S-055`
- **Acceptance criteria:**
  - Scenarios cover phase 1 auth capture, phase 2 authed capture, and artifact sanitization.
  - Each scenario maps only to flows/checks that exist today.
  - The suite is represented behaviorally, not as shell-command prose.
  - The mapping explicitly points to existing Maestro flows and runner/sanitizer
    artifacts as the shipped verification surface.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 1 → 0 | High |
  | F | 2 | 3 docs/files | High |
  | D | 0 | docs authoring | High |
  | T | 2 | mapping to existing runner/flows | High |
  | A | 0 | criteria explicit | High |
  | K | 0 | no code coupling | High |
  | P | 0 | no runtime impact | High |
  | X | 2 | few docs | High |

  **Base 13 · penalties none · Final 13 → Low → auto-execute.**

### Completion record (2026-06-12)

- Added `mobile/bdd/s-055-maestro-suite.feature` with three retrospective
  behavioral scenarios covering the shipped `S-055` suite:
  `SC-SUITE-1`, `SC-SUITE-2`, `SC-SUITE-3`.
- Updated `mobile/bdd/README.md` with the `S-055` mapping rows, pointing to the
  existing phase YAMLs and to the runner sanitization evidence already delivered.
- Kept the backfill behavioral and retrospective: the spec describes the suite's
  observable guarantees, and its verification remains backed by shipped Maestro
  flows plus runner/sanitizer artifacts.

---

## MBF-T3 — Ledger cross-references + consistency sync

- **Status:** [x] Done — 2026-06-12
- **Type:** Docs sync · **Effort:** S
- **RRI:** 13 → band **Low (0–25)** → **auto-execute**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Haiku 4.5` · thinking Off
- **Depends on:** MBF-T1, MBF-T2
- **Objective:** Cross-link the new BDD sources of truth from the historical `S-050`
  and `S-055` ledgers and leave the mobile BDD inventory internally consistent.
- **Inputs:** MBF-T1 and MBF-T2 outputs; `docs/tasks/s-050-mobile-client.md`;
  `docs/tasks/s-055-maestro-screenshot-suite.md`.
- **Outputs:** updated cross-references in the two historical ledgers and any needed
  note in `mobile/bdd/README.md`.
- **Acceptance criteria:**
  - Readers of `S-050` and `S-055` can discover the retrospective BDD specs quickly.
  - No mapping row points to a missing file.
  - `make qa-docs` passes if no unrelated repo issue blocks it.
  - The cross-references describe the correct verification shape for each slice:
    shipped test evidence for `S-050`, shipped Maestro/runner artifacts for `S-055`.
- **RRI variable table (script output):**

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C | 0 | raw CC 1 → 0 | High |
  | F | 2 | 3 docs/files | High |
  | D | 0 | docs sync only | High |
  | T | 2 | qa-docs / mapping review | High |
  | A | 0 | criteria explicit | High |
  | K | 0 | no code coupling | High |
  | P | 0 | no runtime impact | High |
  | X | 2 | few docs | High |

  **Base 13 · penalties none · Final 13 → Low → auto-execute.**

### Completion record (2026-06-12)

- Added explicit retrospective BDD cross-references to
  `docs/tasks/s-050-mobile-client.md` and
  `docs/tasks/s-055-maestro-screenshot-suite.md`, so readers of the historical
  ledgers can discover the new `.feature` sources of truth directly.
- Verified that `mobile/bdd/README.md` now points only to existing files for
  `S-050`, `S-055`, and `S-060`.
- Closed the mobile BDD backfill loop without rewriting the original slice
  execution histories.
