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
