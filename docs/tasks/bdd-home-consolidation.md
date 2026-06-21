---
type: TaskList
title: "Tasks: BDD Home Consolidation to docs/bdd"
status: closed
plan: docs/plan/bdd-home-consolidation.md
---
# Tasks: BDD Home Consolidation to `docs/bdd`

**Plan:** `docs/plan/bdd-home-consolidation.md`
**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`, `AGENTS.md`.

## Status legend

- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
BDC-T1 (canonicalize BDD home + move/delete + reference sync)
```

| Task | Title | Depends on | RRI | Band | Effort |
|---|---|---|---|---|---|
| BDC-T1 | Canonicalize BDD home and consolidate mobile specs into `docs/bdd/` | — | 33 | Moderate | M |

---

## BDC-T1 — Canonicalize BDD home and consolidate mobile specs into `docs/bdd/`

- **Status:** [x] Done — 2026-06-21
- **Type:** Docs / repository structure
- **RRI:** 33 → band **Moderate (26-40)** → **approval required before execution**
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4` · thinking Off
- **Objective:** Make `docs/bdd/` the only repository home for BDD artifacts by
  moving canonical specs out of `mobile/bdd/`, deleting redundant out-of-home
  copies, rewriting references, and preserving the scenario source of truth.
- **Inputs:** `docs/bdd/`, `mobile/bdd/`, historical BDD plans/tasks that encode
  the split-home rule.
- **Outputs:**
  - moved or retained `.feature` files only under `docs/bdd/`;
  - retired duplicate or superseded files removed from `mobile/bdd/`;
  - rewritten repository references that now resolve to `docs/bdd/*`;
  - a clear duplicate-disposition record in this ledger's completion note.
- **Acceptance criteria:**
  - No `.feature` BDD source remains outside `docs/bdd/`.
  - No surviving `mobile/bdd/*` BDD file duplicates a `docs/bdd/*` source.
  - Scenario IDs and behavioral text are preserved except for path-only moves.
  - Any deletion is justified as duplicate retirement or post-move cleanup, not
    silent content loss.
  - Repository references to moved BDD specs resolve to `docs/bdd/*`.
  - `docs/bdd/README.md` covers the moved mobile scenarios or clearly points to
    their canonical files.
  - No materially governing document still states that `mobile/bdd/` is a valid
    canonical BDD home.

### Completion record (2026-06-21)

- Moved `mobile/bdd/s-050-mobile-client.feature` to
  `docs/bdd/s-050-mobile-client.feature`.
- Moved `mobile/bdd/s-055-maestro-suite.feature` to
  `docs/bdd/s-055-maestro-suite.feature`.
- Moved `mobile/bdd/asset-lifecycle.feature` to
  `docs/bdd/s-060-mobile-asset-lifecycle.feature`.
- Replaced the old mobile BDD index with a canonical `docs/bdd/README.md`
  covering `S-050`, `S-055`, and `S-060` alongside the existing `docs/bdd`
  entries.
- Rewrote affected repository references so they resolve to `docs/bdd/*`.
- Duplicate disposition: no pre-existing duplicate `.feature` copies were found
  in `docs/bdd/`; all three mobile specs were canonical-only moves, not
  duplicate deletions.
