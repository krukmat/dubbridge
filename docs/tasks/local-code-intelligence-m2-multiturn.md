# Local Code Intelligence M2 — Tasks

Behavioral coverage contract: unit-v1 + integration-v1

## T1 — Compact model-visible working history

Status: pending

Changes:
- keep full `transcript` unchanged;
- replace replay of raw assistant tool-call JSON with compact action summaries;
- replace verbose tool results with compact results including current source hash for edited files;
- never replay generated source/replacement bodies as history.

Acceptance:
- large write/patch bodies absent from later history summaries;
- audit transcript still contains raw response/result evidence.

## T2 — Replace active source snapshot on repair

Status: pending

Changes:
- make source context an explicit replaceable snapshot in the system/task message;
- on formatter/acceptance repair, refresh the provider then replace the current source snapshot rather than append another full source block.

Acceptance:
- exactly one active source snapshot is model-visible after repair;
- current worktree source wins.

## T3 — Add deterministic diagnostic compaction

Status: pending

Changes:
- add bounded deterministic formatter/test diagnostic extraction;
- include command, return code, failure/error/assertion/test/file-location signals;
- retain full raw diagnostics in `transcript` only.

Acceptance:
- signal lines survive;
- unrelated bulk output is bounded;
- no model-based summarization.

## T4 — Feed repair hints into CKG refresh

Status: pending

Changes:
- extend `ContextProvider.render_refresh()` with optional hints;
- include edited paths and compact diagnostic summary in CKG repair retrieval text;
- preserve existing authorization intersection and fallback.

Acceptance:
- repair discovery receives edited path/diagnostic anchors;
- out-of-scope diagnostic paths remain scope gaps only.

## T5 — Regression/CI closure

Status: pending

Changes:
- add focused unit tests for history and diagnostics;
- extend provider tests for repair hints;
- add session/integration coverage for snapshot replacement;
- run existing local-agent gates.

Acceptance:
- new focused tests green;
- current local-agent integration gate green or any unrelated baseline failure explicitly classified;
- no merge to `main`.
