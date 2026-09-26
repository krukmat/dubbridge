---
type: Audit
title: "RRI v2 design validation and evidence limits"
status: complete
date: 2026-09-07
task: docs/tasks/rri-v2-model-and-formula.md
---
# RRI v2 design validation

## Scope, authorization, and review applicability

Owner request: "pues trabaja en el modelo y luego en la formula", following the
RRI evaluation in the same conversation. Scope is model/formula design and
documentation verification. The owner has authorized this work; no repeated
permission request is needed. Production calculator repair, policy activation,
empirical data collection, commits, and pushes are outside this deliverable.

Baseline HEAD: `7d31d8bb394153751532fad4e9fbb59aa9bc9aca`.
Initial user-owned modification: `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`;
the agent did not edit that file. All five new artifacts are named in the
[plan](../plan/rri-v2-model-and-formula.md).

Task-analysis review: n/a — docs/planning-only exemption.
Code-solution review: n/a — docs/planning-only exemption.
Primary design review: Codex; not an independent empirical or code review.
Ollama restart/precheck: n/a — no local-model role invoked.
Behavioral coverage/owner code certification: n/a — no production code changes.

## Design review findings and dispositions

| Finding | Disposition |
|---|---|
| Difficulty is latent; minutes and risk are different constructs | Four-output model, with resource predictions separate |
| Another weighted sum would reproduce unsupported scale assumptions | Ordinal max screen plus complete named profile and breadth; no fitted weights invented |
| Maximum hides breadth and non-dominant increases | Mandatory H/profile; coarse-screen limitation explicitly documented |
| Equal levels across axes are not automatically comparable | Explicit design assumption, tested through independent assessor pilot |
| Unknown can yield a deceptively precise score | Admissible bounds and separate assessment completeness; `[100,100]` can still be incomplete |
| Missing tests and difficult test oracles are different | V measures oracle obligations; harness debt and uncertainty are separate |
| Security can be counted repeatedly | Risk register outside ICI; shared technical evidence IDs and feature ablations |
| Effort model could impose equal ordinal increments | Learned nonnegative cumulative step effects instead of linear ordinal values |
| P90 sums and censored tasks can mislead | No summed-quantile claim; explicit censoring contract and prediction abstention |
| Existing routing creates executor/verification confounding | Capture policy/strata; no causal attribution to a model or intrinsic difficulty |

These are primary-agent design checks, not peer-review certifications. Reflection
logs are mandatory for development tasks; this documentation work is exempt.

## Existing evidence inspected

| Artifact | Available evidence | Missing for prospective effort validation |
|---|---|---|
| [Original RRI plan](../plan/rri-integration.md) | Adoption rationale and hand-chosen weights/rubric | No empirical weight fit or held-out predictive comparison in the plan |
| [Calculator plan](../plan/rri-calculator-script.md) | Deterministic arithmetic design and correctness motivation | Determinism is not predictive calibration |
| [S-150-T2c assessment](s-150-t2c-rri.md) | Seven-path scope, CC input, domain/coupling rationale, legacy RRI 65 | No complete prospective new-axis snapshot plus standardized effort outcomes |
| [S-230 failed-attempt receipt](low-rri/s-230-t1-prod-attempt1-failed.json) | Patch and suggested commands | Does not establish verified success, active duration, or full retry trajectory |
| [S-230 applied-attempt receipt](low-rri/s-230-t1-staging-attempt1-applied.json) | Applied patch receipt | No joined prospective feature/resource dataset established by this inspection |

This is a small provenance/availability inspection, not an exhaustive audit of
all telemetry in the repository. No historical task was assigned a fabricated
new score or resource outcome. No coefficients were fitted.

## Reproducible mathematical check

The Python block below checks a mathematical specification, not a production
implementation. Enumerating admissible profiles provides an independent oracle
for interval bounds. It also checks coordinatewise monotonicity and the published
point scenarios. It does not prove that humans agree with the rubric or that it
predicts implementation difficulty.

Run from the repository root:

```bash
python3 -B - <<'PY'
from pathlib import Path
doc = Path('docs/audit/rri-v2-design-validation.md').read_text()
source = doc.split('```python\n', 1)[1].split('\n```', 1)[0]
exec(compile(source, 'rri-v2-mathematical-check', 'exec'))
PY
```

```python
from itertools import product

def summary(z):
    if len(z) != 4 or any(type(v) is not int or not 0 <= v <= 4 for v in z):
        raise ValueError('expected four integer levels in 0..4')
    return 25 * max(z), tuple(sum(v >= k for v in z) for k in (4, 3, 2, 1))

def bounds(ranges):
    if len(ranges) != 4:
        raise ValueError('expected four axis ranges')
    lo, hi = zip(*ranges)
    if any(a > b for a, b in ranges):
        raise ValueError('reversed interval')
    return summary(lo), summary(hi)

profiles = list(product(range(5), repeat=4))
steps = 0
for z in profiles:
    score, h = summary(z)
    assert score in (0, 25, 50, 75, 100)
    assert 0 <= h[0] <= h[1] <= h[2] <= h[3] <= 4
    for j in range(4):
        if z[j] == 4:
            continue
        raised = tuple(v + (i == j) for i, v in enumerate(z))
        new_score, new_h = summary(raised)
        assert new_score >= score and all(a >= b for a, b in zip(new_h, h))
        assert sum(new_h) == sum(h) + 1
        steps += 1

axis_ranges = [(a, b) for a in range(5) for b in range(a, 5)]
boxes = members = 0
for ranges in product(axis_ranges, repeat=4):
    low, high = bounds(ranges)
    values = [summary(z) for z in product(*(range(a, b+1) for a, b in ranges))]
    assert low[0] == min(s for s, _ in values)
    assert high[0] == max(s for s, _ in values)
    for j in range(4):
        assert low[1][j] == min(h[j] for _, h in values)
        assert high[1][j] == max(h[j] for _, h in values)
    boxes += 1
    members += len(values)

examples = [
    ((0, 0, 0, 0), (0, (0, 0, 0, 0))),
    ((1, 1, 0, 1), (25, (0, 0, 0, 3))),
    ((2, 2, 1, 2), (50, (0, 0, 3, 4))),
    ((2, 2, 3, 3), (75, (0, 2, 4, 4))),
    ((3, 4, 4, 3), (100, (2, 4, 4, 4))),
    ((4, 1, 0, 1), (100, (1, 1, 1, 3))),
]
for z, expected in examples:
    assert summary(z) == expected
assert [s[0] for s in bounds(((1,1),(1,1),(0,4),(1,1)))] == [25,100]
assert [s[0] for s in bounds(((4,4),(0,4),(0,0),(1,1)))] == [100,100]
assert summary((4,0,0,0))[0] == summary((0,0,4,0))[0]

bad = [(-1,0,0,0), (5,0,0,0), (0.5,0,0,0), (True,0,0,0),
       (float('nan'),0,0,0), (0,0,0)]
for z in bad:
    try:
        summary(z)
    except ValueError:
        pass
    else:
        raise AssertionError('invalid profile accepted')
try:
    bounds(((3,1),(0,0),(0,0),(0,0)))
except ValueError:
    pass
else:
    raise AssertionError('reversed interval accepted')
print(f'PASS: {len(profiles)} profiles; {steps} monotone steps')
print(f'PASS: {boxes} interval boxes; {members} admissible profile evaluations')
print(f'PASS: {len(examples)} distinct point scenarios; 2 unknown-bound scenarios')
print(f'PASS: {len(bad)} invalid profiles and reversed-interval rejection')
```

## Verification results

The mathematical block above ran successfully on 2026-09-07:

```text
PASS: 625 profiles; 2000 monotone steps
PASS: 50625 interval boxes; 1500625 admissible profile evaluations
PASS: 6 distinct point scenarios; 2 unknown-bound scenarios
PASS: 6 invalid profiles and reversed-interval rejection
```

- Final local document check: five files, 23 local Markdown links resolved; code fences
  balanced, frontmatter present, no trailing whitespace. External sources were
  inspected through web retrieval separately; this is not a live URL crawler.
- JSON example: parsed with Python `json.loads`, agreed with ICI, retained null
  effort quantiles and `candidate_only` mode.
- `git diff --check`: passed (tracked diff); the explicit whitespace check above
  also covers these new, untracked documents.
- `make qa-docs`: passed, exit 0. Documentation consistency, 8 behavioral-coverage
  tests and checker, 7 BDD-map tests and checker, 24 task-coverage tests and task
  completion checker, roadmap drift, and OKF frontmatter all passed.

The plan and task ledger are closed for **design work only**. The two proposals
remain `Proposed`; no production policy, calculator, routing, or roadmap status
was changed. All five authored files remain uncommitted.

Empirical predictive validity remains `not_calibrated` regardless of the
mathematical and documentation checks. No unit coverage or independent review
certification is claimed for a production implementation.

## Legacy RRI computation for this authorized design task

Inputs reflect documentation design, not the hypothetical future production
implementation: C0 (no code), T0 (documentation checks), D2 (measurement design),
A1 (bounded objective with design choices), K1/P1 (linked inactive documents),
X3 (one workflow/model context), plus `arch_decision +12`. T3 verifies the chosen
design rather than deciding policy, so D1/A0/X2 and no decision penalty apply.
The calculator labels supplied values High by default; this is its output, not
an empirical confidence certification. Exact commands and unmodified outputs
follow. The T3 Low-band local-delegation text is superseded for docs-only work
by the workflow guide: primary-agent authoring is the applicable route.

### Parent

```bash
python3 scripts/rri.py --C 0 --T 0 --K 1 --P 1 --D 2 --A 1 --X 3 --penalty arch_decision --touches docs/plan/rri-v2-model-and-formula.md --touches docs/tasks/rri-v2-model-and-formula.md --touches docs/proposals/rri-v2-model.md --touches docs/proposals/rri-v2-formula.md --touches docs/audit/rri-v2-design-validation.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | agent-supplied score | High |
| F files | 2 | --touches -> 5 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 1 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 21
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 33 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered

### T1

```bash
python3 scripts/rri.py --C 0 --T 0 --K 1 --P 1 --D 2 --A 1 --X 3 --penalty arch_decision --touches docs/proposals/rri-v2-model.md --touches docs/plan/rri-v2-model-and-formula.md --touches docs/tasks/rri-v2-model-and-formula.md --touches docs/audit/rri-v2-design-validation.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | agent-supplied score | High |
| F files | 2 | --touches -> 4 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 1 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 21
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 33 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered

### T2

```bash
python3 scripts/rri.py --C 0 --T 0 --K 1 --P 1 --D 2 --A 1 --X 3 --penalty arch_decision --touches docs/proposals/rri-v2-formula.md --touches docs/plan/rri-v2-model-and-formula.md --touches docs/tasks/rri-v2-model-and-formula.md --touches docs/audit/rri-v2-design-validation.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | agent-supplied score | High |
| F files | 2 | --touches -> 4 files | High |
| D domain | 2 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 1 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 21
**Penalties applied:** arch_decision (+12, manual flag)
**Final RRI:** 33 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
**Gates for this band:** Confirm tests exist in the affected area.
**Decomposition:** not triggered

### T3

```bash
python3 scripts/rri.py --C 0 --T 0 --K 1 --P 1 --D 1 --A 0 --X 2 --touches docs/proposals/rri-v2-model.md --touches docs/proposals/rri-v2-formula.md --touches docs/plan/rri-v2-model-and-formula.md --touches docs/tasks/rri-v2-model-and-formula.md --touches docs/audit/rri-v2-design-validation.md
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | agent-supplied score | High |
| F files | 2 | --touches -> 5 files | High |
| D domain | 1 | agent-supplied (no rubric match) | High |
| T coverage | 0 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 1 | agent-supplied (no rubric match) | High |
| P impact | 1 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 15
**Penalties applied:** none
**Final RRI:** 15 -> band Low (0-25) -> Effort S . Codex Local Qwen Developer via Ollama . Claude Local Qwen Developer via Ollama . thinking Off
**Gates for this band:** Local delegation: delegate to local Qwen Developer via Ollama; validate and apply only an in-scope diff; review against requirements; verify; report.
**Decomposition:** not triggered

## T4 follow-up gap review — 2026-09-07

Owner request: "dale una revision para verificar que no haya gaps o
inconsistencias". Primary Codex reviewed and corrected the same five documents;
this is not an independent peer review. The current candidate is
`rri-v2-design-0.2`; the original arithmetic rule and its exhaustive checks are
unchanged. The version change records substantive measurement/evaluation contract
clarifications; no old assessments are silently relabeled as the new version.

The parent command recorded above was rerun before corrections with identical
inputs and output: base 21, `arch_decision +12`, final 33 / Moderate / Effort M.
Authorization covers this bounded review and correction. Phase-1 and phase-2
independent development reviews remain n/a under the documentation exemption.

| Finding | Consequence before correction | Disposition in 0.2 |
|---|---|---|
| High: `family` meant both task lineage and reusable work category | Holding out all families would leave the family intercept unsupported on every test task | Separate `lineage_id` for leakage/clustering from `work_family` for compatible learned effects |
| High: child maxima were labeled parent bounds | Coverage alone does not prove compositionality; interacting obligations can exceed the envelope | Require direct parent assessment and reconciliation; child maxima are a diagnostic only |
| High: positive log target omitted legitimate zeros and stopping semantics | Zero human involvement cannot be logged; failed-run consumption is not resources to successful completion | Define target endpoint, positive-population restriction, zero/missing distinction, and abstention or separately validated zero-aware modeling |
| Medium: selection/refitting and "best baseline" were ambiguous | Holdout-based selection or future labels could leak into fitting or promotion | Freeze candidates and selection rules; select on validation; quarantine spanning lineages; freeze comparator and interval method before holdout |
| Medium: generic interval coverage did not identify endpoints | Readers could mistake P50/P90 for a central 80% or 90% interval | Define marginal quantile coverage and the continuous-distribution 40% span; distinguish scenario-conditional ranges |
| Medium: additive effort equation could appear to address interactions | Separate moderate axes can have joint effects absent from the equation | Explicit limitation; interaction candidates need preregistration and validation |
| Medium: reliable scoring could be mistaken for complete construct coverage | Diagnosis/legacy comprehension may not fit existing anchors despite assessor agreement | Pilot unmatched obligations; preserve unresolved assessments and revise rubric rather than force low scores |
| Medium: missing-data and assessment-status rules were incomplete | Resolved estimates could look like direct measurements; unsupported scenarios or selective abstention could inflate reported accuracy | Distinguish judgment from observation, define completeness, abstain outside support, report abstention rate and common evaluation population |
| Low: pseudocode validated axis inputs before applicability | Inapplicable documentation tasks could fail missing-axis validation | Validate scope/applicability first; validate axis fields only for applicable tasks |
| Low: closed ledger still said integration QA had not passed | Historical handoff wording contradicted completion | Synchronize T1/T2 verification wording, versions, plan, and T4 closure |

### Verification scope and remaining gaps

The exhaustive Python block checks arithmetic on tuples and interval boxes only.
It does **not** test a production JSON schema, unknown version handling, evidence
provenance, applicability dispatch, model support detection, fitted quantiles, or
assessment completeness. Those require the future collector/schema implementation
and its behavioral tests. No machine schema or production calculator is delivered
by this documentation review.

The following remain empirical adoption requirements, not closed by these edits:

- Coverage and cross-axis comparability of the rubric, and independent assessor
  agreement on version 0.2; unmatched diagnosis/comprehension cases need a pilot.
- Prospective labeled outcomes with explicit target endpoint, zeros/censoring,
  reproducible work-family size units, and supported executor strata.
- Identifiable coefficients and held-out calibration, including combined moderate
  obligations and abstention behavior. No numeric coefficients are available.
- Owner-fixed resource/success tolerances, evaluation uncertainty procedure, and
  future hazard/routing policy adoption. The active RRI still governs execution.

Source retrieval note: the previously cited Koenker/Bassett PDF could not be
retrieved again through the web tool during T4 (redirect/safety fetch errors).
No fresh external-source verification is claimed. The P50/P90 span clarification
follows directly from quantile endpoints for a continuous distribution.

T4 renewed checks passed on 2026-09-07:

- Replayed the embedded mathematical check: 625 profiles, 2,000 monotone steps,
  50,625 boxes, 1,500,625 admissible profile evaluations; all scenarios and invalid
  tuple/reversed-interval checks passed.
- `make qa-docs`: exit 0; documentation consistency, behavioral coverage,
  BDD mapping, task completion evidence, roadmap drift, and frontmatter passed.
- Five-document local check: 23 local Markdown links resolved; frontmatter and
  balanced fences present; no trailing whitespace; example JSON parsed and its
  ICI/breadth/version/null-effort values matched the specification. Checked that
  applicability precedes axis validation and the old parent-bound claim is gone.
- `git diff --check`: exit 0. The explicit local check also covers untracked files.
- Plan/task closure and candidate version synchronized. All five authored files
  remain uncommitted; the pre-existing user task edit was not modified by T4.

Disposition: review and bounded documentation corrections complete. Candidate
remains inactive and `not_calibrated`; remaining empirical requirements above are
not waived or reported as completed.

## T5 final review — 2026-09-07

Owner request: "revision final". Primary Codex reviewed the final five-document
set against all ten T4 findings. Task-analysis review and code-solution review:
n/a, documentation-only exemption. This is a final primary-agent review, not an
independent scientific validation. The exact T3 scoring command was rerun:
base 15, no penalties, final RRI 15 / Low / Effort S; parent design envelope 33.

### Final assessment

No additional blocking inconsistency was identified within the candidate design
scope. No model/formula edits were needed; both remain version
`rri-v2-design-0.2`, `Proposed`, inactive, and `not_calibrated`.

- Measurement judgments, observed evidence, uncertainty ranges, and technical
  completeness have distinct meanings; no point ICI implies complete evidence.
- Every formula axis maps to the rubric; examples and breadth ordering agree
  with the equation. Typed size and operational hazards remain separate.
- Parent assessment is direct; the child envelope is only diagnostic. Neither
  that envelope nor additive effort effects claim to establish joint difficulty.
- Lineage isolation and reusable work-family effects are separated; selection
  and refitting precede the untouched holdout under declared rules.
- Positive-target, zero, missing, failed, and censored records have explicit
  limits. Quantile/scenario ranges do not imply success probabilities.
- The audit distinguishes tuple arithmetic checks from future machine-schema,
  collector, and fitted-predictor tests. Documentation completion does not imply
  empirical validation or permission to replace the active RRI.

Remaining adoption requirements are exactly those recorded under T4: rubric
coverage/comparability and assessor agreement; prospective outcome collection;
calibration and support/abstention evaluation; owner-defined tolerances and an
explicit governance decision. This review does not establish that no future gap
can exist, or that the new candidate outperforms the active RRI.

### Reviewed proposal fingerprints

```text
e3cc23b7a59d540ca5a40ae7b85100e4f39c3480274250559901f37cac03e811  docs/proposals/rri-v2-model.md
0a194217137574faf5160c74a8515ff709aa680608531219ca25a115e47643b7  docs/proposals/rri-v2-formula.md
```

### Final verification

- Replayed the embedded check: all 625 profiles, 2,000 monotone steps, 50,625
  interval boxes, 1,500,625 admissible evaluations, scenario and invalid-input
  checks passed.
- Five-document metadata/fences/whitespace check passed; 23 local links resolved.
  JSON example, version 0.2, ICI/breadth arithmetic, null effort predictions,
  applicability ordering, and removal of the parent-bound claim checked.
- `make qa-docs`: passed, exit 0, including documentation consistency, behavioral
  coverage, BDD mapping, task completion evidence, roadmap drift, and frontmatter.
- Plan/task closure synchronized; final whitespace/fence/status checks and
  `git diff --check` passed. Proposals remain unchanged and uncommitted.

External sources were not revalidated in this final local consistency pass;
source retrieval limitations from T4 remain recorded above.
