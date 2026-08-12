---
type: Audit
title: "S-150-T2b-ii-a RRI evidence"
status: Active
---

# S-150-T2b-ii-a RRI evidence

**Task:** Candidate delivery-scope query and decoding helpers
**Calculated:** 2026-08-12
**Planned changed paths:** `crates/db/src/target_language_repo.rs`,
`apps/api/tests/delivery_scope_repo_test.rs`

## Calculation

```text
python3 scripts/rri.py --cc 5 --T 4 --A 0 --X 2 --D 3 --K 3 --P 3 \
  --touches crates/db/src/target_language_repo.rs \
  --touches apps/api/tests/delivery_scope_repo_test.rs \
  --platform dubbridge
```

| Variable | Score | Basis |
|---|---:|---|
| C cyclomatic | 0 | One bounded query/mapping helper; raw CC 5. |
| F files | 1 | Two planned paths. |
| D domain | 3 | `crates/db` anchor floor. |
| T coverage | 4 | There is no direct test for the new delivery-scope contract yet. |
| A ambiguity | 0 | The decomposed ledger defines HP-1 and EC-1. |
| K coupling | 3 | PostgreSQL repository anchor floor. |
| P impact | 3 | Internal repository API: it only returns query candidates and makes no authorization decision or write. |
| X context | 2 | Target-language repository, a focused integration test, and the existing migration/ledger contract. |

**Penalties:** none. The helper does not take a caller-supplied `project_id`,
authorize access, or decide ownership: it decodes persisted candidates only.
`S-150-T2b-ii-b` owns exact-project selection and fail-closed enforcement inside
its writer transaction.

**Final RRI:** `39` — **Moderate** — **Effort M**.

## Routing conclusion

This is a local-first task. The `target_language_repo.rs` target is 164 lines and
the integration test is new, so every full-read path is within the 500-line local
delegation guard. The approved route is
`scripts/local-agent/run_local_task.py` with
`nemotron-3.5-lightning:30b-a3b-q4_K_M`, allowing at most two evidence-backed
repairs; only an unavailable local route, scope violation, or exhausted repair
budget may take the recorded cloud fallback.

## Phase-1 task-analysis review

The previous RRI 49 / phase-1 PASS was superseded before approval because the
proposed helper accepted a caller-supplied project and therefore mixed candidate
decoding with an ownership boundary. The replanned RRI 39 requires a new phase-1
review before presentation.

`Task-analysis review: gemma .agent/peer-task-review-S-150-T2b-ii-a-v2.json - PASS`

Gemma reviewed the local-first scope and returned `PASS` with no findings.
