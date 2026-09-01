---
type: Audit
title: "X26-T7 implementation/control incidents"
status: recorded
related:
   - docs/tasks/tiger-style-adaptation.md
   - docs/audit/x26-t7-implementation.md
---

# X26-T7 implementation/control incidents

## Context

`X26-T7` implemented ASR-worker guard clauses and narrow exception handling on `main` in commit `f166a55173ac119191ea189c3c2e9ea0c3ae4bd3`.

Per the owner-directed execution policy for this sequence, implementation is not blocked by unrelated repository control failures. Controls are observed and failures are documented for later review.

## CI result relevant to T7

- `python-complexity`: **PASS**. The T6 Ruff gate accepted the T7 worker and test changes.
- `fmt`, `clippy`, `cargo-check`, `maintainability`, `roadmap-drift`, `peer-workflow-review`, `config-secrets`, `mobile`, and `s3-integration` were also green at the time this incident note was recorded.
- The broader CI run still had unrelated/long-running jobs in progress when this note was written; their result is not required for T7 implementation closure.

## INC-T7-01 — `qa-docs` fails on historical S-150 review commit references

**Disposition:** pre-existing / unrelated to T7; documented, not fixed in T7.

The `qa-docs` job reaches `scripts/check-task-unit-coverage.sh` and fails because review evidence in `docs/tasks/s-150-translation-dubbing.md` references commit SHAs that no longer resolve as valid commit objects. The CI log lists 18 affected S-150 task entries, including S-150-T1a, T1c-ii, T2a, multiple T2b/T2c children, and T3a–T3c.

This failure does not reference `workers/asr-worker-py`, the T7 audit evidence, or the Python complexity gate. The same S-150 historical-reference failure was already observed during earlier X26 work.

**Follow-up:** repair or deliberately re-baseline the stale S-150 review-evidence commit references in a separately scoped documentation-maintenance task. Do not fold that repository-history repair into T7.

## T7 implementation status

T7 remains **implemented**. The relevant Python complexity gate is green, and the unrelated `qa-docs` failure is retained here for traceability rather than treated as a reason to reopen the worker change.
