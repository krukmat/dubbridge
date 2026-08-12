---
type: Audit
title: "S-150-T2b-ii-b RRI evidence"
status: Active
---

# S-150-T2b-ii-b RRI evidence

**Task:** Atomic delivery claim and dispatch persistence
**Calculated:** 2026-08-12
**Changed paths:** `crates/db/src/translation_repo.rs`, new
`crates/db/src/translation_delivery_repo.rs`, `crates/db/src/lib.rs`, and new
`apps/api/tests/translation_delivery_repo_test.rs`.

## Calculation

```text
python3 scripts/rri.py --cc 8 --T 2 --A 0 --X 3 --D 3 --K 4 --P 4 \
  --touches crates/db/src/translation_repo.rs \
  --touches apps/api/tests/localization_repo_test.rs \
  --penalty auth_security --platform dubbridge
```

| Variable | Score | Basis |
|---|---:|---|
| C cyclomatic | 1 | The bounded create-or-reuse transaction has an estimated raw CC of 8. |
| F files | 1 | One repository module and its focused integration-test surface. |
| D domain | 3 | `crates/db` anchor floor. |
| T coverage | 2 | Adjacent real-PostgreSQL claim tests exist; the dispatch persistence contract is new. |
| A ambiguity | 0 | The child ledger supplies exact HP/EC cases, transaction ordering, and exclusions. |
| K coupling | 4 | One transaction coordinates target scope, claims, status, and the dispatch outbox. |
| P impact | 4 | The writer enforces a persisted project/asset/target ownership boundary before writes. |
| X context | 3 | Repository, candidate helper, outbox schema, and live PostgreSQL fixture are material context. |

**Penalties:** `auth_security` (+10). The exact persisted ownership selection
must fail closed before the first claim or outbox write.

**Final RRI:** `52` — **Med-high** — **Effort L**.

## Routing conclusion

The current `translation_repo.rs` is 542 lines and the existing
`localization_repo_test.rs` is 1,194 lines. Under the target-file size gate,
the child cannot be delegated to the local runner as written. The resolved
implementation route is cloud after the ADR-038 refinement/primary-receipt
gate. The exact approved task card plus Matias's 2026-08-12 execution confirmation
preauthorized the capability/risk takeover selection `gpt-5.6-sol` at `high` for
this packet; the task-local receipt records the enforced `CLOUD_REQUIRED` route.

## Task-local review override

Matias selected `muse-glimmer:30b-q4_K_M` as the phase-2 code-solution reviewer
on 2026-08-12. This replaces Gemma for phase 2 only; it does not alter the
already-passed Gemma phase-1 review, the RRI, HITL approval, Reflection count,
or D14 fallback protocol.
