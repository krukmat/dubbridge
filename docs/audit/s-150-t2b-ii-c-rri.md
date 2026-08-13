---
type: Audit
title: "S-150-T2b-ii-c RRI evidence"
status: Active
---

# S-150-T2b-ii-c RRI evidence

**Task:** Guarded dispatch enqueue-failure transition
**Calculated:** 2026-08-13
**Paths:** `crates/db/src/translation_delivery_repo.rs`,
`apps/api/tests/translation_delivery_repo_test.rs`

## Calculation

```text
python3 scripts/rri.py --cc 5 --T 1 --A 2 --X 2 --D 3 --K 3 --P 3 \
  --touches crates/db/src/translation_delivery_repo.rs \
  --touches apps/api/tests/translation_delivery_repo_test.rs
```

| Variable | Score | Basis |
|---|---:|---|
| C | 0 | Bounded conditional transition; estimated raw CC 5. |
| F | 1 | One production module and one focused integration suite. |
| D | 3 | `crates/db` anchor floor. |
| T | 1 | Focused live-PostgreSQL suite already exists. |
| A | 2 | The prior `active` wording conflicts with the persisted `pending` state and required an explicit interpretation. |
| K | 3 | PostgreSQL state transition scoped by a composite delivery identity. |
| P | 3 | Internal persisted delivery state and ownership identity. |
| X | 2 | Delivery module, outbox migration, and focused test fixture. |

**Penalties:** none.
**Final RRI:** **35 — Moderate — Effort M.**

## Resolved transition contract

- `pending` is the sole eligible persisted source state and changes to
  `enqueue_failed` with the supplied error detail.
- A repeat against an already `enqueue_failed` row returns `AlreadyFailed` and
  does not overwrite `error_detail` or `updated_at`.
- `acknowledged`, absent, stale-generation, and identity-mismatched rows never
  transition. The public result distinguishes `Marked`, `AlreadyFailed`, and
  non-transition outcomes.
- The repository disposition `Active` is an API classification for persisted
  `pending`; it is not a distinct stored state and does not alter this guard.

## Routing conclusion

RRI 35 uses the Moderate route: explicit HITL approval, then the local-first
implementer `qwen3.6:35b-a3b` in a disposable worktree.
Both full-read paths are below the 500-line local target-file threshold at
presentation time (189 and 398 lines). Cloud takeover is
`gpt-5.6-terra` at `medium` if the local runner is unavailable, its scope gate
fails, or its two evidence-backed repair attempts are exhausted.
