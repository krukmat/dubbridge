-- S-160-T1a: append-only review decisions per ADR-030
-- Current state is derived from the latest decision row, never updated in place.
CREATE TABLE review_decisions (
    id                  UUID        PRIMARY KEY,
    review_task_id      UUID        NOT NULL REFERENCES review_tasks (id) ON DELETE CASCADE,
    verdict             TEXT        NOT NULL,
    comment             TEXT,
    reviewer_subject_id UUID        NOT NULL,
    happened_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT review_decisions_verdict_check
        CHECK (verdict IN ('approved', 'rejected'))
);

CREATE INDEX review_decisions_task_happened_idx
    ON review_decisions (review_task_id, happened_at DESC, id DESC);

CREATE RULE review_decisions_no_update AS ON UPDATE TO review_decisions DO INSTEAD NOTHING;
CREATE RULE review_decisions_no_delete AS ON DELETE TO review_decisions DO INSTEAD NOTHING;
