---
type: TaskList
title: "Local Code Intelligence M3 — Reviewer Intelligence Tasks"
status: implemented-audit-pending
description: "Ordered task ledger for local-only CKG-enriched reviewer packets."
plan: docs/plan/local-code-intelligence-m3-reviewer-intelligence.md
---

# Local Code Intelligence M3 — Reviewer Intelligence Tasks

Behavioral coverage contract: unit-v1

## Milestone status

Status: **implementation complete; independent/local audit pending on `feat/ckg-reviewer-intelligence`**.

Base: M1 + M2 merged to `main` by PR #6 (`f3adf34b5722890356d359892334812d66f9f453`).

Deterministic code baseline: `086518312bb5b47adb97774473d5a6ae8b838f65`.

Dedicated evidence: `local-agent-context` run **82** — success across M1, M2, M3, and existing Gemma reviewer regressions.

Task chain:

```text
M3-T1 -> M3-T2 -> M3-T3 -> M3-T4
```

No task is marked repository-closure `[x] Done` yet because the independent/local review and real CBM/Ollama audit have not been executed in this environment.

## M3-T1 — Define deterministic review-packet contract

Status: **implemented; deterministic validation passed**

Effort: S

Depends on: M1/M2 merged baseline and existing reviewer stack.

Implemented:
- reusable `review-context-v1` packet builder;
- mandatory authoritative diff and supplied acceptance material;
- source-body-free machine-readable metadata;
- optional CKG impact section using only the remainder reviewer budget;
- whole-entry fit: optional impact is dropped rather than truncating mandatory review material;
- explicit `acceptance_supplied` evidence when no task acceptance source is available.

Happy paths considered:
- HP-1: acceptance text + normal diff -> packet contains both unchanged plus stable metadata/impact sections.

Edge cases considered:
- EC-1: optional impact exceeds remaining budget -> optional entries are omitted whole while diff + supplied acceptance remain complete.

Unit evidence:
- HP-1: `scripts/review_context_test.py::test_no_ckg_cli_is_deterministic_and_needs_no_backend` — passed in run 82.
- EC-1: `scripts/review_context_test.py::test_mandatory_diff_and_acceptance_are_never_truncated_for_impact` and `scripts/review_context_test.py::test_optional_entries_are_fit_whole_inside_impact_budget` — passed in run 82.

Evidence emitted:
- deterministic packet/metadata assertions in `scripts/review_context_test.py`;
- `review-context-v1` metadata can be persisted via `--metadata-out` / `GEMMA_REVIEW_CONTEXT_METADATA`.

## M3-T2 — Add CKG depth-1 impact retrieval

Status: **implemented; deterministic validation passed**

Effort: M

Depends on: M3-T1 and M1 `ckg_adapter.py`.

Implemented:
- changed-path + lightweight changed-definition anchors derived from the authoritative diff;
- existing `CodebaseMemoryCLIAdapter` reused at depth 1;
- deterministic review ranking: changed-symbol region -> tests -> callers -> types/references -> dependencies;
- coverage checked across changed paths plus discovered impact candidates;
- source re-read from the current worktree rather than graph bodies;
- worktree-contained read-only default, allowing unmodified callers/tests to be reviewed locally;
- optional explicit task authority reuses `LocalAgentBoundary.check_path()`;
- out-of-authority/unreadable candidates become source-body-free scope gaps;
- CBM unavailable/partial coverage degrades to mandatory diff-based review.

Happy paths considered:
- HP-1: changed implementation symbol + related test -> related test is selected into local review context.
- HP-2: caller and dependency candidates -> caller ranks ahead of lower-priority dependency context.

Edge cases considered:
- EC-1: explicit task authority rejects a graph-related path -> no source body is rendered; scope gap records the rejection.
- EC-2: CBM unavailable or coverage partial -> enrichment falls back without breaking mandatory review material.

Unit evidence:
- HP-1/HP-2: `scripts/review_context_test.py::test_depth_one_candidates_are_ranked_for_review_value` — passed in run 82.
- HP-1 worktree authority: `scripts/review_context_test.py::test_default_worktree_scope_can_include_unmodified_related_source` — passed in run 82.
- EC-1: `scripts/review_context_test.py::test_unauthorized_related_source_becomes_scope_gap_without_body` — passed in run 82.
- EC-2: `scripts/review_context_test.py::test_unavailable_ckg_falls_back_to_mandatory_review_packet` and `scripts/review_context_test.py::test_partial_coverage_disables_optional_enrichment` — passed in run 82.

## M3-T3 — Integrate enrichment into the existing local reviewer

Status: **implemented; deterministic validation passed**

Effort: M

Depends on: M3-T2 and existing `gemma-code-review.py`/Makefile flow.

Implemented:
- `qa-gemma-review` creates the authoritative diff once and passes it through `scripts/review_context.py` before the existing reviewer wrapper;
- optional `REVIEW_TASK_FILE` supplies acceptance/task context;
- optional `REVIEW_CONTEXT_ALLOWED_PATHS` supplies an explicit task read boundary;
- metadata output is task-scoped when `GEMMA_REVIEW_TASK_ID` is present;
- existing `gemma-code-review.py` response parsing, multi-pass behavior, receipts and findings handling remain unchanged;
- `qa-peer-workflow-review` is not routed through `review_context.py`, preserving the pre-M3 cross-vendor/cloud disclosure contract.

Happy paths considered:
- HP-1: local reviewer target -> enriched packet reaches the existing reviewer and carries diff/task/context inputs.

Edge cases considered:
- EC-1: CKG unavailable -> builder returns a valid mandatory packet and existing reviewer can proceed.
- EC-2: cross-vendor target -> M3 review-context builder is not invoked and no selected CKG source is injected.

Unit/integration evidence:
- HP-1: `scripts/review_context_makefile_test.py::test_local_gemma_target_enriches_packet_before_existing_reviewer` — passed in run 82.
- EC-1: `scripts/review_context_test.py::test_unavailable_ckg_falls_back_to_mandatory_review_packet` — passed in run 82.
- EC-2: `scripts/review_context_makefile_test.py::test_cross_vendor_peer_target_does_not_invoke_review_context` — passed in run 82.
- existing `scripts/gemma_code_review_test.py` suite — passed in run 82.

## M3-T4 — Regression, fallback and milestone handoff

Status: **remote deterministic validation complete; independent/local audit pending**

Effort: S

Depends on: M3-T3.

Implemented/verified:
- M3 tests added to `.github/workflows/local-agent-context.yml`;
- M1/M2 context regressions execute in the same job;
- existing Gemma reviewer regression suite executes after M3 wiring tests;
- run 82 at code baseline `086518312bb5b47adb97774473d5a6ae8b838f65` passed all dedicated steps;
- general CI failures were compared against base `main@f3adf34b5722890356d359892334812d66f9f453`.

Happy paths considered:
- HP-1: full dedicated M1/M2/M3 + reviewer suite -> green at a single code baseline.

Edge cases considered:
- EC-1: CKG unavailable -> fallback metadata + mandatory reviewer packet remain usable.
- EC-2: impact set exceeds configured budget -> mandatory review material remains intact and optional impact is bounded.

Unit evidence:
- HP-1: GitHub Actions `local-agent-context` run 82, including `context_provider_test.py`, `multiturn_context_test.py`, `context_runtime_test.py`, `prompt_builder_test.py`, `integration_test.py`, `review_context_test.py`, `review_context_makefile_test.py`, and `gemma_code_review_test.py` — passed.
- EC-1: `scripts/review_context_test.py::test_unavailable_ckg_falls_back_to_mandatory_review_packet` — passed in run 82.
- EC-2: `scripts/review_context_test.py::test_mandatory_diff_and_acceptance_are_never_truncated_for_impact` and `scripts/review_context_test.py::test_optional_entries_are_fit_whole_inside_impact_budget` — passed in run 82.

General CI attribution:
- base `main@f3adf34b...` CI run 592 fails `test`, `qa-docs`, and `coverage`;
- M3 branch general CI shows the same inherited gate classes;
- `qa-docs` specifically fails because `actions/checkout@v4` uses `fetch-depth: 1` while existing S-150 review receipts reference historical commit objects unavailable in the shallow checkout;
- this baseline problem is not modified as part of M3.

Remaining local evidence before merge approval:
- real `codebase-memory-mcp` discovery/coverage against a representative changed symbol;
- inspect generated local packet and `review-context-v1` metadata;
- prove an unmodified related test/caller is selected with the real backend;
- exercise explicit `REVIEW_CONTEXT_ALLOWED_PATHS` rejection;
- exercise CBM-unavailable fallback;
- invoke the real local Gemma/Muse reviewer and confirm the packet remains reviewable within the configured context;
- confirm cross-vendor review receives no M3 CKG source context.

## Remote verification summary

Code baseline: `086518312bb5b47adb97774473d5a6ae8b838f65`

Dedicated workflow: `local-agent-context` run 82 — **SUCCESS**.

Changed implementation surfaces at that baseline:
- `.github/workflows/local-agent-context.yml`
- `Makefile`
- `scripts/review_context.py`
- `scripts/review_context_test.py`
- `scripts/review_context_makefile_test.py`
- this M3 plan/task documentation.

## Closure boundary

Remote implementation is complete. Repository task closure and merge approval remain intentionally pending until the independent/local review evidence is produced. No real CBM/Ollama execution is claimed here.
