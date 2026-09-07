---
type: TaskList
title: "RRI v2 candidate collector and calculator tasks"
status: complete
plan: docs/plan/rri-v2-candidate-calculator.md
---
# RRI v2 candidate collector and calculator tasks

> **Superseded scope note (2026-09-07, ADR-045):** the "must remain
> byte-for-byte untouched" boundary below described this ledger's own frozen
> scope at the time it closed. By explicit owner override the same day,
> `docs/adr/ADR-045-rri-v2-authority-replacement.md` subsequently replaced
> `scripts/rri.py`'s formula authority with a bridged version of the v2
> construct. This ledger's history, RRI, and review evidence remain accurate
> as a record of what was built here; they no longer describe the current
> state of `scripts/rri.py`. See ADR-045 and
> `docs/audit/rri-v2-authority-replacement-2026-09-07.md`.

**Scope:** Candidate-only implementation of `rri-v2-design-0.2`. The current
RRI/HITL calculator, rules, authorization, routing, and policies are explicitly
out of scope and must remain byte-for-byte untouched.

**Behavioral coverage contract:** behavior-v2.

**Owner authorization and route:** On 2026-09-07, the owner explicitly waived
another task presentation/approval request for this validated adjustment and
directed Codex to implement it directly, without a local-model implementer.
This authoring-route exception is limited to this ledger's frozen scope. It does
not change the parent RRI 33, phase-2 independent review, reflection, evidence,
or the active RRI/HITL authorization rules.

## Parent RRI report

```bash
python3 scripts/rri.py --C 2 --T 0 --K 2 --P 1 --D 2 --A 1 --X 3 --touches scripts/rri_v2_candidate.py --touches scripts/rri_v2_candidate_test.py --touches docs/schemas/rri-v2-candidate-assessment.schema.json --touches docs/fixtures/rri-v2-candidate-valid.json --touches docs/fixtures/rri-v2-candidate-invalid.json --touches docs/plan/rri-v2-candidate-calculator.md --touches docs/tasks/rri-v2-candidate-calculator.md --touches docs/audit/rri-v2-candidate-calculator-verification.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 2 | Estimated bounded validation/dispatch logic | High |
| F files | 3 | `--touches` -> 8 files | High |
| D domain | 2 | Candidate measurement contract; no active-rule path | High |
| T coverage | 0 | New executable unit/CLI evidence is required | High |
| A ambiguity | 1 | Versioned proposals and documented examples constrain scope | High |
| K coupling | 2 | Schema, fixtures, calculator and audit must agree | High |
| P impact | 1 | Candidate output only; active authorization unaffected | High |
| X context | 3 | Model/formula, audit, schema and Python test conventions | High |

**Base value:** 100 x (weighted / 5) = 33
**Penalties applied:** none
**Final RRI:** 33 -> band Moderate (26-40) -> Effort M. Codex Balanced.
Claude Balanced. thinking Off.
**Gates:** explicit HITL approval; per-task Ollama restart/precheck; Gemma
phase-1 and phase-2 review; two Reflection passes; behavioral certification and
owner final verification. **Decomposition:** not triggered.

Task-analysis review: gemma
`docs/audit/gemma-evidence/rri-v2-candidate-calculator-phase1.json` - PASS.
Code-solution review: gemma `docs/audit/gemma-evidence/T2.json` - PASS.

## Live phase checklist

- [x] Restart Ollama + local-stack precheck — Codex (`completed`: server PID
  88971 -> 87423; Gemma reduced-profile warm/review pass recorded in phase-1 artifact).
- [x] Analyze and scope — Codex (`completed`: proposals, audit, active-policy boundary, RRI scored).
- [x] Phase 1 review — gemma4:26b-a4b-it-qat (`completed`: PASS,
  `docs/audit/gemma-evidence/rri-v2-candidate-calculator-phase1.json`).
- [x] Approval — owner (`completed`: explicit same-session waiver recorded above).
- [x] T1 schema and fixtures — Codex (`completed`).
- [x] T2 candidate collector/calculator — Codex (`completed`).
- [x] T3 reflect, verify, and synchronize — Codex (`completed`).
- [x] Phase 2 review — gemma4:26b-a4b-it-qat (`completed`: PASS).
- [x] Close — Codex (`completed`).

## T1 — Define the versioned assessment envelope and fixtures

- **Status:** [x] Done — 2026-09-07
- **Type:** development
- **Effort:** S
- **Complexity:** Low
- **RRI:** 21 / Low (leaf); parent envelope remains 33 / Moderate.
- **Depends on:** approved parent envelope
- **Allowed paths:** `docs/schemas/rri-v2-candidate-assessment.schema.json`,
  `docs/fixtures/rri-v2-candidate-*.json`,
  `scripts/rri_v2_candidate_test.py`, this ledger, linked plan/audit.
- **Objective:** Define a closed, versioned machine envelope and documented
  applicable, unknown, inapplicable, and invalid fixtures.
- **Acceptance criteria:** closed schema rejects unrecognized fields and
  unknown version; axis evidence has status/method/evidence references; a
  fully inapplicable record requires an explicit reason and does not supply a
  technical profile; fixture values match the formula's contract examples.
- **HP-1:** `(1,1,0,1)` with observed/estimated evidence validates as an
  applicable maintenance assessment.
- **EC-1:** a missing `Q`, a fractional level, or a reversed range is rejected
  before a result can be emitted.
- **EC-2:** an out-of-rubric planning assessment with an explicit reason is
  valid and is ready to yield `technical: null`, not a fake zero profile.
- **Evidence to emit:** schema/fixture test selectors and audit fixture matrix.
- **Status artifacts affected:** this ledger, linked plan, verification audit.
- **Handoff:** Implement only the candidate envelope; do not alter active RRI
  JSON or its schema/CLI behavior.

### Happy paths considered

- **HP-1:** A complete maintenance assessment validates against the closed,
  versioned envelope.

### Edge cases considered

- **EC-1:** Missing Q, fractional level, or reversed range fails closed.
- **EC-2:** An evidence-backed planning assessment is inapplicable, never a
  fabricated zero profile.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | Closed candidate schema accepts the valid contract fixture | unit | `scripts/rri_v2_candidate_test.py::test_schema_is_a_closed_draft_2020_12_candidate_contract` | passed |
| EC-1 | Edge case | Missing axis and malformed levels/ranges fail closed | unit | `scripts/rri_v2_candidate_test.py::test_missing_axis_reversed_range_unknown_version_and_status_mismatch_fail_closed` | passed |
| EC-2 | Edge case | Planning input is inapplicable rather than numerically zero | unit | `scripts/rri_v2_candidate_test.py::test_planning_record_returns_null_technical_result` | passed |

### Owner final verification

- Owner: Codex (orchestrator of record)
- Date: 2026-09-07
- Statement: I verified every happy path and edge case defined for this task has executable unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/rri_v2_candidate_test.py`; `make qa-docs`; `git diff --check`
- Review artifact: docs/audit/gemma-evidence/T1.json

## T2 — Implement the separate candidate collector and calculator

- **Status:** [x] Done — 2026-09-07
- **Type:** development
- **Effort:** M
- **Complexity:** Moderate
- **RRI:** 30 / Moderate; parent envelope remains 33 / Moderate.
- **Depends on:** T1
- **Allowed paths:** `scripts/rri_v2_candidate.py`,
  `scripts/rri_v2_candidate_test.py`, this ledger, linked plan/audit.
- **Objective:** Load the T1 envelope, validate it, and produce a deterministic
  candidate-only ordinal result without effort prediction or authorization.
- **Acceptance criteria:** validate applicability before axes; point and range
  profiles calculate `B`, `ICI=25B`, and ordered `H`; unknown axes enclose
  bounds without midpoint; errors are structured/nonzero; every result fixes
  `candidate_only`, existing-RRI authorization, `not_calibrated`, and null
  `p50`/`p90`.
- **HP-2:** a coordinated `(2,2,1,2)` assessment returns ICI 50 and breadth
  `(0,0,3,4)` while retaining the raw named profile.
- **HP-3:** `(1,1,[0,4],1)` returns ICI bounds `[25,100]`, no point estimate,
  and unresolved uncertainty.
- **EC-3:** NaN/boolean/out-of-range values, an unknown rubric version, or a
  status/value contradiction returns a validation error and no partial report.
- **EC-4:** the planning fixture dispatches before missing-axis validation and
  returns `technical: null` with its reason, not an error or numeric score.
- **Evidence to emit:** unit and CLI test selectors, example outputs, and
  audit record of authorization/prediction invariants.
- **Status artifacts affected:** this ledger, linked plan, verification audit.
- **Handoff:** The implementation is separate from `scripts/rri.py`; no
  coefficients, fitted-model loading, routing decision, or active-policy call
  is permitted.

### Happy paths considered

- **HP-2:** A coordinated profile returns its named profile, ICI 50, and
  breadth `(0,0,3,4)`.
- **HP-3:** An unresolved temporal axis returns only enclosure bounds.

### Edge cases considered

- **EC-3:** Invalid version/type/status values fail with no success JSON.
- **EC-4:** Inapplicable planning dispatches before axis validation.

### Reflection log

Required passes: 2 (`30` -> Moderate)

#### Pass 1

- **Draft verdict:** Candidate output preserves points, ranges, null effort predictions, and active-RRI isolation.
- **Critique findings:** Checked that a degenerate ICI range caused by known L4 cannot label the incomplete profile fully assessed.
- **Revisions applied:** Kept point `ici` and `bottleneck` null whenever any axis remains unresolved, while retaining the exact `[100,100]` enclosure.

#### Pass 2

- **Draft verdict:** CLI and in-process validation fail before producing a partial result.
- **Critique findings:** Checked JSON `NaN`, booleans, unknown size, and inapplicable malformed axes for silent-low paths.
- **Revisions applied:** Rejected non-finite JSON at load time; preserved strict boolean/finite checks and inapplicability-before-axis dispatch.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-2 | Happy path | Coordinated point profile returns ICI and breadth specified by the formula | unit | `scripts/rri_v2_candidate_test.py::test_documented_point_scenarios` | passed |
| HP-3 | Happy path | Unknown temporal profile returns bounds without midpoint | unit | `scripts/rri_v2_candidate_test.py::test_unknown_axis_encloses_without_a_midpoint` | passed |
| EC-3 | Edge case | Invalid values, versions, ranges, and status combinations fail closed | unit | `scripts/rri_v2_candidate_test.py::test_documented_invalid_levels_are_rejected` | passed |
| EC-4 | Edge case | Inapplicable record dispatches before axis shape validation | unit | `scripts/rri_v2_candidate_test.py::test_inapplicability_dispatches_before_axis_shape_validation` | passed |

### Peer Reviewer evidence

- Reviewer: gemma
- Command: `curl http://127.0.0.1:11434/api/chat` with `gemma4:26b-a4b-it-qat`, `think=false`, `num_ctx=32768`, `num_predict=1200`
- Artifact: docs/audit/gemma-evidence/T2.json
- Verdict: PASS
- Findings: none
- Muse Glimmer fallback: not triggered — reason: Gemma returned structured PASS.
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a — reason: n/a
- disposition_divergence: none
- Primary-agent disposition: accepted; no repair required.

### Owner final verification

- Owner: Codex (orchestrator of record)
- Date: 2026-09-07
- Statement: I verified every happy path and edge case defined for this task has executable unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m py_compile scripts/rri_v2_candidate.py scripts/rri_v2_candidate_test.py`; `python3 scripts/rri_v2_candidate_test.py`; `make qa-review-budget REVIEW_PATHS='scripts/rri_v2_candidate.py scripts/rri_v2_candidate_test.py docs/schemas/rri-v2-candidate-assessment.schema.json'`; `make qa-docs`; `git diff --check`
- Review artifact: docs/audit/gemma-evidence/T2.json

## T3 — Verify behavior, limits, and closure artifacts

- **Status:** [x] Done — 2026-09-07
- **Type:** development
- **Effort:** S
- **Complexity:** Low
- **RRI:** 21 / Low (leaf); parent envelope remains 33 / Moderate.
- **Depends on:** T2
- **Allowed paths:** `scripts/rri_v2_candidate_test.py`,
  `docs/audit/rri-v2-candidate-calculator-verification.md`, this ledger,
  linked plan.
- **Objective:** Run documented mathematical scenarios and negative cases,
  verify isolation from active RRI, and record results plus unresolved limits.
- **Acceptance criteria:** all listed HP/EC cases map to executable evidence;
  formula scenarios, uncertainty, invalid input and inapplicability pass;
  `scripts/rri.py` and active policies are unchanged; no predictive validity,
  calibration, or policy-adoption claim is made.
- **HP-4:** all published point scenarios reproduce their ICI/breadth outputs.
- **EC-5:** invalid input produces no success JSON; inapplicable input produces
  only the documented null technical result.
- **Evidence to emit:** verification audit with exact commands, outcomes,
  limits, review receipts, reflection log and behavioral-coverage mapping.
- **Status artifacts affected:** this ledger, linked plan, audit; roadmap/ADR
  status is n/a because no product dependency or architecture decision changes.
- **Handoff:** Record failures and skipped gates; do not call synthetic cases
  evidence of calibration or prediction quality.

### Happy paths considered

- **HP-4:** All published point scenarios reproduce their specified formula
  outputs under the candidate calculator.

### Edge cases considered

- **EC-5:** Invalid input has no success JSON; inapplicable input has only the
  documented null technical result.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-4 | Happy path | Published formula scenarios reproduce ordinal outputs | unit | `scripts/rri_v2_candidate_test.py::test_documented_point_scenarios` | passed |
| EC-5 | Edge case | CLI rejects invalid input and emits null technical output only for inapplicability | unit | `scripts/rri_v2_candidate_test.py::test_invalid_cli_emits_no_success_json`; `scripts/rri_v2_candidate_test.py::test_planning_record_returns_null_technical_result` | passed |

### Owner final verification

- Owner: Codex (orchestrator of record)
- Date: 2026-09-07
- Statement: I verified every happy path and edge case defined for this task has executable unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/rri_v2_candidate_test.py`; `python3 scripts/rri_v2_candidate.py --input docs/fixtures/rri-v2-candidate-valid.json`; `python3 scripts/rri_v2_candidate.py --input docs/fixtures/rri-v2-candidate-unknown.json`; `python3 scripts/rri_v2_candidate.py --input docs/fixtures/rri-v2-candidate-inapplicable.json`; `make qa-docs`; `git diff --check`
- Review artifact: docs/audit/gemma-evidence/T3.json

### Leaf RRI reports

T1/T3 leaf command:

```bash
python3 scripts/rri.py --C 1 --T 0 --K 1 --P 1 --D 1 --A 0 --X 2 --touches docs/schemas/rri-v2-candidate-assessment.schema.json --touches docs/fixtures/rri-v2-candidate-valid.json --touches docs/fixtures/rri-v2-candidate-invalid.json --touches scripts/rri_v2_candidate_test.py --touches docs/plan/rri-v2-candidate-calculator.md --touches docs/tasks/rri-v2-candidate-calculator.md
```

**Final RRI:** 21 -> band Low (0-25) -> Effort S. No penalties;
decomposition not triggered.

T2 leaf command:

```bash
python3 scripts/rri.py --C 2 --T 0 --K 2 --P 1 --D 2 --A 1 --X 2 --touches scripts/rri_v2_candidate.py --touches scripts/rri_v2_candidate_test.py --touches docs/plan/rri-v2-candidate-calculator.md --touches docs/tasks/rri-v2-candidate-calculator.md --touches docs/audit/rri-v2-candidate-calculator-verification.md
```

**Final RRI:** 30 -> band Moderate (26-40) -> Effort M. No penalties;
decomposition not triggered.
