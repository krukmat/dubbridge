---
type: TaskList
status: in_progress
---

# Handoff: Antares Element 3 — `scripts/antares/*` reconciliation

**Decomposed; Subtasks A and C closed 2026-08-05.** RRI 58 (Complex)
triggered mandatory decomposition — see
`docs/audit/antares-t4-element3-rri.md` for the full pre-decomposition
score, T2a–T2e disposition, and the 3-way split (Subtask A route decision /
Subtask B implementation / Subtask C disposition doc sync). Subtask A (RRI
26 Moderate) is approved and closed: the plan's Element 3 section and
decision-points table in `docs/plan/antares-local-runtime-adoption.md` now
record the resolved route — adopt `antares tool query --stdin`/`sweep
--stdin` as direct CLI subprocess calls, retire the harness's
live-invocation role, retain T2a–T2e as the synthetic-fixture/replay-test
path only. Subtask C (RRI 18 Low) is approved and closed: T2a and T2e's
rows in `docs/tasks/antares-security-specialist-advisor.md` now carry the
same narrowed-disposition note inline, and T2e gained a full disposition
subsection cross-referencing T2a's existing correction and this handoff's
resolved route.

Remaining work under this handoff: **Subtask B** (implement the resolved
route — requires its own rescore against the actual diff and its own
approval; the RRI 48 figure in the audit artifact was an explicit
placeholder for the *other* branch of the decision and does not apply). Not
started; no code in `scripts/antares/*.py` has been touched.

Original framing preserved below for provenance.

**Unblocked 2026-08-05.** Phase B's comparison artifact now exists:
`docs/evaluations/antares-phase-b-comparison.md`. Result: the existing
harness cannot consume real Antares wire-format output — confirmed
empirically (three real-shaped inputs rejected as `MALFORMED_TOOL_CALL`);
no translation layer exists anywhere in `scripts/antares/*`. This resolves
the "add a subprocess-invocation layer vs. write a translation layer"
framing in favor of: adopting the CLI (or an equivalent translation layer)
is the only way to make live invocation work at all — the harness's own
schema was never real. This task card still needs to be drafted, scored
with `scripts/rri.py` against the resolved scope, and presented for
explicit human approval before any implementation — nothing here
authorizes editing `scripts/antares/*`.

1. **Task**: Element 3 (Phase D) — reconcile `scripts/antares/*`'s
   invocation model against `antares tool query --stdin` / `antares tool
   sweep --stdin`. Goal: decide and implement whether the harness invokes
   the CLI as a subprocess (replacing direct model-stream consumption) or
   is retired/narrowed in favor of the CLI path outright.
2. **Governing docs**: `docs/plan/antares-local-runtime-adoption.md` §
   "Element 3", § "Orchestration and cross-plan dependencies" (dependency
   graph + decision points table), `docs/audit/antares-t3c-1-rri.md` (RRI
   precedent for `arch_decision` penalty on this same code area).
3. **Files/paths likely in scope** (confirm against Phase B evidence before
   finalizing — do not assume this list is complete):
   - `scripts/antares/harness.py` (418 lines) — invocation entrypoint,
     `dispatch_tool_call` at line 201.
   - `scripts/antares/tool_call_parser.py` (169 lines) + its test — scope
     decision required: narrow to replay-fixture/synthetic-test path only,
     or retire, per the plan's open decision.
   - `scripts/antares/terminal_state.py` (126 lines) + its test — same open
     scope decision as above.
   - `scripts/antares/replay_fixtures.py`, `harness_test.py` — likely
     survive as the synthetic-test path regardless of the decision.
   - Five completed Med-high T2 subtasks (T2a–T2e) — the task card must
     explicitly decide their disposition: narrow to test-only, retire, or
     retain as the invocation path. Do not let this be an implicit side
     effect of the code change.
4. **Before drafting the card**:
   - Read the Phase B comparison artifact in full; it determines whether
     this task is "add a subprocess-invocation layer" or "retire the
     harness's live-invocation path and keep it as a test fixture consumer
     only" — these are different-sized changes and must not be scored
     interchangeably.
   - Run `scripts/rri.py` against the *actual* resolved scope (not the
     unresolved either/or currently in the plan). Expect the
     `arch_decision` penalty to apply (cross-file architecture decision on
     security-relevant parsing code, per the T3c-1 precedent).
   - If the resulting RRI is 56+, mandatory decomposition applies before
     implementation — do not draft a single monolithic task card.
5. **Acceptance criteria for this handoff** (preparing the card, not
   implementing):
   - Task card exists with RRI computed against Phase-B-resolved scope.
   - T2a–T2e disposition is an explicit stated decision in the card, not
     left implicit.
   - If RRI ≥ 56, the card shows the decomposition, not a single task.
6. **Stop condition**: once the task card is drafted and RRI is computed,
   stop and present it for explicit human approval per
   `docs/policies/HITL_AUTONOMY_POLICY.md` (RRI > 25 always requires
   approval, even mid-session). Do **not** start implementation.
