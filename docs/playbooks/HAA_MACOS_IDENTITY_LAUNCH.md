---
type: Playbook
status: proposed-deployment
date: 2026-11-24
---

# S4.1 — macOS identity launch candidate

Inventory at 3de677f: only non-system UID 501 detected, admin member;
current Python owned by 501. Regression 130 PASS / one skip. No install.

## Topology

Keep owner UID 501 for administration and interactive HAA/Touch ID.
Create dedicated executor, agent-worker, verifier identities with distinct
non-root UIDs, no login/admin access; allocate free UIDs/GIDs locally.
No current owner credentials or HOME shared with worker/verifier. No
passwordless elevation except the executor-to-verifier rule below.
Reserve verifier UID exclusively for one bounded verification at a time.

Proposed root-owned canonical installation base: /Library/DubBridgeHAA.
Root-owned bin/verify-launch (source scripts/haa/verify_launch.py), fixed
root-owned runtime/bin/python3 (regular executable, no symlink at this
path), protected runtime/stdlib/site-packages and executor code.
Install a verified Python >=3.11 and pinned dependency set; do not run
or copy a mutable user-owned virtualenv as root. Verify parent ownership,
ACLs, group membership, imports and install provenance before use.

Separate roots:
- state and target: executor private 0700, not writable/readable by agents;
- scratch: worker private 0700, dedicated account;
- exports: executor-owned 0750, dedicated verifier export group with
  executor and verifier only; the agent must not belong to this group;
- verifier-work: verifier-owned 0700, never reused as a source of authority.

Executor service identity is non-root (launchd UserName/GroupName).
The HAA server/credential isolation and signed owner approver app remain
distinct gates, not automatically solved by these three accounts.

## Fixed identity transition

Candidate MacOSVerifierLauncher uses only /usr/bin/sudo -n with numeric
verifier UID/GID, fixed root-owned helper, no command-line arguments.
Root-owned sudoers rule template (resolve names on host, inspect and
validate using visudo, never install this text blindly):

    <executor_name> ALL=(<verifier_name>:<verifier_group>) NOPASSWD: NOSETENV: /Library/DubBridgeHAA/bin/verify-launch ""

The empty quoted argument list means the helper can be invoked without
arguments only. The stdin payload is bounded and admitted by the trusted
executor. The helper checks its actual non-root UID/GID, exact read-only
export root, private scratch and command shape, resets environment and
execs the pinned command. No root target, sudo shell, ALL command grant,
agent permission, or shell parsing. Review actual effective sudo policy
(including other includes/group rules) before enabling even a test.
References: https://man7.org/linux/man-pages/man5/sudoers.5.html
and https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html
Verify syntax against the installed macOS sudo version.

The executor sees sudo/child exit status; it does not accept an agent
PASS JSON. An exclusive fsynced marker precedes launch and survives
timeouts, crashes and residual processes. Process-table checks on real
and effective UID run before and after. They are observations, not proof
of process containment or network isolation. Dedicated UID non-elevation,
no other services/login, accurate process visibility and trusted cleanup
must be validated on host. A residual or ambiguous state requires
administrator reconciliation; no automatic marker removal/retry.
Per-command scratch persists for controlled cleanup and must be bounded.

## Next one-time local gate

Privileged provisioning is the next unavoidable owner decision; no actual
install has happened. First prepare exact account/group allocation,
root-owned runtime source and dependency pin, and effective sudoers/ACL
checks. After the owner approves the concrete local changes, use only a
synthetic target and fake secrets. Prove UID isolation, executor/agent/
verifier access matrix, immutable export, no unexpected elevation,
actual exit 0 and nonzero, residual/timeout/restart quarantine, process
cleanup, and applicable network restrictions. Then run full suite and
capture exact HEAD. No real HAA credentials or Touch ID at this stage.

No production daemon, privileged installation, actual network sandbox,
credential provisioning or host isolation is certified by this patch.
Activation remains NO_GO.
