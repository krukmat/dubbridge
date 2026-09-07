---
type: Audit
title: "MVP0-P2P Bare runtime generic naming cleanup RRI"
task: P0
date: 2026-08-27
---

# MVP0-P2P — Bare runtime generic naming cleanup

The repository owner requested that runtime code no longer encode the temporary
task identifier `P0`. This is a behavior-preserving, Android-only naming cleanup
inside P0's already-approved mobile scope; task/ledger identifiers remain `P0`
as historical workflow records.

## RRI

**Command:**

```text
python3 scripts/rri.py --platform rn --cc 1 --touches mobile/App.tsx --touches mobile/app.config.ts --touches mobile/package.json --touches mobile/src/p2p/AndroidBareRuntimeProbe.tsx --touches mobile/src/p2p/bare-bridge.ts --touches mobile/src/p2p/bare-worklet.ts --touches mobile/__tests__/p2p/bare-bridge.test.ts --D 0 --K 0 --P 0 --T 0 --A 0 --X 3
```

**Result:** RRI **11** — Low / Effort S. The primary agent performed the direct
rename; no behavior, API, native configuration value, or security boundary
changed.

## Renamed runtime surface

| Previous temporary name | Generic runtime name |
|---|---|
| `BareP0Bridge` / `BareP0BridgeError` | `BareBridge` / `BareBridgeError` |
| `P0_WORKLET_*` | `BARE_WORKLET_*` |
| `AndroidBareProofBootstrap` | `AndroidBareRuntimeProbe` |
| `p0BareProof` / `EXPO_PUBLIC_P0_BARE_PROOF` | `bareRuntimeProbe` / `EXPO_PUBLIC_BARE_RUNTIME_PROBE` |
| `p0:android` | `android:bare-probe` |

## Verification

- `cd mobile && npm run typecheck` — PASS.
- `cd mobile && npm test -- --runInBand __tests__/p2p/bare-bridge.test.ts` — PASS
  (5/5 Bare bridge tests).
- `cd mobile && npm run lint` — PASS.
- Android 34 ARM64 development build installed with the generic probe flag and
  logged `[Bare runtime probe] ping=pong` and `shutdown=complete`.

P0 remains `AWAITING_OWNER_FINAL_VERIFICATION`; this cleanup neither closes P0
nor activates P1.
