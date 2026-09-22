---
type: Prompt
title: "Historical continuation prompt — P4.T1-r1 product storage URI fix"
governs: "docs/tasks/mvp0-p2p-p4-mobile-sync.md § P4.T1-r1"
status: superseded
date: 2026-09-22
---

# Continuation prompt — P4.T1-r1

> **Superseded 2026-09-22. Do not execute this prompt as an active task.**
> P4.T1-r1 is already implemented and CI-verified. The former immediate
> P5.T3 device rerun was subsequently removed from the development critical
> path by `docs/audit/mvp0-p2p-p5-t3-sequencing-replan-2026-09-22.md`.

```text
CURRENT DISPOSITION

P4.T1-r1
- Implemented: file: URI -> filesystem path exactly once at the Corestore boundary.
- bare-url@^2.5.2 is a direct dependency.
- focused storage-path tests and current mobile gate pass.
- committed Bare worklet was regenerated and drift-checked.
- P4.T1-r1 requires no further source implementation.

SEQUENCING

Do NOT require an immediate P5.T3 Android rerun to continue development.

P5 now has two milestones:
- P5-DEV = P5.T0-T2 formally closed.
- P5-CERT = P5.T3 physical Android certification PASS.

P6 activation requires:
P3 PASS + P4 PASS + P5-DEV.

P5.T3 remains an open release-certification obligation, but it is outside the
development critical path. Compatible exact-RC evidence from T7p may close it;
otherwise P7.T2 must produce/resolve the physical SYNC -> VERIFY -> PLAYBACK
and no-remote-media-fallback evidence.

A failed or unresolved P5.T3 prevents P7 PASS and T9g GO.

AUTHORITATIVE REFERENCES
- docs/tasks/mvp0-p2p-p4-mobile-sync.md
- docs/tasks/mvp0-p2p-p5-local-playback.md
- docs/tasks/mvp0-p2p-p6-dashboard.md
- docs/tasks/mvp0-p2p-p7-certification.md
- docs/audit/p4-t1-r1-storage-uri-fix-evidence-2026-09-22.md
- docs/audit/mvp0-p2p-p5-t3-sequencing-replan-2026-09-22.md
- docs/audit/go-live-octubre-2026-mirror.html

REGLAS
- Nunca imprimir JWT, CK, envelopes ni tokens de invitación.
- No marcar P5 aggregate PASS por CI.
- No tocar transient-drive como parte de este cierre.
```
