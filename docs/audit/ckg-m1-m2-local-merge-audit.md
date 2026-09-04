---
type: Audit
title: "CKG M1–M2 Local Merge Audit — Entry Point"
status: active
description: "Single entry point for the local merge-readiness audit of the consolidated CKG M1 and M2 implementation."
---

# CKG M1–M2 Local Merge Audit — Entry Point

## Purpose

This is the single entry point for the local audit of the consolidated M1 + M2 implementation on `feat/ckg-context-provider` before any merge to `main`.

The audit outcome must be one of:

- **MERGE** — no merge-blocking finding remains;
- **MERGE WITH FIXES** — bounded findings require correction before merge;
- **DO NOT MERGE** — correctness, authorization, source-authority, fallback, auditability, repository-gate, or operational evidence makes the candidate unsafe to merge.

This is an audit/validation pass, not a feature-development milestone. Do not expand scope into reviewer intelligence, cloud handoff, graph-assisted RRI, Neo4j/GraphRAG, `request_context`, or context-window optimization.

## Audit target and freeze rule

Repository: `krukmat/dubbridge`

Branch: `feat/ckg-context-provider`

Pull request: `#6` → `main`

M1 + M2 implementation baseline before audit-document fixes:

`f687f94db6ea904c425cb5f77f5f678df3aa1431`

Base `main` revision incorporated by that baseline:

`c65b4583ab86adcbe1805b6a6288603d13cef54a`

At audit start, freeze the actual candidate HEAD:

```bash
git fetch origin
git switch feat/ckg-context-provider
git pull --ff-only
AUDIT_HEAD="$(git rev-parse HEAD)"
printf 'AUDIT_HEAD=%s\n' "$AUDIT_HEAD"
git status --short
git merge-base --is-ancestor origin/main "$AUDIT_HEAD"
```

The branch must remain unchanged while evidence is being produced. If HEAD moves, re-freeze it and rerun every affected gate/evidence item.

## Authoritative references

Use these documents instead of reconstructing design intent from commits.

### Repository governance

- [`CLAUDE.md`](../../CLAUDE.md) — repository development gates and mandatory OKF frontmatter rule.
- [`docs/knowledge/README.md`](../knowledge/README.md) — closed OKF `type` vocabulary and frontmatter contract.
- [`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`](../playbooks/AGENT_WORKFLOW_GUIDE.md) — repository workflow authority.

### M1 — ContextProvider / CKG retrieval

- [M1 plan](../plan/local-code-intelligence-context-provider.md)
- [M1 task ledger](../tasks/local-code-intelligence-context-provider.md)
- [CKG manifest schema](../schemas/ckg-context-manifest-v1.yaml)

### M2 — Multi-turn efficiency

- [M2 plan](../plan/local-code-intelligence-m2-multiturn.md)
- [M2 task ledger](../tasks/local-code-intelligence-m2-multiturn.md)

### Executable validation contract

- [`local-agent-context` workflow](../../.github/workflows/local-agent-context.yml)

## Primary implementation surfaces

Use the PR diff as the source of truth for the full changed-file set. These are the main implementation surfaces to inspect:

- [`context_provider.py`](../../scripts/local-agent/context_provider.py)
- [`ckg_adapter.py`](../../scripts/local-agent/ckg_adapter.py)
- [`ckg_manifest.py`](../../scripts/local-agent/ckg_manifest.py)
- [`context_budget.py`](../../scripts/local-agent/context_budget.py)
- [`session_loop.py`](../../scripts/local-agent/session_loop.py)
- [`working_history.py`](../../scripts/local-agent/working_history.py) — M2 model-visible history compaction
- [`diagnostics.py`](../../scripts/local-agent/diagnostics.py) — M2 deterministic bounded diagnostics
- [`runner_file_tools.py`](../../scripts/local-agent/runner_file_tools.py)
- [`ollama_lifecycle.py`](../../scripts/local-agent/ollama_lifecycle.py)
- [`cli.py`](../../scripts/local-agent/cli.py)

Start with:

```bash
git diff --stat origin/main..."$AUDIT_HEAD"
git diff --name-status origin/main..."$AUDIT_HEAD"
git diff --check origin/main..."$AUDIT_HEAD"
```

## Invariants to prove

The audit must prove behavior, not merely confirm that tests execute.

### 1. Provider separation

`session_loop.py` consumes the `ContextProvider` seam. CKG backend behavior must not leak into the session loop or filesystem authorization layer.

### 2. Authorization dominates retrieval

The fundamental invariant is:

```text
selected_context ⊆ allowed_paths
```

Graph relevance, repair hints, diagnostics, callers/callees, stale graph data, or text search must never widen task authority. An unauthorized candidate may become a `scope_gap`; its source body must never become model-visible.

Any unauthorized source exposure is a merge blocker.

### 3. Worktree source is authoritative

The graph is discovery metadata. Current tracked/staged/relevant-untracked worktree state must define freshness identity, and source read from the actual task worktree must win over stale graph content.

### 4. CKG is optional

Expected behavior:

```text
healthy CKG + adequate coverage -> selective context
partial/stale coverage          -> source/legacy fallback
CKG unavailable                 -> legacy fallback
```

Fallback must preserve authorization and editing semantics.

### 5. M2 compacts model context, not audit evidence

- raw model/tool/test evidence remains in the lossless transcript;
- `write_file` / `apply_patch` bodies are not replayed as conversational history;
- exactly one current source snapshot is model-visible;
- successful edits refresh current source from the worktree without CBM re-indexing;
- formatter/acceptance repair uses deterministic bounded diagnostics and deliberate provider/CKG refresh;
- repair hints do not widen `allowed_paths`.

### 6. Runtime budget/lifecycle use invocation state

Active `num_ctx` / `num_predict` values drive prompt/retrieval budgeting. Runner-owned Ollama models are explicitly unloaded on completion without affecting unrelated/injected transports.

## Audit sequence

### A. Repository integrity and mandatory documentation gates

Run from repository root:

```bash
git diff --check origin/main..."$AUDIT_HEAD"
make qa-okf-frontmatter
```

`make qa-okf-frontmatter` is a **mandatory merge gate** for this candidate. It is not waived by the general-CI baseline rule below. Every changed Markdown file under `docs/` must comply with the repository OKF contract.

Also inspect the changed docs for broken/nonexistent references. A documentation path presented as an implementation surface must resolve to a real file.

### B. Architecture and authorization

Review the provider seam, adapter, file boundary, CLI construction, and session loop against the M1 plan.

Exercise at least one graph-discovered dependency outside `allowed_paths` and retain evidence that:

- it is rejected / represented only as an allowed manifest decision or `scope_gap`;
- its source is not rendered to the model;
- diagnostics mentioning that path cannot bypass the boundary.

### C. CKG adapter, freshness, and fallback

Validate or inspect evidence for:

- exact disposable task worktree indexing;
- clean vs dirty worktree identity;
- relevant untracked content affecting worktree identity;
- modified worktree source overriding graph snapshot content;
- repair-time graph refresh;
- healthy selective retrieval;
- unavailable-backend fallback;
- stale/insufficient-coverage fallback.

### D. Budgeting and prompt contract

Exercise the default runtime profile and at least one reduced `num_ctx` profile. Verify mandatory task/acceptance constraints survive while lower-ranked optional source is dropped first.

### E. M2 multi-turn behavior

Inspect an edit/test/repair sequence and prove:

- lossless transcript retains raw evidence;
- working history uses compact action/result records;
- generated source/patch bodies are not replayed;
- one active source snapshot remains;
- edits call current-source refresh without graph re-index;
- failures produce bounded deterministic diagnostics;
- repair refresh receives edited paths + diagnostics without authorization widening.

### F. Dedicated automated validation

Run exactly these repository tests at `AUDIT_HEAD`:

```bash
python3 scripts/local-agent/context_provider_test.py
python3 scripts/local-agent/multiturn_context_test.py
python3 scripts/local-agent/context_runtime_test.py
python3 scripts/local-agent/prompt_builder_test.py
python3 scripts/local-agent/integration_test.py
```

Record command, exit code, and `AUDIT_HEAD`. Earlier remote runs are reference evidence only; they do not replace local execution after audit fixes.

Historical remote reference: `local-agent-context` run 66 succeeded on the consolidated implementation baseline `f687f94db6ea904c425cb5f77f5f678df3aa1431`.

### G. Real local CBM + Ollama smoke

This evidence is operator-local and required before final merge approval.

Use the repository's existing local-agent launch path. Compare:

```text
--context-provider legacy
```

with:

```text
default/--context-provider auto
--ckg-binary codebase-memory-mcp
--ckg-manifest <audit-output-path>.json
```

The smoke should cover:

1. real `codebase-memory-mcp` installation/index behavior;
2. healthy selective retrieval;
3. backend-unavailable fallback;
4. an edit followed by formatter/test repair;
5. real Ollama implementer invocation;
6. post-session model unload/residency;
7. macOS memory pressure/swap observation;
8. absence of an unexpected long-lived heavy CBM process.

## Required audit evidence

Retain at least:

- `AUDIT_HEAD` and base `main` SHA;
- `git diff --check` result;
- `make qa-okf-frontmatter` result;
- five dedicated local test results;
- representative task description / task identifier;
- legacy vs auto outcome;
- `ckg-context-manifest-v1` output;
- selected paths and scope gaps;
- authorization negative-case evidence;
- worktree identity before/after relevant edit;
- fallback reason/evidence;
- M2 compact-vs-lossless transcript evidence;
- Ollama and CBM residency observations;
- memory-pressure/swap observation.

The CKG manifest must not contain source bodies.

## General CI baseline — attribution rule

The incorporated `main@c65b4583ab86adcbe1805b6a6288603d13cef54a` already had failures in the general repository CI (`test`, `qa-docs`, `coverage`). A failure that reproduces unchanged on the exact base SHA is not by itself evidence of an M1/M2 regression.

However:

> Candidate-specific deterministic gates introduced or violated by the PR are not baseline exceptions.

In particular, `make qa-okf-frontmatter` must pass for the CKG docs before merge even if another `qa-docs` sub-check has a known base failure.

Any new failure, changed failure mode, or M1/M2-specific failure must be attributed before merge.

## Finding format

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

- **Critical** — authorization/boundary disclosure or destructive/unbounded behavior.
- **High** — source-authority/fallback failure, material audit loss, broken real runner path, or major reliability regression.
- **Medium** — deterministic repository gate failure or bounded correctness/operability defect requiring correction before merge when repository policy makes it a gate.
- **Low** — documentation accuracy, clarity, or maintainability defect without behavior impact.

## Exit criteria

A final **MERGE** decision requires all of the following:

- `make qa-okf-frontmatter` passes at the audited HEAD;
- changed audit references resolve to real repository files;
- no open Critical/High finding attributable to M1/M2;
- no open Medium finding that violates a mandatory repository merge gate;
- `selected_context ⊆ allowed_paths` demonstrated, including a negative case;
- current-worktree source authority demonstrated;
- selective CKG path and unavailable-backend fallback demonstrated;
- manifest inspected and source-body-free;
- M2 compact-history / single-snapshot / repair behavior demonstrated;
- five dedicated local tests pass at `AUDIT_HEAD`;
- real CBM + Ollama smoke passes on the target Mac;
- model/CBM residency shows no unacceptable operational regression;
- general-CI failures are compared with the exact base and any delta is attributed;
- remaining Medium/Low findings are fixed or explicitly accepted only when repository policy permits acceptance;
- final decision records the exact audited HEAD.

Any fix committed after audit start changes the candidate. Re-freeze `AUDIT_HEAD` and rerun affected gates/evidence.

## Audit result template

```md
# CKG M1–M2 Local Merge Audit Result

Audit date:
Auditors:
Audit HEAD:
Base main SHA:
PR: #6

## Repository gates

- git diff --check:
- make qa-okf-frontmatter:
- changed-doc reference check:
- general CI delta vs base:

## Dedicated validation

- context_provider_test.py:
- multiturn_context_test.py:
- context_runtime_test.py:
- prompt_builder_test.py:
- integration_test.py:

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
