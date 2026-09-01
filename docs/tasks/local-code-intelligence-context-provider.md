---
type: TaskList
title: "Local Code Intelligence Context Provider — Tasks"
status: complete
description: "M1 task ledger for the local code intelligence context provider."
---

# Local Code Intelligence Context Provider — Tasks

Behavioral coverage contract: unit-v1

## Milestone status

Implementation status: **complete on branch `feat/ckg-context-provider`**.

Remote validation status:
- context-provider unit tests: implemented;
- runtime/CBM contract tests: implemented;
- prompt-builder regression tests: included in the dedicated workflow;
- current local-agent integration tests: included in the dedicated workflow;
- GitHub CI: must be green at the final branch head before handoff.

Operator-local validation intentionally remains outside this milestone's remote evidence:
- real `codebase-memory-mcp` binary on the target Mac;
- real Ollama/model invocation;
- end-to-end memory/residency observation on the 32 GB host.

No merge to `main` is part of this task.

## T1 — Runtime budget and model lifecycle reconciliation

Status: **complete**

Effort: S

Changes:
- build the local-developer system prompt from the active runtime `num_ctx` / `num_predict` values;
- explicitly unload the implementer at runner exit;
- preserve current chat and tool-call behavior.

HP-1: default 32K/8K invocation builds the same logical local-developer contract and runs normally.

EC-1: a reduced `--num-ctx 16384` invocation validates against 16K rather than the import-time 32K profile.

EC-2: runner completion triggers an explicit model unload so a later heavy role does not overlap by keep-alive alone.

Acceptance:
- unit coverage for runtime prompt construction;
- unit coverage for unload helper/call path;
- existing local-agent tests remain green.

Evidence: `context_runtime_test.py`, `context_provider_test.py`, dedicated workflow.

## T2 — Introduce the context-provider seam

Status: **complete**

Effort: M

Changes:
- add `ContextProvider` contract;
- add `LegacyContextProvider` preserving current complete-authorized-context rendering;
- make `session_loop.py` use the provider for initial and repair context;
- keep `RunnerFileTools.preload_context()` only as legacy compatibility/fallback.

HP-1: a legacy provider renders exactly the authorized file context expected by the current runner.

EC-1: repair refresh uses the same provider seam instead of calling `preload_context()` directly.

Acceptance:
- behavior-preserving tests for initial and repair rendering;
- no CKG-specific logic in `session_loop.py`.

Evidence: provider tests plus current local-agent integration tests.

## T3 — Add worktree identity and `ckg-context-manifest-v1`

Status: **complete**

Effort: M

Changes:
- derive base repository revision;
- derive deterministic worktree-state hash from tracked, staged, and relevant untracked state;
- hash untracked file contents so content-only changes alter the identity;
- add retrieval manifest schema/model and serialization.

HP-1: clean worktree manifest binds to base revision and records `dirty=false`.

EC-1: dirty worktree with unchanged HEAD produces a distinct state hash.

EC-2: modifying the contents of an existing untracked file changes the worktree state hash.

Acceptance:
- deterministic unit tests for clean/dirty identity;
- manifest does not store source bodies.

Evidence: `ckg_manifest.py`, schema, runtime/provider tests.

## T4 — Add backend-neutral CKG adapter and deterministic retrieval policy

Status: **complete**

Effort: M

Changes:
- add backend-neutral candidate/result types;
- add one-shot `codebase-memory-mcp` CLI adapter using JSON on stdin;
- unwrap CBM's MCP JSON envelope fail-closed;
- index the exact disposable task worktree rather than a sibling/shared checkout;
- define Minimum Useful Graph labels/relationships;
- implement anchor ordering, authorization filtering, ranking, and budget fit;
- use task specification plus acceptance criteria/tests as retrieval input;
- depth 1 default;
- refresh the incremental worktree index before repair-context retrieval.

HP-1: explicit target symbol ranks ahead of direct dependency and text-only candidate.

EC-1: dependency outside `allowed_paths` is recorded as a scope gap and no source is rendered.

EC-2: candidate set larger than retrieval budget drops lowest-ranked optional context without truncating task constraints.

EC-3: repair retrieval requests an incremental graph refresh against the changed worktree.

Acceptance:
- unit tests for ranking, authorization, budget fit, CBM transport, exact-worktree indexing, and refresh;
- subprocess calls are bounded and parsed fail-closed.

Evidence: `ckg_adapter.py`, `context_provider_test.py`, `context_runtime_test.py`.

## T5 — Add `CKGContextProvider`, coverage/source fallback, and selective rendering

Status: **complete**

Effort: M

Changes:
- resolve candidates through the adapter;
- check graph coverage;
- use current dirty worktree source as authority;
- render selected source only;
- emit `ckg-context-manifest-v1` evidence;
- fall back to legacy source path when CKG is unavailable or coverage is insufficient;
- keep graph snapshot identity separate from current source identity.

HP-1: a task under a large authorized directory renders only selected task-relevant source.

EC-1: partial/stale graph coverage triggers source fallback.

EC-2: unavailable CKG backend leaves the existing local runner usable through legacy fallback.

EC-3: dirty worktree source wins over graph snapshot content.

Acceptance:
- provider unit/integration coverage for selection and fallback;
- existing task authorization and editing semantics unchanged.

Evidence: provider/runtime tests, manifest evidence, fallback state in runner output.

## Handoff boundary

When final remote CI is green, **no additional implementation remains on the remote side for milestone 1**. The next action is the operator-local smoke on the target Mac. Any defect found by that smoke becomes a follow-up against this branch/PR rather than an unverified claim in this ledger.

## Deferred follow-ups

These are intentionally outside milestone 1 and are not blockers for this PR:

- working-history compaction;
- selective diagnostic/repair compaction;
- reviewer post-change graph impact packet;
- bounded cloud takeover context;
- per-role context-window reductions after real-use evidence.
