-- S-160-T1a: publication rows are anchored to the governing review task per ADR-030
-- review_task_id uniquely identifies the governed (project, asset, target language) unit.
CREATE TABLE publications (
    id            UUID        PRIMARY KEY,
    review_task_id UUID       NOT NULL UNIQUE REFERENCES review_tasks (id) ON DELETE CASCADE,
    state         TEXT        NOT NULL,
    published_by  UUID        NOT NULL,
    published_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT publications_state_check
        CHECK (state IN ('published'))
);
