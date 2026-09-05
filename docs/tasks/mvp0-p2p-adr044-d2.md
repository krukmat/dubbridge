---
type: TaskList
title: "Tasks: ADR044-D2 key-envelope closure"
status: complete
slice: MVP0-P2P
parent: ADR044-D2
---

# ADR044-D2 — key-envelope closure

- **Parent RRI:** 70 — Complex — Effort L.
- **Owner approval:** explicit 2026-09-05.
- **Owner selection:** `K1`.
- **Local/device precheck:** `n/a` — cloud environment; no evidence simulated.
- **Phase-1/2 review:** `n/a` — ADR/plan/task-ledger-only exemption.
- **ADR status:** remains `Proposed`.
- **P2/P3 source authorization:** none.

## Task map

| Task | Objective | Status | RRI |
|---|---|---|---:|
| `ADR044-D2-S1` | Freeze constraints/register | Complete | 24 |
| `ADR044-D2-S2` | Neutral cipher/wrap/envelope matrix | Complete | 24 |
| `ADR044-D2-S3` | Device-key/binding/expiry/revocation matrix | Complete | 24 |
| `ADR044-D2-S4` | Runtime/fail-closed release matrix | Complete | 24 |
| `ADR044-D2-OWNER` | Select coherent contract | Complete — `K1` | human |
| `ADR044-D2-S5` | Mechanical codification | Complete | 23 |
| `ADR044-D2-SYNC` | Canonical status propagation | Complete | 25 |

## Selected contract

See `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md` and ADR-044 question 2. K1 means AES-256-GCM package encryption, server-wrapped CK, HPKE P-256 device envelope, non-exportable Android Keystore private key, no external hardware or StrongBox requirement, one invitation/viewer/active-device MVP scope, fail-closed release, and no silent K2 fallback.

## Closure

D2 is resolved. `ADR044-D3` publication/outbox state and recovery semantics is next. ADR-044 remains `Proposed`; D4 acceptance and P2/P3 source work remain blocked.
