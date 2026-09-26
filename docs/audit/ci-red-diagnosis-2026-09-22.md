---
type: Audit
title: "CI red diagnosis — feature/p2p-mvp-core"
date: 2026-09-22
status: diagnosis_complete_fix_not_started
---

# CI red diagnosis — 2026-09-22

Scope: read-only diagnosis of the latest `ci` run on
`feature/p2p-mvp-core` before applying any dependency/workflow/coverage fixes.

Latest inspected run:
- workflow run: `35710807157`
- HEAD: `584b22c2515d5120419c605d606a4ec05858f609`
- conclusion: failure

The failures are isolated to three jobs. The Rust build/test/mobile gates are
otherwise green in the same run: `cargo-check`, `test`, `fmt`, `clippy`,
`release-build`, `mobile`, `qa-docs`, `roadmap-drift`,
`config-secrets`, `maintainability`, `python-complexity`, and
`peer-workflow-review` all passed.

## 1. deny — actionable dependency vulnerability

Observed failure:
- advisory: `RUSTSEC-2026-0285`
- affected lockfile version: `rustls 0.23.40`
- advisory: TLS 1.3 handshake messages can be accepted across encryption-level
  boundaries
- cargo-deny reports the solution as: upgrade `rustls` to `>= 0.23.45`

The current `Cargo.lock` resolves:
- `rustls 0.23.40`
- `hyper-rustls 0.27.9`
- `rustls-platform-verifier 0.7.0`
- `tokio-rustls 0.26.4`

Recommended minimal next action:
1. locally run `cargo update -p rustls --precise <safe-compatible-version>`
   using at least `0.23.45`;
2. inspect the complete lockfile diff rather than hand-editing checksums;
3. run `make qa-deny`, `cargo check --workspace --all-features`,
   `cargo test --workspace --all-features -- --test-threads=1`, and the
   mTLS/P2P focused suites;
4. score/review the dependency change under the normal workflow before commit.

Do **not** add the advisory to `deny.toml ignore` as the default fix: a patched
compatible release exists.

Fix not applied in this diagnostic task.

## 2. s3-integration — dead/inaccessible MinIO service image

The job fails before repository checkout/tests, during GitHub Actions service
initialization:

`docker pull minio/minio:edge-cicd@sha256:29e8e516...`

returns:

`pull access denied for minio/minio, repository does not exist or may require 'docker login'`

Therefore this is not an S3 adapter test failure. The test never starts.

The workflow currently pins:

`minio/minio:edge-cicd@sha256:29e8e51691d11e779468f275002779b221fd3902518d103e35c8a8bb2ef0f3ea`

Recommended minimal next action:
1. select a currently pullable public MinIO server image;
2. pin it immutably (release tag + digest), rather than switching to floating
   `latest`;
3. verify the existing health command and `minio/mc` client still work;
4. rerun only `s3-integration`, then the full workflow.

Do not weaken the mandatory S3 integration gate.

Fix not applied in this diagnostic task.

## 3. coverage — real threshold miss, not a test failure

All tests in the coverage run complete successfully; the gate fails only at:

`cargo llvm-cov --workspace --summary-only --fail-under-lines 90`

Latest total:
- line/region summary shown by llvm-cov: **88.28%** at the first coverage column
- required floor: **90%**

This is not solved by the mobile Jest coverage because this job is Rust
workspace coverage.

Largest low/zero-covered Rust hotspots in the latest report include:

- `apps/worker-runner/src/p2p_publication_runtime.rs` — 0%
- `crates/db/src/p2p_audience_repo.rs` — 0%
- `crates/db/src/p2p_envelope_repo.rs` — 0%
- `apps/api/src/routes/p2p_envelope.rs` — ~25%
- `apps/worker-runner/src/p2p_activation.rs` — ~26%
- `apps/api/src/routes/p2p_dashboard.rs` — ~29%
- `apps/api/src/routes/p2p_audience.rs` — ~30%
- `crates/connectors/src/p2p_availability.rs` — ~62%

A generated fixture binary
`crates/p2p/src/bin/package_build_and_materialize_fixture.rs` also reports 0%;
before adding tests merely to move the aggregate number, determine whether that
binary is legitimately executable production scope or should be explicitly
classified under the repository's existing coverage-ignore policy.

Recommended approach:
- do not lower `COVERAGE_MIN=90`;
- do not broadly expand `COVERAGE_IGNORE_REGEX` to make the number green;
- first add focused behavioral coverage to high-value P2/P3 runtime/repos/routes
  that currently have real product logic and very low coverage;
- treat legitimate tooling/fixture binaries separately and document any ignore
  decision explicitly.

Because several low-covered files are security/authorization/publication
boundaries, coverage remediation should be decomposed into small task-scoped
leaves rather than one broad "raise coverage" patch.

## Proposed execution order after P5.T3 evidence is frozen

1. `CI-D1 rustls` — lockfile dependency repair + security/mTLS regression tests.
2. `CI-D2 MinIO` — immutable image replacement + S3 integration rerun.
3. `CI-D3 coverage inventory` — classify executable/test-only files, then add
   focused tests in small leaves until >=90%.

These fixes are independent from the P5 Android certification result and should
not be used to reinterpret a P5 PASS/BLOCKED verdict.

## Closure state

`CI diagnosis = COMPLETE`

`CI repair = NOT STARTED`
