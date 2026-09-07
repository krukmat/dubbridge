---
type: Audit
title: "RRI v2 candidate collector and calculator verification"
status: complete
date: 2026-09-07
task: docs/tasks/rri-v2-candidate-calculator.md
---
# RRI v2 candidate collector and calculator verification

## Scope and authorization boundary

This record covers the candidate-only implementation of
`rri-v2-design-0.2`. The owner explicitly authorized execution without another
task presentation and requested direct Codex implementation without a local
model implementer. The active `scripts/rri.py`, RRI/HITL policies, routing,
and authorization rules were deliberately left unchanged.

Phase-1 task-analysis review passed in
[the Gemma artifact](gemma-evidence/rri-v2-candidate-calculator-phase1.json).
Phase-2 code-solution review passed for the complete implementation; its
per-ledger receipts are [T1](gemma-evidence/T1.json),
[T2](gemma-evidence/T2.json), and [T3](gemma-evidence/T3.json).

## Verification results

| Check | Result | Evidence |
|---|---|---|
| Published schema/fixture contract | PASS | `SchemaContractTest`; parses the Draft 2020-12 document, pins the candidate version/mode, and checks the closed top-level/axis contract |
| Formula scenarios | PASS | Six point profiles reproduce documented ICI and breadth values |
| Uncertainty | PASS | Unknown Q encloses `[25,100]` without an ICI midpoint; a known L4 does not make unknown I complete |
| Invalid input | PASS | Missing axis, unknown version, reversed range, boolean/fractional/NaN/out-of-range values, status mismatch, and unknown size represented as zero fail closed |
| Inapplicability | PASS | Planning fixture emits `technical: null` and reason before axis validation |
| Candidate boundary | PASS | Every output fixes `candidate_only`, existing-RRI/HITL authorization, `not_calibrated`, and null P50/P90 |
| Active-RRI isolation | PASS | `git diff --exit-code -- scripts/rri.py docs/policies/RRI_POLICY.md docs/policies/HITL_AUTONOMY_POLICY.md docs/playbooks/AGENT_WORKFLOW_GUIDE.md` exited 0 |
| Reviewability/documentation | PASS | `make qa-review-budget` passed (0/6283 lines); `make qa-docs` and `git diff --check` passed |

Commands run:

```bash
python3 -m py_compile scripts/rri_v2_candidate.py scripts/rri_v2_candidate_test.py
python3 scripts/rri_v2_candidate_test.py
python3 scripts/rri_v2_candidate.py --input docs/fixtures/rri-v2-candidate-valid.json
python3 scripts/rri_v2_candidate.py --input docs/fixtures/rri-v2-candidate-unknown.json
python3 scripts/rri_v2_candidate.py --input docs/fixtures/rri-v2-candidate-inapplicable.json
git diff --exit-code -- scripts/rri.py docs/policies/RRI_POLICY.md docs/policies/HITL_AUTONOMY_POLICY.md docs/playbooks/AGENT_WORKFLOW_GUIDE.md
make qa-review-budget REVIEW_PATHS='scripts/rri_v2_candidate.py scripts/rri_v2_candidate_test.py docs/schemas/rri-v2-candidate-assessment.schema.json'
make qa-docs
git diff --check
```

## Limitations

- The JSON Schema is a published Draft 2020-12 contract and the standard-library
  collector verifies its declared closed candidate contract. The repository's
  Python environment has no installed `jsonschema` package (nor AJV), so no
  third-party Draft meta-schema run was added or claimed.
- The calculator proves deterministic contract behavior only. It does not prove
  the rubric's construct validity, assessor agreement, prospective calibration,
  effort prediction, or routing utility.
- No fitted coefficients, quantiles, probabilities, prediction intervals, or
  model-selection decision exist. `prediction_status=not_calibrated` and null
  effort quantiles are mandatory output invariants.
- Synthetic fixtures are formula-contract evidence, not historical DubBridge
  outcomes. Prospective shadow collection and the promotion criteria in the
  proposals remain future work.

No commit, push, policy/roadmap/ADR change, or modification of pre-existing
user-owned changes was performed.
