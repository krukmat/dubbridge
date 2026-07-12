---
type: TaskList
title: "Tasks: CI security dependency and scheduled push review"
status: completed
plan: docs/plan/ci-security-quick-xml.md
---

# Tasks: CI security dependency and scheduled push review

## CI-SEC-1 - Remove vulnerable quick-xml dependency chain

- **Status:** [x] Done
- **Type:** development / dependency security
- **Effort:** M
- **RRI:** 37 -> Moderate
- **Depends on:** none
- **Scope:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, and storage compatibility

### Objective

Move the locked object-store dependency graph to the verified official upstream
reference that admits `quick-xml >=0.41.0`, without suppressing RustSec advisories
or changing the ADR-006 storage contract.

### Happy paths considered

- **HP-1:** resolved dependency graph contains patched `quick-xml` and the deny
  policy remains strict -> structural contract tests and `cargo deny check` pass.
- **HP-2:** upgraded object-store API -> existing in-memory storage put/get,
  delete, missing-key, list, and file-streaming behavior remains passing.

### Edge cases considered

- **EC-1:** manifest or lockfile drifts from the approved upstream revision ->
  the structural security-contract test fails.
- **EC-2:** upstream API differs from `0.11` -> adapt only compile-proven call
  sites and preserve existing error mapping and key behavior.

### Acceptance Criteria

- The official tag resolves to the approved commit.
- `Cargo.lock` records `quick-xml >=0.41.0` and the pinned upstream revision.
- Neither RustSec advisory is ignored.
- Storage tests, workspace check, and `make qa-deny` pass.

### Reflection strategy

Required passes: 2 (`37` -> Moderate).

1. Verify dependency integrity, compile compatibility, and storage behavior.
2. Re-read the final graph and diff for advisory suppression, unintended source
   broadening, or ADR-006 boundary changes.

Task-analysis review: gemma `/tmp/dubbridge-ci-security-task-review-v4.json` - PASS

### Completion evidence

- Verified `refs/tags/v0.14.1-rc1` resolves to
  `c7316d29face118e7409eead0cda098f38589428` before editing.
- Pinned the full revision and constrained cargo-deny to the official upstream
  URL with `required-git-spec = "rev"`; advisory ignores remain empty.
- Updated the resolved graph to `object_store 0.14.1` / `quick-xml 0.41.0` and
  imported the compile-required `ObjectStoreExt` trait without changing storage
  behavior.

### Gemma Reviewer evidence

- Model: `gemma4:12b-mlx`
- Command: scoped packet piped to `python3 scripts/gemma-code-review.py --passes 3 --task-id CI-SEC-2026-0194-0195 --out /tmp/dubbridge-ci-security-gemma-review.json -`
- Passes run / usable: `3/0`
- Quorum: failed
- Aggregate status: `BLOCKED` (no parseable aggregate)
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Artifacts: no aggregate persisted; all three passes violated the tagged output contract
- Isolated adjudicator: `spawned` — trigger: zero usable Gemma passes
- disposition_divergence: `none`
- Primary-agent disposition: D14 found no blocking issues; structural evidence was added and re-reviewed

Code-solution review: d14 `.agent/d14-ci-security-review.json` - PASS

### Reflection log

Required passes: 2 (`39` post-implementation -> Moderate)

#### Pass 1

- **Draft verdict:** The dependency update removed the vulnerable XML release but
  exposed one compile-time upstream API change.
- **Critique findings:** `ObjectStore` operations moved behind `ObjectStoreExt`;
  the source authorization also needed to remain narrower than an organization-wide
  allowance.
- **Revisions applied:** Imported only `ObjectStoreExt`, pinned a full revision,
  required rev-qualified Git dependencies, and allowed only the official repository.

#### Pass 2

- **Draft verdict:** Runtime and policy gates passed, but closure initially relied
  on command evidence for pin integrity.
- **Critique findings:** The approved revision, patched lock entry, and empty
  advisory-ignore contract needed stable test evidence.
- **Revisions applied:** Added `scripts/dependency_security_test.py` and obtained a
  second D14 `PASS` over the final scoped solution.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | patched quick-xml lock entry and strict source/advisory policy remain present | `scripts/dependency_security_test.py::ObjectStoreSecurityContract::test_lock_uses_patched_quick_xml`, `scripts/dependency_security_test.py::ObjectStoreSecurityContract::test_source_policy_requires_rev_and_keeps_advisory_ignores_empty` | passed |
| HP-2 | Happy path | upgraded object-store retains put/get, delete, missing-key, list, and file-streaming behavior | `crates/storage/src/s3.rs::tests::put_get_round_trip`, `crates/storage/src/s3.rs::tests::delete_then_get_returns_not_found`, `crates/storage/src/s3.rs::tests::get_missing_key_returns_not_found`, `crates/storage/src/s3.rs::tests::list_keys_returns_sorted_canonical_ingest_keys`, `crates/storage/src/s3.rs::tests::put_file_round_trip` | passed |
| EC-1 | Edge case | manifest or lock drift from the approved revision is detected | `scripts/dependency_security_test.py::ObjectStoreSecurityContract::test_manifest_and_lock_pin_the_verified_upstream_revision` | passed |
| EC-2 | Edge case | upstream API adaptation preserves storage error and key behavior | `crates/storage/src/s3.rs::tests::get_missing_key_returns_not_found`, `crates/storage/src/s3.rs::tests::list_keys_returns_sorted_canonical_ingest_keys` | passed |

### Owner final verification

- Owner: `Codex`
- Date: 2026-07-12
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `git ls-remote https://github.com/apache/arrow-rs-object-store.git refs/tags/v0.14.1-rc1`; `python3 scripts/dependency_security_test.py`; `cargo test -p dubbridge-storage --all-features`; `make qa-local`; `cargo deny check`

## CI-SEC-2 - Audit scheduled CI runs with Push Reviewer

- **Status:** [x] Done
- **Type:** development / CI
- **Effort:** S
- **RRI:** 20 -> Low
- **Depends on:** CI-SEC-1
- **Scope:** `.github/workflows/push-review.yml`, `scripts/gemma_push_ops_test.py`

### Objective

Allow the existing advisory workflow to consume completed scheduled CI runs while
preserving its self-hosted, post-pipeline, non-authoritative boundary.

### Happy paths considered

- **HP-1:** completed push CI -> Push Reviewer job runs.
- **HP-2:** completed scheduled CI -> Push Reviewer job runs and receives the
  completed run ID, branch, and head SHA.

### Edge cases considered

- **EC-1:** completed pull-request CI -> Push Reviewer job remains excluded.
- **EC-2:** model/review failure -> artifacts and advisory summary remain visible
  without changing primary CI truth.

### Acceptance Criteria

- The job condition accepts exactly `push` or `schedule` source events.
- Structural tests cover both accepted events and pull-request exclusion.
- Existing self-hosted and advisory wiring remains covered.
- Collect-only replay of run `29141325673` records the scheduled run and failed
  dependency-policy evidence without model dispatch.

Task-analysis review: gemma `/tmp/dubbridge-ci-security-task-review-v4.json` - PASS

### Completion evidence

- Expanded the workflow-run condition to accept source event `push` or `schedule`.
- Added explicit structural coverage for both accepted events and pull-request
  exclusion.
- A collect-only replay of GitHub run `29141325673` recorded `event: schedule`,
  failed job `deny`, failed step `Run dependency policy gate`, and the available
  log containing both RustSec identifiers without dispatching the model.

### Gemma Reviewer evidence

- Model: `gemma4:12b-mlx`
- Command: same scoped three-pass code-review command recorded under CI-SEC-1
- Passes run / usable: `3/0`
- Quorum: failed
- Aggregate status: `BLOCKED` (no parseable aggregate)
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Artifacts: no aggregate persisted; all three passes violated the tagged output contract
- Isolated adjudicator: `spawned` — trigger: zero usable Gemma passes
- disposition_divergence: `none`
- Primary-agent disposition: D14 reviewed the final workflow and test diff and returned PASS

Code-solution review: d14 `.agent/d14-ci-security-review.json` - PASS

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | completed push CI is accepted by the workflow condition | `scripts/gemma_push_ops_test.py::PushReviewOpsWiring::test_workflow_audits_push_and_schedule_but_not_pull_requests` | passed |
| HP-2 | Happy path | completed scheduled CI is accepted and run context remains wired | `scripts/gemma_push_ops_test.py::PushReviewOpsWiring::test_workflow_audits_push_and_schedule_but_not_pull_requests`, `scripts/gemma_push_ops_test.py::PushReviewOpsWiring::test_workflow_is_post_pipeline_self_hosted_and_advisory` | passed |
| EC-1 | Edge case | pull-request CI remains excluded | `scripts/gemma_push_ops_test.py::PushReviewOpsWiring::test_workflow_audits_push_and_schedule_but_not_pull_requests` | passed |
| EC-2 | Edge case | review failure remains separate from primary CI and artifacts/summary run unconditionally | `scripts/gemma_push_ops_test.py::PushReviewOpsWiring::test_workflow_is_post_pipeline_self_hosted_and_advisory` | passed |

### Owner final verification

- Owner: `Codex`
- Date: 2026-07-12
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/gemma_push_ops_test.py`; `python3 scripts/gemma-push-review.py --run-id 29141325673 --collect-only --out-dir /tmp/dubbridge-scheduled-replay.UvwEzR`; `DUBBRIDGE_SKIP_GEMMA_PUSH_REVIEW=1 make qa-gemma-push-review`
