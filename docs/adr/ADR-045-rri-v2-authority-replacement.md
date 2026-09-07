---
type: ADR
title: "ADR-045: RRI v2 (rri-v2-design-0.2) authority replacement"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-045: RRI v2 (rri-v2-design-0.2) Authority Replacement

- **Status:** Accepted
- **Date:** 2026-09-07
- **Deciders:** DubBridge owner
- **Scope:** agent workflow scoring mechanism only (`scripts/rri.py`'s final
  band/score computation). No change to HITL approval requirements, band
  boundaries, reviewer chains, autonomy gates, or any downstream policy that
  consumes a band label.
- **Owner approval:** the owner explicitly directed a full replacement of the
  v1 weighted-sum authority by the v2 candidate design ("realiza el reemplazo
  directo... ya fue validado"), after being shown that the v2 design
  documents (`docs/proposals/rri-v2-model.md`,
  `docs/proposals/rri-v2-formula.md` §7) and prior task ledgers
  (`docs/tasks/rri-v2-candidate-calculator.md`,
  `docs/tasks/rri-v2-model-and-formula.md`) had described a staged shadow-
  scoring adoption sequence, not an immediate authority swap. The owner
  overrode that staged sequence explicitly. See `docs/audit/rri-v2-authority-
  replacement-2026-09-07.md` for the full record of that exchange.

## Context

`docs/policies/RRI_POLICY.md` (Status: Active) previously computed RRI as a
weighted sum of eight 0-5 variables (C, F, D, T, A, K, P, X), normalized to
0-100, plus additive penalties. `docs/proposals/rri-v2-model.md` and
`docs/proposals/rri-v2-formula.md` (version `rri-v2-design-0.2`) proposed a
different construct: a four-axis ordinal technical-difficulty profile
`(L,I,Q,V)`, each 0-4, reduced to a bottleneck `B = max(L,I,Q,V)` and a
provisional `ICI = 25*B` screening index, explicitly excluding size and risk
from that index. Both were implemented as an isolated, non-authoritative
candidate (`scripts/rri_v2_candidate.py`, `docs/tasks/rri-v2-candidate-
calculator.md`) that computed a parallel result and never affected
`scripts/rri.py`, `RRI_POLICY.md`, or any routing/HITL gate.

The formula proposal's own adoption sequence (§7) called for prospective
shadow scoring alongside the active v1 calculator before any routing change,
with promotion contingent on an empirical evaluation that has not been run
(`prediction_status=not_calibrated`, no fitted coefficients, no P50/P90).
The owner reviewed this and explicitly chose to skip that staged sequence and
replace the v1 authority immediately, accepting the empirical gap.

## Decision

`scripts/rri.py` is amended so the four-part ordinal technical profile
`(L,I,Q,V)` and its bottleneck `B`/`ICI = 25*B` determine the technical-
difficulty component of the final RRI band, replacing the v1 weighted-sum
formula as the authoritative computation. Concretely:

1. **Axis derivation.** `(L,I,Q,V)` are derived deterministically from the
   same inputs the CLI already collects — no new agent-supplied fields:
   - `L` (logical/algorithmic obligations) ← `C` (cyclomatic complexity)
   - `I` (contract/integration obligations) ← `K` (coupling/side effects)
   - `Q` (state/temporal obligations) ← `D` (domain complexity — DubBridge's
     D floor already captures async/state-heavy domains at D>=4)
   - `V` (verification/oracle obligations) ← `T` (test-coverage risk)

   Each existing 0-5 score maps to a 0-4 level via `min(4, score)`, so a
   legacy 5 ("critical") collapses onto the top v2 level rather than being
   diluted.

2. **Risk/domain floor preserved.** `rri-v2-formula.md` §3 is explicit that
   ICI excludes size and risk ("a simple permission change can have ICI 25
   and high operational risk"). The existing ADR-anchored D/P/K floor table
   and the existing penalty table (`auth_security`, `arch_decision`,
   `no_tests_high_impact`, etc.) are retained unchanged and combined into a
   `risk_band_rri` figure using the same per-variable unit contribution the
   v1 formula used for D/P/K alone, plus the full penalty total.

3. **Final band = max(ici_band, risk_band).** `ici_band_rri` maps each ICI
   step (0/25/50/75/100) onto the RRI point that anchors the equivalent band
   ceiling (25/40/55/70/100). The final score is
   `max(ici_band_rri, risk_band_rri)`, so neither a low technical-difficulty
   reading can suppress a high domain/security risk floor, nor vice versa.
   This is a deliberate fail-closed combination, not an average.

4. **No calibrated effort prediction anywhere in the routing path.** P50/P90
   and any fitted coefficient remain out of scope. `prediction_status`
   stays `not_calibrated`. This ADR replaces only the *technical-difficulty
   scoring mechanism* that feeds the band; it does not introduce or
   authorize any effort/time prediction.

5. **Band table, gates, reviewer chains, and HITL thresholds are unchanged.**
   `docs/policies/RRI_POLICY.md`'s band table (Low/Moderate/Med-high/
   Complex/High/Very high/Excessive), its approval gates, its Codex/Claude
   capability mapping, and every reviewer-chain binding in
   `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` continue to apply exactly as
   written, keyed off the same 0-100 final score — only how that final score
   is computed changed.

6. **`scripts/rri.py`'s CLI contract is unchanged.** Every existing
   `--cc/--C/--auto-cc/--D/--K/--P/--T/--A/--X/--touches/--F/--penalty/
   --platform/--json` flag behaves identically; historical task-ledger
   commands remain replayable. Legacy weighted-sum output is retained in
   the markdown/JSON report as `legacy_weighted_base`, labeled explicitly as
   an audit-only figure that no longer determines the band.

7. **`scripts/rri_v2_candidate.py` remains the schema-validated envelope
   tool** for producing a fuller, evidence-referenced v2 assessment record
   (explicit per-axis status/method/evidence, ranges, uncertainty) when that
   level of detail is independently warranted. Most tasks continue to score
   through `scripts/rri.py`'s CLI, which is now the day-to-day authoritative
   path.

## Consequences

**Positive:**

- Technical/reasoning difficulty and domain/security risk are now scored as
  two explicit, separately-visible inputs (`ici_band_rri`, `risk_band_rri`)
  instead of being blended into one opaque weighted sum — every RRI report
  now shows which of the two drove the final band.
- A single dominant obligation (e.g., one interacting algorithmic invariant)
  can no longer be diluted by five easy variables the way an average-based
  formula allowed; `max()` surfaces true bottlenecks.
- The existing ADR-anchored anchor rubric (auth, rights, migrations, etc.)
  keeps its exact floor values and continues to force a high band
  regardless of how the ICI axes score — no security regression from
  adopting the new construct.

**Negative / accepted risk:**

- This adopts `rri-v2-design-0.2` before its own proposed empirical
  validation (`docs/proposals/rri-v2-formula.md` §7 steps 1-4: pilot,
  freeze, prospective cohort, comparison against baselines). No prospective
  labeled data, assessor-agreement study, or fitted comparison against the
  v1 formula exists. The owner explicitly accepted this gap rather than
  running the staged shadow-scoring sequence first.
- Band boundaries for a given set of inputs can shift versus v1 (verified:
  see `scripts/rri_test.py::BaseFormula` and `::TechnicalProfileV2`) — some
  tasks that scored Moderate under v1 may now score Med-high or higher (or
  lower) under the same inputs, because `max()` behaves differently from a
  weighted average. This is intentional per the design's stated purpose
  (surfacing bottlenecks an average would hide), not a defect, but it means
  historical RRI reports computed under v1 are not directly comparable to
  reports computed after this ADR.
- The axis derivation (`C→L, K→I, D→Q, T→V`) is a deterministic bridge
  invented for this replacement, not part of the original `rri-v2-formula.md`
  design (which assumed axes would be independently assessed per the
  `rri_v2_candidate.py` schema, each with its own evidence/method/status).
  It reuses existing agent judgments rather than requiring four separate new
  judgments, at the cost of coupling axes that the original design treated
  as independent (e.g., `Q` inheriting `D`'s domain-complexity judgment
  rather than a dedicated state/temporal-obligation judgment).

**Neutral:**

- No change to which model/tier/reviewer a given final band routes to; only
  how the final band number is computed changed.
- `docs/proposals/rri-v2-model.md` and `docs/proposals/rri-v2-formula.md`
  remain `Proposed` as *measurement-model* documents (they still describe
  the fuller independently-assessed-axis design that `rri_v2_candidate.py`
  implements); this ADR documents the narrower bridge actually wired into
  `scripts/rri.py`'s day-to-day authority.

## Alternatives considered

- **Shadow scoring first, then promote after calibration** (the sequence
  `rri-v2-formula.md` §7 itself proposes). Rejected by explicit owner
  instruction — the owner considered the design already validated by the
  prior review/verification passes (`docs/audit/rri-v2-design-validation.md`,
  `docs/audit/rri-v2-candidate-calculator-verification.md`) and did not want
  to wait for a further empirical cohort before adopting the new
  construct's scoring behavior.
- **Independent per-axis assessment (the full `rri_v2_candidate.py` schema)
  as the day-to-day authority**, replacing `scripts/rri.py`'s CLI entirely.
  Rejected for this change: it would require re-authoring every canonical
  doc's RRI invocation examples and every historical task-ledger command
  into the four-axis JSON envelope, which is unbounded scope beyond what
  was authorized. The bridge in this ADR keeps the existing CLI contract
  stable while still making the v2 construct authoritative.
- **Average or weighted mixture of ICI and the legacy formula.** Rejected
  per `rri-v2-formula.md` §1's own rationale against averaging: it would
  let easy variables dilute a genuine bottleneck, defeating the purpose of
  adopting the max-based construct at all.

## Related

- `docs/policies/RRI_POLICY.md` — band table, gates, reviewer chains (status
  unchanged by this ADR; the formula section is superseded by this ADR)
- `docs/proposals/rri-v2-model.md`, `docs/proposals/rri-v2-formula.md` —
  measurement-model and full-formula design this ADR partially adopts
- `docs/audit/rri-v2-design-validation.md`,
  `docs/audit/rri-v2-candidate-calculator-verification.md` — prior
  candidate-only design/implementation review
- `docs/audit/rri-v2-authority-replacement-2026-09-07.md` — this ADR's
  implementation record: exact commands, test results, and doc propagation
- `scripts/rri.py`, `scripts/rri_test.py` — authoritative calculator and
  its updated tests
- `scripts/rri_v2_candidate.py` — retained schema-validated envelope tool
