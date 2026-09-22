---
type: Audit
title: "P4.T1-r1 product storage URI filesystem-path repair evidence"
date: 2026-09-22
task: P4.T1-r1
status: implementation_verified_device_pending
---

# P4.T1-r1 — product storage URI filesystem-path repair

## Result

**IMPLEMENTED + CI VERIFIED. Device confirmation remains pending as part of the
P5.T3 Android rerun.**

The confirmed Android defect was that the product worklet passed a scoped
`file:` URI directly to Corestore, which interprets string storage as a
filesystem path. The observed device failure was `ENOENT stat "file:"`.

The repair keeps the host-to-worklet URI contract unchanged and converts the
scoped URI exactly once with `bare-url.fileURLToPath` immediately before
Corestore construction. Conversion failures are mapped to
`PRODUCT_STORAGE_CONFIG_INVALID` before Corestore/Hyperdrive/Hyperswarm are
constructed.

## Implementation

- `e44d00f497fe0f7128ce3a31a797b13840dafb2c` —
  `fix(p4): convert product storage URI to filesystem path`
- `469e45c55652a431382138d00aa3f406b2bccc21` —
  focused storage-boundary tests
- `575677e735c7bd3830a3d1e85652811632f6b31f` —
  regenerated committed Bare worklet
- committed worklet blob: `759ab5955d3ed29b4d4a19e1be0dd807ab6c899d`
- generated worklet sha256:
  `2f1f79cdf1d62a2b7cd8fafd3819b4ccb5039b71b519fbf7e954d1654a7bfd1b`

No `mobile/package.json` or lockfile change was made. `bare-url` remains the
already-packaged transitive dependency for this narrowly scoped repair.

## RED -> GREEN proof

GitHub Actions run `35736737748` executed the current focused test against the
exact pre-fix source `ed55050e6aaba916f3f4ce26d387a0aff74204ae`, then restored
the current source and reran it.

RED:
- suite: **FAIL**
- tests: **6 failed / 1 passed / 7 total**
- expected: the pre-fix source violates the new storage-path contract

GREEN:
- suite: **PASS**
- tests: **7/7 PASS**

The focused test proves:
- a `%20` root opens real Corestore/Hyperdrive at the decoded absolute account path;
- no relative `file:` tree is created in the process cwd;
- account scopes map to distinct decoded filesystem directories;
- non-empty authority, encoded slash, NUL and non-`file:` schemes fail closed
  before product dependencies are constructed;
- `%2520` is decoded once to a literal `%20`, not twice.

## Full mobile gate

GitHub Actions run `35735766081`, source/test HEAD
`469e45c55652a431382138d00aa3f406b2bccc21`:

- mobile job: **PASS**
- strict typecheck: PASS
- lint: PASS
- Jest: **61/61 suites PASS**
- Jest: **441/441 tests PASS**
- `product-storage-path.test.ts`: PASS
- `p2p.product-sync.test.ts`: PASS
- `p5-device-certification.test.ts`: PASS
- `p2p.provider-account-change.test.tsx`: PASS

Rust-only fmt/clippy failures in that workflow are outside this TypeScript repair
and do not execute the mobile path.

## Generated artifact drift check

One-shot clean-check run `35736354778` checked the committed branch from a
fresh checkout:

`Bare worklet bundle is current: sha256=2f1f79cdf1d62a2b7cd8fafd3819b4ccb5039b71b519fbf7e954d1654a7bfd1b`

The temporary generation/check workflows removed themselves after use; no helper
workflow remains in the repository.

## Remaining gate

The acceptance item tied to the actual Android runtime remains intentionally
open: rerun P5.T3 from a fresh invitation on the final branch revision and prove
that SYNC no longer terminates with `ENOENT stat "file:"`. Record whatever next
stage is observed; this artifact does not infer a device PASS from CI.

The development-only P1 `transient-drive.ts` URI/path issue remains a separate
named residual and was not changed.
