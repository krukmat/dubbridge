---
type: Proposal
title: "RRI v2 candidate formula and calibration specification"
status: Proposed
version: rri-v2-design-0.2
date: 2026-09-07
---
# RRI v2 candidate formula

**Derived from the [measurement model](rri-v2-model.md); uncalibrated.**
[ADR-045](../adr/ADR-045-rri-v2-authority-replacement.md) (2026-09-07) adopted
§1's bottleneck/ICI construct as authoritative in `scripts/rri.py`, ahead of
the §7 adoption sequence below (shadow scoring, then calibration, then
routing change) — the owner explicitly directed skipping that sequence. §5's
effort equation remains entirely unimplemented and uncalibrated: no
coefficients exist anywhere, `prediction_status=not_calibrated`, P50/P90 stay
null in every code path, including the now-authoritative one. This
specification has two levels: an immediately calculable ordinal technical
summary and a separate empirical effort equation. The first has no learned
coefficients but is now wired into band determination via ADR-045's bridge.
The second must not emit predictions before calibration.

## 1. Formula available before historical calibration

For a task in the implementation rubric, let `z = (L,I,Q,V)`, with every axis
an evidenced integer in `{0,1,2,3,4}`. Define:

```text
B(z)   = max(L, I, Q, V)
ICI(z) = 25 × B(z)
n_k(z) = Σ[j ∈ {L,I,Q,V}] 1[z_j ≥ k], for k = 1,2,3,4
H(z)   = (n_4, n_3, n_2, n_1)

technical_summary = {profile: z, bottleneck: B, display_index: ICI, breadth: H}
```

`ICI` means **Implementation Complexity Index**, a provisional ordinal screening
index. The factor 25 merely displays five ordered levels on a 0–100 scale.
It is not an estimated weight, percentage, probability, or amount of effort.
ICI 100 is not "twice as complex" as ICI 50. Prefer presenting `B` and the named
axes; show ICI only with its ordinal label.

`H` retains how many dimensions reach each level. It distinguishes a task with
one demanding dimension from one with four without inventing cross-axis weights.
It is descriptive: do not turn lexicographic H into an effort or safety ranking.
Retain the named profile because `(4,0,0,0)` and `(0,0,4,0)` need different skills
despite the same ICI and H. If a consumer stores only ICI, it loses essential data.

### Why maximum rather than a weighted mean

The initial decision is to expose the strongest supported reasoning obligation.
An average allows easy dimensions to dilute a difficult one. Maximum uses only
the ordering of common rubric levels, without assuming equal effort increments.
The assumption that level 3 is comparable across axes is a **rubric design
assumption to assess in the pilot**, not established science.

Maximum is deliberately coarse. Raising a non-dominant axis may leave ICI
unchanged; H and the profile show that change. Widespread moderate obligations
may require more effort than one demanding obligation; size and the eventual
effort model address that distinction. Maximum does not model synergistic
interactions and is not claimed to outperform the old RRI empirically.

An equal-weight average, arbitrary weighted sum, RMS, or product would introduce
unsupported interval-scale assumptions. A bottleneck-plus-average mixture would
add an uncalibrated mixing coefficient. None is adopted merely to produce a
more finely graduated number.

## 2. Unknown, estimated, inapplicable, and invalid inputs

For admissible per-axis ranges `l_j ≤ z_j ≤ u_j`:

```text
B_lower   = max_j(l_j)          B_upper   = max_j(u_j)
ICI_range = [25 B_lower, 25 B_upper]
H_lower   = (Σ 1[l_j≥4], Σ 1[l_j≥3], Σ 1[l_j≥2], Σ 1[l_j≥1])
H_upper   = (Σ 1[u_j≥4], Σ 1[u_j≥3], Σ 1[u_j≥2], Σ 1[u_j≥1])
```

These are enclosure bounds over the admissible axis box, not confidence
intervals. If cross-axis constraints exclude some combinations, the enclosure
may be loose; retain those constraints and do not claim the extremes are jointly
attainable. No midpoint or expected score is implied.

- Entirely unknown axis: `[0,4]`. Do not replace it with 0 or 1.
- Estimated point: keep `estimated` status even if `[l,u]` is a singleton.
- Fully inapplicable task (e.g. planning): `technical_summary=null`, reason given.
- Applicable task with proven absence of temporal changes: Q=0, not missing.
- Collector unavailable: unknown metric; an axis may still be supported by
  independent inspected evidence. Record that method; never invent a measurement.
- Out-of-range/fractional/boolean levels, reversed intervals, missing axis keys,
  NaN, or an unknown rubric version: validation error, no summary.

An unresolved axis prohibits the label `fully_assessed` even when another axis
fixes the bottleneck at 4 and therefore ICI_range is `[100,100]`. Distinguish
certainty of the summary from completeness of its underlying evidence. Until an
adopted policy exists, no ICI point/range authorizes implementation or model use.

## 3. Size, risk, and decomposition

Neither S nor R enters ICI. Both remain mandatory fields in the assessment
envelope. Twenty mechanical edits can have ICI 0 while still having significant
work size; a simple permission change can have ICI 25 and high operational risk.
Both still follow the existing RRI/HITL policy.

Assess each coherent child and the integration task. Their componentwise envelope
is a decomposition diagnostic, not a proven bound on parent difficulty:

```text
envelope_lower_j = max(child_1_lower_j, ..., child_m_lower_j, integration_lower_j)
envelope_upper_j = max(child_1_upper_j, ..., child_m_upper_j, integration_upper_j)
```

Independently assess the coherent parent using the same rubric. Coverage of all
child and cross-child obligations is necessary but does not prove that several
obligations at one level cannot jointly trigger a higher parent anchor. For
example, two children at L2 may together require the interacting-invariant
reasoning of L3. Reconcile any parent/envelope discrepancy with evidence; keep
both profiles and their scopes. Missing integration obligations remain unknown.
Never average child complexity to justify an easier parent. Preserve the original
parent snapshot and active approval envelope as evidence of any redesign.

Size aggregates only under its method's rules and with unique obligation/site
identifiers; do not count a shared contract twice. Total human work can be summed
over disjoint recorded phases, but quantiles generally cannot: adding child P90s
does **not** produce parent P90. Elapsed duration depends on dependencies,
parallelism, and shared resources; do not equate it to summed work.

## 4. Worked scenarios

These are synthetic **contract examples**, not measured DubBridge outcomes.
Profiles follow the declared scenario assumptions; real scoring needs evidence.
H is ordered `(n4,n3,n2,n1)`.

| Scenario | Assumptions supporting the profile | `(L,I,Q,V)` | ICI | H | Separate consideration |
|---|---|---|---:|---|---|
| Mechanical rename at 20 verified sites | No semantics/contract/state changes; exact transformation oracle | (0,0,0,0) | 0 | (0,0,0,0) | Size=20 sites; work is not zero |
| Same rename across 10 or 11 files | Identical 20 sites and obligations; only file packaging differs | (0,0,0,0) | 0 | (0,0,0,0) | No artificial 11-file jump |
| Internal constant adjustment | Direct rule, local behavior, unit oracle | (1,1,0,1) | 25 | (0,0,0,3) | Small maintenance task |
| Sensitive permission default adjustment | Same local obligations/oracle as above; larger harm if wrong | (1,1,0,1) | 25 | (0,0,0,3) | High risk; existing safeguards apply |
| Coordinated request/schema mapping | Multi-stage mapping, producer/consumer agreement, controlled integration oracle | (2,2,1,2) | 50 | (0,0,3,4) | Count contracts separately |
| Cancellation race in a short function | Ordering interactions and schedule exploration dominate | (2,2,3,3) | 75 | (0,2,4,4) | Few files do not make it easy |
| Distributed replay recovery | Algorithmic invariant across partial failure and restart; fault oracle | (3,4,4,3) | 100 | (2,4,4,4) | Integration/state dominate |
| Isolated demanding algorithm | Several algorithmic obligations; deterministic test oracle | (4,1,0,1) | 100 | (1,1,1,3) | Different skill needs from distributed recovery |
| New behavior with unresolved temporal semantics | Other axes evidenced at 1; Q unknown | (1,1,[0,4],1) | [25,100] | bounds | Investigate Q; no point estimate |
| Dominant known logic, unresolved integration | L4 is supported, I unknown | (4,[0,4],0,1) | [100,100] | bounds | Still incomplete; no `fully_assessed` claim |
| Planning document | Outside implementation rubric | n/a | n/a | n/a | Separate task family |

## 5. Empirical effort equation, after calibration

The [model](rri-v2-model.md) specifies resource targets and collection. Fit separate
models for human active minutes and agent execution seconds (and cost, if useful);
do not combine their units. For a chosen positive target `E_r`, typed size `s`,
reusable work family `f` (not `lineage_id`), and executor/environment vector `c`,
start with:

```text
η_τ = α_(r,f,τ)
    + β_(r,f,τ) log(1+s)
    + Σ[j ∈ {L,I,Q,V}] Σ[k=1..4] γ_(r,j,k,τ) 1[z_j ≥ k]
    + θ_(r,τ)^T c

Q_τ(E_r | s,z,c,f) = exp(η_τ)

β ≥ 0, γ ≥ 0                  τ ∈ {0.50, 0.90}
```

`Q_τ` is the conditional quantile. Fit by minimizing quantile (pinball) loss on
`log(E_r)` with preregistered regularization and nonnegative constraints. Each
ordinal step gets its own learned increment: a 1→2 step need not equal 3→4.
The intercept supplies a positive setup baseline when s=0 and z=0. Coefficients
and environment encodings are versioned model artifacts, not policy constants.
The additive form does not learn cross-axis interactions. If combined moderate
obligations are systematically underestimated, preregister a small interaction
candidate and compare it on validation data; neither max nor this additive model
currently solves that empirical question.

**Target eligibility:** Define whether the target is consumption until stopping
or resources until verified completion. A failed run has observed consumption,
but its resources-to-completion are not observed; these targets cannot share
labels. Record legitimate zero consumption (for example, no human intervention)
separately. This log-positive model describes the declared positive-target
population only; it cannot predict unconditional human effort when zeros occur.
Do not replace zero with an arbitrary epsilon. A separately validated zero-mass
plus positive model or a nonnegative-target model is required for that use.
Missing telemetry is unknown, never zero. Fix target units and size method/unit
per model version; `log(1+s)` operates on the numeric count in that fixed unit.

Quantile regression has an established statistical foundation in
[Koenker and Bassett, Regression Quantiles (1978)](https://people.eecs.berkeley.edu/~jordan/sail/readings/koenker-bassett.pdf).
This particular feature set, constraints, log link, and resource target remain
local modeling choices requiring validation.

Use a specific typed size per compatible family; no pooling "20 sites" and
"20 COSMIC points" as the same input. The initial candidate should be reduced
(fewer steps/axes or size-only) if data cannot support all parameters. Fit only
supported executor/family strata; unseen combinations abstain rather than receive
fabricated effects. Changes in model/hardware or review policy require reassessment.

P50 and P90 must be noncrossing on the supported domain; use a joint constraint or
documented quantile rearrangement and evaluate the resulting predictions. P90 is
a budget quantile, not a 90% confidence interval for a parameter and not a promise
of 90% success. Show observed holdout coverage and uncertainty before claiming
calibration. For continuous outcomes, the span [P50,P90] has nominal 40% coverage,
not 80% or 90%; ties can change interval coverage. No central prediction interval
is specified by this two-quantile model. Predictions for unknown z require
justified scenario treatment; with no joint distribution, return ranges of
scenario-conditional quantiles, not a fabricated mixture P90 or a prediction
interval. Abstain if any required scenario lies outside the supported domain.

Separate quantile models describe resource consumption, not semantic success
probability. A future success-under-budget classifier needs its own outcomes,
calibration, and selection-bias analysis. The difficulty profile may help route
skills, but this uncalibrated equation does not select an agent.

**Censoring:** Ordinary quantile fitting is only appropriate for compatible,
fully observed target records. Censored tasks cannot be silently removed from
the evaluation population or treated as completed at the timeout. If censoring
is material, use a declared censoring-aware estimator or restrict the estimand
explicitly and withhold general deployment claims.

**No numeric coefficients are supplied:** a data-collection and evaluation design
is available, but no adequate prospective labeled cohort has been established in
this task. `prediction_status=not_calibrated`; all effort quantiles are null.

## 6. Output example and evaluation order

Illustrative assessment, not an actual permission-change authorization:

```json
{
  "schema_version": "rri-v2-design-0.2",
  "mode": "candidate_only",
  "task_kind": "maintenance",
  "technical": {
    "profile": {"L": 1, "I": 1, "Q": 0, "V": 1},
    "bottleneck": 1,
    "ici": 25,
    "breadth": [0, 0, 0, 3],
    "scale": "ordinal"
  },
  "size": {"method": "unique_transformation_sites", "value": 1},
  "uncertainty": {"status": "fully_assessed", "unresolved": []},
  "risk": {"hazards": ["sensitive_permission_default"], "likelihood": null},
  "effort": {"prediction_status": "not_calibrated", "p50": null, "p90": null},
  "authorization": "governed_by_existing_RRI_and_HITL"
}
```

The production assessment envelope would also require task/baseline, assessor,
per-axis evidence, timestamps, and methods from the model. This example omits
those fields for readability and is not a complete machine schema.

```text
validate assessment version, scope, task kind and applicability evidence
if task kind is outside rubric:
    return inapplicable technical result with reason
validate required axis keys, evidence, statuses, levels and intervals
compute named technical profile, bottleneck, breadth and admissible bounds
attach typed size, unresolved questions and risk register
if calibrated model + supported inputs + valid target/stratum:
    compute versioned effort quantiles and report calibration evidence
else:
    abstain from effort prediction with a reason
emit candidate report; existing RRI/HITL remains the authorization source
```

## 7. Adoption sequence and unresolved empirical questions

1. Repair active measurement defects through a separately scoped development
   change with its required independent review; this document changes no code.
2. Implement a separate candidate collector/calculator and schema after design
   adoption; verify missing-data behavior and all contract examples.
3. Run prospective shadow scoring alongside active RRI, without changing routing.
4. Fit and compare the effort candidate under the model's preregistered protocol.
5. Only after evaluation, adopt explicit hazard/evidence gates and routing rules;
   propagate any policy decision through canonical workflow artifacts.

Open empirical questions: cross-axis level comparability; assessor agreement;
how much size explains; whether state and integration add distinct information;
whether the maximum screen misses combined moderate difficulty; how well results
transfer between Rust/mobile/tooling and between executors. Favor the simplest
model that passes prospective validation. If the new features do not help, keep
the useful measurement corrections and reject the unsupported predictor.

Reproducible checks and current evidence limits:
[design validation audit](../audit/rri-v2-design-validation.md).
