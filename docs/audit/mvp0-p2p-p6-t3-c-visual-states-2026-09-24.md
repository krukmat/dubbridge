---
type: Audit
title: "P6.T3.C — loading, empty and error visual pass"
status: in_progress
task: P6.T3
block: T3.C
date: 2026-09-24
---

# P6.T3.C — loading / empty / error visual pass

## Activation finding

DESIGN.md requires operational loading, empty and error states to center cleanly
and remain usable on mobile. The shared `StateView` implementation explicitly
requires a growing parent when nested in a ScrollView.

Current drift:

- My Content already used a ScrollView but its content container did not grow.
- Invites used a static screen despite variable-length invitation and playback
  content, so longer content had no scroll path.

## Bounded correction

- `MyContentScreen`: keep `scroll`; add `contentContainerStyle` with
  `flexGrow: 1`.
- `InvitesScreen`: enable `scroll` and the same `flexGrow: 1` contract.
- Preserve existing loading/empty/error/retry copy and `StateView`.
- Add component assertions proving both screens retain the grow contract.

No backend/API/schema, auth, sync verification, playback or action-eligibility
logic is modified.

## RRI

C=1, F=2 for production source plus bounded test/docs evidence, D=0, T=1,
A=0, K=1, P=0, X=1. Technical bottleneck remains 1, ICI 25; risk/domain input
is below that anchor.

**Final RRI: 25 / Low.**

## Expected evidence

- exact-head mobile typecheck/lint/tests;
- full repository CI;
- existing retry/empty tests remain green;
- new layout assertions remain green.

Final Android device screenshots belong to T3.D, not this block.
