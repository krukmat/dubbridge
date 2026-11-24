---
type: Audit
status: candidate-regression-pass-host-provisioning-required
date: 2026-11-24
---

# S4.0 expanded regression / S4.1 inventory

Owner-reported exact clean checkout: 3de677fd3f662a2f65343c47c49829a0d2fec08b.
Python 3.15.2, cryptography 50.0.2. 131 tests in 11.607 s,
OK (skipped=1), exit 0: 130 PASS, one skip, no failures.
No separate test rerun or skip diagnosis is claimed here.

Darwin arm64; only detected non-system UID 501; current user belongs to
admin. /usr/bin/git, /usr/bin/python3, /bin/launchctl and /usr/bin/sudo
are present and root-owned. Current Python executable owner is UID 501.
The inventory did not check ACLs, sudo policy, interpreter dependencies,
actual service identities or credential isolation.

Decision: do not run protected execution with UID 501 or the user-owned
Python. Provision three dedicated non-root identities (executor, standard
agent worker, verifier), with a narrowly scoped verifier export group.
The existing owner keeps the interactive Touch ID app and admin role.
UIDs/GIDs must be allocated against the actual directory service; no
assumed numeric values. This is a deployment refinement, not three new
interactive accounts or additional hardware. No account creation now.

The new tree-export regression passed but actual separate-identity launch
and isolation remain uncertified. Do not activate enforcement, migrate
existing plans or perform another biometric ceremony. Do not infer
non-elevation from admin group membership or a successful sudo command.
