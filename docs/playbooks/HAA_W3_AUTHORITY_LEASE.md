---
type: Playbook
title: "W3 AuthorityLease and Escalation Guard"
governs: "preparatory W3.T3.5–T3.6 implementation"
---

# W3.T3.5–T3.6 — AuthorityLease + Escalation Guard

> **Status: preparatory code only.** W2 HAA profiles/adapter are still in
> feature branches and the physical MacBook Touch ID round-trip remains
> pending. W3.T3.5/T3.6 are NOT wired to DubBridge runners and are NOT
> certified production enforcement.

## Scope / branch discipline

This wave continues on the **single existing W3 integration branch**
`feature/haa-hitl-w3-request-verify`. Do not create new feature branches
for individual W3 tasks. Preserve separate commits for audit/review. No
new CI jobs or GitHub Actions.

## T3.5 — Internal lease

`scripts/haa/authority_lease.py` defines one `AuthorityLease` bound to:

- exact HAA request ID, execution ID and action digest;
- repository, plan, task, session, branch and approved *base* commit SHA;
- the original task-start scope digest and allow-listed paths;
- `edit`, `test`, `review`, `commit` capabilities, bounded by 60–1800 s;
- creation/expiry time and `ACTIVE`/`REVOKED` status.

Only the trusted executor entry point `activate_verified_task_lease()`
can assemble a lease for production use. It **calls**
`verify_live_task_authorization()` first, which requires HAA APPROVED,
`authorizeAndConsume`, an operator-pinned ACTIVE authority key, detached
grant verification, and unchanged live Git/card. It verifies the exact
checkpoint again before creating the lease.

This is **not** HAA grant reuse: the signed HAA `ExecutionGrant` is
single-use, while the resulting DubBridge-internal lease can authorize
repeated in-scope operations after later live checks.

### Persistence / recovery

`LeaseStore` writes private JSON files (directory 0700 / file 0600)
using an exclusive POSIX lock, atomic rename/link, and fsync. Every record
has an HMAC-SHA256 envelope with an **executor-only signing key** of at
least 32 bytes. The HMAC key is never stored with a lease. A different key,
tampered record, malformed TTL or invalid scope fails closed.

The key must be provisioned from an operator-controlled secret store when
W3.T3.8 is implemented and must never be exposed to model/tool subprocesses.
The existing runner strips most environment variables from agent subprocesses,
but **T3.8 must validate that separation end to end**.

Crash after successful HAA consume but before lease persistence leaves no
usable lease; an operator must resolve it rather than auto-reuse/auto-approve
the consumed grant. Lost signing keys invalidate persisted leases, they do not
trigger fallback approval.

## T3.6 — Escalation guard

`scripts/haa/lease_guard.py` reads the sealed record before every intended
operation and checks the current trusted repo/task context.

| Condition | Outcome |
|---|---|
| Valid active lease + matching task/session/branch/scope + in-scope path/capability | `ALLOW` |
| Expired/revoked lease | `REQUIRE_HAA`; expired lease is durably revoked |
| New task/session/branch/repository/RRI/scope/capability | `REQUIRE_HAA`, revoke current lease |
| Git HEAD advanced by an in-scope task commit (approved base remains ancestor) | Remains valid, subject to operation/path guard |
| History rewritten or no longer descends from approved base | `REQUIRE_HAA`, revoke |
| Path outside allowedPaths | `REQUIRE_HAA`, revoke |
| Traversal, path through symlinked ancestor or unsafe reserved path | No `ALLOW`; fail closed |
| `merge` | `REQUIRE_HAA` exact `git.merge.v1` |
| `plan_close` | `REQUIRE_HAA` exact `dubbridge.plan.close.v1` |
| `push`, `deploy`, `secrets`, `delete`, `overwrite`, etc. | `BLOCK` until dedicated ActionProfile and executor exist |
| Invalid HMAC, missing signing key, unsafe store, corrupt card or untrusted Git context | `BLOCK` |
| `edit`/`commit` with no touched-path evidence | `BLOCK` |

`ALLOW` means only that the lease guard accepts this bounded operation:
it **does not** bypass file-system sandboxing, command allowlists, task
review, owner approval, or higher-priority HITL policy.

### TOCTOU and enforcement caveat

The preparatory guard is not a lock around the actual operation. W3.T3.8
must re-evaluate it at the authoritative runner boundary under the
appropriate operation lock and obtain the actual touched file set from
trusted tooling (not only model-provided prose). Until then it cannot
serve as the final safety barrier for concurrent operations or all
filesystem side effects. No live runner is calling this guard.

## Local verification (no CI)

```bash
python3 -m unittest scripts.haa.authority_lease_test
python3 -m unittest scripts.haa.lease_guard_test
sh scripts/haa/run-tests.sh
```

Behavioral evidence to collect:

- HP-1: one trusted exact consumed grant -> one sealed task lease;
- HP-2: repeated in-scope operations without additional HAA ceremonies;
- HP-3: HEAD advance with base ancestor remains inside same task;
- EC-1: tampering, key replacement or malformed lease -> BLOCK;
- EC-2: expiry, RRI/session/branch/repo/scope change -> revoke/reapprove;
- EC-3: paths outside scope, traversal, symlink parent -> never ALLOW;
- EC-4: plan close/merge never covered by task lease;
- EC-5: restart retains only intact leases under the same executor key.

**Review status:** tests are committed but local full-repository execution and
independent band-routed code review have not yet been performed.
The W2 physical smoke is still the gate to live activation.
