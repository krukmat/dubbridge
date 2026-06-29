---
type: TaskList
title: "Tasks: Gemma local review consolidation and D14 fallback narrowing"
status: complete
plan: docs/plan/gemma-review-discrepancy-triage.md
---

# Tasks: Gemma local review consolidation and D14 fallback narrowing

> **Plan:** `docs/plan/gemma-review-discrepancy-triage.md`
> **Status:** Complete — T0/T1/T2/T3 Done 2026-06-29. `make qa-docs`
> remains non-zero because `qa-gemma-review` reports advisory findings requiring
> disposition; deterministic docs checks pass.
> **Execution note:** RRI must be computed per task before execution. Approval is
> required whenever the computed band requires it.

## Task summary

| ID | Title | Effort | Status | Depends on |
|---|---|---|---|---|
| T0 | Contract capture and handoff framing | S | Done | — |
| T1 | Consolidate Gemma review output and narrow D14 fallback | L | Done | T0 |
| T2 | Workflow and evidence wording sync | S | Done | T1 |
| T3 | Verification and closeout | S | Done | T1, T2 |

## T0 — Contract capture and handoff framing

- **Status:** Done — 2026-06-29
- **Effort:** S
- **Depends on:** —

### Objective

Record the clarified rule so the next implementation instance starts with the
correct contract rather than the previous disagreement-triggered D14 behavior.

### Acceptance criteria

- The plan and task ledger state that Gemma remains primary, uses 3 passes, and
  hands one consolidated package to the developer.
- The plan and task ledger state that D14 is fallback-only when Gemma does not
  produce usable output.
- The next instance can start implementation from this ledger without needing to
  reconstruct the policy from chat history.

### Agent handoff prompt

`T0 — Confirm the documented contract, then move to T1. Do not implement the workflow change yet.`

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-29
- Statement: Contract clarified and captured before T1: Gemma keeps the 3-pass default, the passes are compiled into one developer-review packet, duplicates are consolidated, and D14 is fallback-only when no usable local review result exists.

---

## T1 — Consolidate Gemma review output and narrow D14 fallback

- **Status:** Done — 2026-06-29
- **Effort:** L
- **Depends on:** T0
- **RRI:** 51 (Med-high)
- **Affected:** `scripts/gemma-code-review.py`, `scripts/adjudicator-packet.py`,
  `scripts/parse-review-findings.py`, `scripts/gemma_code_review_test.py`,
  `scripts/adjudicator_packet_test.py`, `scripts/parse_review_findings_test.py`

### Objective

Change the review workflow so Gemma's multi-pass output is consolidated into one
developer-review package and disagreement buckets stop triggering D14 by
themselves.

### Inputs

- Current D14 trigger logic in `scripts/adjudicator-packet.py`
- Current consolidated finding output in `scripts/parse-review-findings.py`
- The contract recorded in the linked plan

### Outputs

- Updated multi-pass aggregate behavior
- Updated trigger logic
- Updated consolidated review output
- Updated script tests proving the new behavior

### Acceptance criteria

- D14 no longer triggers from `pass_specific`, `location_inconsistent`, or
  `severity_inconsistent` alone.
- D14 still triggers when Gemma is unavailable, stalls, returns invalid output,
  or otherwise fails to produce a usable review result.
- The old quorum/degraded gate is removed from the multi-pass aggregate path:
  one or more parseable passes produce a usable aggregate; zero parseable passes
  fails closed.
- The developer-facing output still includes every reconciled bucket instead of
  hiding disagreement.
- Duplicate findings are consolidated into one developer-review entry with
  source buckets preserved.
- The updated tests cover both the new non-trigger path and the fallback path.

### Happy paths considered

- **HP-1:** Gemma completes all 3 passes and produces disagreement buckets; the
  result is emitted as one consolidated review package and no D14 trigger fires.
- **HP-2:** Gemma completes all 3 passes and produces consensus findings; those
  findings remain visible in the consolidated package for developer disposition.
- **HP-3:** Gemma completes all 3 passes with no findings; the output remains
  clean and D14 is not invoked.

### Edge cases considered

- **EC-1:** Gemma is unavailable, stalls, or produces invalid/unparseable output;
  D14 fallback still fires.
- **EC-2:** Disagreement and consensus findings coexist; all findings remain in
  the package, but disagreement alone does not force D14.
- **EC-3:** A finding appears in multiple reconciliation buckets; the package
  still reports it clearly enough for developer review.
- **EC-4:** Existing tests that assume disagreement-triggered D14 fail and must
  be updated in the same change.

### Agent handoff prompt

`T1 — Update the trigger/reporting scripts and their tests so Gemma 3-pass output becomes a single developer-review packet and D14 is used only when Gemma fails. Stop after tests pass; do not start doc sync.`

### Closure gate evaluation

- Development task: yes.
- RRI band: 51 (Med-high).
- Gemma Reviewer / D14 closure gate: not applicable under the current
  `AGENT_WORKFLOW_GUIDE.md` Step 1, which mandates Gemma Reviewer / D14 closure
  review for RRI 0-40 development tasks only.
- Required closure gates for this task: Reflection log (3 passes), unit coverage
  certification, owner final verification, and targeted script verification.

### Closure note

- `scripts/gemma-code-review.py` now writes a multi-pass aggregate when at least
  one configured pass is parseable. Zero parseable passes remains the fail-closed
  no-usable-result path.
- `degraded` and quorum gating were removed from the produced aggregate and audit
  record for this path. `passes_succeeded` remains as factual telemetry.
- `scripts/adjudicator-packet.py` now triggers D14 only when the aggregate is
  missing, empty, `BLOCKED`, or the caller marks Gemma output unusable.
- `scripts/parse-review-findings.py` now compiles one developer-review packet,
  deduplicates repeated findings, and preserves source buckets.
- T1 scope expanded to include `scripts/gemma-code-review.py` and
  `scripts/gemma_code_review_test.py` because the old quorum gate lived in the
  producer, not only in the parser/trigger scripts.

### Reflection log

Required passes: 3 (`RRI 51` -> `Med-high`)

#### Pass 1

- Draft verdict: D14 trigger logic now matches the clarified contract: findings,
  severity, band, and disagreement do not escalate when a usable Gemma aggregate
  exists.
- Critique findings: The producer still enforced the old quorum rule, so the
  contract would be false if only parser/trigger scripts changed.
- Revisions applied: Included `scripts/gemma-code-review.py` in T1 and changed
  zero parseable passes to be the only multi-pass aggregate failure.

#### Pass 2

- Draft verdict: Developer-facing reporting compiles all buckets and removes
  `degraded` from text/JSON output.
- Critique findings: Top-level `findings[]` and `reconciliation.consensus` can
  describe the same finding, so simple bucket collection would duplicate entries.
- Revisions applied: Added exact-finding deduplication with `source_buckets`
  preserved for reviewer context.

#### Pass 3

- Draft verdict: Tests now cover the new fallback-only D14 rule, one-pass-usable
  aggregate behavior, zero-pass fail-closed behavior, and duplicate consolidation.
- Critique findings: The original `parse_review_findings_test.py` bucket test
  used identical findings across buckets, which now correctly collapsed under
  deduplication.
- Revisions applied: Updated that test to use distinct findings and added a
  separate duplicate-consolidation test.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | 3-pass output with disagreement buckets stays local and does not trigger D14 | `scripts/adjudicator_packet_test.py::TestShouldAdjudicateDisagreement::test_no_fire_both_disagreement_types`; `scripts/parse_review_findings_test.py::TestCollectFindings::test_reconciliation_buckets_all_collected` | passed |
| HP-2 | Happy path | Consensus findings remain visible for developer disposition without D14 escalation | `scripts/adjudicator_packet_test.py::TestShouldAdjudicateConsensus::test_no_fire_consensus_major_moderate_band`; `scripts/gemma_code_review_test.py::MultiPassCli::test_consensus_finding_in_aggregate` | passed |
| HP-3 | Happy path | 3 clean passes produce clean output and do not invoke D14 | `scripts/adjudicator_packet_test.py::TestShouldAdjudicateNoFire::test_low_band_pass_no_findings`; `scripts/gemma_code_review_test.py::MultiPassCli::test_hp1_three_of_three_pass_exit_zero` | passed |
| EC-1 | Edge case | No usable Gemma aggregate triggers fallback / fails closed | `scripts/adjudicator_packet_test.py::TestShouldAdjudicateGemmaBlocked::test_fires_missing_aggregate_without_explicit_flag`; `scripts/gemma_code_review_test.py::MultiPassCli::test_zero_of_three_parseable_fails_no_aggregate`; `scripts/gemma_code_review_test.py::ReconcileUnit::test_aggregate_status_blocked_when_all_parseable_passes_blocked` | passed |
| EC-2 | Edge case | Disagreement and consensus findings coexist in the package without disagreement-driven D14 | `scripts/parse_review_findings_test.py::TestCollectFindings::test_reconciliation_buckets_all_collected`; `scripts/adjudicator_packet_test.py::TestShouldAdjudicateDisagreement::test_no_fire_severity_inconsistent_low_band` | passed |
| EC-3 | Edge case | Duplicate findings across buckets consolidate while preserving source buckets | `scripts/parse_review_findings_test.py::TestCollectFindings::test_duplicate_findings_consolidated_with_source_buckets` | passed |
| EC-4 | Edge case | Tests that assumed disagreement, major findings, or band alone triggered D14 are updated to the new contract | `scripts/adjudicator_packet_test.py::TestShouldAdjudicateConsensus::test_no_fire_consensus_blocking_low_band`; `scripts/adjudicator_packet_test.py::TestShouldAdjudicateBand::test_no_fire_med_high_band_no_findings`; `scripts/adjudicator_packet_test.py::TestShouldAdjudicateDisagreement::test_no_fire_location_inconsistent_low_band` | passed |

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-29
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior. T1 removes quorum/degraded gating from the local multi-pass review path, consolidates duplicate developer-facing findings, and keeps D14 as fallback only when Gemma produces no usable local review result.
- Commands run: `python3 scripts/adjudicator_packet_test.py` (49/49 pass), `python3 scripts/parse_review_findings_test.py` (17/17 pass), `python3 scripts/gemma_code_review_test.py` (54/54 pass)

---

## T2 — Workflow and evidence wording sync

- **Status:** Done — 2026-06-29
- **Effort:** S
- **Depends on:** T1
- **RRI:** 9 (Low)
- **Affected:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `docs/policies/HITL_AUTONOMY_POLICY.md`

### Objective

Synchronize the written workflow with the script behavior so the repository no
longer documents disagreement-triggered D14 when the code has moved to
developer-packet consolidation.

### Acceptance criteria

- The workflow guide says Gemma's configured passes are consolidated for
  developer review.
- The workflow guide says D14 is fallback-only when Gemma does not produce
  usable output.
- Any policy wording that still implies disagreement-triggered D14 is updated or
  removed.

### Agent handoff prompt

`T2 — Sync workflow wording to match the implemented Gemma/D14 behavior. Do not begin final closeout.`

### Closure gate evaluation

- Task type: docs/policy-only.
- Gemma Reviewer / D14 closure gate: exempt under `AGENT_WORKFLOW_GUIDE.md`
  because this task does not modify product/runtime code.
- Unit coverage certification: exempt because this is docs/policy-only work.

### Closure note

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` now describes Gemma's configured
  passes as one consolidated developer-review packet.
- The workflow guide now states that duplicate findings are consolidated, source
  buckets are preserved, and disagreement buckets are review metadata rather
  than D14 triggers.
- The D14 trigger table and development closure checklist now route to D14 only
  when Gemma is unavailable, stalls, returns invalid output, returns `BLOCKED`,
  or no usable consolidated result is produced.
- `docs/policies/HITL_AUTONOMY_POLICY.md` now matches the fallback-only language
  and no longer describes a quorum threshold.
- `Effort` was corrected from M to S to match the computed RRI Low band.

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-29
- Statement: I verified the workflow guide and HITL policy now match the T1 script behavior: no quorum gate, no disagreement-triggered D14, consolidated local review output when usable, and D14 fallback only for unusable Gemma output.
- Commands run: `python3 scripts/rri.py --C 0 --touches docs/playbooks/AGENT_WORKFLOW_GUIDE.md --touches docs/policies/HITL_AUTONOMY_POLICY.md --T 0 --A 0 --X 2 --D 0 --K 1 --P 1` (RRI 9), `rg -n "quorum|degraded|<2 passes|2/N|severity_inconsistent_count > 0|location_inconsistent_count > 0|Consensus blocking|Band ≥|Med-high.*trigger|passes run / succeeded" docs/playbooks/AGENT_WORKFLOW_GUIDE.md docs/policies/HITL_AUTONOMY_POLICY.md` (only the intended "there is no quorum gate" wording remains)

---

## T3 — Verification and closeout

- **Status:** Done — 2026-06-29
- **Effort:** S
- **Depends on:** T1, T2
- **RRI:** 5 (Low)

### Objective

Run the targeted checks, record evidence, and leave the workflow change ready
for a later merge/review step.

### Acceptance criteria

- Targeted script tests pass.
- The ledger records what changed and what still needs human approval or follow-up.
- `make qa-docs` is either run successfully or explicitly deferred with the
  reason recorded.

### Agent handoff prompt

`T3 — Run verification, record the evidence, and stop. Do not begin unrelated workflow or product changes.`

### Closure gate evaluation

- Task type: verification/docs closeout.
- Gemma Reviewer / D14 closure gate: exempt as docs/verification-only work.
- Unit coverage certification: exempt because this task does not modify
  development behavior beyond the already-certified T1 script changes.

### Verification evidence

- `python3 scripts/adjudicator_packet_test.py`: passed (49/49).
- `python3 scripts/parse_review_findings_test.py`: passed (17/17).
- `python3 scripts/gemma_code_review_test.py`: passed (54/54).
- `python3 scripts/check_okf_frontmatter.py`: passed after removing invalid
  non-ADR `governed_by` refs from this plan/ledger frontmatter.
- `bash scripts/check-task-unit-coverage.sh`: passed.
- `bash scripts/check-roadmap-drift.sh`: passed.
- `bash scripts/check-doc-consistency.sh`: passed after adding the missing
  ADR-035 index row to `docs/adr/README.md`.
- `make qa-docs`: attempted. It failed at `qa-gemma-review` because Gemma
  returned two minor advisory findings, and `parse-review-findings.py`
  correctly exits non-zero when findings require disposition.

### Gemma qa-docs finding disposition

- Final finding 1: `scripts/gemma-code-review.py:625`, minor, `findings`,
  `consensus`. Disposition: accepted as advisory, no code change required. The
  implemented contract intentionally treats one or more parseable Gemma passes as
  a usable consolidated developer packet, removes `degraded`, and preserves
  `passes_succeeded` as telemetry. Unit tests and workflow docs cover 1/N usable
  output and zero-usable fallback.
- Final finding 2: `scripts/gemma_code_review.py:625`, minor,
  `likely_false_positive`, out-of-scope path variant. Disposition: no action.
  The referenced push-reviewer `quorum`/`degraded` code path is outside this
  plan unless a shared utility is directly reused, and the local closure workflow
  docs now explicitly describe 3/1 usable output with no quorum gate.
- Earlier finding: `scripts/adjudicator_packet.py:34` /
  `scripts/gemma-code-review.py:628`, minor, `pass_specific` /
  `likely_false_positive`. Disposition: accepted and repaired before final
  verification. `gemma-code-review.py` now propagates aggregate status `blocked`
  when all parseable passes return `BLOCKED`, while still producing a usable
  `pass` or `findings` aggregate when a blocked pass has a usable peer. Added
  unit coverage for both cases.

### Owner final verification

- Owner: Matias Kruk
- Date: 2026-06-29
- Statement: I verified the targeted script tests pass, the workflow/policy docs
  are synchronized with the implemented no-quorum contract, deterministic docs
  checks pass, and remaining `make qa-docs` non-zero output is from Gemma
  findings that have dispositions above.
- Commands run: `python3 scripts/rri.py --C 0 --touches docs/tasks/gemma-review-discrepancy-triage.md --touches docs/plan/gemma-review-discrepancy-triage.md --T 0 --A 0 --X 2 --D 0 --K 0 --P 0` (RRI 5), `python3 scripts/adjudicator_packet_test.py`, `python3 scripts/parse_review_findings_test.py`, `python3 scripts/gemma_code_review_test.py`, `make qa-docs` (failed at Gemma findings; disposition recorded), `bash scripts/check-doc-consistency.sh`, `bash scripts/check-task-unit-coverage.sh`, `bash scripts/check-roadmap-drift.sh`, `python3 scripts/check_okf_frontmatter.py`
