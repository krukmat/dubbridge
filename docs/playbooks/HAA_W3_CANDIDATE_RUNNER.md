---
type: Playbook
status: candidate
---

# W3 candidate controller (B1–B6)

`scripts/haa/runner_authority.py` provides a candidate trusted preflight,
lease/card validation, existing boundary composition, operation lock and
revocation on close. The command isolation interface intentionally has no
default implementation. Unknown/protected operations are not supported.
Candidate tests cover low/high-band, modified or unbound cards and existing
scope rejection.

**No live runner wiring is claimed.** The direct and normalized CLI paths
still use the historical workflow; the common runner must be installed
behind a genuinely trusted controller before enforcement can be activated.
An injectable controller or Python lock is not a process isolation boundary.
No callback or isolation adapter may be supplied by agent text. The candidate
controller currently does not implement commit, task-start acquisition,
real OS isolation, or a production dispatcher. Its edit callback must be
owned by trusted file tools and will need the host boundary for TOCTOU safety.
The existing CLI remains untouched until these prerequisites are resolved.
