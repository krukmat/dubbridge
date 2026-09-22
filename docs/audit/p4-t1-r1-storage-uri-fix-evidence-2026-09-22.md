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

Owner decision 2026-09-22: declare `bare-url@^2.5.2` as a direct mobile dependency.
`mobile/package.json` and the lockfile now declare it explicitly; the existing lock
resolution remains `bare-url@2.5.2`, so this changes dependency ownership only, not
the resolved runtime version.

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

## Direct dependency follow-up — 2026-09-22

Owner approved declaring `bare-url@^2.5.2` directly in the mobile package.
The lockfile continues to resolve `bare-url@2.5.2`; no runtime version changed.

The dependency ownership change altered the deterministic packed output, so the
committed Bare worklet was regenerated from a clean GitHub Actions checkout.

- dependency declaration: `2b2009e8dcc03b2178a560c0ca9401ee08e23944`
- Rust fmt/clippy cleanup unrelated to this repair: `6debd60ad7fcef2ec3c2427910930ba2eaf91fbf`
- regenerated worklet: `929addbebb8e4b12dee3f27eff6fd0fde0a92100`
- committed worklet blob: `3280493d917fe7e39a0f9f20471f02bba3693b3a`
- regenerated/check sha256:
  `dcfd5a437ed4839f886ae3ab495775c2ca1a1aa37072ec8a3ff057b8e2ec7a24`
- `npm run build:bare-worklet`: PASS
- `npm run check:bare-worklet`: PASS
- temporary regeneration workflow removed in the same worklet commit

The Android fresh-invitation rerun remains the only device gate for this repair;
this follow-up still does not mark P5.T3 or aggregate P5 PASS.

