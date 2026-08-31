# Local Code Intelligence — M2 Multi-turn Efficiency

## Status

Implementation branch: `feat/ckg-m2-multiturn` (stacked on M1).

No merge to `main` is part of this milestone.

## Objective

Reduce model-visible context growth across edit/test/repair turns without weakening the lossless audit transcript, task authorization, acceptance gates, or source authority.

M2 addresses three sources of avoidable duplication:

1. generated `write_file` / `apply_patch` payloads are replayed to the model on every later turn;
2. raw formatter/test output is injected into repair prompts;
3. repair refresh appends another source snapshot instead of replacing the model-visible snapshot.

## Design decisions

### D1 — Full audit transcript remains lossless

`result.transcript` keeps raw model responses and full runner results. M2 only compacts the `messages` list sent back to the model.

### D2 — Compact working history

Successful model actions are represented to later turns as compact action/result records. Generated source bodies and patch replacements are not replayed as conversational history. For edited files, the compact result records the current source hash; current source is obtained from the worktree/context provider when needed.

### D3 — One current source snapshot

The model-visible source snapshot lives in the system/task context. On a repair refresh, the current snapshot is replaced rather than appended. Previous source snapshots remain unnecessary because the worktree is authoritative.

### D4 — Deterministic diagnostics

Formatter and acceptance failures are reduced deterministically to bounded diagnostics: failing command/return code, error/failure/assertion lines, test names, file:line locations, and a small surrounding neighborhood. Raw logs remain in the audit transcript.

No LLM is used to summarize diagnostics.

### D5 — Repair hints guide CKG refresh

Repair refresh includes edited paths and the deterministic diagnostic summary as retrieval hints. These hints are additional anchors for the existing CKG provider; they never widen `allowed_paths`.

### D6 — Existing repair and turn budgets remain unchanged

M2 reduces supplied context but does not change RRI, max turns, max repairs, acceptance semantics, or model bindings.

## Non-goals

- reviewer graph context;
- cloud handoff changes;
- context-window reductions;
- interactive `request_context`;
- graph-assisted RRI;
- semantic log summarization;
- changes to the lossless audit format.

## Acceptance cases

### HP-M2-1 — Large write does not replay generated source

After a large `write_file`, later model turns contain a compact action/result summary and current source only through the active context snapshot, not the previous tool-call JSON payload.

### HP-M2-2 — Repair replaces source snapshot

A failed formatter or acceptance run refreshes the context provider and replaces the active source snapshot. The model-visible message list contains one active `Authorized source context` snapshot.

### HP-M2-3 — Diagnostics are bounded and useful

A Rust compiler/test failure produces a deterministic summary containing the failing command/return code and relevant error/test/file-location lines while omitting unrelated bulk output.

### EC-M2-1 — Full raw evidence remains available

Compaction does not remove raw assistant responses, raw formatter output, or raw test output from the audit transcript.

### EC-M2-2 — Repair hints stay authorized

A diagnostic mentioning a file outside `allowed_paths` may become a graph candidate/scope gap, but its source is never exposed unless already authorized.

### EC-M2-3 — CKG fallback remains usable

If CKG refresh fails, the legacy provider still supplies the current authorized source snapshot; history/diagnostic compaction remains active.

## Validation boundary

Remote tests cover deterministic compaction, diagnostic extraction, source-snapshot replacement, CKG repair hints, and existing local-agent integration behavior. Real Ollama/CBM behavior on the target Mac remains operator-local smoke validation.
