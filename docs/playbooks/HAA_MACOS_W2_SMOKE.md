---
type: Playbook
title: "HAA W2 macOS Touch ID smoke"
governs: "physical W2 authority-adapter validation on the owner's MacBook"
---

# HAA W2 macOS Touch ID smoke

> **Scope:** W2 only. This gate proves authority acquisition and detached
> verification. It does not create an `AuthorityLease`, mutate DubBridge
> workflow state, commit, push, merge, release, deploy, or close a real plan.

## Required branches

Use these two worktrees/repos:

- DubBridge: `feature/haa-hitl-w1-contract-policy`
- HAA: `feature/dubbridge-action-profiles`

The default layout assumes sibling directories:

```text
<parent>/
  dubbridge/
  human-approval-authority/
```

If not, set:

```bash
export HAA_DUBBRIDGE_REPO_PATH=/absolute/path/to/dubbridge
```

## Prerequisites

- Mac with Touch ID / Secure Enclave configured;
- full Xcode with an Apple Development signing identity;
- Node.js 24+ for HAA;
- Python 3 with `venv` support;
- network access for the temporary pip install of
  `scripts/haa/requirements.txt`;
- HAA dependencies installed with `npm install`.

If multiple Apple teams are configured:

```bash
export HAA_APPLE_TEAM_ID=XXXXXXXXXX
```

## Run

From the HAA repository:

```bash
npm install
npm run validate:dubbridge-macos
```

The validator creates only temporary HAA state and an isolated Python venv. It:

1. runs the full DubBridge W2 authority unit gate;
2. starts an isolated HAA server;
3. provisions/registers one temporary Secure Enclave authenticator;
4. builds an exact `dubbridge.task.start.v1` action;
5. proves the executor is denied while the request is PENDING;
6. shows the trusted action and asks for Touch ID;
7. consumes and detached-verifies the task-start grant;
8. builds an exact `dubbridge.plan.close.v1` action over a synthetic closure
   manifest;
9. again proves pre-approval denial;
10. asks for a second Touch ID;
11. consumes and detached-verifies the plan-close grant;
12. confirms both HAA requests end in `CONSUMED`;
13. removes the temporary authenticator and validation state.

## What to inspect on Touch ID prompts

### Task start

Confirm the trusted display identifies:

- `START DUBBRIDGE TASK`;
- repository `krukmat/dubbridge`;
- the W2 smoke task/plan;
- the current branch and HEAD;
- allowed path `scripts/haa/**`;
- capabilities `review, test`;
- bounded lease TTL.

### Plan close

Confirm the trusted display identifies:

- `CLOSE DUBBRIDGE PLAN`;
- repository/plan/branch;
- current final SHA;
- `1/1` completed tasks;
- validation `PASS`;
- closure/evidence digests.

## PASS

Expected terminal line:

```text
PASS DubBridge W2: task-start + plan-close Touch ID authority smoke
```

Only this physical result may close W2.T2g.

## Fail-closed expectations

- Esc, close, timeout, challenge expiry, or interaction error must never produce
  a verified grant.
- A failed or missing Python crypto dependency stops before the ceremony.
- A PENDING request cannot authorize.
- A changed action/state after approval must be denied.
- HAA unavailability is a blocker, never an implicit approval.

## Troubleshooting

For macOS signing/keychain failures, follow HAA
`docs/MACOS-RUNBOOK.md`. In particular, `OSStatus -34018` means the app lacks
the required provisioned identity; do not weaken Secure Enclave or Touch ID
controls to bypass it.
