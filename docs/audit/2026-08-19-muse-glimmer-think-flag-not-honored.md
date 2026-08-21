---
type: Audit
title: "Incident: muse-glimmer:30b-q4_K_M ignores think:false under real review packets"
status: open
---

# Incident: `muse-glimmer:30b-q4_K_M` ignores `think:false` under real review packets

## Summary

`gemma_local.build_chat_payload` sets `"think": false` in the Ollama `/api/chat`
request for every local role, including the Gemma/Muse Glimmer Reviewer chain
(`scripts/gemma-code-review.py`). For `muse-glimmer:30b-q4_K_M`, this flag is
demonstrably **not honored** once the request carries a realistic review
system-prompt + packet (the same `system_prompt` in `gemma-code-review.py:186-204`
plus a real ~270-line/~2.8k-token diff) — the model consumes the full
`num_predict` budget generating invisible content and returns
`done_reason: "length"` with **empty `content`**, rather than the expected
`STATUS:`/`SUMMARY:`/`FINDING` tagged block.

This is exactly the failure mode already named in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before implementing`,
Step 0: *"A silent `done_reason: 'length'` with empty `content` (thinking-mode
exhausting the token budget before any visible output) is a known failure mode."*
That doc's Local resource-recovery protocol assumes this shows up as a **memory/
capacity** symptom; this incident shows the same signature can also be a pure
**prompt-template/think-flag** defect, reproducible with normal memory headroom
and normal per-token throughput (~7 tok/s, no stall in the underlying inference
loop itself).

## How this was found

While closing `LRPC-2` (`docs/tasks/local-role-prompt-canonicalization.md`), three
consecutive `make qa-gemma-review` runs against the same small, correct diff
(`scripts/local-agent/prompt_builder.py` + its test, ~270 diff lines) appeared to
stall — CPU time on the `llama-server` process advanced only ~1s per 10s of wall
clock, no output for 5-20+ minutes, across two Ollama restarts and one full
model-unload/reload cycle.

Root-cause isolation (bisection against the live model, outside the wrapper
script):

1. A trivial one-line prompt (`"Reply with ONLY: {...}"`) always completed in
   5-20s at normal throughput (~7.6 tok/s), at both `num_ctx=16384` and
   `num_ctx=131072`. Ruled out: host memory pressure, `num_ctx` mismatch against
   the model's native `16384` context window, and general model/inference-loop
   health.
2. The real `gemma-code-review.py` system prompt alone (`186-204`) + a trivial
   user message: completed normally (24.5s, `done_reason: "stop"`).
3. A trivial system prompt + the real diff packet alone: completed normally
   (32.5s, `done_reason: "stop"`, `prompt_eval_count: 2821`).
4. The **real system prompt + the real diff packet together**, exactly as
   `gemma-code-review.py` builds it, with `num_predict` capped to `1024` (down
   from the production default `4096`) to make the failure observable in bounded
   time: **`done_reason: "length"`, `content: ""`, `eval_count: 1024`** (the full
   budget consumed), `eval_duration: ~142s` at normal per-token throughput.

So neither half alone triggers it; only the specific combination of this
system-prompt's phrasing with a real-size packet does, and the model spends the
entire token budget on invisible output before returning nothing — consistent
with unsuppressed internal "thinking" tokens that `think: false` (the API-level
flag) does not actually gate for this model/chat-template combination. No script
in this repository (`gemma_local.py`, `gemma-code-review.py`,
`run_local_task.py`, `run_analysis.py`) currently prepends a textual `/no_think`
(or equivalent) directive to the prompt itself — every role relies solely on the
API flag.

## Scope / blast radius

- Confirmed affected: `scripts/gemma-code-review.py` (Gemma/Muse Glimmer
  Reviewer, phase-1 and phase-2 review for every RRI 0-55 development task —
  see `AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`).
- Likely affected (not yet independently reproduced, same `build_chat_payload`
  path and same `think` handling): `scripts/local-agent/run_local_task.py`
  (Qwen/local-agent developer role) and `scripts/local-architect/run_analysis.py`
  (Local Architect / ADR-037/ADR-038 refinement).
- Not affected: `prompt_builder.py` itself (LRPC-2's deliverable) — it is a pure
  function with no network IO; this incident is entirely in the consumer
  scripts' request construction, which LRPC-2 explicitly does not touch
  (deferred to LRPC-3/4/5 per `docs/plan/local-role-prompt-canonicalization.md`
  § Architecture).
- `gemma_local.DEFAULT_REVIEW_MODEL = "muse-glimmer:30b-q4_K_M"` — the memory
  `feedback_gemma_reviewer_model_binding.md` recorded this as `gemma4:26b-a4b-it-qat`
  per ADR-036; that binding is now **stale relative to the current code** and
  should be corrected separately.

## Why this task's closure was not blocked on fixing it

Per `AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer / Muse Glimmer Reviewer §
Availability`, three failed passes against the primary/intermediate-fallback
model (the same model in this repo's current binding) with no usable
consolidated result is exactly the trigger for the mandatory D14
context-isolated-adjudicator fallback. `LRPC-2`'s phase-2 review was completed
via D14 instead of blocking on this infrastructure defect; see
`docs/tasks/local-role-prompt-canonicalization.md § LRPC-2 § Peer Reviewer
evidence` for that record.

## Suggested remediation (not yet scoped as a task)

- Prepend a literal `/no_think` (or the correct directive for this model's
  chat template) to the system prompt for `muse-glimmer:30b-q4_K_M` specifically,
  or confirm via Ollama/model documentation what actually suppresses this
  model's internal reasoning under its `chatml` template
  (`--chat-template chatml` is visible in the running `llama-server` invocation).
- Add a bounded-`num_predict` canary check inside `gemma-code-review.py`'s
  pass loop: if a pass returns `done_reason == "length"` with empty `content`,
  treat it as a distinct, explicitly logged failure class (not merged into the
  generic "unavailable/invalid output" bucket) so this is diagnosed faster next
  time, without needing manual bisection.
- Correct `feedback_gemma_reviewer_model_binding.md` (agent memory) — the actual
  current default reviewer model is `muse-glimmer:30b-q4_K_M`
  (`scripts/gemma_local.py:32`), not `gemma4:26b-a4b-it-qat`.
- This should become its own scored task (RRI unknown, likely Moderate given it
  touches `gemma_local.py`'s shared `build_chat_payload` and affects three
  consumer scripts) once prioritized — not scoped or estimated here.

## Status update (2026-08-21)

Still open — none of the remediations above have been implemented. Verified on
2026-08-21: no `/no_think` directive exists anywhere in `scripts/`, and every
local role still relies solely on the API-level `think` flag
(`scripts/gemma_local.py:172`).

Two corrections from `docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md`:

- The memory-binding remediation bullet is **resolved** — `DEFAULT_REVIEW_MODEL`
  is confirmed `muse-glimmer:30b-q4_K_M` (`scripts/gemma_local.py:32`) and the
  agent memory now records that.
- This defect is **more consequential than described here**. The `no usable
  review passes` path (`scripts/gemma-code-review.py:652-659`) returns `3`
  without writing `--out`, which lands directly on the stale-result fail-open in
  `Makefile:118-131` (that audit § D1). The two compose: this model's
  characteristic failure can silently mint a `PASS` receipt sourced from the
  previous task's review. Fixing the Makefile (change **C1**) is therefore
  higher priority than fixing the think flag (change **C4**).

## Status update (2026-08-21, C4 closed)

**C4 implemented and empirically verified.** `Makefile` change C1 (GEG-2)
landed first (commit `dc81cb9`), then C4 landed as a direct-authorship change
(not local-delegated — this task modifies the shared transport
`run_local_task.py` itself depends on; the orchestrator/user judged the
circularity risk did not outweigh proceeding with local delegation for other
work, but this specific task was kept as direct authorship per explicit user
instruction).

- `scripts/gemma_local.py`: added `GemmaThinkOverrunError` (distinct from the
  generic length-cutoff `RuntimeError` — raised only when `done_reason ==
  "length"` **and** accumulated content is empty); added
  `THINK_DIRECTIVE_MODELS = {"muse-glimmer:30b-q4_K_M"}` and
  `THINK_DIRECTIVE_TEXT = "/no_think"`; `build_chat_payload` prepends the
  directive to the system prompt only when `think=False` and the model is in
  that set.
- `scripts/gemma-code-review.py`: extracted the per-pass attempt loop into
  `_run_review_pass` (raw CC 10, B) so the new `"think_overrun"`
  classification did not add to `main()`'s pre-existing CC 34 (E) — `main()`
  dropped to CC 29 (D) post-extraction, confirmed via `radon cc -s`.
  `"think_overrun"` passes are excluded from `succeeded` (same as `"fail"`)
  but tracked and logged separately (`think_overrun_count` in both the
  stderr diagnostic and the audit-log record).
- 7 new unit tests added (`scripts/gemma_local_test.py`,
  `scripts/gemma_code_review_test.py`), covering HP-1/HP-2/EC-1/EC-2 from the
  task definition below. Full suite: 104/104 passing.
- **Live empirical verification** against the real running `muse-glimmer:30b-q4_K_M`
  (not a mock): a ~1.3k-token review packet with a realistic system prompt,
  `think=False`, `num_ctx=65536`, `num_predict=4096`. Pre-fix, this exact
  shape was documented above (§ How this was found) as producing
  `done_reason:"length"` with empty content. Post-fix, two consecutive runs
  both returned `done_reason: "stop"` with real, correctly-formatted
  `STATUS/SUMMARY/FINDING` contract content (400+ chars). Transcript:
  `docs/audit/evidence/c4-post-fix-verification-2026-08-21.txt`.
### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, band-primary for RRI 26-55 —
  passed explicitly via `DUBBRIDGE_REVIEW_MODEL`, not the module default
  `DEFAULT_REVIEW_MODEL` which is muse-glimmer; see
  `feedback_gemma_reviewer_model_binding` memory)
- Command: `DUBBRIDGE_REVIEW_MODEL=gemma4:26b-a4b-it-qat GEMMA_REVIEW_TASK_ID=C4 make qa-gemma-review REVIEW_PATHS="scripts/gemma_local.py scripts/gemma-code-review.py scripts/gemma_local_test.py scripts/gemma_code_review_test.py"`
- Artifact: `docs/audit/gemma-evidence/C4.json`; per-pass results
  `/tmp/dubbridge-gemma-review-C4.pass{1,2,3}.json`
- Verdict: `PASS` (3/3 passes compiled; aggregate status `findings`, both
  findings `minor` severity — `parse-review-findings.py` only fails closed on
  `blocking`/`major`, confirmed via `scripts/parse-review-findings.py`
  exiting 0 on this result)
- Findings: 2 minor —
  (1) consensus, `scripts/gemma_local.py:191`, hardcoded
  `THINK_DIRECTIVE_MODELS`/`THINK_DIRECTIVE_TEXT` couples the workaround to
  one model name;
  (2) pass-specific, `scripts/gemma-code-review.py:380`, string-matching
  `"STATUS PASS cannot include findings"` for the format-retry branch is
  fragile to message drift.
- Muse Glimmer fallback: not triggered — reason: Gemma produced a usable
  3/3-pass aggregate, no fallback condition met.
- D14 fallback: not triggered — reason: n/a, Gemma primary succeeded.
- D14 provider route: n/a
- disposition_divergence: `none` (single-agent disposition, no adjudicator
  invoked)
- Primary-agent disposition: both findings **accepted as noted, no revision**.
  (1) is the task's own explicit design choice — "scoped to the specific
  model(s) confirmed to need it, not applied blindly" (Design decision 1
  above); a config/registry generalization would be premature abstraction for
  a single empirically-confirmed case. (2) is **pre-existing code carried
  over verbatim** by the `_run_review_pass` extraction, not new logic
  introduced by this task — the string-match branch existed identically in
  the original inline loop; out of scope for C4.

### Reflection log

Required passes: 2 (RRI 40 → Moderate)

#### Pass 1

- **Draft verdict:** `GemmaThinkOverrunError` added and distinguished from
  the generic length-cutoff `RuntimeError` by empty-vs-non-empty content;
  `THINK_DIRECTIVE_MODELS`/`THINK_DIRECTIVE_TEXT` prepend `/no_think` only for
  `muse-glimmer:30b-q4_K_M` when `think=False`; `_run_review_pass` extraction
  landed with a third `"think_overrun"` classification. 104/104 tests
  passing. Live-verified twice against the real running model: `done_reason:
  stop` with real content both times.
- **Critique findings:**
  - `payload.get("model")` in the `GemmaThinkOverrunError` raise site assumes
    every caller sets `"model"` in the payload dict; a hand-built payload
    missing it would degrade the exception message to `"None: ..."` rather
    than failing loudly.
  - The single-pass path (`args.passes == 1`) does not get the new
    `"think_overrun"` pass-loop classification — needed to confirm this
    doesn't silently swallow the failure differently than before.
  - `THINK_DIRECTIVE_MODELS` as a one-member set: confirmed intentional, not
    premature abstraction — matches the task's own scoping language.
- **Revisions applied:**
  - Verified the single-pass path: `GemmaThinkOverrunError` subclasses
    `RuntimeError`, and `main()`'s single-pass branch has no local
    `try/except` around `stream_chat`, so it propagates uncaught to the
    top-level `except (RuntimeError, OSError)` handler
    (`scripts/gemma-code-review.py:745`) — exits 1 with the error message
    printed, same fail-closed behavior as any other single-pass failure. No
    change needed: pass-loop classification is inherently a multi-pass
    concept (EC-1/EC-2 are both defined in terms of "pass" classification),
    never in scope for single-pass mode.
  - `payload.get("model")` degrading to `"None"` in its message: confirmed
    unreachable in practice (every real caller builds the payload via
    `build_chat_payload`, which always sets `model`); no defensive code added
    for a boundary that can't occur with actual callers, per the
    don't-validate-what-can't-happen principle. Documented as reviewed, not
    changed.

#### Pass 2

- **Draft verdict:** Implementation stable, full suite green, empirical
  verification recorded and persisted as a committed evidence transcript.
- **Critique findings:**
  - Needed to confirm the `_run_review_pass` extraction preserved exact
    behavior on the pre-existing `"ok"`/`"fail"` paths (the whole point of an
    extraction is zero behavior change on the untouched paths).
  - Needed to confirm `main()`'s CC actually dropped as the design decision
    intended, not just assumed.
  - `docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md`'s own
    "Status artifacts affected" list named this file's status section as
    something to update before closure — not yet done at that point.
- **Revisions applied:**
  - Re-ran the full pre-existing multi-pass test suite (20 tests covering
    format-retry, truncation, per-pass-artifact-writing, and audit-emission)
    unchanged and green — confirms extraction equivalence.
  - Verified via `radon cc -s`: `main()` dropped from raw CC 34 (E) to CC 29
    (D); `_run_review_pass` sits at CC 10 (B); `stream_chat` at CC 11 (C, was
    10); `build_chat_payload` at CC 3 (A, was 1) — all in line with the RRI
    card's `--cc 12` estimate for the genuinely-new logic.
  - Updated this file's "Status update" section with the closure evidence
    (this section itself).

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | muse-glimmer + think=False → `/no_think` prepended, normal parse unaffected | `scripts/gemma_local_test.py::Payload::test_think_directive_prepended_for_muse_glimmer_when_think_false` | passed |
| HP-2 | Happy path | other model + think=False → system prompt unchanged | `scripts/gemma_local_test.py::Payload::test_think_directive_not_prepended_for_other_models` | passed |
| EC-1 | Edge case | `done_reason:"length"` + empty content → `GemmaThinkOverrunError`, classified `"think_overrun"` distinctly in the pass loop | `scripts/gemma_local_test.py::StreamChat::test_stream_chat_length_done_reason_with_empty_content_is_think_overrun`, `scripts/gemma_code_review_test.py::MultiPassCli::test_think_overrun_pass_excluded_but_others_succeed`, `scripts/gemma_code_review_test.py::MultiPassCliAudit::test_think_overrun_count_recorded_in_audit_record` | passed |
| EC-2 | Edge case | `done_reason:"length"` + non-empty content → existing generic `RuntimeError`, still classified `"fail"`, never reclassified | `scripts/gemma_local_test.py::StreamChat::test_stream_chat_rejects_length_done_reason_with_content` | passed |

Additional coverage beyond the four required cases: `test_think_directive_not_prepended_when_think_true` (guards the `think=True` branch stays untouched); `test_all_think_overrun_fails_no_aggregate` (all-think-overrun run behaves like all-fail — exit non-zero, no aggregate written).

Acceptance criterion 3 (live empirical verification): satisfied non-unit-test
evidence — `docs/audit/evidence/c4-post-fix-verification-2026-08-21.txt`
(two consecutive real-model runs, both `done_reason: stop` with parseable
content, replacing the documented pre-fix `length`+empty signature).

Acceptance criterion 4 (no behavior change for other models): covered by
HP-2 and by the full 104/104 suite passing unchanged for every
non-muse-glimmer code path.

### Owner final verification

- Owner: `matias` (kruk.matias@gmail.com)
- Date: `2026-08-21`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, and that
  acceptance criterion 3 (live empirical verification against the real
  running model) was independently satisfied outside the unit-test suite.
- Commands run: `python3 -m pytest scripts/gemma_local_test.py
  scripts/gemma_code_review_test.py -q`; `python3 -m radon cc
  scripts/gemma_local.py scripts/gemma-code-review.py -s`; `python3
  /tmp/verify_c4.py` (twice, live against `muse-glimmer:30b-q4_K_M` via
  Ollama); `DUBBRIDGE_REVIEW_MODEL=gemma4:26b-a4b-it-qat
  GEMMA_REVIEW_TASK_ID=C4 make qa-gemma-review REVIEW_PATHS="scripts/gemma_local.py
  scripts/gemma-code-review.py scripts/gemma_local_test.py
  scripts/gemma_code_review_test.py"`; `make qa-docs`.

**Task status: `[x] Done`.**

## Scoped task: C4 — think-flag remedy + distinct failure class

- **Status:** `[x] Done` — approved 2026-08-21 ("arranca" sequence approval +
  explicit routing confirmation "realizalo tu mismo sin local"); implemented,
  reviewed, and closed same day. Full closure record in § Status update
  (2026-08-21, C4 closed) above.
- **Effort:** M · **RRI:** 40 (Moderate) · **Type:** development
- **RRI evidence:** `python3 scripts/rri.py --cc 12 --touches scripts/gemma_local.py --touches scripts/gemma-code-review.py --T 1 --A 2 --X 2 --D 3 --K 3 --P 2`
  → `C=2 (raw CC 12, post-refactor stream_chat) · F=1 (2 files) · D=3 (workflow/
  external-integration state machine, no anchor-rubric match) · T=1 (gemma_local_test.py
  / gemma_code_review_test.py exist) · A=2 (prepend location + classification shape need
  judgment) · K=3 (external Ollama API integration) · P=2 (changes shared internal
  behavior every local role's transport depends on) → base 40 → band Moderate,
  no penalties triggered`. A naive first pass scored this Complex (58) by
  attributing `gemma-code-review.py:main()`'s full pre-existing CC 34 to the
  task (`main()` is E-rated/34, the file's most complex function by far) — see
  Design decision 1 below for why that attribution is not used.
- **Objective:** Make the shared local-model transport (`scripts/gemma_local.py`)
  actually suppress `muse-glimmer:30b-q4_K_M`'s internal reasoning when
  `think=False` is requested, and make a `done_reason: "length"` +
  empty-`content` response a distinct, explicitly logged failure class in
  `scripts/gemma-code-review.py`'s pass loop instead of falling into the
  generic parse-failure bucket.
- **Context:** Root-caused by direct bisection against the live model (§ How
  this was found, above): the real system prompt + a real-size packet
  (~2.8k tokens) makes `muse-glimmer:30b-q4_K_M` consume its entire
  `num_predict` budget on invisible reasoning and return empty content,
  regardless of the API-level `think: false` flag. No script in the repo
  prepends a textual think-suppression directive — every role relies solely
  on the flag (verified 2026-08-21, § Status update above). This is the
  band-primary reviewer for RRI 0–25 and the band-intermediate-fallback for
  RRI 26–55 (`AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`), so every
  local review at every band that falls through to this model hits this
  defect deterministically, not intermittently — sequenced by the governing
  audit (`docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md:341-343`)
  as the fix to land before re-evaluating the muse-glimmer binding itself.
- **Related documents:** this file (root cause, bisection evidence);
  `docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md` § C4 (change
  entry), lines 341-343 (sequencing); `scripts/gemma_local.py`;
  `scripts/gemma-code-review.py`; `docs/playbooks/AGENT_WORKFLOW_GUIDE.md §
  Mandatory workflow before implementing` Step 0 (names this exact symptom as
  a known failure mode, currently mis-filed under the memory/capacity
  resolution path).
- **Design decision 1 — extract per-pass execution instead of editing `main()`
  inline.** `scripts/gemma-code-review.py:main()` is already CC 34 (E-rated,
  `radon cc -s`). Adding the new classification branch as one more inline
  `except` clause in the multi-pass loop would bury new decision logic inside
  an already very-high-complexity function, which is both a real
  maintainability regression and a scoring artifact (it would score this task
  Complex/56+, forcing mandatory decomposition of what is otherwise a bounded
  two-file fix). Extracting the existing per-pass attempt loop (already
  self-contained: build request → `stream_chat` → `parse_review_response`,
  with its one format-retry branch) into a new `_run_review_pass(...)` helper
  is motivated directly by this task's own requirement — a distinct
  classification needs its own decision point — not an unrelated cleanup, and
  it is independently unit-testable. `main()`'s own touched surface shrinks to
  a call-site swap plus one new branch on the returned classification.
- **Outputs:**
  - `scripts/gemma_local.py`: new `GemmaThinkOverrunError(RuntimeError)`,
    raised from `stream_chat` when `done_reason == "length"` **and** the
    joined streamed content is empty/whitespace-only; the existing generic
    `RuntimeError("response cut by token limit...")` remains for a
    `length` cutoff that *did* produce partial visible content (a materially
    different failure — genuine truncation, not suppressed-reasoning
    overrun).
  - `scripts/gemma_local.py`: `THINK_DIRECTIVE_MODELS` set (seeded with
    `muse-glimmer:30b-q4_K_M`) + `THINK_DIRECTIVE_TEXT = "/no_think"`;
    `build_chat_payload` prepends the directive to `system_prompt` when
    `think` is `False` and `model` is in the set. Gated by model, not
    applied globally — `DEFAULT_MODEL`/Gemma's own template is unaffected.
  - `scripts/gemma-code-review.py`: new `_run_review_pass(...)` helper
    (extracted per-attempt loop); the multi-pass loop in `main()` gains a
    third pass-result class `"think_overrun"` alongside `"ok"`/`"fail"`,
    counted separately in the aggregate and in `append_audit_log`, and named
    explicitly in the `no usable review passes` message when every failure in
    a run is think-overrun (so the failure is diagnosable from the log
    without re-running manual bisection).
- **Acceptance criteria:**
  1. `gemma_local_test.py` / `gemma_code_review_test.py` cover: (a) a
     `done_reason:"length"` + empty-content stream raises
     `GemmaThinkOverrunError`, not the generic `RuntimeError`; (b) a
     `done_reason:"length"` + non-empty-content stream still raises the
     existing generic `RuntimeError`; (c) `build_chat_payload` prepends
     `/no_think` for `muse-glimmer:30b-q4_K_M` with `think=False` and leaves
     the prompt unmodified for every other model and for `think=True`.
  2. `gemma-code-review.py`'s multi-pass loop classifies a think-overrun
     failure distinctly from a parse/format failure in both the printed
     summary and `append_audit_log`'s record.
  3. **Live empirical verification** (not just unit-mocked): reproduce the
     original failure signature by sending a real-size packet (a real diff +
     the production `gemma-code-review.py` system prompt) to the live
     `muse-glimmer:30b-q4_K_M` via Ollama with the *pre-fix* code, confirm
     `done_reason:"length"` + empty content; re-run the identical packet
     post-fix; confirm the directive changes the outcome (either a normal
     tagged response, or — if the directive alone is insufficient — a
     `GemmaThinkOverrunError` raised cleanly instead of an opaque generic
     failure). Record the transcript.
  4. No behavior change for any model other than `muse-glimmer:30b-q4_K_M`
     (Gemma Developer's `DEFAULT_MODEL`/`DEFAULT_FALLBACK_MODEL` path is
     untouched).
- **Behavioral examples:**
  - **HP-1:** `muse-glimmer:30b-q4_K_M` review request with `think=False` →
    payload's system prompt carries the `/no_think` prefix; a normal
    `STATUS:`/`SUMMARY:` response parses exactly as before.
  - **HP-2:** any other model (e.g. `gemma4:26b-a4b-it-qat`) with
    `think=False` → payload unchanged, no directive prepended.
  - **EC-1:** `done_reason:"length"` with empty content (think-overrun
    signature) → `GemmaThinkOverrunError`, classified `"think_overrun"` in
    the pass loop, distinct count in the aggregate/audit log — never silently
    merged into `"fail"`.
  - **EC-2:** `done_reason:"length"` with non-empty content (genuine
    truncation, e.g. a very long findings list) → existing generic
    `RuntimeError`, classified `"fail"` as before — this task must not
    reclassify real truncation as think-overrun.
- **Evidence to emit:** phase-1 and phase-2 review artifacts; the live
  empirical transcript from acceptance criterion 3; `make qa-docs` output;
  Reflection log (2 passes, Moderate).
- **Status artifacts affected:** this file (`## Status update` →
  resolved/landed); `docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md`
  (mark **C4** landed, and re-open the "revisit the muse-glimmer binding"
  question it defers to post-C4, lines 341-343); the
  `feedback_muse_glimmer_think_flag_defect` agent memory (the defect section
  moves from "still open" to closed, once evidence confirms the remedy
  actually changes the empirical outcome — not merely that code shipped).
- **Implementation routing — resolved.** RRI 40 Moderate defaults to
  local-first via `scripts/local-agent/run_local_task.py`
  (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Local-first ... implementation
  routing`). The orchestrator recommended extending the GEG-2 local-dev
  override to this task by the same circularity rationale (this task
  modifies the exact shared transport `run_local_task.py` itself depends on).
  The owner's first response ("ejecuta la task con implementor local...")
  initially pointed the other way; on the orchestrator surfacing that this
  contradicted the circularity rationale and asking for explicit
  confirmation, the owner confirmed direct authorship: **"realizalo tu mismo
  sin local"** (2026-08-21). Implemented directly by the primary agent, no
  `run_local_task.py` delegation.
- **Antares touchpoint:** skipped (typed). No task-relevant CWE hypothesis on
  the watchlist (`scripts/antares/cwe_watchlist.py`: CWE-89 SQL injection,
  CWE-306 missing-auth-for-critical-function, CWE-22 path traversal) applies
  to local-model HTTP transport retry/classification logic.

## Related

- `docs/audit/2026-08-21-muse-glimmer-role-fitness-review.md` — role-fitness
  measurement and the five review-pipeline defects this incident feeds into
- `docs/tasks/local-role-prompt-canonicalization.md` § LRPC-2
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Mandatory workflow before
  implementing (Step 0, local resource-recovery protocol), § Gemma Reviewer /
  Muse Glimmer Reviewer § Availability
- `scripts/gemma_local.py` (`DEFAULT_REVIEW_MODEL`, `build_chat_payload`)
- `scripts/gemma-code-review.py` (`build_review_payload`, system prompt)
