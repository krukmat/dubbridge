---
type: Plan
title: "Plan: Review Decision Shadow Layer"
status: active
---
# Plan: Review Decision Shadow Layer

## Objective

Add a **decision-model shadow layer** to DubBridge review routing without changing the current reviewer authority or RRI bindings. The first candidate is `nimble:9b-q4_K_M` through Ollama `/v1/systemone`; Clef Flash remains an optional multimodal provider later.

The design deliberately separates work that can be completed and verified in a cloud/CI environment from work that genuinely requires the owner's local Apple Silicon/Ollama runtime.

## Non-goals

- Do not replace `gpt-oss:20b`, Gemma, cross-vendor review, D14, RRI, HITL, or owner approval.
- Do not grant a decision model authority to emit an official PASS/FAIL verdict.
- Do not alter `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` or ADR-046 during shadow evaluation.
- Do not require Ollama for unit tests, packet construction, normalization, corpus handling, or metrics.

## Architecture

```text
review packet + RRI + deterministic evidence
                  |
                  v
      review_decision.py (cloud-safe)
      - state contract
      - System One questions
      - fail-closed eligibility
                  |
             request.json
                  |
        LOCAL RUNTIME BOUNDARY
                  |
                  v
 systemone_local_adapter.py -> Ollama /v1/systemone
                  |
            raw-response.json
                  |
        CLOUD-SAFE AGAIN
                  |
                  v
 review_decision.py normalize
                  |
          normalized.json
                  |
                  +---------- current reviewer stays authoritative
                  |            gpt-oss / Gemma / peer / D14
                  |                        |
                  v                        v
           review_decision_shadow.py capture
                  |
             shadow.jsonl
                  |
                  v
           review_decision_shadow.py evaluate
```

## Design decisions

### D1 — System One is a provider seam, not a Nimble-specific contract

DubBridge owns a stable request/decision schema. The provider model name is data. The same request shape can therefore be evaluated with Nimble, Tev1, or other Jev/System One-compatible text decision models without changing review policy.

The initial binding is `nimble:9b-q4_K_M` because its memory footprint is suitable for the 32 GB local host and the review packets are predominantly text. Clef Flash is reserved for a later multimodal path where screenshots or visual evidence materially contribute.

### D2 — Shadow only until promotion evidence exists

The decision layer is advisory. Existing reviewer chains continue unchanged. A future fast-path requires a separate policy/ADR change after the evaluation gate succeeds.

### D3 — Fail closed by construction

A prospective future fast-path can only be marked eligible when all of these are true:

- RRI is Low (`0-25`);
- packet is complete and untruncated;
- tests and contracts are explicitly recorded as `pass`;
- security, migration, architecture and dependency-sensitive flags are explicitly `false`;
- the decision model selects `risk=low`, `evidence_complete=true`, `scope=expected`, `failure_domain=none`, and `suggested_review=none`;
- each selected decision clears the calibrated probability threshold.

Missing deterministic metadata is not interpreted as safe. It blocks eligibility.

### D4 — `confidence` is not correctness probability

System One `confidence` is retained for diagnostics only. Promotion thresholds use the selected-option probabilities and DubBridge calibration evidence, not the provider `confidence` field.

### D5 — Prospective shadow evidence is primary

Historical review outputs are useful ground truth, but many committed review artifacts do not preserve the exact original task/diff packet. Replaying a model against the review output would leak the verdict and invalidate the measurement. Therefore:

- historical artifacts may seed labels only when the original input packet can be reconstructed independently;
- the primary promotion dataset is **prospective shadow capture** against the exact packet used by the authoritative reviewer.

## Cloud-safe deliverables

The following components require no Ollama/model installation to implement or test:

- `scripts/review_decision.py` — state schema, System One request builder, response normalization, fail-closed fast-path evaluator;
- `scripts/systemone_local_adapter.py` — thin HTTP adapter whose transport is mockable in unit tests;
- `scripts/review_decision_shadow.py` — historical ground-truth extraction, paired shadow capture, and critical false-negative/escalation/agreement/latency metrics;
- unit tests for all of the above;
- `make qa-review-decision` target.

## Local-only work

Only these operations require the owner's local host:

1. verify Ollama `>=0.35` and pull `nimble:9b-q4_K_M`;
2. execute saved System One requests through `systemone_local_adapter.py`;
3. capture real M5 Pro RAM/latency and model-residency evidence;
4. accumulate at least 30-50 authoritative shadow comparisons with enough blocking/major cases to make the safety result meaningful;
5. optionally repeat the same dataset with Clef Flash where visual evidence exists.

No local source-code implementation is required for the initial adapter; local work is runtime validation and evidence collection.

## Promotion metrics

Minimum gate before considering any workflow-policy change:

- `critical_false_negatives = 0`;
- critical escalation recall `>= 0.95`;
- no malformed/truncated response accepted as eligible;
- measured latency/RAM acceptable on the M5 Pro 32 GB host;
- reviewer/owner agreement reported, but not used alone as a safety gate;
- sufficient sample size and critical-case representation documented explicitly.

Passing these metrics only authorizes a **policy review**. It does not automatically activate a fast-path.

## Future promotion scope

If shadow evidence passes, the first eligible production change is intentionally narrow: RRI Low only, after deterministic tests/contracts, with all sensitive-change flags false. Everything else keeps the existing reviewer route.

Clef Flash is a later optional provider for screenshot/UI evidence and must not increase the authority of the decision layer.


## Automatic capture seam (C6)

`peer-workflow-review.py` now queues the exact supplied review content before
authoritative reviewer routing. The queue operation does not invoke Nimble and is
wrapped so any shadow failure is warning-only. The authoritative RRI-resolved
reviewer chain remains unchanged.

Each pending item carries a canonical `request_sha256`, the future local response
must echo that hash, and pairing only occurs after both a normalized decision and
the authoritative review artifact exist. Runtime execution remains isolated behind
the explicit `review_decision_queue.py run-local` command.
