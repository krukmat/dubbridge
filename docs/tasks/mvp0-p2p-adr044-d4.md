---
type: TaskList
title: "Tasks: ADR044-D4 consolidated acceptance"
status: done
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-first.md
---

# ADR044-D4 — consolidated ADR-044 acceptance

## Scope

Review the fully consolidated ADR-044 after D1-D3, resolve only closure-blocking
inconsistencies, obtain/record owner disposition, and propagate the ADR status.
This task does **not** implement or approve P2 source work.

## Gate and owner authorization

- D1 `O3 parallel`: complete.
- D2 `K1`: complete.
- D3 `O4`: complete.
- Owner instruction on 2026-09-05: `trabaja con todo lo necesario para desbloquear P2`.
- The instruction is recorded as explicit authorization to execute this bounded D4
  closure and accept ADR-044 **only if** the consolidated review finds no blocking
  inconsistency. It is not authorization to implement P2.

## RRI

D4 is a governance/architecture closure task over documentation only. Conservative
parent score: **RRI 48 — Med-high — Effort L**. The score retains the architecture-
decision penalty even though D1-D3 already froze the substantive choices, because
changing an ADR from `Proposed` to `Accepted` is itself a repository-wide
architecture commitment.

Inputs used for the conservative score:

- C=0 (non-development/document review), F=3 (6-10 status artifacts),
- D=3, T=0, A=0, K=3, P=4, X=4,
- `arch_decision +12`.

No Ollama/local-stack action is required: docs/ADR/task-ledger-only task, so the
local precheck is `n/a`. Phase-1/phase-2 peer review is `n/a` under the documented
ADR/docs exemption; the integrated D4 review is the acceptance evidence.

## Review result

**PASS — ADR-044 is internally coherent and ready for acceptance.**

### Pass 1 — authority and security boundary

PASS.

- O3 keeps audience authorization backend-owned and separate from ADR-032.
- K1 keeps ciphertext-only publication, non-exportable device-key custody, and
  fail-closed envelope release.
- O4 keeps publication readiness authoritative in PostgreSQL and prevents queue,
  transport, or Availability Node state from becoming authorization/readiness truth.
- Hyperdrive/ciphertext possession never becomes authorization.

### Pass 2 — lifecycle / failure semantics

PASS.

- K1 revocation blocks future envelope release without claiming impossible remote
  deletion of an already disclosed volatile CK.
- O4 covers durable intent, duplicate delivery, unknown remote outcome,
  remote-success/ACK-loss, and duplicate-after-Ready under same-lineage idempotency.
- `PreparationStatus::Ready` remains independent from `P2P_READY`, preserving S-120
  and ASR sequencing.

### Pass 3 — scope / downstream gates

PASS.

Open questions 4-7 are intentionally phase-specific rather than ADR-acceptance
blockers:

1. Availability Node deployment/auth/observability remains a P2 deployment gate.
2. Certification profile remains a P7 gate.
3. Full ADR-018 P2P audit-event inventory remains a P2/P3 closure gate.
4. Persistent cache/device/background lifecycle remains a P4 lifecycle gate.

None contradicts the accepted audience-delivery boundary. They must be resolved by
the named phase before that phase can close.

## Owner disposition

**ACCEPT.** The owner-directed unblock instruction authorizes the bounded D4
closure after the three-pass review returned PASS. ADR-044 may therefore change
from `Proposed` to `Accepted`.

Acceptance of ADR-044 removes the architecture prerequisite for P2 planning and
presentation. It does **not** authorize P2 source edits; P2 still requires its own
plan, RRI, task decomposition, Compact Approval Task Card, and explicit execution
approval.

## Verification / status propagation

Required canonical state after D4 sync:

```text
D1  O3 parallel   ✅ complete
D2  K1            ✅ complete
D3  O4            ✅ complete
D4  ADR acceptance ✅ complete

ADR-044 = Accepted
P2 = READY FOR PLAN / RRI / HITL, NOT SOURCE-AUTHORIZED
P3 = blocked on P2 PASS
```

Evidence: `docs/audit/mvp0-p2p-adr044-d4-acceptance.md`.
