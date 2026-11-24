---
type: Audit
status: final
date: 2026-11-24
---

# S4.1–S4.3 — Official Python runtime verification and G0 (2026-11-24)

HEAD: `74e834dfa596e5ef73878dd9aec36df77cfbc7f8` (clean). Unprivileged
inspection only; no sudo, no provisioning, no HAA calls, no Touch ID.

## Runtime provenance — PASS (provenance only)

| Check | Result |
|---|---|
| Path | `/Library/Frameworks/Python.framework/Versions/3.14/bin/python3.14`, regular file |
| Version / arch | 3.14.10 (`v3.14.10:b703b2c5e70`), universal2 (arm64 + x86_64), OpenSSL 3.5.9 |
| Signature | Developer ID Application: Python Software Foundation (BMM5U3QVKW), hardened runtime, `--strict` valid, satisfies DR |
| Receipts | `org.python.Python.PythonFramework-3.14` (+ Documentation, Applications, UnixTools) |
| Receipt BOM integrity | 3471/3471 BOM files present, CRC matches (AppleDouble entries excluded); CRC is drift detection, not cryptographic proof |
| Deep seal | Not clean: 5662 `file added` only (ensurepip `pip`, docs); zero modified/missing |
| `-I` flags | isolated=1, no_user_site=1; sys.path = framework stdlib + framework site-packages (pip only) |
| ACL entries | none |
| SHA-256 executable | `8255b46f6de3ed938594a87dd24c77b548888fcfb4c94b63a28990dbc5c40db6` |
| SHA-256 `Versions/3.14/Python` | `911c6fc490da724c682a88115498b4058296a63d3d9167f193409b3b1569880d` |

## Runtime protection — BLOCKED

The python.org installer sets the framework `root:admin` with group write
(executable `0775`; 9766 entries under `Versions/3.14` are group-writable;
`Versions/` is `root:wheel 0775`). The owner account (UID 501) is a member
of `admin`, so it can modify the interpreter and stdlib **without
authentication**. This fails the playbook requirement of a root-owned,
protected interpreter, stdlib and ancestor chain. The candidate code is
already fail-closed on it: `macos_verifier_launcher._root_owned` rejects
any `0o022` bit in the chain and would raise `UNTRUSTED_LAUNCHER_INSTALL`.

## Dependency environment — PREPARED (unprivileged)

Resolved from `scripts/haa/requirements.txt` (`cryptography>=45,<46`),
binary-only, downloaded unprivileged; every SHA-256 matches the PyPI JSON
digest. Hashed lock: `scripts/haa/requirements-cp314-macos-arm64.lock.txt`.

| Wheel | SHA-256 |
|---|---|
| cryptography 45.0.7 cp311-abi3 universal2 | `3be4f21c…6a00cee` |
| cffi 2.1.1 cp314 arm64 | `661c298b…d6a72` |
| pycparser 3.12 py3 | `a076f3f4…c85e` |

Owner environment's cryptography 50.0.2 was not reused.

## G0 — PASS (candidate tests)

Throwaway venv from the official interpreter, `--no-index --require-hashes`,
clean environment, `-I`: `certify_candidate.py` → 135 tests, 134 pass,
1 skip, 0 failures, 20.1 s; status `PASS_CANDIDATE_TESTS`; activation
`NO_GO`, host isolation `NOT_TESTED`, backend `NOT_CERTIFIED`.

## G1 inventory (read-only) — no collision

`/Library/DubBridgeHAA` absent; no `dubbridge`/`haa` users or groups;
`/etc/sudoers.d` empty. Effective sudo policy unreadable without the owner's
password (`sudo -n -l` refused) — not verified.

## Gate status

- G0: PASS. G1: **BLOCKED** (runtime protection; admin authentication
  required and owner-typed). G2/G3 provisioning probes: not started.
- Activation: NO_GO (unchanged).

## Owner decisions required before G1

1. **Runtime protection.** Either remove group write from the 3.14
   framework (`chmod -R g-w` on `Versions/3.14` and `g-w` on `Versions/`,
   owner-typed admin auth; any later python.org update/reinstall re-adds it
   and is drift requiring re-certification), or remove UID 501 from
   `admin` (not recommended — breaks owner administration). No fallback
   to CLT 3.9 and no wrapper.
2. **Dependency location.** The helper runs the pinned framework
   interpreter with `-I`, which only imports from the framework's own
   site-packages. Reviewed wheels must therefore be installed there
   (administratively, from the hashed lock, no index) or the launcher must
   be changed to a separately protected pinned environment. Not decided.


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

## G1 execution checkpoint at 6554495 (2026-11-24)

- **Runtime protection: PASS (re-checked).** `Versions/` and `Versions/3.14`
  tree: 0 group/world-writable non-symlink entries, 0 non-root owners, no
  ACLs, no absolute external symlinks. Executable and `Versions/3.14/Python`
  SHA-256 unchanged from the recorded values. `-I` import path is limited to
  the framework stdlib, lib-dynload and site-packages. Original modes are the
  ones recorded above (`0775`, 9766 group-writable entries); the change was
  applied by the owner before this checkpoint.
- **Dependencies: STAGED, not installed.** Wheels re-downloaded unprivileged
  with `--require-hashes --only-binary :all: --no-deps`; all three SHA-256
  match the lock. Unpacked unprivileged: 154 regular files / 25 dirs, no
  `.pth`, `sitecustomize`/`usercustomize`, symlinks or collisions with the
  existing site-packages; console script `bin/cffi-gen-src` excluded. An
  enumerated root install (system `install`/`shasum` only, per-file hash
  verified at destination, collision and ancestor aborts) and exact rollback
  were prepared and dry-run unprivileged (PASS, cryptography 45.0.7 imports).
  Manifest SHA-256 `2683113a…c4b824`.
- **G1 provisioning: PREPARED, not executed.** Inventory unchanged: no
  `/Library/DubBridgeHAA`, no HAA users/groups, `/etc/sudoers.d` empty,
  UIDs/GIDs 471–473 free. Planned: hidden non-login non-admin
  `_dbhaaexec` 471, `_dbhaaagent` 472, `_dbhaaverify` 473 (primary group
  `_dbhaaexport` 473, members executor+verifier only); layout root 0755,
  state/target executor 0700, scratch agent 0700, exports executor:export
  0750, verifier-work verifier 0700; helper = `verify_launch.py` blob
  `22aaba83…f968` as root 0755; one `visudo`-validated rule
  `_dbhaaexec ALL=(_dbhaaverify:_dbhaaexport) NOPASSWD: NOSETENV: verify-launch ""`;
  synthetic canaries only; pre/post effective sudo policy dumped as root.
  Exact rollback removes only these resources.
- **Blocker:** every remaining step needs owner-typed administrator
  authentication (`sudo -n` refused; this agent session is non-interactive
  and must not collect passwords). G2 probes and the regression under the
  installed site-packages depend on it. G3 = this record.
- Activation: NO_GO. Host isolation: NOT_TESTED. No HAA calls, Touch ID or
  real credentials.

## G1–G3 executed (2026-11-24, code HEAD 10c1fc5)

Admin authentication was typed by the owner into the native macOS
authorization dialog per step; the agent never handled it. No HAA calls,
Touch ID, real credentials or network tests.

**Stage A deps: PASS.** 154 files / 25 dirs installed root:wheel into the
3.14 site-packages, each SHA-256 verified at destination; no collisions or
startup hooks. Post-install: 0 group/world-writable or non-root entries in
`Versions/3.14`.

**G0 regression (installed deps, `-I`, non-root UID): PASS** — 135 tests,
134 pass, 1 skip, 20.6 s; python 3.14.10, cryptography 45.0.7.

**G1 provisioning: PASS.** Manifest:

| Item | Value |
|---|---|
| `_dbhaaexec` | uid 471, gid 471, + `_dbhaaexport` |
| `_dbhaaagent` | uid 472, gid 472 (not in export group) |
| `_dbhaaverify` | uid 473, primary `_dbhaaexport` 473 |
| All three | hidden, shell `/usr/bin/false`, home `/var/empty`, password `*`, not admin |
| Layout | root 0755 root:wheel; bin 0755 root; state/target 0700 exec; scratch 0700 agent; exports 0750 exec:export; verifier-work 0700 verifier |
| Helper | `bin/verify-launch` root:wheel 0755, SHA-256 `22aaba83…f968` |
| Sudoers rule | `/etc/sudoers.d/dubbridge-haa` 0440, SHA-256 `4589809f…b57f`; `visudo -c` OK |
| Effective policy | executor: only `(_dbhaaverify : _dbhaaexport) NOSETENV NOPASSWD: verify-launch ""`; agent and verifier: no sudo. Pre-existing policy: `root` and `%admin` ALL only |
| Code for probes | `git archive` of HEAD `scripts/haa` + synthetic driver, extracted root:wheel 0755/0644 at `code/` |

Note: macOS adds the new accounts to `everyone`, `localaccounts`,
`_lpoperator` and `com.apple.sharepoint.group.1` automatically; not
evaluated for file-sharing exposure.

**G2 empirical probes (real identities, synthetic data): 48/48 PASS.**

| Probe | Result |
|---|---|
| Agent → executor canaries, state, target repo refs, install root, bin, helper replace/rename/delete, code, site-packages | Denied (12/12) |
| Agent → exports, export file, verifier-work | Denied (4/4) |
| Verifier → exact export read | Allowed; write/create/rename/delete/root-write denied (5/5) |
| Verifier → executor state/target, agent scratch, helper write | Denied (5/5) |
| Elevation: agent/verifier any sudo, agent→helper, executor→root, executor→verifier shell, helper with args, SETENV | Denied (7/7) |
| Real `ExactTreeVerifier`+`MacOSVerifierLauncher` valid command | exit 0 observed; command saw uid/euid/gid/egid 473/473/473/473 |
| Nonzero command | exit 7 observed, no PASS substitution |
| Wrong path / escaping symlink / wrong identity / extra argument / bad helper source | `INVALID_LAUNCH_SOURCE` ×2 / `LAUNCH_IDENTITY_CHANGED` / sudo rc 1 / helper 125 |
| Dirty worktree vs exact tree | Export = committed tree; dirty and untracked excluded |
| Timeout → fresh instance | `TimeoutExpired`; marker persisted; instance latched; fresh launcher `VERIFIER_QUARANTINED`; no retry |

**Findings / not certified**
- **Residual process on timeout (FINDING).** Timeout kills `sudo` only; the
  verifier child (`python -I sleep.py`, uid 473) kept running and exited by
  itself ~22 s later. Quarantine correctly held, but no descendant cleanup or
  process containment exists. Marker `state/verifier-in-flight` is left for
  operator reconciliation.
- By design, the helper's fixed PATH lets bound commands run `/bin/sh`;
  restricting commands relies on the executor's command-digest binding.
- **Not tested empirically (BLOCKED):** volatile lease revoke/restart with old
  disk bytes (unit-tested only); actual runner / protected Git ref integration
  (not wired; FS-level agent write to target refs denied); network
  restrictions (none configured, no containment claim).
- Synthetic artifacts retained: exports `verify-probe` plus one export kept
  from the timeout run, 4 `verifier-work/run-*` dirs, `target/repo`, `code/`.
  The G1 rollback script removes only created resources; `code/`, retained
  exports/work dirs and the marker need manual reconciliation first.

Gate status: G0 PASS, G1 PASS, G2 PASS with the findings above, G3 recorded.
Host isolation: partial (filesystem/identity/sudo boundaries PASS; process
containment and network NOT certified). Activation: **NO_GO**.
