# Local Code Intelligence Boundary

This directory contains the model-agnostic boundary between a local code-knowledge-graph result and DubBridge agents.

## Milestone state

M3 is closed for this branch. The operational Analyze/handoff entry point is:

`scripts/code_intelligence/context_gateway.py`

M3 deliberately closes at this CLI boundary; it does **not** require a hard-coded hook inside `run_local_task.py` or any model-specific runner. The current stable backend interchange is `JsonGraphBackend` in `backend.py`, consuming local JSON graph results. A concrete CKG producer may remain external/local and replaceable without changing the gateway contract.

M4 hardens and operationalizes that M3 path. It does not reopen model routing, RRI, or the Analyze architecture.

## Purpose

The tooling moves deterministic repository discovery out of the LLM where possible:

1. a local CKG/backend resolves task-relevant anchors, relationships, files, tests, boundaries, governance, and optional source fragments;
2. the backend result is normalized through `backend.py`;
3. `context_gateway.py` verifies repository freshness and applies target-specific export policy;
4. the gateway writes a `context-receipt.json` plus a bounded `context-capsule.json`;
5. an existing local or cloud agent consumes the capsule;
6. when more context is required, a new local graph result can be evaluated as a bounded expansion linked to the prior receipt.

The source tree, tests, ADRs, and repository policies remain authoritative. Graph output is context-selection evidence, not proof of correctness.

## Trust boundary

`local` and `cloud` are different export targets.

- `secret` and `runtime_data` fragments/relationships are denied for every target.
- deterministic gateway deny rules also block clearly unsafe paths/content such as `.env*`, private-key material, temporary/runtime roots, and selected credential markers even when a backend mislabels them as `task_local`;
- `cross_boundary` and `global_architecture` data are additionally denied for `cloud`;
- cloud metadata is minimized from allowed task-local evidence rather than copied wholesale from backend arrays;
- cloud agents must not receive unrestricted MCP/graph traversal as a substitute for the gateway;
- a local agent may receive richer structural relationships and metadata, but still receives no explicit secret/runtime-data records.

The policy is intentionally small and deterministic. Do not turn it into a generic policy DSL without evidence from real tasks that more complexity is necessary.

## Freshness invariant

The operational path requires the orchestrator/Analyze phase to provide the expected Git revision:

`--expected-git-revision <sha>`

The gateway compares that value with the backend result's `git_revision` before publishing artifacts. A mismatch fails closed and identifies expected versus received revision.

`graph_revision` remains backend provenance for the initial request and becomes an enforced invariant for bounded expansion: an expansion must use the same Git and graph revision as its base receipt.

## Backend interchange

An external local CKG producer should emit a payload shaped like:

```json
{
  "git_revision": "<git-sha>",
  "graph_revision": "<backend-revision>",
  "anchors": ["crate::module::symbol"],
  "files": ["crates/module/src/lib.rs"],
  "symbols": ["crate::module::symbol"],
  "relationships": [
    {
      "from": "crate::module::symbol",
      "to": "crate::other::call",
      "kind": "calls",
      "classification": "task_local"
    }
  ],
  "tests": ["tests::symbol_happy_path"],
  "boundaries": ["storage"],
  "governance": ["ADR-006"],
  "source_fragments": [
    {
      "path": "crates/module/src/lib.rs",
      "start_line": 10,
      "end_line": 20,
      "content": "...",
      "classification": "task_local"
    }
  ]
}
```

Supported classifications:

- `task_local`
- `cross_boundary`
- `global_architecture`
- `secret`
- `runtime_data`

The executable core remains independent of Nemotron, Qwen, Gemma, Claude, Codex, Ollama, or a specific CKG vendor.

## Initial Analyze/handoff CLI

Example from the repository root:

```bash
HEAD_SHA="$(git rev-parse HEAD)"
python3 scripts/code_intelligence/context_gateway.py \
  --task-id S-XYZ-T3 \
  --task "Change a bounded playback behavior" \
  --backend-json /tmp/dubbridge-graph-result.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --output-dir /tmp/dubbridge-context
```

Outputs:

- `/tmp/dubbridge-context/context-receipt.json`
- `/tmp/dubbridge-context/context-capsule.json`

Use the gateway during the **Analyze / handoff** phase. Do not insert it into DubBridge product runtime.

## Bounded expansion

A cloud agent that lacks context does not receive a graph query interface. The local orchestrator resolves additional context into another backend JSON result and evaluates it through the same gateway:

```bash
python3 scripts/code_intelligence/context_gateway.py \
  --task-id S-XYZ-T3 \
  --task "Change a bounded playback behavior" \
  --backend-json /tmp/dubbridge-expanded-graph-result.json \
  --target cloud \
  --expected-git-revision "$HEAD_SHA" \
  --base-receipt /tmp/dubbridge-context/context-receipt.json \
  --expansion-reason "Need the adjacent task-local helper used by the selected symbol" \
  --output-dir /tmp/dubbridge-context-expanded
```

Expansion rules:

- the base receipt hash must verify;
- target, Git revision, and graph revision must match the base receipt;
- the same minimum-disclosure policy is applied again;
- the new receipt records the reason, base receipt hash, decision (`allow`, `reduce`, or `deny`), and added context;
- denied content is never exposed merely because the request is an expansion.

## Tests

```bash
python3 -m py_compile \
  scripts/code_intelligence/backend.py \
  scripts/code_intelligence/context_gateway.py \
  scripts/code_intelligence/context_gateway_test.py

python3 scripts/code_intelligence/context_gateway_test.py
```

The suite covers the original boundary behaviors plus M4 freshness, metadata minimization, defense-in-depth, and bounded expansion cases.

## Audit entry point

The original M1-M3 audit entry point remains:

`docs/audit/local-code-intelligence-boundary-audit.md`

M4 is tracked separately in:

- `docs/plan/local-code-intelligence-m4-operational-adoption.md`
- `docs/tasks/local-code-intelligence-m4-operational-adoption.md`
- `docs/audit/local-code-intelligence-m4-pre-t4-audit.md` — executable S0-S10 procedure
- `docs/audit/local-code-intelligence-m4-pre-t4-audit-result.md` — latest recorded result

### Latest smoke result

The complete M4 pre-T4 smoke sequence was executed from
`feature/local-code-intelligence-boundary` on 2026-09-05 at
`19f5093901ed2b115a59af16c169b7e2a6613a88`.

| Coverage | Result |
|---|---|
| S0-S10 mandatory smokes | PASS |
| Python behavioral suite | 15/15 PASS |
| Stale Git and graph rejection | PASS, fail-closed with no success artifacts |
| Cloud minimization and unsafe-content denial | PASS |
| Determinism and receipt/capsule integrity | PASS |
| Bounded expansion | PASS (`reduce` adds the justified helper; forbidden expansion is denied) |
| `make qa-docs` | PASS |
| Supplementary script QA | 24/24 task-coverage tests and 6/6 roadmap-drift tests PASS |

Verdict: `READY_FOR_T4`. Consult the recorded audit result for the exact HEAD,
merge base, hashes, expected rejection codes, findings, and R0-R12 gate
summary. This authorizes the next ordinary task to exercise M4-T4; it does not
claim that operational adoption has already occurred.

The earlier exploratory findings around stale graph handling and metadata disclosure are now promoted into M4 enforced behavior. Pair-level artifact atomicity remains conditional hardening: do not add transaction machinery unless normal operational use exposes a concrete consumer race/crash-recovery issue.

## Resource model

The CKG is a preprocessing/orchestration layer. Prefer:

```text
index/query -> receipt/capsule -> graph backend idle -> one heavy local model -> verification
```

Do not make any specific local model a dependency of this subsystem. Existing DubBridge routing remains authoritative and can change independently.
