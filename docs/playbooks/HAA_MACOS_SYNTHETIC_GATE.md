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


## Owner-approved official Python runtime (supersedes custom runtime path)

Owner reports installation of the official python.org macOS distribution.
Use that installation instead of downloading/building a standalone runtime
or adapting HAA to the Apple CLT Python 3.9. No extra runtime copy is needed.
Require a supported Python >=3.11 (3.14 preferred), arm64 support and exact
local version/provenance verification before use. Expected framework:
`/Library/Frameworks/Python.framework/Versions/<major.minor>/`.
Resolve the actual executable locally; do not assume a particular version,
use Current, /usr/local/bin, Homebrew, a mutable wrapper or a symlink as
the trusted executable. Verify the canonical executable is a regular file,
root-owned, with protected ancestor directories, framework, stdlib,
dependency/import paths and ACLs. Check installer receipt, applicable
signature validation and record exact version and hashes of the runtime
and dependency manifest. An official package alone is not isolation proof.
Do not use user site packages or writable PYTHONPATH. Runtime/dependency
drift requires a new regression and host certificate before activation.

Reconcile the fixed interpreter check in macos_verifier_launcher.py,
the verify_launch.py shebang and its trusted executable search paths to
the actual verified canonical installation path. Make these small
changes in the existing branch and update corresponding fixtures.
Do not create runtime/bin/python3 symlinks or wrappers; the direct
canonical framework path replaces that old layout requirement.
Use a separately protected, pinned dependency environment; resolve
cryptography according to scripts/haa/requirements.txt (currently
>=45,<46), do not silently reuse the owner environment's 50.0.2.
Prepare verified wheels unprivileged; install only reviewed artifacts
administratively, no pip or repository scripts as root. Run the full
candidate suite under the selected runtime/dependencies, then resume
the approved synthetic G1/G2/G3 gates. If any provenance or isolation
condition is unmet, BLOCKED, no fallback to CLT 3.9.

All previous security restrictions remain: no actual HAA credentials,
Touch ID, enforcement, broad sudo, new branches, CI or development RRI.
No repeat runtime download, no compatibility rewrite, no additional
reviewer. The user's local G0 report at 38534bc was 134 PASS, one skip
(135 tests), but did not certify the newly installed official interpreter.

## Confirmed official interpreter path

Owner reports Python 3.14.10 at `/Library/Frameworks/Python.framework/Versions/3.14/bin/python3.14`, executable owner UID 0.
Launcher and helper now use this fixed path; no code adaptation to 3.9,
wrapper or standalone download. Ownership of the executable is preliminary
evidence only. Still verify full framework, signing/receipt, ACLs, dependency
provenance and runtime integrity. Test the candidate using this interpreter
with compatible pinned dependencies before actual provisioning. The helper
uses isolated Python mode for direct Python commands. The fixture suite has
not been rerun after this path change. Do not infer a security certificate
from the reported executable UID. All prior NO_GO conditions remain.

## Runtime verification result (2026-11-24)

G0 PASS; provenance PASS; protection BLOCKED (group-admin-writable framework).
Stop before G1 until the owner decides runtime hardening and dependency location:
`docs/audit/haa-s41-official-runtime-verification-2026-11-24.md`.


## S4.1 decisions on runtime protection and dependencies

Orchestrator decision following the local evidence at 8283845:
GO conditional on explicit owner administrative action on the installed
Python 3.14 framework, and on successful protection checks. Do not remove
UID 501 from admin. Do not download another Python.

Before any modification, record original modes and verify canonical
paths, ownership, ACLs and symlink boundaries. Limit hardening to
/Library/Frameworks/Python.framework/Versions/3.14 and the Versions
directory itself: remove group-write bits, never follow external
symlinks or change other Python versions. Do not make broad changes
to /Library or shared system directories. Owner types administrative
authentication. If any other unsafe ancestor remains, report BLOCKED
rather than broadening the change automatically. Recheck the complete
runtime and import path, including site-packages, after installation.

Use the existing official framework site-packages rather than introducing
another runtime or launcher layout. This is a shared Python installation:
do not overwrite existing packages, pip or stdlib. Stage binary wheels
unprivileged from requirements-cp314-macos-arm64.lock.txt using hashes,
no index and no root pip. Install only explicitly enumerated, verified
regular files/directories as root-owned non-group/world-writable data.
Do not execute any repository code or package installation hooks as root.
Inspect staged .pth files and startup hooks; unexpected executable startup
content or collisions block installation. Record the installed dependency
manifest and rerun the full regression under the actual interpreter with
isolated mode, as a non-root identity. Framework/dependency updates or
drift invalidate certification and require recertification. The launcher
continues to reject writable ancestors.

These decisions do not authorize weakening the checks, activating HAA,
using real credentials or Touch ID. Continue the previously approved
synthetic gate; stop only at a genuine blocker. G0/provenance PASS is
not host isolation PASS.
