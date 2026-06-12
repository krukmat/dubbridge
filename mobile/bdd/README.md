# BDD specs — S-060 Mobile asset lifecycle

Gherkin acceptance specs for the S-060 slice. Each scenario has a stable ID that maps
to a Maestro flow and to the task-level happy-path / edge-case evidence.

## Convention

- Scenario IDs follow the pattern `SC-<AREA>-<N>` (e.g. `SC-LIST-1`).
- Each scenario maps to exactly one Maestro flow file in `mobile/maestro/`.
- Each Maestro flow maps to one or more `HP-#` / `EC-#` cases in the task ledger
  (`docs/tasks/s-060-mobile-asset-lifecycle.md`).
- Scenario language is behavioral — no implementation calls, no UI selectors.
  Implementation details live in the Maestro YAML and the unit tests.

## Mapping table

| Scenario ID | Description | Task | Maestro flow | HP / EC |
|---|---|---|---|---|
| SC-LIST-1 | Browse my assets (populated list) | T2 | `asset-list.yaml` | HP-1 |
| SC-LIST-2 | Empty asset list | T2 | `asset-list.yaml` | EC-1 |
| SC-DETAIL-1 | Open an asset from the list | T2 | `asset-detail.yaml` | HP-2 |
| SC-INGEST-1 | Upload a new asset (happy path) | T3a, T3b | `asset-ingestion.yaml` | HP-1 |
| SC-INGEST-2 | Upload rejected without rights | T3b | `asset-ingestion.yaml` | EC-1 |

## Spec file

`mobile/bdd/asset-lifecycle.feature`

## Adding new scenarios

1. Add the scenario to `asset-lifecycle.feature` with the next available `SC-<AREA>-<N>` ID.
2. Add the row to the mapping table above.
3. Create or update the corresponding Maestro flow in `mobile/maestro/`.
4. Add `HP-#` / `EC-#` coverage in the implementing task in the ledger.
