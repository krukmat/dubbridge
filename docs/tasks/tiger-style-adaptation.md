---
type: TaskList
title: "Tasks: Tiger Style Adaptation (X26)"
status: planned
slice: tiger-style-adaptation
plan: docs/plan/tiger-style-adaptation.md
governed_by: [ADR-006, ADR-008, ADR-018, ADR-021, ADR-026]
---

# Tasks: Tiger Style Adaptation (X26)

> **Status:** Planned 2026-08-30. Owner resolved D1 (`assert!` always-on), D2
> (lower `too_many_lines` to 70 now), and D3 (Postgres/Redis/MinIO integration
> tests mandatory in CI now) — see `docs/plan/tiger-style-adaptation.md`. No
> task below has been RRI-scored via `scripts/rri.py` yet; `Effort` fields are
> provisional estimates from the illustrative S/M/L/XL rubric
> (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Effort scale`), not the canonical
> RRI-band mapping. Each task must be scored with `scripts/rri.py` immediately
> before it is presented or delegated, per the mandatory workflow — the
> provisional Effort here is planning input only and must be corrected to
> match the computed band if they disagree.

## Ordering and dependencies

```
T0 -> T1 -> T2
T1 -> T3 -> T4
T5                (independent — CI/storage)
T6 -> T7 -> T8 -> T9 -> T10 -> T11   (independent — Python ASR)
T12               (independent — docs-only, no code dependency)
```

---

## X26-T0: Survey functions in the 70–100 line band

**Type:** Mechanical / analysis
**Effort:** S (provisional)
**Depends on:** none
**Status:** [x] Done — artifact created, pending commit approval (see below)
**RRI:** 0 -> band Low (0-25); `python3 scripts/rri.py --C 0 --T 0 --A 0 --X 0
--D 0 --K 0 --P 0 --touches docs/audit/tiger-style-70-100-line-survey.md
--platform dubbridge`. No anchor-rubric floor matched (no governed crate
touched); no full approval card required per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (Low band, mechanical/analysis work
stays with the primary agent, not delegated).
**Task-analysis review:** n/a - mechanical survey, no source/production code
changes (exempt per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Band-routed
peer review).

**Objective:** Enumerate every function currently between 70 and 100 lines
across the Rust workspace, so Phase 1's decomposition work (`X26-T1`) and the
`too_many_lines` threshold flip (`X26-T2`) have a known, bounded blast radius
before either starts.

**Acceptance criteria:**
- A committed table (file, function, line count) covering every workspace
  crate/app, produced via a scratch clippy run at a temporary 70-line
  threshold or an equivalent line-count script — not yet enforced.
- `finalize_ingestion_core` (`crates/ingestion/src/lib.rs:48-145`, 97 lines)
  appears in the table.
- No source file is modified by this task.

**Evidence to emit:** the survey table, committed under
`docs/audit/tiger-style-70-100-line-survey.md` (or equivalent), plus the exact
command used to produce it.

**Status artifacts affected:** this task ledger (mark done), the survey
artifact above.

**Agent handoff prompt:** Enumerate every 70–100 line function workspace-wide
using clippy's `too_many_lines` at threshold 70 in dry-run/report mode (or an
equivalent script); commit the table; do not modify any source file; stop
after the table is committed.

**Stop condition:** Stop once the survey artifact is committed. Do not begin
decomposing any function.

**Result:** 12 functions found at clippy's 70-line `too_many_lines` ceiling
(3 production, 9 test) — table, method, and reproduction command at
`docs/audit/tiger-style-70-100-line-survey.md`. `finalize_ingestion_core`
appears as required (77 lines by clippy's count / 97 by raw file span,
both recorded — see that artifact's Method section for why they differ).
No source file was modified: the survey reused the workspace's own
`clippy::too_many_lines` lint via a temporary `clippy.toml` threshold
(70) and a command-line lint-level override, then reverted `clippy.toml`
to its committed state before this entry was written (verified via `git
diff`/`git status`, zero pending diff on `clippy.toml`). Open scoping
question for `X26-T1` recorded in the survey artifact: whether the 9
test-code rows are in that task's decomposition scope. The survey artifact
itself is created but not yet `git commit`-ed — committing requires your
explicit approval per `docs/policies/HITL_AUTONOMY_POLICY.md` § Always
requires explicit approval.

---

## X26-T1: Decompose functions flagged by the 70–100 line survey

**Type:** Development
**Effort:** M (provisional — likely spans several small development tasks in
practice; split per-function at RRI-scoring time if the combined survey
result is large)
**Depends on:** X26-T0
**Status:** [ ] Planned

**Objective:** Decompose every function `X26-T0` flagged (starting with
`finalize_ingestion_core`) to ≤70 lines each, preserving existing behavior and
test coverage, so `X26-T2` can flip the lint threshold without breaking CI.

**Happy paths considered:**
- **HP-1:** A flagged function is split into named helper functions ≤70 lines
  each; the original call site's public signature and behavior are unchanged;
  existing tests for that function pass unmodified.

**Edge cases considered:**
- **EC-1:** A function whose logic genuinely cannot be cleanly split without
  harming readability (e.g. a single linear sequence with no natural seam) is
  documented as a named exception with justification, not silently `#[allow]`ed.

**Acceptance criteria:**
- Every function in the `X26-T0` survey table is either decomposed to ≤70
  lines or recorded as a named, justified exception.
- `finalize_ingestion_core` is decomposed to ≤70 lines using or extending its
  existing five sub-70-line helpers.
- `make qa-test` and `make qa-coverage` pass unchanged in outcome (same tests
  pass; coverage does not regress below the 90% gate).
- No new `#[allow(clippy::too_many_lines)]` or `#[allow(clippy::
  cognitive_complexity)]` introduced without a recorded justification.

**Evidence to emit:** diff, `make qa-test`/`make qa-coverage` output,
before/after line counts per flagged function.

**Status artifacts affected:** this task ledger, `X26-T0`'s survey artifact
(mark each row resolved).

**Agent handoff prompt:** Using `X26-T0`'s survey table, decompose each
flagged function to ≤70 lines, starting with `crates/ingestion/src/lib.rs`'s
`finalize_ingestion_core`. Governing docs: this plan
(`docs/plan/tiger-style-adaptation.md`), this task's acceptance criteria.
Stop condition: stop once every survey row is resolved and `make qa-test`/
`make qa-coverage` pass; do not touch the `too_many_lines` lint threshold
itself (that is `X26-T2`).

---

## X26-T2: Lower `too_many_lines` from 100 to 70

**Type:** Development (config)
**Effort:** S (provisional)
**Depends on:** X26-T1
**Status:** [ ] Planned

**Objective:** Flip the workspace lint ceiling to match Tiger Style's ~70-line
ideal, now that `X26-T1` has closed the gap it would otherwise open.

**Happy paths considered:**
- **HP-1:** `Cargo.toml`'s `too_many_lines` deny threshold (currently the
  100-line default at `Cargo.toml:65`) is set to 70; `make qa-lint` passes
  with zero new violations.

**Edge cases considered:**
- **EC-1:** A function `X26-T1` recorded as a named exception still fails at
  70; its `#[allow(clippy::too_many_lines)]` plus justification comment is
  the only accepted way to keep `make qa-lint` green for that function.

**Acceptance criteria:**
- `too_many_lines` enforced at 70 lines workspace-wide.
- `make qa-lint` passes.
- Zero unjustified new `#[allow]` attributes (every one traces to an
  `X26-T1`-recorded exception).

**Evidence to emit:** `make qa-lint` output before/after.

**Status artifacts affected:** this task ledger.

**Agent handoff prompt:** Set the `too_many_lines` clippy threshold to 70 in
`Cargo.toml`/`clippy.toml`; run `make qa-lint`; the only acceptable new
`#[allow]` attributes are those `X26-T1` already justified. Stop condition:
stop once `make qa-lint` is green.

---

## X26-T3: Add `assert!` pre/postconditions at safety-critical boundaries

**Type:** Development
**Effort:** M (provisional — likely decomposed per boundary at RRI-scoring
time given it touches four distinct crates)
**Depends on:** X26-T1 (assert on the decomposed, smaller functions)
**Status:** [ ] Planned

**Objective:** Introduce paired precondition/postcondition `assert!` calls
(always-on, compiled into release per D1) at the four safety-critical
boundaries the evaluation found to be assertion-free: rights validation,
finalize, playback grant issuance, audit emission.

**Happy paths considered:**
- **HP-1:** `RightsBasis::validate` asserts a stated positive-space invariant
  (e.g. a validated basis always carries a non-empty owner reference) after
  validation succeeds, without changing its `Result` return contract.
- **HP-2:** `finalize_ingestion_core`'s decomposed helpers (post-`X26-T1`)
  assert a documented precondition on entry (e.g. the pending-ingestion row
  exists and is not already finalized) before proceeding.

**Edge cases considered:**
- **EC-1:** A condition that is actually reachable via external/attacker
  input (not a pure programmer invariant) is *not* converted to `assert!` —
  it stays `Result`-typed. Reviewer must flag any misclassification.
- **EC-2:** `PlaybackGrant::is_valid_at` asserts negative space (e.g. a grant
  already known expired never reaches the "valid" branch) in addition to the
  positive-space case.

**Acceptance criteria:**
- At least one precondition and one postcondition assert added at each of:
  rights validation, finalize, playback grant issuance, audit emission.
- No existing recoverable-error `Result` path is replaced by an assert.
- `make qa-test` passes; coverage does not regress.
- Each new assert has a comment stating the invariant it encodes (why it must
  always hold), per Tiger Style's narrative-naming pillar.

**Evidence to emit:** diff, `make qa-test` output, list of assert sites added
with the invariant each encodes.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R1 closed).

**Agent handoff prompt:** Add paired `assert!` pre/postconditions at the four
boundaries named above, classifying each candidate condition as invariant
(assert) vs. recoverable (stays `Result`) per this plan's Design decisions
section. Stop condition: stop once all four boundaries have at least one
precondition and one postcondition assert and `make qa-test` passes.

---

## X26-T4: Add explicit retry/attempt caps to job/provider/media retry paths

**Type:** Development
**Effort:** M (provisional)
**Depends on:** X26-T3 (no code dependency; sequenced after Phase 2 in the
plan for review-bandwidth reasons, can run in parallel if capacity allows)
**Status:** [ ] Planned

**Objective:** Close the one "explicit bounds" gap the evaluation found: the
`Retryable` disposition path has no visible attempt ceiling.

**Happy paths considered:**
- **HP-1:** A retryable job failure is retried up to a defined maximum
  attempt count, then transitions to a terminal failure state instead of
  retrying indefinitely.

**Edge cases considered:**
- **EC-1:** A job already at the maximum attempt count that fails again does
  not re-enter the retry path; it is durably marked failed (ADR-018 audit row
  emitted).

**Acceptance criteria:**
- `apps/worker-runner/src/translation_fanout.rs`'s `Retryable` disposition
  path (and any equivalent in `crates/jobs`, `crates/providers`,
  `crates/media`) enforces a named, configured maximum attempt count.
- Exceeding the cap is durably audited, not silently dropped.

**Evidence to emit:** diff, unit test proving the cap is enforced,
`make qa-test` output.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R4 closed).

**Agent handoff prompt:** Add an explicit, configured maximum attempt count to
the `Retryable` disposition path in `apps/worker-runner/src/
translation_fanout.rs` and its equivalents in `crates/jobs`/`crates/providers`/
`crates/media`; audit the terminal-failure transition. Stop condition: stop
once the cap is enforced, tested, and audited.

---

## X26-T5: MinIO CI service + `qa-test-s3` mandatory integration test

**Type:** Development (CI/ops)
**Effort:** M (provisional — CI-runner-availability risk)
**Depends on:** none
**Status:** [ ] Planned

**Objective:** Close the actual D3/R12 gap found on re-verification: the S3
integration test in `crates/storage/src/s3.rs:182` is `#[ignore]`d, the
`qa-test-s3` Makefile target it references does not exist, and no CI job
provisions MinIO. Postgres/Redis are already mandatory in CI (see plan's
re-verification section) and need no code change here.

**Happy paths considered:**
- **HP-1:** A new CI job (or an addition to an existing one) starts a
  `minio` service container, sets `DUBBRIDGE_STORAGE_TEST_ENDPOINT`/
  `_ACCESS_KEY_ID`/`_SECRET_ACCESS_KEY`/`_BUCKET`, and `make qa-test-s3` runs
  the previously-`#[ignore]`d S3 integration test to a passing result.

**Edge cases considered:**
- **EC-1:** MinIO service container fails its health check — the CI job fails
  closed (does not silently skip the S3 test), consistent with the "mandatory"
  intent of D3.

**Acceptance criteria:**
- `Makefile` gains a `qa-test-s3` target running the ignored S3 test(s) with
  required env vars, failing if they are unset (matching `qa-test-redis`'s
  existing fail-closed pattern).
- `.github/workflows/ci.yml` provisions a `minio` service container and runs
  `make qa-test-s3` unconditionally in a job.
- `make qa-test`'s own job either gains a Postgres service or the plan's
  decision to rely on the `coverage` job for that coverage is recorded here
  as intentional (no silent gap either way).

**Evidence to emit:** CI run showing the new job passing, `make qa-test-s3`
output.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R12 closed),
`docs/plan/tiger-style-adaptation.md` (confirm re-verification note matches
final implementation).

**Agent handoff prompt:** Add a `qa-test-s3` Makefile target (fail-closed on
missing `DUBBRIDGE_STORAGE_TEST_*` env vars, mirroring `qa-test-redis`), add a
MinIO service container to `.github/workflows/ci.yml`, and wire the job to run
it unconditionally. Stop condition: stop once CI runs the S3 integration test
to a passing result on a real MinIO service.

---

## X26-T6: Python complexity gate for `workers/*-py`

**Type:** Development (tooling)
**Effort:** S (provisional)
**Depends on:** none
**Status:** [ ] Planned

**Objective:** Add a complexity/length enforcement mechanism to the Python
worker surface before `translation-worker-py`/`tts-worker-py` gain real code —
currently none exists at all.

**Happy paths considered:**
- **HP-1:** `ruff` (with complexity rules enabled) or `flake8` + `mccabe` is
  configured scoped to `workers/*-py`, wired into a `make` target, and passes
  against the existing `workers/asr-worker-py/main.py`.

**Edge cases considered:**
- **EC-1:** The gate does not accidentally scope-creep into
  `scripts/*.py` (repo-root agent tooling), which the evaluation explicitly
  marks out of scope.

**Acceptance criteria:**
- A `ruff`/`flake8`+`mccabe` config scoped to `workers/` only.
- A new `make` target (e.g. `qa-python-complexity`) runs it.
- The gate passes against current `workers/asr-worker-py/main.py`.

**Evidence to emit:** new Makefile target output, tool config file.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R8 closed).

**Agent handoff prompt:** Add a `ruff` (or `flake8`+`mccabe`) complexity gate
scoped to `workers/*-py` only, with a new Makefile target; verify it passes
against `workers/asr-worker-py/main.py`. Stop condition: stop once the gate
runs green via the new Makefile target.

---

## X26-T7: ASR worker — guard clauses and narrow exceptions (R2/R3)

**Type:** Development
**Effort:** S (provisional)
**Depends on:** X26-T6 (gate should exist before further worker edits, so new
code is measured from the start)
**Status:** [ ] Planned

**Objective:** Replace `dict.get()`-with-silent-defaults and the broad
`except Exception` catch-all in `workers/asr-worker-py/main.py` with explicit
guard clauses and named exception types.

**Happy paths considered:**
- **HP-1:** A well-formed input with all required fields parses and proceeds
  exactly as today.

**Edge cases considered:**
- **EC-1:** An input missing `audio_uri` (or any other required field)
  raises a specific, named error (not a silently-defaulted empty value) that
  maps to `error.schema.json`'s `invalid_input` code.
- **EC-2:** A `faster-whisper` transcription failure and a file-not-found
  failure now map to two distinct named exception types/error codes instead
  of being flattened into one `except Exception`.

**Acceptance criteria:**
- `main.py:32-34`'s `dict.get()`-with-defaults pattern replaced with explicit
  `if not <condition>: raise <SpecificError>` guard clauses.
- `main.py:70`'s `except Exception as exc` replaced with narrower, named
  exception types mapped to distinct error codes.
- Existing `tests/test_worker.py` behavior preserved or updated to match the
  new explicit contract; `make qa-python-complexity` (from `X26-T6`) still
  passes.

**Evidence to emit:** diff, test output.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R2/R3 closed).

**Agent handoff prompt:** In `workers/asr-worker-py/main.py`, replace
`dict.get()`-with-defaults with explicit guard clauses raising named
exceptions, and replace the broad `except Exception` with narrow, named
exception types mapped to distinct error codes. Stop condition: stop once
tests pass and the complexity gate is green.

---

## X26-T8: ASR worker — bound the model call and validate `language_hint` (R5/R6)

**Type:** Development
**Effort:** S (provisional)
**Depends on:** X26-T7
**Status:** [ ] Planned

**Objective:** Add an explicit timeout and max-audio-duration/size bound
around `WhisperModel(...).transcribe()`, and validate `language_hint` against
a known-language allowlist before it reaches faster-whisper.

**Happy paths considered:**
- **HP-1:** A normal-length audio file with a valid `language_hint` (e.g.
  `"en"`) transcribes as today, within the new bound.

**Edge cases considered:**
- **EC-1:** An audio file exceeding the configured max duration/size is
  rejected before the model call, with a specific error code — not left to
  run unbounded.
- **EC-2:** An unrecognized `language_hint` value (not in the allowlist) is
  rejected at the contract boundary instead of passed through to
  faster-whisper unvalidated.

**Acceptance criteria:**
- A configured timeout wraps the `transcribe()` call.
- A configured max-audio-duration and/or max-file-size check runs before the
  model call.
- `language_hint` is validated against an explicit allowlist.

**Evidence to emit:** diff, tests for both edge cases.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R5/R6 closed).

**Agent handoff prompt:** In `workers/asr-worker-py/main.py`, add a timeout
and max-duration/size bound around `WhisperModel(...).transcribe()`, and
validate `language_hint` against an explicit allowlist before use. Stop
condition: stop once both bounds are enforced and tested.

---

## X26-T9: ASR worker — runtime JSON Schema enforcement (R9)

**Type:** Development
**Effort:** S (provisional)
**Depends on:** X26-T7
**Status:** [ ] Planned

**Objective:** Enforce `input.schema.json`/`output.schema.json`/
`error.schema.json` at the process boundary at runtime, replacing the current
documentation-only contract.

**Happy paths considered:**
- **HP-1:** A conforming input JSON object validates against
  `input.schema.json` and proceeds; the emitted success output validates
  against `output.schema.json`.

**Edge cases considered:**
- **EC-1:** An input with an extra, undeclared property (violating the
  schema's `additionalProperties: false`) is rejected at the boundary with an
  `error.schema.json`-conformant error, not silently accepted.

**Acceptance criteria:**
- `jsonschema` validation (or schema-derived Pydantic models) runs against
  all three schemas at their respective boundaries.
- A new dependency (`jsonschema` or `pydantic`) is added to
  `requirements.txt` and pinned.

**Evidence to emit:** diff, tests for a schema-violating input and output.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R9 closed).

**Agent handoff prompt:** Add runtime `jsonschema` (or Pydantic)
enforcement of `input.schema.json`/`output.schema.json`/`error.schema.json`
in `workers/asr-worker-py/main.py`. Stop condition: stop once all three
schemas are enforced and tested against at least one violating case each.

---

## X26-T10: ASR worker — lock transitive dependencies (R10)

**Type:** Development (dependency management)
**Effort:** S (provisional)
**Depends on:** none
**Status:** [ ] Planned

**Objective:** Lock `faster-whisper`'s transitive dependencies (numpy,
ctranslate2, huggingface-hub, tokenizers, onnxruntime, av) so the Docker build
stops floating on whatever those resolve to.

**Happy paths considered:**
- **HP-1:** `Dockerfile:5`'s `pip install -r requirements.txt` is replaced
  (or supplemented) with an install from a compiled lockfile
  (`requirements-lock.txt` via `pip-compile`, or `uv.lock`), producing
  reproducible transitive-dependency versions across builds.

**Edge cases considered:**
- **EC-1:** A future `faster-whisper` version bump requires regenerating the
  lockfile explicitly — it must not silently drift.

**Acceptance criteria:**
- A committed lockfile pinning every transitive dependency.
- `Dockerfile` updated to install from the lockfile.

**Evidence to emit:** lockfile, Docker build log showing pinned versions.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R10 closed).

**Agent handoff prompt:** Generate a compiled lockfile for
`workers/asr-worker-py/requirements.txt` (via `pip-compile` or `uv`) and wire
`Dockerfile` to install from it. Stop condition: stop once the Docker build
uses only pinned versions.

---

## X26-T11: ASR worker — real-audio/real-model smoke test (R11)

**Type:** Development (test)
**Effort:** M (provisional — model-download cost)
**Depends on:** X26-T7, X26-T8, X26-T9 (exercises the hardened contract)
**Status:** [ ] Planned

**Objective:** Add at least one checked-in, opt-in, real-audio/real-model
smoke test, replacing the current 100%-mocked suite's single point of
unverified truth.

**Happy paths considered:**
- **HP-1:** A short, checked-in real audio fixture transcribes successfully
  through the real `faster-whisper` model (opt-in, gated behind an env var
  given model-download cost) and produces output matching
  `output.schema.json`.

**Edge cases considered:**
- **EC-1:** The smoke test is skipped (not failed) when its gating env var is
  unset, so it does not block ungated local/CI runs — mirroring the existing
  Rust integration-test opt-in pattern, but explicit about the skip (not
  silent).

**Acceptance criteria:**
- A committed, opt-in real-model test exists and passes when explicitly
  enabled.
- The test is documented (README or test docstring) with the env var that
  gates it and the model-download cost it incurs.

**Evidence to emit:** test output from an explicit gated run.

**Status artifacts affected:** this task ledger,
`docs/proposals/tiger-style-adaptation-evaluation.md` (mark R11 closed).

**Agent handoff prompt:** Add one opt-in, real-audio, real-model smoke test
for `workers/asr-worker-py`, gated behind an explicit env var, documented
with its model-download cost. Stop condition: stop once the test passes when
explicitly enabled and is skipped (not silently ignored) otherwise.

---

## X26-T12: S-150 T4–T7 forward-pointer for R13 (docs-only)

**Type:** Docs-only
**Effort:** S (provisional; exempt from RRI-band approval gate per
docs-only classification, but still recorded here for completeness)
**Depends on:** none
**Status:** [ ] Planned

**Objective:** Record, durably, that when S-150 `T4` resumes (currently
ADR/planning-only, blocked on the ADR-028 consent seam), its decomposed
`T5`–`T7` child task cards must require R2/R3-equivalent guard clauses,
R5/R6-equivalent bounds, `X26-T6`'s complexity gate (inherited, not
re-implemented), R9-equivalent schema enforcement, and R10-equivalent
dependency locking as first-commit acceptance criteria.

**Acceptance criteria:**
- A note is added at the point where S-150 `T4`'s decomposition happens (not
  now, since `T4` itself is still parked) referencing
  `docs/plan/tiger-style-adaptation.md` Phase 7.
- This plan and task ledger, plus `docs/plan/roadmap.md`'s X26 row, already
  carry the forward-pointer as of this task's creation — closing this task
  means confirming that pointer is still accurate at the time S-150 `T4`
  actually resumes, not creating new content now.

**Evidence to emit:** none at creation time (forward-pointer only); at S-150
`T4` resumption time, the amended T5–T7 child task cards themselves.

**Status artifacts affected:** `docs/plan/s-150-translation-dubbing.md`,
`docs/tasks/s-150-translation-dubbing.md` (only when S-150 `T4` actually
resumes — not part of this task's closure).

**Agent handoff prompt:** No action until S-150 `T4` resumes. At that point,
confirm its decomposed child tasks carry the acceptance criteria named above
before they are approved.

**Stop condition:** This task stays open (not closed) until S-150 `T4`
resumes and its child tasks are confirmed to carry the required criteria, or
the owner explicitly waives the requirement.

---

## Related documents

- `docs/plan/tiger-style-adaptation.md` — the plan this ledger implements
- `docs/proposals/tiger-style-adaptation-evaluation.md` — R1–R13 source and
  D1–D3 resolutions
- `docs/plan/roadmap.md` — X26 cross-cutting item
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`
