---
type: Playbook
status: candidate
date: 2026-11-24
---

# S4.2/S4.3 — delta certification only

G1 PASS and 48 executed empirical probes PASS at dd7dca5. Do not repeat
account creation, sudoers installation or Python dependency installation.
Keep activation NO_GO. Work only on synthetic resources. No real HAA,
Touch ID or new privileges, no CI/new branch/reviewer. No development
RRI calculation.

## D1 timeout remediation and quarantine

Verify current UID 473 process table (real and effective UID) and record
previous timeout evidence. Keep old quarantine until the owner has
reconciled the incident, confirmed no residual process, and identified
the exact synthetic paths. Archive the marker and affected export/work
artifacts under protected state; no blanket removal or rollback.
Only owner-authorized administrative copying of reviewed root-owned
helper and trusted code bytes; no repository scripts executed as root.
Install v2 helper/launcher together; existing sudoers rule is unchanged.
Regression under official Python 3.14 with pinned installed dependencies.
Probe actual exit 0 and 7, ordinary child+grandchild timeout, reserved
124/125, fresh launcher denial and UID-wide residual check. A detached
child can escape a process group; if it does, retain quarantine and
report partial containment. Never infer all-descendant cleanup.

## D2 real process restart and revocation

Run scripts.haa.lease_process_restart_test as UID 471 from the installed
trusted code with a private synthetic temporary directory. The child
process uses os._exit after journal admission, then a fresh process
denies old ACTIVE bytes and duplicate admission; a second case revokes
before restart. Record actual UID, exact code hash, counts and result.
Use no actual grant or protected production journal. Reuse G1 access
matrix for worker denial; confirm worker cannot mutate the synthetic
journal. A privileged rollback is not covered.

## D3 runner integration (remaining implementation)

Actual runner must run under UID 472 in scratch, use a pinned task card
and kernel-authenticated broker socket, and submit a bounded proposal.
Preserve existing routing/limits; never bypass them to make a test pass.
Use a deterministic model fixture only, no real credentials or external
model request. Require actual edit guard, protected commit CAS, real
verifier UID 473 and canonical task evidence. Direct worker write to
protected Git refs must remain denied. Existing protected_integration_test
uses mocks and does not satisfy this gate. Not ready to claim PASS.

## D4 network and sharing

Read-only inventory: effective access from agent/verifier to synthetic
shared-folder canaries, membership of automatically assigned sharing
groups, and current network boundary. Do not inspect private user files
or transfer data to external endpoints. Use synthetic loopback test only
when it represents the intended policy; no real network containment
claim from it. A scoped network policy and allowed agent model transport
need design/validation before activation; no automatic PF/firewall
changes or removal of macOS-managed groups. Report BLOCKED where no
enforced policy exists.

D1/D2 candidates authored, D3/D4 remain open. Consolidate remaining
local validation once cloud implementation decisions are ready.


## Consolidated D3/D4 checkpoint — 2026-11-25

Source candidates: one accepted broker edit -> digest-bound protected
commit -> canonical plan completion; ProtectedSupervisor and fixed
worker_invocation; PlanValidation exact committed HEAD; private role
client configuration with no proxy/redirect. See
HAA_D3_RUNNER_COMPOSITION.md. D3 remains partial until real worker launch,
socket access, runner deterministic transport and owner-only control
are exercised; no production daemon or real key provisioning.
D4 policy and synthetic sharing probe prepared; see
HAA_D4_NETWORK_AND_SHARING.md. Actual enforceable network boundary
BLOCKED; no firewall change authorized by this source checkpoint.

Next local delta only after protected source artifacts reviewed:
- D1 reconcile prior timeout; install matched v2 helper and code,
  prove ordinary group cleanup and retain quarantine for escape/ambiguity.
- D2 run real process restart/revoke tests as executor.
- D3 exercise fixed worker command as UID 472 using normal runner
  routing and deterministic fixture, real broker and verifier UID 473;
  verify protected ref and canonical journal; no real grant.
- D4 record effective shared-canary access and supported network
  enforcement preconditions. Do not substitute a read-only probe for
  enforcement. No real credentials, Touch ID or activation.

New tests are authored but not executed at this checkpoint. No repeat G1,
no automatic rollback or existing quarantine deletion. A single local
intervention may certify D1/D2 and accessible D3/D4 probes, but cannot
close D4.1c without an actually enforced policy.


### D3 deterministic worker probe

scripts/haa/runner_fixture_probe.py provides a fixed synthetic model
sequence for the actual existing runner main and worker proposal path.
It is restricted to UID 472 and /Library/DubBridgeHAA/scratch/haa-delta-fixture.
Prepare a clean scratch Git repo with tracked README.md matching the
protected synthetic repo preimage. Put card.json and out.json outside
the scratch repo, under that fixture directory. The trusted protected
card and worker copy must match the same synthetic contract (task ID
haa-delta-fixture, Moderate band, README.md only, AC-1, /usr/bin/true).
Use an explicitly synthetic grant fixture through the existing admission
path; never fabricate authority for an actual task. Start the supervisor
and protected listener separately under UID 471; use the kernel peer
check and real verifier under 473. No source code is run as root.
Only the model chat and audit sink are fixtures; the runner performs its
normal Git scope, file tool and command checks. This driver has not been
run. It does not install an IPC group/ACL or daemon and does not replace
the actual worker-identity lifecycle certification.
