---
type: TaskList
title: "Tasks: Nemotron local-developer scope — S and M only"
status: Active
plan: docs/plan/nemotron-s-m-local-scope-2026-08.md
---

# Tasks: Nemotron Local-Developer Scope — S and M Only

## T1 — Record the owner-approved routing decision

- **Status:** [x] Done (2026-08-12)
- **Type:** policy/ADR/docs-only
- **Objective:** Make the authoritative routing documents state that Nemotron is
  the local developer only for S and M, while L is cloud-only after ADR-038's
  evidence gate.
- **In scope:** ADR-036, ADR-038, `RRI_POLICY.md`, `HITL_AUTONOMY_POLICY.md`,
  `AGENT_WORKFLOW_GUIDE.md`, `AGENTS.md`, `AGENTS.override.md`, and the Low-band
  handoff documentation where the active developer binding is named.
- **Out of scope:** reviewer bindings, application runtime code, model installation,
  and historical audit artifacts.
- **Evidence to emit:** ADR amendment and policy synchronization diff.
- **Status artifacts affected:** this ledger, the linked plan, ADR index parity.

**Completion:** ADR-036 Amendment 4 and ADR-038 Amendment 1 record the owner
decision. The RRI/HITL/workflow guidance, generated `AGENTS.override.md`, and
Low-band handoff references are synchronized. `make qa-docs` and `git diff
--check` passed.

## T2 — Bind Low/S delegation to Nemotron

- **Status:** [x] Done (2026-08-12)
- **Type:** development
- **RRI:** 42 Med-high (`C1 F2 D3 T1 A0 K3 P2 X2`, `refactor_and_behavior +8`)
- **Effort:** L
- **Allowed paths:** `scripts/delegate-low-rri.py`,
  `scripts/delegate_low_rri_test.py`, `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`,
  `docs/gemma-local-improve.md`.
- **Full RRI evidence:** `docs/audit/nem-sm-t2-rri.md`.
- **Objective:** Use `nemotron-3.5-lightning:30b-a3b-q4_K_M` by default for an
  eligible Low/S local delegation, without silently falling back to Gemma or Qwen.
- **HP-1:** an eligible Low/S packet with no model override resolves Nemotron.
- **EC-1:** when Nemotron is absent, delegation reports that exact unavailable
  model and does not substitute another local developer.
- **EC-2:** a stalled Nemotron delegation has no implicit alternate-model retry.
- **Evidence to emit:** unit-test output, phase-1/phase-2 review receipts, and
  the task completion record.
- **Status artifacts affected:** this ledger, linked plan, and Low-band handoff docs.
- **Stop condition:** do not alter reviewer model bindings or start T3.

Task-analysis review: muse-glimmer `.agent/peer-task-review-NEM-SM-T2-muse.json` - PASS

Code-solution review: d14 `.agent/peer-code-review-NEM-SM-T2.json` - PASS

### Peer Reviewer evidence

- Reviewer: `d14`
- Command: context-isolated cross-provider review, `gpt-5.6-terra` at `medium`
- Artifact: `.agent/peer-code-review-NEM-SM-T2.json`
- Verdict: PASS
- Findings: none
- Muse Glimmer fallback: triggered — reason: Gemma produced invalid structured output on its initial and immediate retry.
- D14 fallback: triggered — reason: Muse Glimmer returned an empty response.
- D14 provider route: cross-provider — reason: Codex orchestrator → OpenAI D14 reviewer.
- disposition_divergence: none
- Primary-agent disposition: accepted; no findings required repair.

### Reflection log

Required passes: 3 (`RRI 42` → `Med-high`)

#### Pass 1

- **Draft verdict:** the Low/S wrapper defaults to Nemotron and removes the implicit install fallback.
- **Critique findings:** default stalled-path behavior needed a direct test proving no second generation occurs.
- **Revisions applied:** added `test_ec2_default_stall_does_not_retry_another_model` with a one-call assertion.

#### Pass 2

- **Draft verdict:** explicit `--stall-fallback-model` remains opt-in while the default is empty.
- **Critique findings:** the terminal escalation test still expected the retired default fallback wording.
- **Revisions applied:** updated the assertion to expect the direct terminal timeout path.

#### Pass 3

- **Draft verdict:** targeted tests pass and D14 reports no findings.
- **Critique findings:** no issues found.
- **Revisions applied:** none.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | no override resolves Nemotron | `scripts/delegate_low_rri_test.py::CliBehavior::test_hp1_dry_run_uses_nemotron_default_model` | passed |
| EC-1 | Edge case | missing Nemotron does not substitute another developer | `scripts/delegate_low_rri_test.py::ModelResolution::test_ec1_missing_nemotron_does_not_substitute_another_developer` | passed |
| EC-2 | Edge case | default stalled Nemotron does not retry another model | `scripts/delegate_low_rri_test.py::StallFallback::test_ec2_default_stall_does_not_retry_another_model` | passed |

### Owner final verification

- Owner: Codex (orchestrator of record)
- Date: 2026-08-12
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/delegate_low_rri_test.py`; `python3 scripts/gemma_local_test.py`; `git diff --check`
- Review artifact: docs/audit/gemma-evidence/NEM-SM-T2.json

## T3 — Enforce cloud-only Med-high/L execution

- **Status:** [x] Done (2026-08-12)
- **Type:** development
- **RRI:** 47 Med-high (`C1 F1 D4 T1 A0 K4 P3 X2`, `refactor_and_behavior +8`)
- **Effort:** L
- **Allowed paths:** `scripts/local-agent/run_med_high_task.py`,
  `scripts/local-agent/run_med_high_task_test.py`.
- **Full RRI evidence:** `docs/audit/nem-sm-t3-rri.md`.
- **Objective:** Preserve ADR-038 gate validation, but route every valid
  Med-high `GO_LOCAL` outcome to a cloud handoff bundle before a local runner
  can start.
- **HP-1:** a valid `GO_LOCAL` gate result yields a cloud-required bundle and
  launches no local process.
- **EC-1:** a malformed gate input remains fail-closed and still yields a
  cloud handoff bundle.
- **Evidence to emit:** unit-test output, phase-1/phase-2 review receipts, and
  the task completion record.
- **Status artifacts affected:** this ledger and linked plan.
- **Stop condition:** do not change the ADR-038 reviewer, approval, or fallback
  selection protocols.

Task-analysis review: d14 `.agent/peer-task-review-NEM-SM-T3.json` - PASS

Code-solution review: gemma `.agent/peer-code-review-NEM-SM-T3-gemma-retry.json` - PASS

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: Ollama `/api/chat`, `gemma4:26b-a4b-it-qat`, `think=false`
- Artifact: `.agent/peer-code-review-NEM-SM-T3-gemma-retry.json`
- Verdict: PASS
- Findings: none
- Muse Glimmer fallback: not triggered — reason: Gemma's immediate retry returned valid PASS JSON.
- D14 fallback: not triggered for phase 2 — reason: n/a
- D14 provider route: n/a — reason: phase 1 used cross-provider `gpt-5.6-terra` after Gemma/Muse were unusable.
- disposition_divergence: none
- Primary-agent disposition: accepted; no findings required repair.

### Reflection log

Required passes: 3 (`RRI 47` → `Med-high`)

#### Pass 1

- **Draft verdict:** `supervise()` stops every valid Med-high route before the local-runner branch.
- **Critique findings:** the original GO_LOCAL success test still asserted that a local process launched.
- **Revisions applied:** replaced it with an assertion that `popen` is never called and a cloud handoff exists.

#### Pass 2

- **Draft verdict:** no Med-high path reaches the runner, so historic post-launch expectations were unreachable.
- **Critique findings:** four integration tests asserted historical runner statuses through `supervise()`.
- **Revisions applied:** updated them to assert the policy handoff and retained direct runner helper coverage separately.

#### Pass 3

- **Draft verdict:** all 50 supervisor tests pass; Gemma's valid retry reports no findings.
- **Critique findings:** no issues found.
- **Revisions applied:** none.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid GO_LOCAL launches no local process and produces cloud handoff | `scripts/local-agent/run_med_high_task_test.py::SuperviseIntegrationTest::test_hp1_go_local_is_policy_excluded_and_never_launches_runner` | passed |
| EC-1 | Edge case | malformed gate input remains fail-closed with cloud handoff | `scripts/local-agent/run_med_high_task_test.py::SuperviseIntegrationTest::test_ec2_gate_rejection_before_any_launch_emits_bundle` | passed |

### Owner final verification

- Owner: Codex (orchestrator of record)
- Date: 2026-08-12
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/local-agent/run_med_high_task_test.py`; `git diff --check`
- Review artifact: docs/audit/gemma-evidence/NEM-SM-T3.json

## T4 — Verify and close the routing change

- **Status:** [x] Done (2026-08-12)
- **Type:** verification/status-sync
- **Depends on:** T2, T3
- **Objective:** Verify S/M/L defaults and synchronize the closure evidence.

**Verification (2026-08-12):**

- With `DUBBRIDGE_LOW_RRI_MODEL` unset, `scripts/delegate-low-rri.py --dry-run`
  resolves `nemotron-3.5-lightning:30b-a3b-q4_K_M` for S/Low.
- `run_local_task.py` resolves the same default for M/Moderate.
- `run_med_high_task.py` contains the only `supervise()` path for a valid
  Med-high `GO_LOCAL` decision and emits `policy_excluded_local_execution`
  before any `run_supervised_runner()` call.
- `python3 scripts/delegate_low_rri_test.py` (102 tests),
  `python3 scripts/local-agent/run_med_high_task_test.py` (50 tests),
  `make qa-docs`, and `git diff --check` passed.
