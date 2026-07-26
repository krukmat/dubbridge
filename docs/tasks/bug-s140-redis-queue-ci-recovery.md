---
type: TaskList
title: "Bug: S-140 Redis queue CI recovery"
description: "Restore main-branch CI after the Redis-backed queue landing by splitting the recovery into local-first maintainability and dependency-policy slices plus a conditional low-band verification micro-patch and a final status-sync pass."
status: done
plan: docs/plan/s-140-redis-queue-ci-recovery.md
rri: 38
band: Moderate
effort: M
---

# BUG-S140-CI-01 - Redis queue CI recovery

> Surfaced in GitHub Actions on 2026-07-26 and partially reproduced locally the same day.
> Current failures:
> - Run `30202703817` (`a8e1281`) -> job `maintainability` -> `Reject generated-code bloat in backend/mobile diffs`
> - Run `30202703817` (`a8e1281`) -> job `deny` -> `Run dependency policy gate`

- **Task ID:** `BUG-S140-CI-01`
- **Status:** Done — local recovery completed on 2026-07-26; daily/task sync updated in the same pass
- **Effort:** M
- **Complexity:** Moderate
- **RRI:** combined scope was `42 -> Med-high`; split execution now targets `33`, `38`, `0-25 if T3a is triggered`, and `7`
- **Execution workflow:** run each slice independently with fresh RRI at execution time; `T1` and `T2` target the local-first Moderate lane, `T3a` is Gemma-eligible only when it is a true Low-band simple code patch, and `T3b` stays with the primary/orchestrator.

## Objective

Restore `main` CI after `S-140-T3c-i` by removing the maintainability violation
in `crates/jobs/src/lib.rs` and resolving the accompanying `cargo-deny` failure,
while preserving the Redis-backed queue behavior delivered by that task.

## Context

This is an operational blocker because `ci.yml` is red on `main` as of Sunday,
July 26, 2026. The failure is tied to the Redis queue landing, not to unrelated
ongoing local work.

The maintainability failure is already reproducible:

```bash
python3 scripts/check-maintainability.py --base 8ad36d4
```

Observed output:

- `crates/jobs/src/lib.rs: line repeated 9 times in added code; budget is 8: #[async_trait::async_trait]`

The dependency-policy failure is confirmed in GitHub but was not fully
reproduced offline. Local triage established:

- `cargo deny check bans licenses sources` passes
- offline advisory/yank inspection for the newly introduced Redis queue crates
  did not reveal an obvious failing advisory or yanked version
- the failure window still aligns with the `Cargo.toml` / `Cargo.lock` changes
  introduced by the Redis queue landing and must be captured exactly during
  local implementation before any fix is attempted

## Related documents

- `docs/plan/s-140-redis-queue-ci-recovery.md`
- `docs/daily/2026-07-26.md`
- `docs/plan/s-140-subtitle-generation.md`
- `docs/tasks/s-140-subtitle-generation.md`
- `.github/workflows/ci.yml`
- `scripts/check-maintainability.py`
- `deny.toml`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

## Split assessment

Yes: this bug is worth splitting.

- The original bundled scope scored `42 -> Med-high`, mostly because it mixed
  Rust queue code, dependency graph work, and status sync in one packet.
- The maintainability fix alone plans at `33 -> Moderate`.
- The dependency-policy fix alone plans at `38 -> Moderate`.
- The final docs/status sync plans at `7 -> Low`.

That means the real work can move into the local-dev route more cleanly:

- `T1` and `T2` fit the repository's **local-first Moderate** lane.
- `T3` remains a **Low** closure lane, but it is worth splitting by execution
  role:
  - `T3a` is only for an eligible simple code patch, so it may use Gemma.
  - `T3b` is the docs/status closeout, so it stays with the orchestrator.

The split does **not** remove workflow gating entirely: Moderate tasks still
need fresh presentation and approval at execution time under the repo rules.
What it does remove is the need to run the whole incident as one Med-high
bundle.

## T1 - Fix the maintainability regression in `crates/jobs`

- **Status:** [x] Done — maintainability green and jobs crate verification passed on 2026-07-26
- **Type:** development / maintainability recovery
- **Effort:** M
- **RRI:** 33 -> Moderate
- **Depends on:** none
- **Scope:** `crates/jobs/src/lib.rs`

### Happy paths considered

- **HP-1:** `make qa-maintainability` passes for the Redis queue push range after
  reducing or restructuring repeated boilerplate in `crates/jobs/src/lib.rs`
  without changing queue behavior.
- **HP-2:** Existing Redis-backed queue behavior still works after the fix,
  using the tests already introduced for `S-140-T3c-i`.

### Edge cases considered

- **EC-1:** The maintainability fix must not alter queue namespace selection,
  Redis connect timeout semantics, or enqueue return/error behavior.
- **EC-2:** Boilerplate reduction must not hide or remove the async trait
  contract in a way that changes compile-time behavior.

### Inputs

- GitHub Actions run `30202703817`, job `89795325567`
- local repro:
  - `python3 scripts/check-maintainability.py --base 8ad36d4`

### Outputs

- maintainability-clean queue diff
- explicit local evidence showing the gate is green

### Acceptance criteria

- The exact maintainability repro no longer reports the repeated
  `#[async_trait::async_trait]` violation.
- Redis queue tests and the most relevant worker/queue tests still pass.
- The implementation leaves queue behavior intact.

### Files expected to change

- `crates/jobs/src/lib.rs`

### Evidence to emit

- fresh `scripts/rri.py` output at execution time
- exact local repro command for `maintainability`
- passing verification commands
- final note linking the GitHub run and the local fix evidence

### Status artifacts affected

- `docs/daily/2026-07-26.md`
- this ledger

### Stop condition

Stop once `maintainability` is locally reproduced, fixed, and verified. Do not
expand into dependency-policy work; that belongs to `T2`.

### Agent handoff prompt

Fix only the `maintainability` failure from the July 26, 2026 `main` CI run.
Use `python3 scripts/check-maintainability.py --base 8ad36d4` as the starting
repro, remove the repeated `#[async_trait::async_trait]` pattern in
`crates/jobs/src/lib.rs`, and keep Redis queue behavior unchanged.

### Execution record (2026-07-26)

- **RRI at implementation time:** `32 -> Moderate`, recomputed with `python3 scripts/rri.py --auto-cc --T 3 --A 0 --X 1 --D 2 --K 2 --P 2 --touches crates/jobs/src/lib.rs --platform dubbridge`
- **Implementation route:** local-first Moderate lane evaluated first, but blocked by the 2026-07-22 target-file size gate because `crates/jobs/src/lib.rs` is 541 lines (>500). Escalated to direct primary-agent implementation in the same bounded scope.
- **Task-analysis review:** `qwen3.6:27b-q4_K_M` `.agent/peer-task-review-BUG-S140-CI-01-T1.json` - PASS after tightening the execution card with exact commands and source-only boundaries.
- **Code-solution review:** `qwen3.6:27b-q4_K_M` `.agent/peer-code-review-BUG-S140-CI-01-T1.json` - PASS
- **Change summary:** imported `async_trait` once and collapsed the three Redis queue implementations into a single macro-generated pattern so the repeated async-trait line count drops below the maintainability budget without changing queue behavior.

### Happy paths covered

- `HP-1`: [crates/jobs/src/lib.rs](/Users/matias/dubbridge/crates/jobs/src/lib.rs:193) now defines the Redis-backed queue family through one macro, and `python3 scripts/check-maintainability.py --base 8ad36d4` passed with no repeated-line violation.
- `HP-2`: [crates/jobs/src/lib.rs](/Users/matias/dubbridge/crates/jobs/src/lib.rs:382) still exercises the Redis enqueue/retrieve round trip and namespace isolation tests through the unchanged queue API.

### Edge cases covered

- `EC-1`: [crates/jobs/src/lib.rs](/Users/matias/dubbridge/crates/jobs/src/lib.rs:203) preserves the same `REDIS_CONNECT_TIMEOUT`, namespace binding, and `QueueError::Unavailable` mapping as before; the macro only centralizes the repeated implementation pattern.
- `EC-2`: [crates/jobs/src/lib.rs](/Users/matias/dubbridge/crates/jobs/src/lib.rs:55) keeps the async trait object contracts intact (`Arc<dyn ...Queue>` plus `#[async_trait]` on the traits/impls), so downstream object-safe usage does not change.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | maintainability gate passes after boilerplate reduction | `crates/jobs/src/lib.rs::tests::redis_enqueued_job_is_retrievable_from_its_namespace` plus `python3 scripts/check-maintainability.py --base 8ad36d4` | passed |
| HP-2 | Happy path | Redis-backed queue behavior still works after the refactor | `crates/jobs/src/lib.rs::tests::redis_enqueued_job_is_retrievable_from_its_namespace` | passed |
| EC-1 | Edge case | queue namespace selection, timeout, and enqueue error behavior stay fail-closed | `crates/jobs/src/lib.rs::tests::redis_queues_use_distinct_namespaces`; `crates/jobs/src/lib.rs::tests::redis_queue_fails_closed_on_unreachable_server`; `crates/jobs/src/lib.rs::tests::redis_queue_fails_closed_on_malformed_url` | passed |
| EC-2 | Edge case | async trait contract remains object-safe and callable through the queue abstractions | `crates/jobs/src/lib.rs::tests::in_memory_queue_records_jobs`; `crates/jobs/src/lib.rs::tests::in_memory_transcription_queue_records_jobs`; `crates/jobs/src/lib.rs::tests::in_memory_subtitle_queue_records_jobs` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-26`
- Statement: I verified every happy path and edge case defined for this task has unit test or gate evidence that replicates the expected behavior and that the maintainability regression is gone without changing queue semantics.
- Commands run: `python3 scripts/check-maintainability.py --base 8ad36d4`; `cargo test -p dubbridge-jobs --lib`

---

## T2 - Reproduce and fix the `cargo-deny` failure for the Redis queue graph

- **Status:** [x] Done — exact deny repro captured and lockfile fix verified on 2026-07-26
- **Type:** development / dependency-policy recovery
- **Effort:** M
- **RRI:** 38 -> Moderate
- **Depends on:** none
- **Scope:** `Cargo.toml`, `Cargo.lock`, `crates/jobs/Cargo.toml`, `apps/api/Cargo.toml`

### Happy paths considered

- **HP-1:** The exact local `cargo deny check` failure is captured from the
  Redis queue dependency delta before any graph edits are made.
- **HP-2:** `cargo deny check` passes after the graph is corrected, without
  weakening `deny.toml`, adding advisory ignores, or bypassing the policy gate.
- **HP-3:** The dependency fix preserves the Redis queue feature contract added
  by `S-140-T3c-i`.

### Edge cases considered

- **EC-1:** The failure may be environment-sensitive; if so, the task must
  record the precise local repro conditions instead of guessing at the cause.
- **EC-2:** The fix must not solve the gate by broadening allowed sources,
  muting advisories, or reverting the Redis queue feature wholesale.
- **EC-3:** If the graph change affects current S-140 follow-up notes, those
  docs must be synchronized in `T3`.

### Inputs

- GitHub Actions run `30202703817`, job `89795325516`
- local triage:
  - `cargo deny check bans licenses sources`
  - `cargo metadata --locked --offline --format-version 1 > /private/tmp/dubbridge-metadata.json`
  - offline index inspection already performed for `apalis-redis 0.7.4`,
    `redis 0.32.7`, `tokio-test 0.4.5`, `futures-timer 3.0.4`

### Outputs

- exact deny repro
- deny-clean dependency graph
- explicit note describing what actually caused the failure

### Acceptance criteria

- The exact local `cargo deny check` failure is captured and recorded before the fix.
- `cargo deny check` passes after the fix.
- The fix does not weaken dependency policy.
- Any queue-related compile/test verification needed by the graph change passes.

### Files expected to change

- `crates/jobs/Cargo.toml`
- `Cargo.toml`
- `Cargo.lock`
- `apps/api/Cargo.toml`
- `.github/workflows/ci.yml` only if local repro proves the gate wiring is wrong

### Evidence to emit

- fresh `scripts/rri.py` output at execution time
- exact local repro command for `cargo deny check`
- passing deny verification command
- concise explanation of the resolved policy violation

### Status artifacts affected

- `docs/daily/2026-07-26.md`
- this ledger
- `docs/plan/s-140-subtitle-generation.md` only if the chosen dependency fix changes the current T3c follow-up notes
- `docs/tasks/s-140-subtitle-generation.md` only if the chosen dependency fix changes the current T3c follow-up notes

### Stop condition

Stop once the dependency-policy gate is reproduced, fixed, and verified. Do not
pull maintainability cleanup into this slice; that belongs to `T1`.

### Agent handoff prompt

Reproduce the exact `cargo-deny` failure behind the July 26, 2026 `main` CI run
before editing the graph. The incident is already narrowed to the Redis queue
dependency delta (`apalis-redis 0.7.4`, `redis 0.32.7`, `tokio-test 0.4.5`,
`futures-timer 3.0.4`). Resolve it without weakening `deny.toml`, adding
advisory ignores, or reverting the Redis queue feature wholesale.

### Execution record (2026-07-26)

- **RRI at implementation time:** `34 -> Moderate`, recomputed with `python3 scripts/rri.py --C 0 --T 2 --A 0 --X 1 --D 3 --K 3 --P 3 --touches Cargo.toml --touches Cargo.lock --touches crates/jobs/Cargo.toml --touches apps/api/Cargo.toml --platform dubbridge`
- **Implementation route:** local-first Moderate lane evaluated first, but blocked by the 2026-07-22 target-file size gate because `Cargo.lock` is 4424 lines (>500). Escalated to direct primary-agent implementation in the same bounded scope.
- **Exact local repro before fix:** `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check`
- **Resolved policy violation:** the failing gate was not the new Redis queue crates themselves; `cargo-deny` failed on two yanked `spin` entries already present in the lockfile: `spin 0.9.8` via `multer -> axum` and `spin 0.10.0` via `crc-fast -> object_store`. The minimal fix was a lockfile-only patch update to `spin 0.9.9` and `spin 0.10.1`.
- **Task-analysis review:** `qwen3.6:27b-q4_K_M` `.agent/peer-task-review-BUG-S140-CI-01-T2.json` - PASS after tightening the execution card with exact verification commands and a patch-only update constraint.
- **Code-solution review:** `qwen3.6:27b-q4_K_M` `.agent/peer-code-review-BUG-S140-CI-01-T2.json` - PASS

### Happy paths covered

- `HP-1`: `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check` reproduced the exact yanked-crate failure before any graph edit.
- `HP-2`: [Cargo.lock](/Users/matias/dubbridge/Cargo.lock:259) now resolves the two `spin` entries to non-yanked patch releases, and `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check` passed with `advisories ok, bans ok, licenses ok, sources ok`.
- `HP-3`: `cargo check -p dubbridge-api -p dubbridge-worker-runner -p dubbridge-jobs` and `cargo test -p dubbridge-jobs --lib` passed after the lockfile update, so the Redis queue feature contract remained buildable.

### Edge cases covered

- `EC-1`: the repro was environment-sensitive only because the sandbox blocked writes/network; once rerun with a writable `CARGO_HOME` and network access, the exact failure surfaced cleanly and was recorded instead of guessed.
- `EC-2`: no change was made to `deny.toml`, no advisory ignore was added, and no direct dependency major/minor bump was introduced; the fix stayed at the lockfile patch level.
- `EC-3`: the chosen lockfile-only patch did not change the current S-140 follow-up notes, so no extra S-140 plan/task sync was required beyond this ledger and the daily.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | exact local deny failure is captured before the fix | `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check` (pre-fix artifact recorded in execution record) | passed |
| HP-2 | Happy path | deny passes after the graph correction | `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check` | passed |
| HP-3 | Happy path | queue feature contract remains buildable after the lockfile update | `cargo check -p dubbridge-api -p dubbridge-worker-runner -p dubbridge-jobs`; `crates/jobs/src/lib.rs::tests::redis_enqueued_job_is_retrievable_from_its_namespace` | passed |
| EC-1 | Edge case | environment-sensitive repro is captured exactly rather than guessed | `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check` with writable cache/network path | passed |
| EC-2 | Edge case | dependency policy is not weakened to clear the gate | `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check` | passed |
| EC-3 | Edge case | no extra S-140 follow-up docs are needed when the fix stays lockfile-only | this ledger execution record plus unchanged `docs/plan/s-140-subtitle-generation.md` / `docs/tasks/s-140-subtitle-generation.md` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-26`
- Statement: I verified the exact deny failure was captured before the fix, the lockfile-only patch update cleared the yanked-crate policy error without weakening repository policy, and the affected workspace members still build.
- Commands run: `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check`; `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo update -p spin@0.9.8 -p spin@0.10.0`; `cargo check -p dubbridge-api -p dubbridge-worker-runner -p dubbridge-jobs`; `cargo test -p dubbridge-jobs --lib`

---

## T3a - Conditional Low-band verification micro-patch

- **Status:** [x] Not triggered — `T1` and `T2` closed cleanly with no remaining Low-band code patch to delegate
- **Type:** development / verification tail
- **Effort:** S
- **RRI:** recompute at execution time; Gemma route allowed only at `0-25 -> Low`
- **Depends on:** T1, T2
- **Scope:** only a post-fix, in-scope simple code patch that is directly
  required to complete local verification for this incident

### Happy paths considered

- **HP-1:** A tiny post-fix code patch is identified after `T1`/`T2`, stays
  within the accepted fix surface, and can be delegated as a Low-band Gemma
  patch without reopening the incident scope.

### Edge cases considered

- **EC-1:** If the remaining work is docs-only, broad dependency work, or
  recomputes above `25`, `T3a` must not be delegated to Gemma.
- **EC-2:** If no eligible code patch exists, `T3a` is skipped and the closure
  moves directly to `T3b`.

### Acceptance criteria

- The slice is used only when a real simple code patch is still needed after
  `T1` and `T2`.
- Fresh `scripts/rri.py` output confirms the slice remains `RRI 0-25`.
- The patch stays inside the already-approved incident boundary and passes the
  required verification.

### Files expected to change

- resolved at execution time; must stay inside files already justified by `T1`
  or `T2`, or a directly related verification helper

### Evidence to emit

- fresh `scripts/rri.py` output at execution time
- Low-band Gemma delegation packet if triggered
- orchestrator acceptance judgment and verification commands

### Status artifacts affected

- this ledger
- `docs/daily/2026-07-26.md` if the micro-patch changes the incident note

### Stop condition

If no eligible simple code patch exists, or the slice recomputes above `25`,
do not delegate it to Gemma. Skip `T3a` and continue with `T3b`.

### Agent handoff prompt

Only if `T1` and `T2` leave a real Low-band simple code patch behind: apply the
smallest in-scope code-only fix needed to complete local verification, stay
inside the already accepted incident boundary, and stop without touching docs.

---

## T3b - Verify and sync status artifacts

- **Status:** [x] Done — daily + ledger synchronized on 2026-07-26
- **Type:** docs / verification sync
- **Effort:** S
- **RRI:** 7 -> Low
- **Depends on:** T1, T2, T3a if triggered
- **Execution owner:** primary/orchestrator direct

### Acceptance criteria

- Relevant local verification commands are recorded verbatim.
- `docs/daily/2026-07-26.md` reflects the incident as resolved or narrowed.
- Any S-140 follow-up notes affected by the chosen dependency fix are updated in
  the same change.

### Files expected to change

- `docs/daily/2026-07-26.md`
- this ledger
- `docs/plan/s-140-subtitle-generation.md` only if the chosen dependency fix changes the current T3c follow-up notes
- `docs/tasks/s-140-subtitle-generation.md` only if the chosen dependency fix changes the current T3c follow-up notes

### Evidence to emit

- exact local verification commands copied from `T1`/`T2` and `T3a` if triggered
- final incident note stating whether `main` is resolved or narrowed

### Status artifacts affected

- `docs/daily/2026-07-26.md`
- this ledger
- `docs/plan/s-140-subtitle-generation.md` only if the chosen dependency fix changes the current T3c follow-up notes
- `docs/tasks/s-140-subtitle-generation.md` only if the chosen dependency fix changes the current T3c follow-up notes

### Stop condition

Stop once the incident evidence and status docs are synchronized. Do not turn
`T3b` into a new implementation slice.

### Agent handoff prompt

As primary/orchestrator, record the final local verification evidence, update
the daily/task status, and synchronize any S-140 follow-up notes changed by the
accepted fixes. Do not delegate the docs-only closeout.

### Evidence to emit

- verification command list
- updated daily issue state
- any linked S-140 doc sync performed in the same change

### Execution record (2026-07-26)

- `T3a` was not triggered because no residual in-scope code patch remained after `T1` and `T2`.
- S-140 follow-up notes did not need additional sync because the accepted dependency fix stayed lockfile-only and did not change the Redis queue feature plan.
- Verification commands recorded for closure:
  - `python3 scripts/check-maintainability.py --base 8ad36d4`
  - `cargo test -p dubbridge-jobs --lib`
  - `CARGO_HOME=/private/tmp/dubbridge-cargo-home cargo deny check`
  - `cargo check -p dubbridge-api -p dubbridge-worker-runner -p dubbridge-jobs`
  - `make qa-docs`
