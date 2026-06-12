# BDD specs — DubBridge

Gherkin acceptance specs for product-layer slices. Each scenario has a stable ID that maps
to a web (Playwright) or mobile (Maestro) flow and to the task-level happy-path /
edge-case evidence.

## Convention

- Scenario IDs follow the pattern `SC-<AREA>-<N>` (e.g. `SC-ORG-1`).
- Each scenario maps to at least one web flow (`web/e2e/`) and optionally a Maestro flow (`mobile/maestro/`).
- Each flow maps to one or more `HP-#` / `EC-#` cases in the task ledger
  (`docs/tasks/s-100-collaborative-workspace.md`).
- Scenario language is behavioral — no implementation calls, no UI selectors.
  Implementation details live in the E2E flows and the unit tests.

## S-100 — Collaborative localization workspace

Spec file: `docs/bdd/p4-workspace.feature`

| Scenario ID | Description | Task | Web flow | Mobile flow | HP / EC |
|---|---|---|---|---|---|
| SC-ORG-1 | Create an organization and become its owner | S-100-T3, S-100-T5 | `web/e2e/projects.spec.ts` | — | HP-1 |
| SC-MEMBER-1 | Invite a member with a role | S-100-T2, S-100-T3, S-100-T5 | `web/e2e/projects.spec.ts` | — | HP-1 |
| SC-MEMBER-2 | Non-member is denied org access | S-100-T2, S-100-T3 | `web/e2e/projects.spec.ts` | — | EC-1 |
| SC-PROJECT-1 | Create a project and link assets | S-100-T3, S-100-T5, S-100-T6 | `web/e2e/projects.spec.ts` | `mobile/maestro/projects.yaml` | HP-1, HP-2 |
| SC-LANG-1 | Declare target languages for a project | S-100-T3, S-100-T5 | `web/e2e/projects.spec.ts` | — | HP-2 |

## S-110 — Compliance & consent center

Spec file: `docs/bdd/p6-compliance.feature`

| Scenario ID | Description | Task | Web flow | Mobile flow | HP / EC |
|---|---|---|---|---|---|
| SC-AUDIT-1 | View an asset's audit timeline | S-110-T3, S-110-T4 | `web/e2e/compliance.spec.ts` | `mobile/maestro/compliance.yaml` | HP-1 |
| SC-AUDIT-2 | Audit view is ownership-scoped | S-110-T3, S-110-T4 | `web/e2e/compliance.spec.ts` | — | EC-1 |
| SC-RIGHTS-1 | View the rights ledger for an asset | S-110-T3, S-110-T4 | `web/e2e/compliance.spec.ts` | `mobile/maestro/compliance.yaml` | HP-2 |
| SC-CONSENT-1 | Grant voice consent | S-110-T2, S-110-T3, S-110-T4, S-110-T5 | `web/e2e/compliance.spec.ts` | `mobile/maestro/compliance.yaml` | HP-1 |
| SC-CONSENT-2 | Revoke voice consent | S-110-T2, S-110-T3, S-110-T4, S-110-T5 | `web/e2e/compliance.spec.ts` | `mobile/maestro/compliance.yaml` | HP-2, EC-1 |
| SC-CONSENT-3 | Synthesis blocked without consent | S-110-T2, S-110-T3, S-110-T6 | `web/e2e/compliance.spec.ts` | `mobile/maestro/compliance.yaml` | EC-1 |

## Adding new scenarios

1. Add the scenario to the relevant `.feature` file with the next available `SC-<AREA>-<N>` ID.
2. Add the row to the mapping table for that slice above.
3. Create or update the corresponding web (Playwright) or Maestro flow.
4. Add `HP-#` / `EC-#` coverage in the implementing task in the ledger.
