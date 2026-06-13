# BDD specs — DubBridge

Gherkin acceptance specs for product-layer slices. Each scenario has a stable ID that maps
to an executable product surface or backend authorization/gate evidence and to the
task-level happy-path / edge-case evidence.

## Convention

- Scenario IDs follow the pattern `SC-<AREA>-<N>` (e.g. `SC-ORG-1`).
- Each user-facing scenario maps to an executable mobile surface through Maestro
  and/or component tests. Backend-only authorization and fail-closed gate scenarios
  map to backend integration/unit evidence.
- Each flow maps to one or more `HP-#` / `EC-#` cases in the task ledger
  (`docs/tasks/s-100-collaborative-workspace.md`).
- Scenario language is behavioral — no implementation calls, no UI selectors.
  Implementation details live in the E2E flows and the unit tests.

## S-100 — Collaborative localization workspace

Spec file: `docs/bdd/p4-workspace.feature`

| Scenario ID | Description | Task | Executable evidence | Mobile flow | HP / EC |
|---|---|---|---|---|---|
| SC-ORG-1 | Create an organization and become its owner | S-100-T3, S-105-T1 | mobile component + mock-gateway tests | `mobile/maestro/projects.yaml` | HP-1 |
| SC-MEMBER-1 | Invite a member with a role | S-100-T2, S-100-T3, S-105-T2 | mobile component + backend tests | — | HP-1 |
| SC-MEMBER-2 | Non-member is denied org access | S-100-T2, S-100-T3 | backend authorization tests | — | EC-1 |
| SC-PROJECT-1 | Create a project and link assets | S-100-T3, S-100-T6, S-105-T1 | mobile component + mock-gateway tests | `mobile/maestro/projects.yaml` | HP-1, HP-2 |
| SC-LANG-1 | Declare target languages for a project | S-100-T3, S-105-T2 | backend mutation + mobile read tests | `mobile/maestro/projects.yaml` | HP-2 |

## S-110 — Compliance & consent center

Spec file: `docs/bdd/p6-compliance.feature`

| Scenario ID | Description | Task | Executable evidence | Mobile flow | HP / EC |
|---|---|---|---|---|---|
| SC-AUDIT-1 | View an asset's audit timeline | S-110-T3, S-110-T5 | mobile component tests | `mobile/maestro/compliance.yaml` | HP-1 |
| SC-AUDIT-2 | Audit view is ownership-scoped | S-110-T3 | backend ownership integration tests | — | EC-1 |
| SC-RIGHTS-1 | View the rights ledger for an asset | S-110-T3, S-110-T5 | mobile component tests | `mobile/maestro/compliance.yaml` | HP-2 |
| SC-CONSENT-1 | Grant voice consent | S-110-T2, S-110-T3, S-110-T5 | mobile component + mock-gateway tests | `mobile/maestro/compliance.yaml` | HP-1 |
| SC-CONSENT-2 | Revoke voice consent | S-110-T2, S-110-T3, S-110-T5 | mobile component + mock-gateway tests | `mobile/maestro/compliance.yaml` | HP-2, EC-1 |
| SC-CONSENT-3 | Synthesis blocked without consent | S-110-T2 | backend consent-gate unit tests | — | EC-1 |

## S-160 — Human review & publication workspace

Spec file: `docs/bdd/s-160-review.feature`

| Scenario ID | Description | Task | Executable evidence | Mobile flow | HP / EC |
|---|---|---|---|---|---|
| SC-REVIEW-1 | Reviewer sees their queue | S-160-T3, S-160-T6 | backend queue tests + mobile component tests | `mobile/maestro/review.yaml` | HP-1 |
| SC-REVIEW-2 | Approve a derived output | S-160-T2, S-160-T3, S-160-T6 | backend decision tests + mobile component tests | `mobile/maestro/review.yaml` | HP-1 |
| SC-REVIEW-3 | Reject a derived output | S-160-T2, S-160-T3, S-160-T6 | backend decision tests + mobile component tests | `mobile/maestro/review.yaml` | HP-2 |
| SC-PUBLISH-1 | Publish a reviewed asset | S-160-T2, S-160-T3, S-160-T6 | backend publication-gate tests + mobile component tests | `mobile/maestro/review.yaml` | HP-1 |
| SC-PUBLISH-2 | Publication blocked without approval | S-160-T2, S-160-T3 | backend publication-gate tests | `mobile/maestro/review.yaml` | EC-1 |
| SC-NOTIFY-1 | Reviewer notified of assignment | S-160-T4, S-160-T6 | notification emit tests + mobile component tests | `mobile/maestro/review.yaml` | EC-1 |

## Adding new scenarios

1. Add the scenario to the relevant `.feature` file with the next available `SC-<AREA>-<N>` ID.
2. Add the row to the mapping table for that slice above.
3. Create or update the corresponding mobile Maestro/component flow, or backend
   evidence when the scenario is an authorization or governance invariant.
4. Add `HP-#` / `EC-#` coverage in the implementing task in the ledger.
