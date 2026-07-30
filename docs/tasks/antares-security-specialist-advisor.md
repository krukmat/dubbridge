---
type: TaskList
title: "Tasks: Antares bounded vulnerability-localization advisor"
plan: docs/plan/antares-security-specialist-advisor.md
status: proposed
slice: antares-security-specialist-advisor
---

# Tasks: Antares Bounded Vulnerability-Localization Advisor

## Revision note (2026-07-29)

This ledger was **materially rewritten** after the model's real capability was
verified against Cisco's release material and the `fdtn-ai/antares-1b` model card.
See `docs/plan/antares-security-specialist-advisor.md` § Corrections against the
previous revision.

The revision also separates the **security-advisor workflow** from the Antares
model capability. A primary agent or human security specialist may formulate a
CWE hypothesis, threat model, tests, and remediation advice. Antares may then be
used as one bounded localization tool over an existing repository snapshot. It
does not perform those specialist responsibilities itself.

## Objective

Evaluate Antares as a **CWE-directed, advisory-only, observe-first** localization
tool at bounded refinement, post-implementation, and post-CI touchpoints. It may
inform the primary security advisor, but it never enters RRI authority, the
band-routed reviewer chain, or task closure.

## Creation-task RRI and review exemption

This documentation package is plan/task-ledger-only work, exempt from the
development-task review gates.

- `Task-analysis review: n/a - plan/task-ledger-only exemption`
- `Code-solution review: n/a - plan/task-ledger-only exemption`

Recompute RRI with `scripts/rri.py` immediately before executing every task below.
The values in the summary table are **planning estimates, not authority**.

## Delegation constraint

Several tasks below will create files in `scripts/antares/`. The target-file-size
gate for local-first delegation (`docs/policies/RRI_POLICY.md`) applies: keep each
harness module under 500 lines so RRI 26–55 local delegation stays viable. If a
module cannot be kept under the threshold, decompose it before delegating rather
than escalating by default.

## Task order

```text
T0 (done) -> T0a -> T1 -> T2a -> T2b -> T2c (decomposed: T2c-1 -> T2c-2) -> T2d -> T2e -> T3 -> T4 -> T5
```

## Task summary

| Task | Status | Preliminary RRI | Effort | Depends on |
|---|---|---:|---|---|
| T0 Define role charter and authority boundary | `[x] Done` | 31 Moderate (execution) | M | - |
| T0a Correct charter and close design gaps | `[x] Done` | 47 Med-high (execution) | L | T0 |
| T1 Runtime and model-access preflight | `[x] Done (owner-waived)` | 49 Med-high (technical preflight blocked) | L | T0a |
| T2 Sandboxed agentic harness and artifact schema | `[~] Decomposed (2026-07-29)` | 86 Very high (pre-execution) | XL | T1 |
| T2a Tool-call parser and terminal-state contract | `[x] Done (2026-07-29)` | 45 Med-high (execution) | L | T1 |
| T2b Command allowlist and canonical path containment | `[x] Done (2026-07-30)` | 50 Med-high (execution) | L | T2a |
| T2c Ephemeral sandbox runner and resource enforcement | `[~] Decomposed (2026-07-30)` | 56 Complex (pre-decomposition) | L | T2b |
| T2c-1 Sandbox process execution and isolation | `[x] Done (2026-07-30)` | 49 Med-high (planning) | L | T2b |
| T2c-2 Resource budget, wall-timeout, teardown | `[x] Done (owner-waived, 2026-07-30)` | 53 Med-high (execution) | L | T2c-1 |
| T2d Versioned artifact schema and redacted trace contract | `[x] Done (owner-waived, 2026-07-30)` | 50 (Med-high) | S/M-equivalent | T2c-2 |
| T2e-pre Decompose oversized T2c-2/T2d modules for local-first delegation eligibility | `[ ] Open` | 52 Med-high | L | T2c-2, T2d |
| T2e Replay fixtures and integrated harness verification | `[ ] Open` | 44 Med-high (preliminary) | L | T2e-pre |
| T3 CWE watchlist and context-complete packet construction | `[ ] Open` | Recompute | TBD | T2e |
| T4 Ground-truth calibration and observe-only workflow pilot | `[ ] Open` | Recompute | TBD | T2e, T3 |
| T5 Promote, narrow, or retire on evidence | `[ ] Open` | Recompute | TBD | T4 |

## T0 - Define role charter and authority boundary

- **Status:** `[x] Done` — 2026-07-29
- **Type:** docs / workflow policy
- **Effort:** M
- **Preliminary RRI:** 18 Low
- **Execution RRI:** 31 Moderate
- **Depends on:** none

### Objective

Record Antares as a read-only security advisor with explicit authority limits.

### Acceptance criteria

- Antares is described as advisory only.
- `scripts/rri.py` remains the only canonical RRI authority.
- Human approval, the primary agent, and the band-routed review chain remain
  authoritative.
- A required human disposition state is defined for every Antares finding.

### Completion record (2026-07-29)

- Defined Antares as a bounded read-only advisor with an explicit non-authority
  boundary in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.
- Synchronized HITL wording in `docs/policies/HITL_AUTONOMY_POLICY.md`.
- Recorded the T0 boundary in the slice plan.
- Verification: `make qa-docs`
- RRI artifact: `docs/audit/antares-t0-rri.md`

### Post-hoc correction notice (2026-07-29)

The **authority** boundary written by T0 is correct and survives this revision
unchanged — Antares still approves nothing, computes no RRI, and replaces no
reviewer.

The **capability** description written by T0 is not accurate and is corrected by
T0a. The charter states Antares may suggest "threat surfaces, security rationale,
and recommended follow-up tests"; the model card states it cannot generate
vulnerability explanations and is not an instruction-following model. Those three
claims describe capabilities the model does not have.

This does not reopen T0 — its acceptance criteria were all about authority, and all
four are still satisfied. T0a carries the capability correction.

## T0a - Correct charter and close design gaps

- **Status:** `[x] Done` - 2026-07-29
- **Type:** docs / workflow policy
- **Effort:** L
- **Execution RRI:** 47 Med-high
- **RRI artifact:** `docs/audit/antares-t0a-rri.md`
- **Depends on:** T0

### Objective

Align the Antares charter and implementation plan with the documented model
capability, while closing the sandbox, provenance, evaluation, runtime, scope, and
finding-consumption gaps before any implementation task starts.

### Acceptance criteria

- The charter describes Antares as a **CWE-directed file-level localizer**.
- The claims "threat surfaces", "security rationale", and "recommended follow-up
  tests" are removed or restated as primary-agent responsibilities.
- The charter states that a CWE identifier is a required **input**.
- The charter states the output is a **ranked file list plus exploration trace**.
- The charter defines three optional advisory touchpoints: refinement over the
  existing baseline, post-implementation over the candidate snapshot, and post-CI.
- Every touchpoint requires an externally supplied CWE hypothesis and records a
  skip reason when no justified CWE exists.
- Threat modeling, security rationale, tests, remediation, and finding disposition
  remain responsibilities of the primary agent or human security specialist.
- Antares never computes RRI, replaces the reviewer of record, gates CI, or blocks
  closure.
- File F1 `0.209` is described as a macro-average that indicates substantial
  localization uncertainty, not as a per-output failure probability.
- The future harness contract defines argv-only command execution, option-level
  allowlisting, path/symlink containment, network and credential isolation,
  resource/output/time limits, and teardown.
- The artifact contract includes schema and component versions, model provenance,
  hashes, terminal-state enums, candidate validation, and human disposition audit
  fields.
- Runtime access, pinned model artifact, macOS ARM64 compatibility, memory, and
  latency are a fail-closed preflight before harness implementation.
- Evaluation separates ground-truth calibration from operational triage metrics;
  human rejection is not labeled a false positive without adjudicated truth.
- Finding ownership, SLA, deduplication, retention/redaction, and follow-up task
  linkage are defined before the pilot.
- Scope narrowing preserves deterministic dependency/context closure rather than
  scanning changed files alone.
- Unsupported benchmark/cost claims are removed and sources use direct official
  URLs with a verification date.
- The T0 `Effort` mismatch is corrected from S to M for execution RRI 31.
- The existing authority boundary from T0 is preserved verbatim in substance.

### Evidence to emit

- Diff of the corrected charter sections.
- A gap-to-control matrix in the plan.
- Full RRI evidence in `docs/audit/antares-t0a-rri.md`.

### Status artifacts affected

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- this ledger and the slice plan

### Note

The workflow and HITL files carried **uncommitted** T0 changes. T0a lands as one
coherent correction rather than preserving the capability over-claims as an
intermediate state.

### Completion record (2026-07-29)

- Approval: explicit user approval after the Compact Approval Task Card v2.
- Corrected the workflow and HITL charter so the primary agent or human specialist
  owns the CWE hypothesis, threat model, rationale, tests, remediation, and final
  disposition; Antares only localizes candidate files and returns its trace.
- Preserved optional refinement, candidate-snapshot, and post-CI touchpoints while
  keeping all three non-gating and requiring a justified external CWE.
- Added fail-closed planning controls for model/runtime preflight, argv-only
  sandbox execution, provenance, complete terminal states, context closure,
  ground-truth calibration, triage ownership/SLA, deduplication, and retention.
- Corrected the benchmark interpretation and T0 Effort/RRI mismatch.

Unsupported claims removed or reassigned:

- `threat surfaces`: reassigned to the primary security advisor because Antares
  requires a supplied CWE and only returns candidate files.
- `security rationale`: reassigned because the model card states Antares does not
  explain why a file is vulnerable.
- `recommended follow-up tests`: reassigned because tests are not part of the
  model's file-ranking and exploration-trace output contract.

Evidence and verification:

- Gap-to-control matrix: `docs/plan/antares-security-specialist-advisor.md`
- RRI artifact: `docs/audit/antares-t0a-rri.md`
- `Task-analysis review: n/a - docs/policy-only exemption`
- `Code-solution review: n/a - docs/policy-only exemption`
- Commands run: `make qa-docs`; `git diff --check`; targeted `rg` semantic scan
  for stale capability, File F1, post-CI-only, benchmark, and Effort claims.
- Result: passed. The only stale-claim text found is this ledger's historical
  post-hoc correction notice, retained intentionally for auditability.
- Reflection and unit coverage certification: n/a - docs/policy-only task.
- Next task: T1 remains open and has not been presented or authorized.

## T1 - Runtime and model-access preflight

- **Status:** `[x] Done (owner-waived)` - 2026-07-29
- **Type:** evaluation / operational readiness
- **Effort:** L
- **Presentation RRI:** 49 Med-high
- **RRI artifact:** `docs/audit/antares-t1-rri.md`
- **Depends on:** T0a

### Objective

Prove that an approved, pinned Antares artifact can run on the actual self-hosted
macOS ARM64 environment before investing in the harness or CI integration.

### Acceptance criteria

- Hugging Face access approval is confirmed; tokens are used only for acquisition
  and are absent from runtime packets and logs.
- The model ID, repository revision, quantization, file digest, license, inference
  runtime/version, and local endpoint contract are pinned.
- The acquired artifact digest is verified against the pinned provenance before
  the first inference call; any mismatch records `BLOCKED`.
- Partial download, interrupted acquisition, or any artifact-verification failure
  records `BLOCKED`; no retry may substitute another revision, quantization, or
  provider path silently.
- The representative packet for this preflight is fixed up front: one
  repository-snapshot run over the current `HEAD` rooted in the tracked
  `apps/` + `crates/` tree only, using an explicitly supplied CWE identifier and
  generic description solely to prove runtime behavior. It is a runtime fixture,
  not a production watchlist decision, and its exact input must be recorded in the
  evidence.
- That representative scoped packet completes on the target runner, recording
  cold start, peak memory, swap growth, command latencies, total latency, and
  terminal result.
- Preflight thresholds are objective and fail closed: cold start must be
  `<= 300s`; every measured terminal-command latency in the representative run
  must be `<= 10s`; total representative packet latency must be `<= 900s`; peak
  RSS must stay within `min(24 GiB, 75% of physical RAM)`; on Apple Silicon with
  unified memory, swap growth during the representative run must stay `<= 1 GiB`.
- Compatibility with the selected macOS ARM64 runtime is demonstrated rather than
  inferred from H100 or generic Ollama claims.
- A successful representative run must terminate as `vulnerable_files` or
  `no_vulnerability_found` with local runner exit code `0`; degraded runtime,
  crash, timeout, partial download, digest mismatch, incompatible backend, or
  resource-limit breach records `BLOCKED` and stops T2.

### Evidence to emit

- `docs/evaluations/antares-runtime-preflight.md`
- `docs/evaluations/antares-runtime-preflight.json` with pinned provenance,
  host/runtime facts, measured resources, timings, terminal result, and blocked
  reason when applicable. Minimum top-level fields:
  `status`, `blocked_reason`, `model_id`, `model_revision`, `quantization`,
  `model_digest`, `license`, `runtime`, `runtime_version`, `endpoint`,
  `host_arch`, `host_ram_bytes`, `artifact_digest_verified`, `cold_start_ms`,
  `peak_rss_bytes`, `swap_growth_bytes`, `command_latencies_ms`,
  `total_latency_ms`, `terminal_result`, `representative_packet`.

### Status artifacts affected

- this ledger
- `docs/plan/antares-security-specialist-advisor.md`

### Execution record (2026-07-29)

- Approval: explicit user approval after the Compact Approval Task Card v2.
- Technical result: `BLOCKED`.
- Owner waiver: explicit owner instruction on 2026-07-29 to accept T1 as
  sufficient to proceed despite the failed local-runtime preflight.
- Slice result: T1 is closed as an owner-waived gate. T2 is authorized to start,
  but it must not claim that local `antares-1b` runtime is proven on this host.
- Evidence:
  - `docs/evaluations/antares-runtime-preflight.md`
  - `docs/evaluations/antares-runtime-preflight.json`
- Access blocker:
  - `curl -I -L --max-time 20 https://huggingface.co/fdtn-ai/antares-1b/resolve/main/config.json`
    returned `HTTP/2 401` with `x-error-code: GatedRepo`.
  - `HF_TOKEN`, `HUGGINGFACE_HUB_TOKEN`, and `HUGGING_FACE_HUB_TOKEN` were all
    `unset` in the execution environment.
- Runtime blocker:
  - `huggingface_hub`, `transformers`, `vllm`, and `sglang` were absent from the
    active Python environment.
  - Docker was available through `colima`, but `docker model` was not exposed as
    a usable Model Runner path on this host.
  - `ollama list` succeeded, proving local Ollama availability only; it did not
    satisfy T1 because Antares itself was neither acquired nor pinned there and
    silent runtime substitution is forbidden by the task.
- Host facts recorded in evidence:
  - `macOS 26.5.2`, `arm64`, `Apple M5`, `34359738368` bytes RAM.
- Representative packet:
  - fixed as a runtime-only fixture over `HEAD` scoped to `apps/` + `crates/`
    using `CWE-20 Improper Input Validation`, but not executed because the task
    blocked before lawful acquisition and digest verification.
- Commands run:
  - environment/token presence probe
  - gated-repo HEAD probe to Hugging Face
  - package-availability probe for `huggingface_hub`, `transformers`, `vllm`,
    `sglang`
  - `docker info`
  - `ollama list`
  - host probes for RAM and CPU identity
- Unblock conditions:
  - approved Hugging Face access for the exact Antares artifact
  - one supported macOS ARM64 runtime path installed and pinned
  - rerun of the fixed representative packet meeting the stated cold-start,
    latency, peak-RSS, and swap-growth thresholds

### T1 recovery plan

The technical preflight remains blocked; this section preserves the bounded plan
for clearing the recorded runtime gaps if the slice later requires a proven local
`antares-1b` path on this host. The owner waiver above authorizes T2 without
claiming that this recovery has happened.

| Step | Owner | Action | Stop condition |
|---|---|---|---|
| R1 | Authorized account owner | Accept the gated Hugging Face conditions for `fdtn-ai/antares-1b` and expose a short-lived token to the execution host without committing or logging it | `GatedRepo`, missing token, or unapproved account: remain blocked |
| R2 | Primary agent | Create a disposable macOS ARM64 Python environment and install exact versions of the official Transformers loading path and measurement dependencies | Installation failure or unsupported backend: remain blocked; do not substitute Ollama or third-party weights |
| R3 | Primary agent | Resolve and pin the model commit, acquire the official artifact, generate and verify a SHA-256 manifest, and record runtime/device provenance | Revision drift, partial download, or digest mismatch: remain blocked |
| R4 | Primary agent | Run the fixed `HEAD` / `apps/` + `crates/` / `CWE-20` fixture in a read-only snapshot with bounded terminal commands | Any sandbox, timeout, malformed-output, or terminal-state violation: remain blocked |
| R5 | Primary agent | Record cold start, command latencies, total latency, peak RSS, swap growth, and terminal result in both T1 evidence files | Any threshold failure, non-zero exit, or missing evidence: T1 remains blocked; otherwise authorize T2 review |

The `350m` official variant was probed on 2026-07-29 and is also gated, so it is
not an alternate recovery route. The primary runtime route is Transformers with
PyTorch on the existing Apple Silicon host, selected because the official model
card documents Transformers loading and the current preflight host is macOS
ARM64. Any move to `vllm`, `sglang`, Ollama, Docker Model Runner, or a community
quantization requires an explicit task revision with new provenance and RRI.

The external access gate in R1 is not agent-solvable: the agent cannot accept
terms or manufacture an authorized Hugging Face credential. Once R1 is cleared,
R2–R5 are the agent's execution responsibility and T2 remains prohibited until
the regenerated T1 evidence has `status: PASS`.

## T2 - Sandboxed agentic harness and artifact schema

- **Status:** `[~] Decomposed` - 2026-07-29
- **Type:** development / security-sensitive tooling
- **Original pre-execution RRI:** 86 Very high
- **RRI artifact:** `docs/audit/antares-t2-rri.md`
- **Depends on:** T1

### Objective

Implement the terminal loop in an ephemeral, default-deny sandbox and normalize
every success, negative, degraded, and refused outcome into a versioned artifact.

### Happy paths considered

- **HP-1:** a valid CWE packet runs to `submit_vulnerable_files` within 15 calls
  and returns only canonical, in-snapshot candidate paths plus the full trace.
- **HP-2:** `submit_no_vulnerability_found` becomes an explicit negative result,
  not an error and not an ambiguous empty list.

### Edge cases considered

- **EC-1:** command budget exhaustion preserves the partial trace and records
  `budget_exhausted`.
- **EC-2:** shell metacharacters, environment assignment, an unapproved option,
  `find -exec`, or any non-allowlisted executable are refused before execution.
- **EC-3:** absolute paths, `..`, or a symlink escaping the snapshot are refused
  after canonical-path containment checks.
- **EC-4:** malformed tool-call JSON, runtime unavailability, timeout, output-limit
  exhaustion, or model failure each produce a distinct durable terminal state.

### Acceptance criteria

- Parse structured tool calls into argv and invoke without a shell; reject command
  substitution, pipelines, redirects, control operators, and environment prefixes.
- Apply executable- and option-level allowlists; path operands must resolve inside
  the read-only snapshot after symlink resolution.
- Use an ephemeral container/process sandbox with network disabled, credentials
  removed, read-only mounts/root, dropped privileges, and teardown after each run.
- Enforce 15 model calls, a 10-second per-command timeout, total wall timeout,
  2-CPU/4GB limits, PID cap, and bounded stdout/stderr/model output.
- Version the schema and record model/runtime/harness/watchlist provenance, packet
  and snapshot hashes, timestamps, exact scope/exclusions, validated candidates,
  result semantics, terminal state, and auditable human disposition fields.
- Store raw traces outside committed paths with redaction and retention controls;
  committed summaries must not contain source excerpts, prompts, or secrets.
- Replay fixtures cover every HP/EC case deterministically.

### Evidence to emit

- Harness modules under `scripts/antares/` (each under 500 lines).
- JSON schema and one redacted example for every terminal state.
- Replay fixtures and sandbox-escape regression tests.

### Status artifacts affected

- this ledger and the slice plan

### Security note

Any inability to enforce argv-only execution, containment, resource bounds, or
credential isolation is blocking. A command name allowlist alone is insufficient.

### Decomposition record (2026-07-29)

The full T2 scope was re-scored before presentation and measured **RRI 86 -> Very
high**, which triggers the mandatory decomposition gate under
`docs/policies/RRI_POLICY.md`. The user approved decomposition on 2026-07-29.

Replacement subtasks:

- `T2a` isolates tool-call parsing and the terminal-state contract so malformed
  model output can fail closed before any execution path exists.
- `T2b` isolates executable/option allowlisting and canonical containment so the
  policy surface is testable without sandbox runtime noise.
- `T2c` isolates ephemeral execution, limits, isolation, and teardown.
- `T2d` isolates the versioned artifact schema, trace-redaction contract, and
  durable terminal-state normalization.
- `T2e` integrates the preceding units under deterministic replay fixtures and
  regression tests.

`T3` now depends on `T2e`, not on the undecomposed T2 umbrella task.

## T2a - Tool-call parser and terminal-state contract

- **Status:** `[x] Done` - 2026-07-29
- **Type:** development / security-sensitive tooling
- **Execution RRI:** 45 Med-high
- **Effort:** L
- **RRI artifact:** `docs/audit/antares-t2a-rri.md`
- **Depends on:** T1
- **Decomposed from:** T2

### Objective

Parse Antares terminal-tool messages into a strict internal command/request model
and a fail-closed terminal-state envelope before any command policy or sandbox
execution exists.

### Happy paths considered

- **HP-1:** a valid `terminal` tool call becomes a structured argv request with
  stable argument ordering preserved from the model payload.
- **HP-2:** `submit_no_vulnerability_found` is normalized as an explicit negative
  terminal result rather than an empty candidate list or implicit success.

### Edge cases considered

- **EC-1:** malformed tool-call JSON records `malformed_tool_call` with no partial
  execution attempt.
- **EC-2:** an unsupported tool name or malformed submit payload is rejected into
  a distinct durable terminal state before policy evaluation.
- **EC-3:** duplicate or ambiguous terminal submissions fail closed instead of
  silently preferring one payload.
- **EC-4:** a type-mismatched payload field (e.g. an integer where an argv string
  is expected) is rejected as `malformed_tool_call` before it reaches the policy
  layer, not coerced.

### Acceptance criteria

- Only the documented `terminal`, `submit_vulnerable_files`, and
  `submit_no_vulnerability_found` actions are accepted.
- The parser returns structured argv data, never a shell command string.
- Parser errors are durable and machine-distinguishable from sandbox/runtime
  errors.
- Candidate-path payloads are preserved for later containment validation but are
  not treated as trusted paths at parse time.

### Evidence to emit

- Parser module and unit fixtures for valid and invalid tool-call payloads.
- One redacted example for each parser-produced terminal state.

### Status artifacts affected

- this ledger and the slice plan

### ADR-038 routing record (2026-07-29)

- Qwen27 (`qwen3.6:27b-q4_K_M`) advisory refinement: `route_recommendation: GO_LOCAL`.
  Artifact: `docs/audit/gemma-evidence/antares-t2a-phase1-refinement.json`.
- Primary route receipt: `GO_LOCAL`, concurring with Qwen27; no ADR-038 s.6 hard
  exclusion applies (not auth/security enforcement itself, no rights/consent
  invariant, no schema/migration/release cut, no unresolved ADR, scope bounded
  to two files under 500 lines).
- `med_high_gate.py` result: `GO_LOCAL` (`"Qwen27 and primary both recommend
  GO_LOCAL."`).
- Bounded `qwen3.6:35b-a3b` session (`run_med_high_task.py`, 8-turn/300s/0-repair
  budget): **did not reach success**. The model spent all 8 turns on
  `run_command` reconnaissance (including two calls against a hallucinated,
  non-existent path `/home/user/repos/antares/scripts/antares/`), never called
  `write_file` or `finish`, and `run_local_task.py` exited non-zero
  (`total_turns_exhausted` / `runner_nonzero_exit`). Per ADR-038, Med-high has
  **zero** repair attempts — this correctly triggered immediate escalation
  rather than a local retry.
- Supervisor emitted the full ADR-038 §5 evidence bundle automatically:
  `/private/tmp/.../scratchpad/t2a-escalation-bundle.json` (session-local
  scratch path; bundle contents are reproduced in this record's routing
  summary above and in the local-session transcript
  `/private/tmp/.../scratchpad/t2a-med-high-result.json`).
- Escalation route taken: primary agent (Claude Code, cloud) implemented T2a
  directly, per ADR-038 §4/§6, using the same approved task card, acceptance
  criteria, and HP/EC set the local session was given.

### Fix applied to the ADR-038 tooling during this task

While preparing the Qwen27 refinement call, `scripts/local-architect/run_analysis.py`
truncated its JSON response mid-string at both `--num-predict 2048` and `6144`
(`Unterminated string` / `Model response is not valid JSON`). Root cause: the
script never set Ollama's `num_ctx`, so the session ran under Ollama's own
default context window (shared between prompt and response), which the
`med-high-refinement-v1` schema's ten fields could exceed regardless of
`num_predict`. Separately, the profile's prompt placed no bound on array
length or item verbosity, so the model had no signal to stay compact.

Fix (RRI 22, Low band, agent-direct execution per
`docs/policies/HITL_AUTONOMY_POLICY.md` local-delegation rules — this is
infrastructure logic in a governance-gating script, not a mechanical patch
eligible for Gemma delegation):

- Added an explicit `--num-ctx` CLI flag (default `8192`) threaded through
  `Config`, the `/api/generate` request `options`, and the artifact's
  `runtime` provenance block.
- Added an explicit compactness instruction to the `med-high-refinement-v1`
  prompt (max 6 items per array field, max 2 sentences per item, max 4
  claims).
- Added `test_hp1d_sends_num_ctx_and_records_it_in_runtime_provenance` to
  `run_analysis_test.py`; updated the two existing `Config(...)` fixtures for
  the new required field. All 18 tests in the suite pass.
- Verified against the real Ollama endpoint: the retried refinement call
  (`--num-ctx 8192`) completed successfully
  (`docs/audit/gemma-evidence/antares-t2a-phase1-refinement.json`,
  `"success": true`).
- Gemma Reviewer evidence: `docs/audit/gemma-evidence/antares-t2a-run-analysis-fix.json`
  (formal receipt: `verdict: PASS`). Note: the reviewed `git diff HEAD` packet
  also contained unrelated pre-existing uncommitted changes from a different,
  already-in-progress task (`agent-session-preflight-gate`); all 3 findings
  from the 3-pass run are against `scripts/agent-preflight.py` /
  `scripts/agent_preflight_test.py` from that unrelated task, not against
  `scripts/local-architect/run_analysis.py` or its test file — confirmed by
  inspecting `changed_paths` and the per-finding `path` field in the raw
  aggregate (`/tmp/dubbridge-gemma-review.json`). Zero findings target this
  fix's files.

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: manual Ollama `/api/generate` invocation with `think: false`,
  `num_ctx: 16384`, full contents of the 5 new `scripts/antares/` files plus
  the approved acceptance criteria and hard constraints.
- Artifact: `docs/audit/gemma-evidence/antares-t2a.json`
- Verdict: `PASS`
- Findings: none. Reviewer summary: "The implementation strictly adheres to
  all acceptance criteria (HP-1, HP-2, EC-1 through EC-4) and hard
  constraints... Hard constraints are met: no shell execution, filesystem
  access, network calls, or subprocess invocations exist. Files are under
  500 lines. All 29 unit tests pass."
- Gemma fallback: not triggered -- `qwen3.6:27b-q4_K_M` responded successfully
  on the first call.
- D14 fallback: not triggered.
- disposition_divergence: `none`
- Primary-agent disposition: accepted (no findings to disposition).

### Reflection log

Required passes: 3 (`45` -> `Med-high`)

#### Pass 1

- **Draft verdict:** implementation complete, 29 unit tests pass, covers the
  three documented Antares tool-call actions with distinct terminal states
  for every HP/EC case.
- **Critique findings:**
  - The internal payload schema (`{"tool": ..., "payload": {"candidates": [...]}}`)
    is an assumption, not a literal transcription of an observed Antares
    wire format -- T1 never reached live inference, so no real transcript
    exists to validate against.
  - The bounded local-session transcript (from the failed `qwen3.6:35b-a3b`
    attempt) showed a *different* tool-call shape
    (`{"function": {"name": ..., "arguments": {...}}}`, the generic
    Ollama/OpenAI-style function-calling frame) -- a real discrepancy between
    my assumed schema and at least one plausible wire format.
- **Revisions applied:** none yet; carried the finding into Pass 2 for a
  considered resolution rather than a reflexive rewrite.

#### Pass 2

- **Draft verdict:** confirmed the Pass 1 discrepancy is real but
  immaterial: the local-session transcript's format is the *generic local
  runner's* tool-calling envelope (used for any locally-run agentic model),
  not evidence of Antares's own `<tool_call>` text-embedded protocol, which
  the model card describes as emitted inside the model's own generated text
  (a ReAct-style loop), not via native API function-calling.
- **Critique findings:**
  - The T2a task definition explicitly requires building "from the
    documented model-card contract, not from an observed transcript" -- so
    adopting the local-runner's incidental format would have been *less*
    justified than the current action-name-based schema, not more.
  - The internal schema was undocumented as an abstraction, which could
    mislead whoever implements T2c into assuming this parser consumes
    Antares's raw text output directly.
- **Revisions applied:** rewrote the `tool_call_parser.py` module docstring
  to explicitly name the internal schema as a normalized abstraction, state
  that no live Antares transcript exists, and assign responsibility for
  translating the model's real `<tool_call>` output into this schema to the
  future T2c harness.

#### Pass 3

- **Draft verdict:** docstring correction applied; implementation stable.
- **Critique findings:** no issues found. Verified: EC-4 is covered for both
  `terminal` argv and `submit_vulnerable_files` candidates (not just one);
  no stale references to the pre-rename `parser.py`/`parser_test.py`
  filenames remain anywhere in the module, test, or this ledger's evidence
  text; the acceptance command passes identically whether invoked from the
  repository root or from inside `scripts/antares/`.
- **Revisions applied:** none -- final grep and full test re-run both came
  back clean.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid `terminal` call becomes structured argv with stable order | `scripts/antares/tool_call_parser_test.py::ParseTerminalCallTest::test_hp1_valid_terminal_call_preserves_argv_order` | passed |
| HP-2 | Happy path | `submit_no_vulnerability_found` is an explicit negative result | `scripts/antares/tool_call_parser_test.py::ParseSubmitNoVulnerabilityFoundTest::test_hp2_no_vulnerability_found_is_explicit_negative_result` | passed |
| EC-1 | Edge case | malformed tool-call JSON -> `MALFORMED_TOOL_CALL`, no partial execution | `scripts/antares/tool_call_parser_test.py::ParseMalformedJsonAndUnsupportedToolTest::test_ec1_malformed_json_records_malformed_tool_call` | passed |
| EC-2 | Edge case | unsupported tool name / malformed submit payload -> distinct terminal state | `scripts/antares/tool_call_parser_test.py::ParseMalformedJsonAndUnsupportedToolTest::test_ec2_unsupported_tool_name_is_rejected_distinctly` | passed |
| EC-3 | Edge case | duplicate terminal submissions fail closed | `scripts/antares/tool_call_parser_test.py::CheckDuplicateSubmissionTest::test_ec3_two_terminal_submissions_fail_closed_as_duplicate` | passed |
| EC-4 | Edge case | type-mismatched payload field rejected, never coerced | `scripts/antares/tool_call_parser_test.py::ParseTerminalCallTest::test_ec4_terminal_call_with_integer_argv_element_is_malformed_not_coerced` | passed |

Full suite: `python3 -m pytest scripts/antares/tool_call_parser_test.py scripts/antares/terminal_state_test.py -q` -> 29 passed.

### Owner final verification

- Owner: `matias`
- Date: `2026-07-29`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, and that the
  ADR-038 Med-high routing was followed in full (Qwen27 refinement, primary
  receipt, gate evaluation, one bounded local attempt that failed cleanly
  into escalation, cloud implementation with the same approved acceptance
  criteria) rather than skipped after the local attempt's failure.
- Commands run: `python3 -m pytest scripts/antares/tool_call_parser_test.py scripts/antares/terminal_state_test.py -q`;
  `python3 -m pytest scripts/local-architect/run_analysis_test.py -q`;
  `GEMMA_REVIEW_TASK_ID=antares-t2a-run-analysis-fix make qa-gemma-review`.

- Task-analysis review: n/a - decomposition of T2 already carried an
  approved Compact Approval Task Card v2 for T2a; no separate phase-1 review
  was re-run for this already-approved subtask.
- Code-solution review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2a.json - PASS

## T2b - Command allowlist and canonical path containment

- **Status:** `[x] Done` - 2026-07-30
- **Type:** development / security-sensitive tooling
- **Execution RRI:** 50 Med-high
- **Effort:** L
- **RRI artifact:** `docs/audit/antares-t2b-rri.md`
- **Depends on:** T2a
- **Decomposed from:** T2

### Objective

Validate executables, options, and path operands against the read-only navigation
policy before any sandboxed command is launched.

### Happy paths considered

- **HP-1:** an allowlisted read-only command with approved options and in-snapshot
  operands is converted into a validated command plan.
- **HP-2:** a model-submitted candidate path that resolves inside the snapshot is
  marked containment-valid for downstream execution or artifact recording.

### Edge cases considered

- **EC-1:** shell metacharacters, environment assignment, redirects, pipelines,
  substitutions, or control operators are refused before execution.
- **EC-2:** a disallowed option such as `find -exec` is refused even when the
  executable itself is allowlisted.
- **EC-3:** absolute paths, `..`, or symlinks escaping the snapshot are refused
  after canonical-path resolution.

### Acceptance criteria

- Executable allowlist and option-level rules are explicit and fixture-testable.
- Path validation canonicalizes after symlink resolution and rejects escape.
- Validation failures are durable and machine-distinguishable from parse and
  sandbox failures.
- The policy layer performs no shell evaluation and no filesystem mutation.

### Evidence to emit

- Command-policy module, path-containment tests, and escape-regression fixtures.

### Status artifacts affected

- this ledger and the slice plan

### Session routing override (2026-07-30)

By explicit user instruction on 2026-07-30, this session resolves all
Med-high (RRI 41-55) implementations directly to cloud (Claude Code),
bypassing the ADR-038 gate (Qwen27 refinement -> `med_high_gate.py` ->
bounded local `qwen3.6:35b-a3b` attempt) until further notice in this
session. This affects only the code-authoring surface: band-routed
independent review (phases 1/2), 3 Reflection passes, and the human
approval gate all ran unchanged per ADR-038/this guide.

### Completion record (2026-07-30)

- Approval: explicit user approval after the Compact Approval Task Card v2
  presented for T2b (RRI 50, Med-high).
- Implemented `scripts/antares/command_policy.py` (executable/option
  allowlist: `grep`, `find`, `cat`, `ls`, `head`, `tail`, `wc`, with explicit
  per-executable option sets; `find -exec` and any other non-listed option
  refused even on an allowlisted executable) and
  `scripts/antares/path_containment.py` (canonical resolution after symlink
  follow, absolute-path and `..` rejection, whole-batch fail-closed on any
  escaping path).
- Extended `scripts/antares/terminal_state.py` with six new terminal-state
  kinds for T2b outcomes (`COMMAND_PLAN_VALID`, `PATH_CONTAINMENT_VALID`,
  `COMMAND_REJECTED_SHELL_SYNTAX`, `COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED`,
  `COMMAND_REJECTED_OPTION_NOT_ALLOWED`, `PATH_REJECTED_CONTAINMENT_ESCAPE`),
  kept distinct from T2a's parser states per the module's existing
  fail-closed contract.
- Both new modules perform zero shell evaluation (argv-only,
  metacharacter/environment-assignment-prefix rejection as defense in depth)
  and zero filesystem mutation (read-only `Path.resolve()` / `relative_to()`
  only).
- File sizes: `command_policy.py` 165 lines, `path_containment.py` 92 lines
  -- both well under the 500-line target-file-size gate.

### Reflection log

Required passes: 3 (`50` -> `Med-high`)

#### Pass 1

- **Draft verdict:** implementation complete, 52/53 tests passing at this
  point (one intentionally-revealing failure below), covers HP-1/HP-2 and
  EC-1/EC-2/EC-3.
- **Critique findings:**
  - `find src -name` (an option-with-value token truncated at the end of
    argv, i.e. the pattern value is missing) silently passed as
    `COMMAND_PLAN_VALID` instead of being rejected -- the `i += 2` skip
    logic advanced past the end of `args` with no bounds check.
  - Checked the environment-assignment heuristic (`head.isidentifier()`) for
    false positives against glob-style operands (e.g. `*.rs`); confirmed low
    residual risk since a real filename would need to look like
    `identifier=value` to trigger it, which is not a realistic repository
    path shape.
- **Revisions applied:**
  - Added an explicit bounds check in `command_policy.py::validate_command`:
    an option-with-value token with no following element now returns
    `COMMAND_REJECTED_OPTION_NOT_ALLOWED` instead of falling through the
    loop as valid.
  - Added `test_ec2_option_with_value_truncated_at_end_of_argv_is_rejected`
    to `command_policy_test.py`.

#### Pass 2

- **Draft verdict:** fix from Pass 1 verified; re-examined boundary
  interactions between the global shell-metacharacter scan and the
  per-executable/per-option checks.
- **Critique findings:**
  - Verified that an executable token itself carrying an embedded
    metacharacter (e.g. `"grep;rm"` as one argv element) is caught by the
    global `_is_shell_unsafe` scan before the allowlist lookup runs --
    confirmed by direct probe, no gap found.
  - Verified `TERMINAL_SUBMISSION_KINDS` in `terminal_state.py` correctly
    excludes the two new T2b success kinds (`COMMAND_PLAN_VALID`,
    `PATH_CONTAINMENT_VALID`) -- they are policy-validation outcomes, not
    Antares session terminal submissions, so T2a's duplicate-submission
    check must not and does not treat them as such.
  - Re-ran the full T2a suite (29 tests) in isolation to confirm the
    `terminal_state.py` additions caused zero regression.
- **Revisions applied:** none -- no issues found requiring a code change.

#### Pass 3

- **Draft verdict:** stable; final consistency sweep.
- **Critique findings:** no issues found. Verified no stale cross-references
  to renamed/removed symbols exist across `scripts/antares/*.py`; all three
  touched/created modules parse cleanly via `ast.parse`; the full 53-test
  suite passes identically whether invoked from the repository root.
- **Revisions applied:** none.

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: manual Ollama `/api/generate` invocation (`think: false`,
  `num_ctx: 16384`, `num_predict: 1200`) with the `terminal_state.py` diff,
  full contents of the two new modules, and the approved acceptance
  criteria/hard constraints. First attempt timed out at 180s; per the
  mandatory one-retry discipline, retried immediately with the same packet
  at a 280s timeout, which completed successfully -- no fallback to Gemma or
  D14 was needed.
- Artifact: `docs/audit/gemma-evidence/antares-t2b.json`
- Verdict: `PASS`
- Findings: none.
- Gemma fallback: not triggered -- the retry against `qwen3.6:27b-q4_K_M`
  succeeded.
- D14 fallback: not triggered.
- disposition_divergence: `none`
- Primary-agent disposition: accepted (no findings to disposition).
- Review artifact: docs/audit/gemma-evidence/antares-t2b.json

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | allowlisted command + approved options + in-snapshot operands -> validated command plan | `scripts/antares/command_policy_test.py::ValidateCommandHappyPathTest::test_hp1_allowlisted_command_with_approved_options_is_valid_plan` | passed |
| HP-2 | Happy path | candidate path resolving inside snapshot -> containment-valid | `scripts/antares/path_containment_test.py::CheckPathContainmentTest::test_hp2_all_valid_paths_produce_containment_valid` | passed |
| EC-1 | Edge case | shell metacharacters / env assignment / redirects refused before execution | `scripts/antares/command_policy_test.py::ValidateCommandShellSyntaxTest::test_ec1_pipe_character_is_rejected` (plus `test_ec1_semicolon_is_rejected`, `test_ec1_command_substitution_is_rejected`, `test_ec1_environment_assignment_prefix_is_rejected`, `test_ec1_output_redirect_is_rejected`) | passed |
| EC-2 | Edge case | disallowed option (e.g. `find -exec`) refused even on an allowlisted executable | `scripts/antares/command_policy_test.py::ValidateCommandExecutableAndOptionTest::test_ec2_find_exec_is_rejected_even_though_find_is_allowed` (plus `test_ec2_disallowed_option_on_allowlisted_executable_is_rejected`, `test_ec2_option_with_value_truncated_at_end_of_argv_is_rejected`) | passed |
| EC-3 | Edge case | absolute path / `..` / symlink escape refused after canonical resolution | `scripts/antares/path_containment_test.py::ResolveWithinSnapshotTest::test_ec3_symlink_escaping_snapshot_is_rejected` (plus `test_ec3_absolute_path_is_rejected`, `test_ec3_dotdot_traversal_is_rejected`, and the `command_policy_test.py::ValidateCommandPathContainmentTest` equivalents) | passed |

Full suite: `python3 -m pytest scripts/antares/ -q` -> 53 passed.

### Owner final verification

- Owner: `matias`
- Date: `2026-07-30`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, that the
  session-scoped Med-high-to-cloud routing override was applied and
  recorded as directed, and that band-routed independent review (phase 2),
  3 Reflection passes, and unit coverage certification all completed before
  this task was marked Done.
- Commands run: `python3 -m pytest scripts/antares/ -q`; manual
  `qwen3.6:27b-q4_K_M` review invocation (see Peer Reviewer evidence above).

- Task-analysis review: n/a - decomposition of T2 already carried an
  approved Compact Approval Task Card v2 for T2b as a named decomposed
  subtask; no separate phase-1 review was re-run for this already-approved
  subtask (same precedent as T2a).
- Code-solution review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2b.json - PASS

## T2c - Ephemeral sandbox runner and resource enforcement

- **Status:** `[~] Decomposed` - 2026-07-30
- **Type:** development / security-sensitive tooling
- **Pre-decomposition RRI:** 56 Complex (mandatory decomposition gate, RRI >= 56)
- **RRI artifact:** `docs/audit/antares-t2c-rri.md`
- **Depends on:** T2b
- **Decomposed from:** T2
- **Decomposed into:** T2c-1, T2c-2

### Objective

Run already-validated argv commands in an ephemeral, default-deny sandbox with
network isolation, read-only mounts, dropped privileges, bounded resources, and
reliable teardown.

### Decomposition rationale

Presentation-time RRI scored 56 (Complex band), one point past the Med-high
ceiling, driven primarily by `D=4` (this task spans a genuine OS-level process
isolation boundary, not pure validation) and `P=4` (a defect here is a sandbox
escape or resource-exhaustion class failure, per
`docs/policies/RRI_POLICY.md` § Decomposition triggers, "Final RRI >= 56" is an
unconditional hard gate). Split into T2c-1 (process execution and isolation) and
T2c-2 (resource budget, wall-timeout, teardown), each independently scoring in
the Med-high band (49 and 51) with `A=1`, per the split target. Full rationale
and both subtask RRI computations: `docs/audit/antares-t2c-rri.md`.

### Status artifacts affected

- this ledger and the slice plan

## T2c-1 - Sandbox process execution and isolation

- **Status:** `[x] Done` - 2026-07-30
- **Type:** development / security-sensitive tooling
- **Effort:** L (RRI 49 Med-high, planning estimate — recompute before implementation)
- **RRI artifact:** `docs/audit/antares-t2c-rri.md` § T2c-1
- **Depends on:** T2b
- **Decomposed from:** T2c

### Objective

Spawn an already-validated `COMMAND_PLAN_VALID` argv inside an isolated
subprocess — network disabled, read-only mounts, dropped privileges, credentials
stripped — and return captured output with measured timing.

### Scope boundary with T2c-2

T2c-1 owns the single-process lifecycle: launching, isolating, timing out, and
killing one command's subprocess. T2c-2 owns aggregate session accounting: the
15-command wall budget, hard CPU/RAM/PID/output-size caps, and teardown
verification across the whole run. Concretely:

- T2c-1 captures stdout/stderr as produced until the process terminates (by
  completion or by its own per-command timeout); it does not impose an output
  *size* cap — that is a T2c-2 resource cap, layered on top in the next
  subtask. "Bounded" in T2c-1's context means bounded by process termination,
  not by a byte-size ceiling.
- T2c-1 is responsible for killing the specific subprocess instance when its
  own per-command timeout fires. T2c-2 is responsible for the aggregate
  command counter, the wall-clock budget across all commands, and confirming
  teardown actually completed after every exit path (including ones T2c-1
  triggered).

### Happy paths considered

- **HP-1:** a validated read-only command completes inside the sandbox and returns
  captured stdout/stderr plus measured timing.

### Edge cases considered

- **EC-2:** runtime unavailability or sandbox bootstrap failure produces
  `runtime_unavailable` without falling back to unsandboxed execution.

### Acceptance criteria

- The runner uses network-disabled, credential-stripped, read-only execution with
  dropped privileges for every invocation.
- Per-command timeout is enforced; a command that exceeds it is killed by
  T2c-1 itself, not left running for a later layer to notice.
- No success path exists outside the sandbox boundary — a subprocess launched
  without isolation active is never treated as a valid result.
- Measured timing and captured stdout/stderr are emitted per invocation.

### Evidence to emit

- Sandbox runner module (`scripts/antares/sandbox_runner.py`), isolation
  fixtures, and bootstrap-failure/timeout tests.

### Status artifacts affected

- this ledger and the slice plan

### Task-analysis review (phase 1)

- Review artifact: `docs/audit/gemma-evidence/antares-t2c-1-phase1.json`
- Verdict: `PASS` — 2 MINOR findings (T2c-1/T2c-2 boundary wording for
  per-command-kill responsibility, and "bounded" output meaning), both
  folded into the Objective/Scope-boundary/Acceptance-criteria text above.
  No BLOCKING or MAJOR findings.

`Task-analysis review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2c-1-phase1.json - PASS`

### Peer Reviewer evidence

- Reviewer: `gemma` (fallback)
- Command: manual Ollama `/api/generate` request against `gemma4:26b-a4b-it-qat`
  with the full diff, task acceptance criteria, and independently-verified test
  results (64/64 passing, empirical sandbox-exec network-block verification)
- Artifact: `docs/audit/gemma-evidence/antares-t2c-1-phase2.json`
- Verdict: `PASS`
- Findings: none — HP-1/EC-2 confirmed satisfied, fail-closed network-isolation
  boundary confirmed enforced (no unisolated success path), no resource-leak or
  security concern raised, scope boundary with T2c-2 confirmed respected
- Gemma fallback: `triggered` — reason: `qwen3.6:27b-q4_K_M` first attempt timed
  out at the socket level (280s) against the ~600-line review packet; the
  mandatory immediate retry against the same packet completed but returned an
  empty `response` (`done_reason: length`, all 2048 tokens consumed by
  thinking with no content emitted) — an unusable result under the "invalid
  output" fallback trigger. First Gemma attempt itself truncated mid-finding
  (`done_reason: length`) but returned a legible `PASS` verdict plus complete
  HP-1/EC-2 findings before the cutoff; a second Gemma call with a larger
  token budget and an explicit conciseness instruction returned a complete,
  non-truncated response (`done_reason: stop`) answering all four review
  questions, used as the final result.
- D14 fallback: `not triggered` — reason: Gemma produced a usable consolidated
  result on the second call
- disposition_divergence: `none`
- Primary-agent disposition: accepted — no findings to disposition; verdict
  confirmed consistent with the Reflection log below (both Reflection-pass
  fixes were already in place at review time)

`Code-solution review: gemma docs/audit/gemma-evidence/antares-t2c-1-phase2.json - PASS`

### Reflection log

Required passes: 3 (`49` → `Med-high`)

#### Pass 1

- **Draft verdict:** initial `run_sandboxed` used a stripped environment and a
  no-op-unless-root privilege drop, but never actually disabled network access
  at the OS level — the credential-stripped environment does not stop an
  allowlisted binary from making a network call.
- **Critique findings:**
  - The "network-disabled ... for every invocation" acceptance criterion was
    not met: no mechanism in the draft actually blocked network syscalls, so
    every `SANDBOX_EXECUTION_COMPLETE` success path was returned without a
    verified isolation guarantee.
- **Revisions applied:**
  - Introduced an injectable `NetworkIsolation` strategy; added
    `MacosSandboxExecIsolation` (real `sandbox-exec` deny-network profile) and
    `UnavailableNetworkIsolation` (fails every run closed to
    `SANDBOX_RUNTIME_UNAVAILABLE` rather than running unisolated).
  - Empirically verified with a live `curl` call under the profile (exit 6,
    `http_code 000`) that the network block is real, not just unit-tested
    against a mock.

#### Pass 2

- **Draft verdict:** network isolation was now real, but two resource-hygiene
  gaps remained: the macOS sandbox profile temp file was never cleaned up, and
  a timeout only killed the `sandbox-exec` wrapper PID — a grandchild process
  it forked could survive as an orphan.
- **Critique findings:**
  - `MacosSandboxExecIsolation.wrap` wrote a `/tmp/antares-sandbox-*.sb` file
    per invocation with no corresponding cleanup — a per-session resource leak
    that will compound once T2c-2's 15-command budget starts calling this
    repeatedly.
  - `process.kill()` in the timeout path only signals the immediate PID.
    Under the real macOS wrapper, that PID is `sandbox-exec`, not the actual
    command — a forked grandchild could keep running after the reported
    timeout, which is exactly the "teardown leaves a process running" failure
    class the RRI evidence names as the P=4 driver for this task.
- **Revisions applied:**
  - Added `cleanup()` to the `NetworkIsolation` protocol and
    `MacosSandboxExecIsolation`; `run_sandboxed` now calls it on every exit
    path (success, timeout, bootstrap failure).
  - Switched to `start_new_session=True` plus `os.killpg` on timeout so the
    kill reaches the whole process group, with a `process.kill()` fallback if
    the group kill itself fails.
  - Added a live test (`test_timeout_kills_grandchild_process_spawned_under_the_wrapper`)
    that spawns `sleep 5` under the real `sandbox-exec` wrapper and confirms
    via `pgrep` that nothing survives the timeout kill.

#### Pass 3

- **Draft verdict:** functionally complete and re-verified (64/64 tests
  passing); reviewed acceptance criteria, HP-1/EC-2 coverage, and the module
  docstring for accuracy.
- **Critique findings:**
  - The module docstring stated "dropped privileges" as an unqualified
    guarantee, but the implementation is a documented no-op when the host
    process is not running as root (the common case in dev/CI) — the
    docstring overstated what the code actually enforces in that case.
- **Revisions applied:**
  - Reworded the module docstring to state the privilege-drop behavior
    precisely (enforced only when the host runs as root; a documented no-op
    otherwise), matching what `_drop_privileges`'s own docstring already said.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | validated command completes in sandbox, captured stdout/stderr + timing | `scripts/antares/sandbox_runner_test.py::RunSandboxedHappyPathTest::test_hp1_validated_command_completes_with_captured_output_and_timing` | passed |
| HP-1 | Happy path | non-zero exit / stderr also captured correctly | `scripts/antares/sandbox_runner_test.py::RunSandboxedHappyPathTest::test_hp1_captures_stderr_and_nonzero_exit_code` | passed |
| EC-2 | Edge case | missing snapshot root fails closed to runtime_unavailable | `scripts/antares/sandbox_runner_test.py::RunSandboxedRuntimeUnavailableTest::test_ec2_missing_snapshot_root_is_runtime_unavailable` | passed |
| EC-2 | Edge case | no network isolation available never falls through to an unisolated success | `scripts/antares/sandbox_runner_test.py::RunSandboxedRuntimeUnavailableTest::test_ec2_unavailable_network_isolation_is_runtime_unavailable_not_unisolated_run` | passed |
| EC-2 | Edge case | subprocess bootstrap failure (missing executable) fails closed | `scripts/antares/sandbox_runner_test.py::RunSandboxedRuntimeUnavailableTest::test_ec2_bootstrap_failure_from_nonexistent_executable_is_runtime_unavailable` | passed |

Supporting evidence beyond the approved HP/EC cases (per-command timeout kill,
real network-block verification, resource cleanup — all Reflection-pass fixes):

| Behavior | Unit test evidence | Result |
|---|---|---|
| per-command timeout kills the subprocess | `scripts/antares/sandbox_runner_test.py::RunSandboxedTimeoutTest::test_timeout_kills_subprocess_and_returns_timed_out_state` | passed |
| macOS sandbox-exec profile actually blocks network (live syscall test) | `scripts/antares/sandbox_runner_test.py::MacosSandboxExecIsolationTest::test_macos_isolation_actually_blocks_network_when_available` | passed |
| timeout kill reaches a grandchild forked under the wrapper, not just the wrapper PID | `scripts/antares/sandbox_runner_test.py::MacosSandboxExecIsolationTest::test_timeout_kills_grandchild_process_spawned_under_the_wrapper` | passed |
| sandbox profile temp file is cleaned up after `wrap()`/after a full run | `scripts/antares/sandbox_runner_test.py::MacosSandboxExecIsolationTest::test_cleanup_removes_the_profile_file_written_by_wrap`, `::test_run_sandboxed_cleans_up_profile_file_after_completion` | passed |

Full suite: `python3 -m pytest scripts/antares/ -q` -> 64 passed.

### Owner final verification

- Owner: `Matias Kruk`
- Date: `2026-07-30`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior, including the empirically-verified network-isolation and process-group-kill guarantees added during Reflection.
- Commands run: `python3 -m pytest scripts/antares/ -q` (64 passed),
  `python3 -m pytest scripts/antares/sandbox_runner_test.py -v`,
  `python3 -m py_compile scripts/antares/sandbox_runner.py
  scripts/antares/sandbox_runner_test.py scripts/antares/terminal_state.py`,
  `make qa-fmt`

## T2c-2 - Resource budget, wall-timeout, and teardown enforcement

- **Status:** `[x] Done (owner-waived, 2026-07-30)` — implementation,
  Reflection log, and phase-2 review complete. Owner final verification was
  explicitly waived rather than performed as a personal test-by-test check —
  see that section below for the verbatim waiver record.
- **Type:** development / security-sensitive tooling
- **Effort:** L (RRI 53 Med-high, presentation-time recomputation — `docs/audit/antares-t2c-rri.md` § T2c-2 addendum, 2026-07-30)
- **RRI artifact:** `docs/audit/antares-t2c-rri.md` § T2c-2
- **Depends on:** T2c-1 (`[x] Done`, 2026-07-30)
- **Decomposed from:** T2c

### Objective

Enforce CPU/RAM/PID/output resource caps and the 15-command wall-time budget on
top of T2c-1's runner, and guarantee teardown under every termination path.

### Scope boundary with T2c-1

T2c-1 owns single-process lifecycle (launch, isolate, per-command timeout, kill
on that timeout). T2c-2 owns aggregate session accounting: the 15-command
counter, wall-clock budget across the whole run, per-command CPU/RAM/PID/output
caps, and confirming (not assuming) teardown after every T2c-1 exit path
(`SANDBOX_EXECUTION_COMPLETE`, `SANDBOX_RUNTIME_UNAVAILABLE`,
`SANDBOX_COMMAND_TIMED_OUT`).

### Happy paths considered

- **HP-2:** a multi-command run stops cleanly after a successful terminal
  submission within the 15-call budget.

### Edge cases considered

- **EC-1:** per-command timeout, wall-time exhaustion, or output-limit breach
  produces a distinct degraded terminal state.
- **EC-3:** teardown runs even after timeout or sandbox violation.

### Acceptance criteria

- CPU, RAM, PID, and output caps are enforced deterministically per command.
- **Output cap uses streaming/polling with early-abort, not post-hoc
  truncation.** Phase-1 review (Gemma, `docs/audit/gemma-evidence/antares-t2c-2-phase1.json`)
  flagged post-hoc truncation as insufficient: `process.communicate()` buffers
  unboundedly before returning, so a process that writes arbitrarily large
  output can OOM the supervisor before any cap is checked — converting an
  exhaustion attempt into a host-level DoS. The runner must read
  stdout/stderr incrementally (polling or a reader thread) and kill the
  process once the byte cap is exceeded, not after `communicate()` returns.
- Resource limits (CPU/RAM/PID via `RLIMIT_CPU`/`RLIMIT_AS`/`RLIMIT_NPROC`)
  compose with T2c-1's `_drop_privileges` into a single `preexec_fn` — Python's
  `Popen` accepts only one. Ordering matters: setting limits must happen
  before privilege drop where the limit itself requires privilege to raise,
  and must not silently no-op after privileges are already dropped.
- Wall timeout and the 15-command budget produce explicit degraded termination
  states, never a silent stop. Budget exhaustion is a pre-flight guard (checked
  before starting a command); per-command timeout is a runtime result of an
  already-started command — the two are sequential checks, not competing
  outcomes, and must remain distinguishable in the reported terminal state
  (e.g. command #15 timing out reports `SANDBOX_COMMAND_TIMED_OUT`, not budget
  exhaustion; a 16th command is refused before it starts).
- Teardown executes on every exit path (success, timeout, violation, exhaustion) —
  verified with an active post-kill check (e.g. confirm no process from the
  killed group remains after a short grace period), not assumed from the kill
  call succeeding. The verification step itself must not use an unbounded
  read (do not reintroduce the output-cap problem inside teardown logic).
- If POSIX resource-limit primitives are unavailable on the host platform, the
  runner fails closed to `SANDBOX_RUNTIME_UNAVAILABLE` rather than running
  without enforced caps — consistent with T2c-1's network-isolation precedent.

### Evidence to emit

- Budget/teardown module (`scripts/antares/sandbox_budget.py`), resource-cap
  fixtures, and teardown-verification tests covering every termination path.

### Status artifacts affected

- this ledger and the slice plan

### Peer Reviewer evidence (phase 1 — task-analysis review)

- Reviewer: `gemma` (fallback — see below)
- Command: manual Ollama `/api/chat` invocation, packet at
  `/private/tmp/claude-501/-Users-matias-dubbridge/5a356528-f43c-46c4-ba10-cbfd75ef2044/scratchpad/t2c2-phase1-packet.txt`
- Artifact: `docs/audit/gemma-evidence/antares-t2c-2-phase1.json`
- Verdict: `PASS` (with architectural guidance, folded into acceptance criteria above)
- Findings: scope decomposition confirmed clean; output-cap mechanism must be
  decided now (streaming, not post-hoc); `preexec_fn` composition and
  post-kill teardown verification flagged as concrete implementation risks;
  command #15 budget-vs-timeout sequencing clarified; platform fail-closed
  assumption confirmed sound
- qwen3.6:27b-q4_K_M attempts: 2 (first: socket timeout at 280s, no response;
  retry: `done_reason: length`, empty `content` after 2048 tokens consumed
  entirely by internal `thinking` — both unusable per policy)
- Gemma fallback: `triggered` — reason: qwen unavailable after mandatory
  immediate retry (empty content both attempts)
- D14 fallback: `not triggered` — reason: Gemma fallback produced a usable,
  complete (`done_reason: stop`) result on first call
- disposition_divergence: `none`
- Primary-agent disposition: accepted all findings; incorporated into
  acceptance criteria (output-cap mechanism, preexec_fn composition note,
  teardown verification requirement, command #15 sequencing) before
  presentation

```
Task-analysis review: gemma docs/audit/gemma-evidence/antares-t2c-2-phase1.json - PASS
```

### Reflection log

Required passes: 3 (`53` → `Med-high`)

#### Pass 1 (resource-limit composition)

- **Draft verdict:** `_compose_preexec` composes `RLIMIT_CPU`/`RLIMIT_AS`/`RLIMIT_NPROC`
  with `_drop_privileges` into one `preexec_fn`, limits-before-privilege-drop, per
  acceptance criteria. `_resource_limits_available()` initially excluded only
  `RLIMIT_AS` on Darwin.
- **Critique findings:** empirical testing surfaced two real platform bugs, not
  design flaws: (1) `RLIMIT_NPROC` on Darwin/BSD is scoped to the entire UID
  system-wide, not the sandboxed command's own process tree, so the module
  default (16) made trivial multi-process pipelines (`yes | head`) fail with
  `fork: Resource temporarily unavailable`; (2) `_verify_teardown` only caught
  `ProcessLookupError` around the post-kill existence probe, but macOS can raise
  `PermissionError` for a stale/reused pgid, causing test errors.
- **Revisions applied:** extended `_resource_limits_available()`'s Darwin
  exclusion to cover `RLIMIT_NPROC` as well as `RLIMIT_AS` (owner-approved,
  Option 1 of two presented) — no cap value is simultaneously a real per-command
  bound and compatible with an ordinary shell pipeline on this platform, so the
  whole session fails closed there rather than enforce a partial/fake cap.
  Updated the `_preexec_without_darwin_unenforceable_rlimits` test double
  (renamed from `_preexec_without_rlimit_as`) to match. Widened
  `_verify_teardown`'s exception handling to `(ProcessLookupError,
  PermissionError)` in both the polling loop and the final check.

#### Pass 2 (output-cap safety)

- **Draft verdict:** `_read_capped` never calls `process.communicate()`; the
  main read loop polls via `selectors` with a bounded chunk size and checks the
  byte total after every chunk, satisfying the phase-1 review's core
  requirement.
- **Critique findings:** re-running the suite surfaced a genuine resource leak
  (unclosed stdout/stderr pipe fds and un-reaped zombies on the
  kill/cap-exceeded exit path — `ResourceWarning` and `subprocess still
  running` warnings under test). The isolated phase-2 reviewer additionally
  found the "drain remaining output after the process exits" branch used a raw
  blocking `os.read` in a `while True` loop despite a comment claiming it was
  non-blocking — a grandchild still holding a pipe's write end open (e.g. `yes`
  outliving `sh` in `yes | head`) could hang it indefinitely, reintroducing the
  exact unbounded-read failure class this module exists to prevent.
- **Revisions applied:** added `_close_process_pipes()` and `process.wait()`
  calls on both exit paths to reap zombies and release fds. Replaced the
  drain-on-exit loop's raw `os.read` with the same `selectors`-based,
  time-bounded pattern (0.1s deadline, 0.02s per-select timeout) used by the
  main read loop.

#### Pass 3 (teardown verification)

- **Draft verdict:** `_kill_process_group` → `_verify_teardown` → `process.wait()`
  ordering on the kill path is deliberate: the kernel will not recycle
  `process.pid` until the zombie is reaped, so verifying before reaping avoids a
  TOCTOU where a recycled pid could resolve to an unrelated process.
- **Critique findings:** the isolated phase-2 reviewer found `_verify_teardown`'s
  boolean return value was computed but discarded at both call sites — a kill
  that could not be confirmed within its grace period was silently reported as
  a plain `SANDBOX_OUTPUT_CAP_EXCEEDED`/`SANDBOX_COMMAND_TIMED_OUT`, indistinguishable
  from a clean kill, defeating the "actively verified" acceptance criterion.
  Separately, `_kill_process_group`'s `process.kill()` fallback was unguarded
  and could itself raise `ProcessLookupError` on the same already-exited race
  `_verify_teardown` already treats as success, risking an unhandled exception
  out of `run_budgeted`. A third finding: the wall-budget-vs-command-timeout
  discriminator (`effective_timeout < command_timeout_seconds`) was a
  post-hoc float comparison rather than a decision made once at the point of
  computation, fragile at exact equality.
- **Revisions applied:** added `TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED`
  and wired the kill path to return it when `_verify_teardown` is `False`,
  before the cap/timeout branches. Wrapped `_kill_process_group`'s
  `process.kill()` fallback in its own `try/except` for the same exception
  family. Replaced the float comparison with a `wall_budget_is_binding`
  boolean computed once from `remaining_wall_seconds() <= command_timeout_seconds`
  at the same point `effective_timeout` is derived, so exact equality correctly
  attributes to the wall budget. The clean-exit path's own
  `_verify_teardown(process)` call after `process.wait()` was deliberately left
  unwired to a distinct outcome — the process has already exited on its own at
  that point (no kill was issued), so there is nothing for verification to
  meaningfully fail; this was confirmed as an intentional scope boundary, not
  an oversight, by both the primary agent's own analysis and the phase-2
  reviewer's independent re-check.

### Peer Reviewer evidence (phase 2 — code-solution review)

- Reviewer: `d14` (context-isolated cloud subagent; general-purpose, isolated
  from the orchestrator's development transcript — fed only the diff/files and
  acceptance criteria, per the D14 isolation-packet contract)
- Session-scoped routing note: `qwen3.6:27b-q4_K_M` was attempted first per the
  canonical 26–55 band chain. Two consecutive attempts against the same
  packet both failed with a socket timeout (600s, then 900s with a bounded
  `num_predict`) — Ollama was independently confirmed to be under contention
  from another concurrent local process at the time (`llama-server` actively
  consuming CPU). Per explicit owner instruction for this session only
  ("dado que hay otro proceso usando los modelos locales... prioriza derivar a
  cloud los reviews... define agentes enfocados en la tarea a realizar
  aislados del orquestador"), the fallback for this task was redirected from
  the local Gemma step to an isolated cloud subagent instead, preserving the
  D14 isolation contract (context-isolated, advisory-only, diff + criteria
  only, no development transcript). This is a one-session routing
  substitution, not a change to the standing band-routed reviewer policy.
- Command: `Agent` tool invocation, `subagent_type: general-purpose`,
  isolation packet at
  `/private/tmp/claude-501/-Users-matias-dubbridge/5a356528-f43c-46c4-ba10-cbfd75ef2044/scratchpad/t2c2-sandbox_budget.py`,
  `t2c2-sandbox_budget_test.py`, `t2c2-terminal_state.diff`
- Artifact: agent transcript (`ae34b3ac176c56d67`), two calls — initial review
  and post-fix re-review
- Verdict: initial `FINDINGS` (2 BLOCKING, 3 MAJOR, 3 MINOR) → **`PASS`** after
  all 8 findings were fixed and re-verified independently by the same isolated
  agent against the post-fix files
- Findings (initial pass): (1) BLOCKING — `_verify_teardown`'s return value
  discarded, unconfirmed kills unreported; (2) BLOCKING — no test coverage for
  that failure branch; (3) MAJOR — blocking `os.read` in the drain-on-exit
  loop could hang on a surviving grandchild; (4) MAJOR — `_kill_process_group`
  fallback could raise past the caller; (5) MAJOR — fragile float-comparison
  timeout/wall-budget discriminator; (6) MINOR — under-commented redundant
  teardown check; (7) MINOR — weak CPU-limit test assertion (elapsed time only,
  no outcome-kind check); (8) MINOR — no test for the kill-fallback exception
  path
- qwen3.6:27b-q4_K_M attempts: 2 (both: socket timeout, no response — packet
  size against a contended local model, consistent with the phase-1 review's
  prior experience with this model on large packets)
- Gemma fallback: `not triggered` — reason: session-scoped owner override
  redirected the fallback path to an isolated cloud subagent instead of local
  Gemma, specifically because Ollama was independently confirmed contended by
  another process
- D14 fallback: `triggered` — reason: `qwen3.6:27b-q4_K_M` unavailable after
  the mandatory immediate retry; routed to the context-isolated subagent per
  the standing D14 fallback contract, substituting a cloud isolated agent for
  the usual local Balanced-tier subagent for this session only
- disposition_divergence: `none` — every finding was independently reproduced
  and verified against the actual code by the primary agent before a fix was
  applied (see Reflection log above), not merely accepted on the reviewer's
  assertion; the reviewer's post-fix re-check independently confirmed all 8 as
  FIXED with no new findings
- Primary-agent disposition: accepted all 8 findings; all fixed and covered by
  new/tightened unit tests; one informational observation from the re-review
  (clean-exit path's own unwired `_verify_teardown` call) evaluated and
  confirmed as an intentional scope boundary, not a defect

```
Code-solution review: d14 (isolated cloud subagent ae34b3ac176c56d67) - PASS
```

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-2 | Happy path | command within budget completes, counter increments | `scripts/antares/sandbox_budget_test.py::RunBudgetedHappyPathTest::test_hp2_command_within_budget_completes_and_increments_counter` | passed |
| HP-2 | Happy path | a full 15-command run stops cleanly within budget | `scripts/antares/sandbox_budget_test.py::RunBudgetedHappyPathTest::test_hp2_a_full_multi_command_run_stops_cleanly_within_budget` | passed |
| EC-1 | Edge case | 16th command refused before starting (pre-flight) | `scripts/antares/sandbox_budget_test.py::RunBudgetedCommandBudgetTest::test_ec1_sixteenth_command_is_refused_before_starting` | passed |
| EC-1 | Edge case | command #15 timing out reports timeout, not exhaustion | `scripts/antares/sandbox_budget_test.py::RunBudgetedCommandBudgetTest::test_ec1_command_number_fifteen_timing_out_reports_timeout_not_exhaustion` | passed |
| EC-1 | Edge case | wall budget already exhausted refuses before starting | `scripts/antares/sandbox_budget_test.py::RunBudgetedWallBudgetTest::test_ec1_wall_budget_already_exhausted_is_refused_before_starting` | passed |
| EC-1 | Edge case | wall budget exhausted mid-command is distinct from command timeout (incl. exact-equality discriminator fix) | `scripts/antares/sandbox_budget_test.py::RunBudgetedWallBudgetTest::test_ec1_wall_budget_exhausted_mid_command_is_distinct_from_command_timeout` | passed |
| EC-1 | Edge case | output cap breach aborts early, not after buffering full output | `scripts/antares/sandbox_budget_test.py::RunBudgetedOutputCapTest::test_ec1_output_cap_breach_aborts_early_not_after_communicate` | passed |
| EC-1 | Edge case | resource limits unavailable fails closed (generic) | `scripts/antares/sandbox_budget_test.py::RunBudgetedRuntimeUnavailableTest::test_ec1_resource_limits_unavailable_fails_closed` | passed |
| EC-1 | Edge case | Darwin host fails closed end-to-end, unpatched | `scripts/antares/sandbox_budget_test.py::RunBudgetedDarwinFailClosedTest::test_ec1_darwin_host_fails_closed_without_any_patching` | passed |
| EC-3 | Edge case | teardown confirmed after timeout kill | `scripts/antares/sandbox_budget_test.py::RunBudgetedTeardownTest::test_ec3_teardown_confirmed_after_timeout_kill` | passed |
| EC-3 | Edge case | teardown confirmed after output-cap kill | `scripts/antares/sandbox_budget_test.py::RunBudgetedTeardownTest::test_ec3_teardown_confirmed_after_output_cap_kill` | passed |
| EC-3 | Edge case | unconfirmed teardown is its own distinct, observable outcome | `scripts/antares/sandbox_budget_test.py::RunBudgetedTeardownTest::test_ec3_unconfirmed_teardown_is_its_own_distinct_outcome` | passed |
| EC-3 | Edge case | kill-fallback race (already-exited process) does not raise | `scripts/antares/sandbox_budget_test.py::KillProcessGroupTest::test_fallback_kill_swallows_already_exited_race_instead_of_raising` | passed |

Supporting (non-`HP`/`EC`-tagged, but load-bearing for the acceptance criteria):
`ResourceLimitsAvailabilityTest` (Darwin/non-Darwin fail-closed detection),
`SessionBudgetUnitTest` (preflight/remaining-seconds arithmetic),
`ComposedPreexecTest::test_cpu_rlimit_actually_terminates_a_cpu_bound_loop`
(empirical proof of `RLIMIT_CPU` enforcement via signal-killed exit code, not
just elapsed time), `RunBudgetedOutputCapTest::test_hp2_output_within_cap_is_captured_completely`.

Full suite: 19/19 passed (`python3 -m unittest scripts.antares.sandbox_budget_test`,
also clean under `-W error::ResourceWarning`, confirming no fd/zombie leaks).
T2c-1's own 19 tests (`sandbox_runner_test.py`, `terminal_state_test.py`)
remain passing, unaffected by the `terminal_state.py` addition.

### Owner final verification

- **Status: WAIVED (not a genuine verification).** Asked explicitly whether
  the owner had personally run or reviewed the verification commands before
  authorizing closure (`AskUserQuestion`, 2026-07-30). The owner's answer was
  "Autorizo cerrar sin verificación manual explícita" — an explicit,
  documented waiver of personal test-by-test review, not a statement that the
  owner verified each `HP-#`/`EC-#` case's unit test evidence genuinely
  covers the claimed behavior. Recorded per
  `docs/policies/HITL_AUTONOMY_POLICY.md`'s waiver provision: "A user may
  waive this checkpoint only by explicitly authorizing execution without
  another approval for a clearly bounded task; record that waiver."
- Owner: `Matias Kruk`
- Date: `2026-07-30`
- Waiver statement (verbatim): "Autorizo cerrar sin verificación manual
  explícita."
- Agent-run commands (owner did not independently run or review these before
  waiving): `python3 -m unittest scripts.antares.sandbox_budget_test -v`;
  `python3 -W error::ResourceWarning -m unittest
  scripts.antares.sandbox_budget_test -v` (confirms no fd/zombie leaks);
  `python3 -m unittest scripts.antares.sandbox_runner_test
  scripts.antares.terminal_state_test` (confirms T2c-1 unaffected).

## T2d - Versioned artifact schema and redacted trace contract

- **Status:** `[x] Done (owner-waived, 2026-07-30)`
- **Type:** development / security-sensitive tooling
- **Effort:** Moderate/Med-high band (RRI 50) — see `docs/audit/antares-t2d-rri.md`
- **Depends on:** T2c-2
- **Decomposed from:** T2

### Objective

Normalize every one of the 20 `TerminalStateKind` values (`scripts/antares/terminal_state.py`)
into a versioned artifact with provenance, terminal-state semantics, a concrete
redacted trace-reference contract, and mandatory human-disposition fields.

### Redacted trace-reference contract (resolves qwen27/Gemma Phase-1 BLOCKING finding)

- Raw `stdout`/`stderr`/`detail`/prompt/source-excerpt content is **never**
  written into the committed artifact. Instead the artifact stores a
  `trace_ref` object:
  ```json
  {
    "trace_ref": {
      "content_hash": "sha256:<hex>",
      "storage_uri": "file://<local-fs-path-outside-docs/>",
      "byte_length": 12345,
      "redaction_version": 1
    }
  }
  ```
- `storage_uri` must resolve outside any path `make qa-docs` or `git` tracks
  (e.g. under a gitignored `var/antares-traces/` root); the schema validator
  rejects any `storage_uri` that resolves inside a tracked path.
- `content_hash` is SHA-256 over the raw (pre-redaction) trace bytes, so a
  human reviewer with access to the raw store can verify the committed
  artifact matches a specific trace without the trace itself being committed.
- `redaction_version` is a small integer identifying which redaction ruleset
  produced the reference, so future rule changes don't silently reinterpret
  old artifacts.
- Terminal states with no trace content (e.g. `MALFORMED_TOOL_CALL`) set
  `trace_ref: null` rather than omitting the field.
- **Validator scope boundary:** the schema validator checks shape and
  consistency only (field presence, enum membership, the category-conditional
  required fields below, the EC-2/EC-4 rejection rules). It does **not**
  perform I/O to confirm `content_hash` matches the bytes at `storage_uri` —
  that would require reading the raw (non-committed) store at validation
  time, which the validator has no access to by design. The hash-integrity
  guarantee is instead a property of the **writer**: the writer function
  (part of this task's "Evidence to emit") computes `content_hash` directly
  from the raw bytes at artifact-creation time, before writing either the
  raw trace or the `trace_ref`. A validation fixture asserts the writer's
  round-trip (write raw bytes -> compute hash -> read back via
  `storage_uri` -> hash matches), which is a writer-module test, not a
  schema-validator test.

### Per-category payload shape (resolves qwen27 MAJOR finding, Focus 2)

The common base fields below apply to all 20 kinds. Each category then adds
its own required fields; the validator enforces category-conditional
requirements (e.g. `SUBMITTED_VULNERABLE_FILES` without `candidates` is
rejected regardless of the fact that `candidates` is merely "present" on the
struct).

**Common base (all 20 kinds):** `schema_version`, `kind`, `finding_id`,
`artifact_id`, `supersedes` (nullable), `provenance`, `trace_ref` (nullable),
`disposition`.

| Category | Kinds | Category-specific required fields | `trace_ref` |
|---|---|---|---|
| T2a parser (7) | `PARSED_TERMINAL_CALL`, `SUBMITTED_VULNERABLE_FILES`, `SUBMITTED_NO_VULNERABILITY_FOUND`, `MALFORMED_TOOL_CALL`, `UNSUPPORTED_TOOL_NAME`, `MALFORMED_SUBMIT_PAYLOAD`, `DUPLICATE_TERMINAL_SUBMISSION` | `argv` on `PARSED_TERMINAL_CALL`; `candidates` (non-empty, EC-4) on `SUBMITTED_VULNERABLE_FILES`; `detail` on all rejection kinds | always `null` (no subprocess trace exists at parse time) |
| T2b policy (6) | `COMMAND_PLAN_VALID`, `PATH_CONTAINMENT_VALID`, `COMMAND_REJECTED_SHELL_SYNTAX`, `COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED`, `COMMAND_REJECTED_OPTION_NOT_ALLOWED`, `PATH_REJECTED_CONTAINMENT_ESCAPE` | `argv` on `COMMAND_PLAN_VALID`; `candidates` on `PATH_CONTAINMENT_VALID`; `detail` on all rejection kinds | always `null` (still pre-execution) |
| T2c-1 execution (3) | `SANDBOX_EXECUTION_COMPLETE`, `SANDBOX_RUNTIME_UNAVAILABLE`, `SANDBOX_COMMAND_TIMED_OUT` | `argv`, `exit_code` (nullable only for `SANDBOX_RUNTIME_UNAVAILABLE`), `elapsed_seconds` | non-null on `SANDBOX_EXECUTION_COMPLETE`/`SANDBOX_COMMAND_TIMED_OUT` (stdout/stderr exist); `null` on `SANDBOX_RUNTIME_UNAVAILABLE` (nothing ran) |
| T2c-2 budget (4) | `SANDBOX_OUTPUT_CAP_EXCEEDED`, `SANDBOX_WALL_BUDGET_EXCEEDED`, `SANDBOX_BUDGET_EXHAUSTED`, `SANDBOX_TEARDOWN_UNCONFIRMED` | `elapsed_seconds`; `budget: {limit, consumed, unit}`; `teardown_grace_seconds` additionally required on `SANDBOX_TEARDOWN_UNCONFIRMED` | non-null (a command ran before the budget/teardown condition fired) |

### Mandatory human-disposition fields (resolves qwen27/Gemma Phase-1 BLOCKING finding)

Every artifact instance carries a `disposition` object, required (not
optional) on every terminal state, including non-finding states:

```json
{
  "disposition": {
    "state": "needs-human-review",
    "reviewer": null,
    "reviewed_at": null,
    "note": null
  }
}
```

- `state` is one of the enum values: `needs-human-review` (default at
  creation), `accepted-now`, `accepted-follow-up`, `rejected`. This mirrors
  the four dispositions already named in `AGENT_WORKFLOW_GUIDE.md`'s Antares
  authority-boundary section.
- `state` starts at `needs-human-review` for every terminal state at
  creation time — including negative (`SUBMITTED_NO_VULNERABILITY_FOUND`) and
  degraded/rejected states — so nothing is ever created pre-closed (EC-3).
  Moving to any other `state` requires `reviewer` and `reviewed_at` to be
  non-null in the same write (schema validator enforces this pairing).
- The artifact schema is append-only per ADR-006 (immutable artifacts): a
  disposition change is written as a new versioned artifact revision with a
  `supersedes: <artifact_id>` back-reference, never an in-place mutation of
  a committed file.
- **Stable identity across revisions (resolves qwen27 MINOR finding, Focus 5
  — "provenance audit trail for asynchronous human disposition updates"):**
  `artifact_id` changes on every revision (it identifies one immutable
  file); `finding_id` is generated once at creation (e.g. a UUID) and copied
  unchanged into every subsequent revision in that finding's supersede
  chain. A consumer resolving "the current disposition of finding X" walks
  the chain by `finding_id` and takes the revision no other committed
  artifact names in its `supersedes` field (the chain head). Without a
  stable `finding_id`, a consumer has no non-ambiguous way to ask "what is
  the latest disposition for this specific finding" across an append-only
  revision history — this was the concrete gap the finding flagged, not
  addressed by `supersedes` alone.
- Backwards-compatibility guarantee: out of scope for this task by design —
  `schema_version` starts at `1` and this task ships only one version;
  compatibility across versions is deferred to the future migration task
  that EC-5's `schema_version` field anticipates. This is a scope boundary,
  not an unaddressed gap.

### Happy paths considered

- **HP-1:** `vulnerable_files` records validated candidate paths, provenance,
  and a `trace_ref` per the contract above, without the raw stdout/stderr or
  source excerpts appearing anywhere in the committed artifact.
- **HP-2:** `no_vulnerability_found` records an explicit negative result with
  the same provenance, `trace_ref`, and `disposition` fields as a positive
  result — disposition starts at `needs-human-review`, not auto-closed.

### Edge cases considered

- **EC-1:** degraded states (`SANDBOX_BUDGET_EXHAUSTED`, `SANDBOX_COMMAND_TIMED_OUT`,
  `SANDBOX_OUTPUT_CAP_EXCEEDED`) serialize to a distinct `kind` value each
  (no collapsing into one generic "degraded" bucket) and carry
  `disposition.state = needs-human-review` at creation.
- **EC-2:** a validation fixture asserts that no committed fixture or example
  file contains a raw trace body; only `trace_ref` objects with hashes/URIs
  are present. The schema validator rejects any artifact instance where a
  trace-bearing field (`stdout`/`stderr`/`detail`) is non-empty alongside a
  populated `trace_ref`.
- **EC-3:** an artifact with `disposition.state = needs-human-review` is
  distinguishable from a closed one by state alone (never by field absence);
  a fixture asserts a needs-human-review artifact remains valid against the
  schema (open findings are not schema violations).
- **EC-4 (new, from Phase-1 review):** `SUBMITTED_VULNERABLE_FILES` with an
  empty `candidates` tuple is rejected by the schema validator as malformed
  (a positive finding kind must carry at least one candidate path).
- **EC-5 (new, from Phase-1 review):** a `schema_version` field is present on
  every artifact; the validator rejects an artifact whose `schema_version` it
  does not recognize, rather than attempting best-effort parsing. Migration
  between versions is out of scope for this task (single version, `1`, at
  first release) but the field's presence is required now so a future
  migration task has something to key off.
- **EC-6 (new, this pass):** a disposition-update fixture writes an initial
  revision (`finding_id = F`, `artifact_id = A1`, `supersedes = null`), then
  a second revision (`finding_id = F`, `artifact_id = A2`,
  `supersedes = A1`). The fixture asserts: (a) both revisions share the same
  `finding_id`; (b) resolving "current disposition of `F`" returns `A2`, not
  `A1`; (c) a malformed revision that changes `finding_id` mid-chain, or
  that sets `supersedes` to a non-existent `artifact_id`, is rejected by the
  validator.

### Acceptance criteria

- The schema is versioned (`schema_version` field, starting at `1`) and
  covers every one of the 20 `TerminalStateKind` values with a distinct,
  non-collapsed `kind` string and the category-specific required fields in
  the payload-shape table above.
- Provenance includes model/runtime/harness versions plus packet/snapshot
  hashes (existing requirement, unchanged).
- `disposition` (per the four-state enum above) and `trace_ref` (per the
  contract above) are both mandatory, non-optional fields on every artifact
  instance, including negative and degraded/rejected states.
- `finding_id` is stable across a supersede chain; `artifact_id` is unique
  per revision; the validator rejects a chain break (EC-6).
- Committed summaries, fixtures, and examples exclude prompts, source
  excerpts, credentials, and secrets; validated by EC-2's fixture assertion.
- A malformed `SUBMITTED_VULNERABLE_FILES` (empty `candidates`) is rejected
  at validation time (EC-4).

### Evidence to emit

- Schema module (`scripts/antares/artifact_schema.py` or similar), a
  writer function (computes `content_hash` from raw bytes at creation time),
  a validator function (shape/consistency only, no I/O — see validator
  scope boundary above), validation fixtures (EC-2, EC-4, EC-6, plus the
  writer hash round-trip fixture), and one redacted example artifact per
  each of the 20 `TerminalStateKind` values.

### Status artifacts affected

- this ledger and the slice plan

### Implementation note (ADR-038 routing)

RRI = 50 (Med-high, 41-55) — `docs/audit/antares-t2d-rri.md`. Qwen27
(`qwen3.6:27b-q4_K_M`, `scripts/local-architect/run_analysis.py`
`med-high-refinement-v1` profile) returned `route_recommendation: GO_LOCAL`.
The primary agent issued its own hash-bound route receipt and **downgraded**
to `CLOUD_REQUIRED`, invoking the ADR-038 §6 hard exclusion: this schema
mechanically enforces the mandatory-human-disposition governance contract for
Antares (an advisory-only security tool), which is a fail-closed governance
invariant. `scripts/local-agent/med_high_gate.py` confirms
`{"route": "CLOUD_REQUIRED", "reason": "Primary receipt downgraded GO_LOCAL to
cloud."}`. Implemented directly by the primary (cloud) agent, per ADR-038 §5.

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M`
- Phase 1 command: Ollama `/api/chat`, `docs/audit/gemma-evidence/antares-t2d-phase1.json`
- Phase 2 command: Ollama `/api/chat` with `"think": false` (see note below on
  why), `docs/audit/gemma-evidence/antares-t2d-phase2.json`
- Phase 1 verdict: `BLOCKED` (pass 1, 6 findings: 2 BLOCKING, 2 MAJOR, 2 MINOR)
  → task definition revised (redacted trace-reference contract, per-category
  payload shape, mandatory human-disposition fields, stable
  `finding_id`/`artifact_id` identity, validator scope boundary, concrete
  EC-4/5/6) → re-reviewed (pass 2) → `PASS`, all 6 pass-1 findings
  independently confirmed resolved against the revised text, plus one new
  MINOR accepted-follow-up (supersede-chain retention/tombstone policy, out
  of scope for T2d).
- Phase 2 verdict: `PASS`. 8 findings returned, 7 `NONE` + 1 `MINOR`
  (`_validate_storage_uri` has no explicit `~`-style absolute-path rejection).
  Independently verified: `~`-prefixed URIs are already rejected via the
  `ALLOWED_TRACE_STORAGE_PREFIX` allowlist check (`storage_uri_outside_allowed_root`),
  confirming the reviewer's own hedge ("though the repo-relative check likely
  covers this") was correct. Closed with an explicit regression test
  (`test_storage_uri_tilde_path_is_rejected`) rather than left implicit.
- Operational note: the first two Phase-2 attempts (default request, no
  `think` parameter) each timed out at 700s wall-clock with decode stuck at
  ~4.4-4.7 tokens/second (`n_decoded` reaching 1853 and 2633 respectively
  before the client `curl` hit `--max-time` and the in-progress generation
  was discarded — `stream: false` means no partial output is recoverable on
  timeout). `ollama show qwen3.6:27b-q4_K_M` confirmed the model has a
  `thinking` capability that was never disabled in those requests. Adding
  `"think": false` to the third attempt cut decode to ~29 tokens/second (a
  ~6x change) and the model completed in 206.6s with `done_reason: "stop"`
  (natural completion, not truncation) — strong evidence the earlier
  timeouts were consumed by invisible reasoning tokens rather than the
  requested 8-finding response itself.
- Gemma fallback: not triggered (both phases resolved directly against
  `qwen3.6:27b-q4_K_M`).
- D14 fallback: not triggered.
- disposition_divergence: `none`.
- Primary-agent disposition: accepted all 7 phase-2 `NONE` findings as
  confirming correctness; the 1 `MINOR` finding was independently verified
  (not just accepted on the model's word) and closed with a new regression
  test. Phase-1's 6 original findings were resolved via genuine task-definition
  redesign (not cosmetic reclassification) prior to the passing re-review.

```
Task-analysis review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2d-phase1.json - PASS
Code-solution review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2d-phase2.json - PASS
```

### Reflection log

Required passes: 3 (`50` → `Med-high`)

#### Pass 1

- **Draft verdict:** initial `artifact_schema.py` validated `content_hash`
  by checking only the `"sha256:"` prefix, not that the remaining 64
  characters were valid lowercase hex.
- **Critique findings:** a malformed hash (e.g. `sha256:not-actually-hex`)
  would pass validation, silently breaking the writer's hash-integrity
  guarantee described in the redacted trace-reference contract.
- **Revisions applied:** added `_is_valid_sha256_hex()` (prefix + exact
  64-char lowercase-hex check) and wired it into `_validate_trace_ref_field`;
  added `test_malformed_content_hash_is_rejected`.

#### Pass 2

- **Draft verdict:** `validate_supersede_chain` correctly resolved linear
  chains, but had no test coverage for a forked chain (two artifacts naming
  the same `supersedes` target) or the single-artifact baseline case.
- **Critique findings:** an unproven fork/cycle path is exactly the kind of
  gap a schema meant to protect an append-only audit trail (ADR-006) cannot
  afford to leave unverified — EC-6 explicitly names "rejects a chain break."
- **Revisions applied:** added `test_ec6_rejects_forked_chain_with_two_heads`
  and `test_ec6_single_artifact_chain_resolves_to_itself`; also added
  `test_ec6_rejects_dangling_supersedes` for the third EC-6 sub-case
  (non-existent `supersedes` target).

#### Pass 3

- **Draft verdict:** `validate_supersede_chain` computed chain-relational
  properties (head resolution, `finding_id` consistency) without first
  calling `validate_artifact()` on each element.
- **Critique findings:** a chain composed of individually schema-invalid
  artifacts (e.g. one with an unrecognized `schema_version`) could report a
  clean, resolvable head — the chain-level check and the artifact-level
  check were independent, so passing one didn't imply the other.
- **Revisions applied:** `validate_supersede_chain` now validates every
  element via `validate_artifact()` before computing chain properties;
  added `test_ec6_chain_rejects_an_individually_malformed_artifact` as proof.

Gemma/qwen27 Phase-1 findings were dispositioned by task-definition revision
(not code changes, since Phase 1 precedes implementation) — see the pass-2
resolution detail in `docs/audit/gemma-evidence/antares-t2d-phase1.json`.
Phase-2's single MINOR finding is dispositioned above under Peer Reviewer
evidence, not as a fourth Reflection pass (it arrived after all 3 required
passes were already complete, per the closure-checklist ordering).

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `SUBMITTED_VULNERABLE_FILES` carries `trace_ref`, provenance, and no raw stdout/stderr/source excerpt in the committed artifact | `scripts/antares/artifact_schema_test.py::HappyPathTest::test_hp1_vulnerable_files_carries_trace_ref_and_no_raw_content` | passed |
| HP-2 | Happy path | `SUBMITTED_NO_VULNERABILITY_FOUND` records an explicit negative result with the same provenance/`trace_ref`/`disposition` shape, not auto-closed | `scripts/antares/artifact_schema_test.py::HappyPathTest::test_hp2_no_vulnerability_found_matches_positive_result_shape` | passed |
| EC-1 | Edge case | degraded kinds serialize to distinct `kind` values and start `needs-human-review` | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec1_degraded_states_are_distinct_and_start_needs_human_review` | passed |
| EC-2 | Edge case | validator rejects a populated `trace_ref` alongside non-empty raw stdout/stderr | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec2_rejects_raw_trace_alongside_populated_trace_ref` | passed |
| EC-2 | Edge case | no committed example carries raw trace content | `scripts/antares/artifact_schema_test.py::ExampleArtifactsTest::test_examples_never_carry_raw_trace_content`, `CommittedExampleFixtureTest::test_committed_examples_contain_no_raw_trace_fields_and_validate` | passed |
| EC-3 | Edge case | a `needs-human-review` artifact is schema-valid (open findings are not violations) | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec3_needs_human_review_is_schema_valid_not_a_violation` | passed |
| EC-3 | Edge case | moving off `needs-human-review` without `reviewer`+`reviewed_at` is rejected | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec3_disposition_missing_reviewer_on_non_open_state_is_rejected` | passed |
| EC-4 | Edge case | `SUBMITTED_VULNERABLE_FILES` with empty `candidates` is rejected | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec4_empty_candidates_with_submitted_vulnerable_files_is_rejected` | passed |
| EC-5 | Edge case | an unrecognized `schema_version` is hard-rejected, not best-effort parsed | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec5_unrecognized_schema_version_is_rejected` | passed |
| EC-6 | Edge case | two revisions sharing `finding_id` resolve to the latest (`supersedes`-referenced) revision | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec6_chain_shares_finding_id_and_resolves_to_latest_revision` | passed |
| EC-6 | Edge case | a `finding_id` change mid-chain is rejected | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec6_rejects_finding_id_change_mid_chain` | passed |
| EC-6 | Edge case | a `supersedes` reference to a non-existent `artifact_id` is rejected | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec6_rejects_dangling_supersedes` | passed |
| EC-6 | Edge case | a forked chain (two artifacts naming the same `supersedes` target) is rejected | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec6_rejects_forked_chain_with_two_heads` | passed |
| EC-6 | Edge case | a single-artifact chain resolves to itself | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec6_single_artifact_chain_resolves_to_itself` | passed |
| EC-6 | Edge case | a chain containing an individually schema-invalid artifact is rejected before chain properties are computed | `scripts/antares/artifact_schema_test.py::EdgeCaseTest::test_ec6_chain_rejects_an_individually_malformed_artifact` | passed |

Supporting (non-`HP`/`EC`-tagged, but load-bearing for the acceptance criteria):
`test_writer_hash_roundtrip` and `test_writer_roundtrip_fails_if_bytes_are_tampered`
(writer/validator scope-boundary guarantee — content-hash integrity is the
writer's responsibility, proven by round-trip, not the validator's, per the
explicit scope-boundary decision); `CategoryFieldTest::test_all_20_kinds_are_covered_and_partitioned`
(category partition completeness, the assertion-based self-check quoted in
the implementation); `test_t2c1_runtime_unavailable_forbids_trace_ref` and
`test_t2c1_execution_complete_requires_trace_ref` (category-conditional
`trace_ref` nullability from the payload-shape table);
`test_t2c2_teardown_unconfirmed_requires_grace_seconds`; `test_malformed_content_hash_is_rejected`;
`test_storage_uri_outside_allowed_root_is_rejected` and
`test_storage_uri_tilde_path_is_rejected` (phase-2 MINOR finding closure);
`ExampleArtifactsTest` and `CommittedExampleFixtureTest` (all 20 committed
example artifacts generate, serialize, round-trip, and validate — the "one
redacted example artifact per kind" evidence-to-emit requirement).

Full suite: 28/28 passed (`python3 -m unittest scripts.antares.artifact_schema_test -v`).

### Owner final verification

- **Status: WAIVED (not a genuine verification).** Presented two questions via
  `AskUserQuestion` (2026-07-30): (1) how to handle the commit-provenance
  finding below, (2) how to close owner verification. For (2), the owner
  selected "Waiver explícito (como en T2c-2)" — an explicit, documented
  waiver of personal test-by-test review, not a statement that the owner
  independently verified each `HP-#`/`EC-#` case's unit test evidence
  genuinely covers the claimed behavior. Recorded per
  `docs/policies/HITL_AUTONOMY_POLICY.md`'s waiver provision, mirroring the
  T2c-2 precedent above.
- Owner: `Matias Kruk`
- Date: `2026-07-30`
- Waiver selection (verbatim option label): "Waiver explícito (como en
  T2c-2)" — Autorizás cerrar sin revisión manual caso-por-caso.
- Agent-run commands (owner did not independently run or review these before
  waiving): `python3 -m unittest scripts.antares.artifact_schema_test -v`
  (28/28 passed).

### Commit-provenance note (owner-reviewed, 2026-07-30)

Before closure, `git rev-parse HEAD` (this checkout has `core.bare = true`,
so `git status`/`git add`/`git diff` fail here with "must be run in a work
tree," but ref resolution works) showed this task's implementation files
(`scripts/antares/artifact_schema.py`, `artifact_schema_test.py`, all 20
`examples/*.json`, and `docs/audit/antares-t2d-rri.md`) were **already
committed** — under commits `da8e06a` ("S-140: add review-task subtitle
artifact identity", 17:02:46) and `844e847d` ("Docs: fix antares audit
frontmatter type", 17:05:34) — before this task's own closure record existed
and without the primary agent ever running `git commit` this session. Byte
comparison (`git show HEAD:<path>` vs. on-disk) confirmed the committed
content was identical to the in-progress, not-yet-reviewed working copy.
Root cause: both commits share the owner's own git identity and touch
unrelated `S-140` files in the same commit, indicating a concurrent session
in this same shared checkout ran a broad `git add` that swept in T2d's
uncommitted files as a side effect — not a repo hook (`.githooks/` has only
`pre-push`) and not a background daemon. Presented to the owner via
`AskUserQuestion`; selected disposition: **"Dejarlo así"** — leave the
existing commits as-is rather than rewrite shared history, since the content
is correct and another session may still be active. This note exists so a
future reader of `da8e06a`/`844e847d` understands why T2d's files appear
there under unrelated messages, predating T2d's own Phase-2 review and
Reflection log.

## T2e-pre - Decompose oversized T2c-2/T2d modules for local-first delegation eligibility

- **Status:** `[x] Done`
- **Type:** development / pure refactor (no intended behavior change)
- **Effort:** L
- **RRI:** 52 Med-high (`python3 scripts/rri.py --touches scripts/antares/artifact_schema.py
  --touches scripts/antares/artifact_validators.py --touches
  scripts/antares/artifact_trace_writer.py --touches
  scripts/antares/artifact_examples.py --touches
  scripts/antares/artifact_serialization.py --touches
  scripts/antares/sandbox_budget.py --touches
  scripts/antares/sandbox_process_io.py --touches
  scripts/antares/sandbox_resource_limits.py --touches
  scripts/antares/sandbox_session_budget.py --C 1 --D 3 --K 3 --P 4 --T 1 --A 1
  --X 3 --penalty refactor_and_behavior`; base 44 + refactor_and_behavior
  penalty +8 = 52)
- **Depends on:** T2c-2, T2d
- **Blocks:** T2e (resolves T2e's Delegation-routing note as decompose-first,
  not escalate — owner directive, 2026-07-30)
- **Decomposition-trigger note:** `scripts/rri.py`'s own output flags this task
  under `docs/policies/RRI_POLICY.md § Decomposition triggers`: "The +8 penalty
  is active (refactor + behavior change combined) — always separate refactor
  from functional change into distinct tasks/commits." That rule is satisfied
  by construction here: this task is refactor-only (zero intended behavior
  change) and is kept as its own task, strictly preceding T2e's functional
  harness-building work. It does not require this refactor to be split further
  into per-file subtasks; the two modules are decomposed together because both
  block the same downstream gate and share the same acceptance shape
  (existing tests pass unchanged).

### Implementation routing (resolved — owner override, one-off exception)

**Resolved by explicit owner override, Matias Kruk, 2026-07-30: `CLOUD_REQUIRED`.**
`sandbox_budget.py` implements the sandbox's process-isolation / RLIMIT /
teardown-verification security boundary; the owner judged that even a
zero-behavior-change refactor of this module should go to cloud
implementation rather than the local `qwen3.6:35b-a3b` session, given the
task's own reason for existing is to *reduce* the line count that would
otherwise be read/authored in one local-model turn — undermining that goal by
routing a security-adjacent split through the local model was the owner's
stated concern.

Procedurally this differs from T2d's downgrade (`docs/tasks/
antares-security-specialist-advisor.md` § T2d Implementation note): there, the
primary agent ran the Qwen27 advisory step first (`route_recommendation:
GO_LOCAL`) and then exercised its own downgrade authority. Here the owner
decided the route directly, before Qwen27 was invoked, so the Qwen27
advisory-refinement step is skipped as a formality that cannot change the
outcome — the primary/owner may always downgrade toward cloud per ADR-038 §4
("may downgrade, never upgrade"), and an explicit owner instruction is at
least as authoritative as the primary agent's own downgrade judgment.

**Scope of the exception:** this resolves routing for **T2e-pre only**. It is
not a change to `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`'s or
`docs/policies/HITL_AUTONOMY_POLICY.md`'s general Med-high ADR-038 routing
contract, and does not set a precedent that security-adjacent Med-high tasks
route to cloud by default — each task's routing is still resolved
independently at its own gate.

`scripts/local-agent/med_high_gate.py` result (recorded for audit parity with
the T2c-2/T2d precedent format): `{"route": "CLOUD_REQUIRED", "reason":
"Owner override, 2026-07-30 — one-off exception scoped to T2e-pre; security-
adjacent refactor target (sandbox_budget.py) routed to cloud ahead of the
Qwen27/local gate rather than through it."}`. Implementation proceeds with the
primary (cloud) agent, per ADR-038 §5, once this task itself is approved below.

### Objective

Split `artifact_schema.py` (574 lines) and `sandbox_budget.py` (540 lines) —
both already over the 500-line target-file-size gate for RRI 26-55
local-first delegation (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Target-file
size gate) — into smaller single-responsibility modules using concrete design
patterns, with **zero intended behavior change**, so T2e's implementer can
read every file it touches in full and stay under the gate.

### Phase 1 — Task-analysis review

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: Ollama `/api/chat`, `"think": false`
- Artifact: `docs/audit/gemma-evidence/antares-t2e-pre-phase1.json`
- Pass 1: `FINDINGS` (1 MAJOR, 3 MINOR). The MAJOR finding (EC-1's bare-name
  requirement scoped only to the 4 mocked symbols, not every name
  `run_budgeted` calls into a new submodule) was independently confirmed via
  grep against `scripts/antares/sandbox_budget.py` — which also surfaced an
  additional dependency the finding didn't name, `_drop_privileges` (a T2c-1
  re-export accessed directly by the test), requiring no change since it
  stays in the core file but needed stating explicitly. All 3 MINOR findings
  (test-entrypoint ambiguity, missing circular-import check, evidence section
  omitting test-pass output) were accepted as legitimate drafting gaps.
- Pass 2: `PASS`. All 4 pass-1 findings independently reconfirmed resolved
  against the revised text. One new MINOR finding, self-qualified by the
  reviewer as conditional on the decomposition table being absent from the
  deliverable — **rejected as a false positive**: the table is present
  unchanged at `### Proposed decomposition` above; the pass-2 prompt
  deliberately excerpted only the changed sections, which is the likely
  cause. Same class of issue as T2e phase-1 pass-2 finding #7
  (`docs/audit/gemma-evidence/antares-t2e-phase1.json`).
- Gemma fallback: not triggered. D14 fallback: not triggered.
- disposition_divergence: `none`.

```
Task-analysis review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2e-pre-phase1.json - PASS
```

### Proposed decomposition (verified against current file contents)

`artifact_schema.py` (574 lines) →

| New file | Contents | Pattern |
|---|---|---|
| `artifact_schema.py` (core, ~200 lines) | imports, constants, `ValidationError`, `DispositionState`, `T2A_KINDS`..`T2C2_KINDS` partition sets + self-check asserts, dataclasses (`Provenance`, `TraceRef`, `Disposition`, `Budget`, `Artifact`), `_category_of` | — |
| `artifact_validators.py` (~180 lines) | `_validate_storage_uri`, `_is_valid_sha256_hex`, `_validate_trace_ref_field`, `_validate_disposition`, `_validate_category_fields`, `validate_artifact`, `validate_supersede_chain` | Strategy: `_validate_category_fields`'s if/elif chain becomes a dispatch table keyed by category string |
| `artifact_trace_writer.py` (~40 lines) | `compute_content_hash`, `write_raw_trace`, `verify_trace_ref_roundtrip` | formalizes a boundary the module's own docstrings already name 3 times ("a writer-module concern instead") |
| `artifact_serialization.py` (~55 lines) | `artifact_to_dict`, `artifact_from_dict` | Data Mapper |
| `artifact_examples.py` (~80 lines) | `_example_provenance`, `_example_trace_ref`, `generate_example_artifacts` | fixture/example generation, not core schema |

`sandbox_budget.py` (540 lines) →

| New file | Contents | Pattern |
|---|---|---|
| `sandbox_budget.py` (core, ~200 lines) | imports, constants, `run_budgeted` | Facade composing the modules below |
| `sandbox_resource_limits.py` (~70 lines) | `_resource_limits_available`, `_compose_preexec` | RLIMIT/Darwin-detection concern isolated from orchestration |
| `sandbox_session_budget.py` (~60 lines) | `SessionBudget` dataclass | pure accounting, no process I/O |
| `sandbox_process_io.py` (~150 lines) | `_kill_process_group`, `_verify_teardown`, `_close_process_pipes`, `_read_capped` | process supervision/teardown/incremental-read concern |

### Happy paths considered

- **HP-1:** every existing assertion in `artifact_schema_test.py` and
  `sandbox_budget_test.py` passes **unmodified** against the decomposed
  modules — proving zero behavior change from the split alone.
- **HP-2:** every file in `scripts/antares/` measures under 500 lines via
  `wc -l` after the split (verifies the gate this task exists to satisfy).

### Edge cases considered

- **EC-1:** `sandbox_budget_test.py` calls
  `unittest.mock.patch.object(_MODULE, "_resource_limits_available", ...)`,
  `unittest.mock.patch.object(_MODULE, "_compose_preexec", ...)`, and
  `unittest.mock.patch.object(_MODULE, "_verify_teardown", return_value=False)`
  (lines 86, 90, 279) plus direct `_MODULE._kill_process_group(...)` (line
  308) and `_MODULE._resource_limits_available = lambda: False` (line 336)
  calls, where `_MODULE` is `sandbox_budget.py` loaded by
  `importlib.util.spec_from_file_location`. Python resolves a bare name inside
  a function against its *defining module's* globals at call time, so
  `run_budgeted` must keep calling these as **bare names** after `from
  sandbox_process_io import _verify_teardown, _kill_process_group,
  _close_process_pipes, _read_capped` / `from sandbox_resource_limits import
  _compose_preexec, _resource_limits_available` re-export-style imports into
  `sandbox_budget.py`'s own namespace — not as
  `sandbox_process_io._verify_teardown(...)` qualified access. The latter
  would make `patch.object(_MODULE, "_verify_teardown", ...)` silently patch
  an attribute `run_budgeted` never reads, defeating
  `test_teardown_unconfirmed_surfaces_as_its_own_kind` (the existing test
  guarding `SANDBOX_TEARDOWN_UNCONFIRMED`) without a loud failure in every
  case. **This requirement is not limited to the four names above** — it
  applies to *every* name `run_budgeted` calls that moves into a new
  submodule, including `_close_process_pipes` and `_read_capped` (both move
  to `sandbox_process_io.py` per the table above but are not directly mocked
  today, only called; a qualified-access regression on either would not fail
  loudly, it would just silently stop being patchable in a future test). One
  existing dependency needs no change at all: `_drop_privileges` (line 54,
  `_drop_privileges = _SANDBOX_RUNNER_MOD._drop_privileges`, a T2c-1
  re-export) is accessed directly as `_MODULE._drop_privileges()` by the test
  (line 64) and by `run_budgeted` itself — it stays in the core
  `sandbox_budget.py` file under this decomposition (it is not one of the
  functions moved to any of the three new `sandbox_*` files), so the same
  bare-name guarantee holds trivially and requires no re-export work.
- **EC-2:** `artifact_schema_test.py` and `sandbox_budget_test.py` both pull
  every public symbol off the loaded module object (e.g. `_MOD.artifact_from_dict`,
  `_MODULE.SessionBudget`) rather than importing named symbols directly —
  confirmed via grep, zero `mock.patch` calls in `artifact_schema_test.py` and
  the four `_MODULE`-attribute patches above in `sandbox_budget_test.py`. Every
  name currently resolvable on the top-level module object must remain
  resolvable there after the split (via re-export imports at the top of each
  core file), or the existing test files would need modification — which
  would violate HP-1.
- **EC-3:** no other module in `scripts/antares/` imports from
  `artifact_schema.py` or `sandbox_budget.py` besides their own test files
  (confirmed via repo-wide grep) — so this task's blast radius is contained to
  the two modules and their two test files; a future finding of a third
  consumer during implementation would be a genuine scope surprise requiring
  re-verification of this claim, not a silent assumption to keep carrying
  forward.
- **EC-4:** the 5-way `artifact_schema.py` split introduces cross-file
  dependencies that did not exist before (e.g. `artifact_serialization.py`
  needs the dataclasses defined in core `artifact_schema.py`;
  `artifact_validators.py` needs constants/helpers from core). The
  implementer must verify no circular import exists among the 5 new
  `artifact_*` files or the 4 new `sandbox_*` files before closing the task —
  a cycle is a silent blocker that will not show up in the `wc -l` gate and
  must be caught by actually importing each module, not just by inspection.

### Acceptance criteria

- `artifact_schema.py` and `sandbox_budget.py`, and every new file each is
  split into, measure under 500 lines.
- Every existing test in `artifact_schema_test.py` and `sandbox_budget_test.py`
  passes with **zero modifications** to either test file's assertions (import
  wiring inside the test file's own `importlib` bootstrap may be touched only
  if a new file must be loaded the same way; the assertions themselves must
  not change). Both test files must keep loading their target via
  `importlib.util.spec_from_file_location` pointed at `sandbox_budget.py` /
  `artifact_schema.py` specifically (the core file each test already names) —
  not a new `__init__.py` or package entrypoint — so every symbol remains
  reachable as `_MODULE.<name>` exactly as today.
- No behavior change: `make qa-test` (or the equivalent scoped `cargo`/`python`
  unittest invocation for `scripts/antares/`) is green before and after, and a
  diff of `wc -l` and public-symbol names on the top-level module objects
  shows the same public surface, just relocated.
- No circular imports among the new files (EC-4): each new module imports
  cleanly on its own, verified by executing every existing test file, not by
  inspection alone.

### Evidence to emit

- The 9 files listed in the decomposition table above.
- A short before/after `wc -l` table per file in the closure record.
- The full test output (pass/fail count, not just exit code) for
  `artifact_schema_test.py` and `sandbox_budget_test.py` after the split, as
  direct evidence for HP-1 and EC-1/EC-2, not only the `wc -l` proof for HP-2.

### Status artifacts affected

- this ledger (this section plus T2e's Delegation-routing note, updated to
  point at this task's outcome instead of leaving the choice open)
- `docs/plan/antares-security-specialist-advisor.md` if T2e's dependency
  changes materially affect the phase narrative

### Implementation note (git state)

The repository's git config unexpectedly showed `core.bare=true` plus a
fresh `AUTO_MERGE`/`COMMIT_EDITMSG` (unrelated to this task, not created by
this session) partway through implementation, making `git status`/`git diff`
fail. Per explicit owner instruction, this was treated as out-of-band and
ignored for the remainder of this task: implementation, review, and
verification below use only file reads/writes and `python3`/`pytest`
commands, no `git` commands. The Phase-2 review packet's description of the
change was therefore built manually from the actual old/new file contents
already read during implementation, not from `git diff`.

### Reflection log

Required passes: 3 (`52` → Med-high)

#### Pass 1

- **Draft verdict:** initial 9-file split written per the decomposition
  table (Strategy dispatch table in `artifact_validators.py`, Data Mapper in
  `artifact_serialization.py`, Facade in `sandbox_budget.py`), each file
  using the pre-existing `importlib.util.spec_from_file_location` loading
  convention for cross-file dependencies.
- **Critique findings:**
  - A correctness risk was identified and empirically proven *before*
    writing any production code: independently re-loading the same source
    file via `spec_from_file_location`/`module_from_spec`/`exec_module`
    produces distinct, non-`==`-comparable Enum class objects (confirmed
    with a 6-line standalone repro). Splitting `SessionBudget` (which
    constructs `TerminalState(kind=TerminalStateKind.X, ...)` values that
    flow back through `run_budgeted` to the test file) into its own file
    risked exactly this: two different `TerminalStateKind` copies that
    print identically but fail `==`.
  - After adding a "check `sys.modules` before re-loading" guard to every
    new file, two concrete test failures still occurred
    (`test_ec1_sixteenth_command_is_refused_before_starting`,
    `test_ec1_wall_budget_already_exhausted_is_refused_before_starting`) —
    root-caused to `sandbox_runner.py` (existing, unmodified, out of this
    task's scope) unconditionally re-loading `terminal_state.py` without
    any `sys.modules` check, clobbering the shared registry entry between
    core's own load and `sandbox_session_budget.py`'s load.
- **Revisions applied:** added the check-first `_load_sibling_module`
  helper to every new/rewritten file (also breaks the core↔validators
  circular-load path for the `artifact_schema.py` family); reordered core
  `sandbox_budget.py`'s load sequence so `sandbox_session_budget.py` (and
  transitively `terminal_state.py`) resolves before anything loads
  `sandbox_runner.py`. Re-verified empirically (`sb.TerminalStateKind is
  ssb.TerminalStateKind` → `True`) and via the full suite (111 passed).

#### Pass 2

- **Draft verdict:** full suite green (111/111) after the Pass-1 fix;
  re-read every new file line-by-line as an independent reviewer.
- **Critique findings:** `artifact_validators.py::_validate_disposition`
  looked up `_ARTIFACT_SCHEMA_MOD.DispositionState` freshly on every call
  instead of once at module level, inconsistent with every other
  cross-module reference in the same file and a needless per-call
  attribute lookup.
- **Revisions applied:** moved `DispositionState =
  _ARTIFACT_SCHEMA_MOD.DispositionState` to module level alongside the
  other re-exports; removed the in-function lookup. Full suite re-confirmed
  green (111/111).

#### Pass 3

- **Draft verdict:** ran the Phase-2 code-solution review
  (`qwen3.6:27b-q4_K_M`) against the Pass-2 state; re-ran after applying its
  accepted findings.
- **Critique findings (from Phase-2 review, independently verified before
  acceptance):**
  - MAJOR (accepted): the load-order dependency relied on a comment, not a
    structural guarantee, and could silently regress if reordered later.
  - MAJOR (accepted, mitigated — full fix out of scope): mixing the new
    check-first loader with legacy always-re-execute loaders
    (`sandbox_runner.py`, out of scope) makes identity depend on load
    history in general; the *specific* risk is closed by the fix below.
  - MINOR (verified, no change needed): confirmed via direct read that
    `terminal_state.py`'s only top-level statements are two
    `frozenset(...)` definitions — no side effects, safe to re-execute.
  - MINOR (accepted, out of scope): dynamic `importlib` loading is the
    pre-existing convention for every file in `scripts/antares/`, not
    something this task introduced; not fixed here.
  - New finding on the fix itself (accepted): the identity checks initially
    used bare `assert`, stripped under `python -O`/`PYTHONOPTIMIZE`.
- **Revisions applied:** added explicit `if TerminalState is not
  _SANDBOX_SESSION_BUDGET_MOD.TerminalState: raise RuntimeError(...)` guards
  (and the `TerminalStateKind` equivalent) in `sandbox_budget.py`, mirroring
  the existing `T2A_KINDS`-style self-check-assert precedent already present
  in `artifact_schema.py`, but using `if/raise` instead of bare `assert` so
  the check cannot be optimized away. Verified the guard actually fires
  under a deliberately-reordered throwaway copy, both with and without
  `python -O` (confirmed no repo toolchain currently sets `-O`/
  `PYTHONOPTIMIZE`, via grep of `Makefile`/`.github/workflows/*.yml`). Added
  `t2e_pre_decomposition_test.py` for HP-2's line-count claim (was
  previously evidenced only by a `wc -l` command, not a unit test). Full
  suite green (112/112, including the new test).

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: Ollama `/api/chat`, `"think": false`
- Artifact: `docs/audit/gemma-evidence/antares-t2e-pre-phase2.json`
- Verdict: `PASS` (pass 1 `FINDINGS` 2 MAJOR/2 MINOR → pass 2 `FINDINGS` 1
  new MAJOR-as-raised, all 4 original findings resolved/accepted → pass 3
  primary-agent-certified `PASS` after independently reproducing the fix
  under `python -O`)
- Findings: see Reflection log above; every finding accepted, mitigated, or
  independently verified as a non-issue; no finding rejected as a false
  positive in this review.
- Gemma fallback: not triggered. D14 fallback: not triggered.
- disposition_divergence: `none`.

```
Code-solution review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2e-pre-phase2.json - PASS
```

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | every existing `artifact_schema_test.py`/`sandbox_budget_test.py` assertion passes unmodified against the decomposed modules | `scripts/antares/artifact_schema_test.py` (28 tests) + `scripts/antares/sandbox_budget_test.py` (19 tests), full run: `python3 -m pytest scripts/antares/artifact_schema_test.py scripts/antares/sandbox_budget_test.py -v` | passed (47/47) |
| HP-2 | Happy path | every file in `scripts/antares/` measures under 500 lines after the split | `scripts/antares/t2e_pre_decomposition_test.py::DecomposedFileSizeTest::test_hp2_every_decomposed_file_stays_under_500_lines` | passed |
| EC-1 | Edge case | `run_budgeted` calls every function moved into a new submodule (not just the 4 originally-mocked ones) as a bare name resolved from `sandbox_budget`'s own namespace, so `mock.patch.object`/direct attribute reassignment on `_MODULE` still take effect | `scripts/antares/sandbox_budget_test.py::RunBudgetedRuntimeUnavailableTest::test_ec1_resource_limits_unavailable_fails_closed`, `::RunBudgetedTeardownTest::test_ec3_unconfirmed_teardown_is_its_own_distinct_outcome`, `::KillProcessGroupTest::test_fallback_kill_swallows_already_exited_race_instead_of_raising` | passed |
| EC-2 | Edge case | every symbol previously resolvable on the top-level module object (`_MOD.<name>`/`_MODULE.<name>`) remains resolvable there after the split, via re-export | full suite (every test in both `_test.py` files reads at least one `_MOD.<name>`/`_MODULE.<name>` attribute) — `python3 -m pytest scripts/antares/artifact_schema_test.py scripts/antares/sandbox_budget_test.py -v` | passed (47/47) |
| EC-3 | Edge case | no third-party consumer outside the two test files depends on `artifact_schema.py`/`sandbox_budget.py` | repo-wide grep (`grep -rln "artifact_schema\.py\|sandbox_budget\.py" --include="*.py" scripts/`) confirms only the split's own files and their test files; the one substring hit outside `scripts/antares/` (`scripts/gemma_push_review_test.py::test_failure_artifact_schema`/`test_blocked_artifact_schema`) is a test-method-name false positive, unrelated to the actual file | confirmed, no unit test applicable (structural/grep-based claim) |
| EC-4 | Edge case | no circular imports among the 9 new files; the multi-copy `TerminalStateKind`/`TerminalState` identity risk from splitting a shared-Enum-consuming concern across sibling files is closed | `scripts/antares/sandbox_budget.py`'s own `if TerminalState is not ... raise RuntimeError(...)` guard (exercised implicitly by every `sandbox_budget_test.py` test that reaches `run_budgeted`'s `SANDBOX_BUDGET_EXHAUSTED`/`SANDBOX_WALL_BUDGET_EXCEEDED` paths, e.g. `RunBudgetedCommandBudgetTest::test_ec1_sixteenth_command_is_refused_before_starting`, `RunBudgetedWallBudgetTest::test_ec1_wall_budget_already_exhausted_is_refused_before_starting`); full suite `python3 -m pytest scripts/antares/ -q` | passed (112/112) |

### Owner final verification

- Owner: `Matias Kruk`
- Date: `2026-07-30`
- Statement: reviewed the Unit coverage certification table, Reflection log,
  and Peer Reviewer artifact (`docs/audit/gemma-evidence/antares-t2e-pre-phase2.json`)
  for this task and confirmed the work is acceptable ("si esta ok").
- Commands run: `python3 -m pytest scripts/antares/ -q` — 112/112 passed.

**Status: `[x] Done`.**

## T2e - Replay fixtures and integrated harness verification

- **Status:** `[ ] Open`
- **Type:** development / security-sensitive tooling
- **Effort:** TBD from execution RRI
- **Preliminary RRI:** 44 Med-high (planning estimate, not authority — recompute
  at presentation time per `docs/policies/RRI_POLICY.md`)
- **Depends on:** T2e-pre
- **Decomposed from:** T2

### Objective

Assemble the parser, policy, sandbox, and artifact layers into one deterministic
harness surface with replay fixtures and regression tests for every approved
happy path and edge case. This is the first T2 subtask that composes all four
prior layers (`tool_call_parser.py`/`terminal_state.py`,
`command_policy.py`/`path_containment.py`, `sandbox_runner.py`/
`sandbox_budget.py`, `artifact_schema.py`) at once; it adds no new terminal-state
kinds and defines no new encoding — it proves the existing, already-tested
per-layer contracts keep their failure boundaries when wired together.

### Phase 1 — Task-analysis review

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: Ollama `/api/chat`, `"think": false`
- Artifact: `docs/audit/gemma-evidence/antares-t2e-phase1.json`
- Pass 1: `BLOCKED` (2 BLOCKING, 2 MAJOR, 1 MINOR). Two findings (the >500-line
  delegation-constraint gap and the missing composed-layer poisoned-payload
  edge case) were independently confirmed against the repo and accepted as-is.
  Three findings (deterministic-replay definition, EC-2 encoding mechanism,
  entrypoint interface type) were downgraded from BLOCKING/MAJOR to MINOR on
  independent verification — the model was not given `sandbox_runner_test.py`
  or `terminal_state.py` in pass 1, and both files already establish the
  conventions the findings claimed were missing.
- Pass 2: `BLOCKED`. All 5 pass-1 findings independently reconfirmed resolved
  against the revised text. 2 new findings raised: (6) a phrasing issue that
  conflated the standard ADR-038 routing step with a new code-acceptance
  criterion — accepted and reworded; (7) a claim that
  `submit_vulnerable_files`/`submit_no_vulnerability_found` were undefined —
  **rejected as a false positive**: `terminal_state.py` lines 28-29 define
  both as `TerminalStateKind` members from T2a, already validated end-to-end
  by T2d's `test_hp1_vulnerable_files_carries_trace_ref_and_no_raw_content` /
  `test_hp2_no_vulnerability_found_matches_positive_result_shape`; the pass-2
  prompt omitted the enum body, which is the likely cause. Closed with an
  inline citation in the task text rather than a code change (none was
  needed).
- Pass 3: `PASS`. Both pass-2 findings independently reconfirmed resolved, no
  new findings.
- Gemma fallback: not triggered. D14 fallback: not triggered.
- disposition_divergence: `none` (every finding's disposition here matches the
  final accepted/rejected state recorded in the artifact).

```
Task-analysis review: qwen3.6:27b-q4_K_M docs/audit/gemma-evidence/antares-t2e-phase1.json - PASS
```

### Delegation-routing note (resolved)

`artifact_schema.py` (574 lines) and `sandbox_budget.py` (540 lines) already
exceeded the 500-line target-file-size gate for RRI 26-55 local-first
delegation (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Target-file size gate).
**Resolved by owner directive, 2026-07-30: option (b), decompose-first via
design patterns** rather than escalate directly to cloud — see **T2e-pre**,
now a hard dependency of this task. T2e's implementer reads the
decomposed, sub-500-line modules T2e-pre produces, not the original
oversized files.

### Happy paths considered

- **HP-1:** a fully valid packet replays deterministically to
  `submit_vulnerable_files` with canonical validated candidates and a complete
  trace reference.
- **HP-2:** a fully valid packet replays deterministically to
  `submit_no_vulnerability_found` with no ambiguity in result semantics.

### Edge cases considered

- **EC-1:** command-budget exhaustion preserves the partial trace and records
  `budget_exhausted` (concrete target: `sandbox_budget.py`'s
  `DEFAULT_COMMAND_BUDGET = 15` check against `_commands_started`).
- **EC-2:** parser, policy, sandbox, and artifact failures remain distinct in
  the integrated harness output, encoded as the already-defined, already
  fully-partitioned `TerminalStateKind` enum (`terminal_state.py`) — T2e proves
  each layer's real failure is wired through to its own existing kind
  end-to-end; it does not define a new distinction mechanism.
- **EC-3:** sandbox-escape regression fixtures fail closed across replays.
- **EC-4:** a structurally valid but semantically poisoned payload from an
  upstream layer (e.g., a policy-approved command whose execution triggers
  pathological subprocess behavior) still resolves to a bounded
  budget/timeout-enforced terminal state through the composed harness, never
  an unbounded hang — proving T2c-2's wall-timeout/budget guarantee holds when
  triggered by composed-layer input, not only by `sandbox_runner_test.py`'s
  direct unit fixtures.

### Acceptance criteria

- Deterministic replay exists for every T2 HP/EC behavior carried into the
  decomposed subtasks. "Deterministic" follows the convention already
  established in `sandbox_runner_test.py`: structural/semantic fields (`kind`,
  `exit_code`, `candidates`, `provenance`, `trace_ref`, disposition shape) are
  byte-identical across replays of the same fixture input; wall-clock-derived
  fields (`elapsed_seconds`, timestamps) are bounded/range-checked, not
  byte-compared.
- The integrated harness emits only versioned artifact output and externalized
  redacted trace references.
- Regression tests prove that parser, policy, containment, sandbox, and artifact
  layers keep their failure boundaries when composed.
- `submit_vulnerable_files` and `submit_no_vulnerability_found` are not new
  targets this task invents: both are existing `TerminalStateKind` members
  (`terminal_state.py` lines 28-29,
  `SUBMITTED_VULNERABLE_FILES`/`SUBMITTED_NO_VULNERABILITY_FOUND`) already
  produced by the T2a parser and already validated end-to-end by T2d's
  `test_hp1_vulnerable_files_carries_trace_ref_and_no_raw_content` /
  `test_hp2_no_vulnerability_found_matches_positive_result_shape`. HP-1/HP-2
  here replay the same two outcomes through the composed harness, not through
  a single layer in isolation.

The delegation-routing note above (escalate vs. decompose-first for the
>500-line modules) is a **routing prerequisite resolved through the standard
ADR-038 gate this task already goes through as a Med-high task** — it is not
an additional code-acceptance criterion and not new administrative scope.

### Evidence to emit

- Integrated harness entrypoint — a plain importable Python module (e.g.
  `harness.py` with a replay-driving function), consistent with every T2a-T2d
  module's convention of no CLI wrapper — plus a replay corpus and regression
  tests for all T2 happy-path and edge-case behaviors.

### Status artifacts affected

- this ledger and the slice plan

## T3 - CWE watchlist and context-complete packet construction

- **Status:** `[ ] Open`
- **Type:** development / security policy
- **Effort:** TBD from execution RRI
- **Depends on:** T2e

### Objective

Build deterministic packets from an externally justified CWE hypothesis while
retaining the repository context required to localize beyond changed files.

### Happy paths considered

- **HP-1:** a refinement or review packet includes the supplied CWE, generic
  description, baseline/candidate snapshot identity, changed paths, and a
  deterministic dependency/context closure.
- **HP-2:** a post-CI watchlist entry produces a bounded packet whose included and
  omitted paths are explicit and reproducible.

### Edge cases considered

- **EC-1:** no justified CWE is supplied, so the touchpoint records `skipped` with
  a reason instead of asking Antares to invent one.
- **EC-2:** resolved context exceeds the configured size budget, so construction
  fails closed or uses a documented deterministic partition; it never silently
  drops dependencies.
- **EC-3:** credentials, `.env` files, `config/production.toml`, generated output,
  or paths resolving outside the snapshot are excluded and reported.

### Acceptance criteria

- CWE sources are limited to a human/primary-advisor hypothesis, a justified
  repository watchlist, or a mapped advisory; Antares never infers its own input.
- Every watchlist entry names its repository boundary and owner; CWE-732 remains
  excluded from the initial watchlist because it is a documented weak class.
- Changed-path mode adds deterministic import/dependency, manifest, and governing
  security-boundary context; changed files alone are not treated as sufficient.
- Scope, omissions, partitioning, and hashes are stable and fixture-testable.
- Packets above the model's practical size limit do not run as normal successes.

### Evidence to emit

- Versioned watchlist with owner and per-entry justification.
- Packet schema, fixtures for each touchpoint, and context-closure tests.

### Status artifacts affected

- this ledger and the slice plan

## T4 - Ground-truth calibration and observe-only workflow pilot

- **Status:** `[ ] Open`
- **Type:** development / CI / evaluation
- **Effort:** TBD from execution RRI
- **Depends on:** T2e, T3

### Objective

Measure localization quality against known truth, then measure operational value
at refinement, post-implementation, and post-CI touchpoints without creating a
new gate.

### Happy paths considered

- **HP-1:** pre-fix snapshots with patch-derived vulnerable-file labels produce
  task-level precision/recall/File F1, while paired patched snapshots produce a
  true-negative metric.
- **HP-2:** an eligible task with an explicit CWE invokes Antares over the existing
  baseline during refinement and over the candidate snapshot after implementation;
  the primary security advisor dispositions the candidates without delaying
  approval, reviewer-of-record verdict, CI truth, or closure.
- **HP-3:** post-CI runs emit redacted summaries and operational metrics while raw
  traces remain uncommitted and retention-bounded.

### Edge cases considered

- **EC-1:** runtime failure writes a degraded artifact and leaves primary CI and
  workflow state unchanged.
- **EC-2:** no eligible CWE records a skip; a forced generic sweep is forbidden.
- **EC-3:** undisposed candidates exceed the SLA and are reported as backlog to the
  named owner; they are not silently closed.

### Acceptance criteria

- Fix the calibration corpus, pilot window/sample, watchlist schedule, concurrency,
  runtime budget, stopping rules, and promotion thresholds before execution.
- Calibration uses known vulnerable snapshots and patch-derived implementation-file
  ground truth; patched snapshots are evaluated separately for true negatives.
- Report macro-averaged task metrics and uncertainty; do not infer per-output
  correctness from aggregate File F1.
- Operational metrics include volume, dispositions, SLA age, deduplication rate,
  triage time, accepted-follow-up conversion, runtime, and resource cost.
- `rejected` means a human disposition, not a false positive; only adjudicated
  ground truth may support false-positive/precision claims.
- Define triage owner, queue/ledger, SLA, deduplication key, retention/redaction,
  and the link from `accepted-follow-up` to a task/refinement record.
- `logs/antares/` is ignored by Git; only redacted summaries are commit-eligible,
  and CI artifacts use an explicit retention period.
- Antares remains non-blocking and cannot satisfy or replace either review phase.

### Evidence to emit

- Calibration report and machine-readable results.
- Pilot report, run artifacts, disposition ledger, and advisory-only replay fixture.

### Status artifacts affected

- this ledger and the slice plan
- `.github/workflows/push-review.yml` and `.gitignore` if the post-CI lane attaches
- the named calibration/pilot reports and finding ledger

## T5 - Promote, narrow, or retire on evidence

- **Status:** `[ ] Open`
- **Type:** docs / workflow decision
- **Effort:** TBD from execution RRI
- **Depends on:** T4

### Objective

Make an explicit operating decision from calibration and pilot evidence.

### Acceptance criteria

- The decision cites repository calibration and operational evidence separately.
- Retirement is a genuine outcome; vendor benchmark results cannot override failed
  local feasibility, quality, or triage-value thresholds.
- If retained, the decision fixes eligible touchpoints, CWE sources, schedule,
  owner, SLA, retention, and resource budget.
- The authority boundary remains advisory-only; Antares never becomes RRI authority,
  reviewer of record, autonomous remediator, or closure gate.

### Evidence to emit

- Final promote/narrow/retire decision with threshold-by-threshold rationale.

### Status artifacts affected

- this ledger and the slice plan
- workflow/policy docs and roadmap only if the lane is retained

## Cancelled capability assumptions

Preserved for auditability. These capabilities remain cancelled even though the
localizer may participate as a bounded sub-tool in refinement and review workflows.

### Cancelled - Antares as the task-refinement security specialist

Antares cannot create the threat model, choose a justified CWE, explain security
rationale, or recommend tests. The primary agent or human specialist owns those
steps; Antares may only localize files in the existing baseline after a CWE exists.

### Cancelled - Antares as the post-implementation reviewer of record

Antares output is triage evidence, not a review verdict. It may run over the
candidate snapshot, but cannot pass/block review, replace the band-routed reviewer,
or delay closure solely because it is unavailable.

### Cancelled - Antares to canonical RRI reconciliation contract

Antares does not propose RRI inputs; it returns ranked file paths. The primary
agent may independently use verified repository facts in normal RRI analysis, but
there is no Antares-specific RRI channel and `docs/policies/RRI_POLICY.md` remains
unchanged.
