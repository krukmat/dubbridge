---
type: Audit
title: "CKG M3 Reviewer Intelligence — Local Merge Audit Entry Point"
status: active
description: "Single entry point for independent local audit of M3 reviewer intelligence before merge."
---

# CKG M3 Reviewer Intelligence — Local Merge Audit Entry Point

## Purpose

Audit `feat/ckg-reviewer-intelligence` before merge to `main` and decide:

- **MERGE** — no merge-blocking finding remains;
- **MERGE WITH FIXES** — bounded findings require correction before merge;
- **DO NOT MERGE** — correctness, source-disclosure, fallback, budget, or reviewer-isolation defects make the candidate unsafe.

This audit covers M3 only. M1 + M2 are already in `main` and are regression dependencies rather than review scope unless M3 changes their behavior.

## Freeze the candidate

```bash
git fetch origin
git switch feat/ckg-reviewer-intelligence
git pull --ff-only
AUDIT_HEAD="$(git rev-parse HEAD)"
BASE_HEAD="$(git merge-base origin/main "$AUDIT_HEAD")"
printf 'AUDIT_HEAD=%s\nBASE_HEAD=%s\n' "$AUDIT_HEAD" "$BASE_HEAD"
git status --short
```

Restart/revalidate affected evidence if the branch HEAD changes during audit.

The deterministic implementation baseline before audit-document commits is:

`086518312bb5b47adb97774473d5a6ae8b838f65`

Dedicated reference evidence: `local-agent-context` run **82** — success.

## Authoritative references

Read in this order:

1. [M3 plan](../plan/local-code-intelligence-m3-reviewer-intelligence.md)
2. [M3 task ledger](../tasks/local-code-intelligence-m3-reviewer-intelligence.md)
3. [M1 plan](../plan/local-code-intelligence-context-provider.md) — CKG/source-authority background
4. [M2 plan](../plan/local-code-intelligence-m2-multiturn.md) — regression baseline
5. [`CLAUDE.md`](../../CLAUDE.md) and [Agent Workflow Guide](../playbooks/AGENT_WORKFLOW_GUIDE.md) — repository gates

Primary M3 implementation:

- [`scripts/review_context.py`](../../scripts/review_context.py)
- [`scripts/review_context_test.py`](../../scripts/review_context_test.py)
- [`scripts/review_context_makefile_test.py`](../../scripts/review_context_makefile_test.py)
- [`Makefile`](../../Makefile) — `qa-gemma-review` integration
- [`local-agent-context.yml`](../../.github/workflows/local-agent-context.yml)

Existing contracts that should remain behavior-compatible:

- [`scripts/local-agent/ckg_adapter.py`](../../scripts/local-agent/ckg_adapter.py)
- [`scripts/gemma-code-review.py`](../../scripts/gemma-code-review.py)
- [`scripts/peer-workflow-review.py`](../../scripts/peer-workflow-review.py)

## Core invariants

### 1. Mandatory review material wins

Supplied acceptance/task context and the authoritative diff must not be truncated to fit optional CKG impact source. Optional impact is removed by whole entry when its remainder budget is exhausted.

### 2. CKG is discovery, worktree is source authority

Graph results select candidates only. Any rendered related source must be read from the current review worktree. Reject absolute paths, `..` escapes, and symlink escapes.

### 3. Local-only enrichment

M3-selected source may enter `qa-gemma-review` only. `qa-peer-workflow-review` / cross-vendor routing must not receive the new CKG source blocks.

Any M3 CKG source disclosure to a cross-vendor/cloud peer is a **merge blocker**.

### 4. Review authority is explicit

Without task `allowed_paths`, the local reviewer may read only CKG-selected files contained in the current worktree; repository `.cbmignore` keeps generated/runtime/sensitive paths out of graph discovery.

When `REVIEW_CONTEXT_ALLOWED_PATHS` / `--allowed-path` is supplied, `LocalAgentBoundary.check_path()` must dominate graph relevance. Rejected related source becomes a source-body-free scope gap.

### 5. Failure preserves the existing review path

Missing CBM, invalid response, incomplete coverage, unsafe source read, or zero optional budget must not make the existing diff-based local review unusable.

### 6. Reviewer semantics are unchanged

M3 must not change reviewer identity/routing, RRI thresholds, pass count, findings parser, verdict semantics, receipts, or D14 behavior.

## Deterministic audit commands

From repository root:

```bash
git diff --check origin/main..."$AUDIT_HEAD"
make qa-okf-frontmatter

python3 scripts/local-agent/context_provider_test.py
python3 scripts/local-agent/multiturn_context_test.py
python3 scripts/local-agent/context_runtime_test.py
python3 scripts/local-agent/prompt_builder_test.py
python3 scripts/local-agent/integration_test.py
python3 scripts/review_context_test.py
python3 scripts/review_context_makefile_test.py
python3 scripts/gemma_code_review_test.py
```

Also compare the general CI result to the exact base. At plan handoff, `main@f3adf34b5722890356d359892334812d66f9f453` already fails `test`, `qa-docs`, and `coverage`. The `qa-docs` failure is the shallow-checkout historical-review-receipt problem. Do not classify an unchanged base failure as an M3 regression; investigate every new or changed failure mode.

## Real local CBM + reviewer smoke

Use a representative code change with at least one unmodified related test/caller.

Build/inspect the M3 packet through the existing review path, for example by invoking `make qa-gemma-review` with the normal task-scoped variables used by the team. When task acceptance is available, supply `REVIEW_TASK_FILE=<ledger>` and `GEMMA_REVIEW_TASK_ID=<task-id>`.

Validate:

1. real `codebase-memory-mcp` indexes/discovers the current worktree;
2. changed-definition anchors yield a direct test/caller where one exists;
3. rendered source matches the current worktree, not stale graph text;
4. `GEMMA_REVIEW_CONTEXT_METADATA` contains paths/decisions/budget but no source bodies;
5. optional impact fits the reviewer budget without altering mandatory diff/task text;
6. explicit `REVIEW_CONTEXT_ALLOWED_PATHS` rejects an otherwise relevant related file and records a scope gap;
7. unavailable CBM returns a valid mandatory packet and the local reviewer still runs;
8. real Gemma/Muse review accepts the enriched packet at the configured context profile;
9. a cross-vendor peer invocation does not contain `BEGIN REVIEW IMPACT` source blocks or `review-context-v1` enrichment.

## Findings

Use:

```text
ID:
Severity: Critical | High | Medium | Low
Area:
Evidence / file:line:
Reproduction:
Expected:
Actual:
Impact:
Merge blocker: yes | no
Recommendation:
Status: open | fixed | accepted
```

Suggested severity:

- **Critical** — cross-vendor source disclosure, worktree escape, explicit boundary bypass.
- **High** — stale graph body treated as authority, fallback breaks normal review, mandatory diff/acceptance lost.
- **Medium** — incorrect ranking/budget/metadata that materially weakens review but remains bounded.
- **Low** — documentation, clarity, maintainability, or non-behavioral issue.

## Exit criteria

A **MERGE** decision requires:

- deterministic commands above pass at the audited HEAD, except exact-base general-CI failures with documented attribution;
- an unmodified related test/caller is observed through real CBM when the repository graph contains one;
- worktree source authority is demonstrated;
- explicit task-boundary rejection is demonstrated;
- CKG-unavailable fallback is demonstrated;
- enriched local packet is reviewable by the real local reviewer;
- cross-vendor non-enrichment is demonstrated;
- no open Critical/High finding;
- any merge-blocking Medium is fixed and revalidated;
- final decision records the exact `AUDIT_HEAD`.

## Result template

```md
# M3 Reviewer Intelligence Audit Result

Audit date:
Auditors:
AUDIT_HEAD:
BASE_HEAD:

## Deterministic gates
- diff --check:
- qa-okf-frontmatter:
- M1/M2 regression suite:
- M3 review-context suite:
- Makefile wiring suite:
- Gemma reviewer regression suite:
- general CI delta vs base:

## Real local validation
- CBM discovery/coverage:
- unmodified related test/caller:
- worktree source authority:
- explicit boundary negative case:
- CBM-unavailable fallback:
- local reviewer enriched packet:
- cross-vendor non-enrichment:

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
