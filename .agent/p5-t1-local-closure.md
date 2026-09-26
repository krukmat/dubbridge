# P5.T1 local closure packet

## Goal

Close the open P5.T1 evidence gap before device certification.

Current branch: `feature/p2p-mvp-core`.

Do not create a new branch. Do not start P5.T3/Android device certification in this task.

## Why this task exists

P5.T1 is currently blocked because the real `ProductPlaybackRuntime` loopback server has no integration coverage. Existing tests cover only pure helpers.

The missing evidence is specifically the network-facing fail-closed boundary:
- loopback bind/session-token scoping
- path/traversal rejection
- ciphertext integrity/decryption delivery path
- deterministic stop/listener teardown
- CK zeroization on stop/start-error
- no remote audience-media fallback

Relevant ledger:
- `docs/tasks/mvp0-p2p-p5-local-playback.md`
- `docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`

Relevant implementation:
- `mobile/src/p2p/runtime/product-playback-runtime.ts`
- `mobile/src/p2p/runtime/product-package-runtime.ts`
- `mobile/__tests__/p2p/product-playback-runtime.test.ts`

## First gate: RRI

Before source changes, compute P5.T1's actual RRI with `scripts/rri.py`, using the real touched paths and conservative security/network values.

If RRI >= 26:
- do not implement yet
- produce the repository Compact Approval Task Card v2
- report the RRI/band and exact proposed writable paths
- stop for owner approval

If RRI <= 25:
- continue locally under the Low-band route

Do not under-score security/fail-closed/network scope to force Low.

## Preferred implementation shape

Keep production behavior unchanged unless a test exposes a real defect.

Prefer adding focused integration/component tests around the actual `ProductPlaybackRuntime`, using a controlled fake/mock for the TCP and package-runtime dependencies only where necessary.

Evidence must exercise the runtime class itself, not just helper exports.

Required cases:
1. Valid verified package session:
   - binds only to `127.0.0.1`
   - returns randomized token URL
   - GET manifest succeeds
   - rewritten segment URLs remain under the same session token
   - `Cache-Control: no-store`
2. Foreign/wrong token request is denied.
3. Traversal request is denied.
4. Tampered ciphertext/digest/authenticated decrypt failure is denied.
5. Stop:
   - closes listener
   - destroys active sockets
   - closes package runtime
   - zeroizes the in-memory CK
6. Start failure after CK decode also zeroizes CK and closes partially-open state.
7. No test may introduce an HTTP/S3 fallback path.

If testing CK zeroization cleanly requires a very small test seam, keep it private/minimal. Do not redesign crypto or runtime ownership.

## Scope guards

Do not:
- change P2/P3/P4 contracts
- change K1/AAD format
- change schema/migrations
- change authorization semantics
- add remote media fallback
- start Android/emulator/device certification
- consume `tmp/p5-invitation-token.txt`

If the test reveals a production defect, stop and report the exact defect before broadening the implementation.

## Verification

Run at minimum:
- focused Jest tests for `product-playback-runtime`
- existing P5 playback controller/lease tests
- `npm run typecheck`
- relevant lint if feasible

Then run the broader P2P mobile test set if runtime is reasonable.

Update P5 task/audit status only if the acceptance evidence truly closes T1. If T1 closes, re-evaluate T2's transitive blocker; do not automatically mark T3 PASS.

## Required output

```
P5.T1: PASS | BLOCKED | AWAITING_APPROVAL

branch:
HEAD:
RRI:
band:

production code changed: YES/NO
test files changed:
- ...

evidence:
- loopback bind:
- session token:
- traversal:
- tamper/decrypt fail-closed:
- stop/listener teardown:
- CK zeroization:
- no remote fallback:

tests:
- ...

P5.T1 ledger status:
P5.T2 status after T1:
P5.T3 started: NO

If BLOCKED:
exact blocker:
minimal next action:
```
