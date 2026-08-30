---
type: Audit
title: "P1.A1b proof-storage and RPC contract"
task: P1.A1b.0
status: accepted_for_downstream_task_presentation
date: 2026-08-30
---

# P1.A1b proof-storage and RPC contract v1

## Decision

P1.A1b uses a proof-only runtime factory, not `P2PService` or the normal
product `BareRuntimeClient`. The host constructs one run directory as:

```ts
new Directory(Paths.cache, "dubbridge-p2p", "proofs", runId).uri
```

`runId` is factory-generated and must match `^[a-z0-9]{8,64}$`. The factory
passes the resulting `file:` URI unchanged as the sole worklet start argument.
The worklet reads it at `Bare.argv[0]`. It is bootstrap configuration, never an
RPC request field, receipt field, log field, or product API parameter.

This preserves the Expo-owned cache authority while avoiding an unsafe URI/path
conversion. The installed `react-native-bare-kit` declaration supports
`Worklet.start(filename, source, args)`; Bare exposes those values as
`Bare.argv`. The Pear mobile guide uses the same host-to-worklet argument
mechanism with an Expo directory value.

## Runtime contract

| Item | Frozen value |
|---|---|
| Command | `OPEN_CLOSE_TRANSIENT_DRIVE` (next versioned runtime command) |
| Request | `{ protocolVersion: 1 }` only |
| Bootstrap | `Bare.argv[0]`: non-empty `file:` URI from the proof factory |
| Success receipt | Exactly `{ capability: "transient-hyperdrive-corestore", schema_version: 1 }` |
| Invalid bootstrap | `PROOF_STORAGE_CONFIG_INVALID`, redacted; no Corestore/Hyperdrive creation |
| Normal close | `await drive.close()` only; Hyperdrive closes its Corestore |
| Partial construction | `await store.close()` only when no drive exists |
| Network/product boundary | No Hyperswarm, discovery, replication, `P2PService`, or persistent product state |

The receipt deliberately omits directory strings, public keys, discovery keys,
and raw errors. P1.A1c retains ownership of the later granular dependency/open/
close error taxonomy.

## Dependency decision

`bare-fs@4.8.1` is currently transitive. P1.A1b does not import it directly:
the installed Hyperdrive package maps its Bare `fs` import to `bare-fs`, and
Corestore/Hyperdrive own storage access. Therefore no package or lockfile change
is authorized for P1.A1b. Its bundle check must prove this mapping resolves; a
failure is a dependency/bundle result for P1.A1c, not a reason to silently add a
direct dependency.

## Required source surface and tests

P1.A1b may change only the five paths listed in its revised ledger entry. Its
focused test must prove the exact `Bare.argv` argument and two-field receipt,
then prove invalid `runId`/bootstrap configuration is rejected before a storage
handle or network activity exists. An X28 device-runtime failure is classified
`Environment/Blocked`, never a source-test failure.

## Evidence basis

- Installed `expo-file-system@~56.0.8`: `Paths.cache` yields a `Directory` and
  `Directory.uri` is a `file:///` URI.
- Installed `react-native-bare-kit@0.15.0`: `Worklet.start` accepts string
  arguments; the worklet API passes them to Bare.
- Installed `corestore@7.12.2` and `hyperdrive@13.3.3`: Hyperdrive's close path
  closes its Corestore.
- Official references: [Expo FileSystem](https://docs.expo.dev/versions/latest/sdk/filesystem/),
  [Bare mobile guide](https://github.com/holepunchto/pear-docs/blob/main/guide/making-a-bare-mobile-app.md),
  and [Bare runtime API](https://github.com/holepunchto/bare).
