---
type: Audit
title: "Muse Glimmer role-fitness review and review-evidence integrity findings"
status: open
---

# Muse Glimmer role-fitness review and review-evidence integrity findings

**Date:** 2026-08-21 · **Scope:** every recorded `muse-glimmer:30b-q4_K_M`
invocation in this repository, across both roles it holds
(band-routed Reviewer; ADR-037/ADR-038 Local Architect refinement).

**Question asked:** how often was Muse Glimmer productive vs. erroneous or
non-responsive, and is it worth keeping in its current role?

**Answer in one line:** the model's own hit rate (78% productive) does not
justify replacing it, but auditing it exposed **five verified defects in the
review-evidence pipeline itself** — including a fail-open path that fabricates
`PASS` receipts — which matter considerably more than the model binding does.

## 1. Correction to the first-pass analysis

Two claims from the initial pass are corrected here; both were wrong in ways
that mattered.

| First-pass claim | Corrected finding |
|---|---|
| Counts partly derived from `docs/audit/gemma-evidence/*.json` receipts | **Receipts cannot attribute a reviewer at all** — `make qa-gemma-review` hardcodes `"reviewer":"gemma"` (D2 below). Counting must rest on ledger prose only. |
| "Prioritise the `think:false` fix" | **Wrong priority.** The think-flag defect fails *loudly* (empty content, visible, forces D14). D1 below fails *silently and open*, fabricating `PASS` evidence. D1 outranks it. |

A third point was missed entirely in the first pass and is material: two of the
"successful" reviews only passed because the packet was **trimmed** (§2.2).

## 2. Measurement

### 2.1 Reviewer role (phase-1 / phase-2, band-routed)

Counted at **task-phase granularity** from ledger prose, the only trustworthy
source given D2. 18 invocations.

| Outcome | Count | Tasks |
|---|---|---|
| Productive, clean | 12 | `S-230-T4a/b/c/d/f/h/i/j/k`, `local-model-stack-restructure T4c`, `S-150-T2b-ii-b`, `S-150-T2c-iv-b-debt-1` |
| Productive, only after resource-recovery on a **trimmed** packet | 2 | `S-230-T4e`, `S-230-T4g` |
| Erroneous / no response | 4 | `S-230-T4l`, `LRPC-2`, `LRPC-3`, `NEM-SM-T2` |

**14/18 = 78% productive · 4/18 = 22% failed.**

Failure detail:

- **`LRPC-2`** — 3/3 passes returned `done_reason: length` with empty content;
  escalated to D14. Root-caused in
  `docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md`.
- **`NEM-SM-T2`** — empty response as intermediate fallback; escalated to D14.
- **`S-230-T4l`** — never completed; user terminated the 3-pass/30B run as
  excessively slow, then switched to Gemma. Latency-driven, not a model error
  per se — but see D1, because the interrupt is what triggered the fail-open.
- **`LRPC-3`** — not a model failure but a **chain misconfiguration**: ran as
  band primary when policy makes it the intermediate fallback, and returned
  **self-contradictory verdicts on the identical packet** (pass 1 `BLOCKED`,
  6 findings; pass 2 `PASS`, 0 findings). Superseded by a Gemma re-run;
  recorded in `docs/audit/gemma-evidence/LRPC-3-phase1.json`.

Counter-evidence worth keeping in view: Muse Glimmer produced **genuine
blocking findings that were accepted and fixed** on `S-230-T4g` (2 rounds),
`S-230-T4i`, and `S-230-T4j` — under-specified delegation packets that would
otherwise have reached the local developer. Every recorded blocking finding was
dispositioned as *accepted and corrected*, never overridden as a false
positive. The role is doing real work when it answers.

### 2.2 Review-coverage degradation (missed in the first pass)

`S-230-T4e` and `S-230-T4g` are recorded as `PASS`, but both only completed
after the local resource-recovery protocol shrank the packet:

- `S-230-T4e` — phase 1 and phase 2 both re-run on trimmed packets
  (`~30 lines`; `172 lines`) after the originals stalled
  (`docs/tasks/s-230-poc-v1-digitalocean.md:2184-2194`).
- `S-230-T4g` — **phase-2 code-solution review ran on a reduced excerpt**
  (file header + one function reference + insertion point) *instead of the
  full 378-line file*
  (`docs/tasks/s-230-poc-v1-digitalocean.md:2461-2463`).

This is a **review-quality** finding, not an availability one: a `PASS` issued
against a fraction of the diff is materially weaker evidence than a `PASS`
against the whole diff, and nothing in the evidence block distinguishes the
two. The 30B memory footprint is what forces the trimming.

### 2.3 Local Architect refinement role (ADR-037 / ADR-038)

6 invocations, **6/6 technically successful** — no empty responses, valid
schema-conformant JSON, coherent scope/risk/stop-condition output:
`LRPC-3`, `LRPC-4`, `LRPC-5`, `LRPC-6`, `S-230-T1`,
`module-split-gate-tooling-T1`.

2/6 (`LRPC-6`, `module-split-gate-tooling-T1`) recommended `GO_LOCAL` and were
downgraded to cloud by the primary. That is not an error — ADR-038 explicitly
permits downgrade and forbids upgrade — but it is a mild optimism bias worth
noting.

**This role shows no reliability problem.** The failures are confined to the
Reviewer role, and specifically to large real review packets, which is exactly
what the think-flag incident predicts.

## 3. Verified defects requiring change

All five were confirmed against the code, not inferred. Empirical tests are
noted where run.

### D1 — Stale-result fail-open fabricates `PASS` receipts *(critical)*

`Makefile:118-131`.

`gemma-code-review.py` writes `--out` **only after all passes complete**
(`scripts/gemma-code-review.py:665-667`), and returns `3` without writing when
no pass is usable (`:652-659`). The Makefile invokes it as
`... --out "$(GEMMA_REVIEW_RESULT)" - && echo ...;` — the trailing `;`
terminates the conditional, so **execution continues regardless of failure**.

`GEMMA_REVIEW_RESULT` defaults to the fixed, task-unscoped path
`/tmp/dubbridge-gemma-review.json` (`Makefile:11`).

Consequence chain, on any interrupt or unusable run:

1. `--out` is never written → the **previous task's** result file survives intact.
2. `parse-review-findings.py` reads that stale file. **Empirically verified:**
   a stale successful result returns `status: pass, findings: none`, **exit 0**.
3. `findings_status=0` → `verdict="PASS"` → a receipt is written **for the
   current task ID**, sourced from a different task's review.
4. `check-task-unit-coverage.sh` accepts it (see D3).

This is not hypothetical — **it fired on `S-230-T4l`**, whose ledger records a
receipt "containing content from an unrelated prior task's diff (T4k's paths,
not T4l's)" and attributes it to a "suspected stale/overlapping process, root
cause not fully isolated"
(`docs/tasks/s-230-poc-v1-digitalocean.md:3395-3410`). **The root cause is now
isolated:** it is this deterministic control-flow bug, not a race.

Live proof at the time of writing: `/tmp/dubbridge-gemma-review.json` still
holds T4l's run from `Aug 21 10:10`, with
`changed_paths: ["scripts/test-production-images.sh"]` — armed to be misread by
the next task that fails or is interrupted.

**Why this is the top finding:** Muse Glimmer's *documented* failure mode
(all passes empty → `return 3` → no write) lands **directly** on this path. The
two defects compose into silent evidence fabrication in a repository whose
stated core principle is fail-closed (ADR-008, ADR-018).

### D2 — Receipts misattribute the reviewer *(high)*

`Makefile:127` writes `"reviewer":"gemma"` **unconditionally**, whatever
`DUBBRIDGE_REVIEW_MODEL` resolved to. The sibling `qa-peer-workflow-review`
target does this correctly, extracting the real reviewer from the artifact
(`Makefile:199`).

Verified blast radius — ledger says Muse Glimmer ran, receipt says `gemma`:

| Task | Ledger `Model:` | Receipt `reviewer` |
|---|---|---|
| `S-230-T4k` | `muse-glimmer:30b-q4_K_M` | `gemma` |
| `T4a` | `muse-glimmer:30b-q4_K_M` | `gemma` |
| `T4b` | `muse-glimmer:30b-q4_K_M` | `gemma` |

Across all receipts: **45 say `gemma`**, only 3 say `muse-glimmer` (and those 3
appear hand-written — the Makefile cannot emit that value). Since band-routing
policy depends on knowing *which* reviewer produced a verdict, the committed
audit trail cannot currently answer that question.

**Blocked on a prerequisite:** the aggregate result JSON carries **no model
field** — verified keys are `changed_paths`, `findings`, `format_warnings`,
`status`, `summary`. The Makefile has no source to extract from, which is
presumably why it hardcodes. Fixing D2 requires first recording the resolved
model in the aggregate.

### D3 — Closure gate validates neither verdict, reviewer, nor scope *(high)*

`scripts/check-task-unit-coverage.sh:217-231` validates only that the receipt
exists, is valid JSON, its `task_id` matches the section, and its `commit_sha`
is reachable. It **never** checks `verdict`, **never** checks `reviewer`, and
**never** checks that `changed_paths` intersects the task's touched files.

So D1's fabricated receipt and D2's misattributed receipt both pass the gate
unchallenged. The only thing that caught T4l was a human noticing a
`changed_paths` mismatch by eye.

### D4 — The think-flag remedy was never scoped as a task *(medium)*

`docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md` is still
`status: open`, carries concrete remediation, and is referenced by exactly one
ledger (`local-role-prompt-canonicalization.md`). Verified: **no `/no_think`
directive exists anywhere in `scripts/`** — every role still relies solely on
the API-level `think` flag (`scripts/gemma_local.py:172`), which this model
demonstrably ignores under real packets. No task or plan implements the fix.

### D5 — The active slice is outside the enforcement marker *(medium)*

`docs/tasks/s-230-poc-v1-digitalocean.md` does **not** declare
`Behavioral coverage contract: unit-v1`, so `make qa-docs` does not enforce the
review-artifact gate on **any** S-230 task, including all 17 T4 children. The
4111-line ledger contains exactly **one** `Review artifact:` line, and receipts
exist for only `T4k`/`T4l` out of `T4a`–`T4l`.

The 11 Muse Glimmer reviews in the T4 chain therefore closed on **self-reported
prose alone**, with no committed receipt and no automated check. The prose is
detailed and credible (PIDs, packet files, verdicts, findings) — but it is
unverifiable by automation, which is precisely what the GEG-1 gate exists to
prevent.

### D6 — The evidence gate accepts only Rust test references *(high, added 2026-08-21)*

Found while closing GEG-2, by deliberately enabling the marker on the GEG ledger
and checking that the gate actually fires — passing `make qa-docs` proves nothing
if the sections are not being read.

`scripts/check-task-unit-coverage.sh:107` matches unit-test evidence with
`` `[^`]+\.rs::[A-Za-z_][A-Za-z0-9_:]*` `` — the `\.rs` is literal. A Python or
shell test reference such as
`` `scripts/gemma_review_makefile_test.py::QaGemmaReviewMakefileTarget::test_ec1_...` ``
can **never** satisfy it. So a development task whose tests are not Rust cannot
pass the coverage certification at all, no matter how genuine its evidence is.

This is strictly worse than D5. D5 says a ledger can sit outside the marker; D6
says that even a ledger *inside* the marker fails if its work is Python or
shell — which is all of GEG-1, all of GEG-2, and every `scripts/` task. Enabling
the marker on `docs/tasks/gemma-evidence-artifact-gate.md` produced 32
violations across 4 sections, of which 24 were this single cause.

Two smaller defects surfaced in the same check and belong with it:

- **The `Review artifact:` line format in use does not match the validator.**
  `validate_review_evidence()` matches `^[[:space:]]*-?[[:space:]]*Review artifact:`,
  but every completed section in the GEG ledger writes
  `- **Review artifact:** …` with markdown bold, which that regex rejects. It has
  never surfaced because those sections are pre-cutover *and* the ledger is
  outside the marker — two independent reasons the check never ran. GEG-2's own
  sections were written in the bolded house style and corrected to the
  documented unbolded form at closure.
- **`Owner final verification` `Statement:` is matched per line.** A statement
  wrapped across two markdown lines fails even when its text is exactly right.
  GEG-1e's statement has this defect today.
- **The enforcement marker is detected by bare substring grep
  (`scripts/check-task-unit-coverage.sh:433`).** Any `docs/tasks/*.md` file that
  quotes the marker string verbatim — in documentation, an example, or a
  sentence saying a ledger does *not* declare it — silently opts that entire
  file into enforcement. This was found by writing exactly such a sentence and
  watching `make qa-docs` start failing.

**Not fixed here.** Widening the regex changes the evidence contract for the
whole `docs/tasks/` corpus and interacts with
`docs/policies/RRI_POLICY.md § Review evidence gate`; it is not inside GEG-2's
declared scope (D1/D2/D3). Recorded as change **C7** below.

### D7 — The gate's own ledger is outside the gate *(medium, added 2026-08-21)*

`docs/tasks/gemma-evidence-artifact-gate.md` — the ledger that specifies and
implements the review-evidence gate — does not declare
`Behavioral coverage contract: unit-v1`. None of its completed sections
(GEG-1a–1e, GEG-2a–2c) is enforced by `make qa-docs`. This is D5's shape applied
to the gate itself, and it is why D6 went unnoticed through GEG-1's entire
delivery: GEG-1e's closure certified "full-corpus regression passes clean",
which was true and simultaneously said nothing about its own sections.

Enabling the marker is blocked on D6 — the ledger's evidence is Python and shell
throughout, so opting in today would fail 32 checks for reasons unrelated to
evidence quality. Sequenced as change **C8**, after C7.

## 4. Required changes, prioritised

Each is stated as a proposed task. **RRI values are not computed here** —
`scripts/rri.py` must be run before any of these is presented for approval; the
band column is an unverified estimate for sequencing only.

| # | Change | Files | Est. band (unverified) | Status |
|---|---|---|---|---|
| **C1** | Make `qa-gemma-review` fail closed: abort before `parse-review-findings.py` when the review command fails; scope `GEMMA_REVIEW_RESULT` per task ID; delete/ignore any pre-existing result before invoking | `Makefile` | Low | ✅ landed 2026-08-21 as **GEG-2a** (RRI 38, Moderate) |
| **C2** | Record the resolved reviewer model in the aggregate result JSON, then extract it for the receipt instead of hardcoding `gemma` | `scripts/gemma-code-review.py`, `Makefile` | Low–Moderate | ✅ landed 2026-08-21 as **GEG-2b** (RRI 34, Moderate) |
| **C3** | Extend the closure validator to check `verdict`, `reviewer`, and `changed_paths` scope intersection | `scripts/check-task-unit-coverage.sh` | Moderate | ✅ landed 2026-08-21 as **GEG-2c** (RRI 52, Med-high) — **narrowed**, see below |
| **C4** | Implement the think-flag remedy: model-specific textual `/no_think` prepend + a distinct, logged failure class for `done_reason:length` with empty content (do not merge it into the generic "unavailable" bucket) | `scripts/gemma_local.py`, `scripts/gemma-code-review.py` | Moderate | ⬜ open |
| **C5** | Add `Behavioral coverage contract: unit-v1` to the S-230 ledger and backfill the missing T4 receipts, or record typed overrides | `docs/tasks/s-230-poc-v1-digitalocean.md`, `docs/audit/gemma-evidence/` | Low (docs) | ⬜ open — owner chose the **re-run** route over typed overrides; deliberately sequenced *after* C1/C2, since backfilling before them would reproduce D1/D2 at volume |
| **C6** | Require the evidence block to state when a review ran on a **trimmed/reduced packet**, so a degraded `PASS` is distinguishable from a full-coverage one | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | Low (docs) | ⬜ open |
| **C7** | Widen the coverage gate's test-reference regex beyond `.rs::` so Python/shell development tasks can satisfy it; match the `Review artifact:` line with or without markdown bold; match a `Statement:` wrapped across lines; anchor marker detection to a real declaration line instead of a bare substring grep (defect **D6**) | `scripts/check-task-unit-coverage.sh`, `scripts/check_task_unit_coverage_test.py`, `docs/policies/RRI_POLICY.md` | Moderate | ⬜ open |
| **C8** | Add `Behavioral coverage contract: unit-v1` to `docs/tasks/gemma-evidence-artifact-gate.md` and bring GEG-1e's legacy closure blocks up to the gate's format (defect **D7**) | `docs/tasks/gemma-evidence-artifact-gate.md` | Low (docs) | ⬜ open — blocked on C7 |

**C1 is the only one that should be treated as urgent.** Until it lands, every
interrupted or unusable review run can silently mint a `PASS` receipt for the
wrong task — and Muse Glimmer's characteristic failure mode is exactly the
trigger.

C1 and C2 both touch `qa-gemma-review` and should be sequenced together to
avoid two consecutive rewrites of the same recipe.

**C3 landed narrower than proposed — read this before assuming otherwise.**
This row proposed `changed_paths` **scope intersection**. GEG-2c deliberately
did not implement that: no canonical, machine-parseable declared-scope source
exists across the ledger corpus to intersect against, and `commit_sha` is `HEAD`
*at review time* while the reviewed content is the working-tree diff against it,
so intersecting with that commit's file list would fail every honest receipt.
What shipped is **presence and non-emptiness** — a receipt can no longer be
minted with no diff bound to it, but a receipt bound to the *wrong* diff is not
detected. **Containment for D1 is C1's fail-close, not C3's validator**, and
C3 does not retroactively catch the `S-230-T4l` artifact. Genuine diff↔task
correspondence remains an open follow-up. Full rationale and the verified T4l
artifact contents: `docs/tasks/gemma-evidence-artifact-gate.md` § GEG-2c
§ Design decision.

**Live confirmation of C1–C3.** The three closure reviews for GEG-2a/2b/2c ran
through the repaired pipeline and their receipts demonstrate all three fixes at
once: task-scoped result paths (C1), `reviewer` recording the model that
actually ran — `gemma4:26b-a4b-it-qat`, a value the pre-C2 recipe was
structurally incapable of writing — and non-empty `changed_paths` matching each
reviewed scope (C3). GEG-2b's two rounds produced `FINDINGS-ACKED` then `PASS`
from the same recipe, evidencing that verdict derivation discriminates rather
than defaulting.

## 5. Verdict on the role

**Trade-off, stated explicitly:**

- **Keeping it** — 78% productive, with genuine blocking findings caught and
  corrected on three separate tasks and zero recorded false positives. It is
  the only Low-band primary reviewer; removing it pushes that volume onto D14,
  which is a Balanced-tier adjudicator not designed for routine throughput.
  Cost: the think-flag defect keeps producing empty responses (~90–150s wasted
  per occurrence before fallback), the 30B footprint keeps forcing packet
  trimming that silently weakens coverage, and 3-pass latency is high enough
  that a user has already interrupted a run over it.
- **Replacing it** — removes the empty-response class and the trimming
  pressure, but discards a reviewer with a demonstrated record of catching real
  defects, and requires re-binding and re-validating the whole Low-band chain.

**Recommendation: keep the binding, fix the pipeline.**

The evidence does not support blaming the model for the worst problem found.
Every Muse Glimmer failure was *loud* — empty content, visible, correctly
escalated to D14 by the existing chain. The silent, fail-open, evidence-
fabricating behaviour is the **repository's own tooling** (D1/D2/D3), and it
would misbehave identically behind any reviewer model. Swapping the binding
would leave that untouched while discarding a reviewer that works 78% of the
time.

Revisit the binding **after C1 and C4 land**, when the failure rate can be
measured against a pipeline that reports honestly. If the empty-response rate
stays materially above ~20% after the `/no_think` fix, that is the point at
which re-binding becomes the evidence-backed call — and the rollback trigger in
`AGENT_WORKFLOW_GUIDE.md § Handoff prompt format` (escalation rate `> 40%` over
a rolling 20-task window) is the existing policy hook for it.

One caveat on the recommendation's own confidence: `S-230-T4e`/`T4g` show that
some recorded `PASS` verdicts rest on trimmed packets, so the true
review-quality rate is likely **below** the 78% availability figure. C6 exists
to make that measurable rather than guessed.

## 6. Related

- `docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md` — root cause of
  the empty-response class
- `docs/audit/agent-workflow-binding-history.md` — 2026-08-11 rebinding that put
  Muse Glimmer in both roles
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Band-routed peer review,
  § Gemma Reviewer / Muse Glimmer Reviewer, § Review artifact receipt and
  REVIEW-OVERRIDE lines
- `docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md`,
  `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
- `docs/tasks/s-230-poc-v1-digitalocean.md` § S-230-T4e, § S-230-T4g,
  § S-230-T4l
