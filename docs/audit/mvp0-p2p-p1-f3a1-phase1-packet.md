---
type: Audit
task: P1.F3a.1
phase: task-analysis
date: 2026-08-27
---

# P1.F3a.1 — Phase-1 review packet

Review a **prospective task card**, not an implementation diff. No source files
have changed. Return `PASS`, `FINDINGS`, or `BLOCKED` under the reviewer
contract, with findings only when the frozen task is unsafe, incomplete, or
internally inconsistent.

## Frozen goal

Replace the development-only P0 diagnostic host with
`P2PDevelopmentHarness`, wired only in `App.tsx`, so it invokes the existing
ADR-043 `P2PService` / `BareRuntimeClient` seam. Preserve every P0
characterization before the later, separate F3a.2 deletion task.

## Allowed changes

- `mobile/App.tsx`: replace only the P0 diagnostic component wiring with the
  new development harness; retain the existing configuration gate without
  editing configuration.
- `mobile/src/p2p/development/P2PDevelopmentHarness.tsx`: add the explicit,
  Android-only diagnostic harness using `useP2PService`.
- `mobile/__tests__/p2p/p2p-development-harness.test.ts`: add the P0-to-ADR-043
  characterization mapping and tests.

## Must preserve / must not change

- P0 probe, bridge, protocol, inline worklet, and all existing P0 tests remain
  byte-for-byte unchanged as the independent parity oracle.
- No deletion, renamed P0 import, configuration/dependency/script cleanup, or
  Android proof; those are F3a.2/F3b.
- No Hyperdrive, Corestore, Hyperswarm, discovery/network activation, product
  P2P API, persistence/identity, backend/HTTP/HLS, UI, or iOS work.
- Normal provider mounting stays inert. Only an explicit enabled Android
  harness may start a runtime; exactly one active owner must exist.

## Acceptance criteria

- **HP-F3a.1:** enabled Android harness completes `initialize → ping →
  shutdown` through the existing service/runtime seam, while P0 still passes
  unchanged.
- **EC-F3a.1:** malformed/remote replies, release during startup, and late or
  closed operations are verified as the replacement protocol's typed/redacted
  failures; no P0 case may be unmapped and no duplicate active runtime owner is
  allowed.
- Record the P0-to-ADR-043 behavioral map, an unchanged-P0 inventory, focused
  test output, typecheck, lint, full Jest, 90% direct-scope coverage, three
  Reflections, phase-2 review, and status synchronization at closure.

## Verified baseline facts

- `App.tsx` currently renders `AndroidBareRuntimeProbe` inside `P2PProvider`
  behind `Constants.expoConfig?.extra?.bareRuntimeProbe === true`.
- `P2PService` is inert at construction and supplies `initialize`, `ping`, and
  `shutdown` over one `BareRuntimeClient`.
- `BareRuntimeClient` implements typed versioned protocol errors and releases
  its worklet when startup/shutdown transitions fail.
- Existing `bare-bridge.test.ts` covers P0 lifecycle, malformed reply,
  startup release, and late reply; `p2p-provider.test.tsx` covers probe
  lifecycle/release behaviour. These files are reference-only in F3a.1.

## RRI and route

RRI 47 Med-high, no penalties. Phase 1 uses Gemma with Muse Glimmer then D14
as fallbacks. Explicit owner approval is required before implementation. At
RRI 46–55, post-approval Muse refinement plus the route receipt yields the
card-bound cloud implementer regardless of `GO_LOCAL`/`CLOUD_REQUIRED`.
