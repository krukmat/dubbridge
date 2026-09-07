---
type: Proposal
title: "RRI v2 measurement model: difficulty, size, uncertainty, and risk"
status: Proposed
version: rri-v2-design-0.2
date: 2026-09-07
---
# RRI v2 measurement model

**Candidate specification; not empirically calibrated.**
[ADR-045](../adr/ADR-045-rri-v2-authority-replacement.md) (2026-09-07) adopted
a bridged version of this model's technical profile as the authoritative
scoring mechanism in `scripts/rri.py`, ahead of the empirical validation
this proposal's own adoption sequence called for — the owner explicitly
overrode that sequence. The bridge derives `(L,I,Q,V)` deterministically from
existing agent-supplied variables rather than the independent per-axis
assessment this document describes; see ADR-045 for the exact scope. The
[active RRI policy](../policies/RRI_POLICY.md) continues to govern approval,
review, and model routing, now computed via that bridge. This proposal still
defines the fuller independently-assessed-axis construct that
[the formula](rri-v2-formula.md) and `scripts/rri_v2_candidate.py` implement
in full.

## 1. Construct and intended decisions

Implementation difficulty is the difficulty of establishing a specified behavior
and its invariants across the affected logic and boundaries, under a declared
verification contract. It is a latent construct, not directly measurable in
minutes, files, cyclomatic complexity, or tokens. An ordinal technical profile
operationalizes that definition; observed effort and success can test its utility
but are not identical to it.

Return four separate outputs, plus any calibrated effort prediction:

| Output | Representation | Intended use |
|---|---|---|
| Technical difficulty `Z` | Four anchored ordinal levels `(L,I,Q,V)`, each 0–4 or an interval | Identify the reasoning/verification bottleneck and useful decomposition seams |
| Work size `S` | Typed count with a declared measurement method and scope | Estimate amount of work within comparable task families |
| Uncertainty `U` | Missing evidence, unresolved decisions, admissible level ranges, provenance | Decide what to investigate; prevent unsupported point estimates |
| Operational risk `R` | Named hazards, impact/exposure/reversibility, governing rule references | Determine safeguards separately from implementation difficulty |
| Effort `E`, when supported | Conditional quantiles in an explicit resource unit | Budget execution, verification, and review under a specified environment |

Do not call a risk score complexity. Do not interpret a high-risk one-line change
as a difficult algorithm. Do not infer that a large repeated edit is technically
hard because it touches many files. Conversely, a small concurrent state change
can demand substantial reasoning.

## 2. Unit of assessment and time boundary

The unit is a **coherent change request**, with acceptance criteria, explicit
non-goals, baseline revision, and stable lineage/work-family identifiers. Its file
count is descriptive, not its identity. Capture:

- `task_id`, `parent_id`, `lineage_id`, `work_family`, `kind`, `baseline_sha`, `snapshot_at`;
- `measurement_stage`: `pre_implementation` or `post_implementation`;
- affected behaviors, symbols/contracts, planned paths, acceptance/oracle refs;
- rubric version, assessor, evidence refs, tools and versions;
- executor/model version, reasoning setting, hardware, toolchain, context policy,
  and verification/review policy as separate environment fields.

Freeze predictions before implementation. Planned paths/symbols and complexity of
new code are estimates even when their counts are deterministic. Post-change CC,
actual diff size, repairs, and duration are outcomes, never pre-change inputs.
Scope revisions create a new snapshot; preserve the original for evaluation.

`lineage_id` groups a parent, children, retries, and scope revisions for leakage
control and clustered evaluation. `work_family` is a reusable category with a
compatible size method and target (for example, mechanical maintenance). These
are different identifiers: the effort equation can learn an effect for a recurring
work family, never an intercept for an unseen task lineage. Declare task kind to
work-family assignment rules before collecting the evaluation cohort.

For edited code, inspect functions materially involved in the behavior or its
required comprehension, with a reason for each inclusion. A complex unrelated
function elsewhere in a touched file is not automatically in scope. A context
dependency required to establish an invariant can be in scope without being edited.

## 3. Measurement evidence and tool rules

Each feature has `value` or `[lower,upper]`, `unit`, `status`, `method`, and
`evidence_refs`. Status is one of `observed`, `estimated`, `unknown`, or
`not_applicable`. Assessor agreement is measured separately; an agent-supplied
number is not automatically "High confidence". Technical levels are rubric
judgments and retain `estimated` status even when based on observed evidence;
`observed` describes directly collected inputs, not proven intrinsic difficulty.
`fully_assessed` means all applicable axes have evidenced, resolved levels with
no open scoring questions; it does not mean their judgments are objectively true.

| Evidence | Collection rule | Interpretation limit |
|---|---|---|
| Cyclomatic complexity | Language-aware CFG/AST tool; record per-function values, max, affected function count, tool/version/scope | Branch structure, not full implementation difficulty; never substitute cognitive complexity under the same metric name [S1] |
| Cognitive complexity | Separate metric/collector identifier, if used | A comprehension proxy; do not apply McCabe cutoffs to it |
| Contract graph | Enumerate boundary contracts and dependency edges, identifying which semantics change | Count supports size; coordination semantics support I |
| State/invariant inventory | States, events, transitions, temporal obligations, atomicity and failure scenarios | Do not count hypothetical Cartesian products as reachable states |
| Verification evidence | Oracle, controlled inputs, integration fixtures, nondeterminism, assertions, mutation evidence where useful | Code coverage shows execution reach, not assured defect detection [S3] |
| Requirement/change inventory | Behavioral changes or formal functional-size measurement | Task prose length and arbitrary acceptance-bullet counts are not size measures |

Collector failure, unsupported syntax, partial analysis, or no scoped symbols
must never report a valid low measurement. Preserve exit status and diagnostics.
Clippy's cognitive lint is threshold-based (default 25); no warning is not a CC
measurement [S2]. Unknown data stay unknown. `not_applicable` requires evidence
of inapplicability, not simply the absence of a tool.

## 4. Technical profile rubric

Each axis uses levels **0 mechanical, 1 local, 2 coordinated, 3 interacting,
4 demanding**. These are order labels, not equally spaced effort quantities.
Select the highest applicable evidenced anchor on each axis. Record why the next
level is not supported; if multiple levels remain plausible, record their range.

| Level | L — logical/algorithmic obligations | I — contract/integration obligations |
|---|---|---|
| 0 | No semantic logic change; mechanical transformation proven by a rule | No behavioral contract change |
| 1 | Direct rule or mapping; independent cases with exact expected results | Contained contract change with no coordinated consumer semantics |
| 2 | Interdependent conditions or multi-stage transformation using known local invariants | Producer/consumer or persistence contracts must agree under a controlled rollout |
| 3 | Nontrivial algorithmic invariant, interacting rules, or a required performance bound needing explicit reasoning | Version/ownership/lifecycle compatibility across independently evolving components |
| 4 | Several interacting algorithmic invariants or correctness and resource bounds requiring substantial derivation | Cross-boundary invariant must survive partial failure, mixed versions, and recovery |

| Level | Q — state/temporal obligations | V — verification/oracle obligations |
|---|---|---|
| 0 | No state or temporal behavior changes | Exact mechanical oracle: specified text/AST transformation or equivalent |
| 1 | Local sequential state with explicit deterministic transitions | Deterministic unit oracle with enumerated relevant cases |
| 2 | Async/transactional lifecycle under known ordering, cancellation, and ownership rules | Controlled integration/contract oracle across real boundary semantics |
| 3 | Races, retries, cancellation, or ordering interact and need schedule/failure analysis | Schedule/fault exploration, performance experiment, or partial oracle requires designed controls |
| 4 | Distributed consistency/liveness across crash/restart, duplication, and partial failure | Correctness requires multiple complementary or statistical oracles with explicit validity/error criteria |

Examples describe obligations, not automatic directory or framework floors.
Using async syntax alone does not imply Q3. Working in auth does not automatically
imply L4. Merely lacking tests does not imply V4: record test debt in the execution
environment and uncertainty about the oracle. An undefined verification strategy
is unresolved, not an evidenced top-level oracle.

The axes can overlap: a distributed invariant can legitimately affect I and Q.
Store the shared evidence identifier. Do not pretend these are independent
probabilistic events or add a second penalty for the same observation. Any future
predictor must test whether both features add out-of-sample information.

**Comparability:** Two assessors independently score a pilot without seeing each
other's ratings or the old RRI. Measure exact agreement and linearly weighted
kappa per axis [S7], show disagreement counts, and revise confusing anchors. Kappa
depends on prevalence and does not establish construct validity. A substantive
rubric revision increments its version and requires a new assessment sample.
The coverage of these four axes is itself a hypothesis. In the pilot, record
hard tasks that the anchors cannot express, including difficult diagnosis or
required comprehension of legacy behavior. Do not force unmatched obligations
into a convenient low anchor: mark the affected assessment unresolved and revise
the rubric if needed. Keep diagnostic evidence about behavior separate from the
final patch size. Agreement alone cannot establish that important difficulty
dimensions are covered.

## 5. Work size without file-count cliffs

Select the size method by task kind before scoring:

- **Functional change:** COSMIC changed functional size when requirements and
  boundaries support a rules-compliant measurement. Count qualifying data
  movements added, modified, or deleted; retain the decomposition and method
  version [S4]. Do not call a home-grown count "COSMIC".
- **Bounded maintenance/refactor:** unique transformation sites and affected
  behavioral contracts, recorded separately. Generated copies and formatting
  churn are separate counts. Use one preregistered size feature within a family;
  do not add these unlike units together.
- **Algorithmic/non-functional change:** family-specific units, such as kernels
  or explicitly changed constraints; supplement with L and V. If no reproducible
  size definition exists, abstain from the size-based predictor.
- **Documentation/planning:** outside this implementation rubric. Use a separate
  work family, never manufacture zero technical scores for an inapplicable task.

Keep raw file counts and changed-symbol counts for diagnostics and baselines.
They may become predictive features if validation supports them, but they do not
define universal complexity thresholds. Splitting a file changes no technical
score if obligations and behavior remain identical.

## 6. Uncertainty and operational risk

For every unresolved point record the question, plausible alternatives,
affected dimensions, evidence needed, and owner. With no useful information an
axis range is `[0,4]`; with evidence restricting it, narrow the range. An interval
is a set of plausible scores, **not** an 80% confidence interval. Do not assign
probabilities to alternatives without a justified model or elicitation process.

Risk is a hazard register: authorization/ownership changes, sensitive data,
irreversible mutations, compatibility, blast radius, exposure, and recovery.
State impact and likelihood evidence separately; unknown likelihood remains
unknown. Do not multiply ordinal severity and likelihood into an expected loss.
Expected loss is meaningful only with justified probabilities and impact units.
Changing only risk leaves `(L,I,Q,V)` unchanged but may change required safeguards.

Existing RRI/HITL gates remain authoritative during all candidate evaluation.
Future adoption must explicitly map hazard rules and evidence requirements to
approval/review; it cannot simply reuse the old numerical band thresholds.

## 7. Outcomes and empirical validation contract

The immediate deliverable is an operational definition, not a claim that it
predicts agent performance. Prospective data must separate:

- human active minutes (implementation, verification, review, and repair phases);
- agent execution seconds and tokens/cost by exact executor and hardware;
- elapsed calendar duration, including queues and approval waits, separately;
- success against frozen acceptance tests, repairs, escaped defects over a fixed
  observation window, and infra/tool failures separately from semantic failures.

Do not sum human minutes and machine seconds into an unnamed "effort" number.
Include abandoned/timed-out tasks and explicit censoring; otherwise the model
learns only easy successes. Retries belong to one task; child tasks and integration
belong to one lineage. Record risk-driven extra review so its cost is not mistaken
for intrinsic technical difficulty. Observational executor effects are not causal:
hard tasks are already routed to stronger agents under the active policy.

Validation protocol:

1. Pilot the collection/rubric on varied completed tasks only to find data and
   anchor defects. Label retrospective reconstructions; never treat them as
   trustworthy prospective forecasts or use unavailable pre-change information.
2. Freeze rubric, candidate feature sets, target, baseline candidates, and
   evaluation/selection rules. Collect a
   prospective cohort. Existing audit receipts may seed case discovery, not
   fabricated effort labels. The [audit](../audit/rri-v2-design-validation.md)
   describes the limited existing sample inspected for this design.
3. Partition chronologically by disjoint lineages; never scatter siblings,
   retries, or revised snapshots across partitions. Lineages spanning a cutoff
   must be held out or quarantined under a frozen rule, never moved into training
   with future outcomes. Reusable work families may occur in all partitions;
   unseen work families remain unsupported. Fit preprocessing and coefficients
   on training records; select hyperparameters/features and the comparator using
   validation records only. Refit on training plus validation only if declared
   in advance. Freeze everything before opening the final holdout.
4. Compare training-median and comparable-task/size-only baselines, a model using
   the old RRI, and the candidate. Old RRI is not measured in minutes: fit its
   effort mapping under the same training/refit protocol before comparing errors.
5. Report MAE in the target unit, median absolute error, paired error differences
   with lineage-clustered uncertainty, quantile loss, and quantile coverage.
   P50 and P90 are assessed separately (nominal 50% and 90%); their span is not
   an 80% prediction interval. Any additional interval must declare endpoints,
   nominal coverage, and width. Record the scored population and abstention rate,
   comparing systems on the same records; abstention must not hide hard cases.
   Separately assess false-easy cases using a frozen success/resource definition.
   Baselines and effect sizes matter more than an attractive fitted curve [S6].
6. Report by language, work family, executor, risk/review policy, and time period.
   Test ablations for correlated axes and size. Sparse subgroups stay unsupported.
   Add data until uncertainty supports the decision; no fixed sample size alone
   certifies validity. Preregister collection endpoints to avoid repeated peeking.

**Proposed promotion criterion:** On the untouched prospective holdout, the paired
MAE difference (candidate minus the baseline selected on validation data and
frozen before holdout access) must have its 95% interval
entirely below zero. Resource/success and interval-quality tolerances must be fixed
by the owner before collection and also pass; improvement in MAE cannot compensate
for unsafe false-easy classifications. These are proposed evaluation decisions,
not research-established universal thresholds. Freeze the interval estimator,
lineage resampling unit, and primary target/comparison before holdout access;
do not select the winning baseline or subgroup after inspecting test results.
Until tolerances and adequate data exist, status is `not_calibrated`; no fitted coefficients or promotion claim.

## 8. Scientific support and limits

The cited techniques motivate measurement and evaluation. They do not validate
this project's particular rubric or aggregation rule.

- **S1:** [NIST SP 500-235, Structured Testing](https://www.nist.gov/publications/structured-testing-testing-methodology-using-cyclomatic-complexity-metric).
  Formal control-flow/testing measure, not a universal effort estimator.
- **S2:** [Clippy lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html#cognitive-complexity-threshold).
  Thresholded cognitive diagnostics cannot provide all cyclomatic values.
- **S3:** [Inozemtseva and Holmes, ICSE 2014](https://cs.uwaterloo.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf).
  In five Java systems, controlling suite size reduced the coverage/effectiveness
  association; this does not establish universal behavior for Rust or agent tests.
- **S4:** [COSMIC measurement process](https://cosmic-sizing.org/cosmic-sizing/intro/measurement-process/).
  Standardized functional sizing; a different construct from technical difficulty.
- **S5:** [CMU SEI, Software Cost Estimation Explained](https://www.sei.cmu.edu/blog/software-cost-estimation-explained/).
  Parametric estimation separates size and cost drivers. COCOMO's project-scale
  coefficients are not transplanted to agent patches.
- **S6:** [Shepperd and MacDonell, Evaluating prediction systems in software project estimation](https://openrepository.aut.ac.nz/items/673935d8-131d-449e-9e48-816730b33ad7).
  Baseline-relative evaluation and effect sizes; the protocol above is a local
  design applying these principles, not a verbatim standard.
- **S7:** [Cohen, Weighted kappa (1968)](https://pubmed.ncbi.nlm.nih.gov/19673146/).
  Agreement with scaled disagreement; diagnostic agreement is distinct from
  validating the complexity construct or predicting effort.

Sources consulted 2026-09-07. Mathematical validation and unresolved limitations:
[design audit](../audit/rri-v2-design-validation.md).
