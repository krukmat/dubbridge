---
type: TaskList
title: "Tasks: Workflow-policy consolidation after documentation reduction"
status: completed
plan: docs/plan/workflow-policy-consolidation.md
---

# Tasks: Workflow-policy consolidation after documentation reduction

**Plan:** `docs/plan/workflow-policy-consolidation.md`
**Authoritative guide:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
**Related policy:** `docs/policies/HITL_AUTONOMY_POLICY.md`

## Status legend

- [ ] Not started · [~] In progress · [x] Done

## Dependency order

```text
T0 -> T1 -> T2 -> T3 -> T4 -> T5 -> T6 -> T7
```

| Task | Title | Type | Depends on | Execution gate |
|---|---|---|---|---|
| T0 | Record the consolidation plan and ledger | Planning/docs | — | Done in this change |
| T1 | Normalize authority and projection sources | Policy/docs | T0 | Re-read authority sources |
| T2 | Normalize repair, decomposition, and cloud routing | Policy/docs | T1 | Re-read ADR-038/040 and RRI policy |
| T3 | Normalize Qwen/Gemma local-role terminology | Policy/docs | T2 | Verify bindings in code and RRI policy |
| T4 | Reconcile ADR-031/S-200 current-state prose | Docs | T3 | Verify ADR-031 and S-200 ledger |
| T5 | Correct workflow archive scope claims | Docs/audit | T4 | Compare archive with active Step 0 |
| T6 | Regenerate projection, QA, and status closeout | Docs/config | T5 | All source tasks complete |
| T7 | Replace full projection with bounded Codex bootstrap | Tooling/docs | T6 | User-approved bounded follow-up; RRI 30 |

## T0 — Record the consolidation plan and ledger

- **Status:** [x] Done — 2026-08-20
- **Objective:** Preserve the review findings as a bounded, ordered execution
  plan rather than making untracked follow-up edits.
- **Outputs:** this ledger and
  `docs/plan/workflow-policy-consolidation.md`.
- **Acceptance criteria:**
  - every reported issue has a task, owner surface, dependency, and closure
    check;
  - regeneration and QA are last, not interleaved with source edits;
  - scope excludes runtime/code/ADR decision changes.
- **Completion record:** Created the plan and ledger; no corrective source
  document was changed by T0.

## T1 — Normalize authority and projection sources

- **Status:** [x] Done — 2026-08-20
- **Objective:** Establish one unambiguous precedence statement across all active
  workflow orientation documents, then identify the exact source set that feeds
  the generated override.
- **Inputs:**
  - `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative);
  - `docs/policies/HITL_AUTONOMY_POLICY.md`;
  - `README_AGENT_ORDER.md`, `AGENTS.md`;
  - `scripts/generate-agents-override.py`.
- **Allowed outputs:** the active authority summaries above; no hand edit to
  `AGENTS.override.md` in this task.
- **Acceptance criteria:**
  - no active document says `CLAUDE.md` wins a conflict covered by the workflow
    guide;
  - the reader can determine that `CLAUDE.md` covers only gaps in the guide;
  - the documented authority order and the generator's source list are mutually
    understandable without relying on historical prose.
- **Evidence to emit:** concise source-to-precedence comparison in the task
  completion record.
- **Status artifacts affected:** this ledger, the plan, and the generated
  override in T6.

### Completion record

- **Classification:** policy/docs-only; phase 1 and phase 2 peer review are
  not applicable under the workflow guide.
- **RRI:** 16 (Low; Effort S). Direct primary-agent execution; no local model
  invocation or Ollama precheck was required.
- **Task-analysis review:** n/a — policy/docs-only exemption.
- **Code-solution review:** n/a — policy/docs-only exemption.
- **Source-to-precedence comparison:**

  | Source | Established role after T1 |
  |---|---|
  | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | Highest authority for every workflow topic it covers. |
  | `CLAUDE.md` (project and global) | Applies only when the workflow guide is silent. |
  | `AGENTS.md` and `docs/policies/HITL_AUTONOMY_POLICY.md` | Summaries that must not override the guide. |
  | `README_AGENT_ORDER.md` | Orients readers in the same precedence order and identifies the generated projection's sources. |
  | `scripts/generate-agents-override.py` | Generates `AGENTS.override.md` only from `AGENTS.md`, the workflow guide, HITL policy, roadmap, and architecture overview; it does not consume README or `CLAUDE.md`. |

- **Edits:** corrected HITL's conflict clause; reordered README so the guide is
  read first; documented the generator source set. `AGENTS.md` and the
  authoritative guide already matched this rule and were not changed.
- **Verification:** `git diff --check`; targeted active-document searches; and
  `python3 scripts/generate-agents-override.py --check >/dev/null` passed.
- **Deferred:** `AGENTS.override.md` remains intentionally unchanged until T6,
  when all generator inputs are final.

## T2 — Normalize repair, decomposition, and cloud routing

- **Status:** [x] Done — 2026-08-20
- **Objective:** Remove the contradictory implication that Moderate repair
  exhaustion escalates directly to cloud, while preserving the bounded cloud
  fallback and ADR-040 module-tramo exception.
- **Inputs:**
  - `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`;
  - `docs/policies/HITL_AUTONOMY_POLICY.md`;
  - `docs/policies/RRI_POLICY.md`;
  - ADR-038 and ADR-040.
- **Allowed outputs:** active workflow/policy summaries and generated inputs;
  no modification to routing scripts or ADR decisions.
- **Acceptance criteria:**
  - every Moderate whole-task path says: two evidence-backed repairs → scored
    Low-band decomposition by default → last-resort cloud if that route cannot
    proceed;
  - no Med-high sentence implies that a whole task becomes locally authored from
    an ADR-038 `GO_LOCAL` result;
  - tables, handoff text, and escalation wording use the same sequence;
  - ADR-040 remains the only stated exception for an independently qualified
    local module tramo.
- **Evidence to emit:** a before/after route matrix covering Moderate,
  Med-high whole-task, and ADR-040 module routes.
- **Status artifacts affected:** this ledger, the plan, and generated override
  in T6.

### Completion record

- **Classification:** policy/docs-only; phase 1 and phase 2 peer review are
  not applicable under the workflow guide.
- **RRI:** 16 (Low; Effort S). Direct primary-agent execution; no local model
  invocation or Ollama precheck was required.
- **Task-analysis review:** n/a — policy/docs-only exemption.
- **Code-solution review:** n/a — policy/docs-only exemption.
- **Route matrix after T2:**

  | Route | Local authoring budget | Next step after budget exhaustion | Cloud role |
  |---|---|---|---|
  | Moderate whole task (RRI 26–40) | `qwen3.8:27b-mlx`, 2 evidence-backed repairs | Decompose remaining work into scored Low-band subtasks | Last resort only when decomposition cannot proceed; operational runner/model and scope failures retain their separately approved takeover condition. |
  | Med-high whole task (RRI 41–55) | None; `GO_LOCAL` is advisory evidence only | Not applicable | Every valid refinement/receipt outcome, including `GO_LOCAL`, emits the ADR-038 cloud handoff. |
  | ADR-040 qualified local module | 2 evidence-backed local repairs for the disjoint low-CC module only | May use scored Low-band decomposition for remaining module work | High-CC or hard-excluded modules use their ADR-040 cloud route and budget. |

- **Edits:** removed every active Moderate whole-task statement that sent 2/2
  repair exhaustion straight to cloud; stated Low-band decomposition before
  last-resort cloud; separated operational/scope takeover conditions; and
  made explicit that a Med-high whole-task `GO_LOCAL` result has no local
  authoring or repair budget.
- **Verification:** reran `scripts/rri.py` (RRI 16); searched all three active
  routing sources for repair/escalation and `GO_LOCAL` terms; and ran
  `git diff --check` successfully.
- **Deferred:** generator projection remains for T6, after all source tasks
  are complete.

## T3 — Normalize Qwen/Gemma local-role terminology

- **Status:** [x] Done — 2026-08-20
- **Objective:** Align active developer-role wording with the Qwen binding while
  retaining Gemma/Muse Glimmer as band-routed reviewers.
- **Inputs:**
  - `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`;
  - `AGENTS.md`, `README_AGENT_ORDER.md`;
  - `docs/policies/RRI_POLICY.md`;
  - `scripts/delegate-low-rri.py`, `scripts/rri.py`, and their paired unit
    tests;
  - `docs/gemma-local-improve.md` (the active summary linked by the workflow
    guide).
- **Acceptance criteria:**
  - active local code-authoring references name Qwen Developer and
    `qwen3.8:27b-mlx` where a concrete binding is appropriate;
  - Gemma references remain only for reviewer roles or deliberately labelled
    historical/proposed artifacts;
  - no active route calls the Qwen delegation path an "eligible-Gemma" route.
- **Evidence to emit:** a scoped search result for developer-role terminology
  plus an explanation of any intentional historical exception.
- **Status artifacts affected:** this ledger, plan, and generated override in T6.

### Completion record

- **Classification:** policy/docs-only terminology consolidation with a narrow,
  user-authorized execution-contract correction in `scripts/rri.py` and
  `scripts/delegate-low-rri.py`; phase 1 and phase 2 peer review are not
  applicable under the workflow guide. No local model was invoked, so the
  Ollama precheck was not required.
- **RRI report (2026-08-20):**

  ```text
  **Platform:** dubbridge

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
  | F files | 3 | --touches -> 6 files | High |
  | D domain | 0 | agent-supplied (no rubric match) | High |
  | T coverage | 0 | agent-supplied | High |
  | A ambiguity | 1 | agent-supplied | High |
  | K coupling | 1 | agent-supplied (no rubric match) | High |
  | P impact | 2 | agent-supplied (no rubric match) | High |
  | X context | 2 | agent-supplied | High |

  **Base value:** 100 x (weighted / 5) = 18
  **Penalties applied:** none
  **Final RRI:** 18 -> band Low (0-25) -> Effort S . Codex Local Qwen Developer via Ollama . Claude Local Qwen Developer via Ollama . thinking Off
  **Gates for this band:** Local delegation: delegate to local Qwen Developer via Ollama; validate and apply only an in-scope diff; review against requirements; verify; report.
  **Decomposition:** not triggered
  **Advisory:** AGENTS.md: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** README_AGENT_ORDER.md: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** scripts/delegate-low-rri.py: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** scripts/rri.py: no anchor-rubric match — agent judgment governs D/P/K
  ```

- **Task-analysis review:** n/a — policy/docs-only exemption.
- **Code-solution review:** n/a — policy/docs-only exemption.
- **Edits:** replaced active Low-band authoring references with Qwen Developer;
  corrected the RRI calculator's low-band output and delegation wrapper's help,
  validation, timeout, audit-bundle, and transport-error terms; made attempt
  bundles identify `qwen38`; and updated paired tests plus the active linked
  local-role summary.
- **Intentional retained Gemma references:** Gemma remains named only for its
  reviewer/push-reviewer roles and the shared legacy transport module
  `scripts/gemma_local.py`. That module name and the associated audit-log path
  are implementation lineage, not an active developer-route label.
- **Verification:** scoped terminology search leaves only reviewer-role
  references; `python3 -m unittest scripts/delegate_low_rri_test.py
  scripts/rri_test.py` passed (166 tests); rerunning `scripts/rri.py` emitted
  `Local Qwen Developer via Ollama`; `git diff --check` passed. The generated
  override remains intentionally deferred to T6.

## T4 — Reconcile ADR-031/S-200 current-state prose

- **Status:** [x] Done — 2026-08-20
- **Objective:** Keep the roadmap and architecture overview aligned with the
  completed S-200 JWT implementation, while keeping old ADR-024 behavior only as
  clearly superseded history.
- **Inputs:**
  - `docs/adr/ADR-031-*.md`;
  - `docs/tasks/s-200-mobile-jwt-credential-auth.md`;
  - `docs/architecture.md`;
  - `docs/plan/roadmap.md`;
  - `docs/audit/roadmap-history.md`.
- **Acceptance criteria:**
  - active roadmap prose no longer says a device must terminate at the opaque
    session-gateway boundary or cannot hold a long-lived token;
  - architecture status and runtime-surface prose describe S-200 as delivered,
    not merely planned or pending;
  - the backend-issued HS256 bearer JWT, transparent gateway relay, and secure
    device storage are consistently attributed to ADR-031/S-200;
  - superseded ADR-024 narrative is either concise and explicitly historical or
    moved to the archive.
- **Evidence to emit:** exact ADR/task references supporting every current-state
  statement retained in active documents.
- **Status artifacts affected:** roadmap, architecture overview, archive as
  needed, this ledger, and the generated override in T6.

### Completion record

- **Classification:** docs-only; phase 1 and phase 2 peer review are not
  applicable under the workflow guide. Direct primary-agent execution; no local
  model invocation or Ollama precheck was required.
- **RRI:** 13 (Low; Effort S). Full report was calculated with `scripts/rri.py`
  for the four updated documentation/status files; no penalties applied.
- **Task-analysis review:** n/a — docs-only exemption.
- **Code-solution review:** n/a — docs-only exemption.
- **Current-state evidence retained in active docs:** ADR-031 §Decision 2 defines
  the backend-issued HS256 token and algorithm pinning; §Decision 3 makes the
  gateway a transparent relay; §Decision 4 requires device secure storage. Its
  `Implemented by` field records S-200 T1–T7. The S-200 ledger's progress table
  records all decomposed implementation tasks through T7 as done, and T7's
  completion record confirms the roadmap status update.
- **Edits:** replaced the active roadmap's obsolete ADR-024 opaque-session rule
  with the delivered ADR-031/S-200 bearer-JWT and transparent-relay model. The
  architecture overview already described that delivered state, so it needed no
  edit. `docs/audit/roadmap-history.md` deliberately retains ADR-024's prior model
  as dated historical narrative and needed no change.
- **Verification:** reran `scripts/rri.py`; checked the active roadmap and
  architecture for opaque-session / long-lived-token wording; and ran
  `git diff --check` successfully. Generated `AGENTS.override.md` remains deferred
  to T6.

## T5 — Correct workflow archive scope claims

- **Status:** [x] Done — 2026-08-20
- **Objective:** Make the new workflow-detail archive accurate about whether it
  contains a full relocation or only supporting rationale and examples.
- **Inputs:**
  - `docs/audit/agent-workflow-guide-detail-archive.md`;
  - active Step 0 in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.
- **Acceptance criteria:**
  - the archive does not describe itself as complete/full if it omits the old
    PID/listener restart procedure;
  - either its label is narrowed to rationale/example material or the missing
    procedure is deliberately restored, with the chosen path documented;
  - the active guide retains a safe, actionable Step 0 regardless of archive
    depth.
- **Evidence to emit:** a short completeness decision naming the retained active
  procedure and the archive's final scope.
- **Status artifacts affected:** archive, workflow guide only if needed, this
  ledger, plan, and generated override in T6.

### Completion record

- **Classification:** docs/audit-only; phase 1 and phase 2 peer review are not
  applicable under the workflow guide. Direct primary-agent execution; no local
  model invocation or Ollama precheck was required.
- **RRI:** 13 (Low; Effort S). Full report was calculated with `scripts/rri.py`
  for the archive, task ledger, and plan; no penalties applied.
- **Task-analysis review:** n/a — docs/audit-only exemption.
- **Code-solution review:** n/a — docs/audit-only exemption.
- **Completeness decision:** chose the plan's preferred narrow label. The archive
  now identifies its Step 0 content as the warm-up command and supporting
  rationale, not a full relocation. The active guide retains the authoritative,
  actionable task-boundary restart, active-runner protection, replacement
  PID/listener verification, ordered resource-recovery protocol, evidence, and
  role-scope requirements.
- **Edits:** corrected the archive heading and scope statement only; the active
  guide already retained every operational requirement, so it was not changed.
- **Verification:** compared the archive against active Step 0; ran
  `python3 scripts/rri.py --touches docs/audit/agent-workflow-guide-detail-archive.md --touches docs/tasks/workflow-policy-consolidation.md --touches docs/plan/workflow-policy-consolidation.md --cc 1 --D 0 --K 1 --P 1 --T 0 --A 1 --X 1` (RRI 13); and ran `git diff --check` successfully. Generated `AGENTS.override.md` remains deferred to T6.

## T6 — Regenerate projection, QA, and status closeout

- **Status:** [x] Done — 2026-08-20
- **Objective:** Produce the generated override from the final source state and
  prove the documentation set is internally valid.
- **Inputs:** completed T1–T5 source edits and both new audit documents.
- **Outputs:** regenerated `AGENTS.override.md`, final task status, and QA
  evidence.
- **Acceptance criteria:**
  - `AGENTS.override.md` is produced by
    `python3 scripts/generate-agents-override.py --write`, not edited manually;
  - `make qa-docs`, `make qa-roadmap-drift`, the focused prompt-builder tests,
    and `git diff --check` pass;
  - the final staging guidance explicitly includes
    `docs/audit/agent-workflow-guide-detail-archive.md` and
    `docs/audit/roadmap-history.md`;
  - every T1–T5 completion record cites its evidence and no unresolved active
    contradiction remains in the scoped search.
- **Evidence to emit:** exact commands and results, generated-file confirmation,
  and a final scoped search report.
- **Status artifacts affected:** `AGENTS.override.md`, this ledger, and the plan.

### Completion record

- **Classification:** docs/config-only; phase 1 and phase 2 peer review are not
  applicable under the workflow guide. Direct primary-agent execution; no local
  model invocation or Ollama precheck was required.
- **RRI:** 13 (Low; Effort S). Calculated with:

  ```bash
  python3 scripts/rri.py --touches AGENTS.override.md --touches docs/tasks/workflow-policy-consolidation.md --touches docs/plan/workflow-policy-consolidation.md --cc 1 --D 0 --K 1 --P 1 --T 0 --A 1 --X 1
  ```

  No penalties applied.
- **Task-analysis review:** n/a — docs/config-only exemption.
- **Code-solution review:** n/a — docs/config-only exemption.
- **Generated projection:** `python3 scripts/generate-agents-override.py --write`
  regenerated `AGENTS.override.md`; an immediately regenerated `--check` stream
  matched that file byte-for-byte with `cmp -s`.
- **Verification:**
  - `make qa-docs` — passed.
  - `make qa-roadmap-drift` — passed (`Roadmap drift check passed.`).
  - `python3 -m unittest scripts/local-agent/prompt_anchors_test.py scripts/local-agent/prompt_builder_test.py` — passed (14 tests).
  - `git diff --check` — passed.
- **Scoped search:** checked authority precedence, Moderate repair exhaustion,
  Med-high `GO_LOCAL`, local-developer naming, opaque sessions, and archive
  completeness across active authority, policy, roadmap, architecture, and archive
  sources. Remaining matches are explicitly labelled historical/superseded
  ADR-024 context, the archive's statement that it is *not* a complete relocation,
  or the unapproved proposed X27 Gemma Push Reviewer item; no unresolved active
  contradiction remains in this scope.
- **Staging guidance:** no files were staged or committed. When committing this
  work, include both new audit records:
  `docs/audit/agent-workflow-guide-detail-archive.md` and
  `docs/audit/roadmap-history.md`.

## T7 — Replace full projection with bounded Codex bootstrap

- **Status:** [x] Done — 2026-08-20
- **Objective:** Reduce Codex's always-loaded repository instructions from the
  five-document concatenation to a compact bootstrap while retaining authority,
  approval/safety boundaries, deterministic regeneration, and explicit
  task-time routing to canonical sources.
- **Inputs:** `AGENTS.md`, `README_AGENT_ORDER.md`,
  `scripts/generate-agents-override.py`, its focused unit tests, and OpenAI's
  current lean-prompt guidance.
- **Allowed outputs:** `AGENTS.md`, `README_AGENT_ORDER.md`,
  `scripts/generate-agents-override.py`,
  `scripts/generate_agents_override_test.py`, generated
  `AGENTS.override.md`, this ledger, the linked plan, and the original
  `docs/tasks/agents-override-sync.md` delivery record whose generated-source
  contract is superseded by T7.
- **Out of scope:** changing workflow/HITL semantics, ADRs, roadmap or
  architecture content, runtime code, commit, push, or external actions.
- **Happy path examples:**
  - **HP-1:** running the generator produces a byte-exact copy of the bounded
    `AGENTS.md` bootstrap and the documentation drift check passes.
  - **HP-2:** a task that needs workflow, policy, roadmap, architecture, ADR, or
    slice detail is explicitly routed to read the relevant canonical file at
    task time rather than receiving all of them in every session.
- **Edge case examples:**
  - **EC-1:** a hand edit to `AGENTS.override.md` still fails the byte-exact
    drift check and names the regeneration command.
  - **EC-2:** a missing, empty, or oversized bootstrap fails closed before
    `--write` can replace the existing override.
- **Acceptance criteria:**
  - only `AGENTS.md` is a generator content source;
  - the bootstrap states authority/read-on-demand routing plus non-negotiable
    approval, safety, closure, language, and RRI requirements;
  - the generator enforces a maximum output size consistent with the proposed
    3k–6k-token ceiling and remains deterministic;
  - focused generator/drift tests, `make qa-docs`, `make qa-roadmap-drift`, and
    `git diff --check` pass;
  - closure reports exact before/after bytes, lines, words, and an estimated
    token range.
- **Evidence to emit:** phase-1 and phase-2 review artifacts, focused unit-test
  output, QA output, byte-exact generator comparison, and size delta.
- **Status artifacts affected:** this ledger, the plan, `AGENTS.override.md`,
  `README_AGENT_ORDER.md`, and `docs/tasks/agents-override-sync.md`.
- **Approval:** Matias explicitly authorized the bounded proposal with
  `ajústalo según tu propuesta` on 2026-08-20.
- **RRI report (2026-08-20):**

  ```text
  **Platform:** dubbridge

  | Variable | Score | Evidence | Confidence |
  |---|---|---|---|
  | C cyclomatic | 0 | raw CC 3 -> score 0 (policy CC table) | High |
  | F files | 3 | --touches -> 8 files | High |
  | D domain | 1 | agent-supplied (no rubric match) | High |
  | T coverage | 2 | agent-supplied | High |
  | A ambiguity | 1 | agent-supplied | High |
  | K coupling | 2 | agent-supplied (no rubric match) | High |
  | P impact | 2 | agent-supplied (no rubric match) | High |
  | X context | 2 | agent-supplied | High |

  **Base value:** 100 x (weighted / 5) = 30
  **Penalties applied:** none
  **Final RRI:** 30 -> band Moderate (26-40) -> Effort M . Codex Balanced . Claude Balanced . thinking Off
  **Gates for this band:** Confirm tests exist in the affected area.
  **Decomposition:** not triggered
  **Advisory:** AGENTS.md: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** AGENTS.override.md: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** README_AGENT_ORDER.md: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** scripts/generate-agents-override.py: no anchor-rubric match — agent judgment governs D/P/K
  **Advisory:** scripts/generate_agents_override_test.py: no anchor-rubric match — agent judgment governs D/P/K
  ```

- **Resolved implementation route:** documentation synthesis remained with the
  primary orchestrator. Python authoring used the Moderate local runner first,
  then the required post-budget Low-band decomposition. The two Moderate
  attempts produced correct diagnoses but no usable tool call; WPC-T7a and
  WPC-T7b reconstructed the generator/tests locally, and the orchestrator
  assembled the equivalent stricter verified variant under the documented
  tooling-failure exception. Review band, RRI, approval, and Reflection count
  remained Moderate throughout.

### Completion record

- **Classification:** development task (Python generator and tests) plus bounded
  agent-facing documentation. Antares refinement and post-implementation
  touchpoints: typed skip — no task-relevant CWE hypothesis exists on the T3a
  watchlist for deterministic documentation generation.
- **Task-analysis review:** `gemma .agent/wpc-t7-phase1-review.json - PASS`.
  The reviewer requested an explicit oversized fail-closed test and confirmed
  architecture/ADR content stayed outside the change; both dispositions are
  reflected in scope and tests.
- **Approval:** Matias authorized the bounded change on 2026-08-20 with
  `ajústalo según tu propuesta`; no further approval was required for the
  policy-mandated Low-band decomposition.

### Implementation routing evidence

- **Moderate attempt 1/2:** `.agent/wpc-t7-local-card.json` passed Gemma packet
  review at `.agent/wpc-t7-local-card-phase1.json`; result
  `.agent/wpc-t7-local-run.json` aborted as
  `malformed_tool_call_repeated`. Qwen correctly identified the required source
  reduction, byte guard, and tests, but applied no worktree edit.
- **Moderate attempt 2/2:** the narrowed repair packet passed Gemma at
  `.agent/wpc-t7-local-card-repair1-phase1.json`; result
  `.agent/wpc-t7-local-run-repair1.json` again aborted as
  `malformed_tool_call_repeated`, with no worktree edit. The whole-task repair
  budget was exhausted.
- **Post-budget WPC-T7a (RRI 10, Low):** generator-only packet
  `.agent/wpc-t7a-generator-packet.md`; phase-1 Gemma fallback `PASS` at
  `.agent/wpc-t7a-generator-phase1.json`; Qwen result
  `.agent/wpc-t7a-generator-result.json` applied in the disposable worktree.
- **Post-budget WPC-T7b (RRI 13, Low):** tests-only packet
  `.agent/wpc-t7b-tests-packet.md`; phase-1 Gemma fallback `PASS` at
  `.agent/wpc-t7b-tests-phase1.json`; Qwen result
  `.agent/wpc-t7b-tests-result.json` applied in the disposable worktree.
- **Bounded repair 1/1:** primary review found that WPC-T7b mocked routing links
  instead of checking the real bootstrap. The literal before/after repair packet
  `.agent/wpc-t7b-tests-repair-packet.md` passed Muse Glimmer at
  `.agent/wpc-t7b-tests-repair-phase1.json`; Qwen result
  `.agent/wpc-t7b-tests-repair-result.json` applied and the merged-state suite
  passed 15/15.
- **Tooling-failure exception:** the two Moderate runs independently diagnosed
  and proposed the correct fix but their agentic wrapper could not construct a
  usable tool call. The primary's already-present equivalent implementation was
  retained only after the Low reconstructions and repair proved the same
  behavior; no new substantive logic was substituted after that route.
- **Cloud escalation:** not triggered.

### Module-split routing evidence

- **Evaluation:** no split. The two Python files are both low-CC (`C=0` from raw
  CC 3/2) and do not satisfy ADR-040's heterogeneous `C>=2` plus `C<=1`
  trigger. Documentation files are not code-authoring modules. After the
  whole-task Moderate budget failed, the required Low decomposition separated
  generator and test authorship instead.

### Reflection log

Required passes: 2 (`RRI 30` → `Moderate`)

#### Pass 1

- **Draft verdict:** the bounded generator, compact routing bootstrap, and
  missing/empty/oversized guards matched HP-1, HP-2, and EC-2.
- **Critique findings:** the first `AGENTS.md` Purpose wording could still be
  read as instructing agents to bulk-read every canonical document; the initial
  local routing test simulated links instead of checking the real bootstrap.
- **Revisions applied:** clarified task-dependent on-demand loading in
  `AGENTS.md`; replaced the mocked routing test through the bounded local repair
  with a real `generator.generate()` assertion set.

#### Pass 2

- **Draft verdict:** all focused behavior and drift tests passed, with generated
  output byte-exact and below the ceiling.
- **Critique findings:** the real routing assertion did not yet name
  `docs/policies/RRI_POLICY.md`; the first phase-2 packet unnecessarily included
  the 132 KiB generated-file deletion and was not reviewable in bounded time.
- **Revisions applied:** added the RRI policy route assertion; rebuilt the
  phase-2 packet from source/test diffs plus independent byte-exact and size
  evidence. No production behavior changed.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: `python3 scripts/peer-workflow-review.py --phase code --rri 30 --caller codex --task-id WPC-T7-final --artifact .agent/wpc-t7-phase2-review-final.json --qwen-model gemma4:26b-a4b-it-qat --no-think --num-ctx 131072 --num-predict 8192 --temperature 0`
- Artifact: `.agent/wpc-t7-phase2-review-final.json`
- Verdict: `PASS`
- Findings: none
- Muse Glimmer fallback: not triggered — final Gemma packet was usable.
- D14 fallback: not triggered — final Gemma packet was usable.
- D14 provider route: n/a — no D14 trigger.
- disposition_divergence: `none`
- Primary-agent disposition: accepted the no-findings verdict. The initial
  oversized phase-2 invocation was interrupted before any verdict and replaced
  with the minimal, semantically equivalent review packet.
- **Code-solution review:** `gemma .agent/wpc-t7-phase2-review-final.json - PASS`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | generator produces the exact bounded bootstrap and regenerated drift passes | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest.test_hp1_generate_matches_fixed_concatenation`; `scripts/check_doc_consistency_agents_override_test.py::AgentsOverrideDriftCheckTest.test_hp1_check_passes_after_regeneration` | passed |
| HP-2 | Happy path | bootstrap routes workflow, policy, roadmap, architecture, ADR, plan, and task detail on demand | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest.test_hp2_bootstrap_routes_to_canonical_sources_on_demand` | passed |
| EC-1 | Edge case | hand-edited or newline-only generated drift fails closed | `scripts/check_doc_consistency_agents_override_test.py::AgentsOverrideDriftCheckTest.test_ec1_check_fails_closed_on_hand_edited_drift`; `scripts/check_doc_consistency_agents_override_test.py::AgentsOverrideDriftCheckTest.test_ec1_check_catches_trailing_newline_only_drift` | passed |
| EC-2 | Edge case | missing, empty, or oversized source cannot replace/create output | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest.test_ec2_missing_source_file_exits_nonzero`; `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest.test_ec2_empty_source_file_exits_nonzero`; `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest.test_ec2_oversized_bootstrap_exits_nonzero`; `scripts/generate_agents_override_test.py::GenerateAgentsOverrideRealFilesystemTest.test_ec2_write_mode_does_not_modify_output_on_oversized_source` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-08-20`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior.
- Commands run:
  - `python3 scripts/generate-agents-override.py --write`
  - `python3 -m py_compile scripts/generate-agents-override.py scripts/generate_agents_override_test.py`
  - `python3 -m unittest scripts/generate_agents_override_test.py` — 15 passed.
  - `COVERAGE_FILE=/tmp/wpc-t7.coverage python3 -m coverage run --branch -m unittest scripts/generate_agents_override_test.py`
  - `COVERAGE_FILE=/tmp/wpc-t7.coverage python3 -m coverage report -m scripts/generate-agents-override.py` — 96%.
  - `python3 -m unittest scripts/check_doc_consistency_agents_override_test.py` — 5 passed.
  - `make qa-docs` — passed.
  - `make qa-roadmap-drift` — passed.
  - `python3 -m unittest scripts/local-agent/prompt_anchors_test.py scripts/local-agent/prompt_builder_test.py` — 14 passed.
  - `git diff --check` — passed.
  - `cmp -s AGENTS.override.md <(python3 scripts/generate-agents-override.py --check)` — passed under zsh.

### Size and context evidence

- Before: 142,105 bytes; 2,406 lines; 18,878 words; estimated 35k–45k tokens.
- After: 9,843 bytes; 205 lines; 1,291 words; estimated 2.5k–3.2k tokens.
- Delta: 132,262 bytes removed (93.1%); estimated 32k–42k fewer always-loaded
  tokens per session that loads the repository override.
- Generator ceiling: 24,576 UTF-8 bytes; oversized input exits before any write.
- Generated comparison: `AGENTS.override.md` matched `--check` byte-for-byte.
- Status synchronization: this ledger, linked plan, generated override,
  `README_AGENT_ORDER.md`, and `docs/tasks/agents-override-sync.md` are aligned.

## Execution handoff

> Execute only the next unchecked task in dependency order from this ledger.
> Governing files: `docs/plan/workflow-policy-consolidation.md`,
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, and the task-specific inputs above.
> Stop condition: record evidence and status for the completed task; do not begin
> the next task or commit/push without the applicable workflow gate.
