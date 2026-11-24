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
