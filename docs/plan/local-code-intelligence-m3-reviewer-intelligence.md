---
type: Plan
title: "Local Code Intelligence — M3 Reviewer Intelligence"
status: active
description: "Plan for enriching local code-review packets with authorized, budgeted CKG impact context."
---

# Local Code Intelligence — M3 Reviewer Intelligence

## Objective

Improve the existing local review gate by adding deterministic impact context derived from the M1 CKG without changing reviewer routing, RRI policy, review verdict semantics, or cloud disclosure behavior.

The reviewer should receive the authoritative diff and task acceptance context first, then a bounded local-only impact packet containing directly related tests, callers, callees/types/references, and scope gaps. If CKG is unavailable or unsafe, review remains usable through the existing diff-based path.

## Affected files

Primary expected surfaces:

- `scripts/review_context.py` (new)
- `scripts/review_context_test.py` (new)
- `scripts/local-agent/ckg_adapter.py`
- `scripts/local-agent/context_budget.py`
- `scripts/gemma-code-review.py`
- `scripts/gemma_code_review_test.py`
- `Makefile`
- `.github/workflows/local-agent-context.yml` or the smallest existing workflow that owns these Python contracts

Supporting docs/tasks may change as evidence is produced.

## Design decisions

### D1 — Enrich packets, do not create a new reviewer

M3 reuses the existing reviewer chain. It changes the packet producer only. Reviewer models, pass counts, RRI routing, D14 behavior, and findings contracts remain unchanged.

### D2 — Local-only enrichment

CKG-selected source context is supplied only to the local reviewer path. Cross-vendor/cloud review continues to receive its existing packet contract until a separately approved cloud-handoff phase defines disclosure policy.

### D3 — Diff and acceptance are mandatory

Packet priority is:

1. review/task metadata and acceptance criteria;
2. authoritative git diff;
3. CKG impact context;
4. optional lower-ranked related context.

CKG context is the first material removed when the packet exceeds the reviewer budget. M3 must never silently truncate the diff or mandatory acceptance constraints to make room for graph context.

### D4 — Changed paths/symbols seed impact retrieval

Seeds come from the authoritative diff plus task/acceptance text. The first implementation is depth-1 only and prioritizes:

1. directly related tests;
2. direct callers;
3. interfaces/types/traits/references;
4. direct callees/dependencies.

Depth-2 traversal is not part of M3.

### D5 — Reuse M1 boundaries

M3 reuses the existing CKG adapter and repository path authorization semantics where a task boundary exists. A graph relation never grants source authority. Unauthorized/unreadable related paths may be represented as scope gaps, but their source bodies are never included.

### D6 — Fail-safe fallback

If CBM is missing, graph discovery fails, coverage is insufficient, or the optional impact packet cannot be built safely, the existing reviewer remains usable with the current diff-based packet. CKG enrichment is evidence-enhancing, not a single point of failure.

### D7 — Deterministic packet evidence

The packet builder exposes machine-readable metadata sufficient to audit:

- changed paths;
- selected impact paths and relation/reason;
- scope gaps;
- graph/fallback status;
- selected-context token estimate and budget.

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

## Validation strategy

M3 is complete when deterministic tests prove:

- a changed symbol/path can surface a directly related test/caller in the local review packet;
- out-of-authority related source is not disclosed;
- CKG unavailability preserves diff-only review;
- mandatory diff/acceptance content wins over optional CKG context under budget pressure;
- the cross-vendor review path is unchanged and receives no new CKG source context;
- existing reviewer parser/result contracts remain green.
