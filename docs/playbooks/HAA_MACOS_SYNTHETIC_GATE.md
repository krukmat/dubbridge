---
type: Playbook
status: approved-local-gate
date: 2026-11-24
---

# S4.1–S4.3 — Consolidated synthetic macOS gate

Owner approved proceeding with the bounded provisioning candidate at
362cdab. This does not authorize production activation, broad sudo access,
changes to existing accounts, real HAA credentials, or a physical ceremony.
Use the existing feature branch. Preserve user data and existing settings.
Development RRI suspended; no CI, new branch, merge or independent reviewer.

## G0 — regression (unprivileged)

From the clean checkout and prepared Python environment run
`python3 scripts/haa/certify_candidate.py`. Record HEAD and counts. The
131-test run at 3de677f was PASS; the newer launcher tests are not yet run.
Stop for regressions, and fix reproducible defects without weakening tests.

## G1 — one-time provisioning (owner/admin)

Read HAA_MACOS_IDENTITY_LAUNCH.md and HAA_PROTECTED_MACOS_DEPLOYMENT.md.
Inspect the actual directory service, available IDs, effective sudo policy
and existing /Library/DubBridgeHAA before making changes. If there is an
existing installation or an unexpected identity/name/permission collision,
stop rather than overwrite. Prepare an exact change manifest and rollback
list; allocate dedicated non-login non-admin executor, agent and verifier
identities and least-privilege groups. Preserve UID 501 and its credentials.
Use root-owned, verified Python >=3.11 and pinned dependencies; never
run repository scripts, user-owned interpreters or installers as root.
Do not weaken the identity boundary if a trusted runtime is unavailable.

Create only synthetic state/target/scratch/export/verifier-work under the
approved layout. Use fake canary files, no production repository or HAA
key material. Install reviewed helper bytes as a root-owned executable.
Install only the narrow executor-to-verifier, no-arguments sudo rule after
reviewing effective policy and validating syntax with visudo. No general
sudo access, root target, environment preservation or agent-to-verifier
delegation. No unattended password caching or password collection. The
owner types any requested administrator authentication directly.

Record the changes actually made, including IDs, modes and hashes (not
credentials or raw sensitive configuration). No persistent production
launchd service is started at this gate. If uncertain or any negative
probe allows access, stop; retain evidence and provide exact rollback for
only newly created resources. Do not remove existing users or files.

## G2 — empirical isolation / verifier provenance

Run fixed, synthetic probes from the actual identities, not mocked UIDs.
Use known non-sensitive canaries and a temporary Git repository.

| Probe | Expected |
|---|---|
| Agent access to executor canary, state, target and installation | No read of private data; no write/rename/delete |
| Agent access to verifier export or private scratch | Denied |
| Verifier access to exact export | Read succeeds; writes/rename/delete denied |
| Verifier access to executor private state/target | Denied |
| Agent/verifier unexpected elevation or delegation | Denied; inspect effective policy, do not execute a privileged shell |
| Executor fixed helper with valid request | Actual non-root verifier UID/GID; observed zero exit |
| Fixed helper nonzero command | Observed nonzero; no PASS substitution |
| Wrong identity, path, argument shape, escaping link | Denied |
| Dirty worktree versus exact Git tree | Export matches committed tree |
| Timeout/residual process and fresh launcher instance | Quarantine persists, no automatic retry |
| Volatile lease revoke/restart and old disk bytes | No recovered active authority |
| Actual runner and protected Git ref bypass | Direct worker cannot mutate protected target |
| Cleanup and applicable network restrictions | Evidence; no unproven containment claim |

Use controlled, bounded synthetic commands only. Do not test against real
credentials, network services or private data. Any ambiguous process cleanup
requires operator reconciliation; do not remove quarantine simply to pass.
A process-table observation alone does not prove all descendant containment.
If network restrictions, trusted runtime, ACLs or effective sudo policy
cannot be verified, report BLOCKED, not PASS.

## G3 — report

Provide exact HEAD, regression result, provisioning manifest, probe outcomes
PASS/FAIL/BLOCKED, residual process/rollback status, and remaining blockers.
Publish redacted evidence to the existing branch. Do not publish personal
paths, secrets, passwords, keys, biometrics or raw environment.
No real HAA calls, Touch ID or enforcement. S4.4 is separately gated by
review of S4.1–S4.3 and actual HAA credential/runner integration.
