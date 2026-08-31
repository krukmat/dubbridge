# Local Code Intelligence Context Provider

## Status

Milestone 1 implementation: **complete on `feat/ckg-context-provider`**.

Remote completion requires the final branch head to pass the dedicated context-provider workflow. Real CBM/Ollama/model execution on the target Mac is intentionally operator-local validation and is not claimed by this branch.

No merge to `main` is part of this milestone.

## Objective

Introduce an orchestrator-owned context-selection seam for the local agent runner so model-visible source can be selected deterministically instead of always preloading the complete authorized working set.

The first milestone preserves current authorization, editing tools, acceptance gates, handoff schemas, and source authority. A local Code Knowledge Graph (CKG) is an implementation behind the context-provider seam, not a product/runtime subsystem.

## Affected files

- `scripts/local-agent/cli.py`
- `scripts/local-agent/session_loop.py`
- `scripts/local-agent/runner_file_tools.py`
- `scripts/local-agent/context_provider.py`
- `scripts/local-agent/context_budget.py`
- `scripts/local-agent/ckg_adapter.py`
- `scripts/local-agent/ckg_manifest.py`
- `scripts/local-agent/ollama_lifecycle.py`
- associated tests/workflow/schema

## Design decisions

### D1 — One `ContextProvider` seam

`session_loop.py` does not own backend-specific graph logic. It consumes a provider that renders initial and repair context.

Implementations:

- `LegacyContextProvider`: behavior-compatible complete authorized context.
- `CKGContextProvider`: graph-guided, authorized, ranked, budgeted context selection.

### D2 — `RunnerFileTools` remains filesystem tooling

`RunnerFileTools` owns checked reads/writes/patches. Context strategy stays outside it. `preload_context()` remains for compatibility/fallback.

### D3 — Reuse existing authorization semantics

No second `allowed_paths` matcher. Candidate paths use the existing boundary. Rejected graph dependencies become scope gaps and are never exposed to the model.

### D4 — Minimum Useful Graph v1

Nodes: File, Module/Crate, Function/Method, Struct/Enum, Trait, Test.

Edges: CONTAINS, IMPORTS/USES, CALLS, IMPLEMENTS, REFERENCES, TESTS.

Text/semantic-style search may resolve anchors, but it is not authority.

### D5 — Deterministic retrieval v1

1. Resolve explicit paths/symbols/tests from task specification and acceptance text.
2. Traverse direct relationships at depth 1 by default.
3. Intersect with authorization.
4. Rank candidates.
5. Fit selected context into a retrieval token budget.
6. Read current source from the worktree immediately before rendering.

Ranking order:

1. explicit task path/symbol;
2. HP/EC or acceptance test;
3. direct dependency/callee;
4. direct caller;
5. signature/type context;
6. text-only candidate.

Depth 2 is not part of milestone 1.

### D6 — Delta-aware graph view

Index the exact disposable task worktree. Dirty worktree files remain source-authoritative. The incremental index is refreshed before repair-context retrieval so structural edits are reflected without introducing a separate overlay-graph subsystem.

### D7 — Existing Capsule/Attempt Bundle remain unchanged

Retrieval evidence uses `ckg-context-manifest-v1`. The manifest records selection decisions, source hashes, graph snapshot/worktree identity, coverage, scope gaps, and token budget, but not source bodies.

### D8 — Full invocation budget

The active runtime `num_ctx`/`num_predict` values drive prompt and retrieval budgeting. Import-time 32K assumptions do not validate invocations running a smaller profile.

### D9 — Explicit heavy-model lifecycle

Normal live-runner completion requests model unload rather than relying on the transport keep-alive period after the runner exits.

### D10 — Source-authoritative fallback

- healthy graph + coverage -> selective CKG context;
- partial/stale coverage -> legacy source fallback with manifest evidence;
- unavailable graph -> legacy source workflow;
- task/acceptance constraints are never silently truncated by the CKG selector.

## Non-goals for milestone 1

- reviewer graph context;
- cloud handoff changes;
- interactive `request_context`;
- governance/ADR GraphRAG;
- RRI changes;
- Neo4j;
- hosted graph or external embedding service;
- history/diagnostic compaction;
- reducing context windows.

## Backend

The first adapter targets `codebase-memory-mcp` through bounded one-shot CLI calls using JSON on stdin. The adapter unwraps the MCP JSON response envelope fail-closed. The rest of the runner depends only on the provider/adapter contract.

## Rollback

`LegacyContextProvider` remains available. CKG errors, stale/partial coverage, or an unsafe/empty selected packet fall back without widening authority or changing editing semantics.

## Validation boundary

Remote evidence covers deterministic selection, authorization, budgeting, manifest/freshness, adapter transport contracts, fallback, prompt regression, and current local-agent integration tests.

The remaining operator-local smoke should exercise:
- the real `codebase-memory-mcp` installation and project indexing behavior;
- a real Ollama implementer invocation;
- manifest output from an actual DubBridge task;
- unload/residency behavior on the target 32 GB Apple Silicon host.
