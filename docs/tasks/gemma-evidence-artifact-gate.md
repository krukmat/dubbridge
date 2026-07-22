---
type: TaskList
title: "Tasks: Gemma/Peer Review Evidence Artifact Gate"
plan: docs/plan/gemma-evidence-artifact-gate.md
status: proposed
slice: GEG
rri: 48
band: Med-high
effort: L
---

# Tasks: Gemma/Peer Review Evidence Artifact Gate

Governing plan: `docs/plan/gemma-evidence-artifact-gate.md`
Governing ADR: ADR-034 (audit log stays git-ignored/local; unaffected by this task)

> **Split note:** GEG-1 (Effort L, RRI 48) is broken into five sequential
> subtasks, GEG-1a..GEG-1e, so each unit of work carries a small, bounded
> context instead of one L-sized task. The RRI 48 / Med-high band and the
> cross-vendor peer review closure requirement apply to the **group as a
> whole** (see Closure Requirements at the end of this file) — individual
> subtasks are not independently RRI-scored or independently closed with
> `[x] Done`; they are marked complete against their own acceptance criteria,
> and the group closes once GEG-1e passes.
>
> **Implementation route (RRI_POLICY.md, owner override 2026-07-21):** RRI
> 26–55 (Moderate + Med-high) routes to the **local-first implementation
> path** by default — `scripts/local-agent/run_local_task.py` in a disposable
> worktree, implementer resolved from `DUBBRIDGE_LOCAL_AGENT_MODEL` (default
> `qwen3.6:35b-a3b`). This applies per-subtask, not just to the group: each of
> GEG-1a–1e is Effort S/M and individually eligible, regardless of the
> group's overall Effort L / RRI 48 classification — Effort does not gate the
> routing decision, RRI band does. The primary agent (Claude Code, this
> session) remains orchestrator of record: it authors each subtask's
> delegation contract, applies the 3 Reflection passes to the local diff, and
> owns the repair budget (1 evidence-backed local attempt per subtask before
> escalating to cloud implementation — the Med-high, not Moderate, budget).
> Cross-vendor peer review, Reflection passes, and the RRI 41+ human approval
> gate are unchanged by this routing; only who authors the diff changes.

## Dependency order

```mermaid
flowchart LR
    D0["Prerequisite (external):\nOption C pre-push fix\ncommitted"] -.blocks.-> A
    A["GEG-1a\nReceipt schema +\nMakefile wiring"] --> B["GEG-1b\nValidator: artifact path"]
    B --> C["GEG-1c\nValidator: 3 override branches\n+ overrides ledger"]
    C --> E["GEG-1d\nPolicy/guide doc updates"]
    E --> F["GEG-1e\nCutover + full-corpus\nregression + tests"]

    classDef pending fill:#00000000,stroke-dasharray: 3 3,color:#6b6459
    class D0 pending
```

- **GEG-1a → GEG-1b → GEG-1c → GEG-1d → GEG-1e is a strict chain.** Each
  subtask reads/extends the output of the one before it; none are safely
  parallelizable.
- **External prerequisite — resolved.** Option C (the `.githooks/pre-push`
  fix moving Gemma/peer review out of pre-push into closure + CI) is
  committed as of `65f2b1e` (`fix(qa): stop running Gemma/peer review on
  every push`). GEG-1a edits the same `Makefile` region (`qa-gemma-review`,
  `qa-peer-workflow-review`, the new `qa-docs-review` target) on top of that
  commit, so this dependency is no longer blocking.
- No dependency on S-140 or any other product slice.

## GEG-1a — Receipt schema + Makefile wiring

- **Status:** Pending — unblocked (Option C landed at `65f2b1e`); ready for
  local-first delegation per the Implementation route note above.
- **Effort:** S
- **Objective:** Define the committed receipt schema and wire
  `GEMMA_REVIEW_TASK_ID` into `make qa-gemma-review` (mirroring the existing
  `PEER_REVIEW_TASK_ID` pattern already in `make qa-peer-workflow-review`) so
  both review targets write `docs/audit/gemma-evidence/<task_id>.json` when a
  task id is supplied.
- **Context:** First link in the chain — nothing downstream (validator,
  overrides, docs) can be built or tested without a real receipt file to
  point at. Kept isolated so it can be tested standalone before any ledger
  logic changes.
- **Related documents:** `docs/plan/gemma-evidence-artifact-gate.md` (Design
  §1), `Makefile` (`qa-gemma-review`, `qa-peer-workflow-review` targets),
  `scripts/gemma-code-review.py`, `scripts/peer-workflow-review.py`.
- **Inputs:** Existing `PEER_REVIEW_TASK_ID` wiring in `Makefile` as the
  pattern to mirror for `GEMMA_REVIEW_TASK_ID`.
- **Outputs:**
  - `Makefile`: `qa-gemma-review` accepts `GEMMA_REVIEW_TASK_ID`; both review
    targets write the receipt when a task id is supplied.
  - Receipt schema fixed as: `{task_id, commit_sha, reviewer, verdict,
    timestamp}`, written to `docs/audit/gemma-evidence/<task_id>.json`.
- **Acceptance criteria:**
  1. `make qa-gemma-review GEMMA_REVIEW_TASK_ID=<id>` writes a valid JSON
     receipt with all five fields to `docs/audit/gemma-evidence/<id>.json`.
  2. `make qa-peer-workflow-review PEER_REVIEW_TASK_ID=<id>` does the same
     (extends existing wiring rather than duplicating it).
  3. Omitting the task id on either target leaves current behavior
     (ephemeral `/tmp` output only, no committed receipt) unchanged.
  4. `commit_sha` is captured via `git rev-parse HEAD` at review time.
  5. `logs/gemma-audit/` (ADR-034) and the existing `/tmp` `--out` JSON are
     untouched by this change.
- **Pseudocode:**
  ```json
  {
    "task_id": "GEG-1",
    "commit_sha": "<git rev-parse HEAD at review time>",
    "reviewer": "gemma | codex | claude | d14",
    "verdict": "PASS | FINDINGS-ACKED",
    "timestamp": "2026-07-22T18:00:00Z"
  }
  ```

## GEG-1b — Ledger validator: artifact path

- **Status:** Pending — depends on GEG-1a merged (needs a real receipt to
  validate against).
- **Effort:** S
- **Objective:** Extend `validate_gemma_reviewer_evidence` in
  `scripts/check-task-unit-coverage.sh` so a `Review artifact:` line is
  checked against the actual receipt file (not just textual presence), and
  make the check apply to **every** completed development section regardless
  of RRI band — closing the current RRI ≥ 41 no-check gap for this one path.
  Override branches are explicitly out of scope here (see GEG-1c).
- **Context:** Second link — needs GEG-1a's receipt file format to exist
  before it can be parsed and cross-checked. Deliberately scoped to the
  artifact-happy-path only so the override-branch logic (more surface area,
  three sub-types) is reviewed as its own unit in GEG-1c.
- **Related documents:** `scripts/check-task-unit-coverage.sh`
  (`validate_gemma_reviewer_evidence`), `docs/plan/gemma-evidence-artifact-gate.md`
  (Design §2).
- **Inputs:** Receipt schema and write path from GEG-1a.
- **Outputs:** Updated validator: band-agnostic invocation; artifact-path
  branch parses the receipt and checks `task_id` match + `commit_sha`
  reachability from reviewed history.
- **Acceptance criteria:**
  1. Validator now runs for every `is_completed_development_section`
     regardless of RRI (closes the RRI ≥ 41 gap for the artifact path).
  2. Valid receipt with matching `task_id` and reachable `commit_sha` → pass.
  3. Missing receipt file → fail.
  4. Receipt with mismatched `task_id` → fail.
  5. Receipt whose `commit_sha` is not reachable from reviewed history →
     fail.
  6. Sections with neither `Review artifact:` nor any override line still
     fail with a clear message (override branches themselves are GEG-1c;
     this AC only requires that absence of both is not silently accepted).
- **Pseudocode:**
  ```
  if section has "Review artifact:" line:
      receipt = parse_json(docs/audit/gemma-evidence/<task_id>.json)
      fail unless receipt exists, is valid JSON,
                 receipt.task_id == section.task_id,
                 receipt.commit_sha reachable from HEAD
      pass
  else:
      fail: "missing Review artifact" # override branches added in GEG-1c
  ```

## GEG-1c — Validator: three override branches + overrides ledger

- **Status:** Pending — depends on GEG-1b merged (extends the same
  conditional the artifact path lives in).
- **Effort:** M
- **Objective:** Add the three typed `REVIEW-OVERRIDE:` branches (`urgency`,
  `not-applicable`, `pipeline-failure`) to the validator, each requiring its
  companion field, and create the new append-only
  `docs/audit/gemma-review-overrides.md` ledger that every accepted override
  must also appear in.
- **Context:** Third link — this is where the plan's exception design
  (urgencies, legitimate non-applicability, pipeline failures) actually gets
  enforced, extending the existing `D14-OVERRIDE` grammar precedent in
  `scripts/check-review-budget.py` rather than inventing a new pattern.
- **Related documents:** `docs/plan/gemma-evidence-artifact-gate.md`
  (Design §3), `scripts/check-review-budget.py` (`D14-OVERRIDE` precedent),
  `docs/policies/HITL_AUTONOMY_POLICY.md`.
- **Inputs:** Validator skeleton from GEG-1b; `D14-OVERRIDE` regex pattern.
- **Outputs:**
  - Validator: `REVIEW-OVERRIDE: <type> — <reason>` branch with per-type
    companion-field checks.
  - New file `docs/audit/gemma-review-overrides.md` (append-only ledger).
- **Acceptance criteria:**
  1. `REVIEW-OVERRIDE: urgency — <reason>` requires companion
     `Waiver-by: <name>` naming a human approver; an agent cannot self-issue
     it (no valid `Waiver-by` → fail).
  2. `REVIEW-OVERRIDE: pipeline-failure — <reason>` requires companion
     `Failed-attempt: <evidence>` citing a falsifiable failed run (timestamp
     + outcome, or CI job/step reference); an unevidenced assertion fails.
  3. `REVIEW-OVERRIDE: not-applicable — <reason>` requires companion
     `Scope-note: <why>` explaining the absent reviewable diff.
  4. Every accepted override must also have a matching row in
     `docs/audit/gemma-review-overrides.md`; a missing row fails the gate
     even if the task file's override line is otherwise complete.
  5. An override type outside the three named ones fails.
- **Pseudocode:**
  ```
  elif section has "REVIEW-OVERRIDE: <type> — <reason>" line:
      fail unless type in {urgency, not-applicable, pipeline-failure}
      fail unless companion field present per type
          (Waiver-by | Scope-note | Failed-attempt)
      fail unless matching row exists in
                 docs/audit/gemma-review-overrides.md
      pass
  else:
      fail: "missing Review artifact or REVIEW-OVERRIDE"
  ```

## GEG-1d — Policy/guide documentation updates

- **Status:** Pending — depends on GEG-1c merged (docs describe the finished
  contract, not an in-progress one).
- **Effort:** S
- **Objective:** Document the artifact-or-override contract and all three
  override types (with companion fields) in the three governing docs.
- **Context:** Fourth link — deliberately sequenced after the mechanism is
  built and tested, not before, so the docs describe actual behavior rather
  than intent that might still shift during GEG-1b/1c implementation.
- **Related documents:** `docs/policies/RRI_POLICY.md`,
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `docs/policies/HITL_AUTONOMY_POLICY.md`.
- **Inputs:** Finished validator behavior from GEG-1b + GEG-1c.
- **Outputs:** Updated sections in all three docs naming the artifact path
  and all three override types plus companion fields.
- **Acceptance criteria:**
  1. `docs/policies/RRI_POLICY.md` documents the artifact-or-override
     requirement applies at every RRI band.
  2. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` documents the `Review
     artifact:` and `REVIEW-OVERRIDE:` line formats and where the receipt
     file lives.
  3. `docs/policies/HITL_AUTONOMY_POLICY.md` documents that `urgency`
     overrides require human `Waiver-by` and cannot be agent-self-issued.
  4. OKF frontmatter validation (`make qa-okf-frontmatter`) still passes on
     all three edited files.

## GEG-1e — Cutover + full-corpus regression + tests

- **Status:** Pending — depends on GEG-1a through GEG-1d merged.
- **Effort:** M
- **Objective:** Define the grandfather cutover point, run the new validator
  against the full `docs/tasks/*.md` corpus with no false positives on
  pre-cutover sections, and add test coverage for every validator branch.
- **Context:** Final link — this is where the whole chain gets proven
  against real repository state rather than in isolation, and where the
  group's acceptance criteria (originally AC 10–12 of the unified GEG-1 task)
  get satisfied.
- **Related documents:** `scripts/check-task-unit-coverage.sh`,
  `docs/plan/gemma-evidence-artifact-gate.md` (Risks R1).
- **Inputs:** Complete validator (GEG-1b + GEG-1c) and updated docs
  (GEG-1d).
- **Outputs:** Cutover date/commit recorded in the script and its comments;
  passing full-corpus run; new tests for all validator branches.
- **Acceptance criteria:**
  1. A cutover point (date or commit) is defined so historical Done sections
     predating this task are not retroactively broken; the script and its
     comments state the cutover explicitly.
  2. `bash scripts/check-task-unit-coverage.sh` (full `docs/tasks/*.md`
     corpus) passes with no false positives against pre-cutover sections.
  3. New tests cover: valid artifact → pass; artifact with mismatched
     `task_id` → fail; each override type complete → pass; each override
     type missing its companion field → fail; override present in the task
     file but absent from `docs/audit/gemma-review-overrides.md` → fail; no
     evidence at all → fail.

## Scope (applies to the GEG-1a–1e group)

- **In:** The artifact receipt schema and its write path; the ledger
  validator rewrite (band-agnostic + artifact/override logic); the three
  typed overrides and their companion-field checks; the overrides ledger;
  policy/guide doc updates; tests for the new validator branches; the
  grandfather cutover.
- **Out:** Any change to ADR-034 (audit log location/format/retention), to
  the PPR band-routing rule itself, to `gemma-code-review.py`'s `/tmp`
  `--out` behavior, or to `.githooks/pre-push` itself (Option C is a
  separate, already-implemented fix — see Dependency order above for why its
  *landing* is nonetheless a blocking prerequisite for GEG-1a). No
  retroactive rewrite of existing Done task files beyond what the
  grandfather clause (GEG-1e AC 1) requires.

## Risks

Carried from the plan (`docs/plan/gemma-evidence-artifact-gate.md#risks`):
corpus break on rollout (R1, mitigated by GEG-1e AC 1), override abuse (R2,
mitigated by mandatory human `Waiver-by` + committed overrides ledger in
GEG-1c), CI portability of the receipt (R3, receipt is committed by the
closing agent locally in GEG-1a, not generated by CI), and replay risk on a
stale receipt (R4, `commit_sha` reachability check in GEG-1b — exact
semantics an implementation decision).

## Closure Requirements (group: GEG-1a–1e)

This is a `Type: development` task group at RRI 48 (Med-high, ≥ 26), so the
mandatory review gate applies before Done/coverage certification for the
group — **not** skippable by this task's own mechanism, and **not**
satisfied piecemeal per subtask. Per PPR band routing
(`docs/plan/portable-peer-review-gate.md`), RRI 41+ routes **phase-2
code-solution review to the cross-vendor peer** (`make
qa-peer-workflow-review`), not Gemma; D14 is the fallback if the peer CLI is
unavailable. Closure order, run once after GEG-1e completes:

1. Confirm Type: development, RRI 48 ⇒ cross-vendor peer review (not Gemma)
   applies for phase-2 code-solution review; D14 fallback if peer CLI
   unavailable.
2. Run `make qa-peer-workflow-review` (or D14 fallback) over the combined
   GEG-1a–1e implementation diff; record the result per the existing closure
   report contract.
3. Reflection log (RRI ≥ 26 requires it).
4. Unit coverage certification for all HP-#/EC-# cases across all five
   subtasks, including GEG-1e's validator-branch tests.
5. Owner final verification.
6. Sync `docs/plan/gemma-evidence-artifact-gate.md` status and this file's
   frontmatter to `done`.
7. Mark `[x] Done` for the group.

## Diagram

```mermaid
flowchart TD
    S["Done + Type: development section\n(any RRI)"] --> E{Evidence line present?}
    E -- "Review artifact:" --> R[Load docs/audit/gemma-evidence/&lt;task_id&gt;.json]
    R --> RV{task_id matches AND\ncommit_sha reachable?}
    RV -- yes --> PASS[Gate: PASS]
    RV -- no --> FAIL[Gate: FAIL]

    E -- "REVIEW-OVERRIDE: urgency" --> U{Waiver-by: &lt;human&gt; present?}
    U -- yes --> L1{Row in\ngemma-review-overrides.md?}
    U -- no --> FAIL

    E -- "REVIEW-OVERRIDE: pipeline-failure" --> PF{Failed-attempt: evidence present?}
    PF -- yes --> L2{Row in\ngemma-review-overrides.md?}
    PF -- no --> FAIL

    E -- "REVIEW-OVERRIDE: not-applicable" --> NA{Scope-note: reason present?}
    NA -- yes --> L3{Row in\ngemma-review-overrides.md?}
    NA -- no --> FAIL

    L1 -- yes --> PASS
    L1 -- no --> FAIL
    L2 -- yes --> PASS
    L2 -- no --> FAIL
    L3 -- yes --> PASS
    L3 -- no --> FAIL

    E -- "neither present" --> FAIL
```

Execution has not started. Approve this task to proceed. Option C is already
committed (`65f2b1e`), so GEG-1a has no remaining blocking dependency.
