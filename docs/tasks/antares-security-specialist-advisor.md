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
T0 (done) -> T0a -> T1 -> T2a -> T2b -> T2c -> T2d -> T2e -> T3 -> T4 -> T5
```

## Task summary

| Task | Status | Preliminary RRI | Effort | Depends on |
|---|---|---:|---|---|
| T0 Define role charter and authority boundary | `[x] Done` | 31 Moderate (execution) | M | - |
| T0a Correct charter and close design gaps | `[x] Done` | 47 Med-high (execution) | L | T0 |
| T1 Runtime and model-access preflight | `[x] Done (owner-waived)` | 49 Med-high (technical preflight blocked) | L | T0a |
| T2 Sandboxed agentic harness and artifact schema | `[~] Decomposed (2026-07-29)` | 86 Very high (pre-execution) | XL | T1 |
| T2a Tool-call parser and terminal-state contract | `[x] Done (2026-07-29)` | 45 Med-high (execution) | L | T1 |
| T2b Command allowlist and canonical path containment | `[ ] Open` | Recompute | TBD | T2a |
| T2c Ephemeral sandbox runner and resource enforcement | `[ ] Open` | Recompute | TBD | T2b |
| T2d Versioned artifact schema and redacted trace contract | `[ ] Open` | Recompute | TBD | T2c |
| T2e Replay fixtures and integrated harness verification | `[ ] Open` | Recompute | TBD | T2d |
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

- **Status:** `[ ] Open`
- **Type:** development / security-sensitive tooling
- **Effort:** TBD from execution RRI
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

## T2c - Ephemeral sandbox runner and resource enforcement

- **Status:** `[ ] Open`
- **Type:** development / security-sensitive tooling
- **Effort:** TBD from execution RRI
- **Depends on:** T2b
- **Decomposed from:** T2

### Objective

Run already-validated argv commands in an ephemeral, default-deny sandbox with
network isolation, read-only mounts, dropped privileges, bounded resources, and
reliable teardown.

### Happy paths considered

- **HP-1:** a validated read-only command completes inside the sandbox and returns
  bounded stdout/stderr plus measured timing.
- **HP-2:** a multi-command run stops cleanly after a successful terminal
  submission within the 15-call budget.

### Edge cases considered

- **EC-1:** per-command timeout, wall-time exhaustion, or output-limit breach
  produces a distinct degraded terminal state.
- **EC-2:** runtime unavailability or sandbox bootstrap failure produces
  `runtime_unavailable` without falling back to unsandboxed execution.
- **EC-3:** teardown runs even after timeout or sandbox violation.

### Acceptance criteria

- The runner uses network-disabled, credential-stripped, read-only execution with
  dropped privileges and teardown after each run.
- Per-command timeout, wall timeout, CPU/RAM/PID/output caps, and the 15-command
  budget are enforced deterministically.
- No success path exists outside the sandbox boundary.
- Measured timing/resource fields needed by later artifacts are emitted.

### Evidence to emit

- Sandbox runner module, isolation fixtures, timeout/output-limit tests, and
- teardown verification evidence.

### Status artifacts affected

- this ledger and the slice plan

## T2d - Versioned artifact schema and redacted trace contract

- **Status:** `[ ] Open`
- **Type:** development / security-sensitive tooling
- **Effort:** TBD from execution RRI
- **Depends on:** T2c
- **Decomposed from:** T2

### Objective

Normalize every success, negative, degraded, refused, and skipped result into a
versioned artifact with provenance, terminal-state semantics, and a redacted
trace-reference contract.

### Happy paths considered

- **HP-1:** `vulnerable_files` records validated candidate paths, provenance, and
  an external trace reference without leaking source excerpts into committed docs.
- **HP-2:** `no_vulnerability_found` records an explicit negative result with the
  same provenance and audit fields as a positive result.

### Edge cases considered

- **EC-1:** degraded states such as `budget_exhausted`, `command_timeout`, or
  `output_limit` remain durable and distinguishable.
- **EC-2:** raw traces remain outside committed paths and the artifact records the
  redaction/retention contract instead of embedding sensitive content.
- **EC-3:** undisposed findings remain operationally open instead of appearing
  closed by omission.

### Acceptance criteria

- The schema is versioned and covers every terminal state named by the plan.
- Provenance includes model/runtime/harness versions plus packet/snapshot hashes.
- Human disposition fields are mandatory in the durable contract.
- Committed summaries exclude prompts, source excerpts, credentials, and secrets.

### Evidence to emit

- Schema module, validation fixtures, and one redacted example per terminal state.

### Status artifacts affected

- this ledger and the slice plan

## T2e - Replay fixtures and integrated harness verification

- **Status:** `[ ] Open`
- **Type:** development / security-sensitive tooling
- **Effort:** TBD from execution RRI
- **Depends on:** T2d
- **Decomposed from:** T2

### Objective

Assemble the parser, policy, sandbox, and artifact layers into one deterministic
harness surface with replay fixtures and regression tests for every approved
happy path and edge case.

### Happy paths considered

- **HP-1:** a fully valid packet replays deterministically to
  `submit_vulnerable_files` with canonical validated candidates and a complete
  trace reference.
- **HP-2:** a fully valid packet replays deterministically to
  `submit_no_vulnerability_found` with no ambiguity in result semantics.

### Edge cases considered

- **EC-1:** command-budget exhaustion preserves the partial trace and records
  `budget_exhausted`.
- **EC-2:** parser, policy, sandbox, and artifact failures remain distinct in the
  integrated harness output.
- **EC-3:** sandbox-escape regression fixtures fail closed across replays.

### Acceptance criteria

- Deterministic replay exists for every T2 HP/EC behavior carried into the
  decomposed subtasks.
- The integrated harness emits only versioned artifact output and externalized
  redacted trace references.
- Regression tests prove that parser, policy, containment, sandbox, and artifact
  layers keep their failure boundaries when composed.

### Evidence to emit

- Integrated harness entrypoint, replay corpus, and regression tests for all T2
  happy-path and edge-case behaviors.

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
