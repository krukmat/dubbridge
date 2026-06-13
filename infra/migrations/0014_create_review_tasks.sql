-- S-160-T1a: review task anchor per ADR-030
-- One review task governs one (project, asset, target language) unit.
-- Add composite uniqueness so review_tasks can enforce project/org and target/project scope.
ALTER TABLE projects
    ADD CONSTRAINT projects_id_org_unique
        UNIQUE (id, org_id);

ALTER TABLE target_languages
    ADD CONSTRAINT target_languages_id_project_unique
        UNIQUE (id, project_id);

CREATE TABLE review_tasks (
    id                  UUID        PRIMARY KEY,
    org_id              UUID        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    project_id          UUID        NOT NULL,
    asset_id            UUID        NOT NULL,
    target_language_id  UUID        NOT NULL,
    assignee_subject_id UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    assigned_at         TIMESTAMPTZ,

    CONSTRAINT review_tasks_project_scope_fk
        FOREIGN KEY (project_id, org_id)
        REFERENCES projects (id, org_id)
        ON DELETE CASCADE,
    CONSTRAINT review_tasks_project_asset_fk
        FOREIGN KEY (project_id, asset_id)
        REFERENCES project_assets (project_id, asset_id)
        ON DELETE CASCADE,
    CONSTRAINT review_tasks_target_language_scope_fk
        FOREIGN KEY (target_language_id, project_id)
        REFERENCES target_languages (id, project_id)
        ON DELETE CASCADE,
    CONSTRAINT review_tasks_assignee_membership_fk
        FOREIGN KEY (org_id, assignee_subject_id)
        REFERENCES org_members (org_id, subject_id),
    CONSTRAINT review_tasks_unique_review_unit
        UNIQUE (project_id, asset_id, target_language_id)
);

CREATE INDEX review_tasks_org_assignee_idx ON review_tasks (org_id, assignee_subject_id);
CREATE INDEX review_tasks_project_idx ON review_tasks (project_id);
