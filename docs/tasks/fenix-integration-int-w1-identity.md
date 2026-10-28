---
type: TaskList
title: "DB-INT-W1.T2 — Service identity execution tasks"
status: planned
slice: INT-W1
plan: docs/plan/fenix-integration-int-w1-identity.md
governed_by: [ADR-008, ADR-030, ADR-031]
---

# DB-INT-W1.T2 — DubBridge task ledger

This ledger tracks **real implementation**. The 2026-10-28 design is a proposal, not an accepted security ADR or deployed credential service. GitHub Actions were reported disabled by the owner; no new CI was launched here.

| Leaf | Deliverable | Depends on | State |
|---|---|---|---|
| T2a | RRI + independent security review + ADR for S2S-only JWT and scope | INT-W1.T1 | PROPOSED |
| T2b | Real integration principal PostgreSQL schema, queries and revocation state | T2a PASS | BLOCKED |
| T2c | Operator-authorized issuance/rotation/revocation, narrow scope and expiry | T2b PASS | BLOCKED |
| T2d | Dedicated read-only review queue route reusing existing repository | T2c PASS | BLOCKED |
| T2e | Real API+DB negative security and interoperability evidence | T2d PASS | BLOCKED |

## T2a acceptance

- [x] Source inspection found broad mobile user scopes and independent org membership checks.
- [x] Draft dedicated route, service identity, revocation and bounded token lifecycle.
- [ ] Finalize security ADR and exact service authority, org/project binding, token type/audience/key model and claim semantics.
- [ ] Calculate RRI using the repository script and record actual independent readiness/security review; do not invent PASS.
- [ ] Freeze exact implementation paths, test plan and permission-denial matrix.

## T2b–T2e acceptance

- [ ] Operator-issued credential cannot be forged or issued via user login and cannot access mobile or mutation routes.
- [ ] Persisted service principal cannot read another organization or project.
- [ ] Expired, revoked or rotated credential fails closed on every request.
- [ ] All results originate from the actual PostgreSQL-backed DubBridge API.
- [ ] Verify jointly with Fenix ToolRegistry/Blackboard; no mock or fake E2E acceptance.
- [ ] Local QA and peer code review pass before declaring closure.

## Gate

Only docs/planning has been committed here; runtime code, migrations and credentials remain unchanged. Further implementation follows existing `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` review, RRI and user approval gates.
