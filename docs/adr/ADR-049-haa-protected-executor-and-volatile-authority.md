---
type: ADR
status: Accepted
date: 2026-11-24
---

# ADR-049 — Protected executor and volatile task authority

**Follow-up owner decision (2026-11-24):** the RRI calculation requires reevaluation and is suspended until further notice for development of this HAA ↔ DubBridge feature. Do not recalculate or automatically reinstate it; preserve historical scores. Resumption requires an explicit owner decision after reevaluation. This does not suspend runtime HITL/authorization rules or technical security gates.


Status: **Accepted by owner 2026-11-24; deployment and certification pending.**
Scope: DubBridge software-factory mediated execution and closure on the
existing MacBook. Amends deployment assumptions of ADR-048, not HAA v1.

## Evidence and decision drivers

E1 showed same-UID read/delete and restore of pre-revocation sealed bytes
returns ACTIVE. E3 passed 82 tests (81 pass, one expected skip) on 0713e1f.
See `docs/audit/haa-e1-e3-local-validation-2026-11-24.md`.
A valid MAC proves authenticity of bytes, not freshness; 0700/0600 do not
separate processes of the same identity. HAA physical approval is already
proved for the W2 synthetic smoke; repeating that ceremony cannot solve E1.

## Accepted architecture

One protected, non-root, non-login service identity on the existing Mac,
provisioned once by the owner/admin. Use a root-owned service definition and
versioned, non-agent-writable executor installation (launchd is the candidate
supervisor, not a certified implementation). No broad sudo/root executor.
No extra hardware, VM, cloud subscription or new biometric method required.

The protected service domain owns:
- installed trusted interpreter/runtime, code, dependency set and configuration;
- HAA service signing/storage and executor credentials, provisioned with role
  separation; verify HAA server deployment under a protected identity too;
- authoritative task-card pin, canonical RRI result and trusted task/plan
  source, operation lock, state journal and final protected Git/evidence target;
- a service-private IPC endpoint accepting bounded typed proposals, never
  caller-supplied authority, shell commands, callbacks or RRI downgrades.

The interactive owner session continues running the provisioned signed
HAA macOS approver; keep the existing Secure Enclave/Touch ID enrollment and
trusted display. Approver credentials and installed app must not be exposed
to agent-readable files or replaceable code. Verify this property rather than
assuming Keychain alone isolates same-UID processes. HAA remains the source
of exact human authorization; DubBridge owns workflow and leases.

Agents and arbitrary repository-controlled verification commands run outside
the protected service identity. They use a scratch worktree and have no
write access to the protected target, journal, executable or credentials.
The protected executor mediates bounded patches and commits, never imports
agent-generated code or runs its tests under its own identity. Tests run in
an unprivileged worker boundary; validate worker isolation, process lifecycle,
output/patch provenance, Git hooks/config, symlink/hardlink and network
constraints. Keep service IPC role checks explicit, even on localhost.

Owner/account agents may keep their present UID only if they have no
unattended elevation, service credential access or write path to the
protected target. If this cannot be demonstrated, move agent execution to
a dedicated standard worker account or stop activation. An administrator
deliberately elevating, kernel compromise and whole-host privileged backup
rollback are outside this scoped threat model. Do not claim control of
arbitrary shell commands, cloud agents or GitHub writes outside mediated
channels. Protected remote credentials/branch enforcement are separately
required if the owner expects remote effects to be impossible to bypass.

## Anti-rollback without new hardware

Active AuthorityLease objects are **volatile** and held only in the
protected service's process memory. Persist audit/checkpoint records and
terminal consumption/revocation history in a protected transactional store,
not reusable ACTIVE capability files. Do not load or migrate old sealed
ACTIVE leases as authority. A crash/restart creates a fresh unpredictable
service generation and invalidates every prior lease. Any uncertain request
or consumption remains terminal/non-authorizing; no automatic repost or
consume. Fresh authority is required at the next task start.

For every operation check service generation, live lease state, expiry
(monotonic elapsed clock plus bounded wall-time checks), actual scope, current
task/session and protected state under the same trusted operation boundary.
Cancel/finish/revoke removes the lease from live authority; old files or
restored IPC messages never recreate it. Protected transactions bind
requestId/executionId/actionDigest/grantDigest and journal disposition
uniquely. Audit and evidence are not bearer credentials.

Plan closure needs a durable protected transaction, not a volatile lease.
Persist consume-intent before the HAA call; after verification and exact
state recheck, atomically commit the protected canonical plan transition and
grant digest, then produce a read-only ledger projection. A crash cannot
promote an ambiguous closure to DONE. A changed/missing projection or
journal fails closed; no automatic recovery by importing an older snapshot.

This design avoids a separate hardware monotonic counter for the
demonstrated same-UID attacker: that attacker cannot write the protected
store, and an old ACTIVE file is never a source of authority. It does NOT
prove resistance to a privileged snapshot rollback. If that becomes an
explicit requirement, introduce an independent monotonic anchor in a new
ADR with proof; do not repurpose HAA internals or silently add VEL.

## Alternatives

- HMAC + chmod under UID 501: rejected, empirically bypassed.
- Keychain-only key storage: insufficient; authentic old records still replay.
- Root daemon executing agent commands: rejected, privilege amplification.
- Separate VM or remote authority store: possible fallback, more cost and
  setup; not baseline without owner decision.
- Dedicated service identity plus volatile leases: preferred, lowest
  complexity consistent with the bounded attacker model; host proof pending.

## Gates and RRI

Historical RRI 100 evidence is retained. The owner waived development RRI
scoring and numeric routing/decomposition gates for this integration only
on 2026-11-24. No runtime RRI/HITL rule is changed. Risk analysis, bounded
acceptance criteria, appropriate decomposition, tests and architecture
acceptance recorded; technical gates remain required. Neither waiver substitutes for isolation.

No enforcement, merge, real closure or additional Touch ID now. Require
protected deployment proof, actual runner integration and W2 compatibility
before integrated E4. No new branches or CI.
