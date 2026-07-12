---
type: Evaluation
title: "ADR-036 Benchmark Corpus — Stage 1 task cards"
status: in-progress
---

# ADR-036 Benchmark Corpus

## Purpose

15–20 task cards derived from this repository's own git history, for T7 to
run through the local-agent runner (`scripts/local-agent/run_local_task.py`,
built in T6a–T6d). Each card is scoped to be executable **without further
interpretation**: allowed paths, an HP/EC acceptance-test contract (ADR-036
§4 shape), verification commands, and the original human/Claude-authored
commit as the reference answer. The runner's transcript and pass/fail result
for each card become the raw evidence T8 uses for the Stage 1 go/no-go
verdict — they do not by themselves constitute the verdict.

Every reference commit below was verified to exist in this repository's
history via `git show <sha>` before being included in this corpus.

## Category coverage

| Category | Required | Cards |
|---|---|---|
| Rust | ≥2 | RC-01, RC-02, RC-03, RC-04 |
| Mobile | ≥2 | MC-01, MC-02, MC-03, MC-04 |
| CI | ≥1 | CC-01, CC-02, CC-03 |
| Docs | ≥1 | DC-01, DC-02 |
| Refactor | ≥2 | RF-01, RF-02, RF-03 |

Total: 16 cards.

## Constraints (all cards)

- No card requires production credentials.
- No card requires network access beyond `OLLAMA_HOST` (the runner's own
  model endpoint).
- All verify commands are drawn from `make qa-*` targets or `cargo`/`npm`
  invocations already documented in `CLAUDE.md`.

---

## RC-01 — Replace `.expect()` panic with a typed error

- **Reference commit:** `19ba29c` — "fix(ingestion): replace expect() with
  ok_or() for rights_basis in finalize_ingestion_core"
- **Category:** Rust
- **Allowed paths:** `crates/ingestion/src/lib.rs`
- **Scope:** `finalize_ingestion_core` calls
  `pending.rights_basis.clone().expect("validated rights_basis")`. This
  panics if the invariant (validated upstream) is ever violated across two
  distinct call sites. Replace the `.expect()` with `.ok_or(...)?` returning
  `IngestionServiceError::Validation(IngestionError::MissingRightsBasis)`,
  preserving identical behavior for all current (valid) callers.
- **HP-1:** existing passing-path tests for `finalize_ingestion_core`
  continue to pass unchanged (no behavior change for valid input).
- **EC-1:** a test that constructs a pending record with `rights_basis: None`
  and asserts `finalize_ingestion_core` returns
  `Err(IngestionServiceError::Validation(IngestionError::MissingRightsBasis))`
  instead of panicking.
- **Verify commands:** `cargo test -p ingestion`; `cargo clippy -p ingestion -- -D warnings`.

## RC-02 — Cap unbounded body read in a proxy handler

- **Reference commit:** `2ec44b1` — "fix(gateway): cap
  public_proxy_handler body to 16 KiB"
- **Category:** Rust
- **Allowed paths:** `apps/gateway/src/proxy.rs`
- **Scope:** `public_proxy_handler` calls `to_bytes(body, usize::MAX)`,
  allowing unbounded memory consumption from an oversized request body on a
  GET-only playback relay route. Cap the read to 16 KiB and return
  `413 Payload Too Large` (not `502 Bad Gateway`) when the cap is exceeded.
- **HP-1:** a request with a body under 16 KiB is proxied through unchanged
  (existing behavior preserved).
- **EC-1:** a request with a body over 16 KiB is rejected with HTTP 413, not
  502 and not an unbounded read.
- **Verify commands:** `cargo test -p gateway`; `cargo clippy -p gateway -- -D warnings`.

## RC-03 — Add unit tests to close a coverage gap

- **Reference commit:** `130ff8d` — "test(s-130-t3): add unit tests for
  providers crate to meet coverage gate"
- **Category:** Rust
- **Allowed paths:** `crates/providers/src/lib.rs` (test module only — no
  production code changes)
- **Scope:** `crates/providers` has line coverage far below the 90% gate.
  Add unit tests covering: `StubAsrWorkerClient` (ok and err paths),
  `AsrError` `Display` formatting, `SubprocessAsrWorkerClient` spawn
  failure, successful JSON output parsing, non-zero exit with a JSON error
  payload, timeout with process reap, and the default-timeout constant.
- **HP-1:** `cargo llvm-cov` reports `crates/providers` line coverage ≥90%
  after the new tests are added.
- **EC-1:** every new test is deterministic (no flaky timing assumptions —
  see RC-04 for the specific pitfall to avoid).
- **Verify commands:** `cargo test -p providers`; `make qa-coverage`
  (scoped check: `crates/providers` coverage ≥90%).

## RC-04 — Fix a race condition in a subprocess test

- **Reference commit:** `5fd1c15` — "fix(providers): drain stdin in all
  subprocess echo tests"
- **Category:** Rust
- **Allowed paths:** `crates/providers/src/lib.rs`
- **Scope:** A subprocess-based test (`subprocess_client_parses_valid_output_json`)
  writes to a child process's stdin without draining the read side first;
  `sh` ignores stdin, so `write_all` races the process's exit and
  intermittently fails with `BrokenPipe` on CI. Apply the same drain-before-write
  fix already used in the sibling non-zero-exit test in the same file.
- **HP-1:** the test passes deterministically across at least 20 consecutive
  local runs (`cargo test -p providers -- --test-threads=1 --nocapture`
  looped 20×) with zero `BrokenPipe` failures.
- **EC-1:** the fix does not change the test's assertions about parsed
  output — only the write/read ordering around the child process.
- **Verify commands:** `cargo test -p providers`.

---

## MC-01 — Extract a component to satisfy the maintainability gate

- **Reference commit:** `24f81b1` — "refactor(mobile): extract
  StatusFilterChips to satisfy maintainability gate"
- **Category:** Mobile
- **Allowed paths:** `mobile/src/screens/AssetListScreen.tsx`,
  new file `mobile/src/components/StatusFilterChips.tsx`
- **Scope:** A single hunk inside `AssetListScreen.tsx`'s filter bar exceeds
  the maintainability gate's contiguous-added-lines budget. Extract the
  status-filter `ScrollView` block into a new `StatusFilterChips` component,
  preserving all existing `testID`s and behavior, so both the extraction
  site and the new file stay under the budget.
- **HP-1:** existing `AssetListScreen` tests pass unchanged (same rendered
  output, same `testID`s, same filter behavior).
- **EC-1:** `make qa-maintainability` passes on the diff (no oversized
  contiguous block remains).
- **Verify commands:** `cd mobile && npm test -- AssetListScreen`; `make qa-maintainability`.

## MC-02 — Introduce an accessible color token and apply it

- **Reference commit:** `b375cb4` — "fix(a11y): add primaryStrong token and
  fix WCAG AA on small-text label uses"
- **Category:** Mobile
- **Allowed paths:** `mobile/src/theme/tokens.ts`,
  `mobile/src/components/ScreenHeader.tsx`,
  `mobile/src/screens/AssetDetailScreen.tsx`,
  `mobile/src/screens/ConfigErrorScreen.tsx`,
  `mobile/src/screens/OrganizationMembersScreen.tsx`,
  `mobile/src/screens/ReviewDetailScreen.tsx`,
  `mobile/src/screens/UploadScreen.tsx`,
  `mobile/__tests__/theme.tokens.test.js`
- **Scope:** `color.primary` (#E50914) yields 3.84:1 contrast on the app
  canvas — acceptable for large UI (buttons/icons, 3:1 threshold) but below
  the 4.5:1 WCAG AA threshold required for small text. Add a
  `color.primaryStrong` token (#FF3333, 5.06:1) following the existing
  `*Strong` token pattern (`successStrong`, `warningStrong`, `infoStrong`),
  and switch every small-text (`type.label`) use of `color.primary` to
  `color.primaryStrong` across the listed screens/component.
- **HP-1:** a contrast-ratio unit test asserts `primaryStrong` on canvas
  meets ≥4.5:1.
- **EC-1:** a contrast-ratio unit test asserts the original `primary` token
  still meets the ≥3:1 large-UI threshold (i.e., the fix does not regress
  the large-UI use case by moving `primary` itself).
- **Verify commands:** `cd mobile && npm test -- theme.tokens`;
  `npm run typecheck`.

## MC-03 — Add a static check script for a design-system rule

- **Reference commit:** `ccb8d53` — "fix(a11y): enforce primaryStrong for
  label text via static check"
- **Category:** Mobile
- **Allowed paths:** new file `scripts/check-primary-label-usage.py`,
  `Makefile`
- **Scope:** Add a static-analysis script that scans `mobile/src` for any
  style object combining `...type.label` with `color: color.primary`
  (the violation MC-02 fixed manually) and wire it into `qa-mobile` so
  future regressions are caught automatically at CI/pre-push time, not only
  by manual review.
- **HP-1:** running the script against the current (post-MC-02) `mobile/src`
  tree exits 0 (no violations).
- **EC-1:** a synthetic fixture file containing the violation pattern
  causes the script to exit non-zero with a message naming the offending
  file.
- **Verify commands:** `python3 scripts/check-primary-label-usage.py`;
  `make qa-mobile`.

## MC-04 — Decompose functions to pass a max-lines-per-function gate

- **Reference commit:** `17453ae` — "fix(mobile): decompose
  AssetListScreen and ReviewDetailScreen under max-lines-per-function gate"
- **Category:** Mobile
- **Allowed paths:** `mobile/src/screens/AssetListScreen.tsx`,
  `mobile/src/screens/ReviewDetailScreen.tsx`, and any new hook/component
  files the extraction requires (e.g. a `useAssetListFilter` hook, a
  `ReviewPublicationSection` component)
- **Scope:** Two functions exceed the mobile gate's max-lines-per-function
  threshold. Extract cohesive pieces of each (a filter-state hook out of
  `AssetListScreen`; a publication-related JSX section out of
  `ReviewDetailScreen`) so both functions drop under the threshold, with all
  existing behavior, `testID`s, and test coverage preserved.
- **HP-1:** all pre-existing tests for both screens pass unchanged.
- **EC-1:** `make qa-mobile` (which includes the max-lines-per-function
  check) passes on the resulting diff.
- **Verify commands:** `cd mobile && npm run typecheck && npm run lint && npm test`; `make qa-mobile`.

---

## CC-01 — Fix an over-eager exit code in a review gate

- **Reference commit:** `ed262c1` — "fix(peer-review): exit 0 for FINDINGS
  verdict (advisory), exit 1 only for BLOCKED"
- **Category:** CI
- **Allowed paths:** `scripts/peer-workflow-review.py`
- **Scope:** The pre-push hook blocks on any non-`PASS` verdict from
  `peer-workflow-review.py`, including advisory `FINDINGS` verdicts that
  should not stop a push. Per the review contract, only `BLOCKED` is a hard
  stop. Fix the exit-code mapping: `0` for `PASS` or `FINDINGS`, `1` for
  `BLOCKED`, `2` for a script error.
- **HP-1:** a test asserting a `FINDINGS`-verdict input returns exit 0.
- **EC-1:** a test asserting a `BLOCKED`-verdict input still returns exit 1
  (the fix must not weaken the actual blocking case).
- **Verify commands:** `python3 -m unittest scripts/peer_workflow_review_test.py`
  (or the project's equivalent test entry point for this script).

## CC-02 — Break an infinite CI trigger loop with `[skip ci]`

- **Reference commit:** `378b8ac` — "fix(push-review): add [skip ci] to bot
  commits to break trigger loop"
- **Category:** CI
- **Allowed paths:** `scripts/push_review_commit.py`
- **Scope:** The push-review bot's own commits re-trigger CI, which
  re-triggers push-review, creating an infinite bot-commit loop. Append
  `[skip ci]` to the bot's commit message so its own commits do not
  re-trigger the workflow that produced them.
- **HP-1:** a test asserting the generated commit message string ends with
  `[skip ci]`.
- **EC-1:** the fix touches only the commit-message string — the
  `git commit`/`git push` invocation shape is unchanged.
- **Verify commands:** `python3 -m unittest` (the test module covering
  `push_review_commit.py`, if present) or a direct inspection assertion per
  the HP-1 contract above.

## CC-03 — Extend a pre-push hook to cover a previously-skipped path

- **Reference commit:** `ef6e2a2` — "fix(hooks): run qa-maintainability on
  mobile-only pushes"
- **Category:** CI
- **Allowed paths:** `.githooks/pre-push`
- **Scope:** The pre-push hook's change-detection only recognized Rust- and
  docs-impacting paths; a push touching only `mobile/` files skipped QA
  entirely, letting maintainability violations reach CI undetected. Add a
  `MOBILE_CHANGED` detection branch (matching `^mobile/`) and run
  `make qa-maintainability` when it fires, alongside the existing Rust/docs
  branches.
- **HP-1:** a simulated push whose changed-file list contains only a
  `mobile/` path triggers the `make qa-maintainability` branch (verified via
  the hook's own echo/trace output, since it's a shell script rather than a
  unit-testable module).
- **EC-1:** a simulated push whose changed-file list contains neither Rust-,
  docs-, nor mobile-impacting paths still skips QA entirely (the "nothing
  relevant changed" early-exit is preserved).
- **Verify commands:** manual invocation of `.githooks/pre-push` with a
  crafted `CHANGED_FILES`/`DIFF_RANGE` environment matching each scenario
  above (no live git push required).

---

## DC-01 — Rewrite a top-level README for a new audience

- **Reference commit:** `0a85a86` — "docs: rewrite README for general
  audience, add DEVELOPMENT_REFERENCE"
- **Category:** Docs
- **Allowed paths:** `README.md`, new file `DEVELOPMENT_REFERENCE.md`
- **Scope:** Rewrite `README.md` for two audiences (casual readers and
  creators who want to share work in other languages): lead with a plain
  analogy, show screenshots early, avoid internal jargon (e.g. write "bring
  in" instead of "ingest", "video" instead of "asset"). Move the previous
  developer-oriented content (architecture, workspace map, ADR index,
  roadmap, full setup) into a new `DEVELOPMENT_REFERENCE.md` so the README
  itself stays audience-appropriate.
- **HP-1:** `README.md` contains no occurrences of the internal jargon terms
  named above in user-facing prose (a simple grep-based check).
- **EC-1:** every fact moved out of the old README (ADR count, workspace
  structure) is still present and accurate in `DEVELOPMENT_REFERENCE.md` —
  no information silently dropped.
- **Verify commands:** `make qa-docs` (OKF frontmatter + doc-consistency
  checks, since both are files under repo root governed by the same
  frontmatter contract as `docs/`).

## DC-02 — Expand an existing doc/script pair with a small factual addition

- **Reference commit:** `b93b259` — "Expand agent preflight context docs"
- **Category:** Docs
- **Allowed paths:** `scripts/agent-preflight.py`,
  `scripts/agent_preflight_test.py`
- **Scope:** The preflight summary's "before implementation" guidance was
  too terse ("Analyze affected files and governing docs."). Expand it to
  explicitly name the categories of governing docs to check
  (`docs/architecture.md`, ADRs, `docs/plan/roadmap.md`, the slice
  plan/task ledger, BDD/product docs, relevant policies/configs), and add
  the same items to the sentinel-payload `requirements` list.
  This is deliberately the smallest card in the corpus (single-file prose
  edit plus a matching test update) — a floor case for the runner's
  smallest unit of useful work.
- **HP-1:** `agent_preflight.preflight_summary()` contains the string
  `"docs/architecture.md"`.
- **EC-1:** the existing preflight tests (asserting `AGENT_WORKFLOW_GUIDE.md`,
  `scripts/rri.py`, `DESIGN.md` presence) still pass unchanged.
- **Verify commands:** `python3 -m unittest scripts/agent_preflight_test.py`.

---

## RF-01 — Reduce cognitive complexity across several functions to pass a gate

- **Reference commit:** `bc8450d` — "refactor(rust): reduce cognitive
  complexity to pass CC≤15 gate"
- **Category:** Refactor
- **Allowed paths:** `apps/api/src/cleanup.rs`,
  `apps/api/src/routes/ingestion.rs`, `apps/worker-runner/src/main.rs`,
  `crates/ingestion/src/lib.rs`, `crates/domain/src/audit.rs`
- **Scope:** A newly-introduced cognitive-complexity gate (CC≤15) fails on
  `finalize_ingestion_core`, `enqueue_transcription_if_ready`,
  `cleanup_expired_ingestions`, `run_ingest_reconciliation`, and
  `create_ingestion`. Extract helper functions from each to bring cognitive
  complexity under the threshold; where a tracing-macro field expansion
  causes a false overcount, apply a targeted, justified
  `#[allow(clippy::cognitive_complexity)]` instead of a further extraction
  that would hurt readability.
- **HP-1:** `cargo clippy --all-targets -- -D warnings` passes with the
  cognitive-complexity gate enabled (no lint suppressed except the
  explicitly justified one).
- **EC-1:** all pre-existing tests for the touched functions pass unchanged
  (behavior-preserving refactor — no functional diff).
- **Verify commands:** `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo test --workspace`.

## RF-02 — Extract shared components/hooks to eliminate duplication and pass the maintainability gate

- **Reference commit:** `0688f61` — "fix(ci): pass maintainability gate for
  adr-034 commit"
- **Category:** Refactor
- **Allowed paths:** `mobile/src/components/PlaybackStateView.tsx` (new),
  `mobile/src/hooks/usePlaybackLoader.ts` (new), `mobile/src/api/playback.ts`,
  `mobile/src/screens/AssetDetailScreen.tsx`,
  `mobile/src/screens/ReviewDetailScreen.tsx`,
  `mobile/__tests__/asset.screens.test.tsx`,
  `mobile/__tests__/ReviewDetailScreen.test.tsx`
- **Scope:** `AssetDetailScreen` and `ReviewDetailScreen` duplicate playback
  JSX and loading logic, tripping the maintainability gate's
  duplicated-block detector. Extract a shared `PlaybackStateView` component
  and a `usePlaybackLoader` hook; add `resolvePlaybackErrorMessage` to
  `playback.ts`; consolidate repeated test render boilerplate behind shared
  `renderDetail`/`mockGetAsset` test helpers so the test files also drop
  below the gate's repeated-lines threshold.
- **HP-1:** all pre-existing tests for both screens pass unchanged (same
  rendered output for loading/error/success playback states).
- **EC-1:** `make qa-maintainability` passes on the diff (no duplicated
  block or oversized repeated-lines violation remains).
- **Verify commands:** `cd mobile && npm test`; `make qa-maintainability`.

## RF-03 — Narrow an overly broad gate-failure condition

- **Reference commit:** `6841b37` — "fix(gemma): only fail on blocking/major
  findings, not minor/nit"
- **Category:** Refactor
- **Allowed paths:** `scripts/parse-review-findings.py`
- **Scope:** The findings parser exits non-zero whenever *any* finding is
  present, including purely informational `minor`/`nit` severities that
  should not block a push. Change the failure condition to non-zero only
  when a `blocking` or `major` finding exists; `minor`/`nit` findings are
  still printed for visibility but no longer force a non-zero exit.
- **HP-1:** a synthetic findings list containing only `minor`/`nit`
  severities produces exit 0.
- **EC-1:** a synthetic findings list containing at least one `blocking` or
  `major` finding (mixed with minor/nit) still produces exit 1 — the
  narrowing must not silence a real blocker.
- **Verify commands:** `python3 -m unittest` (the test module covering
  `parse-review-findings.py`, if present) or direct invocation against
  fixture JSON matching the HP-1/EC-1 scenarios.
