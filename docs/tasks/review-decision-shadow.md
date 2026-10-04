---
type: TaskList
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

**Status:** DONE — local runtime/schema smoke PASS 2026-10-04  
**Dependency:** C1  
**Effort:** S

- [x] Add standard-library HTTP adapter for `/v1/systemone`.
- [x] Make endpoint and timeout configurable.
- [x] Capture raw response and measured request latency.
- [x] Test transport using a mocked opener; no Ollama dependency in CI/cloud.
- [x] **LOCAL:** verified with Ollama `0.35.0` and `nimble:9b-q4_K_M` on 2026-10-04.

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


## C6 — Automatic shadow capture + batch queue — CLOUD

**Status:** DONE  
**Dependency:** C1–C5  
**Effort:** M

- [x] Hook `peer-workflow-review.py` before authoritative model routing and persist one provider-neutral request + manifest under `.agent/review-decision/pending/`.
- [x] Keep capture default-on but explicitly non-authoritative; any capture failure degrades to a warning and preserves reviewer route/verdict/exit code.
- [x] Bind each local response envelope to the exact request with `request_sha256`.
- [x] Add ingestion + pairing against the authoritative review artifact.
- [x] Add an explicit local batch command; importing/testing the queue requires no Ollama/model.
- [x] Add `--no-shadow-capture`, `--shadow-root`, and optional `--shadow-metadata`.
- [x] Extend `make qa-review-decision` with queue + integration tests.
- [x] Run the cloud-safe suite from the existing `peer-workflow-review` CI entry point.

**HP-1:** normal peer review → request/manifest queued → normal reviewer executes unchanged.  
**EC-1:** queue write/schema failure → warning only → normal reviewer still returns its original verdict/exit code.

**Evidence to emit:** `scripts/review_decision_queue_test.py`, `scripts/peer_review_shadow_capture_test.py`.  
**Status artifacts affected:** this ledger, plan, and shadow runbook.


## C7 — One-command local handoff packaging — CLOUD

**Status:** DONE  
**Dependency:** C6  
**Effort:** S

- [x] Add a cloud-testable preflight that probes local Ollama connectivity, pending queue size, and required model presence without pulling or mutating anything.
- [x] Add an explicit `run` command that only starts System One batch execution after preflight is ready.
- [x] Persist a local-run receipt under the ignored shadow directory.
- [x] Expose `make review-decision-local-check` and `make review-decision-local-run`.
- [x] Add the handoff tests to the cloud-safe review CI gate.

**HP-1:** pending cases + available Nimble model → preflight ready → explicit run consumes the pending batch.  
**EC-1:** model absent → preflight blocks, reports the exact pull command, and never starts the batch.

**Evidence to emit:** `scripts/review_decision_local_handoff_test.py`.  
**Status artifacts affected:** this ledger and shadow runbook.

## C8 — Canonical decision-model lifecycle — CLOUD

**Status:** DONE  
**Dependency:** C6-C7  
**Effort:** S

- [x] Define L2 shadow-only authority in the canonical workflow guide.
- [x] Define L3 evidence thresholds and non-authoritative promotion recommendation.
- [x] Require explicit owner approval + ADR/policy amendment before any L4 fast-path.
- [x] Bound the first possible promotion to fail-closed Low RRI 0–25 only.
- [x] Preserve Moderate+ routing, deterministic gates, and owner/HAA authority.
- [x] Correct the stale peer-review enforcement note now that executable enforcement exists.

**HP-1:** agents can explain and operate L2 without treating Nimble as a reviewer.  
**EC-1:** any attempt to skip a required review before L4 is rejected by the canonical policy.

**Evidence to emit:** canonical lifecycle section in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.  
**Status artifacts affected:** this ledger and the workflow guide.

## L1 — Local Nimble runtime validation — LOCAL ONLY

**Status:** PASS — runtime/schema validated 2026-10-04  
**Dependency:** C1-C7  
**Effort:** S

- [x] Run `make review-decision-local-check` — owner evidence: Ollama 0.35.0 reachable.
- [x] Confirm `nimble:9b-q4_K_M` is present locally.
- [x] Run `make review-decision-local-smoke` — PASS: transport/schema valid, 5/5 typed answers, latency 6363.238 ms, metrics excluded.
- [x] Record runtime evidence in `docs/evaluations/review-decision-local-smoke-2026-10-04.json`.
- [ ] Memory/residency characterization is deferred to L2 batch runs where warm/cold behavior can be measured meaningfully.

**HP-1:** valid request -> typed decision response with all five answers.  
**EC-1:** endpoint/model unavailable -> blocked local validation; no workflow change.

## L2 — Prospective shadow sample — LOCAL MODEL + EXISTING REVIEWERS

**Status:** READY  
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
