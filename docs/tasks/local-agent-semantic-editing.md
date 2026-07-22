---
type: TaskList
title: "Tasks: Local-agent semantic editing"
status: superseded
plan: docs/plan/local-agent-simple-editing.md
---

# Tasks: Local-agent semantic editing

> **Superseded (2026-07-22) by `docs/plan/local-agent-simple-editing.md`.**
> `LASE-T1`–`T5` shipped the Serena/semantic path; `LASE-T6` (the live pilot)
> proved it never converged on an edit. Serena and the semantic tools were then
> removed in favor of a simple read/write/patch runner. `scope_check.py` and
> `organization_gate.py` (from `LASE-T4`) are retained and reused. This ledger
> is kept for history; the active work is tracked in the simple-editing plan.

Governing plan: `docs/plan/local-agent-semantic-editing.md`
Governing ADR: `docs/adr/ADR-036-local-first-agentic-implementation-band.md`

## Status

- [x] `LASE-T1` Serena bootstrap and Rust index
- [x] `LASE-T2` Serena MCP adapter
- [x] `LASE-T3` Symbol-first runner tools
- [x] `LASE-T4` Organization gate
- [x] `LASE-T5` Workflow enforcement and implementer signature
- [ ] `LASE-T6` S-140 live pilot — infra unblocked; local model did not converge, escalation decision pending

## LASE-T1 - Serena bootstrap and Rust index

**Status:** Done
**Effort:** M
**Complexity:** Moderate (`RRI 32`)
**Depends on:** none
**Recommended model:** Codex `GPT-5.2-Codex` / Claude Code `Claude Sonnet 4`

### Objective

Install a pinned Serena release through `uv`, create minimal committed project
configuration for DubBridge, and prove that its Rust language-server backend can
index the workspace. This is config-only: do not add a bootstrap script or modify
the local runner.

### Inputs

- Existing `uv` and `rust-analyzer` installations.
- Serena `1.5.3`, pinned rather than floating on latest.
- DubBridge workspace root.

### Outputs

- Installed `serena` CLI.
- `.serena/project.yml` with every other `.serena/*` path ignored from Git.
- Health-check/index evidence under `.agent/local-agent-semantic-editing/`.

### Acceptance criteria

- `command -v uv` and `uv --version` succeed before any install attempt.
- Workspace root contains readable `Cargo.toml` and `.git`; otherwise fail fast
  before install/index with `DubBridge Rust workspace not detected`.
- `serena --version` reports `1.5.3`.
- `command -v serena` resolves an executable available to later runner tasks.
- Install or replace with `uv tool install --force -p 3.13
  'serena-agent==1.5.3'`; `uv` owns Python 3.13 resolution.
- `serena project index` completes for the Rust workspace.
- `serena project health-check` exits `0`; warnings remain captured in the named
  health-check artifact but do not replace the exit-code requirement.
- A non-zero index or health-check exit is recorded in its artifact and marks
  `LASE-T1` failed; no later task starts and no warning waiver converts it to success.
- Root `.gitignore` contains `.serena/*` followed by
  `!.serena/project.yml`; `git check-ignore` confirms generated memory/cache
  fixtures are ignored while `project.yml` remains trackable.
- No application code or `run_local_task.py` changes are included.

### Evidence to emit

- Exact install/version/index/health-check commands and outputs in
  `.agent/local-agent-semantic-editing/LASE-T1-uv.txt`,
  `LASE-T1-serena-version.txt`,
  `LASE-T1-serena-index.txt`, and `LASE-T1-serena-health-check.txt`.
- Resolved binary path in
  `.agent/local-agent-semantic-editing/LASE-T1-serena-path.txt`.
- Final scope and ignore verification in
  `.agent/local-agent-semantic-editing/LASE-T1-scope.txt`.
- Each evidence file records the command, exit code, stdout, and stderr; the task
  creates `.agent/local-agent-semantic-editing/` before executing commands. Stdout
  and stderr remain in separately labelled sections of the same artifact.
- RRI result and scope diff.
- Task-analysis review artifact.

### RRI

`32` (`Moderate`): two-file config change plus a user-local external tool install
and the explicit choice to adopt Serena for symbol-aware local work.

### Task-analysis review

`n/a` - config-only task exemption.

### Status artifacts affected

- This task ledger only.

### Agent handoff prompt

Install and configure only pinned Serena `1.5.3` for the DubBridge Rust workspace.
Do not integrate it with the runner and do not touch application code. Stop after
version, index, health check, and ignore rules pass.
The procedure must be safe to rerun after an interrupted `uv` operation; it must not
automatically uninstall a Serena installation that may predate this task.

## LASE-T2 - Serena MCP adapter

**Status:** Done
**Effort:** M
**Complexity:** Moderate (`RRI 32`)
**Depends on:** `LASE-T1`

### Objective

Add an isolated adapter that starts Serena in stdio mode and exposes only the
symbol/navigation operations needed by the local runner.

### Constraints

- Production adapter maximum: 250 lines.
- Tests live in a separate file.
- No runner integration in this task.
- Disable Serena shell, broad file-read, and overwrite tools in this context.

### Acceptance criteria

- Adapter can list symbols, read one symbol, find references, and request diagnostics.
- Startup failure and malformed MCP responses fail closed with concise errors.
- Adapter process is terminated after the session.

### Behavioral examples

- `HP-1`: request `process_transcription_job_inner` -> return that symbol and its
  signature/body without returning all of `main.rs`.
- `EC-1`: unknown symbol -> typed not-found result; no fallback to full-file read.
- `EC-2`: Serena unavailable -> runner-facing error; no unstructured fallback.

### Evidence to emit

- Focused unit-test output and adapter line count.

### RRI

`32` (`Moderate`): isolated runner-adapter seam with typed error handling,
stdio MCP framing, and cleanup semantics.

### Task-analysis review

`qwen3.6:27b-q4_K_M` `.agent/peer-task-review-LASE-T2-v5.json` - PASS

### Code-solution review

`qwen3.6:27b-q4_K_M` `.agent/peer-code-review-LASE-T2.json` - BLOCKED

User waiver: residual low-severity packet-review finding explicitly waived by
owner on `2026-07-21`; implementation and local verification proceeded.

### Status artifacts affected

- This task ledger only.

- **Completion record (2026-07-21):**
  - Added [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:1),
    a `203`-line nonblank Serena stdio MCP adapter with a symbol-only temporary
    context, required/forbidden tool validation, JSON-RPC framing, typed
    startup/protocol/not-found/timeout errors, and subprocess/context cleanup.
  - Added focused tests in
    [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:1)
    covering startup success, forbidden-tool rejection, typed symbol-not-found,
    malformed payloads, timeout termination, kill fallback, and temporary
    context-file cleanup.
  - Exhausted the two evidence-backed local-model attempts for this task,
    then completed the adapter directly in the primary checkout without touching
    `run_local_task.py`.

### Happy paths covered

- `HP-1`: targeted symbol lookup returns only the requested symbol payload rather
  than broad file content.
  Code evidence:
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:95)
  narrows `read_symbol()` to one `find_symbol` result with `include_body=True`
  and `max_matches=1`;
  [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:116)
  proves the single-symbol happy path.

### Edge cases covered

- `EC-1`: unknown symbol fails closed with a typed not-found error and no
  fallback to broad file reads.
  Code evidence:
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:100)
  raises `SerenaSymbolNotFoundError`;
  [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:143)
  asserts the exact exception type.
- `EC-2`: Serena unavailable or startup failure returns a concise typed startup
  error.
  Code evidence:
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:74)
  wraps spawn/handshake failures in `SerenaStartupError`;
  [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:89)
  asserts the exact startup failure path.
- Additional fail-closed coverage:
  malformed MCP/tool payloads raise `SerenaProtocolError`, timeouts raise
  `SerenaTimeoutError`, and `close()` removes the temporary context file while
  terminating the subprocess.
  Code evidence:
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:127),
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:139),
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:199),
  [`scripts/local-agent/serena_mcp.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp.py:115),
  [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:164),
  [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:207),
  [`scripts/local-agent/serena_mcp_test.py`](/Users/matias/dubbridge/scripts/local-agent/serena_mcp_test.py:63).

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | requested symbol returns only the targeted symbol payload | `scripts/local-agent/serena_mcp_test.py::SerenaMcpSessionTest.test_read_symbol_returns_single_match` | passed |
| EC-1 | Edge case | unknown symbol returns typed not-found result | `scripts/local-agent/serena_mcp_test.py::SerenaMcpSessionTest.test_unknown_symbol_raises_typed_error` | passed |
| EC-2 | Edge case | Serena unavailable/startup failure returns typed startup error | `scripts/local-agent/serena_mcp_test.py::SerenaMcpSessionTest.test_startup_failure_is_concise` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-21`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m py_compile scripts/local-agent/serena_mcp.py scripts/local-agent/serena_mcp_test.py`; `python3 -m unittest scripts/local-agent/serena_mcp_test.py -v`; `python3 -m trace --count --summary --coverdir .agent/tracecov --module unittest scripts/local-agent/serena_mcp_test.py`

### Agent handoff prompt

Implement only the small Serena MCP adapter and isolated tests. Do not edit
`run_local_task.py`. Keep production code at or below 250 lines.

## LASE-T3 - Symbol-first runner tools

**Status:** Done
**Effort:** M
**Complexity:** Moderate (`RRI 38`)
**Depends on:** `LASE-T2`

### Objective

Replace whole-file source editing in the local runner with Serena symbol tools and
bounded patch/create operations while preserving audit and scope enforcement.

### Acceptance criteria

- Existing source files cannot be replaced through `write_file`.
- Files above 400 lines cannot be returned through a full-file read.
- Tool contract offers symbol overview/read/references/diagnostics and bounded edits.
- New-file creation is limited to `120` lines / `8192` bytes; patches are limited
  to `80` lines / `4096` bytes.
- Audit record lists `semantic_tools[]` and `bounded_edit_metrics[]` entries with
  `tool`, `path`, `line_count`, `byte_count`, and `anchor_matches` when present.

### Behavioral examples

- `HP-1`: model extracts one Rust function into a sibling module using symbol context
  and a bounded call-site patch.
- `HP-2`: model creates a new helper file within the explicit create budget.
- `HP-3`: model requests a symbol overview/reference lookup without broad file reads.
- `EC-1`: model requests full `main.rs` -> rejected with symbol-tool guidance.
- `EC-2`: patch exceeds budget or lacks a unique anchor -> rejected before write.
- `EC-3`: patch anchor matches multiple locations -> rejected rather than applying an
  arbitrary edit.

### Evidence to emit

- Runner unit tests, unchanged scope-check tests, and audit-record fixture.

### RRI

`38` (`Moderate`): runner tool-surface change with semantic context routing,
bounded edit enforcement, audit-schema expansion, and regression-sensitive
integration points.

### Task-analysis review

`qwen3.6:27b-q4_K_M` `.agent/peer-task-review-LASE-T3.json` - BLOCKED

Task card findings from phase 1 were incorporated before closure:
explicit budgets, explicit audit fields, additional HP coverage, and the
ambiguous-anchor edge case. A rerun attempt on `2026-07-21` was interrupted
while the local reviewer stream stalled; owner approval and the incorporated
fixes were used to proceed.

### Status artifacts affected

- This task ledger only.

### Code-solution review

`qwen3.6:27b-q4_K_M` `.agent/peer-code-review-LASE-T3.json` - BLOCKED

User waiver: local reviewer returned non-blocking findings first, those findings
were addressed (atomic create path plus explicit budget tests), and the rerun
was still in progress when owner chose to keep progress moving on `2026-07-21`.

- **Completion record (2026-07-21):**
  - Added [`scripts/local-agent/runner_semantic_tools.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools.py:1),
    a `193`-line helper that enforces the symbol-first tool contract, blocks
    broad existing-source overwrites, caps full-file reads at `400` lines,
    enforces explicit create/patch budgets, routes Serena semantic calls, and
    keeps `O_NOFOLLOW` on both read/write paths for the final filesystem step.
  - Updated [`scripts/local-agent/run_local_task.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task.py:1)
    to advertise the new tool contract, validate the expanded tool schema,
    delegate semantic/bounded operations through the helper, and emit semantic
    audit fields.
  - Added focused tests in
    [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:1)
    covering large-read rejection, existing-file overwrite rejection,
    successful bounded patching, ambiguous-anchor rejection, and semantic/audit
    propagation without requiring a live Serena process.

### Happy paths covered

- `HP-1`: symbol-first lookup plus bounded existing-file edit is wired through
  the runner and reaches the audit record without broad file replacement.
  Code evidence:
  [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:328)
  drives `list_symbols` followed by `apply_patch` through `run_local_task.py`
  and proves the semantic tool and bounded patch metrics are both recorded.
- `HP-2`: bounded creation of a new file remains available for genuinely new
  paths.
  Code evidence:
  [`scripts/local-agent/run_local_task_test.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task_test.py:101)
  proves `write_file` still creates a new file successfully in the runner path.
- `HP-3`: the runner now exposes symbol overview/read/reference/diagnostic calls
  without requiring broad file reads.
  Code evidence:
  [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:56)
  proves `read_symbol`, `find_references`, and `get_diagnostics` route through
  the Serena session interface.

### Edge cases covered

- `EC-1`: broad full-file reads above `400` lines fail closed with semantic
  guidance.
  Code evidence:
  [`scripts/local-agent/runner_semantic_tools.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools.py:82)
  rejects oversized reads;
  [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:93)
  proves the runner records the rejection and recovers.
- `EC-2`: patches above the explicit budget are rejected before any write.
  Code evidence:
  [`scripts/local-agent/runner_semantic_tools.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools.py:179)
  enforces bounded metrics;
  [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:255),
  [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:291)
  prove line and byte budget rejection.
- `EC-3`: ambiguous anchors fail closed rather than applying arbitrary edits.
  Code evidence:
  [`scripts/local-agent/runner_semantic_tools.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools.py:143)
  requires exactly one anchor match;
  [`scripts/local-agent/runner_semantic_tools_test.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools_test.py:218)
  proves the rejection path.
- Additional fail-closed coverage:
  atomic `write_file` creation plus `O_NOFOLLOW` keep symlink/TOCTOU writes from
  escaping the worktree.
  Code evidence:
  [`scripts/local-agent/runner_semantic_tools.py`](/Users/matias/dubbridge/scripts/local-agent/runner_semantic_tools.py:94),
  [`scripts/local-agent/integration_test.py`](/Users/matias/dubbridge/scripts/local-agent/integration_test.py:381)
  and
  [`scripts/local-agent/integration_test.py`](/Users/matias/dubbridge/scripts/local-agent/integration_test.py:405).

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | symbol context plus bounded existing-file patch flows through the runner and audit | `scripts/local-agent/runner_semantic_tools_test.py::SemanticRunnerContract.test_semantic_tool_usage_and_bounded_metrics_reach_audit_record` | passed |
| HP-2 | Happy path | new-file creation remains available within the create budget | `scripts/local-agent/run_local_task_test.py::HP1ToyCardCompletes.test_success_writes_diff_and_transcript` | passed |
| HP-3 | Happy path | symbol read/reference/diagnostic calls route through the semantic session interface | `scripts/local-agent/runner_semantic_tools_test.py::SemanticRunnerContract.test_symbol_read_reference_and_diagnostics_route_through_session` | passed |
| EC-1 | Edge case | full-file read above 400 lines is rejected with semantic guidance | `scripts/local-agent/runner_semantic_tools_test.py::SemanticRunnerContract.test_large_read_file_is_rejected_with_symbol_guidance` | passed |
| EC-2 | Edge case | patch over explicit line/byte budget is rejected before write | `scripts/local-agent/runner_semantic_tools_test.py::SemanticRunnerContract.test_patch_budget_exceeded_is_rejected`; `scripts/local-agent/runner_semantic_tools_test.py::SemanticRunnerContract.test_patch_byte_budget_exceeded_is_rejected` | passed |
| EC-3 | Edge case | ambiguous anchor match is rejected instead of applying an arbitrary patch | `scripts/local-agent/runner_semantic_tools_test.py::SemanticRunnerContract.test_ambiguous_patch_anchor_is_rejected` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-21`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m py_compile scripts/local-agent/run_local_task.py scripts/local-agent/runner_semantic_tools.py scripts/local-agent/runner_semantic_tools_test.py`; `python3 -m unittest scripts/local-agent/runner_semantic_tools_test.py -v`; `python3 -m unittest scripts/local-agent/run_local_task_test.py -v`; `python3 -m unittest scripts/local-agent/integration_test.py -v`

### Agent handoff prompt

Integrate only the proven Serena adapter into the runner. Remove whole-file editing
for existing source, enforce bounded operations, and preserve scope/audit semantics.

## LASE-T4 - Organization gate

**Status:** Done
**Effort:** M
**Complexity:** Moderate (`RRI 35`)
**Depends on:** `LASE-T3`

### Objective

Add an isolated deterministic checker for project-specific source organization.

### Acceptance criteria

- Gate rejects new monolithic growth, composition-root business logic, and new broad
  lint suppressions in changed production code.
- Checker implementation is at most 250 production lines; tests are separate.
- Output is machine-readable and distinguishes pass, violation, and tool failure.

### Behavioral examples

- `HP-1`: thin `main.rs` call site + focused sibling module -> organization gate passes.
- `EC-1`: local model adds another handler body to `main.rs` -> gate reports a violation.
- `EC-2`: tests pass but file-growth budget fails -> gate still reports a violation.

### Evidence to emit

- Focused checker tests, output fixtures, and production line count.

### RRI

`35` (`Moderate`): isolated diff checker with repo-specific structural heuristics,
machine-readable CLI output, and a small focused test seam.

### Task-analysis review

`qwen3.6:27b-q4_K_M` `.agent/peer-task-review-LASE-T4.json` - BLOCKED

Owner-directed proceed note: implementation continued on `2026-07-21` without
waiting for a local reviewer pass artifact.

### Code-solution review

`qwen3.6:27b-q4_K_M` `.agent/peer-code-review-LASE-T4.json` - BLOCKED

Owner-directed proceed note: completion is backed by focused local tests and a
production line-count check.

### Status artifacts affected

- This task ledger only.

- **Completion record (2026-07-21):**
  - Added [`scripts/local-agent/organization_gate.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate.py:1),
    a `161`-line deterministic checker that inspects changed production-code diffs
    for file-growth budget overruns, composition-root leakage in `main.rs`, and
    newly added lint suppressions.
  - Added focused tests in
    [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:1)
    covering the thin-`main.rs` happy path, composition-root rejection, file-growth
    rejection, lint-suppression rejection, diff parsing for new files, and
    machine-readable tool-failure output.

### Happy paths covered

- `HP-1`: a thin `main.rs` call-site change plus a focused sibling module passes.
  Code evidence:
  [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:25)
  asserts no violations for that shape.

### Edge cases covered

- `EC-1`: business logic added to `main.rs` is rejected as composition-root leakage.
  Code evidence:
  [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:37)
  asserts the `composition_root` rule.
- `EC-2`: a file-growth overrun still fails even if the diff would otherwise be valid.
  Code evidence:
  [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:46)
  asserts the `file_growth` rule.
- Additional fail-closed coverage:
  new lint suppressions are rejected, new-file diff parsing preserves add/modify
  status, and CLI exceptions return `tool_failure` JSON.
  Code evidence:
  [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:53),
  [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:59),
  [`scripts/local-agent/organization_gate_test.py`](/Users/matias/dubbridge/scripts/local-agent/organization_gate_test.py:73).

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | thin `main.rs` plus focused module passes | `scripts/local-agent/organization_gate_test.py::OrganizationGateTest.test_hp1_thin_main_and_focused_module_pass` | passed |
| EC-1 | Edge case | business logic in `main.rs` is rejected | `scripts/local-agent/organization_gate_test.py::OrganizationGateTest.test_ec1_business_logic_in_main_is_rejected` | passed |
| EC-2 | Edge case | file-growth overrun is rejected | `scripts/local-agent/organization_gate_test.py::OrganizationGateTest.test_ec2_existing_file_growth_budget_is_rejected` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-21`
- Statement: I verified the defined happy path and edge cases have focused unit-test evidence and the production checker stays under the `250`-line cap.
- Commands run: `python3 -m py_compile scripts/local-agent/organization_gate.py scripts/local-agent/organization_gate_test.py`; `python3 -m unittest scripts/local-agent/organization_gate_test.py -v`; `wc -l scripts/local-agent/organization_gate.py scripts/local-agent/organization_gate_test.py`

### Agent handoff prompt

Implement only the organization checker and focused tests. Do not integrate it with
the runner and do not modify S-140 code.

## LASE-T5 - Workflow enforcement and implementer signature

**Status:** Done
**Effort:** M
**Complexity:** Med-high (`RRI 44`)
**Depends on:** `LASE-T4`
**Recommended model:** Codex `GPT-5.2-Codex` / Claude Code `Claude Sonnet 4`

### Objective

Make semantic preflight and the organization gate mandatory in the RRI 26-55 local
runner, bind both to the `local-implementer` signature, and encode the same fail-closed
contract in the canonical workflow documentation.

### Acceptance criteria

- Existing source tasks start Serena against the disposable worktree and require a
  successful health/index preflight before model execution.
- Docs/config-only and new-file-only tasks may record a typed semantic exemption;
  existing source changes may not.
- Serena failure makes the local route ineligible and emits escalation evidence; no
  fallback exposes broad source reads or whole-file replacement.
- Organization, scope, and independent acceptance gates run before success signing.
- A success audit records semantic context, tools used, changed line/byte budgets,
  organization result, scope result, verification results, implementer model, and
  the `local-implementer` signature.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `docs/policies/HITL_AUTONOMY_POLICY.md`, and `docs/policies/RRI_POLICY.md` state
  the mandatory rule for future RRI 26-55 development tasks.

### Behavioral examples

- `HP-1`: semantic preflight + bounded edit + all gates pass -> signature emitted.
- `EC-1`: Serena is unavailable -> unsigned escalation artifact, no model run.
- `EC-2`: tests pass but organization fails -> no success signature.
- `EC-3`: audit lacks semantic evidence -> artifact validation fails.

### Evidence to emit

- Runner tests, audit schema fixtures, policy consistency check, and signature-order
  test proving no success signature exists before every mandatory gate passes.

### RRI

`44` (`Med-high`): runner workflow enforcement across code plus canonical policy
docs, with fail-closed audit/signature semantics.

### Task-analysis review

`claude` `.agent/peer-task-review-LASE-T5.json` - BLOCKED

User-directed proceed: owner asked to continue immediately after formal task
presentation without waiting for a fresh external review artifact.

### Code-solution review

`claude` `.agent/peer-code-review-LASE-T5.json` - BLOCKED

User-directed proceed: owner prioritized immediate delivery with local runner
tests and signature-order evidence as the active verification set.

### Status artifacts affected

- This task ledger.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`

- **Completion record (2026-07-21):**
  - Added [`scripts/local-agent/runner_workflow_gate.py`](/Users/matias/dubbridge/scripts/local-agent/runner_workflow_gate.py:1)
    to keep semantic preflight and organization-gate execution out of the large
    runner file while enforcing typed exemptions, Serena health/index startup
    checks, and worktree-local organization evaluation.
  - Updated [`scripts/local-agent/run_local_task.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task.py:56)
    so semantic preflight runs before any model turn, organization/scope/
    acceptance gates feed audit validation, and the `local-implementer`
    signature is emitted only when every mandatory gate passes.
  - Extended [`scripts/local-agent/run_local_task_test.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task_test.py:1472)
    with preflight-failure, organization-failure, and missing-semantic-evidence
    coverage for unsigned audits and strict signature ordering.
  - Synchronized the canonical workflow/policy docs so future RRI 26–55 local
    tasks inherit the same mandatory preflight, gate order, and signature rules.

### Happy paths covered

- `HP-1`: semantic preflight/exemption plus passing gates emits the
  `local-implementer` signature.
  Code evidence:
  [`scripts/local-agent/run_local_task.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task.py:792)
  records semantic preflight before execution and rejects failed preflight
  before model turns;
  [`scripts/local-agent/run_local_task.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task.py:840)
  signs only validated success audits;
  [`scripts/local-agent/run_local_task_test.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task_test.py:1495)
  proves the signed happy path.

### Edge cases covered

- `EC-1`: Serena unavailable yields unsigned escalation evidence and no model run.
  Code evidence:
  [`scripts/local-agent/runner_workflow_gate.py`](/Users/matias/dubbridge/scripts/local-agent/runner_workflow_gate.py:69)
  fails closed on Serena CLI/MCP startup failure;
  [`scripts/local-agent/run_local_task_test.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task_test.py:1616)
  asserts `preflight_failed` with an unsigned audit.
- `EC-2`: tests may pass while organization fails, but success is not signed.
  Code evidence:
  [`scripts/local-agent/run_local_task.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task.py:529)
  runs organization evaluation after a passing acceptance signal and converts a
  non-pass result into `organization_violation`;
  [`scripts/local-agent/run_local_task_test.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task_test.py:1645)
  proves the signature remains unsigned.
- `EC-3`: missing semantic evidence invalidates a would-be success audit.
  Code evidence:
  [`scripts/local-agent/run_local_task.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task.py:761)
  validates success-audit prerequisites and downgrades invalid success to
  `audit_invalid`;
  [`scripts/local-agent/run_local_task_test.py`](/Users/matias/dubbridge/scripts/local-agent/run_local_task_test.py:1661)
  asserts the missing-semantic-evidence failure.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | semantic preflight/exemption plus passing gates emits signature | `scripts/local-agent/run_local_task_test.py::AuditLogEmission.test_hp1_success_emits_audit_record` | passed |
| EC-1 | Edge case | Serena unavailable yields unsigned escalation and no model run | `scripts/local-agent/run_local_task_test.py::AuditLogEmission.test_ec4_preflight_failure_emits_unsigned_audit_and_skips_model` | passed |
| EC-2 | Edge case | organization failure after passing tests blocks signature | `scripts/local-agent/run_local_task_test.py::AuditLogEmission.test_ec5_organization_failure_blocks_success_signature_after_tests_pass` | passed |
| EC-3 | Edge case | missing semantic evidence invalidates success audit | `scripts/local-agent/run_local_task_test.py::AuditLogEmission.test_ec6_missing_semantic_evidence_invalidates_success_audit` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-21`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m py_compile scripts/local-agent/runner_workflow_gate.py scripts/local-agent/run_local_task.py scripts/local-agent/run_local_task_test.py`; `python3 -m unittest scripts/local-agent/run_local_task_test.py -v`

### Agent handoff prompt

Integrate the proven adapter and organization checker into the runner, then update
the three canonical workflow/policy documents. Fail closed and never re-enable broad
whole-file source tools.

## LASE-T6 - S-140 live pilot

**Status:** Unblocked — infra fixed and verified; local model did not converge (turn-budget exhausted, zero edits); escalation decision pending owner input
**Effort:** M
**Depends on:** `LASE-T5`

### Objective

Rerun approved `S-140-T2b-i` through the enforced semantic path and use its evidence
as the first end-to-end proof of the permanent workflow.

### Acceptance criteria

- `S-140-T2b-i` produces a focused module extraction and bounded call-site edits.
- No `SubtitleJob` is enqueued by the targeted completion path.
- Worker-runner tests and all task-specific verification commands pass.
- Organization and scope gates pass before a complete `local-implementer` signature.
- No source file is supplied to or regenerated by the model in full.

### Evidence to emit

- Semantic-tool trace, bounded diff statistics, gate outputs, tests, and signed
  local-run artifact.

### Status artifacts affected

- This ledger, `docs/tasks/s-140-subtitle-generation.md`, and linked S-140 plan.

### Agent handoff prompt

Rerun only approved `S-140-T2b-i` through the enforced path and collect the complete
audit evidence. Do not begin `S-140-T2b-ii`.

### Execution record

- 2026-07-21 pilot attempt ran `scripts/local-agent/run_local_task.py` against
  `.agent/local-runs/s-140-t2b-i/S-140-T2b-i.card.json` in detached worktree
  `.agent/worktrees/s-140-t2b-i-live`.
- Serena CLI preflight passed after warming the worktree:
  `serena project index .` and `serena project health-check .` both exited 0.
- The enforced MCP gate still failed before the first model turn:
  `SerenaMcpSession.start()` returned
  `timed out waiting for Serena MCP response`.
- Artifact:
  `.agent/local-runs/s-140-t2b-i/S-140-T2b-i.live.run.json`
  finished `2026-07-21T21:12:25Z` with `status=preflight_failed`,
  `reason=serena_start_failed`.
- No product-code diff was produced; no bounded edit metrics, scope gate,
  organization gate, or signed `local-implementer` result exists for this run.

- **2026-07-22 — five infra blockers found and fixed against the live worktree,
  then two full pilot reruns:**

  1. **MCP wire-protocol mismatch** (`scripts/local-agent/serena_mcp.py`):
     `_send`/`_recv` now use newline-delimited JSON per message (the real
     `mcp.server.stdio.stdio_server` framing), not LSP-style
     `Content-Length` framing. This was the direct cause of the
     `2026-07-21` `serena_start_failed` timeout above.
  2. **`.serena/` vs `scope_check` collision**: Serena's default
     `project_serena_folder_location: "$projectDir/.serena"` (global
     `~/.serena/serena_config.yml`) wrote cache/logs/memories inside the
     disposable worktree itself; `scope_check.py`'s artifact allowlist
     (`_ARTIFACT_DIR_NAMES`) deliberately does not exempt `.serena` (it is
     not a build/dependency artifact), so every real pilot run failed
     `scope_check` on Serena's own bookkeeping, not on any task edit.
     Reproduced live: `check_scope()` against the untouched worktree
     returned `in_scope=False` with 9 offending `.serena/*` paths and zero
     real edits. Fixed by relocating
     `project_serena_folder_location` to
     `/Users/matias/.serena/projects-metadata/$projectFolderName` (a global,
     outside-any-worktree location keyed by folder name) rather than
     widening `scope_check`'s allowlist, which would have blurred its
     fail-closed intent. Verified: `check_scope()` on the same worktree,
     same `allowed_paths`, now returns `in_scope=True`, 0 offending paths.
  3. **`serena project index .` interactive prompt**
     (`scripts/local-agent/runner_workflow_gate.py`): auto-creating
     `project.yml` for a never-indexed worktree prompts interactively per
     detected non-primary language (e.g. `Enable typescript? [y/N]`);
     `run_semantic_preflight`'s `subprocess.run` has no stdin, so the prompt
     hit EOF and failed closed before any model turn. Fixed by passing
     `--language rust` explicitly — matching, not overriding, the repo's own
     canonical `.serena/project.yml` (`languages: [rust]`).
  4. **`serena project health-check .` stray artifact**
     (`scripts/local-agent/runner_workflow_gate.py`): this subcommand writes
     its own `.serena/logs/health-checks/*.log` inside the worktree
     regardless of the relocated `project_serena_folder_location` (fix #2
     only covers the project's main data folder, not this command's log
     sink), reintroducing the exact `scope_check` collision fix #2 closed.
     Fixed by having `run_semantic_preflight` remove any `.serena/`
     directory from the worktree immediately after the CLI preflight
     commands run, before the MCP session or any model turn — again at the
     artifact source rather than widening `scope_check`'s allowlist.
  5. **Uncaught `SerenaAdapterError` crash in `run_loop`**
     (`scripts/local-agent/run_local_task.py`): found on the first full
     rerun after fixes 1-4 — the pilot reached real model turns (16 real
     semantic tool calls, including successful `read_symbol` calls against
     `process_transcription_envelope` and other live symbols), then a
     `list_symbols` call hit a malformed/empty response from the live
     Serena session and raised `SerenaProtocolError` (a `SerenaAdapterError`
     subclass) from inside `apply_tool_call`. `run_loop` only caught
     `MalformedToolCall`/`BoundaryViolation` around that call, so the
     exception escaped uncaught and crashed the whole process with a
     traceback (worktree left clean; the crash occurred on a read-only
     semantic call before any write). Fixed by adding an
     `except SerenaAdapterError` branch sharing the existing
     `malformed_bounces` budget — a single semantic-tool hiccup is now
     bounced back to the model as a retryable tool failure instead of
     killing an otherwise-healthy session. Added regression test
     `run_local_task_test.py::EC2MalformedToolCall.test_semantic_tool_error_is_bounced_not_crashed`.
     Full suite after all five fixes: 129/129 passing.

  **Pilot rerun #1** (post fixes 1-4, before fix 5): semantic preflight
  passed (`status=passed`), post-preflight `scope_check` clean
  (`in_scope=True`, 0 offending paths) — proving fixes 1-4. The model then
  made 16 real semantic tool calls before hitting the fix-5 crash; process
  exited non-zero with an uncaught traceback; checkpointed artifact shows
  `status=in_progress, turn=15`; zero worktree diff (crash was on a
  read-only call).

  **Pilot rerun #2** (post fix 5, full run to completion): exit code `0`.
  Semantic preflight and scope gate both clean as above. The model spent
  its first ~18 turns correctly using `read_file`/`list_symbols`/
  `read_symbol` against real `main.rs` symbols (`enqueue_transcription_if_ready`,
  `process_transcription_job_inner`, `process_transcription_envelope`,
  `main`, etc. — proving the symbol-first tool contract works end-to-end
  against a live Serena session), then switched to paging through the file
  manually via `run_command` (`head`/`sed -n`/`wc -l`) for the remaining
  turns instead of using `apply_patch`/`write_file`. It never attempted an
  edit and never called `finish`. Result:
  `status=budget_exhausted`, `reason=total_turns_exhausted` (`MAX_TOTAL_TURNS=30`),
  worktree diff empty (fail-safe: no partial/out-of-scope edit was left
  behind). This is a turn-budget exhaustion prior to any `finish` call, not
  a failed acceptance-test repair cycle — `MAX_REPAIR_ATTEMPTS` was never
  entered. Artifact:
  `.agent/local-runs/s-140-t2b-i/S-140-T2b-i.live.run.json`.

  **Outcome:** all five infra/runner blockers are fixed and empirically
  verified end-to-end against the real worktree; the enforced semantic path
  (preflight → symbol-first tools → scope gate → organization gate) is now
  provably reachable and survives a full real session without crashing.
  The pilot's acceptance criteria (module extraction, passing tests, signed
  `local-implementer` result) are **not yet met** — the local model did not
  converge on an edit within the turn budget. Per
  `docs/policies/RRI_POLICY.md` (Moderate band, `S-140-T2b-i` RRI 36), the
  local-first route allows at most 2 evidence-backed repair attempts before
  escalating to cloud implementation; this run never reached `finish`, so no
  repair attempt was consumed. Next step (owner decision required): retry
  locally with either a larger `MAX_TOTAL_TURNS` budget or a repair-attempt
  cycle seeded with the existing transcript, or escalate directly to cloud
  implementation per the ADR-036 escalation path.
