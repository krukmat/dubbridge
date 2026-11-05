---
type: Playbook
title: "DubBridge HAA W3 request persistence and live verification"
governs: "W3.T3.3 + T3.4 preparatory components"
---

# W3.T3.3–T3.4 — durable request and live exact authorization

> **Not active in runners.** W2 is still unmerged and physical Touch ID
> certification is pending. These modules are preparatory security components,
> not a completed HITL workflow. Do not use a forged/synthetic approval as
> evidence of a real MacBook authorization.

## Implementation and boundaries

| Component | Purpose | Explicitly does not do |
|---|---|---|
| `scripts/haa/request_checkpoint.py` | Durable, restricted-permission approval checkpoint and HAA lifecycle correlation | Permit agent execution or treat APPROVED as a grant |
| `scripts/haa/live_verify.py` | Reconstruct task-start from trusted card and live Git; verify exact HAA grant against operator-pinned key | Create AuthorityLease, start model, merge, release or close plan |
| `scripts/haa/request_checkpoint_test.py` | Positive and fail-closed persistence/state tests | Physical Touch ID |
| `scripts/haa/live_verify_test.py` | Local Git + pinned key + stale-state tests | Production HAA network certification |

### W3.T3.3 state model

```text
PREPARED (durably persisted before request POST)
  ├─ HTTP create succeeded -> WAITING_FOR_HUMAN
  │                           ├─ PENDING -> WAITING_FOR_HUMAN
  │                           ├─ APPROVED -> APPROVED_NOT_CONSUMED
  │                           ├─ REJECTED -> REJECTED
  │                           ├─ EXPIRED -> EXPIRED
  │                           └─ CONSUMED -> CONSUMED_UNVERIFIED
  └─ POST outcome unknown -> REQUEST_UNCERTAIN (terminal, no auto retry)

APPROVED_NOT_CONSUMED
  └─ W3.T3.4 independent consume + verify
       ├─ exact grant valid, live state stable -> VERIFIED_NO_SIDE_EFFECT
       └─ failure/uncertain consumption -> CONSUMED_UNVERIFIED (terminal)
```

**Important:** A remote HTTP timeout can happen *after* HAA accepted
the request. `REQUEST_UNCERTAIN` never causes an automatic duplicate POST.
A human/operator needs external reconciliation before initiating a new
authorization ceremony. The local attempt is not a grant and contains
no API keys or biometric material.

The checkpoint store uses a private 0700 directory, mode-0600 JSON files,
a POSIX advisory lock for transitions, and atomic same-directory persistence
with fsync. The durable `execution_id` is generated before the request and
never replaced for retries. An attempt ID must be 32 lowercase hex characters.

### W3.T3.4 live state and authority

The trusted orchestrator will eventually provide:

- repository root and operator-approved canonical repository name;
- task-card-v2 path validated using the existing
  `scripts/local-agent/task_card.py` parser;
- trusted workflow session ID and bounded capabilities;
- HAA executor client with separate executor credentials;
- **operator-pinned HAA authority key ID and SHA256 of its PEM** from a
  trusted local configuration (not copied from the HAA response).

The verifier reads and checks the live Git working tree's origin, repository
root, branch and HEAD; it reconstructs the full ActionSpec and compares its
digest and scope to the durable request. It refuses to use
`task_start_actual_state(action)` because that W2 smoke helper echoes
the request rather than checking the Git source of truth.

It checks HAA's current request state, intended audience, request ID and
action digest. It validates the active authority-key metadata against
the pinned key, invokes HAA `authorizeAndConsume` using the durable execution
ID, detached-verifies the returned grant, and re-reads Git/card state.
Only then can it record `VERIFIED_NO_SIDE_EFFECT`.

**No lease is activated**, and a verified record is not automatically
permission to execute any operation. Lease creation and runtime guard
enforcement remain W3.T3.5/T3.8.

### Test gates (local, no GitHub Actions)

From the DubBridge root, after installing
`scripts/haa/requirements.txt` in an isolated Python environment:

```bash
python3 -m unittest scripts.haa.request_checkpoint_test
python3 -m unittest scripts.haa.live_verify_test
sh scripts/haa/run-tests.sh
```

Coverage to verify:

- PENDING -> APPROVED_NOT_CONSUMED does not grant execution;
- rejection/expiration/unknown lifecycle fail closed;
- unknown request result is terminal (no automatic double request);
- immutable request/action/audience/execution correlation;
- repository origin or HEAD mismatch blocks before authorization;
- wrong authority PEM hash blocks before authorization;
- head changes during consumption make grant non-executable;
- no code path changes a runner or plan state.

**Verification classification:** code and tests have been authored on
`feature/haa-hitl-w3-request-verify`, but a full-repo local run,
canonical RRI calculation, independent code review, and MacBook HAA E2E
are still pending. Do not mark these development tasks DONE prematurely.

## Next integration gates

1. Merge W2 HAA ActionProfiles and DubBridge adapter to their respective
   `main` branches through approved process.
2. Run W2's real physical Touch ID smoke and collect a PASS transcript.
3. Run the full W3 local unit suite and band-routed independent review.
4. In W3.T3.5 implement/verify a scope- and TTL-bound `AuthorityLease`.
5. Only in W3.T3.8 attach non-bypass guards to authoritative runner
   entry points. No new GitHub Actions workflows/jobs should be added.
