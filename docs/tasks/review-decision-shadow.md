---
type: Tasks
title: "Tasks: Review Decision Shadow Layer"
status: active
---
# Tasks: Review Decision Shadow Layer

Owner authorization: current-session directive to work directly on `main`, maximizing cloud-executable work and isolating local dependencies.

## C1 — Freeze the decision contract — CLOUD

**Status:** DONE  
**Dependency:** none  
**Effort:** S

- [x] Define one provider-neutral System One request schema.
- [x] Define `risk`, `evidence_complete`, `scope`, `failure_domain`, and `suggested_review` questions.
- [x] Add deterministic case ID/content hash.
- [x] Bound the state payload and make truncation explicit.
- [x] Keep all behavior advisory/shadow-only.

**HP-1:** complete Low-RRI packet + explicit safe deterministic metadata -> valid request artifact.  
**EC-1:** missing deterministic evidence -> future fast-path evaluates false, not safe-by-default.

**Evidence to emit:** `scripts/review_decision_test.py`.  
**Status artifacts affected:** this task ledger and `docs/plan/review-decision-shadow.md`.

## C2 — Build provider runtime seam — CLOUD implementation / LOCAL execution

**Status:** DONE (implementation); LOCAL VALIDATION PENDING  
**Dependency:** C1  
**Effort:** S

- [x] Add standard-library HTTP adapter for `/v1/systemone`.
- [x] Make endpoint and timeout configurable.
- [x] Capture raw response and measured request latency.
- [x] Test transport using a mocked opener; no Ollama dependency in CI/cloud.
- [ ] **LOCAL:** verify against Ollama `>=0.35` and `nimble:9b-q4_K_M`.

**HP-1:** valid saved request -> local endpoint response persisted with latency.  
**EC-1:** unavailable endpoint or invalid JSON -> explicit failure; no synthetic decision.

**Evidence to emit:** mocked adapter coverage in `scripts/review_decision_test.py`, later local live transcript.  
**Status artifacts affected:** local evidence report when executed.

## C3 — Normalize decisions and enforce fail-closed prospective fast-path — CLOUD

**Status:** DONE  
**Dependency:** C1  
**Effort:** S

- [x] Validate all required answer fields/options/probabilities.
- [x] Preserve provider `confidence` only as diagnostic metadata.
- [x] Require Low RRI, full packet, passed deterministic gates and explicit non-sensitive flags.
- [x] Require calibrated selected-option probability threshold.
- [x] Never let malformed/missing values become eligible.

**HP-1:** fully safe Low packet + strong matching model decisions -> shadow `eligible=true`.  
**EC-1:** RRI 26+, truncated packet, unknown checks, sensitive flag, or low probability -> `eligible=false` with reasons.

**Evidence to emit:** `scripts/review_decision_test.py`.

## C4 — Build shadow ground-truth/capture/evaluation tooling — CLOUD

**Status:** DONE  
**Dependency:** C1, C3  
**Effort:** S

- [x] Normalize historical reviewer artifacts across top-level and nested GPT-OSS schemas.
- [x] Mark blocking/major as critical ground truth.
- [x] Join request hash/case ID, normalized decision and authoritative reviewer result.
- [x] Compute critical false negatives, critical escalation recall, optional expected-review agreement, fast-path candidate rate and p50/p95 latency.
- [x] Make promotion readiness fail when no critical evidence exists.
- [x] Document that historical outputs without original packets are labels only, not valid replay inputs.

**HP-1:** blocking reviewer finding + advanced/high decision -> counted as safely escalated.  
**EC-1:** blocking/major reviewer finding + shadow fast-path candidate -> counted as critical false negative.

**Evidence to emit:** `scripts/review_decision_test.py`.

## C5 — Cloud verification entry point — CLOUD

**Status:** DONE  
**Dependency:** C1-C4  
**Effort:** S

- [x] Add `make qa-review-decision` running the complete no-Ollama test suite.
- [x] Keep existing reviewer bindings and authoritative workflow untouched.

**HP-1:** clean Python environment with repository checkout -> all decision-layer tests run without Ollama.  
**EC-1:** no model/runtime installed -> QA still remains runnable.

## L1 — Local Nimble runtime validation — LOCAL ONLY

**Status:** PENDING  
**Dependency:** C1-C5  
**Effort:** S

- [ ] Confirm Ollama `>=0.35`.
- [ ] Pull `nimble:9b-q4_K_M`.
- [ ] Execute at least one task-phase and one code-phase saved request.
- [ ] Record model tag/digest if available, latency, memory pressure/residency and response validity.
- [ ] Confirm `keep_alive=0` releases the model as intended for the current 32 GB stack.

**HP-1:** valid request -> typed decision response with all five answers.  
**EC-1:** endpoint/model unavailable -> blocked local validation; no workflow change.

## L2 — Prospective shadow sample — LOCAL MODEL + EXISTING REVIEWERS

**Status:** PENDING  
**Dependency:** L1  
**Effort:** M

- [ ] Capture the exact request packet before the authoritative review.
- [ ] Run Nimble without changing the reviewer route.
- [ ] Normalize/capture the model result and final reviewer outcome.
- [ ] Accumulate 30-50 cases, including enough blocking/major findings for a meaningful safety measurement.
- [ ] Hand-label `expected_review` only where owner/reviewer intent is unambiguous.

**HP-1:** normal authoritative review produces a paired shadow row.  
**EC-1:** local decision unavailable -> authoritative review continues normally and the missing shadow sample is recorded, not substituted.

## L3 — Promotion report — CLOUD once L2 artifacts exist

**Status:** BLOCKED by L2  
**Dependency:** L2  
**Effort:** S

- [ ] Run `review_decision_shadow.py evaluate` over the captured dataset.
- [ ] Require critical false negatives = 0.
- [ ] Require critical escalation recall >= 0.95.
- [ ] Report agreement, candidate rate, p50/p95 latency, RAM observations and sample composition.
- [ ] Decide `REJECT`, `CONTINUE_SHADOW`, or `READY_FOR_POLICY_REVIEW`.

## L4 — Policy promotion / Low-only fast-path — NOT AUTHORIZED

**Status:** BLOCKED by L3 + explicit owner approval  
**Dependency:** L3  
**Effort:** M

If and only if L3 is `READY_FOR_POLICY_REVIEW`, prepare a separate ADR/policy amendment. No code in C1-C5 changes current reviewer authority.

## V1 — Clef Flash visual comparison — OPTIONAL LATER

**Status:** BACKLOG  
**Dependency:** stable shadow harness  
**Effort:** S/M

Reuse the same provider seam for cases where screenshots or visual evidence materially affect review. Do not add Clef Flash to ordinary text-only cases by default.
