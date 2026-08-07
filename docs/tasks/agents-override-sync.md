---
type: TaskList
title: "Tasks: AGENTS.override.md generation and drift guard"
status: proposed
plan: docs/plan/agents-override-sync.md
---
# Tasks: AGENTS.override.md generation and drift guard

Governing plan: `docs/plan/agents-override-sync.md`
Governing guides: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/RRI_POLICY.md`,
`docs/policies/HITL_AUTONOMY_POLICY.md`
Related: `docs/tasks/antares-security-specialist-advisor.md` § T6,
`docs/tasks/doc-consistency-guardrails.md` (precedent pattern)

## Status Legend
- [ ] Not started · [x] Done · [~] In progress · [!] Blocked

Build order: **AOS1** (single task; no decomposition required at RRI 51).

---

## Task AOS1 — Generate AGENTS.override.md + wire drift check

**Effort:** L (RRI band-derived, Med-high) · **Depends on:** nothing

### Objective
Turn `AGENTS.override.md` from a manually-mirrored duplicate into a generated
artifact with an enforced freshness gate, and close its existing content gap
(missing `HITL_AUTONOMY_POLICY.md` section) in the same change.

### Happy paths considered
- **HP-1:** `AGENT_WORKFLOW_GUIDE.md` (or `AGENTS.md`, or `HITL_AUTONOMY_POLICY.md`)
  is edited, `scripts/generate-agents-override.py` is run, and the regenerated
  `AGENTS.override.md` is byte-identical to the fixed concatenation of the three
  current source files with the `---` seam — `make qa-docs` passes afterward.
- **HP-2:** After the one-time regeneration, `AGENTS.override.md` contains the
  `HITL_AUTONOMY_POLICY.md` content (verifiable via
  `grep -c "Human-in-the-Loop (HITL) Autonomy Policy" AGENTS.override.md` -> `1`),
  closing the gap found during investigation.
- **HP-3:** Running the generator twice in a row with no source changes produces
  byte-identical output both times (idempotent) — no spurious drift-check failure
  from running it more than once.

### Edge cases considered
- **EC-1:** `AGENTS.override.md` is hand-edited (or a source file changes) without
  re-running the generator — the new drift check in
  `scripts/check-doc-consistency.sh` fails closed with a message naming the exact
  command to fix it (`scripts/generate-agents-override.py`), and `make qa-docs`
  exits non-zero.
- **EC-2:** One of the three source files (`AGENTS.md`, `AGENT_WORKFLOW_GUIDE.md`,
  `HITL_AUTONOMY_POLICY.md`) is missing or empty at generation time — the generator
  exits non-zero with a clear error and does **not** write a partial or empty
  `AGENTS.override.md`.
- **EC-3:** The drift check runs in a repo state where `AGENTS.override.md` does
  not exist yet (e.g. a fresh checkout that somehow lost the file) — it fails
  closed with the same "stale, regenerate" message rather than silently passing or
  crashing with an unhandled exception.

### Acceptance criteria
- `scripts/generate-agents-override.py` exists, is deterministic (no LLM calls),
  and implements HP-1/HP-3 and EC-2.
- `scripts/check-doc-consistency.sh` gains a drift-check function reusing the
  generator's output (not a re-implementation of the concatenation logic) and
  implements EC-1/EC-3; it is exercised by the existing `qa-docs` Makefile target
  with no new Makefile target added.
- `AGENTS.override.md` is regenerated once as part of this task; the only content
  delta versus the pre-task file is the addition of the
  `HITL_AUTONOMY_POLICY.md` section (HP-2) — no unrelated reformatting.
- `make qa-docs` passes after the regeneration.

### Evidence to emit
- `git diff --stat` for the regenerated `AGENTS.override.md` (expected: pure
  addition, no reordering of existing lines).
- `make qa-docs` full output (showing the new check running and passing).
- A deliberately-introduced drift (e.g. a throwaway edit to
  `AGENT_WORKFLOW_GUIDE.md` without regenerating) demonstrating the check fails
  closed, then reverted/regenerated to show it passes again.

### Status artifacts affected
- `docs/tasks/antares-security-specialist-advisor.md` § T6 — add a forward
  reference noting the manual-mirror gap T6 hit is now closed by this task's
  drift check (informational only; does not reopen T6).
- No ADR change; no roadmap slice change (cross-cutting tooling, not on the
  media-pipeline sequence).

### Files affected
- `scripts/generate-agents-override.py` (new)
- `scripts/check-doc-consistency.sh` (new drift-check function)
- `AGENTS.override.md` (regenerated)

---

### RRI

```
python3 scripts/rri.py \
  --touches scripts/generate-agents-override.py \
  --touches scripts/check-doc-consistency.sh \
  --touches AGENTS.override.md \
  --touches docs/plan/agents-override-sync.md \
  --touches docs/tasks/agents-override-sync.md \
  --cc 8 --D 2 --K 3 --P 2 --T 3 --A 1 --X 2 \
  --penalty arch_decision
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 8 (estimated: generator ~3-4 branches, drift-check ~4-5 branches) -> score 1 | High |
| F files | 2 | 5 touched paths | High |
| D domain | 2 | agent-supplied — no anchor-rubric match for `scripts/`; judged as normal business logic (concatenation + diff), not pure formatting, because it feeds a governance hash-attestation gate | High |
| T coverage | 3 | agent-supplied — new mechanism, no existing tests, but sits inside an already-exercised script family (`check-doc-consistency.sh` has prior test precedent via G2) | High |
| A ambiguity | 1 | agent-supplied — explicit acceptance criteria, HP/EC set, and design decisions recorded above | High |
| K coupling | 3 | agent-supplied — filesystem I/O + integration into an existing CI-facing gate (`make qa-docs`) | High |
| P impact | 2 | agent-supplied — changes internal dev-tooling behavior only; not public API, auth, or persisted user data | High |
| X context | 2 | agent-supplied — must hold ~5 docs/scripts in mind (3 sources + 2 scripts) | High |

**Base value:** 100 x (weighted / 5) = 39
**Penalties applied:** `arch_decision` (+12) — this changes the maintenance model of
Codex's hash-attested native-instruction source from hand-maintained to generated,
a process/governance-tooling decision, not just a code change.
**Final RRI:** 51 -> band **Med-high (41-55)** -> Effort L
**Gates for this band:** Plan + explicit acceptance criteria required before
approval (satisfied above); ADR-038 Architect-refined single-attempt implementation
gate; band-routed peer review (phases 1 and 2); 3 Reflection passes.
**Decomposition:** not triggered.

---

## Agent handoff prompt (for delegation)

```
Task: AOS1 — docs/tasks/agents-override-sync.md
Plan: docs/plan/agents-override-sync.md

File + line range: AGENTS.override.md:1-1710 (read-only reference for seam format
at line 225); new files scripts/generate-agents-override.py,
scripts/check-doc-consistency.sh (add function, do not restructure existing checks).

Acceptance criteria: see "Acceptance criteria" above (bullets only).

Stop condition: after `make qa-docs` passes and the regenerated
AGENTS.override.md's diff is pure-addition (HITL section only), stop and report.
Do not touch AGENT_WORKFLOW_GUIDE.md, HITL_AUTONOMY_POLICY.md, or AGENTS.md content
— this task only changes how AGENTS.override.md is produced/verified.
```
