# P4.T1 bounded reconnect task packet

## Goal

Close the real P4.T1 implementation gap recorded in
`docs/tasks/mvp0-p2p-p4-mobile-sync.md`: the product sync path currently
performs one source attempt and then returns with a restartable state, while the
existing `reconnect-budget.ts` is not wired into `P2pProductSync.ts`.

Do not execute this source change until the current P5.T3 Android run has
finished and its evidence SHA has been frozen. This change is on the P5
claim->sync->verify path, so P5.T3 must be rerun on the post-P4.T1 HEAD before
aggregate P5 can be treated as exact-revision evidence.

## Current code boundary

Primary:
- `mobile/src/p2p/sync/P2pProductSync.ts`
- `mobile/src/p2p/runtime/reconnect-budget.ts`
- `mobile/__tests__/p2p.product-sync.test.ts`

Only touch `P2PSyncController.ts` if an explicit configuration seam is truly
needed. Prefer keeping the retry budget inside the product sync abstraction.

Existing guarantees to preserve:
- READY only after full manifest + ciphertext verification
- verified cached ciphertext is reused on restart
- cancellation prevents READY
- sign-out clears only the signed-out account
- source session is closed deterministically
- proof/dev topology is not activated
- transport possession never creates authorization/playability

## RRI gate

Before implementation, run the repository calculator with the actual proposed
paths and measured/defensible values.

Planning estimate (not a substitute for running `scripts/rri.py` locally):
- expected parent band: **Med-high / approximately RRI 55**
- reason: async state-machine/domain + retry coupling make the technical
  bottleneck at least level 2 even with good existing coverage

The owner has already approved proceeding with the post-P5 plan in chat, but the
repository workflow still requires recording the actual RRI and the corresponding
Compact Approval Task Card / phase-1 review evidence before source execution.

If actual RRI >= 56, decompose before implementing.

## Preferred behavior

Bound the *automatic reconnect attempts* while reusing the existing persistent
partial cache.

Conceptual flow:

```
sync start
  -> prepare/restartable state
  -> open source session
  -> copy missing/invalid ciphertext only
  -> verify package
  -> READY

transient source/open/read failure
  -> close current session
  -> record bounded disconnect
  -> if budget remains:
       RETRYING -> open fresh source session -> resume from verified cache
     else:
       remain non-READY and return failure
```

Do not retry `PackageVerificationError`; integrity/authenticity failures stay
fail-closed and terminal under the existing semantics.

Cancellation/sign-out must win immediately over retry.

Do not add unbounded timers, background retry workers, global retry engines or
new persistence schema.

## Exact acceptance cases

1. transient source failure -> bounded retry -> success -> READY
2. retry budget exhausted -> throws/returns failure and never READY
3. valid partial ciphertext cache is reused after reconnect
4. invalid cached ciphertext is fetched again; it is never trusted by presence
5. cancellation during the reconnect path stops further source opens and never READY
6. sign-out during the reconnect path clears account state and stops further work
7. package verification failure does not consume reconnect attempts as if it were a peer outage
8. every opened source session is closed, including failed attempts
9. already READY snapshot still bypasses network
10. another account cannot inherit READY/cache lifecycle state

Prefer deterministic tests with no wall-clock sleeps.

## Implementation constraints

- Reuse `createReconnectBudget` / `recordDisconnect`; do not fork a second
  budget model.
- Use a small explicit default retry count unless an existing frozen contract
  defines another value. If no canonical count exists, make the count a
  constructor-level constant/config seam that tests can override without
  changing public protocol/DB schema.
- Do not modify P2/P3 crypto, K1, authorization, descriptor format, RPC version,
  database schema, or Android native code.
- No remote HTTP/S3 media fallback.

## Verification

At minimum:

```bash
cd mobile
npx jest __tests__/p2p.product-sync.test.ts --runInBand
npm run typecheck
npm run lint
npm test -- --runInBand
```

Run the band-routed code-solution review required by the final RRI before
closure.

## Closure impact

A code/test PASS removes P4.T1's named bounded-reconnect blocker. It does not by
itself close P4.T3 or aggregate P4.

Because this change alters the exact P4 sync path consumed by P5.T3, schedule one
fresh P5.T3 device run against the resulting HEAD before aggregate P5/P6 gate
claims.

## Final report

```
P4.T1: PASS | BLOCKED
HEAD:
RRI:
band:
retry policy:
tests:
review:
READY invariant preserved:
cancellation/sign-out:
partial-cache resume:
code changed:
commit:
P5.T3 rerun required: YES
next: P4.T3 certification closure
```
