# Local Code Intelligence Boundary

This directory contains the model-agnostic boundary between a local code-knowledge-graph backend and DubBridge agents.

## Purpose

The tooling moves deterministic repository discovery out of the LLM where possible:

1. a local CKG/backend resolves task-relevant anchors, relationships, files, tests, boundaries, governance, and optional source fragments;
2. the backend result is normalized through `backend.py`;
3. `context_gateway.py` applies target-specific export policy;
4. the gateway writes a `context-receipt.json` plus a bounded `context-capsule.json`;
5. an existing local or cloud agent consumes the capsule.

The source tree, tests, ADRs, and repository policies remain authoritative. Graph output is context-selection evidence, not proof of correctness.

## Trust boundary

`local` and `cloud` are different export targets.

- `secret` and `runtime_data` fragments/relationships are denied for every target.
- `cross_boundary` and `global_architecture` data are additionally denied for `cloud`.
- cloud agents must not receive unrestricted MCP/graph traversal as a substitute for the gateway.
- a local agent may receive richer structural relationships, but still receives no explicit secret/runtime-data records.

The current classification policy is intentionally small. Do not turn it into a generic policy DSL without evidence from real tasks that more complexity is necessary.

## Backend integration

The first stable boundary is JSON, not a vendor SDK. An external local CKG adapter should emit a payload shaped like:

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

A later adapter can wrap `codebase-memory-mcp`, CodeGraph, or another local CKG without changing the gateway contract. Backend selection must have a separate supply-chain/network/telemetry review before being pinned into the workflow.

## CLI

Example from the repository root:

```bash
python3 scripts/code_intelligence/context_gateway.py \
  --task-id S-XYZ-T3 \
  --task "Change a bounded playback behavior" \
  --backend-json /tmp/dubbridge-graph-result.json \
  --target cloud \
  --output-dir /tmp/dubbridge-context
```

Outputs:

- `/tmp/dubbridge-context/context-receipt.json`
- `/tmp/dubbridge-context/context-capsule.json`

Use the gateway during the **Analyze** / handoff phase. Do not insert it into DubBridge product runtime.

## Tests

```bash
python3 scripts/code_intelligence/context_gateway_test.py
```

The tests certify:

- HP-1: valid backend payload normalization;
- EC-1: incomplete revision metadata fails closed;
- HP-2: deterministic receipt/capsule generation;
- EC-2: secret/runtime data never exported;
- EC-3: cross-boundary/global architecture not exported to cloud;
- EC-4: invalid CLI input leaves no successful output artifacts.

## Audit entry point

For local branch review, start at:

`docs/audit/local-code-intelligence-boundary-audit.md`

That document defines the required read order, branch-scope check, executable smoke suite (`S0`–`S8`), exploratory trust-boundary probes (`P1`–`P4`), evidence format, and merge interpretation.

The reusable black-box fixture is:

`scripts/code_intelligence/fixtures/audit-smoke-graph.json`

The exploratory probes are intentional: they let an auditor evaluate current assumptions around graph freshness, backend classification trust, metadata disclosure, and pair-level artifact atomicity without pretending those assumptions are already enforced guarantees.

## Resource model

The CKG is a preprocessing/orchestration layer. Prefer:

```text
index/query -> receipt/capsule -> graph backend idle -> one heavy local model -> verification
```

Do not make any specific local model a dependency of this subsystem. Existing DubBridge routing remains authoritative and can change independently.
