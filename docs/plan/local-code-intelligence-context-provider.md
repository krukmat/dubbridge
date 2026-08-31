# Local Code Intelligence Context Provider

## Objective

Introduce an orchestrator-owned context-selection seam for the local agent runner so model-visible source can be selected deterministically instead of always preloading the complete authorized working set.

The first milestone preserves current authorization, editing tools, acceptance gates, handoff schemas, and source authority. A local Code Knowledge Graph (CKG) is an implementation behind the context-provider seam, not a product/runtime subsystem.

## Affected files

- `scripts/local-agent/cli.py`
- `scripts/local-agent/session_loop.py`
- `scripts/local-agent/runner_file_tools.py`
- new `scripts/local-agent/context_provider.py`
- new `scripts/local-agent/code_intelligence.py`
- new `scripts/local-agent/ckg_context_manifest.py`
- associated tests

## Design decisions

### D1 — One `ContextProvider` seam

`session_loop.py` must not own backend-specific graph logic. It consumes a provider that renders initial and repair context.

Implementations:

- `LegacyContextProvider`: behavior-compatible complete authorized context.
- `CKGContextProvider`: graph-guided, authorized, ranked, budgeted context selection.

### D2 — `RunnerFileTools` remains filesystem tooling

`RunnerFileTools` owns checked reads/writes/patches. Context strategy is moved out of it. `preload_context()` remains temporarily for compatibility/fallback.

### D3 — Reuse existing authorization semantics

No second `allowed_paths` matcher. Candidate paths are checked using the existing boundary. Rejected graph dependencies become scope gaps and are never exposed to the model.

### D4 — Minimum Useful Graph v1

Nodes: File, Module/Crate, Function/Method, Struct/Enum, Trait, Test.

Edges: CONTAINS, IMPORTS/USES, CALLS, IMPLEMENTS, REFERENCES, TESTS.

Semantic search may resolve anchors, but is not authority.

### D5 — Deterministic retrieval v1

1. Resolve explicit paths/symbols/tests from task and acceptance text.
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
6. semantic-only candidate.

Depth 2 is opt-in.

### D6 — Delta-aware graph view

Use a reusable base graph for a repository revision. Dirty worktree files are source-authoritative. v1 does not require a fully reconciled overlay graph.

### D7 — Existing Capsule/Attempt Bundle remain unchanged

Retrieval evidence uses `ckg-context-manifest-v1`. The manifest records selection decisions, source hashes, coverage, scope gaps, and token budget, but not source bodies.

### D8 — Full invocation budget

The active runtime `num_ctx`/`num_predict` values must drive prompt and retrieval budgeting. Import-time 32K assumptions must not validate invocations running a smaller profile.

### D9 — Explicit heavy-model lifecycle

Normal completion must unload the local implementer rather than relying on a long keep-alive period after the runner exits.

### D10 — Source-authoritative fallback

- healthy graph + coverage -> selective CKG context;
- partial coverage -> graph evidence plus targeted source fallback;
- unavailable graph -> legacy source workflow;
- no silent truncation of task constraints.

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

The first adapter targets `codebase-memory-mcp` via one-shot CLI calls. The rest of the runner depends only on the backend-neutral adapter contract.

## Rollback

`LegacyContextProvider` remains available. CKG errors, stale/partial coverage, or an unsafe packet fall back without widening authority or changing editing semantics.
