---
type: Plan
title: "Local Code Intelligence — M3 Reviewer Intelligence"
status: implemented-audit-pending
description: "Plan for enriching local code-review packets with bounded, local-only CKG impact context."
---

# Local Code Intelligence — M3 Reviewer Intelligence

## Status

Implementation is complete on `feat/ckg-reviewer-intelligence`; independent/local runtime audit remains pending before merge.

Deterministic code baseline: `086518312bb5b47adb97774473d5a6ae8b838f65`.

Dedicated `local-agent-context` run **82** passed the M1/M2 regressions plus M3 review-context, Makefile-wiring, and Gemma-reviewer tests at that code baseline.

## Objective

Improve the existing local review gate by adding deterministic impact context derived from the M1 CKG without changing reviewer routing, RRI policy, review verdict semantics, or cloud disclosure behavior.

The reviewer receives the authoritative diff and supplied task/acceptance context first, then a bounded local-only impact packet containing directly related tests, callers, changed-symbol regions, types/references, dependencies, and scope gaps. If CKG is unavailable or unsafe, review remains usable through the existing diff-based path.

## Implemented surfaces

- `scripts/review_context.py` — deterministic packet builder, depth-1 CKG impact retrieval, worktree source reads, optional explicit task boundary, budget and metadata.
- `scripts/review_context_test.py` — packet/budget/retrieval/authority/fallback tests.
- `scripts/review_context_makefile_test.py` — local-review enrichment and cross-vendor non-enrichment wiring tests.
- `Makefile` — `qa-gemma-review` packet-builder integration and task/context inputs.
- `.github/workflows/local-agent-context.yml` — M3 plus M1/M2/reviewer regression coverage.

M1 `ckg_adapter.py`, the existing Gemma/Muse reviewer implementation, and `peer-workflow-review.py` are reused without changing their contracts.

## Design decisions

### D1 — Enrich packets, do not create a new reviewer

M3 reuses the existing reviewer chain. It changes the local packet producer only. Reviewer models, pass counts, RRI routing, D14 behavior, findings parsing, and verdict contracts remain unchanged.

### D2 — Local-only enrichment

CKG-selected source context is supplied only to `qa-gemma-review`. `qa-peer-workflow-review` remains on its pre-M3 packet path; cross-vendor/cloud review receives no M3-selected source until a separately approved cloud-handoff phase defines that disclosure policy.

### D3 — Diff and acceptance have priority over CKG context

Packet priority is:

1. review/task metadata and supplied acceptance criteria;
2. authoritative git diff;
3. CKG impact context;
4. optional lower-ranked related context.

CKG context is the first material removed when the reviewer budget is exhausted. M3 does not truncate the diff or supplied acceptance content to make room for graph context. When acceptance text is unavailable, the packet records that explicitly rather than inventing it.

### D4 — Changed paths/symbols seed depth-1 impact retrieval

Seeds come from the authoritative diff plus supplied acceptance text. Lightweight changed-definition extraction provides symbol anchors to the existing M1 adapter. Review ranking is deterministic:

1. changed-symbol region;
2. directly related tests;
3. direct callers;
4. interfaces/types/references;
5. direct callees/dependencies.

Depth-2 traversal remains outside M3.

### D5 — Worktree-contained local review authority

The CKG remains discovery metadata; source is always re-read from the current review worktree.

Default local review authority is read-only containment inside that worktree, which lets the reviewer inspect unmodified related tests/callers. Path traversal, absolute paths, and symlink escapes are rejected at source-read time.

When the caller supplies an explicit task `allowed_paths` scope, M3 reuses `LocalAgentBoundary.check_path()` before reading a related candidate. M3 does not implement a second task-path matcher. Rejected or unreadable candidates become source-body-free scope gaps.

The CBM index remains subject to the repository `.cbmignore`, which excludes generated/runtime/sensitive paths from graph discovery.

### D6 — Coverage and fail-safe fallback

Coverage is checked for changed paths and discovered impact candidate paths. If CBM is missing, graph discovery fails, coverage is not verified, a supplied task boundary cannot be constructed, or no impact fits safely, mandatory review material remains available through the existing review path.

CKG enrichment is evidence-enhancing, not a single point of failure.

### D7 — Deterministic packet evidence

`review-context-v1` metadata records, without source bodies:

- changed paths and lightweight changed-symbol anchors;
- authority mode;
- selected impact paths and relation/reason;
- scope gaps;
- graph/fallback status and coverage;
- reviewer budget and selected-impact token estimate;
- whether acceptance context was supplied.

No second graph database or hosted service is introduced.

## Dependencies

```text
M1 CKG adapter / source-authority rules ──┐
M2 consolidated local-agent baseline ────┼─> M3-T1 -> M3-T2 -> M3-T3 -> M3-T4
existing Gemma/Muse/D14 reviewer stack ──┘
```

M2 is a baseline dependency rather than a direct algorithmic dependency.

## Non-goals

- changing reviewer models or reviewer routing;
- changing RRI or approval gates;
- sending enriched CKG source to Codex/Claude/cloud reviewers;
- depth-2 graph traversal;
- interactive `request_context`;
- graph-assisted RRI;
- governance/ADR GraphRAG;
- Neo4j;
- general resource/context-window tuning;
- dashboard/observability work.

## Validation state

Deterministic remote evidence at code baseline `086518312bb5b47adb97774473d5a6ae8b838f65`:

- M1 context-provider tests: passed;
- M2 multi-turn tests: passed;
- runtime/CBM contract tests: passed;
- prompt-builder regression tests: passed;
- local-agent integration tests: passed;
- M3 review-context tests: passed;
- M3 Makefile local-only wiring tests: passed;
- Gemma reviewer regression tests: passed;
- `local-agent-context` run 82: success.

General CI on the branch inherits the same `test`, `qa-docs`, and `coverage` failures present on base `main@f3adf34b5722890356d359892334812d66f9f453`. The `qa-docs` failure is the known shallow-checkout receipt-history issue; it is not introduced by M3.

Remaining before merge approval: independent/local audit, including real `codebase-memory-mcp` + Ollama packet inspection on the target environment. No real local runtime execution is claimed by this branch.
