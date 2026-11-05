---
type: Playbook
title: "HAA W3 recovery and failure handling"
governs: "W3.T3.7 preparatory recovery logic"
---

# W3.T3.7 — Recovery & Failure Handling

> **Status:** Code and tests authored, local full-checkout tests and independent
> review pending. Recovery is NOT wired to any DubBridge runner. W2 physical
> Touch ID certification remains a hard prerequisite to live enforcement.
>
> **Branch:** `feature/haa-hitl-w3-request-verify` (the owner's single W3
> branch; no additional branches or GitHub Actions).

## Safety model

Restarts may be ambiguous. An HAA request may have been created even if
the requester did not receive the HTTP response. Likewise, HAA may have
consumed an approval before the executor received or verified the grant.

Recovery code must **never** create a new HAA request, retry
`authorizeAndConsume`, activate an `AuthorityLease`, or resume an agent
runner as a side effect of startup.

Use `scripts/haa/recovery.py` for read/reconcile/revoke only:

- `recover_attempt` / `recover_all_attempts`: inspect checkpoints,
  perform safe remote GET for known request IDs, and report a typed hold;
- `recover_all_leases`: inspect HMAC-sealed leases, revoke expired
  leases, and mark others `NEEDS_LIVE_SCOPE_CHECK`;
- `revoke_leases_for_attempt`: revoke any sealed lease correlated
  to a terminal or uncertain attempt, never silently ignore damaged
  envelopes.

No method returns `ALLOW` or an unverified `ExecutionGrant`.

## State-to-recovery matrix

| Durable local checkpoint | On restart | No automatic action |
|---|---|---|
| `PREPARED` | `REQUEST_UNCERTAIN` | Could have POSTed before crash; operator reconciliation |
| `REQUEST_UNCERTAIN` | Terminal hold | Never duplicate request POST |
| `WAITING_FOR_HUMAN` | Remote GET, verify request ID/digest/audience | No grant or runner |
| `APPROVED_NOT_CONSUMED` | Remote GET, verify request ID/digest/audience | Approval is not execution authority |
| `CONSUMPTION_IN_FLIGHT` | `CONSUMED_UNVERIFIED` | Never retry consumption automatically |
| `CONSUMED_UNVERIFIED` | Terminal hold + related lease revocation | Never claim authorized execution |
| `REJECTED`, `REVOKED`, `EXPIRED` | Terminal hold + related lease revocation | No implicit fallback |
| `CANCELLED_LOCAL`, `WAIT_TIMED_OUT` | Terminal hold + related lease revocation | Remote HAA may still approve, but local state cannot consume |
| `VERIFIED_NO_SIDE_EFFECT` | Hold for lease revalidation | Still not a runner permission |

HAA `REQUESTED` or `PENDING` maps to `WAITING_FOR_HUMAN`.
HAA `REVOKED` is distinct from `REJECTED`. Network unavailability
or mismatched remote state/digest/audience produces
`REMOTE_STATUS_UNAVAILABLE` and does **not** advance the checkpoint.

Local `stop_waiting(store, id, reason=...)` accepts only
`USER_CANCELLED` or `LOCAL_TIMEOUT`. This stops the **local**
attempt but does not impersonate the HAA approver, revoke the remote
authenticator, or issue a fake HAA reject.

## Write-ahead consumption guard

`scripts/haa/live_verify.py` persists this transition:

```text
APPROVED_NOT_CONSUMED
        |
        | atomic local store.transition()
        v
CONSUMPTION_IN_FLIGHT
        |
        | POST HAA authorizeAndConsume
        +--> verified grant + unchanged live Git/card
        |       -> VERIFIED_NO_SIDE_EFFECT
        |
        +--> error, timeout, stale state, bad signature
                -> CONSUMED_UNVERIFIED
```

If the process exits during the HTTP call, the persisted in-flight marker
converts to `CONSUMED_UNVERIFIED` on restart. Even a same-execution-ID
retry, which HAA may support, requires a separately reviewed manual
recovery procedure. This wave intentionally does not automate it.

## Lease restart rules

`recover_all_leases()` verifies every HMAC before reading its
authorization fields. It revokes a lease whose TTL expired during downtime,
and **never** turns an intact active lease into automatic permission.

An intact, unexpired `ACTIVE` record is only a candidate for the W3.T3.6
live guard; the trusted runner will later have to verify the current
task-card, session, repo origin, Git branch/history, scope, touched paths
and operation immediately before use. This final boundary remains W3.T3.8.

Invalid HMAC, missing local executor key, unsafe store permissions, or a
corrupt lease record must block. Do not delete invalid records to make the
gate pass; preserve them for diagnosis.

## T3.7 acceptance

- T3.7.1 Rejection, revocation, cancellation, local timeout, HAA expiry:
  explicit terminal outcomes; no automatic allow.
- T3.7.2 Restart with a known request ID: safe GET/reconcile only;
  initial PREPARED never reposts.
- T3.7.3 Crash while consuming: write-ahead marker prevents accidental
  double consumption or grant replay.
- T3.7.4 Revoked/expired/tampered leases are never executable after restart;
  related known leases can be durably revoked.
- T3.7.5 Negative-path tests check every case, including a fake remote
  outage and time-of-check state drift.

## Local verification (no GitHub Actions)

From DubBridge root:

```bash
python3 -m unittest scripts.haa.recovery_test
python3 -m unittest scripts.haa.request_checkpoint_test scripts.haa.live_verify_test
sh scripts/haa/run-tests.sh
```

Tests are **authored**, not certified against a full checkout in this
development session. Review and full local test PASS remain required
before marking W3.T3.7 DONE.

## Activation / owner checkpoint

- W2 HAA/DubBridge code integration and PR/merge remain outstanding.
- Physical Touch ID gate `npm run validate:dubbridge-macos` is not yet
  recorded as PASS.
- W3.T3.8 runner enforcement must not activate on unreviewed or
  unverified recovery code.
- The only needed physical owner action is the MacBook Touch ID ceremony
  during the W2 gate; do not substitute synthetic evidence.
