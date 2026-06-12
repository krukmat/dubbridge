-- S-100-T1: projects table and project_assets join (ADR-027)
-- Projects group assets within an org. Assets are linked, never reassigned (ADR-023).
CREATE TABLE projects (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX projects_org_idx ON projects (org_id);

-- project_assets is a pure join table; asset ownership (uploader_id) is never modified.
CREATE TABLE project_assets (
    project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    asset_id   UUID NOT NULL REFERENCES assets (id) ON DELETE CASCADE,
    linked_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, asset_id)
);
