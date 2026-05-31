-- T2: S1 migration — assets table with ingestion status state machine
CREATE TABLE assets (
    id          UUID        PRIMARY KEY,
    title       TEXT        NOT NULL,
    uploader_id UUID        NOT NULL,
    status      TEXT        NOT NULL
                            CHECK (status IN (
                                'pending',
                                'finalized',
                                'rejected_missing_rights',
                                'rejected_missing_uploader_context'
                            )),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
