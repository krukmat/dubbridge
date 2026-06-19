-- S-120-T2: preparation lineage and readiness schema
-- Source artifacts retain non-null ingest_token; derived artifacts carry parent_artifact_id.
-- PostgreSQL UNIQUE on ingest_token allows multiple NULLs (each NULL is distinct).
ALTER TABLE artifact_records ALTER COLUMN ingest_token DROP NOT NULL;
ALTER TABLE artifact_records ADD COLUMN parent_artifact_id UUID REFERENCES artifact_records(id);

-- Exactly one of (ingest_token, parent_artifact_id) must be set.
ALTER TABLE artifact_records ADD CONSTRAINT artifact_source_or_derived
    CHECK (
        (ingest_token IS NOT NULL AND parent_artifact_id IS NULL) OR
        (ingest_token IS NULL AND parent_artifact_id IS NOT NULL)
    );

CREATE TABLE asset_preparation_status (
    asset_id     UUID        PRIMARY KEY REFERENCES assets(id),
    status       TEXT        NOT NULL,
    error_detail TEXT,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
