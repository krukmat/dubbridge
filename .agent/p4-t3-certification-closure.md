# P4.T3 certification closure packet

## Goal

Close P4.T3 after P4.T1 bounded reconnect is implemented and verified, reusing
valid evidence from P5.T3 where it proves the P4 product path instead of
duplicating device work.

Do not infer P4 PASS from P5 PASS. P4.T3 still owns its own certification
contract.

## Dependencies

Required before closure:
- P4.T1 bounded reconnect PASS
- P4.T2 already Done
- P5.T3 device run available as supporting integration evidence
- if P4.T1 changes the sync path after that P5.T3 run, rerun P5.T3 on the new
  exact HEAD before using it as exact-revision integrated evidence

## Evidence that P5.T3 may contribute

A successful real-device/emulator path:

`claim -> sync -> verify -> playback`

can support:
- product runtime discovers/opens the real P2P source
- P4 copies the package
- P4 verifies it
- `VerifiedPackageHandle` is accepted by P5
- replicated bytes alone are insufficient without a successful control-plane
  claim/authorization
- tampered/incomplete package is denied if that negative control is executed

Record the exact P5 evidence SHA and do not silently transpose evidence from a
different code revision.

## Evidence P4.T3 must add itself

### 1. Interrupted/resumed sync

Use P4.T1 executable evidence to demonstrate:
- interruption does not restart valid cached ciphertext from zero
- bounded reconnect reaches success when peers recover
- exhausted reconnect never reaches READY

### 2. Account change (distinct from sign-out)

Add executable coverage for:

```
Account A starts/has P4 state
  -> active callbacks/session belong to A
  -> application account changes to B
  -> A work is cancelled/ignored
  -> A cache cannot surface through B
  -> B cannot inherit A READY snapshot
  -> B must establish its own scoped lifecycle
```

Prefer testing through the product controller/cache seam actually used by the
app. A mere static account-key test is insufficient if it does not exercise the
account-change lifecycle.

### 3. Permission vs possession

Record executable evidence that:
- ciphertext transport/cache success alone does not mint a verified/playable
  control-plane grant
- invalid/expired/foreign control-plane authorization cannot proceed into P5

The previously observed expired-invitation `CLAIM_FAILED` may be cited as
supporting control-plane fail-closed evidence, but distinguish P3 claim evidence
from P4 package-readiness evidence.

## Certification artifact

Create one discrete task-scoped audit artifact under `docs/audit/` mapping:

- HP-P4.T3-1
- EC-P4.T3-1
- interruption/resume
- unavailable-peer bounded exhaustion
- corruption
- sign-out
- account-change
- permission-vs-possession
- P4 -> P5 verified handle handoff

The artifact must list exact commands/tests/device evidence and exact commit
SHAs. Do not mark a row PASS based only on prose or screenshots.

No production certification harness is required if existing product tests plus
P5.T3 provide executable evidence for every criterion. If an executable P4
certification helper is still necessary, keep it development-only and narrowly
scoped; score it separately before implementation.

## RRI

Run `scripts/rri.py` on the exact closure/source-test paths before any new code.
Documentation-only evidence synchronization is exempt from phase-2 code review,
but any new account-change/certification source or test seam follows the normal
development-task workflow.

## Status synchronization

Only after every P4.T3 criterion is mapped to passing evidence:
- update `docs/tasks/mvp0-p2p-p4-mobile-sync.md`
- update `docs/plan/mvp0-p2p-p4-mobile-sync.md`
- update parent P2P status artifacts required by the ledger
- do not mark P5/P6 PASS here

## Final output

```
P4.T3: PASS | BLOCKED
HEAD:
P5 evidence SHA reused:
P4.T1 evidence:
account-change:
corruption:
permission-vs-possession:
verified-handle handoff:
audit artifact:
status files synchronized:
remaining P4 blocker:
aggregate P4: PASS | NOT_PASS
```
