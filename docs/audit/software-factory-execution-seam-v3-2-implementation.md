# Software-Factory Execution Seam v3.2 — Implementation Evidence

**Date:** 2026-09-13  
**Baseline:** `feature/p2p-mvp-core`  
**Implementation branch:** `feature/software-factory-execution-seam-v3-2`

## Scope implemented

Wave 1 was implemented as an additive software-factory normalization facade.

Added:

- `scripts/local-agent/execution_contract.py`
- `scripts/local-agent/execution_evidence.py`
- `scripts/local-agent/run_normalized_task.py`
- targeted tests for all three modules
- plan/task/ADR documentation and ADR index entry

Not modified:

- `run_local_task.py`
- `cli.py`
- `session_loop.py`
- `audit_record.py`
- `fallback_selection.py`
- any P2P/product-runtime code

## Architecture preservation

The implementation preserves these existing authorities:

- RRI/workflow — route eligibility and policy;
- CKG/context providers — authorized context;
- `session_loop` — turn, repair and acceptance-test state machine;
- `fallback_selection` — cloud fallback recommendation/authorization;
- existing audit records — execution verification/signature authority;
- reviewer workflow — quality verdict;
- P2P/product runtime — independent runtime plane.

The normalized `execution-summary-v1` artifact is explicitly non-authoritative. It correlates existing evidence and execution economics.

## Evidence semantics

The normalized summary distinguishes:

- execution session;
- model invocation;
- repair attempt (`repair_diagnostic` events);
- test attempt (`test_result` events);
- materialized fallback handoff (existing fallback-selection object present).

A `local_execution_rejected` result may report `cloud_handoff_required`, but it does not increment the materialized fallback-handoff counter until a fallback-selection/checkpoint artifact exists.

Measured runtime token usage is propagated from the existing `StreamUsage` values when available. Missing token usage remains `null`; it is not fabricated.

RRI policy version is recorded (`rri-v2` for current executions) so incompatible scoring generations are not silently combined in analytics.

## Independent targeted verification performed

A local isolated verification reconstructed the pure normalization modules and executed targeted Python tests.

Verified:

- local resolved contract includes a runtime preset and no routing priority;
- cloud-only resolved contract has no local runtime preset;
- current policy version defaults to `rri-v2` for current cards;
- model/repair/test counters remain distinct;
- measured usage aggregates correctly;
- missing usage remains unknown/null;
- fallback receipt is referenced by artifact path and receipt SHA rather than copied as authority;
- required cloud handoff is distinct from materialized fallback handoff;
- stream observer returns the exact underlying result object;
- stream observer re-raises underlying runtime errors and does not invent usage.

Targeted local verification result: **PASS**.

This is not a substitute for repository-native CI, live Ollama execution, or the repository reviewer gate.

## Branch contrast

The branch comparison against `feature/p2p-mvp-core` showed only documentation and `scripts/local-agent` additions/modification to the ADR index.

At the point of comparison there were no changed files under:

- `crates/p2p/**`
- `apps/availability-node/**`
- `mobile/src/p2p/**`
- P2P persistence/domain/runtime paths

The branch was ahead of the baseline and not behind it.

## CI / reviewer status

GitHub combined-status lookup for the branch head returned no attached status contexts at review time.

Therefore:

- repository CI is **not claimed PASS**;
- the Gemma/cross-vendor repository reviewer gate is **not claimed PASS**;
- live Ollama facade execution on the target Mac is **not claimed executed** in this implementation session.

These remain explicit verification items rather than fabricated evidence.

## Architecture conclusion

The delivered Wave 1 implementation matches the v3.2 principle:

> Centralize contracts and evidence; preserve existing behavior authorities.

The additive facade intentionally avoids making itself the default runner until repository-native/live-runtime verification demonstrates equivalence and value.
