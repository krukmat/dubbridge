---
type: Audit
title: "MVP0-P2P P1.A1b phase-1 task-analysis packet"
task: P1.A1b
phase: task-analysis
status: reviewed
date: 2026-08-30
---

# Phase-1 task-analysis packet — P1.A1b

Review this task for presentability only. Do not propose or write a patch.

## Frozen task-ledger scope

- Objective: open an empty transient Hyperdrive/Corestore drive strictly below
  the proof cache root, return a receipt, and close it deterministically.
- Allowed paths: `mobile/src/p2p/runtime/**`.
- HP-A1: a validated temporary path opens Corestore/Hyperdrive and returns only
  `capability` (non-null) and `schema_version`, then cleanly closes.
- Exclusions: no Hyperswarm/discovery/product persistence.
- Evidence: HP-A1 proof log and exact bundle/version record.

## Verified baseline

- The runtime protocol defines only `HANDSHAKE`, `PING`, `SHUTDOWN`, lifecycle,
  and fatal commands. It has no open/close operation or receipt validator.
- The worklet currently imports only `bare-rpc` and the local protocol. It has
  no cache-root source, filesystem adapter, Corestore, or Hyperdrive handling.
- Installed Hyperdrive documentation requires `new Corestore('./storage')` and
  `new Hyperdrive(store)`; Corestore's close path closes its storage.
- Existing unit tests cover protocol/lifecycle behavior but no Corestore or
  Hyperdrive open/close behavior.
- The committed worklet bundle is generated from `worklet.ts` and must be
  regenerated when the worklet changes.

## Current RRI evidence

The minimal verifiable surface is protocol, worklet, generated bundle, and a
focused runtime test. `scripts/rri.py` produced RRI 53 (Med-high), recorded in
`docs/audit/mvp0-p2p-p1-a1b-rri.md`.

## Presentability assessment requested

Determine whether the ledger is sufficiently specified for a Med-high approval
card. In particular, assess whether it must freeze:

1. the RPC command and receipt schema;
2. the trusted source and validation rule for the proof-cache root;
3. the Corestore storage adapter/import contract in the Bare bundle;
4. close ordering and how failures are surfaced; and
5. the test path and an EC for an invalid/out-of-root path.

Return PASS only if all mandatory acceptance and scope facts are already frozen.
