---
type: Proposal
title: "DB-INT-W1.T2b — Candidate PostgreSQL service principal schema"
status: superseded
date: 2026-10-28
---

> **ARCHIVED / NOT FOR MIGRATION (2026-10-29):** Candidate schema for the wrong-direction Fenix → DubBridge principal. **Do not turn this into an applied migration.** Current plan is [DubBridge consumes Fenix](../plan/dubbridge-consumes-fenix-integration.md).
# Candidate Fenix integration principal schema — not a migration

**Status:** proposed; no Rust application changes, no active schema migration, no actual service credential or endpoint implemented. Source owner: DubBridge. Security decision: [ADR-048](../adr/ADR-048-fenix-service-readonly-identity-boundary.md).

This SQL is **implementation candidate code**, isolated under `docs/proposals/` specifically so Rust service initialization does not automatically apply an unreviewed database migration. Once the ADR, RRI, review and local PostgreSQL QA pass, promote to the next free numbered migration in `infra/migrations/` and wire only the approved read-only service runtime.

```sql
-- DB-INT-W1.T2b: DRAFT / NOT APPLIED.
-- Dedicated Fenix read-only integration identities. No plaintext credentials.
-- Rollout requires security ADR acceptance, Rust verification and database QA.
-- Deny cross-org/project references with a composite FK.
CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_id_org_id
    ON projects (id, org_id);

CREATE TABLE integration_principals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consumer TEXT NOT NULL DEFAULT 'fenix'
        CHECK (consumer = 'fenix'),
    allowed_org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    allowed_project_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'revoked')),
    credential_version BIGINT NOT NULL DEFAULT 1
        CHECK (credential_version > 0),
    current_jti UUID,
    last_expires_at TIMESTAMPTZ,
    created_by UUID NOT NULL,
    last_rotated_by UUID,
    revoked_by UUID,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT integration_principals_project_in_org
        FOREIGN KEY (allowed_project_id, allowed_org_id)
        REFERENCES projects (id, org_id) ON DELETE CASCADE,
    CONSTRAINT integration_principals_status_revocation
        CHECK ((status = 'active' AND revoked_at IS NULL AND revoked_by IS NULL)
            OR (status = 'revoked' AND revoked_at IS NOT NULL AND revoked_by IS NOT NULL)),
    CONSTRAINT integration_principals_jti_issued_state
        CHECK ((current_jti IS NULL AND last_expires_at IS NULL)
            OR (current_jti IS NOT NULL AND last_expires_at IS NOT NULL))
);

CREATE UNIQUE INDEX integration_principals_active_project_unique
    ON integration_principals (consumer, allowed_org_id, allowed_project_id)
    WHERE status = 'active';

CREATE UNIQUE INDEX integration_principals_jti_unique
    ON integration_principals (current_jti)
    WHERE current_jti IS NOT NULL;

CREATE INDEX integration_principals_active_lookup
    ON integration_principals (id, allowed_org_id, allowed_project_id, credential_version)
    WHERE status = 'active';
```

## Remaining review questions

- Confirm the migration is safely reversible and does not conflict with new DubBridge migrations.
- Validate operator identity, issuance, rotation and revocation semantics; `created_by`, `current_jti`, and `credential_version` are only schema candidates, not enforcement.
- Decide whether physical deletion of a project should also delete identities or instead retain a revocation/audit tombstone.
- Verify audit correlations, token audience/issuer/key separation, cross-tenant denial, and no mutation scope.
- Run on a **real disposable PostgreSQL database** before moving any SQL into `infra/migrations/`.
