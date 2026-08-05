---
type: TaskList
status: done
---

# Handoff: Antares Phase B — comparative experiment (harness vs. antares-cli)

**Done 2026-08-05.** Comparison artifact:
`docs/evaluations/antares-phase-b-comparison.md`. Decision-points table row
updated in `docs/plan/antares-local-runtime-adoption.md`. Result: the
harness cannot consume real Antares wire-format output today (confirmed
empirically). This unblocks
`docs/tasks/handoff-antares-element3-2026-08-05.md`.

1. **Task**: Phase B comparative experiment (no task ID — pre-Element-3 gate).
   Goal: run the same fixed CWE fixture through both invocation paths and
   record what each produces, to convert the Element 3 decision from
   speculation into measurement. Read-only evaluation; no approval required
   (no tracked code changes, no governed path touched).
2. **Governing docs**: `docs/plan/antares-local-runtime-adoption.md` §
   "Element 3", § "Proposed sequence" (Phase B row), § "Blocking finding: the
   wire-format translation layer was never written".
3. **Files/paths involved**:
   - `scripts/antares/harness.py:201` (`dispatch_tool_call`) — the existing
     harness entrypoint; expects the internal `{"tool": ..., "payload": ...}`
     schema, not real model wire format (`<tool_call>` tags with
     `name`/`arguments`).
   - `scripts/antares/replay_fixtures.py:15` (`_msg`) — builds the internal
     schema; do not treat its fixtures as live-model-shaped input for this
     experiment.
   - `antares tool query --stdin` CLI (external, already validated working
     per T1 R4/R5 — see `docs/evaluations/antares-runtime-preflight.md`
     "R4/R5 execution record" for the exact invocation contract:
     `{"target": <dir>, "cwe_ids": [...], "profile": "antares-local"}`).
4. **Exact steps**:
   - Pick one fixed fixture already used for T1 R4/R5 (`apps/` or `crates/`
     scope, `CWE-20`) so results are comparable to existing evidence.
   - Path A: run the fixture through `antares tool query --stdin` (already
     proven working; reuse the exact command from the R4/R5 record).
   - Path B: run the *same* fixture's raw model tool-call output (if
     obtainable) through `scripts/antares/harness.py::dispatch_tool_call`.
     If the harness cannot consume real wire-format output at all (expected,
     per the "Blocking finding" section), record that as the Path B result
     rather than fabricating a translation layer to make it run — Phase B
     measures the gap, it does not close it.
   - Record for each path: whether it completed, what schema/fields it
     required vs. what was actually available, and any manual translation
     that would be needed to make Path B work.
5. **Acceptance criteria**:
   - A written comparison artifact (e.g.
     `docs/evaluations/antares-phase-b-comparison.md`, OKF frontmatter
     required) covering both paths against the same fixture.
   - Explicitly states whether the harness path can consume real Antares
     wire-format output today, with evidence (not inference).
   - Does not modify `scripts/antares/*.py` or any tracked production code —
     Phase B is read-only per the plan's approval boundary.
6. **Stop condition**: once the comparison artifact is written and the
   Decision-points table row ("Does the existing T2 harness work against
   real Antares output?") in
   `docs/plan/antares-local-runtime-adoption.md` is updated with the
   resolution, stop. Do **not** proceed to scoring or implementing Element 3
   — that is a separate, gated task requiring its own RRI, task card, and
   explicit human approval.
