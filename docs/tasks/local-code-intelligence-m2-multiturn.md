---
type: TaskList
title: "Local Code Intelligence M2 — Tasks"
status: complete
description: "M2 task ledger for compact model-visible multi-turn context."
---

# Local Code Intelligence M2 — Tasks

Behavioral coverage contract: unit-v1 + integration-v1

Milestone status: **complete on `feat/ckg-m2-multiturn`**.

Dedicated validation evidence: `local-agent-context` run 59 passed on code head `779d1429a90fbe1df0eaad8f0c8a6a5ab9b79dcd`.

## T1 — Compact model-visible working history

Status: complete

Changes:
- keep full `transcript` unchanged;
- replace replay of raw assistant tool-call JSON with compact action summaries;
- replace verbose tool results with compact results including current source hash for edited files;
- never replay generated source/replacement bodies as history.

Acceptance:
- large write/patch bodies are absent from compact history summaries;
- current edited source remains available through the single active source snapshot;
- audit transcript still contains raw response/result evidence.

## T2 — Replace active source snapshot on edit/repair

Status: complete

Changes:
- make source context an explicit replaceable snapshot in the system/task message;
- after an edit, re-read the existing selected source from the worktree without a graph re-index;
- on formatter/acceptance repair, refresh the provider/graph then replace the current source snapshot rather than append another full source block.

Acceptance:
- exactly one active source snapshot is model-visible;
- current worktree source wins;
- normal edit turns do not force CBM re-indexing.

## T3 — Add deterministic diagnostic compaction

Status: complete

Changes:
- add bounded deterministic formatter/test diagnostic extraction;
- include command, return code, failure/error/assertion/test/file-location signals;
- retain full raw diagnostics in `transcript` only.

Acceptance:
- signal lines survive;
- unrelated bulk output is bounded;
- no model-based summarization.

## T4 — Feed repair hints into CKG refresh

Status: complete

Changes:
- extend `ContextProvider.render_refresh()` with optional hints;
- include edited paths and compact diagnostic summary in CKG repair retrieval text;
- preserve existing authorization intersection and fallback.

Acceptance:
- repair discovery receives edited path/diagnostic anchors;
- out-of-scope diagnostic paths remain subject to the existing boundary and are never exposed merely because diagnostics mention them.

## T5 — Regression/CI closure

Status: complete

Changes:
- add focused unit tests for history and diagnostics;
- add session coverage for current snapshot replacement and repair refresh;
- run provider, CBM runtime, prompt, and existing local-agent integration gates.

Evidence:
- context-provider tests: passed;
- M2 multi-turn tests: passed;
- runtime/CBM contract tests: passed;
- prompt-builder regression tests: passed;
- current local-agent integration tests: passed;
- dedicated workflow: `local-agent-context` run 59, success.

General repository `ci` was still queued when this ledger was closed; it is not used to claim additional M2-specific evidence.

No merge to `main` was performed.
