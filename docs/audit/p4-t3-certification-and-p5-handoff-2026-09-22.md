---
type: Audit
title: "P4.T3 certification and P5 handoff"
date: 2026-09-22
task: P4.T3
status: pass
---

# P4.T3 — certification and P5 handoff

## Verdict

**PASS. Owner verification complete. Aggregate P4 = PASS.**

No new runtime implementation is required for P4.T3. The product sync,
verification, account-lifecycle and P5-handoff acceptance surface is already
covered by executable evidence, and the P3 PASS activation gate is satisfied.

Exact repository head entering this closure pass:
`5c749189cadfba6d3bd880c673c5aa342db7a14d` — **15/15 CI PASS**.

The prior artifact
`docs/audit/p4-t3-certification-closure-readiness-2026-09-22.md` recorded an
exact-head Android rerun as a blocker. That condition was superseded by the
2026-09-22 sequencing amendment: **P5.T3 is a deferred release-certification
obligation and no longer gates P4 closure**. The physical Android proof remains
mandatory for P5-CERT/P7, but it is not a P4.T3 acceptance prerequisite.

## P4.T3 acceptance mapping

### HP-P4.T3-1

> Product runtime discovers, syncs, verifies and hands off the expected package
> after interruption recovery.

Executable evidence:

- `mobile/__tests__/p2p.product-sync.test.ts`
  - transient source-open failure reconnects once and reaches READY;
  - interrupted ciphertext read reconnects;
  - already verified partial ciphertext is reused;
  - invalid cached ciphertext is re-fetched;
  - each source session closes;
  - READY is emitted only after full manifest/package verification.
- `mobile/__tests__/p2p/p2p-sync-controller.test.ts`
  - explicitly starts the product runtime;
  - verifies the complete package;
  - emits only a narrow `VerifiedP2pPackageHandle` for P5;
  - the handle is bound to account, asset, publication, lineage, manifest
    digest and external publication identity.

Result: **PASS**.

### EC-P4.T3-1

> Replicated bytes without control-plane permission remain unplayable;
> transport success alone does not establish readiness.

Executable evidence:

- `p2p.product-sync.test.ts`
  - corrupt source bytes never become READY;
  - bounded peer exhaustion never becomes READY;
  - cancellation/sign-out prevent READY;
  - READY state is account-scoped.
- `p2p.sync-state.test.ts`
  - illegal or incomplete READY transition is rejected;
  - restored state must match account/publication/lineage.
- `p2p.provider-account-change.test.tsx`
  - authenticated Account A → B transition clears A;
  - sign-out clears the previous account.
- `p2p-playback-controller.test.ts`
  - foreign viewer, asset/publication/lineage mismatch or expiry fail before
    K1 unwrap/playback;
  - current O3 authorization is required even when a verified P4 handle exists.
- `p5-device-certification.test.ts`
  - claim denial stops before sync;
  - sync failure stops before verify/playback;
  - missing verified handle stops before playback;
  - playback denial remains fail-closed.

Result: **PASS**.

## P5 handoff contract

P4 hands only `VerifiedP2pPackageHandle` downstream:

- account scope;
- asset ID;
- publication ID;
- lineage ID;
- manifest digest;
- external publication ID.

P4 does not hand a plaintext CK, private key, bearer credential, authorization
decision or transport-only readiness claim.

P5 must independently re-read current O3 authorization before native K1 unwrap
and playback. Therefore possession of replicated ciphertext or a verified P4
package does not itself grant playback.

## P4.T1-r1 relationship

The Corestore `file:` URI repair remains **implemented + CI verified**.
Its physical Android confirmation is intentionally retained under deferred
P5.T3 release certification. It does not block P4.T3 after the sequencing
amendment.

## Aggregate phase state

- P4.T0 — Done
- P4.T1 — Done
- P4.T2 — Done
- P4.T3 — closure-ready
- P4.T1-r1 — implemented + CI verified; device confirmation deferred to P5.T3

Owner verification was explicitly provided on 2026-09-22 after `780519c5` completed 15/15 CI PASS. **P4.T3 PASS. Aggregate P4 PASS.**
