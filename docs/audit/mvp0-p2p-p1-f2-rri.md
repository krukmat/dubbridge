---
type: Audit
title: "MVP0-P2P P1.F2 mobile service ownership and composition RRI"
task: P1.F2
date: 2026-08-27
---

# MVP0-P2P P1.F2 — RRI assessment

## Command

```text
python3 scripts/rri.py --platform rn --cc 10 --touches mobile/App.tsx --touches mobile/src/navigation/RootNavigator.tsx --touches mobile/src/p2p/AndroidBareRuntimeProbe.tsx --touches mobile/src/p2p/bare-bridge.ts --touches mobile/src/p2p/runtime/BareRuntimeClient.ts --touches mobile/src/p2p/P2PService.ts --touches mobile/src/p2p/P2PProvider.tsx --touches mobile/__tests__/p2p/bare-bridge.test.ts --touches mobile/__tests__/p2p/p2p-service.test.ts --touches mobile/__tests__/p2p/p2p-provider.test.tsx --D 4 --K 4 --P 3 --T 4 --A 0 --X 4
```

## Result

The following is the unmodified Markdown output emitted by `scripts/rri.py`:

**Platform:** rn

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 10 -> score 1 (policy CC table) | High |
| F files | 3 | --touches -> 10 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 4 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 4 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 4 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 55
**Penalties applied:** none
**Final RRI:** 55 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered
**Advisory:** mobile/App.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/navigation/RootNavigator.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/AndroidBareRuntimeProbe.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/bare-bridge.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/BareRuntimeClient.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/P2PService.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/P2PProvider.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/bare-bridge.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/p2p-service.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/p2p-provider.test.tsx: no anchor-rubric match — agent judgment governs D/P/K

## Scoring notes

- **C1:** raw CC 10 bounds the created runtime-client, service-state, and
  provider snapshot paths. Recompute after the implementation diff exists.
- **F3/X4:** the frozen surface has ten files over the mobile composition,
  P0 compatibility, runtime/service/provider, and focused-test modules.
- **D4/K4:** Android Bare ownership, asynchronous lifecycle, React composition,
  and external-store notifications form a platform-specific coupled boundary.
- **T4:** P0 and F1 tests cover the existing bridge/protocol, but no focused
  service/provider ownership tests exist yet.
- **P3/A0:** the service is an internal product API with exact HP-F2/EC-F2,
  allowed paths, and exclusion boundaries; it does not expose network, public
  API, authentication, persistence, or user data behavior.
- No `refactor_and_behavior` penalty applies: F2 adds the ADR-043 seam while
  retaining the P0 probe/bridge source and the normal product remains
  network-inert. Parity migration/deletion is separately gated in P1.F3a.

## Required route

- P1.F1 is closed PASS, so P1.F2 may be presented. Current-session HITL
  approval remains required before source edits.
- RRI 55 follows ADR-038: after approval, restart Ollama and precheck Muse
  Glimmer, obtain its `GO_LOCAL` or `CLOUD_REQUIRED` advisory, and emit the
  primary hash-bound route receipt. Because 55 is in the 46–55 sub-band, the
  authorized implementation still proceeds through the selected cloud packet.
- The owner selected local reviews for F2: Gemma is the phase-1 and phase-2
  reviewer, with Muse Glimmer as the local fallback. The existing MVP0-P2P
  review exception is not invoked for this task. Three Draft → Critique → Revise
  Reflection passes, focused P0/service/provider tests, typecheck, lint, full
  Jest, coverage certification, owner verification, and status synchronization
  remain mandatory.

## Antares refinement touchpoint

`ANTARES-SKIP`: no task-relevant CWE hypothesis exists on the T3a watchlist.
F2 changes mobile composition and a local internal runtime/service boundary; it
does not add auth, secret, authorization, SQL, API-handler, or storage behavior.
