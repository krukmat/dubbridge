---
type: Audit
title: "P5.T3-r1: cold-drive discovery repair evidence"
status: closed
---

# P5.T3-r1 — retained evidence and limits

The repair is already in `3f3ffbd`. This record recovers the task-local RRI
and verifies its regression. The standing MVP0-P2P owner review exception
applies; this evidence record is therefore closed. It does not claim P5-CERT
or Android validation. P5.T3's checklist ownership was transferred to T7p/P7.T2;
no emulator or local-topology work is scheduled.

## Scope and RRI

Files in the repair: `mobile/src/p2p/runtime/product-package-runtime.ts`,
`mobile/__tests__/p2p/product-package-discovery.test.ts`,
`mobile/__tests__/p2p/product-storage-path.test.ts`, and the generated
`mobile/src/p2p/runtime/worklet.bundle.js`.

The original session retained `.agent/p5-t3-recovery/repair-rri.json` and
`repair-scope.md` before implementation, but did not publish them with the
repair. The score was revalidated with:

```sh
python3 scripts/rri.py --C 1 --F 2 --D 3 --T 2 --A 0 --K 3 --P 2 --X 2 --platform dubbridge --json
```

Result: **RRI 70 / Complex**, ICI 75, risk band 20, no penalties. The parent
outcome combines native mobile execution, peer discovery, local package state
and physical evidence; test/provisioning leaves do not reduce its review gate.
This is a record of the existing score, not retroactive review approval.

Task-analysis review: REVIEW-OVERRIDE —
`docs/audit/mvp0-p2p-review-exception.md`.

Code-solution review: REVIEW-OVERRIDE —
`docs/audit/mvp0-p2p-review-exception.md`.

The owner exception covers the P0–P7 MVP0-P2P work. Formal physical behavioral
certification is deliberately not claimed here; it belongs to T7p/P7.T2.

## Reproduced RED and current GREEN

Verification checkout: `8a2310c51448f15237628ba85abac860aec51e0d`.
The worktree was clean before this documentation update. No production source
was modified to reproduce RED: an ignored fixture copied the current test and
its dependencies and loaded the runtime source from `3f3ffbd^`.

- **RED:** the test `reads a cold manifest after a peer connects later than the
  DHT announcement` fails against the previous runtime with
  `RuntimeProtocolError: Product package file is unavailable`.
  One expected failure; the three unrelated failure-mode cases were skipped.
  Local output: `.agent/p5-t3-recovery/red-reproduction.log`.
- **GREEN:** the following current-source command passed **4 suites / 34 tests**:

```sh
cd mobile
node node_modules/jest/bin/jest.js --runInBand \
  __tests__/p2p/product-package-discovery.test.ts \
  __tests__/p2p/product-storage-path.test.ts \
  __tests__/p2p/product-runtime-protocol.test.ts \
  __tests__/p2p.product-sync.test.ts
npm run check:bare-worklet
```

Local outputs: `.agent/p5-t3-recovery/green-verification.log` and
`worklet-drift.log`. Worklet drift check passed; bundle SHA-256:
`08f0e49d56390d369fe4b1675992163b1e1c533c9f06167b7aaadd0fad2eee10`.
Earlier full-suite/typecheck/lint results are recorded in the diagnostic audit;
this follow-up did not rerun or independently re-certify those full results.

The cold-drive test uses real Corestore, Hyperdrive and encrypted replication
streams, with a controlled delayed Hyperswarm replacement. It proves client
read sequencing, not public DHT reachability or Android execution. The runtime
waits for pending client connections and then the initial metadata update for
a cold drive. Both operations retain bounded waits.

| Observation | Layer | Result and limit |
|---|---|---|
| Cold manifest after delayed connection | Component integration | Exact manifest bytes returned; controlled peer transport |
| Missing file after replication | Component integration | Rejected with `PRODUCT_PACKAGE_READ_FAILED` |
| Discovery timeout, false or rejection | Unit | Balanced `findingPeers` completion, resources closed, runtime not open |
| Storage paths, protocol and sync state | Unit/component | Remaining focused suites pass; no device certification implied |

## Primary-agent reflection observations

These four observations document direct analysis, not independent review or
completion of the gated closure sequence.

1. **Client readiness:** announcement completion was insufficient for a cold
   client. Waiting for `swarm.flush()` and pairing `findingPeers()` addresses
   that boundary; the previous source fails the delayed-connection regression.
2. **Metadata readiness:** connection completion can precede Hyperbee's first
   metadata proof. The cold-version update wait prevents a premature missing
   file result; the successful real-store test exercises that sequence.
3. **Failure lifecycle:** discovery failure/timeout still releases store,
   drive and swarm; all three failure-mode tests verify cleanup. A genuinely
   missing file remains an error. The tests do not prove every native
   cancellation race or device teardown control.
4. **Evidence boundary:** the same publication's manifest was readable within
   Compose (626 bytes, one peer), while host probes found zero peers. This
   distinguishes a demonstrated client sequencing defect from the observed
   local connectivity blocker. The Android retry has no proof of the loaded
   worklet hash and never reached VERIFY/PLAYBACK; it cannot certify the fix.

## Handoff

**Owner confirmation 2026-09-26:** local-development work is discarded from
the active scope. The local connectivity finding is retained as historical
evidence and creates no follow-up requirement. This scope decision does not
revert the committed client repair or certify physical playback.

The consuming T7p/P7.T2 physical run must bind the exact RC and loaded worklet,
use a viewer/device pairing provisioned after the last app-state clear, and
use the deployed reachable Availability Node. It must collect the complete
`docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md` evidence, including tamper,
secret/no-fallback and teardown controls. P5 is closed as a feature-delivery
phase; P7/T9g retain the only open certification decision.
