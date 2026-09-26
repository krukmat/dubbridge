---
type: Audit
title: "P1.A1c phase-2 finding disposition and remediation"
task: P1.A1c
phase: 2
date: 2026-08-30
---

# P1.A1c — Phase-2 finding disposition

The initial Phase-2 review (`mvp0-p2p-p1-a1c-phase2-review.json`) returned
advisory minor concerns. No finding was consensus across the three passes.

| Concern | Disposition | Evidence / action |
|---|---|---|
| A cleanup failure replaces the earlier open/ready failure. | Accepted by contract. | The RPC protocol intentionally emits one terminal, redacted error. If cleanup itself fails, `TRANSIENT_DRIVE_CLOSE_FAILED` is the accurate terminal state; no raw root-cause detail may cross the boundary and no receipt is emitted. |
| `store.close()` is not attempted after `drive.close()` fails. | Rejected. | `docs/audit/mvp0-p2p-p1-a1b-storage-contract.md` freezes ownership: once a `Hyperdrive` exists, it owns its `Corestore`; direct store close is allowed only after partial construction when no drive exists. |
| That ownership rule lacked direct regression proof. | Fixed. | Added `EC-A1 returns a close error without directly closing a drive-owned Corestore`, which makes `drive.close()` fail, proves the typed redacted close error, and asserts the store's direct `close()` spy was not called. |

Verification after the test addition:

```sh
cd mobile && npm run build:bare-worklet && npm run typecheck \
  && npm test -- --runInBand __tests__/p2p/runtime-protocol.test.ts \
  && npm run lint && npm run check:bare-worklet
```

Result: PASS (20 focused tests). The corrected packet is reviewed again before
closure.
