---
type: Evaluation
status: ready-for-local-evidence
---
# Review decision shadow runbook

This runbook keeps the authoritative DubBridge reviewer chain unchanged. Nimble is measured beside it.

## 1. Build a request — no local model required

```bash
python3 scripts/review_decision.py packet \
  --content /path/to/exact-review-packet \
  --phase code \
  --rri 18 \
  --task-id EXAMPLE-T1 \
  --metadata /path/to/decision-metadata.json \
  --output /tmp/decision-request.json
```

Example deterministic metadata:

```json
{
  "checks": {"tests": "pass", "contracts": "pass"},
  "deterministic": {
    "security_sensitive": false,
    "migration_change": false,
    "architecture_change": false,
    "dependency_sensitive": false
  }
}
```

Unknown or omitted values intentionally prevent future fast-path eligibility.

## 2. Local-only invocation

```bash
python3 scripts/systemone_local_adapter.py \
  /tmp/decision-request.json \
  --output /tmp/decision-response.json
```

The default endpoint is `http://localhost:11434/v1/systemone`. This is the only step that needs the local Ollama runtime/model.

## 3. Normalize — no local model required

```bash
python3 scripts/review_decision.py normalize \
  --request /tmp/decision-request.json \
  --response /tmp/decision-response.json \
  --output /tmp/decision-normalized.json
```

`fast_path.eligible` is shadow evidence only. It has no authority over the current reviewer chain.

## 4. Run the authoritative reviewer normally

Use the existing RRI-resolved `peer-workflow-review.py` route with no decision-model shortcut.

## 5. Capture the paired case

```bash
python3 scripts/review_decision_shadow.py capture \
  --request /tmp/decision-request.json \
  --normalized /tmp/decision-normalized.json \
  --review-artifact /path/to/authoritative-review.json \
  --output docs/evaluations/review-decision-shadow.jsonl
```

Use `--expected-review` only when the intended target review depth is independently known.

## 6. Evaluate

```bash
python3 scripts/review_decision_shadow.py evaluate \
  docs/evaluations/review-decision-shadow.jsonl \
  --output /tmp/review-decision-metrics.json
```

The promotion gate is not considered until the dataset contains real critical cases. Required safety targets are zero critical false negatives and at least 0.95 critical escalation recall.

## Historical artifacts

`scripts/review_decision_shadow.py ground-truth` can index existing review outcomes:

```bash
python3 scripts/review_decision_shadow.py ground-truth \
  --root docs/audit \
  --output /tmp/review-ground-truth.jsonl \
  --limit 50
```

This output is **ground truth only** unless the exact original review input can be reconstructed without reading the review verdict/findings. Feeding a completed review artifact to the decision model would leak the answer and invalidate the benchmark.


## Automatic C6 capture

Normal `scripts/peer-workflow-review.py` executions now create ignored local
artifacts under `.agent/review-decision/pending/` by default. They do **not**
invoke Nimble and cannot change the authoritative review result.

Inspect pending work:

```bash
python3 scripts/review_decision_queue.py pending
```

The only local-model step is explicit:

```bash
python3 scripts/review_decision_queue.py run-local
```

That command invokes the configured System One endpoint, validates the
`request_sha256`, normalizes the response and pairs it automatically when the
authoritative review artifact is already present. A response for another request
is rejected rather than guessed or re-bound.

For a manually produced response envelope:

```bash
python3 scripts/review_decision_queue.py ingest \
  --manifest .agent/review-decision/pending/<case>.manifest.json \
  --response /path/to/response.json

python3 scripts/review_decision_queue.py pair \
  --manifest .agent/review-decision/pending/<case>.manifest.json
```

Use `--no-shadow-capture` only when a review explicitly must not leave shadow
artifacts. Use `--shadow-metadata <json>` when deterministic tests/contracts and
sensitivity flags are known; absent metadata fails closed for fast-path eligibility.


## Minimal local handoff

The cloud side packages the local work into two commands:

```bash
make review-decision-local-check
```

This is non-mutating. It reports Ollama reachability, the detected version,
pending-case count, whether the required Nimble model is already installed, and
an exact remediation command when the model is absent.

Only after the check reports `ready`:

```bash
make review-decision-local-run
```

The run command processes the captured queue through `/v1/systemone`, validates
request hashes, normalizes decisions, pairs any completed authoritative reviews,
and writes `.agent/review-decision/local-run-receipt.json`. It does not change
reviewer bindings or workflow authority.


## Isolated local smoke

Before waiting for real shadow cases, validate the live local System One path:

```bash
make review-decision-local-smoke
```

The smoke uses a synthetic Low-RRI review state, calls Nimble with
`keep_alive=0`, validates the five typed answers and request hash, records
latency, and writes:

```text
.agent/review-decision/local-smoke-receipt.json
```

This receipt is deliberately excluded from `shadow.jsonl` and all promotion
metrics. Its only purpose is L1 transport/runtime/schema validation.


### 2026-10-04 local smoke result

Owner-executed local evidence:

- Ollama: `0.35.0`
- model: `nimble:9b-q4_K_M`
- transport/schema: PASS
- typed answers: 5/5
- latency: `6363.238 ms`
- semantic observation: `risk=low`, `evidence_complete=true`,
  `scope=expected`, `failure_domain=none`, `suggested_review=none`
- `keep_alive=0`
- excluded from promotion metrics by design

This closes L1 transport/runtime/schema validation. Real task/code observations,
warm-vs-cold latency, memory residency, and reviewer agreement belong to L2.
