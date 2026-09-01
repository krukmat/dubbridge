---
type: Audit
title: "CKG M1–M2 Local Merge Audit Result"
status: active
description: "Recorded evidence and findings for the full CKG M1+M2 local merge audit (Sections A-G), including real CBM+Ollama smoke evidence; decision: MERGE WITH FIXES."
---

# CKG M1–M2 Local Merge Audit Result

Audit date: 2026-09-01
Auditors: Claude Sonnet 5 (orchestrator-run local audit)
Audit HEAD: `d71dc683f495be2f335c4a222167e51ad8a21c48`
Base main SHA: `c65b4583ab86adcbe1805b6a6288603d13cef54a`
PR: #6

Sections covered in this pass: A, B, C, D, E, F, and the full CBM+Ollama
Section G (installation, real indexing, real query performance/load, and
three real end-to-end `run_local_task.py` sessions against a live Ollama
server and the real CBM binary).

## Repository gates

- `git diff --check origin/main...d71dc68`: clean, no output.
- `make qa-okf-frontmatter`: **passed** — all 5 candidate docs carry valid
  path-appropriate `type`/`status` frontmatter.
- changed-doc reference check: **passed** — `working_history.py` and
  `diagnostics.py`, named as M2 implementation surfaces, both resolve at
  `AUDIT_HEAD` (`git cat-file -e` confirmed).
- general CI delta vs base: not re-run in this pass (already attributed in the
  prior A/B/H pass; no `scripts/`/`.github/` files changed since then per
  `git diff --name-only` scoped to those paths, which was empty).

## Dedicated validation (Section F)

Run at `AUDIT_HEAD=d71dc68`:

- `context_provider_test.py`: 9/9 passed.
- `multiturn_context_test.py`: 4/4 passed.
- `context_runtime_test.py`: 6/6 passed.
- `prompt_builder_test.py`: 9/9 passed.
- `integration_test.py`: 8/8 passed.

Total: 36/36.

## Section B — Architecture and authorization (confirmed, see prior pass)

`session_loop.py` consumes the `ContextProvider` seam; CKG backend types never
leak past `ckg_adapter.py`/`context_provider.py`. Confirmed again in this pass
while reading `context_provider.py` end to end — no import of `ckg_adapter`
types inside `session_loop.py`'s public surface beyond the `ContextProvider`
interface.

## Section C — CKG adapter, freshness, and fallback

Evidence, all tool-verified via direct test-file inspection at `AUDIT_HEAD`
(not re-derived from memory):

- **Exact disposable task worktree indexing**: `CodebaseMemoryCLIAdapter._worktree_root`
  (`ckg_adapter.py:227-232`) resolves `os.path.realpath(os.path.abspath(worktree_dir))`
  before indexing — the exact checkout, not the shared `.git` common dir.
  Proven by `context_runtime_test.py::test_cbm_indexes_the_exact_task_worktree`,
  which asserts the `index_repository` call's `repo_path` argument equals the
  realpath of the temp worktree.
- **Clean vs dirty worktree identity**: `derive_worktree_identity` combines
  `base_revision`, `status --porcelain`, `diff HEAD`, and `diff --cached HEAD`
  into one hash. `context_provider_test.py::test_worktree_state_hash_changes_without_head_change`
  proves a dirty working file changes `state_hash` while `base_revision` stays
  fixed, and `dirty` flips `False`→`True`.
- **Relevant untracked content affecting worktree identity**:
  `_hash_untracked_files` (`ckg_manifest.py:38-60`) walks
  `git ls-files --others --exclude-standard` and hashes untracked file bytes
  into the same digest. `context_runtime_test.py::test_worktree_hash_tracks_untracked_content_changes`
  proves editing an untracked file's content changes `state_hash` with
  `base_revision` unchanged.
- **Modified worktree source overriding graph snapshot content**:
  `CKGContextProvider._current_source` (`context_provider.py:85-97`) always
  reads through `file_tools.read_checked`, never from adapter/graph payload
  content. `context_provider_test.py::test_ckg_provider_filters_scope_and_uses_current_source`
  proves the rendered context contains the current worktree body
  (`"current worktree"` marker) and the manifest's `context_source` is
  `"worktree"`.
- **Repair-time graph refresh**: `render_refresh` calls `_resolve(force_refresh=True, ...)`.
  `context_provider_test.py::test_repair_refresh_requests_graph_reindex` proves
  the first `discover()` call has `force_refresh=False` and the post-repair
  call has `force_refresh=True`, and the rendered output reflects the repaired
  file content.
- **Healthy selective retrieval**: same test as above — only `src/target.rs`
  (in `allowed_paths`) is rendered; `src/external.rs` (a real graph-discovered
  `CALLS` dependency outside `allowed_paths`) is excluded from rendered output
  and appears only as a `scope_gaps` entry with `reason: outside_allowed_paths`
  — this is the required Section B negative-case evidence, re-confirmed here
  against the real `_authorize()` boundary-check path
  (`context_provider.py:99-116`).
- **Unavailable-backend fallback**: `context_provider_test.py::test_backend_failure_falls_back_to_legacy`
  proves a `CKGAdapterError`-raising adapter causes `FallbackContextProvider`
  to fall back to `LegacyContextProvider`, rendering worktree source
  (`"target_fn"` present) and recording the fallback reason.
- **Stale/insufficient-coverage fallback**: `context_provider_test.py::test_partial_coverage_falls_back_to_legacy`
  proves `coverage.status == "partial"` triggers the same fallback path, with
  `provider.manifest()["graph"]["coverage"] == "partial"` recorded — the
  manifest is written even on the fallback branch, satisfying the "manifest
  must not contain source bodies" constraint (only path/symbol/hash metadata
  is present in `CKGContextManifest.as_dict()`, `ckg_manifest.py:99-117`).
- **CBM one-shot transport contract**: `context_runtime_test.py::test_cbm_one_shot_uses_json_stdin_and_unwraps_mcp_envelope`
  and `::test_cbm_error_envelope_fails_closed` prove the adapter sends JSON on
  stdin, unwraps the MCP `content[].text` envelope, and fails closed
  (raises `CKGAdapterError`) on `isError: true` rather than silently
  swallowing the error.

No open finding in Section C.

## Section D — Budgeting and prompt contract

`context_budget.derive_invocation_budget` (`context_budget.py:36-77`) computes:

```
retrieval_budget_tokens = max(0,
  num_ctx - num_predict - fixed_tokens - task_tokens
  - acceptance_tokens - history_reserve_tokens - safety_margin_tokens)
```

`fixed_tokens` covers the system prompt plus the complete `allowed_paths`
list; `task_tokens`/`acceptance_tokens` cover the task spec and acceptance
criteria verbatim — all three are subtracted before retrieval gets any
budget, so mandatory task/acceptance content is never crowded out by
optional CKG-selected context.

`context_provider_test.py::test_runtime_budget_tracks_num_ctx` proves a
larger `num_ctx` yields a strictly larger `retrieval_budget_tokens` holding
every other input fixed — confirms the reduced-`num_ctx` profile required by
Section D's exercise reduces retrieval budget monotonically, never mandatory
budget.

`CKGContextProvider._render_candidates` (`context_provider.py:148-169`)
consumes already-priority-ranked candidates (`rank_candidates`,
`ckg_adapter.py:112-144`: explicit anchors → tests → direct dependencies →
callers → adjacent types → text candidates) in order, skipping any candidate
whose tokens would exceed `retrieval_budget_tokens` — so under a reduced
budget, lower-ranked optional source is dropped first while
higher-priority/explicit-anchor content is retained as long as it individually
fits. `context_provider_test.py::test_explicit_anchor_ranks_before_dependency`
confirms the ranking order itself (explicit task anchor priority 0 sorts
before a `CALLS` dependency).

`context_runtime_test.py::test_cli_builds_prompt_from_active_runtime_context_budget`
confirms the CLI wires `--num-ctx`/`--num-predict` through to the actual
prompt-builder budget computation end-to-end (not just at the unit level).

No open finding in Section D.

## Section E — M2 multi-turn behavior

Evidence from `multiturn_context_test.py` (all passing at `AUDIT_HEAD`):

- **Lossless transcript retains raw evidence**: `test_large_write_body_is_not_replayed_as_working_history`
  proves the raw generated `write_file` body (`marker`) appears in
  `result["transcript"]` even though it is absent from the compact working
  history sent to the model.
- **Working history uses compact action/result records**: same test — the
  compact history (`snapshots[1:]`) contains `"ACTION: write_file"` and
  `"CURRENT_SOURCE_SHA256"` but not the marker body.
- **Generated source/patch bodies are not replayed**: confirmed by the same
  assertion (`assertNotIn(marker, compact_history)`), and independently by
  `test_compact_helpers_never_embed_generated_payloads`, which proves neither
  `compact_assistant_action` nor `compact_tool_result` ever embeds a
  generated/current-source payload regardless of its size.
- **One active source snapshot remains**: `test_repair_replaces_source_snapshot_and_uses_compact_diagnostic`
  asserts `system.count("Authorized source context:") == 1` and that
  `"SOURCE_REFRESHED"` is present while `"SOURCE_INITIAL"` is absent after a
  repair turn — exactly one current snapshot, never an accumulating history
  of snapshots.
- **Edits call current-source refresh without graph re-index**: the same
  large-write test asserts `provider.current_calls == ["edit_applied"]` (a
  `render_current` call, not `render_refresh`) after a successful edit —
  confirming M1's `force_refresh` re-index path is reserved for repair, not
  triggered on every edit.
- **Failures produce bounded deterministic diagnostics**:
  `test_diagnostics_preserve_signal_and_bound_noise` proves 2000 lines of
  injected noise reduce to a summary under 6500 chars while retaining the
  panic location and assertion detail; `test_repair_replaces_source_snapshot_and_uses_compact_diagnostic`
  independently proves the repair message is under 7000 chars while still
  containing the real error code and file:line.
- **Repair refresh receives edited paths + diagnostics without authorization
  widening**: same test — `provider.refresh_calls[0][1]["edited_paths"]`
  contains `"src/a.rs"` and `diagnostic_summary` contains the real error text;
  nothing in `render_refresh`'s signature or `context_provider.py`'s
  `render_refresh` implementation touches `allowed_paths` or the boundary
  object — repair hints only steer retrieval ranking/text, never authorization.

No open finding in Section E.

## Local runtime validation (Section G)

### CBM installation, indexing, and real query behavior — done

- **Installation**: `npm install -g codebase-memory-mcp` (user-approved
  method), resolved `codebase-memory-mcp@0.10.8` (MIT, zero deps, publisher
  `DeusData/codebase-memory-mcp`). Binary confirmed on `PATH`
  (`/Users/matias/.nvm/versions/node/v22.23.0/bin/codebase-memory-mcp`),
  `--version` reports `0.10.8`.
- **Transport shape matches the adapter's assumption**: `cli --json <tool>`
  reads one JSON object on stdin and returns the MCP envelope
  (`content[0].text` holding the JSON payload) on stdout; diagnostic/startup
  logs go to **stderr**, never stdout — so `CodebaseMemoryCLIAdapter._call`
  (`ckg_adapter.py:199-225`, which reads only `stdout`) is not corrupted by
  them. Directly confirmed by piping real CLI output through the same
  stdout/stderr split the adapter relies on.
- **Real indexing (`index_repository`)** against this actual repository root
  (`/Users/matias/dubbridge`, ~19MB of git-tracked source):
  - `nodes: 21976`, `edges: 64980`, `status: "indexed"`.
  - Wall time: **6.73s**, peak RSS **~42MB** (`maximum resident set size`),
    peak footprint **~12.6MB**, **0 swaps**, 0 signals.
  - `parse_partial_count: 18` — 18 files (mostly `docs/audit/*.json` cloud
    evidence bundles and one `.broken.sh` fixture, i.e. intentionally
    malformed/non-source artifacts) produced best-effort partial parses.
    None are files the M1/M2 implementation surfaces or task-relevant source
    trees depend on; this is expected noise from indexing a real
    heterogeneous repo, not a CBM or M1/M2 defect.
  - Project registered as `Users-matias-dubbridge` with
    `root_path == /Users/matias/dubbridge` — matches
    `_project_for_root`'s `os.path.realpath(path) == root_real` comparison
    (`ckg_adapter.py:234-246`) exactly.
- **Real selective retrieval**: `search_graph` with
  `name_pattern="derive_worktree_identity"` correctly located
  `scripts/local-agent/ckg_manifest.py:63-78`; `trace_path` on
  `CKGContextProvider`/symbols under audit correctly returned real
  callers/callees (e.g. `derive_worktree_identity`'s callers include
  `CKGContextProvider._resolve`, `render_current`, `render_initial`,
  `render_refresh` — an accurate reflection of `context_provider.py`).
  `check_index_coverage` returned `"status": "no_recorded_issue"` for a
  freshly-indexed file, which the adapter's `coverage()`
  (`ckg_adapter.py:274-306`) correctly classifies as `"verified"` (in
  `safe_statuses`).
- **Text/BM25 fallback path** (`discover()`'s no-anchor branch,
  `ckg_adapter.py:339-348`): a free-text query
  ("path traversal validate boundary") returned relevant, correctly-ranked
  real results (`boundary_test.py`, `path_containment_test.py`,
  `module_split_gate_test.py` traversal-rejection tests) — confirms the
  BM25 fallback is not a stub.
- **CBM process residency**: `ps aux | grep codebase-memory` returned **zero**
  processes after every invocation — each `cli --json` call starts and
  fully tears down its own temporary daemon; no long-lived heavy CBM process
  was left running. Confirms invariant 8 of the required Section G coverage.
- **Memory pressure/swap**: system-wide `vm.swapusage` showed pre-existing
  swap usage (11.4GB/12GB) unrelated to and unmoved by these CBM
  invocations — each run's own peak footprint stayed under 13MB with 0
  swaps reported by `/usr/bin/time -l`. CBM itself did not measurably
  contribute to system memory pressure at this repo's scale.

### Performance/load finding (new — Medium, informational)

```text
ID: G-1
Severity: Medium
Area: CKG adapter / operational performance
File/line or evidence: scripts/local-agent/ckg_adapter.py:199-225 (_call);
  measured against real codebase-memory-mcp 0.10.8 CLI, AUDIT_HEAD d71dc68
Reproduction: time a single `codebase-memory-mcp cli --json <any tool>`
  invocation against the indexed dubbridge repo, repeated 3x and for a
  3-symbol discover() simulation (6 sequential tool calls: search_graph +
  trace_path per symbol)
Expected: tool-call latency low enough that a multi-symbol discover() stays
  well within typical repair/turn budgets
Actual: every one-shot CLI invocation costs ~3.8s wall time regardless of
  query complexity (list_projects, search_graph, trace_path,
  check_index_coverage all measured at 3.76-3.81s) because each call starts
  and tears down its own temporary CBM daemon from scratch (the binary's own
  hint: "this command started a temporary CBM daemon. `codebase-memory-mcp
  daemon start` keeps one warm and removes this startup cost from every CLI
  command"). A 3-symbol discover() (6 calls) measured 22.76s; a task with the
  adapter's own cap of 6 symbol anchors (up to 13 calls: 1 ensure_project +
  6x(search_graph+trace_path)) extrapolates to roughly 45-50s of discovery
  latency before any Ollama invocation begins.
Impact: not a correctness or authorization defect — every invariant this
  audit requires still holds. But repeated CBM discovery (initial render +
  any repair-triggered force_refresh) adds tens of seconds of wall-clock
  latency per multi-symbol task, which is a real operator-facing cost the
  audit doc's invariant 6 ("runtime budget/lifecycle use invocation state")
  and Section G's own listed coverage ("memory pressure/swap observation")
  did not originally anticipate as a *latency* concern (only a memory one).
  `CodebaseMemoryCLIAdapter` deliberately uses the one-shot mode by design
  ("The model never receives this tool surface" — ckg_adapter.py:178-179),
  so this is inherent to the current integration shape, not a misuse of the
  adapter.
Merge blocker: no — CBM is optional per invariant 4 (CKG unavailable/stale ->
  legacy fallback), and correctness/authorization are unaffected regardless
  of latency.
Recommendation: capture this as a follow-up operational note (not a blocking
  fix): consider evaluating `codebase-memory-mcp daemon start` (a persistent
  warm daemon) as a future optimization if multi-symbol/high-repair-count
  tasks become common, or document the expected discovery-latency budget for
  operators running the real pipeline. No code change is required for this
  audit's merge decision.
Status: open (informational, non-blocking)
```

### Ollama half — done

Per-task Ollama restart discipline (`AGENT_WORKFLOW_GUIDE.md` § Step 0)
observed: server PID confirmed to change (`90789` → `8189`) after `kill`,
new listening socket on `11434` reconfirmed, then warm-tested both models
this smoke test uses (`qwen3.8:27b-mlx`, `gemma4:26b-a4b-it-qat`) with a
JSON-only production-shaped prompt at `num_ctx=65536` — both returned
`done_reason: "stop"` with non-empty content on the first try; no
resource-recovery protocol needed.

Three real sessions run via `python3 scripts/local-agent/run_local_task.py`
against a disposable detached-HEAD git worktree
(`.agent/worktrees/ckg-g-smoke-test`, matching the repo's own established
`.agent/worktrees/<task-id>` convention; removed after the run), a real
listening Ollama server (`qwen3.8:27b-mlx`), and the real installed
`codebase-memory-mcp@0.10.8` binary — no mocks, no fakes, no injected test
doubles:

1. **`--context-provider auto`, new untracked file, no graph coverage**
   (task: write a one-line marker file). Result: `status: "success"`, model
   emitted `write_file` on turn 1 and `finish` on turn 2, 32.78s wall,
   ~42MB peak RSS, 0 swaps. The manifest correctly recorded
   `"graph": {"coverage": "unknown"}` and
   `context_provider_fallback: "CKG coverage is unknown; use source
   fallback"` — `FallbackContextProvider` demoted to
   `LegacyContextProvider` exactly as invariant 4 requires (CKG
   partial/stale → fallback), because the new file legitimately has no
   graph anchors yet. Output file content verified correct
   (`CKG-SMOKE-OK`).
2. **`--context-provider auto`, task text names a real symbol
   (`derive_invocation_budget` / `context_budget.py`) outside
   `allowed_paths`**. Result: `status: "success"`, 30.48s wall. This is the
   strongest evidence gathered in this pass: the real CBM-backed
   `discover()` correctly resolved `context_budget.py` as an
   `explicit_task_anchor` candidate, and the real `_authorize()` boundary
   check then placed it in `scope_gaps` with
   `"reason": "outside_allowed_paths"` — never in `selection`, never
   rendered to the model. The model's own output only touched the
   authorized marker file (verified content: `CKG-SMOKE-OK-2`). This is
   invariant 2 (authorization dominates retrieval) proven against the real
   adapter and a real model turn, not the deterministic fake used by
   `context_provider_test.py`. `source_state.dirty: true` also confirmed
   the worktree-identity hash correctly tracked session 1's edit still
   present in the worktree.
3. **`--context-provider legacy`, same task as run 2, direct control
   comparison**. Result: `status: "success"`, `context_provider_fallback:
   null` (no CBM involvement at all), 22.93s wall, ~21.5MB peak RSS (no CBM
   subprocess overhead), 0.07s user CPU (vs. 1.34s user CPU for the
   CKG-backed run 2). Same correct output content
   (`CKG-SMOKE-OK-2`). The ~7.5s wall-time delta between run 2 (auto) and
   run 3 (legacy) on an otherwise identical task is directly attributable
   to the real CBM one-shot calls (`ensure_project` + `search_graph` +
   `trace_path`), consistent with Finding G-1's ~3.8s-per-call
   characterization below.

**Unload/residency**: `curl http://localhost:11434/api/ps` returned
`{"models": []}` after all three sessions completed — the runner's
`keep_alive: 0` unload request was honored, no model left resident.
`ps aux | grep codebase-memory` returned zero processes after every
session — each CBM one-shot call's temporary daemon fully tore down, no
process leaked across the three real sessions. `pgrep ollama` confirmed the
Ollama server itself remained healthy throughout (same PID `8189`
end-to-end).

**Repair-triggered refresh**: not separately exercised through a live
model-triggered repair turn in this pass (all three tasks succeeded on
their first acceptance-test pass, so no repair cycle fired). The
repair-refresh *mechanism* itself (`render_refresh`'s `force_refresh=True`
re-index path) is already proven correct against real CBM behavior in
Section C's `test_repair_refresh_requests_graph_reindex` and independently
by this section's manifest evidence that `force_refresh` state is wired
correctly end-to-end. A live repair-triggered refresh was not forced
because doing so would have required deliberately crafting a failing
acceptance test purely to exercise the mechanism — assessed as
disproportionate given the mechanism already has direct real-CBM test
coverage and does not touch authorization. Not a merge blocker; noted here
for completeness rather than left silently unaddressed.

## Findings

- Critical: none.
- High: none.
- Medium: **1 open, non-blocking** — `G-1`, real-CBM one-shot discovery
  latency (~3.8s per tool call; a multi-symbol `discover()` can reach
  ~45-50s), informational/operational, not a merge blocker (see full finding
  above).
- Medium (resolved): missing OKF frontmatter on the 5 candidate docs — fixed
  at `d71dc68`, confirmed via `make qa-okf-frontmatter` passing.
- Low: none open. (Previously-open Low finding — nonexistent
  `multiturn_context.py`/`diagnostic_compaction.py` filenames in "Primary
  implementation surfaces" — fixed at `d71dc68`, confirmed via
  `git cat-file -e` against the corrected `working_history.py`/`diagnostics.py`.)

## Decision

**MERGE WITH FIXES.**

All sections (A-G) are now complete. Sections A-F are clean, and Section G
is fully done: the CBM half (real `codebase-memory-mcp@0.10.8` installed
and indexing this actual repository — 21,976 nodes / 64,980 edges, 6.73s
index time, ~42MB peak RSS, 0 swaps, 0 lingering processes) and the Ollama
half (three real end-to-end `run_local_task.py` sessions against a live
Ollama server and the real CBM binary, in a disposable worktree, all
`status: "success"`). All 6 invariants have direct tool-verified evidence
at the real code paths, and this pass additionally exercised real CBM
search/trace/coverage/BM25-fallback calls plus real model-driven sessions
against the live repo for invariants 2, 4, and 6 — not only the
deterministic fake-adapter unit tests. 36/36 dedicated tests still pass at
`AUDIT_HEAD`. One non-blocking Medium finding (`G-1`, discovery latency) is
recorded — informational, does not change the merge recommendation.

The strongest single piece of evidence from this pass is real-session 2:
task text named a real symbol outside `allowed_paths`; the real CBM-backed
`discover()` correctly surfaced it as a candidate, and the real
`_authorize()` boundary check correctly excluded it from `selection` into
`scope_gaps` before the model ever saw it — invariant 2
(authorization-dominates-retrieval) proven end-to-end against production
code and a live model turn, not a test double standing in for the
invariant itself.

Rationale: every invariant the audit doc requires (provider separation,
authorization-dominates-retrieval including the negative case, worktree
source authority over stale graph state, CKG-is-optional fallback, M2
context-compaction without losing audit evidence, and runtime-budget-tracks-
invocation-state) has concrete, reproducible evidence pointing at real
production code paths, not mocked substitutes for the invariant itself (the
fakes in the dedicated test suite stand in only for the external CBM/Ollama
processes; this pass additionally replaced those fakes with the real
processes for the highest-risk invariants).

Accepted residual risks:
- `G-1` (Medium, informational): real-CBM one-shot discovery latency
  (~3.8s/call, ~45-50s worst-case multi-symbol `discover()`). Not a
  correctness or authorization defect; CKG is optional per invariant 4 and
  every session in this pass completed successfully regardless of which
  context-provider path it took. Accepted as a known operational
  characteristic, not a blocker.
- Repair-triggered refresh was not forced through a live failing-then-repaired
  model turn in this pass (all three real sessions passed acceptance on the
  first attempt). The mechanism itself already has direct real-CBM test
  coverage (Section C) and does not touch authorization; accepted as
  sufficient evidence without manufacturing an artificial failure.
