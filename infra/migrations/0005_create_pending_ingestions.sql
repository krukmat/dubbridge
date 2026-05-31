-- T1-T1: tuning/hardening — durable pending ingestion sessions
CREATE TABLE pending_ingestions (
    ingest_token     UUID        PRIMARY KEY,
    title            TEXT        NOT NULL,
    storage_key      TEXT        NOT NULL,
    content_type     TEXT        NOT NULL,
    file_size_bytes  BIGINT      NOT NULL,
    checksum         TEXT        NOT NULL,
    rights_owner     TEXT,
    license_type     TEXT,
    source_type      TEXT,
    proof_reference  TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
