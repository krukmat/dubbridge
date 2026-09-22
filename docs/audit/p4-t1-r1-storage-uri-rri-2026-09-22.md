---
type: Audit
title: "P4.T1-r1 product storage URI repair RRI"
date: 2026-09-22
task: P4.T1-r1
---

# P4.T1-r1 — execution RRI

Exact implementation/evidence surface:
- `mobile/src/p2p/runtime/product-package-runtime.ts`
- `mobile/src/p2p/runtime/worklet.bundle.js` (generated artifact; regeneration pending execution environment)
- `mobile/__tests__/p2p/product-storage-path.test.ts`
- `docs/audit/mvp0-p2p-p1-a1b-storage-contract.md`

No package/lockfile change is included; `bare-url` remains the already-packaged
transitive dependency for this repair.

| Variable | Score | Rationale |
|---|---:|---|
| C | 2 | one conversion boundary plus fail-closed validation |
| F | 2 | four implementation/generated/evidence paths |
| D | 2 | persistent product storage boundary; no new state machine/schema |
| T | 1 | focused executable storage-boundary tests added |
| A | 0 | defect mechanism and desired conversion point are confirmed |
| K | 2 | Corestore/Bare URI boundary and generated worklet coupling |
| P | 2 | blocks P4/P5 product path but does not change auth/crypto/schema |
| X | 1 | localized mobile worklet context |

Derived ADR-045 technical profile:
- L=2, I=2, Q=2, V=1
- bottleneck=2
- ICI=50
- ICI band input=55
- risk/domain input=15
- penalties: none

**Final RRI: 55 — Med-high.**

Owner approval is the active 2026-09-22 instruction to execute the filesystem-path
repair between P4.T1 review and the P5.T3 rerun.

Acceptance remains the task-ledger contract: decode the scoped `file:` URI exactly
once immediately before Corestore; reject invalid authority, encoded slash, NUL
and non-file schemes before product dependencies are constructed.
