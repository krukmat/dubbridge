---
type: Playbook
status: candidate
---

# S1 protected macOS deployment contract

Use a dedicated non-login, non-admin service identity and a distinct
standard worker UID. The signed interactive HAA approver remains in the
owner session. No installation or account creation happens in this branch.

## Provisioning sequence (owner/admin, local S4 only)

1. Establish two distinct non-root UIDs; inspect group membership, sudo
   capability and ACLs. The worker must have no unattended elevation.
2. Provision a root-owned versioned installation from a reviewed exact
   commit, including trusted Python interpreter/dependencies. Do not use a
   user-writable Homebrew interpreter or imports. Configure launchd only
   after reviewing the precise service definition and paths.
3. Under an administrator-controlled directory chain, create separate
   executor-owned private state and target roots (0700), and a worker-owned
   private scratch root (0700). No shared writable parents or symlinks.
   Target Git configuration, hooks and credentials belong to the executor.
4. Provision distinct HAA role credentials outside the worktree using
   private service storage; never pass keys via CLI, IPC or agent env.
   Verify HAA server storage/signing identity and the signed approver app
   are protected as well. Do not copy private biometric material.
5. Validate using protected_host.validate_layout and read_private_file.
   These check necessary DAC conditions, not ACLs, sudo, running process
   identity, network policy, runtime imports or actual privilege isolation.
6. Run synthetic probes from the real worker identity: read, write,
   rename, unlink, replace runtime/target, key read, IPC peer mismatch,
   restart and stale lease. No real secrets or production repository.
7. Only after local evidence passes, configure the service endpoint and
   test the real runner paths. Installation commands and a launchd daemon
   must be reviewed against the actual host paths, UIDs and interpreter;
   no generic sudo command or automatic activation is provided.

## IPC contract

Unix-domain socket with kernel-authenticated peer UID (macOS getpeereid),
strict size-bounded JSON typed messages; service chooses active session,
scope, pin, request/execution IDs and actual state. Peer authorization
does not prove human approval. No generic command, path outside pinned
scope, caller-supplied grant/lease, raw shell or private key response.
Separate owner-only controls from worker proposals. A request from an
agent using the owner's UID must not acquire owner-only privileges.
A service backend must refuse startup if peer identity cannot be checked.

No protocol-v1 wire change. No activation, new user creation or key rotation
in remote development. The full host deployment is not yet certified.

## S4.0 expanded PASS / S4.1 inventory

Owner-reported 131 tests (130 pass, one skip), 11.607 s, clean HEAD
`3de677fd3f662a2f65343c47c49829a0d2fec08b`. See `docs/audit/haa-s41-host-inventory-2026-11-24.md`. Only UID 501 detected, admin member;
current Python owned by UID 501. Dedicated service/worker/verifier and
protected runtime must be provisioned. No real identity proof or activation.

## S4.1 deployment candidate checkpoint

The owner-reported regression at 3de677f remains PASS for that exact code
only. MacOSVerifierLauncher, non-root helper and four new mocked launcher
regressions authored at fd212d2db7882645bd6e345d4e8dc4d5f918de03; these new tests are NOT RUN.
See docs/playbooks/HAA_MACOS_IDENTITY_LAUNCH.md for the narrow non-root
identity transition and the proposed protected layout. UID/group allocation,
root-owned interpreter/dependency provenance, actual sudo/ACL policy and
process/network isolation still require a single consolidated local
provisioning intervention. Do not infer authorization to install generic
sudo rules or execute repository code as root. No actual host changes.
S3.7b implementation candidate prepared; certification remains open.
Activation NO_GO, no new Touch ID, no RRI/CI/independent reviewer.
