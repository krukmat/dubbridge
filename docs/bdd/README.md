# BDD specs — S-100 Collaborative localization workspace

Gherkin acceptance specs for the S-100 slice. Each scenario has a stable ID that maps
to a web (Playwright) or mobile (Maestro) flow and to the task-level happy-path /
edge-case evidence.

## Convention

- Scenario IDs follow the pattern `SC-<AREA>-<N>` (e.g. `SC-ORG-1`).
- Each scenario maps to at least one web flow (`web/e2e/`) and optionally a Maestro flow (`mobile/maestro/`).
- Each flow maps to one or more `HP-#` / `EC-#` cases in the task ledger
  (`docs/tasks/s-100-collaborative-workspace.md`).
- Scenario language is behavioral — no implementation calls, no UI selectors.
  Implementation details live in the E2E flows and the unit tests.

## Mapping table

| Scenario ID | Description | Task | Web flow | Mobile flow | HP / EC |
|---|---|---|---|---|---|
| SC-ORG-1 | Create an organization and become its owner | S-100-T3, S-100-T5 | `web/e2e/projects.spec.ts` | — | HP-1 |
| SC-MEMBER-1 | Invite a member with a role | S-100-T2, S-100-T3, S-100-T5 | `web/e2e/projects.spec.ts` | — | HP-1 |
| SC-MEMBER-2 | Non-member is denied org access | S-100-T2, S-100-T3 | `web/e2e/projects.spec.ts` | — | EC-1 |
| SC-PROJECT-1 | Create a project and link assets | S-100-T3, S-100-T5, S-100-T6 | `web/e2e/projects.spec.ts` | `mobile/maestro/projects.yaml` | HP-1, HP-2 |
| SC-LANG-1 | Declare target languages for a project | S-100-T3, S-100-T5 | `web/e2e/projects.spec.ts` | — | HP-2 |

## Spec file

`docs/bdd/p4-workspace.feature`

## Adding new scenarios

1. Add the scenario to `p4-workspace.feature` with the next available `SC-<AREA>-<N>` ID.
2. Add the row to the mapping table above.
3. Create or update the corresponding web (Playwright) or Maestro flow.
4. Add `HP-#` / `EC-#` coverage in the implementing task in the ledger.
