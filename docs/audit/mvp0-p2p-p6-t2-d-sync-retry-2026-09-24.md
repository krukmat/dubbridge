---
type: Audit
title: "P6.T2.D — viewer Sync / Retry Sync"
status: pass
task: P6.T2
block: T2.D
date: 2026-09-24
---

# P6.T2.D — Sync / Retry Sync

## Result

**PASS 2026-09-24.**

Implementation head: `a99a712ca86bd6686fccca373c4e4d285b1bbeee`.
GitHub Actions run: `36025590927` — **15/15 PASS**.
Mobile: **64/64 suites, 495/495 tests**; `InvitesScreen.test.tsx` PASS; P3 T2c3 self-check PASS.

## Boundary

T2.D does not implement a sync state machine. It only bridges the already-frozen
T2.B projection to the existing P4 façade:

```text
T2.B projection
  Pending + exact descriptor ── Sync ──┐
  FAILED                   ─ Retry ────┤
                                      ↓
                       P2PSyncController.startSync()
                                      ↓
                      P4 cache / verified sync state
                                      ↓
                          T2.B refresh + projection
```

P4 retains discovery, download, verification, retry and READY/FAILED authority.

## Implemented behavior

- `projection.action === sync` exposes **Sync**.
- `projection.action === retry_sync` exposes **Retry Sync**.
- The action handler revalidates:
  - current authenticated `auth.userId`;
  - matching viewer subject;
  - active authorization;
  - exact asset/publication/lineage descriptor;
  - allowed Sync/Retry projection.
- `accountScope = auth.userId`.
- Calls only `P2PSyncController.startSync(descriptor, accountScope)`.
- Per-descriptor key `publicationId/lineageId` prevents duplicate in-flight jobs.
- Completion refreshes the authoritative inbox and P4 snapshot.
- A thrown startSync remains fail-closed, displays a row error, refreshes P4 state
  and never fabricates Available or Play.

## Behavioral evidence

| Case | Evidence | Result |
|---|---|---|
| Pending + exact descriptor → Sync → P4 state re-projection | `InvitesScreen.test.tsx` | PASS |
| FAILED → Retry Sync → P4 state re-projection | `InvitesScreen.test.tsx` | PASS |
| inactive authorization → no Sync | `InvitesScreen.test.tsx` | PASS |
| mismatched descriptor → no Sync | `InvitesScreen.test.tsx` | PASS |
| duplicate press → one P4 start for descriptor | `InvitesScreen.test.tsx` | PASS |
| startSync error → fail-closed / no Available / no Play | `InvitesScreen.test.tsx` | PASS |

## Reflection log — RRI 55 / Med-high

1. **Authority pass:** PASS. UI action eligibility is derived from T2.B; P4 remains
   the sole sync-state authority and the handler revalidates identity/descriptor.
2. **Concurrency/failure pass:** PASS. The descriptor-level in-flight guard prevents
   duplicate jobs; error paths do not synthesize local authority.
3. **Scope pass:** PASS. No P4 runtime/cache/state-machine code changed and no
   T2.E playback wiring was introduced.

The standing MVP0-P2P owner review exception remains applicable. The owner
explicitly approved execution of T2.D before implementation; aggregate parent
owner verification is intentionally retained for T2.H.

## Gate corrections

CI exposed two quality/type issues and both were corrected without relaxing behavior:

1. `useInvitesActions.ts` exceeded the mobile declaration budget (22 > 20) →
   trivial alias/helper consolidation;
2. TypeScript required widening the test descriptor fixture and explicitly
   narrowing optional `auth.userId` before P4 delegation.

## Out of scope / next

- no Play wiring;
- no new P4 state machine;
- no background/offline expansion;
- HPKE emulator remains disabled.

Next leaf: **T2.E — Available + Play via verified P4 handle + existing P5 playback**.
