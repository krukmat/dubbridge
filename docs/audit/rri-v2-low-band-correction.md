---
type: Audit
title: "RRI v2 Low-band correction verification"
status: closed
---
# RRI v2 Low-band correction

Owner authorization (2026-09-08): "cambialo directamente", approving the
previously proposed ICI 25 -> RRI 25 correction. Primary author: Codex.

## Pre-change parent score

Conservative governance envelope: D4 (agent routing), P4 (autonomy gate),
K2 (contained calculator consumers), T0 (70 existing passing tests), A0,
X2; production change is a table entry, CC1. Policy/security penalties
are explicit. This parent remains authoritative after the change and is not
rescored down by its own output. Work is decomposed in the linked plan;
owner directly authorizes this bounded amendment and its implementation.

```sh
python3 scripts/rri.py --cc 1 --D 4 --K 2 --P 4 --T 0 --A 0 --X 2 --touches scripts/rri.py --touches scripts/rri_test.py --touches docs/policies/RRI_POLICY.md --touches docs/adr/ADR-045-rri-v2-authority-replacement.md --touches docs/plan/rri-v2-low-band-correction.md --touches docs/tasks/rri-v2-low-band-correction.md --touches docs/audit/rri-v2-low-band-correction.md --penalty arch_decision --penalty auth_security
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 3 | --touches -> 7 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Technical profile (v2, ADR-045):** L=0 I=2 Q=4 V=0 -> bottleneck B=4 -> ICI=100
**Risk/domain band input:** 47 (D/P/K weighted + penalties 22)
**Penalties applied:** arch_decision (+12, manual flag); auth_security (+10, manual flag)
**Final RRI:** 100 = max(ici_band 100, risk_band 47) -> band Very high (86-100) -> Effort XL . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Do not implement directly. Produce an ADR + risk analysis + decompose into subtasks.
**Decomposition:** triggered by RRI >= 56 — split before implementing
**Advisory:** scripts/rri.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/rri_test.py: no anchor-rubric match — agent judgment governs D/P/K

## Verification and limitations

Implementation tests, independent solution review and final documentation QA passed. No empirical calibration is claimed. The independent-axis redesign
is deferred. Risk/P-only changes outside path anchors are not guaranteed a
high score by the existing formula; this amendment does not introduce new
risk floors or imply that all security changes remain in the same band.
Existing explicit HITL requirements for governance-critical invariants apply.

## Propagation

Update active policy and ADR-045 decision text; keep 2026-09-07 audit results
historical. ADR status/title unchanged. No runtime boundary, product slice,
or dependency changes: architecture and roadmap updates are not applicable.
Pre-existing user edits in P2P ledgers/roadmap/tests are preserved.

## Decomposition scoring

All phases retain the RRI100 parent review/approval envelope. These scores
are descriptive, not a route around that envelope.

### T1 decision and risk specification

```sh
python3 scripts/rri.py --cc 1 --A 0 --X 2 --D 2 --K 1 --P 4 --T 0 --penalty arch_decision --penalty auth_security --touches docs/plan/rri-v2-low-band-correction.md --touches docs/tasks/rri-v2-low-band-correction.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 1 | --touches -> 2 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Technical profile (v2, ADR-045):** L=0 I=1 Q=2 V=0 -> bottleneck B=2 -> ICI=50
**Risk/domain band input:** 38 (D/P/K weighted + penalties 22)
**Penalties applied:** arch_decision (+12, manual flag); auth_security (+10, manual flag)
**Final RRI:** 55 = max(ici_band 55, risk_band 38) -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered

### T2 inseparable code/policy invariant

```sh
python3 scripts/rri.py --cc 1 --A 0 --X 2 --D 4 --K 2 --P 4 --T 0 --penalty arch_decision --penalty auth_security --touches scripts/rri.py --touches scripts/rri_test.py --touches docs/policies/RRI_POLICY.md --touches docs/adr/ADR-045-rri-v2-authority-replacement.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 2 | --touches -> 4 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no rubric match) | High |
| P impact | 4 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Technical profile (v2, ADR-045):** L=0 I=2 Q=4 V=0 -> bottleneck B=4 -> ICI=100
**Risk/domain band input:** 47 (D/P/K weighted + penalties 22)
**Penalties applied:** arch_decision (+12, manual flag); auth_security (+10, manual flag)
**Final RRI:** 100 = max(ici_band 100, risk_band 47) -> band Very high (86-100) -> Effort XL . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Do not implement directly. Produce an ADR + risk analysis + decompose into subtasks.
**Decomposition:** triggered by RRI >= 56 — split before implementing
**Advisory:** scripts/rri.py: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** scripts/rri_test.py: no anchor-rubric match — agent judgment governs D/P/K

### T3 verification and closure

```sh
python3 scripts/rri.py --cc 1 --A 0 --X 2 --D 1 --K 1 --P 0 --T 0 --touches docs/audit/rri-v2-low-band-correction.md --touches docs/tasks/rri-v2-low-band-correction.md --touches docs/plan/rri-v2-low-band-correction.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 2 | --touches -> 3 files | High |
| D domain | 1 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 0 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Technical profile (v2, ADR-045):** L=0 I=1 Q=1 V=0 -> bottleneck B=1 -> ICI=25
**Risk/domain band input:** 5 (D/P/K weighted + penalties 0)
**Penalties applied:** none
**Final RRI:** 40 = max(ici_band 40, risk_band 5) -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered

## Phase-1 review disposition

Task-analysis review: claude docs/audit/rri-v2-low-band-correction-phase1.json - PASS

Independent reviewer accepted the bounded patch. Its illustrative EC-1 vector
D2/K2 actually yields ICI50, not ICI25; the executable regression instead
uses D1/K1/P5/T1 plus auth_security and arch_decision, yielding risk37 and
ICI25. Its phrase "not a regression" about sensitive inputs is too broad:
B1/P-sensitive tasks can change from 40 to 25; active policy and amendment
explicitly disclose that consequence. Review prose is retained unmodified.

Regression-first run: `python3 scripts/rri_test.py TechnicalProfileV2` failed
with the expected 40 !=25 results (18 failures including profile subtests).
Only then was the production mapping changed. Initial `make qa-docs` passed.
The repo review wrapper is incompatible with system Python3.9 and assumes
`claude review --stdin`; review used installed Claude's supported `-p`
interface instead, isolated temporary cwd, no tools/MCP/session persistence.

## Implementation verification

- `make qa-rri`: PASS, 74 calculator tests and 6 roadmap-drift tests.
- `git diff --check`: PASS.
- `python3 scripts/rri.py --cc 1 --F 0 --D 1 --K 1 --P 1 --T 1 --A 0 --X 1`: final25, risk7, Low.
- New exhaustive regression covers all 625 ordinal profiles and 2,000
  one-axis increases; B>=2 final scores are unchanged.
- HP-1 is exercised through subprocess CLI JSON and markdown. EC-1 uses
  risk37 at B1 (not the phase-1 review's mistaken D2/K2 example).
- Auth and rights-ledger floors still yield100 and auth_security penalties.

## Reflection log

Required passes: four (conservative parent RRI100, 56+ envelope).

1. Draft: table-only correction. Critique: local semantics must reach Low
   without lowering B2+. Revise: added exhaustive-profile regression;
   preserved all upper mappings.
2. Draft: risk-preservation claims. Critique: phase-1 illustrative vector
   used D2/K2 and sensitive cases can fall from40 to25. Revise: use the
   executable D1/K1/P5/T1 risk37 vector and explicitly qualify policy/ADR
   claims; no new floors added.
3. Draft: behavioral tests. Critique: testing the lookup alone cannot prove
   CLI wiring or risk dominance. Revise: add real JSON/markdown CLI coverage,
   risk37 regression and auth/rights paths; tests pass.
4. Draft: documentation propagation. Critique: current policy and ADR must
   match while historical adoption reports stay accurate. Revise: amend
   current formula/ADR, link dedicated plan/task/audit, preserve old audit;
   remove accidental empty duplicate heading. No further code changes needed.

## Independent code-solution review

Code-solution review: claude docs/audit/rri-v2-low-band-correction-phase2.json - PASS

Review receipt binds the four-file implementation/policy diff by SHA-256.
Reviewer noted the exhaustive exact-score tests deliberately bind current
risk arithmetic; future risk recalibration must revise expectations. Accepted,
no current defect or code revision. Clarification: auth paths force100 through
D/K technical axes; the auth_security penalty alone does not force100.

## Final closure — 2026-09-08

- `make qa-docs`: PASS after correcting ledger parser requirements
  (exact Happy paths/Edge cases considered headings, numeric RRI field,
  required-pass count, and date without trailing punctuation). Initial
  final-QA attempt failed only these new ledger-format checks; the focused
  `python3 scripts/check-behavioral-coverage.py` then passed and the full
  `make qa-docs` rerun passed.
- `make qa-rri`: PASS (74 calculator +6 roadmap tests), as recorded above.
- `git diff --check`: PASS. Reviewed implementation diff hash revalidated.
- Plan/tasks/audit closed; current policy/ADR synchronized. No product
  roadmap changes needed; pre-existing user work preserved.
- No commit or push. No empirical calibration or independent-axis redesign.
