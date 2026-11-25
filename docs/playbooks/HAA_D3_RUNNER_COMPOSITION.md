---
type: Playbook
status: candidate
date: 2026-11-25
---

# D3 bounded runner composition

Only a protected supervisor may request/consume a grant and start an active
ProtectedSession. The existing worker runner runs as UID 472 in a separate
scratch checkout using its normal card/model/command/route limits. Its
output is not a trusted test result. Do not lower or bypass the existing
runner restrictions. A protected card is pinned to the lease; the worker
copy is not an authority input. Agent runner code must never execute as
UID 471 or root.

The first candidate deliberately supports ONE existing regular UTF-8 file
replacement. Kernel peer UID and fresh window bind the request. After the
first accepted edit the broker seals its window, returns trusted path and
SHA-256 evidence to the protected session, closes its listener, and denies
other proposals. No accepted proposal means no completion. The supervisor
cannot choose a different commit path or content; ProtectedCommit rechecks
the accepted digest during exact tree construction and verification. All
other paths stay at their committed values. Plan completion still requires
observed test exits, Git ref CAS, journal transaction and lease revocation.
No automatic retry after ambiguous side effects.

The actual worker launch, dedicated socket group/ACL and protected owner
supervisor entry point must still be composed and tested. Never provide
agent UID with executor credentials or access to the protected target. The
current owner/admin account is not an agent isolation boundary; its
unattended elevation and existing GitHub credentials are outside this
mediated local target guarantee. Do not claim arbitrary remote Git writes
are impossible.

D3.1a design and D3.2a broker-to-commit binding prepared. D3.1b actual
runner entrypoint integration remains partial. No host activation.

## Supervisor composition candidate

ProtectedSupervisor serializes task request, poll, activation, one bounded
worker window, exact verified commit and independent closure controller.
worker_invocation produces fixed isolated bootstrap argv for the existing
runner under the agent UID. No owner or executor identity transition is
implemented in this module. Actual launch and runner/model transport are
still host integration obligations; the mocked supervisor regression does
not prove real runner or process isolation. Failure latches the supervisor
closed and prevents closure. No production entrypoint activated.
