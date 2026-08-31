# Local Code Intelligence Context Provider — Tasks

Behavioral coverage contract: unit-v1

## T1 — Runtime budget and model lifecycle reconciliation

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

Evidence to emit: test results and changed-file diff.

Status artifacts affected: this task ledger and parent plan.

## T2 — Introduce the context-provider seam

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

Evidence to emit: unit results and diff.

Status artifacts affected: this ledger.

## T3 — Add worktree identity and `ckg-context-manifest-v1`

Effort: M

Changes:
- derive base repository revision;
- derive deterministic worktree-state hash from relevant Git status/diff state;
- add retrieval manifest schema/model and serialization.

HP-1: clean worktree manifest binds to base revision and records `dirty=false`.

EC-1: dirty worktree with unchanged HEAD produces a distinct state hash.

Acceptance:
- deterministic unit tests for clean/dirty identity;
- manifest does not store source bodies.

Evidence to emit: tests and sample manifest fixture.

Status artifacts affected: this ledger.

## T4 — Add backend-neutral CKG adapter and deterministic retrieval policy

Effort: M

Changes:
- add backend-neutral candidate/result types;
- add one-shot `codebase-memory-mcp` CLI adapter;
- define Minimum Useful Graph labels/relationships;
- implement anchor ordering, authorization filtering, ranking, and budget fit;
- depth 1 default.

HP-1: explicit target symbol ranks ahead of direct dependency and semantic candidate.

EC-1: dependency outside `allowed_paths` is recorded as a scope gap and no source is rendered.

EC-2: candidate set larger than retrieval budget drops lowest-ranked optional context without truncating task constraints.

Acceptance:
- unit tests for ranking, authorization, and budget fit;
- subprocess calls are bounded and parsed fail-closed.

Evidence to emit: unit results and adapter command examples.

Status artifacts affected: this ledger.

## T5 — Add `CKGContextProvider`, coverage/source fallback, and selective rendering

Effort: M

Changes:
- resolve candidates through the adapter;
- check graph coverage;
- use current dirty worktree source as authority;
- render selected source only;
- emit `ckg-context-manifest-v1` evidence;
- fall back to legacy source path when CKG is unavailable.

HP-1: a task under a large authorized directory renders only selected task-relevant source.

EC-1: partial/stale graph coverage triggers targeted source fallback.

EC-2: unavailable CKG backend leaves the existing local runner usable through legacy fallback.

EC-3: dirty worktree source wins over graph snapshot content.

Acceptance:
- provider unit/integration coverage for selection and fallback;
- existing task authorization and editing semantics unchanged.

Evidence to emit: tests, sample manifest, and fallback evidence.

Status artifacts affected: this ledger and parent plan.

## Deferred follow-ups

- working-history compaction;
- selective diagnostic/repair compaction;
- reviewer post-change graph impact packet;
- bounded cloud takeover context;
- per-role context-window reductions after real-use evidence.
