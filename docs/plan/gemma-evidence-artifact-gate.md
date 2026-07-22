---
type: Plan
title: "Gemma/Peer Review Evidence Artifact Gate"
status: proposed
slice: GEG
---

# Gemma/Peer Review Evidence Artifact Gate

> **Status:** Proposed — awaiting approval (RRI 48, Med-high band).
> **Origin:** Surfaced while auditing why `.githooks/pre-push` ran Gemma/peer
> review on every push (fixed separately, uncommitted, Option C in
> `Makefile`/`.githooks/pre-push`). That fix moved review to task closure and CI,
> which raised the follow-up question: how does the system actually know a task's
> recorded "Gemma Reviewer evidence" corresponds to a review that really ran?

## Purpose

`scripts/check-task-unit-coverage.sh` is the deterministic ledger gate that runs
in `make qa-docs` on every push and in CI. For a `Status: [x] Done`,
`Type: development` section it requires several evidence blocks, including
**Gemma Reviewer evidence** — but only checks that specific text lines are
present (`Command:`, `Quorum: met|failed`, `Primary-agent disposition:`). It
never verifies that a real review ran, and it only runs that check at all for
RRI ≤ 40. RRI ≥ 41 sections have **no evidence requirement whatsoever**, even
though ADR-034 §4 states review is "mandatory for all development tasks."

Two gaps, closed together because they share one mechanism:

1. **No execution proof.** The Gemma-evidence text block is self-reported by the
   closing agent. Nothing cross-checks it against `scripts/gemma-code-review.py`'s
   actual output — which is itself weak evidence even when it exists: `--out` is
   optional, never written on `--dry-run`, skipped entirely under
   `DUBBRIDGE_SKIP_GEMMA_REVIEW=1` or the no-code-changes early exit, and even
   when written lands in an ephemeral `/tmp` path with no binding to the reviewed
   commit.
2. **No RRI ≥ 41 gate at all.** Med-high/Complex work — the band PPR routes to
   cross-vendor peer review instead of Gemma — has no ledger-level check that any
   review evidence, of either kind, was recorded.

The central invariant being added:

> A `Status: [x] Done`, `Type: development` section, at any RRI, must carry
> either a verifiable review-artifact reference or an explicit, typed,
> attributable override. Silence is a failure, not a pass.

## Design

### 1. A small, committed receipt — not the `/tmp` JSON, not the audit log

`logs/gemma-audit/YYYY-MM.jsonl` (ADR-034) already has real per-invocation
records (`task_id`, `rri`, `outcome`, `disposition`) but is git-ignored and
local-only by design — it cannot be the portable source for a check that must
also pass in CI and on a fresh clone. `scripts/gemma-code-review.py`'s `--out`
JSON is similarly not portable (ephemeral path, optional, not commit-bound).

This plan adds a third, deliberately minimal artifact: when `make
qa-gemma-review` or `make qa-peer-workflow-review` is invoked with a task id
(`GEMMA_REVIEW_TASK_ID=<id>` / `PEER_REVIEW_TASK_ID=<id>`, the latter already
exists), the wrapper additionally writes a **receipt** —
`{task_id, commit_sha, reviewer, verdict, timestamp}` — to
`docs/audit/gemma-evidence/<task_id>.json`. It is small (a few hundred bytes),
committed alongside the closing commit, and does not change what ADR-034
already decided about the audit log or the existing ephemeral output.

### 2. Ledger check verifies the receipt, not the claim

`check-task-unit-coverage.sh` gains a validator, applied to every completed
development section regardless of RRI band, that requires a `Review artifact:`
line pointing at the receipt file, then parses it and checks `task_id` matches
the section and `commit_sha` is reachable from the reviewed history. A mismatch
or missing file fails the gate the same as missing evidence today.

### 3. A typed, attributable override — extending the existing `D14-OVERRIDE` grammar

Not every closure can produce a receipt, and the design must not silently
degrade into "nobody can ever close an urgent fix." Three named exceptions,
each requiring a companion field the checker also validates so the override is
a citation, not a checkbox:

| Override type | Companion field required | What it means |
|---|---|---|
| `urgency` | `Waiver-by: <human name>` | Expedited closure, human-authorized (never self-issued by the agent), per `HITL_AUTONOMY_POLICY.md`. |
| `not-applicable` | `Scope-note: <why>` | The Done+development section legitimately has no reviewable diff (e.g. pure deletion, generated-file sync). |
| `pipeline-failure` | `Failed-attempt: <evidence>` | Gemma/peer infra was attempted and unavailable — cites the failed run, not just an assertion. |

Every accepted override is also required to appear as a row in a new,
append-only, committed ledger, `docs/audit/gemma-review-overrides.md` — so
exceptions stay visible in one place for periodic review instead of scattering
silently across hundreds of individual task files. This reuses the spirit of
the existing `D14-OVERRIDE: <reason>` pattern in `scripts/check-review-budget.py`
rather than inventing an unrelated fourth escape hatch in the repo.

## Non-goals

- Does not change ADR-034's decision that `logs/gemma-audit/` stays git-ignored
  and local-only.
- Does not remove or weaken the existing `/tmp` ephemeral review output.
- Does not change the PPR band-routing rule (RRI 0–40 → Gemma, RRI 41+ →
  cross-vendor peer, D14 fallback) — the receipt is reviewer-agnostic and is
  written by whichever reviewer ran.
- Does not retroactively invalidate historical Done sections; see the
  grandfather clause in the paired task file's acceptance criteria.

## Risks

- **R1 — Corpus break.** Applying the new rule to all historical
  `docs/tasks/*.md` Done sections at once would fail the gate on entries written
  before this plan existed. Mitigate with a cutover date/commit; only sections
  closed after it must satisfy the new rule.
- **R2 — Override abuse.** A self-service override that's too easy to invoke
  reopens the same silent-bypass problem this plan closes. Mitigate with the
  mandatory `Waiver-by` (human, not agent) for `urgency` and the committed,
  greppable overrides ledger that makes abuse visible on inspection.
- **R3 — CI portability.** The receipt must be committed by whoever closes the
  task (locally, with `GEMMA_REVIEW_TASK_ID` set), not generated by CI itself,
  since CI has neither the audit log nor guaranteed model access. To confirm
  during implementation.
- **R4 — Replay risk.** A valid receipt from an earlier commit could be cited
  against a materially different later diff if only `task_id` is checked.
  `commit_sha` binding is required in the receipt schema; the exact reachability
  check (equality vs. ancestry) is an implementation decision, not fixed here.

## Related documents

- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`
- `docs/plan/portable-peer-review-gate.md` (band × phase routing this plan reuses)
- `docs/policies/RRI_POLICY.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `scripts/check-task-unit-coverage.sh`, `scripts/check-review-budget.py` (D14-OVERRIDE precedent)
