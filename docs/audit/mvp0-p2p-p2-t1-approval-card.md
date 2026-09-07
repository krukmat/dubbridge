---
type: Audit
title: "Compact Approval Task Card v2 — P2.T1"
status: superseded
slice: MVP0-P2P
parent: P2.T1
---

# Compact Approval Task Card v2 — P2.T1

## Superseded

The original `P2.T1 — durable publication identity + transactional outbox persistence` gate bundled domain modeling, SQL schema, transactional writes, read/recovery surfaces, guarded transitions, and integration certification under a planning **RRI 78 High / Effort XL** parent.

On 2026-09-05 the owner requested that complexity be reduced by generating smaller tasks. This card is therefore **not executable and no longer an approval gate**.

Canonical decomposition:

- `P2.T1a` — pure domain identity/state contract — planning RRI **22 Low**;
- `P2.T1b` — PostgreSQL schema + constraints — planning RRI **32 Medium**;
- `P2.T1c` — atomic create/ensure publication + outbox repository write — planning RRI **47 Medium-high**;
- `P2.T1d` — outstanding-work/read queries — planning RRI **36 Medium**;
- `P2.T1e` — guarded persistence transitions + confirmation evidence — planning RRI **44 Medium-high**;
- `P2.T1f` — persistence integration certification — planning RRI **33 Medium**.

Full rationale and boundaries: `docs/audit/mvp0-p2p-p2-t1-decomposition.md`.

Current approval card: `docs/audit/mvp0-p2p-p2-t1a-approval-card.md`.

Every listed score is planning-only until `scripts/rri.py` is executed against that leaf's frozen exact path set. No T1 source implementation was performed under this superseded card.
