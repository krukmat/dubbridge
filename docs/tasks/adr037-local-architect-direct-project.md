---
type: TaskList
title: "Tasks: ADR-037 Local Architect / Complex Analyst direct project use"
plan: docs/plan/adr037-local-architect-direct-project.md
status: active
slice: adr037-local-architect-direct-project
governed_by: [ADR-037, ADR-036, ADR-034]
---

# Tasks: ADR-037 Local Architect / Complex Analyst Direct Project Use

## Objective

Use `qwen3.6:27b-q4_K_M` directly on real DubBridge planning work as a read-only
Local Architect / Complex Analyst, while preserving the existing implementer,
reviewer, RRI, and human-approval chain.

## Creation-task RRI and review exemption

The documentation package that records ADR-037 direct-project mode, this plan, this
ledger, and the ADR index scored `RRI 20` (`Low`, `Effort S`) with
`arch_decision +12`. It is ADR/plan/task-ledger-only work.

- `Task-analysis review: n/a - ADR/plan/task-ledger-only exemption`
- `Code-solution review: n/a - ADR/plan/task-ledger-only exemption`

The RRI values below are preliminary task-card scores produced with
`scripts/rri.py`. Recompute every score immediately before presenting or executing
the task. A recomputed score controls the final Effort, reviewer route, thinking
mode, and approval gate.

## Medium-agent contract

"Medium" in this ledger maps to the canonical `RRI 26-40 Moderate / Effort M /
Balanced capability` band. It is an execution profile, not an authority level.

Medium agents:

- run one bounded task after the workflow-required approval;
- receive fixed inputs, allowed paths, acceptance criteria, and evidence outputs;
- stop rather than widening scope or silently substituting a model/work item;
- may report measurements and recommendations but may not accept architecture,
  approve work, review code officially, or start downstream implementation;
- use normal Moderate-band phase-1 and phase-2 review routes when a task is a
  development task. Operational, evaluation, ADR, plan, and task-ledger exemptions
  are recorded per task and do not confer authority.

## Task order

```text
T0 accepted
  ├─► T1 Resolve/install/fingerprint model
  ├─► T2 Build one-shot wrapper
  └─► T3 Select real work item and freeze packet

T1 + T2 + T3 ─► T4 Run direct Local Architect analysis
T4 ─► T5 Primary verifies and authors target ADR/plan/tasks
T5 + first target milestone ─► T6 Trace downstream outcome and local-stack utility
```

## Task summary

| Task | Status | Preliminary RRI | Effort | Agent grade | Depends on |
|---|---|---:|---|---|---|
| T0 Accept ADR-037 direct-project mode | `[x] Done 2026-07-19` | 20 Low | S | primary + human | - |
| T1 Resolve, install, and fingerprint model | `[x] Done 2026-07-19` | 27 Moderate | M | Medium ops | T0 |
| T2 Build one-shot tool-free wrapper | `[x] Done 2026-07-20` | 27 Moderate | M | Medium developer | T0 |
| T3 Select first real work item and freeze packet | `[x] Done 2026-07-20` | 35 Moderate | M | Medium analyst | T0 |
| T4 Run direct Local Architect analysis | `[x] Done 2026-07-20` | 36 Moderate | M | Medium operator/analyst | T1, T2, T3 |
| T5 Verify and author actual project decision/plan/tasks | `[ ] Ready 2026-07-20` | 50 Med-high | L | primary + peer/human | T4 |
| T6 Record downstream outcome and stack utility | `[ ] Blocked` | 34 Moderate | M | Medium evaluator | T5 + first target milestone |

T0 records the owner's decision to use the role directly on project work. It does
not authorize model download, wrapper implementation, model execution, or target
work-item implementation.

---

## T0 - Accept ADR-037 direct-project mode

- **Status:** `[x] Done 2026-07-19`
- **Effort:** S
- **RRI:** 20 Low; `C0 F2 D0 T0 A0 K0 P0 X3 + arch_decision`
- **Agent grade:** primary + human
- **Depends on:** none
- **Allowed paths:** ADR-037, ADR index, direct-use plan, direct-use ledger
- **Review:** phase 1/phase 2 `n/a` - ADR/plan/task-ledger-only

### Objective

Record that ADR-037 is accepted as a direct-project consultative role, not an
offline pilot or shadow evaluation.

### Acceptance criteria

- Owner decision recorded as direct project use.
- ADR-037 frontmatter/prose and ADR index read `Accepted`.
- Plan and ledger reference direct-project use, not pilot promotion.
- No model is downloaded, installed, loaded, or executed during T0.

### Evidence to emit

- Owner instruction date: 2026-07-19.
- Documentation QA command/results.

### Status artifacts affected

- ADR-037, `docs/adr/README.md`, this plan, this ledger.

### Handoff

T0 is complete. Stop before T1; model installation requires a separate task
presentation and explicit approval.

---

## T1 - Resolve, install, and fingerprint the exact model binding

- **Status:** `[x] Done 2026-07-19`
- **Effort:** M
- **Preliminary RRI:** 27 Moderate; `C0 F0 D3 T2 A0 K3 P1 X2`
- **Agent grade:** Medium ops
- **Depends on:** T0 done
- **Allowed surfaces:** local Ollama registry;
  `docs/evaluations/adr037-direct-project-report.md`; this task entry
- **Task-analysis review:** `gemma`
  `.agent/peer-task-review-t1.json` - `PASS`
- **Code-solution review:** `n/a` - operational/evaluation task; no code change

### Objective

Resolve and, only after explicit task approval, pull `qwen3.6:27b-q4_K_M`. Record
identity and a smoke result without substituting another installed model.

### Steps for a Medium agent

1. Recompute RRI and present the task; wait for explicit approval.
2. Capture `ollama list` and `/api/tags` before mutation.
3. Resolve the exact requested tag. If unavailable, stop `BLOCKED` and record
   registry/error evidence.
4. Pull only the approved tag; record tag, digest, quantization, size, backend, and
   timestamp.
5. Unload every other large model, then run one bounded smoke prompt.
6. Unload Qwen3.6-27B and verify `/api/ps` no longer shows it resident.
7. Write the T1 report section and synchronize this ledger; do not start T4.

### Acceptance criteria

- Exact tag/digest recorded; no silent alias or substitute.
- Smoke response completes and carries model/runtime metadata.
- One-large-model residency is observed and unload is confirmed.
- Missing tag, pull failure, incompatible backend, or memory failure is preserved as
  `BLOCKED`, not hidden by fallback.

### Evidence to emit

- Pre/post `ollama list`, `/api/tags`, `/api/ps`, pull output, digest/quantization,
  smoke transcript, load/unload timestamps.

### Status artifacts affected

- `docs/evaluations/adr037-direct-project-report.md`, this task ledger, plan status.

---

## T2 - Build the one-shot, tool-free invocation wrapper

- **Status:** `[x] Done 2026-07-20`
- **Effort:** M
- **Preliminary RRI:** 27 Moderate; `C1 F1 D3 T1 A0 K2 P1 X2`
- **Agent grade:** Medium developer; default Moderate local implementer routing
- **Depends on:** T0 done
- **Allowed paths:** `scripts/local-architect/run_analysis.py`,
  `scripts/local-architect/run_analysis_test.py`
- **Task-analysis review:** `gemma .agent/peer-task-review-t2.json - PASS`
- **Code-solution review:** `gemma .agent/peer-code-review-t2.json - PASS`

### Objective

Implement a narrow Ollama client that reads one immutable project packet, invokes
one model without tools, validates the structured response, and atomically writes
one provenance-complete result artifact.

### Happy paths considered

- Valid packet plus installed exact model writes one structured artifact.
- Valid response labels claims and supplies every required ADR-037 section.

### Edge cases considered

- Model tag or digest mismatch stops before generation.
- Packet hash mismatch stops before generation.
- Invalid JSON/Markdown envelope, missing section, or timeout records a failed
  artifact without rewriting the packet.
- Output path already exists and overwrite is not explicitly allowed.

### Steps for a Medium agent

1. Recompute RRI and present the development task; wait for approval.
2. Implement packet loading, hash verification, prompt construction, Ollama call,
   response validation, provenance capture, and atomic write.
3. Add focused tests for success, hash mismatch, model mismatch, schema failure, and
   overwrite protection.
4. Run unit tests and required development closure review.
5. Update this ledger and report evidence; do not run the model for project analysis.

### Acceptance criteria

- Wrapper exposes no shell, filesystem-edit, git, network-browse, or review tools to
  the model.
- Result artifact records model tag/digest, packet hash, prompt version, timestamps,
  runtime parameters, and generation statistics available from Ollama.
- Failure modes produce explicit non-success records.
- Tests and required phase-2 review pass before closure.

### Evidence to emit

- Unit-test output, validation examples, command transcript, phase-1/phase-2 review
  artifacts, and wrapper usage snippet.

### Status artifacts affected

- This task ledger; T2 section of `docs/evaluations/adr037-direct-project-report.md`
  if the report exists.

### Outcome

`T2` passed on 2026-07-20. The wrapper now verifies packet hash and exact model
digest before generation, performs one bounded tool-free Ollama call, validates
the structured ADR-037 response, emits provenance-complete success/failure
artifacts, and protects existing outputs unless overwrite is explicitly enabled.
Focused unit tests passed, and the required Moderate-band phase-2 Gemma review
also passed.

---

## T3 - Select the first real work item and freeze the project packet

- **Status:** `[x] Done 2026-07-20`
- **Effort:** M
- **Preliminary RRI:** 35 Moderate; `C0 F2 D4 T2 A1 K2 P1 X4`
- **Agent grade:** Medium analyst
- **Depends on:** T0 done
- **Allowed paths:** `.agent/local-architect/adr037/<work-item-id>/`,
  `docs/evaluations/adr037-direct-project-report.md`, this task entry
- **Task-analysis review:** `n/a` - planning/evaluation-artifact exemption
- **Code-solution review:** `n/a` - planning/evaluation-artifact exemption

### Objective

Choose one real DubBridge work item, defaulting to `S-140`, and freeze a bounded
context packet for Local Architect analysis.

### Steps for a Medium agent

1. Recompute RRI and verify T0.
2. Confirm the owner did not select a different eligible work item. If no override is
   present, use `S-140`.
3. Read only the governing docs, roadmap entries, accepted ADRs, existing slice
   outputs, and product/BDD evidence materially required for that work item.
4. Define objective, non-goals, explicit questions, known constraints, current
   behavior, required behavior, and expected output schema.
5. Freeze repository revision or snapshot identifier and compute packet hash.
6. Redact sensitive content and mark missing context as `UNKNOWN`.
7. Write the packet and report section; do not invoke any model.

### Acceptance criteria

- Work item ID, objective, questions, non-goals, constraints, and included evidence
  are explicit.
- Packet is bounded, immutable, attributable, and hash-addressed.
- Missing or excluded context is recorded instead of guessed.
- Packet does not contain production secrets or live production data.

### Evidence to emit

- Packet path, repository revision/snapshot, input-manifest hash, included/excluded
  evidence list, redaction notes, and unresolved questions.

### Status artifacts affected

- `.agent/local-architect/adr037/<work-item-id>/`, report, this ledger.

### Outcome

`T3` passed on 2026-07-20. No owner override for another eligible roadmap item was
found, so the default `S-140` selection from ADR-037 was retained. The frozen packet
was written to `.agent/local-architect/adr037/S-140/packet.json` with repository
revision `e30653d59465c09e3cb2e8ef060c37b70c300bef` and SHA-256
`1e69aea975e6281e39cc55effbdd312e63d465e63e7f39d88bb6e89fcfbdb02a`. The packet
includes only bounded repository evidence, records exclusions and `UNKNOWN` gaps
explicitly, and contains no secrets or live production data.

---

## T4 - Run direct Local Architect analysis on the selected work item

- **Status:** `[x] Done 2026-07-20` (successful run after wrapper correction 4 / Option C)
- **Effort:** M
- **Preliminary RRI:** 36 Moderate; `C0 F1 D4 T3 A1 K2 P1 X4`
- **Agent grade:** Medium operator/analyst
- **Depends on:** T1, T2, T3
- **Allowed paths:** `.agent/local-architect/adr037/<work-item-id>/`,
  `docs/evaluations/adr037-direct-project-report.md`, this task entry
- **Task-analysis review:** `gemma .agent/peer-task-review-t4.json - PASS`
- **Code-solution review:** `n/a` - operational/evaluation task; no code change

### Objective

Run the exact ADR-037 model against the frozen project packet and preserve the raw
advisory artifact with runtime and provenance evidence.

### Steps for a Medium agent

1. Recompute RRI and present the task; wait for explicit approval.
2. Verify T1 model identity, T2 wrapper availability, and T3 packet hash.
3. Ensure only one large model is resident.
4. Run the wrapper once with the approved parameters.
5. Preserve raw output, provenance, runtime metrics, and validation status.
6. Scan for automatic failure conditions: critical invented repo facts, authority
   claims, controlling-ADR conflicts, missing safety/recovery where material.
7. Update the report and ledger; stop before canonical target-doc edits.

### Acceptance criteria

- Artifact exactly matches the selected packet hash and model digest.
- Output uses the ADR-037 structured schema and claim labels.
- Runtime/provenance evidence is complete.
- Any critical failure invalidates the artifact and is recorded as an incident.

### Evidence to emit

- Raw artifact, wrapper validation result, runtime metrics, model residency evidence,
  automatic-failure scan, and report section.

### Status artifacts affected

- `.agent/local-architect/adr037/<work-item-id>/`, report, this ledger.

### Outcome

`T4` attempted one bounded execution on 2026-07-20 against the frozen `S-140`
packet, but no advisory analysis was produced. The wrapper failed closed before
generation because it sent `POST` to Ollama `/api/tags`, which returned
`HTTP 405 method not allowed`. The failed artifact was preserved at
`.agent/local-architect/adr037/S-140/t4-analysis-artifact.json`; packet/model
expectations remained unchanged, `/api/ps` stayed empty, and no model residency or
structured advisory output was created. `T4` therefore remained blocked pending a
separate wrapper correction task rather than any silent fallback execution.

**Wrapper correction (2026-07-20, RRI 12 Low, `C0 F0 D1 T1 A0 K1 P1 X1`):**
`resolve_model_digest` in `scripts/local-architect/run_analysis.py` called
`fetch_json` against `/api/tags` with the hardcoded `POST` method. `fetch_json`
now accepts an explicit `method` parameter (default `POST`, unchanged for
`/api/generate`), and `resolve_model_digest` passes `method="GET"` with no
request body. Two unit tests were added asserting the HTTP method used per
endpoint; all 6 tests in `run_analysis_test.py` pass. `resolve_model_digest` was
manually re-verified against the live local Ollama instance and now resolves
`qwen3.6:27b-q4_K_M` to the expected digest
`a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e` without
error. This was a Low-band (RRI 0-25) fix, so it did not require the Moderate+
Gemma Reviewer/D14 packet; unit-test evidence above stands in its place. `T4`
itself has not been re-run — re-execution requires its own separate task
presentation and explicit approval per the plan's no-silent-fallback rule, and
remains `Blocked` until that happens.

**Re-execution attempt 2 (2026-07-20) - unhandled client timeout:** with the
GET-method fix in place, `T4` was re-presented and approved, then re-run against
the same frozen `S-140` packet. Packet-hash and model-digest verification both
passed and `qwen3.6:27b-q4_K_M` loaded and became resident in Ollama (confirmed
via `/api/ps`, ~18.5GB VRAM). The `/api/generate` call then exceeded the
wrapper's own `--timeout-seconds 120` default, and `fetch_json` raised a bare
`socket.timeout` that neither of its `except` clauses caught. The exception
propagated unhandled through `run_analysis` and `main`, crashing the process
with a raw traceback and exit code 1 - critically, `write_json_atomic` was never
reached, so no failure artifact was written and the stale HTTP-405 artifact from
attempt 1 remained on disk unchanged despite `--overwrite`. This violated `T2`'s
and `T4`'s own acceptance criteria that failure modes must produce explicit
non-success records. `T4` remained `Blocked`.

**Wrapper correction 2 (2026-07-20, RRI 13 Low, `C0 F0 D1 T1 A0 K1 P1 X2`):**
`fetch_json` in `scripts/local-architect/run_analysis.py` now also catches
`socket.timeout`/`TimeoutError` around the `urlopen` call and converts it into
`AnalysisError("timeout", ...)`, which `main()` already handled generically for
every other `AnalysisError` code (writing a failure artifact via
`build_failure_artifact`). A regression test
(`test_hp3_fetch_json_converts_socket_timeout_to_analysis_error`) monkeypatches
`urlopen` to raise `socket.timeout` and asserts the error surfaces as
`AnalysisError` with `code == "timeout"`. All 7 tests in `run_analysis_test.py`
pass. This was again a Low-band (RRI 0-25) fix; unit-test evidence stands in
place of the Moderate+ review packet.

**Re-execution attempt 3 (2026-07-20) - clean timeout, lost provenance:** `T4`
was re-presented (RRI 35 Moderate, same profile as the original) and approved,
then re-run with `--timeout-seconds 300` against the same frozen packet. The
model was not resident at start (prior keep-alive had expired) and reloaded
successfully; packet hash and model digest both verified. `/api/generate`
still did not return within 300s. This time the timeout-crash fix worked as
intended: the process exited cleanly (code 1, no traceback) and wrote a valid
structured failure artifact with `error.code: "timeout"`. However, the artifact
under-reported provenance: `packet.sha256` and `model.resolved_digest` were
`null` even though both had already been verified successfully before the
`/api/generate` call timed out, because `run_analysis` did not attach the
in-progress `artifact` as `context` when re-raising errors coming from the
`fetcher` call for `/api/tags` or `/api/generate`. `/api/ps` confirmed
`qwen3.6:27b-q4_K_M` was resident post-run (18.5GB VRAM, correct digest), so
the model itself loaded and was presumably generating; only the client-side
timeout budget was insufficient.

**Wrapper correction 3 (2026-07-20, RRI 12 Low, `C0 F0 D1 T1 A0 K1 P1 X1`):**
`run_analysis` now wraps both the `resolve_model_digest` call and the
`/api/generate` `fetcher` call in `try/except AnalysisError`, re-raising with
`context=artifact` so any later failure (including `timeout`) preserves
whatever packet/model provenance was already verified, matching the pattern
already used for `packet_hash_mismatch`/`model_digest_mismatch`/
`invalid_response`. A new regression test
(`test_hp4_generate_timeout_preserves_packet_and_model_provenance_in_failure_artifact`)
asserts `packet.sha256` and `model.resolved_digest` survive a simulated
`/api/generate` timeout. All 8 tests in `run_analysis_test.py` pass. Low-band;
unit-test evidence stands in place of the Moderate+ review packet. `T4` has not
yet been re-run a fourth time - that re-execution requires its own task
presentation and approval, and `T4` remains `Blocked` until a run completes
without a hard crash and either succeeds or produces a valid, provenance-complete
structured failure artifact. Two consecutive genuine timeouts (120s, then 300s)
indicate the next attempt should allow substantially more wall-clock budget
rather than assume another wrapper defect.

**Root-cause diagnosis (2026-07-20) - the timeouts were not a budget problem:**
before attempt 4, the two prior timeouts (120s, 300s) were re-examined rather
than treated as "needs a bigger timeout." `/api/tags` reports
`qwen3.6:27b-q4_K_M` with capabilities `["vision","completion","tools","thinking"]`.
With `stream:false` and no thinking control, Ollama buffers the model's entire,
unbounded chain-of-thought **and** the final response before returning the
single `/api/generate` reply. For a 27B thinking model that reasoning phase can
run for many minutes on this hardware, so any fixed client timeout is racing an
open-ended reasoning chain - raising `--timeout-seconds` (e.g. attempt 4's
planned 900s) only treats the symptom. The correct fix is to control the
reasoning phase itself.

**Wrapper correction 4 - Option C (2026-07-20, RRI 20 Low, `C1 F1 D1 T1 A1 K1 P1 X1`):**
`request_payload` now sends `"think": false` to `/api/generate`, which tells
Ollama to relocate deliberation to a separate `thinking` response field and
return the final `response` without buffering an unbounded interleaved
chain-of-thought first. To keep the failure/success record provenance-complete
(ADR-018) without bloating the artifact with an unbounded reasoning chain, the
success artifact's `generation` block now records `think_disabled: true`,
`thinking_present: <bool>`, and `thinking_sha256: <hash|null>` - a verifiable
fingerprint of any returned deliberation rather than the raw text. Two new
regression tests (`test_hp1b_sends_think_false_and_captures_thinking_provenance`,
`test_hp1c_absent_thinking_field_records_null_provenance`) assert `think:false`
is sent and the thinking provenance fields are populated correctly whether or
not the model returns a `thinking` field. All 10 tests in
`run_analysis_test.py` pass. Low-band; unit-test evidence stands in place of the
Moderate+ review packet. This is the fix targeted at the timeout root cause; a
fourth `T4` re-execution follows against the frozen `S-140` packet, and `T4`
remains `Blocked` until that run completes without a hard crash and either
succeeds or produces a valid, provenance-complete structured failure artifact.

**Re-execution attempt 4 (2026-07-20) - SUCCESS with Option C:** with `think:false`
in place and the model already resident (`/api/ps`: 18.5GB VRAM, correct
digest), `T4` was re-run against the frozen `S-140` packet with
`--timeout-seconds 600 --overwrite`. `/api/generate` returned cleanly in
**3m53s wall-clock** (`total_duration` ~232.8s eval, 1051 output tokens) - well
under budget and, critically, it *completed* rather than timing out, confirming
the root cause was the uncontrolled reasoning phase, not the timeout budget. The
model returned no separate `thinking` field (`thinking_present: false`),
consistent with a deliberation-suppressed fast path. The artifact at
`.agent/local-architect/adr037/S-140/t4-analysis-artifact.json` is
provenance-complete: `success: true`, `status: ok`, `error: null`;
`packet.sha256` == `expected_sha256` (`1e69aea9...`); `model.resolved_digest` ==
`expected_digest` (`a50eda8e...`); `prompt.sha256` and `response.raw_text_sha256`
recorded; new `generation.think_disabled/thinking_present/thinking_sha256`
provenance fields populated. The advisory validates against the ADR-037
structured schema (all eight sections + claim labels from
`{SUPPORTED, INFERRED, UNKNOWN}`).

**Automatic-failure scan (2026-07-20) - PASS, artifact valid:** all four
mandatory conditions were checked against repository and packet evidence:
(1) *Invented repo facts* - none; every `SUPPORTED` claim traces to a packet
excerpt (S-130 READY contract, S-140 has no canonical plan, S-160/S-170 on
fixtures, Rust-first orchestration, feed-the-fail-closed-gate). (2) *Controlling-ADR
conflicts* - none; ADR-006, ADR-018, ADR-030 all exist in `docs/adr/`, were
supplied in the packet constraints, and are applied consistently. (3) *Authority
claims* - none; no recommendation presents itself as an approved canonical
decision, and the Rust-first boundary is preserved. (4) *Missing safety/recovery*
- addressed; fail-closed readiness gating is called out as both a risk and a
recommendation. Genuine uncertainty (exact v1 output format; separate
segmentation model) is correctly labeled `UNKNOWN`, matching the packet's own
`unknowns`. No critical failure condition triggered, so the artifact is **valid**
and no incident is recorded.

**Closure note:** `T4` is an operational/evaluation task; the ledger records
`Code-solution review: n/a - operational/evaluation task; no code change`. The
only code touched during `T4` was the Low-band wrapper (corrections 1-4, each
RRI 0-25), which under `AGENT_WORKFLOW_GUIDE.md` and `HITL_AUTONOMY_POLICY.md`
is exempt from the Gemma Reviewer/D14 packet and carries unit-test evidence
instead (10/10 `run_analysis_test.py` tests pass). The task-analysis review was
`gemma .agent/peer-task-review-t4.json - PASS`. With a valid, provenance-complete
advisory artifact produced and the automatic-failure scan passing, `T4` is
**Done**. `T5` (primary verifies the advisory and authors the canonical
S-140 ADR/plan/tasks) is now unblocked; `T6` remains blocked on `T5`.

---

## T5 - Verify and author the actual project decision, plan, and tasks

- **Status:** `[ ] Blocked`
- **Effort:** L
- **Preliminary RRI:** 50 Med-high; `C2 F3 D4 T2 A3 K3 P2 X4 + arch_decision`
- **Agent grade:** primary + band-routed peer/human
- **Depends on:** T4
- **Allowed paths:** target work-item ADR/plan/tasks/report entries selected in T3
- **Task-analysis review:** pending - RRI 41+ cross-vendor peer/D14 before
  presentation
- **Code-solution review:** `n/a` for docs-only target planning unless a later target
  task includes code

### Objective

The primary agent verifies the advisory artifact against repository evidence and
authors the actual canonical target ADR, plan, and tasks under the normal DubBridge
workflow.

### Steps

1. Recompute RRI for the target canonical planning task.
2. Verify every adopted recommendation against accepted ADRs, architecture docs,
   roadmap, product/BDD docs, and current source evidence.
3. Classify each Local Architect recommendation as accepted, partially accepted,
   rejected, or not evaluated.
4. Author the target canonical ADR/plan/tasks without granting authority to the
   Local Architect artifact.
5. Present and route required approval/review gates for the target work item.
6. Sync the ADR-037 report with the reconciliation summary.

### Acceptance criteria

- Canonical target docs cite verified repository evidence, not model authority.
- All adopted Local Architect claims are fact-checked or rewritten.
- Rejected/partial recommendations are recorded with reasons.
- Normal RRI, phase-1 review, and human approval gates control downstream work.

### Evidence to emit

- Reconciliation table, canonical target docs, review artifacts, approval status,
  and report section.

### Status artifacts affected

- Target work-item ADR/plan/tasks, ADR-037 report, this ledger.

---

## T6 - Record downstream outcome and local-stack utility

- **Status:** `[ ] Blocked`
- **Effort:** M
- **Preliminary RRI:** 34 Moderate; `C0 F2 D3 T2 A1 K2 P1 X4`
- **Agent grade:** Medium evaluator
- **Depends on:** T5 plus the first relevant implementation/review milestone for the
  selected target work item
- **Allowed paths:** `docs/evaluations/adr037-direct-project-report.md`, this ledger
- **Task-analysis review:** `n/a` - evaluation/reporting exemption
- **Code-solution review:** `n/a` - not a code-review task

### Objective

Measure whether the direct Local Architect artifact helped the real project work
after at least one downstream implementation/review milestone exists.

### Steps for a Medium agent

1. Recompute RRI and verify the downstream milestone exists.
2. Compare the pre-planning artifact's accepted/rejected recommendations against
   the primary reconciliation and official implementation/review evidence.
3. Record critical omissions, invented facts, reopened decisions, runtime cost,
   planning usefulness, and any circuit-breaker condition.
4. Update the report and ledger; do not perform code review or change the target
   implementation.

### Acceptance criteria

- Outcome trace links recommendations to primary dispositions and downstream
  evidence.
- Utility assessment is based on recorded project artifacts, not model self-claims.
- Any critical incident disables the consultative lane until correction/retest.
- Report states whether direct use remains healthy, should be retested, or should be
  disabled.

### Evidence to emit

- Recommendation disposition matrix, downstream artifact references, runtime/latency
  summary, incident log if any, and final direct-use health note.

### Status artifacts affected

- `docs/evaluations/adr037-direct-project-report.md`, this task ledger.
