---
type: Plan
title: "Gemma/Peer Review Evidence Artifact Gate"
status: in-progress
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

## GEG-2 — Evidence integrity hardening (2026-08-21)

> **Status: delivered 2026-08-21.** GEG-2a, GEG-2b, and GEG-2c are all `[x] Done`
> in `docs/tasks/gemma-evidence-artifact-gate.md`, each with its own Gemma
> phase-2 review receipt (`docs/audit/gemma-evidence/GEG-2{a,b,c}.json`),
> Reflection log, unit coverage certification, and owner verification. Two
> caveats a later reader must not lose: §6's `changed_paths` question was
> resolved **narrower** than the audit's C3 proposed (presence and
> non-emptiness, not scope intersection), and the receipt backfill for
> `S-230-T4a`–`T4k` (audit change C5) is **not** part of this group — it was
> deliberately sequenced after GEG-2a/2b so it would not reproduce D1/D2 at
> volume.

GEG-1 delivered the receipt, the ledger validator, and the typed overrides.
Auditing every `muse-glimmer:30b-q4_K_M` invocation
(`docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md`) then found that
the *emission* side of that gate is fail-open, and that the receipt cannot
attribute a reviewer at all. GEG-2 closes those three defects.

The audit's own headline is the framing: the model binding is not the problem
(78% productive, every failure loud and correctly escalated). The pipeline that
records its verdicts is.

### 4. `qa-gemma-review` must fail closed (defect D1 → GEG-2a)

`gemma-code-review.py` writes `--out` only on success and returns `3` without
writing when no pass is usable. The Makefile recipe terminates that conditional
with `;`, so execution continues past the failure, and
`parse-review-findings.py` then reads whatever still sits at the fixed,
task-unscoped `/tmp/dubbridge-gemma-review.json` — the **previous task's**
result — and exits `0`. A `PASS` receipt is minted for the current task id from
a different task's review.

This is not hypothetical: it fired on `S-230-T4l`. Precisely: the **result
JSON** it minted the receipt from carried T4k's `changed_paths`; the receipt
itself has no such key (the pre-GEG-2c schema had no field for it), so what
exposed the substitution was a human reading the result file, not the committed
artifact. That is the whole reason D3's validator cannot retroactively catch
this receipt — see `docs/tasks/gemma-evidence-artifact-gate.md` § GEG-2c
§ Design decision. It also composes badly with the known
`muse-glimmer` empty-response defect
(`docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md`), whose
characteristic failure takes exactly this path.

Three fixes, all in the recipe: capture the review command's exit status and
abort before `parse-review-findings.py` when it is non-zero; default
`GEMMA_REVIEW_RESULT` to a task-scoped path; and remove any pre-existing result
file before invoking, so a stale artifact can never be mistaken for a fresh one.

### 5. Receipts must name the reviewer that actually ran (defect D2 → GEG-2b)

The recipe hardcodes `"reviewer":"gemma"` regardless of the resolved model. 45
committed receipts say `gemma`; only 3 say `muse-glimmer`, and those were
hand-written because the Makefile cannot emit that value. Since band-routing
policy turns on *which* reviewer produced a verdict, the committed audit trail
currently cannot answer that question.

The sibling `qa-peer-workflow-review` target already does this correctly,
extracting the reviewer from the artifact. The blocker is upstream: the
aggregate result JSON carries no model field at all (keys are `changed_paths`,
`findings`, `format_warnings`, `status`, `summary`), so the recipe has nothing
to extract. `gemma-code-review.py` must record the resolved model in the
aggregate first; the recipe then reads it, following the peer target's pattern.

### 6. The closure gate must validate content, not just binding (defect D3 → GEG-2c)

Design §2 above deliberately scoped the validator to `task_id` + `commit_sha`,
and R4 flagged the residual replay risk. That residue is now load-bearing: the
validator never checks `verdict`, never checks `reviewer`, and never checks that
the receipt's `changed_paths` bear any relation to the task. D1's fabricated
receipt and D2's misattributed receipt both pass it unchallenged — the only
thing that caught `S-230-T4l` was a human noticing a path mismatch by eye.

`verdict` and `reviewer` are straightforward: require presence, require
`verdict` in the known set, require `reviewer` non-empty. **`changed_paths`
scope is an open design question and the main reason this subtask scores
Med-high**: the receipt's `commit_sha` is `HEAD` *at review time*, while the
reviewed content is the working-tree diff against it, so the receipt's paths
deliberately do not match that commit's file list. Resolving what the validator
can soundly assert — and whether the receipt schema needs to carry the paths at
all — is part of GEG-2c, not a settled premise.

**Resolved 2026-08-21 (GEG-2c).** Presence and non-emptiness, not set-equality:
equality against the receipt's own commit is unsound by construction, and
equality against declared task scope has no canonical machine-parseable source
in the ledger schema today. The check proves a real diff was bound to the
receipt; it cannot prove the *right* diff was. **D1's containment is therefore
GEG-2a's fail-close, not this validator** — the `S-230-T4l` receipt passes
GEG-2c either way. Full rationale, the verified T4l artifact contents, the
grandfather key, and the `qa-peer-workflow-review` consequence:
`docs/tasks/gemma-evidence-artifact-gate.md` § GEG-2c § Design decision.

### Sequencing and the bootstrap constraint

Strictly ordered: GEG-2a → GEG-2b → GEG-2c. GEG-2b cannot emit a reviewer the
aggregate does not record; GEG-2c cannot validate a field GEG-2b has not yet
written.

All three modify the review pipeline that their own phase-1/phase-2 reviews run
through. For GEG-2a in particular, reviewing the fix to the fail-open *via* the
fail-open is circular. Until GEG-2a lands, every review of this group must
delete the stale result file first and confirm the receipt's `changed_paths`
match the subtask's touched files before the verdict is accepted as evidence;
D14 is the fallback if that cannot be established.

**Implementation routing — owner override 2026-08-21.** GEG-1's local-first
routing note does **not** carry to GEG-2. The owner directed that the local
developer wrapper not be used for this group: these subtasks repair the
delegation and review tooling itself, and a mid-task communication failure in
that wrapper would corrupt evidence in precisely the way D1 does. The primary
agent authors the diffs directly. This overrides the standing
maximize-local-delegation directive for this group only; it changes nothing
about RRI, band, the review chain, Reflection counts, or the approval gate.

## Related documents

- `docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md` — the role-fitness
  audit that surfaced defects D1–D5; GEG-2 implements its changes C1–C3
- `docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md` — the
  empty-response defect (change C4) that composes with D1; not in GEG-2 scope
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`
- `docs/plan/portable-peer-review-gate.md` (band × phase routing this plan reuses)
- `docs/policies/RRI_POLICY.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `scripts/check-task-unit-coverage.sh`, `scripts/check-review-budget.py` (D14-OVERRIDE precedent)
