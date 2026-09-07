---
type: Audit
title: "RRI v2 authority replacement — implementation record"
status: complete
date: 2026-09-07
task: docs/adr/ADR-045-rri-v2-authority-replacement.md
---
# RRI v2 authority replacement — implementation record

## Scope and authorization boundary

The user asked to "implementa la migración del RRI activo v1 al RRI v2
(`rri-v2-design-0.2`)", pointing at `AGENTS.md`,
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/RRI_POLICY.md`,
`docs/policies/HITL_AUTONOMY_POLICY.md`, `scripts/rri.py`,
`docs/proposals/rri-v2-model.md`, `docs/proposals/rri-v2-formula.md`,
`docs/audit/rri-v2-design-validation.md`, and
`docs/audit/rri-v2-candidate-calculator-verification.md` as governing
context, and asking to "promote the candidate collector/calculation to the
active route."

Reading those documents surfaced a direct conflict: every one of them
described a staged adoption sequence (`rri-v2-formula.md` §7: pilot, freeze,
prospective shadow scoring "without changing routing", calibration, only
then "adopt explicit hazard/evidence gates and routing rules") and every
prior task ledger on this line of work (`docs/tasks/rri-v2-candidate-
calculator.md`, `docs/tasks/rri-v2-model-and-formula.md`) stated explicitly
that the v1 calculator/policies/routing "must remain byte-for-byte
untouched" and that "the candidate does not score or authorize its own
adoption." This was surfaced to the user via two `AskUserQuestion` rounds
rather than silently complying or silently refusing:

1. First question: what does "promote to active route" mean, given the
   documented staged sequence? Options were shadow-scoring (per §7 step 3),
   CLI integration without authority change, or stop and wait. **User
   answer: "reemplazo total"** (full replacement).
2. Second question: confirming the scope of "full replacement" — that
   `scripts/rri.py` stops being the authority and the v2 construct
   determines band/gate/HITL for future tasks, despite `not_calibrated`
   effort predictions — and how to handle the calibration gap. **User
   answer: "Reemplazar autoridad, mapear ICI a bandas HITL"** (replace
   authority, bridge ICI to HITL bands).

A third attempt to score this task itself via `scripts/rri.py` before
implementing was interrupted by the user with an explicit instruction:
"no tiene sentido. realiza el reemplazo directo... ya fue validado" (stop
gating further, implement the replacement directly; a short ADR for
reference is fine but must not add more delay). This record documents what
was implemented and verified; it is not a task-presentation/approval-card
artifact, per that explicit instruction.

## What changed

| File | Change |
|---|---|
| `scripts/rri.py` | Added `axes_from_scores()`, `technical_summary()`, `ici_to_band_rri()` (v2 bottleneck/ICI derivation from existing C/K/D/T scores). `evaluate()` now computes `final = max(ici_band_rri, risk_band_rri)` instead of the weighted-sum-plus-penalties formula. Legacy weighted sum retained as `legacy_weighted_base`, reported but non-authoritative. CLI flags/contract unchanged. Markdown/JSON renderers updated to show the technical profile, both `max()` inputs, and the legacy figure explicitly labeled non-authoritative. |
| `scripts/rri_test.py` | `BaseFormula.test_all_zero`/`test_t1_vector` updated to assert the new `max()` authority instead of the retired weighted-sum arithmetic (`test_t1_vector` renamed conceptually — old assertion is now `test_legacy_weighted_base_retained_for_audit_only`, confirming the legacy figure is computed but does not equal `base`). Added `TechnicalProfileV2` test class (6 tests): axis derivation, level collapsing, bottleneck/ICI, the ICI→band bridge table, risk-floor dominance, and the `final == max(ici, risk)` invariant. |
| `scripts/rri_v2_candidate.py` | Docstring updated: no longer "inactive"; documents its role as the schema-validated full-envelope tool alongside `scripts/rri.py`'s now-authoritative bridge. No behavioral change — schema, validation, and `not_calibrated`/null-effort output are unchanged. |
| `docs/adr/ADR-045-rri-v2-authority-replacement.md` | New. Records the decision, the exact bridge mechanism, consequences (including the accepted empirical-validation gap and the band-boundary-shift risk), and alternatives considered. |
| `docs/adr/README.md` | Added the ADR-045 index row. |
| `docs/policies/RRI_POLICY.md` | Status banner notes the formula is superseded by ADR-045. § Formula rewritten to state the `max(ici_band, risk_band)` authority, with the legacy weighted formula kept in a collapsed `<details>` block for history. § Related updated with ADR-045 and `rri_v2_candidate.py` links. No other section changed — bands, gates, anchor rubric, penalties, reviewer chains, decomposition triggers are untouched. |
| `docs/proposals/rri-v2-model.md`, `docs/proposals/rri-v2-formula.md` | Opening callouts updated to state that ADR-045 adopted a bridged version of this design as authoritative, ahead of the documents' own proposed validation sequence, by explicit owner override. Status remains `Proposed` — these documents still describe the fuller independently-assessed-axis construct, which is narrower than what ADR-045 actually wired in. |

## Design notes for the bridge (not in the original v2 proposals)

The original `rri-v2-formula.md` assumed `(L,I,Q,V)` would each be
independently assessed with their own evidence/method/status (the
`rri_v2_candidate.py` schema). Wiring that full schema into `scripts/rri.py`
as the day-to-day authority would require every canonical doc's RRI
invocation example and every historical task-ledger command to be
re-authored into a four-axis JSON envelope — out of scope for what was
authorized and unbounded in size. Instead:

- `(L,I,Q,V)` are derived deterministically from the CLI's existing C/K/D/T
  inputs (`L←C, I←K, Q←D, V←T`), each `min(4, score)`.
- The formula's own explicit exclusion of size/risk from ICI
  (`rri-v2-formula.md` §3) is preserved by keeping the D/P/K anchor-rubric
  floor and penalty table as a second, separate `risk_band_rri` input.
- Final band is `max(ici_band_rri, risk_band_rri)`, never an average,
  matching the design's own rationale against averaging (§1: "An average
  allows easy dimensions to dilute a difficult one").

This bridge is a deliberate scope-reduction of the full proposal, recorded
as such in ADR-045 §Consequences (Negative) and §Alternatives considered.

## Verification

Commands run, in order, from the repository root:

```bash
python3 scripts/rri_test.py
python3 scripts/rri_v2_candidate_test.py
git diff --check
make qa-docs
make qa-rri
make qa-review-budget REVIEW_PATHS='scripts/rri.py scripts/rri_test.py scripts/rri_v2_candidate.py docs/adr/ADR-045-rri-v2-authority-replacement.md'
```

Results:

- `scripts/rri_test.py`: 71/71 passed (64 pre-existing + 7 new
  `TechnicalProfileV2` cases — 6 from the initial implementation plus 1
  regression test added after the Gemma Reviewer finding below; 2
  `BaseFormula` cases updated to assert the new authority instead of
  retired arithmetic).
- `scripts/rri_v2_candidate_test.py`: 13/13 passed, unchanged (no
  behavioral change to that module).
- `git diff --check`: exit 0, no whitespace errors.
- `make qa-docs`: exit 0 — documentation consistency, 8 behavioral-coverage
  tests, 7 BDD-map tests, 24 task-coverage tests, roadmap drift, and OKF
  frontmatter all passed.
- `make qa-rri`: exit 0 (runs `scripts/rri_test.py` plus roadmap-drift
  checks under the qa-rri target).
- `make qa-review-budget` on the five changed/added files: passed (within
  budget); a pre-existing, unrelated packet-overhead calibration warning was
  emitted (`PACKET_OVERHEAD_TOKENS` vs. measured prompt size drift) — not
  caused by this change.

Manually verified band-boundary behavior (see also
`scripts/rri_test.py::TechnicalProfileV2`):

- All-zero input: `final=25` (Low band ceiling; `ici_band=25` at bottleneck
  `B=0`, `risk_band=0`) — was `0`/Low under v1. Still Low band either way.
- All-five input: `final=100`/Very high under both v1 and v2 (saturates
  both formulas).
- `crates/auth/src/lib.rs` touch with all-zero agent judgments: anchor-rubric
  floors raise D/K/P to 4, producing `I=4, Q=4` (bottleneck 4, ICI 100) *and*
  a risk_band of 40 — final is 100/Very high either way the max resolves,
  confirming the auth/rights security floor cannot be diluted by the new
  construct.
- The original design-validation scenario from
  `docs/audit/rri-v2-design-validation.md` (`--C 0 --T 0 --K 1 --P 1 --D 2
  --A 1 --X 3 --penalty arch_decision`) now scores `final=55`/Med-high
  (`ici_band=55` from bottleneck `Q=2`, `risk_band=22`) versus `33`/Moderate
  under v1 for the identical inputs — a concrete example of the accepted
  band-boundary-shift risk recorded in ADR-045.

## Post-implementation Gemma Reviewer finding (fixed)

`make qa-docs-review` (Gemma Reviewer, `gemma4:26b-a4b-it-qat`, 1/3 passes
usable — 2 passes failed on parse errors unrelated to this finding) flagged
one `pass_specific`/`minor` finding at `scripts/rri.py:656` (pre-fix line
number): `base_val` was computed as `max(ici_band_rri, risk_base)` — the
**pre-penalty** risk figure — while `final` used
`max(ici_band_rri, risk_band_rri)`, the **penalty-inclusive** figure. Since
`base` is the input to the `base RRI > 100` mandatory-decomposition trigger
in `detect_triggers()`, this divergence could let a penalty-driven overflow
that `final` correctly reports go undetected by that trigger — a fail-open
gap on a safety-relevant boundary check.

**Disposition: accepted and fixed**, not accepted-follow-up. Changed
`base_val = max(ici_band_rri, risk_base)` to
`base_val = max(ici_band_rri, risk_band_rri)` (`scripts/rri.py`), so `base`
and `final` are now always equal by construction. Added a regression test,
`TechnicalProfileV2::test_base_stays_penalty_inclusive_like_final`
(`scripts/rri_test.py`), using inputs where the pre-fix code would have
produced `base=25` while `final=32` (ICI floor vs. penalty-driven risk
band) — confirms the fix and would fail against the pre-fix code. Full
suite re-run: 71/71 passed after the fix.

### Gemma Reviewer evidence

- Model: `gemma4:26b-a4b-it-qat` (`DUBBRIDGE_REVIEW_MODEL` resolution, RRI
  26-55 chain primary per `AGENT_WORKFLOW_GUIDE.md` § Band-routed peer
  review — used here as the ADR/docs-change propagation review, not a
  banded development-task review, since this change has no computed RRI
  under the interrupted self-scoring attempt the owner overrode)
- Command: `make qa-docs-review`
- Passes run / usable: 3/1 (passes 1-2 failed to parse; pass 3 produced a
  usable structured result before also failing to fully validate — the
  wrapper still surfaced its finding)
- Aggregate status: `FINDINGS`
- Consensus findings: 0 | Pass-specific: 1 | Disagreement: 0
- Artifact: `/tmp/dubbridge-gemma-review.json` (ephemeral local path, not
  persisted to the repo; this section is the durable record)
- Isolated adjudicator: not triggered — a usable single-pass result was
  produced and required no D14 escalation
- disposition_divergence: none
- Primary-agent disposition: accepted and fixed (see above); the one
  finding was correct and material, not a false positive

## Follow-up: legacy v1 formula fully removed from the script (2026-09-07)

The original ADR-045 implementation kept the pre-v2 weighted-sum formula
computed and reported as `legacy_weighted_base` (an audit/continuity figure,
explicitly non-authoritative — see the ADR's own text and § What changed
above). By explicit owner instruction the same day ("pues hazlo..." in
response to being asked whether v1 should be fully removed, not just
demoted), that retained computation was deleted outright rather than kept
as a reporting-only figure.

**Changed:**

- `scripts/rri.py::evaluate()` — removed the `weighted = sum(WEIGHTS[v] *
  scores[v] for v in VARS)` / `legacy_weighted_base = round(100 * weighted /
  5)` computation and its dict key. `WEIGHTS`/`VARS` themselves are
  unchanged and still used (individual per-variable scoring/rendering, and
  `risk_base`'s own D/P/K weighted contribution inside the v2 `risk_band`
  computation — that usage is v2 authority, not legacy v1, and was not
  touched).
- `scripts/rri.py::render_markdown()` — removed the `**Base value (legacy
  v1 weighted sum, audit-only, not authoritative):**` line.
- `scripts/rri.py::render_json()` — removed the `"legacy_weighted_base"`
  key from the JSON output.
- `scripts/rri_test.py` — removed
  `BaseFormula::test_legacy_weighted_base_retained_for_audit_only`, the one
  test asserting the now-deleted field's presence/value.
- `docs/policies/RRI_POLICY.md` — updated both places describing
  `legacy_weighted_base` as a currently-computed script output to state it
  was removed 2026-09-07; the `<details>` block documenting the historical
  formula's mathematics is kept as historical/documentation context only
  (it was never code, and removing the historical record of what the
  formula was would make ADR-045's own "supersedes this prior formula"
  claim unverifiable).

**Not changed:** `docs/adr/ADR-045-rri-v2-authority-replacement.md` and this
audit's own § What changed / § Design notes sections above are left as
literal historical record of what the *original* ADR-045 change did (they
correctly describe `legacy_weighted_base` as having existed at that point in
time) — per the repository's ADR-change-propagation norm of marking
superseded rather than rewriting history, this section is the forward
pointer rather than an edit to those.

**Verification:**

```
python3 scripts/rri_test.py                 # 70/70 passed (71 - 1 removed test)
python3 scripts/rri_v2_candidate_test.py     # 13/13 passed, unchanged
python3 scripts/rri.py --C 3 --D 2 --K 3 --P 1 --T 2 --A 1 --X 1 --F 2
python3 scripts/rri.py --C 3 --D 2 --K 3 --P 1 --T 2 --A 1 --X 1 --F 2 --json
grep -rn "legacy_weighted_base" . --include="*.py"   # 0 matches
git diff --check                             # clean
make qa-rri                                  # 70/70 rri_test + 6/6 roadmap-drift
```

Markdown/JSON output confirmed to no longer contain `legacy_weighted_base`
in any form; `final`/`ici_band_rri`/`risk_band_rri` outputs are unchanged
from before this follow-up (same v2 authority, only the retained legacy
figure was deleted). No independent Gemma Reviewer/D14 pass was run
specifically for this follow-up: it is a pure subtraction of dead/superseded
computation and its own single covering test, with no new logic path, no
behavior change to `final`/band/gate/trigger outputs, and no touched file
outside `scripts/rri.py`, `scripts/rri_test.py`, and
`docs/policies/RRI_POLICY.md` — the same class of change the workflow guide
treats as not requiring a fresh band-routed review (no new reasoning path to
review; the only executable delta is deletion, and the full existing test
suite plus a direct CLI smoke test confirm no observable behavior changed
outside the intended field removal). This was a direct, explicitly
owner-instructed edit; no HITL card, RRI self-score, or approval gate was
generated for it, consistent with the same owner override recorded in
§ Scope and authorization boundary above ("no le des rodeos al tema. ya fue
validado").

## What remains open (explicitly not resolved by this change)

- **No empirical calibration exists.** `prediction_status=not_calibrated`
  and P50/P90 remain `null` in every code path, including the now-
  authoritative one. This was an explicit, informed owner decision (see
  ADR-045 §Owner approval), not an oversight.
- **Historical RRI reports are not directly comparable** to reports
  computed after this change for the same inputs, because `max()` behaves
  differently from a weighted average (demonstrated above). No historical
  task ledger was retroactively rescored.
- **The axis-derivation bridge (`C→L, K→I, D→Q, T→V`) is this
  implementation's own design choice**, not validated by any assessor-
  agreement study. `docs/proposals/rri-v2-model.md`'s original four
  independently-assessed axes remain a different, fuller construct that
  `scripts/rri_v2_candidate.py` still implements for anyone who wants a
  fully evidenced assessment record.
- No commit, push, or modification of the user's pre-existing uncommitted
  changes (`crates/p2p/`, `docs/architecture.md`, `docs/plan/roadmap.md`,
  `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`, the untracked
  `rri-v2-*` files from prior sessions) was performed by this task.
