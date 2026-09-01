# CKG M1–M2 Local Merge Audit — Entry Point

## Purpose

This document is the single entry point for the local audit of the consolidated M1 + M2 implementation on `feat/ckg-context-provider` before any merge to `main`.

The audit decision must be one of:

- **MERGE** — no merge-blocking finding remains;
- **MERGE WITH FIXES** — findings are understood and bounded, but fixes are required before merge;
- **DO NOT MERGE** — a correctness, authorization, source-authority, fallback, auditability, or operational issue makes the current candidate unsafe to merge.

This is an audit and validation pass, not a feature-development milestone. Do not expand scope into reviewer intelligence, cloud handoff, graph-assisted RRI, Neo4j/GraphRAG, `request_context`, or context-window optimization.

## Audit target and freeze rule

Repository: `krukmat/dubbridge`

Branch: `feat/ckg-context-provider`

Pull request: `#6` → `main`

M1 + M2 implementation baseline before this audit-packaging document:

`f687f94db6ea904c425cb5f77f5f678df3aa1431`

Base `main` revision incorporated by that baseline:

`c65b4583ab86adcbe1805b6a6288603d13cef54a`

At the beginning of the audit, freeze and record the exact candidate HEAD:

```bash
git fetch origin
git switch feat/ckg-context-provider
git pull --ff-only
AUDIT_HEAD="$(git rev-parse HEAD)"
printf 'AUDIT_HEAD=%s\n' "$AUDIT_HEAD"
git merge-base --is-ancestor f687f94db6ea904c425cb5f77f5f678df3aa1431 "$AUDIT_HEAD"
git status --short
```

The branch must remain unchanged while that audit result is being produced. If HEAD moves, either restart the audit against the new HEAD or explicitly revalidate every affected finding/evidence item.

The implementation baseline above is intentionally older than this entrypoint commit. At handoff, changes after `f687f94d...` should be audit/documentation-only unless a later fix is explicitly made and re-audited.

## Authoritative references

Do not reconstruct the design from commit messages. Use these documents as the primary references:

### M1 — Context selection seam and CKG provider

- [M1 plan](../plan/local-code-intelligence-context-provider.md) — objective, architecture, design decisions D1–D10, backend, rollback, validation boundary.
- [M1 task ledger](../tasks/local-code-intelligence-context-provider.md) — T1–T5 acceptance cases and implementation evidence.
- [CKG manifest schema](../schemas/ckg-context-manifest-v1.yaml) — retrieval evidence contract.

### M2 — Multi-turn efficiency

- [M2 plan](../plan/local-code-intelligence-m2-multiturn.md) — history compaction, one-current-snapshot rule, deterministic diagnostics, repair hints, acceptance cases.
- [M2 task ledger](../tasks/local-code-intelligence-m2-multiturn.md) — T1–T5 implementation/evidence ledger.

### Executable validation contract

- [`local-agent-context` workflow](../../.github/workflows/local-agent-context.yml) — authoritative remote test command list.

### Primary implementation surfaces

Review these source files against the plans rather than treating this list as a substitute for the PR diff:

- [`context_provider.py`](../../scripts/local-agent/context_provider.py)
- [`ckg_adapter.py`](../../scripts/local-agent/ckg_adapter.py)
- [`ckg_manifest.py`](../../scripts/local-agent/ckg_manifest.py)
- [`context_budget.py`](../../scripts/local-agent/context_budget.py)
- [`session_loop.py`](../../scripts/local-agent/session_loop.py)
- [`multiturn_context.py`](../../scripts/local-agent/multiturn_context.py)
- [`diagnostic_compaction.py`](../../scripts/local-agent/diagnostic_compaction.py)
- [`runner_file_tools.py`](../../scripts/local-agent/runner_file_tools.py)
- [`ollama_lifecycle.py`](../../scripts/local-agent/ollama_lifecycle.py)
- [`cli.py`](../../scripts/local-agent/cli.py)

Use the PR diff as the source of truth for the complete changed-file set:

```bash
git diff --stat origin/main..."$AUDIT_HEAD"
git diff --name-status origin/main..."$AUDIT_HEAD"
git diff --check origin/main..."$AUDIT_HEAD"
```

## Architectural invariants to prove

The audit should prove the following invariants, not merely confirm that tests execute.

### 1. Context selection remains an orchestrator concern

`session_loop.py` consumes the `ContextProvider` seam. Backend-specific CKG logic must not leak into the session loop or filesystem tool layer.

Expected shape:

```text
TaskCard / Capsule
        |
        v
 ContextProvider
        |
        +-- LegacyContextProvider
        |
        +-- CKGContextProvider
                |
                +-- anchors / graph retrieval
                +-- authorization intersection
                +-- deterministic ranking
                +-- retrieval budget
                +-- manifest evidence
        |
        v
selected authorized source
        |
        v
existing session loop
        |
        v
Local Implementer
```

### 2. Authorization dominates retrieval

The essential security invariant is:

```text
selected_context ⊆ allowed_paths
```

Graph relevance, repair hints, diagnostics, text search, callers/callees, or stale graph data must never widen task authority.

A candidate rejected by the existing path boundary may be recorded as a `scope_gap`, but its source body must not become model-visible.

### 3. Current worktree source is authoritative

The graph is discovery metadata, not source authority. Dirty tracked/staged/relevant untracked state must affect worktree identity, and the source read immediately from the task worktree must win over any stale graph snapshot.

### 4. CKG is optional, not a single point of failure

Expected behavior:

```text
healthy CKG + adequate coverage -> selective context
partial/stale coverage          -> legacy/source fallback
CKG unavailable                 -> legacy/source fallback
```

Fallback must not widen authority, silently truncate mandatory task constraints, or change editing semantics.

### 5. M2 changes model-visible history, not audit evidence

The full transcript remains lossless. Generated `write_file` / `apply_patch` bodies and bulk diagnostics must not be repeatedly replayed in model-visible working history.

There must be one current source snapshot. After an edit, current selected source is re-read from the worktree without forcing a graph re-index. On formatter/acceptance repair, the provider/graph is refreshed using bounded hints and the active snapshot is replaced.

### 6. Runtime budget and model lifecycle use actual invocation state

`num_ctx` / `num_predict` from the active runtime profile must drive validation and retrieval budget calculations. A smaller runtime profile must not be validated against an import-time 32K assumption.

A live runner-owned Ollama model should be explicitly unloaded on completion. The runner must not unload an unrelated model or an injected test transport.

## Audit sequence

Follow this order so later behavioral tests are interpreted against the intended contracts.

### A. Repository and diff integrity

1. Record `AUDIT_HEAD` and `origin/main` SHA.
2. Confirm clean worktree before testing.
3. Inspect `origin/main...AUDIT_HEAD` and confirm the change is limited to local-agent context/CKG/M2 surfaces, supporting tests/schema/workflow/docs, plus any explicitly audited follow-up fixes.
4. Confirm `main` is an ancestor of the audit candidate or document any later base drift.

### B. Architecture and separation of concerns

Review the provider seam, adapter boundary, filesystem tools, CLI construction, and session loop against the M1 plan. Flag backend-specific graph behavior in `session_loop.py` or a second authorization implementation as a design regression.

### C. Authorization and confidentiality

Exercise at least one task where graph discovery finds a dependency outside `allowed_paths`.

Required evidence:

- candidate becomes rejected/scope gap;
- source is not rendered to the model;
- manifest may identify the decision/path as permitted by its schema but contains no source body;
- repair diagnostics mentioning an unauthorized file do not bypass the same boundary.

Any unauthorized source exposure is a **merge blocker**.

### D. CKG adapter, worktree identity, and freshness

Review one-shot `codebase-memory-mcp` transport for bounded subprocess behavior and fail-closed parsing.

Exercise or inspect evidence for:

- exact disposable task worktree indexing;
- clean vs dirty worktree identity;
- content changes to relevant untracked files changing worktree state hash;
- modified worktree source overriding graph snapshot content;
- graph refresh before repair-context retrieval.

### E. Fallback behavior

Validate both paths, not only the happy path:

1. healthy CKG / adequate coverage → selective source;
2. CKG unavailable → legacy fallback;
3. stale/insufficient coverage → conservative source fallback where practical to reproduce.

The local runner must remain usable when the CKG backend is absent.

### F. Budgeting and prompt contract

Exercise at least the default runtime profile and one reduced `num_ctx` profile.

Verify that:

- retrieval budget derives from active runtime values;
- mandatory task/acceptance constraints are not silently truncated;
- optional lower-ranked context is dropped first;
- model-facing text does not claim that every authorized file was supplied when selective context is active.

### G. M2 multi-turn behavior

Inspect a multi-turn edit/test/repair sequence and prove:

- full transcript retains raw evidence;
- model-visible history uses compact action/result summaries;
- generated source/patch bodies are not replayed as history;
- exactly one active source snapshot is model-visible;
- a successful edit refreshes that snapshot from the worktree without CBM re-indexing;
- formatter/test failure yields bounded deterministic diagnostics;
- repair refresh uses edited paths/diagnostics as hints without widening authorization.

### H. Automated validation

Run the same commands as the dedicated workflow from repository root:

```bash
python3 scripts/local-agent/context_provider_test.py
python3 scripts/local-agent/multiturn_context_test.py
python3 scripts/local-agent/context_runtime_test.py
python3 scripts/local-agent/prompt_builder_test.py
python3 scripts/local-agent/integration_test.py
```

Record command, exit code, and audit HEAD. Do not substitute an earlier workflow run for tests executed against a later modified candidate.

Remote reference evidence for the consolidated M1 + M2 implementation baseline: `local-agent-context` run **66** succeeded on `f687f94db6ea904c425cb5f77f5f678df3aa1431`.

### I. Real local CBM + Ollama smoke

This evidence cannot be established by remote CI and is required before the final merge decision.

Use the repository/team's existing local-agent launch path; do not invent a separate runner contract for the audit.

Run a representative DubBridge task with sufficiently broad `allowed_paths` that the difference between complete preload and selective retrieval is observable.

Control path:

```text
--context-provider legacy
```

Candidate path:

```text
default/--context-provider auto
--ckg-binary codebase-memory-mcp
--ckg-manifest <audit-output-path>.json
```

The current CLI deliberately exposes `auto` and `legacy`; there is no required explicit `ckg` mode. `auto` attempts local CKG and falls back conservatively.

The smoke must cover:

1. real CBM installation and project/index behavior;
2. healthy selective retrieval;
3. backend-unavailable fallback;
4. at least one edit followed by repair/test failure so worktree refresh can be observed;
5. real Ollama implementer invocation;
6. post-session model unload/residency behavior;
7. macOS memory pressure/swap observation on the target host;
8. confirmation that CBM does not leave an unexpected long-lived heavy process after the one-shot workflow.

## Expected audit evidence

Retain enough evidence to reproduce the decision without storing unnecessary source copies.

At minimum record:

- audit HEAD and base SHA;
- local test command results;
- representative TaskCard/Capsule identifier or task description;
- control (`legacy`) vs candidate (`auto`) outcome;
- generated `ckg-context-manifest-v1`;
- selected paths/symbols and scope gaps;
- proof that selected source remained authorized;
- worktree identity before/after relevant edit;
- fallback reason/evidence;
- compact-vs-lossless transcript observations for M2;
- Ollama unload/residency observation;
- CBM process/index behavior;
- memory-pressure/swap observation sufficient to detect an operational regression.

The manifest itself must not contain source bodies.

## General CI baseline — attribution rule

The base revision `main@c65b4583ab86adcbe1805b6a6288603d13cef54a` is not globally green in the general repository `ci` workflow. At the time this audit entrypoint was prepared, both the base and the consolidated feature candidate showed the same failing general-CI jobs:

- `test`;
- `qa-docs`;
- `coverage`.

Known baseline examples include the migration test still expecting 29 applied migrations while the repository contains 31, and the shallow-checkout documentation consistency problem for historical commit references.

Audit rule:

> A failure that reproduces unchanged on the exact base SHA is not, by itself, evidence of an M1/M2 regression. Any new failure, changed failure mode, or failure in an M1/M2-specific gate must be investigated and attributed before merge.

Do not suppress or silently ignore baseline failures. Record the comparison in the audit result.

## Known documentation caveats

These are known pre-audit documentation inconsistencies, not hidden implementation claims:

1. The M1 task ledger was written with an "all final CI green" handoff statement before the current base-SHA general-CI failures were established. For this audit, use the regression-attribution rule above plus the dedicated workflow and local evidence.
2. The M2 task ledger retains a historical reference to `feat/ckg-m2-multiturn` and run 59. M2 is now consolidated into `feat/ckg-context-provider`; run 66 on `f687f94d...` is the relevant remote evidence for the consolidated implementation baseline.

Treat these as Low/documentation findings unless they conceal a different implementation state. They should be corrected before final merge or explicitly accepted in the audit result.

## Finding format

Record findings in a separate audit result document or your team's normal review system. Use at least:

```text
ID:
Severity: Critical | High | Medium | Low
Area:
File/line or evidence:
Reproduction:
Expected:
Actual:
Impact:
Merge blocker: yes | no
Recommendation:
Status: open | fixed | accepted
```

Severity guidance:

- **Critical** — unauthorized source disclosure, boundary bypass, destructive/unbounded behavior, or evidence invalidating the security model.
- **High** — source-authority/fallback correctness failure, material loss of audit evidence, broken real runner path, or regression that can make normal tasks unreliable.
- **Medium** — bounded correctness/operability issue with a safe workaround or incomplete acceptance behavior.
- **Low** — documentation, clarity, maintainability, or non-blocking evidence issue.

## Exit criteria

A final **MERGE** decision requires all of the following:

- no open Critical or High finding attributable to M1/M2;
- `selected_context ⊆ allowed_paths` demonstrated, including a negative/out-of-scope case;
- current-worktree source authority demonstrated;
- healthy selective path and unavailable-backend fallback demonstrated;
- `ckg-context-manifest-v1` inspected and source-body-free;
- M2 compact history + single-snapshot + repair behavior demonstrated;
- five dedicated local test commands pass at `AUDIT_HEAD`;
- real CBM + Ollama smoke passes on the target Mac;
- model/CBM residency does not show an unacceptable operational regression;
- general-CI failures are compared with the exact base and any delta is attributed;
- Medium/Low findings are either fixed or explicitly accepted with rationale;
- the final decision records the exact audited HEAD.

If a fix is committed after the audit starts, update `AUDIT_HEAD` and rerun the tests/evidence affected by that fix before changing the decision to MERGE.

## Audit result template

```md
# CKG M1–M2 Local Merge Audit Result

Audit date:
Auditors:
Audit HEAD:
Base main SHA:
PR: #6

## Automated validation

- context_provider_test.py:
- multiturn_context_test.py:
- context_runtime_test.py:
- prompt_builder_test.py:
- integration_test.py:
- general CI delta vs base:

## Local runtime validation

- codebase-memory-mcp installation/index:
- legacy control:
- auto selective path:
- authorization negative case:
- stale/unavailable fallback:
- worktree edit + repair refresh:
- manifest inspection:
- Ollama invocation:
- Ollama unload/residency:
- CBM process residency:
- memory pressure/swap:

## Findings

- Critical:
- High:
- Medium:
- Low:

## Decision

MERGE | MERGE WITH FIXES | DO NOT MERGE

Rationale:

Accepted residual risks:
```
