---
type: Plan
title: "Local Code Intelligence Boundary"
status: implemented_pending_local_verification
branch: feature/local-code-intelligence-boundary
---

# Local Code Intelligence Boundary

## Objective

Introduce a local, model-agnostic code-intelligence boundary for agent workflows. The layer resolves task-relevant code structure locally, records what was selected, and exports only bounded context to cloud agents.

This is development tooling. It is not a DubBridge product/runtime component.

## Architectural shape

```text
DubBridge repository
       |
       v
Local CKG backend
       |
       v
Backend-neutral adapter
       |
       v
Context Gateway
       |----------------------|
       v                      v
Context Receipt         Context Capsule
(audit/provenance)      (bounded agent input)
       |                      |
       +----------+-----------+
                  v
          existing agent workflow
            |             |
            v             v
        local agents   cloud agents
```

## Design decisions

1. The CKG backend is replaceable. No agent-facing contract depends on a particular vendor or model.
2. Source, tests, ADRs, and repository policy remain authoritative; graph output is advisory context-selection evidence.
3. Local agents may consume richer structural context. Cloud agents receive a bounded capsule and never unrestricted graph traversal.
4. Context export is deny-by-default for explicitly sensitive/global classes and allow-by-selection for task-local evidence.
5. Every generated capsule is accompanied by a receipt bound to repository revision, graph revision, selected anchors, impact evidence, exclusions, and expansion history.
6. Context expansion is explicit and bounded. The cloud side requests additional context by reason; local tooling decides what to export.
7. RRI remains unchanged. Graph-derived impact may later be used as advisory evidence, but this slice does not alter routing thresholds or model bindings.
8. Heavy local model execution and graph indexing must remain sequential on memory-constrained hosts; the code-intelligence layer itself must not require a resident LLM.

## Implemented surface

- `scripts/code_intelligence/backend.py`
- `scripts/code_intelligence/context_gateway.py`
- `scripts/code_intelligence/context_gateway_test.py`
- `scripts/code_intelligence/README.md`
- `docs/schemas/context-receipt-v1.schema.json`
- `docs/tasks/local-code-intelligence-boundary.md`
- this plan

## Module dependencies

- `backend.py` defines a narrow backend-neutral graph result contract and JSON fixture backend.
- `context_gateway.py` consumes the backend contract and produces receipt/capsule JSON.
- `context_gateway_test.py` certifies minimum-disclosure and fail-closed behavior.
- external CKG adapters can be added later without changing receipt/capsule semantics.

## RRI estimate

Pre-implementation estimate: **Moderate (~33)**.

Rationale: a small Python tooling surface across several files, no production runtime or protected product boundary touched, but non-trivial context-selection and export policy behavior. The user's explicit instruction to implement this branch is treated as approval for this scoped plan. No RRI formula or routing policy changes are included.

## Verification state

Implementation is complete for the scoped boundary. Local execution remains required before merge because the current orchestration container cannot resolve GitHub for a branch checkout and GitHub reported no CI run/status for the branch head.

See `docs/tasks/local-code-intelligence-boundary.md` for the exact commands and behavioral test mapping.

## Non-goals

- Neo4j or an enterprise knowledge graph.
- A generic documentary GraphRAG service.
- A new hosted dependency.
- Direct Claude/Codex access to the complete graph.
- Replacing grep/source reads/tests/review.
- Modifying RRI or local-model role bindings.
- Running an LLM for every graph query.
- Benchmark infrastructure or a formal measurable POC.
