# BDD specs — Mobile slices

Gherkin acceptance specs for mobile-only DubBridge slices. Each scenario has a
stable ID that maps to one or more concrete verification artifacts and to the
task-level happy-path / edge-case evidence.

## Convention

- Scenario IDs follow the pattern `SC-<AREA>-<N>` (e.g. `SC-LIST-1`).
- Each scenario maps to one or more concrete verification artifacts.
- Mobile-first executable slices usually map to at least one Maestro flow file in
  `mobile/maestro/`.
- Retrospective slices may instead map to existing unit/integration evidence or
  runner artifacts when no standalone Maestro flow exists.
- Scenario language is behavioral — no implementation calls, no UI selectors.
- Mobile-only `.feature` specs live in `mobile/bdd/` as one file per slice.
  Recommended naming: `s-<slice>-<short-name>.feature`.
- Product-layer slices shared with backend governance contracts live in
  `docs/bdd/`; after S-105 their executable authenticated UI is mobile.

## S-050 — First-party mobile client

Spec file: `mobile/bdd/s-050-mobile-client.feature`

| Scenario ID | Description | Task | Maestro flow | HP / EC |
|---|---|---|---|---|
| SC-AUTH-1 | Sign in through the mobile gateway handoff | `MBF-T1` | — | HP-1, HP-2 |
| SC-AUTH-2 | Login fails closed when the handoff is missing or invalid | `MBF-T1` | — | EC-1, EC-2 |
| SC-AUTH-3 | Token-like session values are rejected on device | `MBF-T1` | — | EC-1, EC-2 |
| SC-NAV-1 | Auth state controls the root navigation tree | `MBF-T1` | — | HP-1, HP-2, HP-3, EC-1, EC-2 |
| SC-ASSET-1 | Browse my asset list and open asset detail | `MBF-T1` | — | HP-1, HP-2 |
| SC-ASSET-2 | Asset surfaces handle empty, failed, or unavailable responses clearly | `MBF-T1` | — | EC-1, EC-2, EC-3 |

## S-055 — Maestro screenshot / visual-audit suite

Spec file: `mobile/bdd/s-055-maestro-suite.feature`

| Scenario ID | Description | Task | Maestro flow | HP / EC |
|---|---|---|---|---|
| SC-SUITE-1 | Capture the unauthenticated auth surface | `MBF-T2` | `auth-surface.yaml` | HP-1 |
| SC-SUITE-2 | Bootstrap an authenticated session without UI login | `MBF-T2` | `authenticated-audit.yaml` | HP-1, HP-2 |
| SC-SUITE-3 | Screenshot artifacts remain free of sensitive session values | `MBF-T2` | `seed-and-run.sh` | EC-1, EC-7 |

## S-060 — Mobile asset lifecycle

Spec file: `mobile/bdd/asset-lifecycle.feature`

| Scenario ID | Description | Task | Maestro flow | HP / EC |
|---|---|---|---|---|
| SC-LIST-1 | Browse my assets (populated list) | T2 | `asset-list.yaml` | HP-1 |
| SC-LIST-2 | Empty asset list | T2 | `asset-list.yaml` | EC-1 |
| SC-DETAIL-1 | Open an asset from the list | T2 | `asset-detail.yaml` | HP-2 |
| SC-INGEST-1 | Upload a new asset (happy path) | T3a, T3b | `asset-ingestion.yaml` | HP-1 |
| SC-INGEST-2 | Upload rejected without rights | T3b | `asset-ingestion-no-rights.yaml` | EC-1 |

## Adding new scenarios

1. Add the scenario to the slice's `.feature` file with the next available `SC-<AREA>-<N>` ID.
2. Add or update the row in the mapping table for that slice.
3. Create or update the corresponding Maestro flow when the slice is mobile-first and executable, or map the scenario to the concrete retrospective verification artifact that already exists.
4. Add `HP-#` / `EC-#` coverage in the implementing task ledger.
