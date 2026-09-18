---
type: Audit
title: "MVP0-P2P P5.T1/T2 automated evidence remediation"
date: 2026-09-18
status: evidence_complete_owner_verification_pending
slice: MVP0-P2P
---

# P5.T1/T2 automated evidence remediation

## Scope

This record closes the **automated evidence gap** identified by
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`
for P5.T1 and the T1-dependent portion of P5.T2.

No P5 production source changed in this remediation. The work added executable
coverage around the existing `ProductPlaybackRuntime` and the existing
`P2PPlaybackController` authorization boundary.

This record does **not** claim P5 phase PASS and does not replace P5.T3 Android
device evidence.

## Exact implementation evidence

Commits:

- `23f91ae` — component coverage for the loopback playback security boundary.
- `234d281` — authorization mismatch coverage for asset/publication/lineage/expiry.
- `3a91e57` — typed socket-harness correction after CI exposed a TypeScript issue.
- `892c21e` — altered-AAD and missing-content-key denial coverage.
- `b154938` — real OS loopback TCP-listener integration proof.

The first CI attempt exposed a test-harness typing defect (TS7022/TS7024).
That defect was confined to test code and was repaired before evidence was
accepted.

## P5.T1 acceptance mapping

### HP-P5.T1-1 — verified HLS is served through the authorized loopback session

PASS at automated/component-integration level.

Evidence in `mobile/__tests__/p2p/product-playback-runtime.test.ts` now proves:

- the real `ProductPlaybackRuntime` starts an actual OS TCP listener through
  its `bare-tcp` seam;
- the listener binds to `127.0.0.1` only;
- the returned playback URL contains a randomized 128-bit session token;
- the encrypted `index.m3u8` is read from the verified package boundary,
  decrypted using the existing P2 AES-256-GCM/AAD contract, and served over
  loopback;
- rewritten HLS segment references remain scoped to the same randomized
  session token;
- responses include `Cache-Control: no-store`.

The real-OS-loopback test uses Node's TCP implementation only as the transport
behind the existing `bare-tcp` factory seam. The production
`ProductPlaybackRuntime` request parsing, session scoping, package reads,
decrypt-on-read, manifest rewrite, response headers and teardown all execute
unchanged.

### EC-P5.T1-1 — fail closed

PASS at automated/component level for the previously missing security paths:

- foreign session token -> denied;
- path traversal -> denied;
- altered ciphertext -> denied;
- altered authenticated AAD/path identity -> AES-GCM authentication failure;
- missing/invalid content key -> startup denied before package/transport open;
- loopback startup failure -> decoded CK is zeroized and partial package state
  is closed;
- explicit stop -> listener closes, active sockets are destroyed, package state
  closes and the transient CK buffer is zeroized.

No HTTP/S3 audience-media fallback was added or invoked. The valid path returns
only a `127.0.0.1` playback URL; negative paths terminate locally.

## P5.T2 acceptance impact

The retrospective blocker on P5.T2 was specifically transitive: T2 could not
claim deterministic CK/session release while T1's actual runtime zeroization was
untested.

That blocker is now removed at the automated evidence layer:

- existing `P2PPlaybackLease` tests prove idempotent release, teardown-before-retry,
  and fail-closed retry when stop fails;
- existing `P2PPlaybackController` tests prove explicit loopback stop;
- new controller coverage proves asset, publication, lineage, viewer and expiry
  mismatches all fail **before** transient K1 unwrap/playback startup;
- T1 runtime evidence now proves the underlying stop actually zeroizes the CK and
  tears down listener/package ownership.

P5.T2 still requires P5.T3 device evidence for visible video playback,
sign-out/error behavior on the Android runtime, and review-path non-regression
before the **phase** can pass.

## CI evidence

GitHub Actions run for `b1549383719a595e2f84aa96677f8d41cbe440b4`:

- workflow run: `35399262269`
- job: `mobile` — **PASS**
- `npm run typecheck` — PASS
- `npm run lint` — PASS
- `npm test` — PASS
- 59 test suites passed
- 426 tests passed
- focused suites passing:
  - `__tests__/p2p/product-playback-runtime.test.ts`
  - `__tests__/p2p/p2p-playback-controller.test.ts`

The repository-wide workflow may contain unrelated job failures; this record
claims only the mobile gate above as evidence for this remediation.

## P5.T3 / device boundary

Still required and intentionally not claimed here:

- real invitation claim on Android;
- real P4 sync and verified-package handoff;
- visible short-video playback through the existing `VideoPlayer`;
- runtime/network proof that media delivery remains loopback-only;
- stop/unmount and restart on Android;
- key/gateway/tamper failure on the device path;
- existing ADR-032 review playback non-regression;
- physical-device hardware-backed Keystore evidence for physical PASS.

The development harness and
`docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md` remain the canonical device
procedure.

## Governance / documentary disposition

The MVP0-P2P owner review exception remains applicable to phase-1/phase-2 peer
review only:

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner
- Scope-note: skips only phase-1 and phase-2 peer review.

This remote remediation does **not** infer owner final verification or fabricate
an RRI record. Therefore P5.T1 and P5.T2 move from **blocked for missing
evidence** to **automated evidence PASS / formal closure pending owner
verification and task-governance synchronization**.

Document status changes must not:

- mark aggregate P5 PASS;
- unblock P6 on the basis of automated tests alone;
- override the formal P4 -> P5 dependency;
- update roadmap Done/PASS state before P5.T3 device evidence and final closure.

## Plan impact

The correct planning state after this remediation is:

```text
P5.T0  Done
P5.T1  automated evidence PASS; formal closure pending
P5.T2  automated evidence PASS; formal closure pending
P5.T3  next executable evidence step: Android certification

aggregate P5 = IN PROGRESS, not PASS
P6 gate      = unchanged / still closed
roadmap PASS = unchanged
```
