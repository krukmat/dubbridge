---
type: TaskList
title: "Local Code Intelligence M3 — Reviewer Intelligence Tasks"
status: active
description: "Ordered task ledger for local-only CKG-enriched reviewer packets."
plan: docs/plan/local-code-intelligence-m3-reviewer-intelligence.md
---

# Local Code Intelligence M3 — Reviewer Intelligence Tasks

Behavioral coverage contract: unit-v1

## Milestone status

Status: **in progress on `feat/ckg-reviewer-intelligence`**.

Base: M1 + M2 merged to `main` by PR #6.

Task order and dependency chain:

```text
M3-T1 -> M3-T2 -> M3-T3 -> M3-T4
```

## M3-T1 — Define deterministic review-packet contract

Status: **in progress**

Effort: S

Depends on: M1/M2 merged baseline and existing reviewer stack.

Changes:
- define a reusable packet builder that treats task acceptance + authoritative diff as mandatory content;
- define optional CKG impact sections and machine-readable metadata;
- define reviewer-budget behavior so optional graph context is dropped before mandatory content;
- keep the builder model-agnostic and read-only.

HP-1: acceptance text + a normal diff -> packet contains both unchanged plus a stable metadata/impact section.

EC-1: optional impact context exceeds its budget -> impact entries are omitted/truncated by whole entry while acceptance + diff remain intact.

Acceptance:
- deterministic unit tests for packet shape and budget priority;
- no model call is required to build or validate the packet;
- no reviewer routing changes.

Evidence to emit:
- `scripts/review_context_test.py` results;
- exact packet metadata assertions.

Status artifacts affected:
- this ledger;
- M3 plan if implementation reveals a contract change.

Agent handoff prompt: implement the smallest deterministic packet abstraction that preserves mandatory acceptance/diff content and gives optional impact entries a bounded remainder budget. Do not change reviewer routing.

## M3-T2 — Add CKG depth-1 impact retrieval

Status: **pending**

Effort: M

Depends on: M3-T1; M1 `ckg_adapter.py` contracts.

Changes:
- derive changed paths and lightweight changed-symbol anchors from the diff where practical;
- query the existing CKG backend for direct related tests/callers/references/types/callees;
- rank depth-1 impact candidates deterministically;
- read current worktree source only after the candidate is permitted by the applicable task/path boundary;
- record rejected/unreadable candidates as scope gaps without source bodies;
- return a safe no-impact/fallback result when CBM is unavailable or coverage is insufficient.

HP-1: changed implementation path/symbol with a directly related test -> related test is ranked into local review impact context.

HP-2: changed symbol with a direct caller -> caller is represented ahead of lower-priority callee/reference context when the budget is tight.

EC-1: graph discovers a related path outside task authority -> path may be reported as a scope gap, but source body is absent.

EC-2: CBM is unavailable/invalid -> impact retrieval reports fallback and packet construction continues.

Acceptance:
- unit tests cover ranking, authorization/scope-gap behavior and backend failure;
- depth remains 1;
- no second authorization matcher is introduced;
- graph content is discovery metadata; current worktree remains source authority.

Evidence to emit:
- focused fake-adapter tests in `scripts/review_context_test.py`;
- assertions proving unauthorized source is absent.

Status artifacts affected:
- this ledger;
- M3 plan if adapter reuse requires a documented seam extraction.

Agent handoff prompt: reuse the M1 adapter and authorization concepts to produce depth-1 review impact candidates. Preserve source authority and fail safely to an empty optional impact section.

## M3-T3 — Integrate enrichment into the existing local reviewer

Status: **pending**

Effort: M

Depends on: M3-T2; existing `gemma-code-review.py` and Makefile review flow.

Changes:
- feed the enriched packet to the existing local Gemma/Muse reviewer path;
- provide the packet builder the diff, task/acceptance context when available, repository/worktree information, reviewer context budget, and optional boundary information;
- preserve existing review response parsing, passes, receipts, finding disposition and D14 fallback;
- keep cross-vendor/cloud peer packets on their current disclosure contract.

HP-1: local code review with healthy CKG -> reviewer input contains mandatory diff/acceptance plus selected impact context.

EC-1: local code review with unavailable CKG -> reviewer receives a valid diff-based packet and executes normally.

EC-2: routing selects cross-vendor reviewer -> no CKG-selected source enrichment is injected into that packet.

Acceptance:
- existing reviewer unit tests remain green;
- new integration tests assert local enrichment and cloud-path non-enrichment;
- no changes to RRI band thresholds, reviewer identity, pass count or verdict semantics.

Evidence to emit:
- `gemma_code_review_test.py` / `peer_workflow_review_test.py` or narrower new unit evidence;
- packet inspection proving local-only enrichment.

Status artifacts affected:
- this ledger;
- M3 plan;
- Makefile/workflow only if needed to expose deterministic inputs.

Agent handoff prompt: integrate the packet builder at the caller boundary. Treat local reviewer enrichment as optional and keep cross-vendor packet construction unchanged.

## M3-T4 — Regression, fallback and milestone closure

Status: **pending**

Effort: S

Depends on: M3-T3.

Changes:
- add/finish focused M3 regression coverage;
- run existing M1/M2 context-provider tests plus reviewer tests affected by the integration;
- verify OKF/doc gates;
- document any operator-local CBM/Ollama validation that remains intentionally outside remote evidence;
- prepare branch for independent audit before merge.

HP-1: full M3 deterministic suite passes with the existing M1/M2 and reviewer contracts.

EC-1: CKG unavailable -> review path remains usable and audit metadata explains fallback without changing review verdict semantics.

EC-2: a large optional impact set -> packet remains within the configured impact budget while mandatory review material remains complete.

Acceptance:
- `make qa-okf-frontmatter` passes;
- M1/M2 dedicated Python context tests remain green;
- M3 review-context tests pass;
- affected reviewer tests pass;
- no new deterministic failure relative to `main` is left unexplained;
- branch documentation is synchronized and ready for local audit.

Evidence to emit:
- CI/workflow run IDs at final HEAD;
- unit coverage certification below;
- final changed-file list and known local-only validation boundary.

Status artifacts affected:
- this ledger (`complete` when closure gates pass);
- M3 plan (`complete` when closure gates pass);
- audit entrypoint for M3 if needed for team handoff.

Agent handoff prompt: close only after deterministic gates pass at the exact branch HEAD. Do not claim real CBM/Ollama execution unless it was actually run on the target environment.

## Unit coverage certification

To be completed task-by-task with concrete test references and recorded pass evidence before milestone closure.

## Owner final verification

To be completed at final branch HEAD after implementation and deterministic validation.
