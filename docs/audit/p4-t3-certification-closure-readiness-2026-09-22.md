---
type: Audit
title: "P4.T3 certification closure readiness"
date: 2026-09-22
task: P4.T3
status: blocked_exact_head_device_rerun
---

# P4.T3 — certification closure readiness

## Current verdict

**BLOCKED only on exact-head integrated Android evidence.**

The two gaps named by the 2026-09-18 retrospective have otherwise been removed:
P4.T1 bounded reconnect is implemented/reviewed, and account-change is now
exercised through the app's provider lifecycle.

This artifact does **not** claim aggregate P4 PASS because the P4→P5 integrated
handoff must be re-executed on Android after P4.T1-r1 changed the product package
open path.

## Evidence already satisfied

| Criterion | Evidence | State |
|---|---|---|
| interrupted/resumed sync | `p2p.product-sync.test.ts`: reconnect after source-open failure and partial-cache reuse | PASS |
| unavailable peer bounded exhaustion | exactly two opens with one reconnect budget; no READY | PASS |
| corruption | invalid cached ciphertext is re-fetched; corrupt source ciphertext never reaches READY | PASS |
| sign-out during reconnect | active source cancelled/closed; account snapshot/cache wiped; no READY | PASS |
| account A -> B | `p2p.provider-account-change.test.tsx` exercises provider identity change and clears A | PASS |
| account isolation | product-sync coverage requires B to establish its own lifecycle | PASS |
| permission vs possession | `p2p-playback-controller.test.ts` rejects foreign viewer, asset/publication/lineage mismatch and expiry before K1 unwrap/playback | PASS |
| claim denial | `p5-device-certification.test.ts` stops before sync/playback when claim is denied | PASS |
| verified-handle gate | certification orchestration refuses playback when verified-handle creation fails | PASS |
| P4.T1 independent review | `docs/audit/p4-t1-bounded-reconnect-code-solution-review-2026-09-22.md` | PASS |
| P4.T1-r1 filesystem repair | RED→GREEN + full mobile gate + worklet drift check | PASS (CI) |

Mobile gate evidence at `469e45c55652a431382138d00aa3f406b2bccc21`:
**61/61 suites, 441/441 tests PASS**, including the product-sync,
provider-account-change, playback-controller and P5 certification suites.

P4.T1-r1 evidence:
`docs/audit/p4-t1-r1-storage-uri-fix-evidence-2026-09-22.md`.

## Remaining blocking evidence

A fresh P5.T3 Android certification run is required on the final branch revision:

`claim -> sync -> verify -> playback`

It must prove on the real product runtime that:
- the repaired Corestore path opens without `ENOENT stat "file:"`;
- product ciphertext is discovered/copied and the package verifies;
- the resulting verified package handle is accepted by P5;
- playback starts only with the current control-plane authorization;
- teardown/non-regression evidence required by P5.T3 is captured.

Because P4.T1-r1 changed the sync/open path after the prior P5 attempt, older
device evidence cannot be transposed to this revision.

## Closure condition

When the exact-head P5.T3 device run passes, append its evidence SHA/run details
here, then synchronize the P4 ledger/plan and parent P2P status artifacts.

Until that run exists:

`P4.T3: BLOCKED`
`aggregate P4: NOT_PASS`
