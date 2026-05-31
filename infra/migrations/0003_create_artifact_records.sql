-- T2: S1 migration — original media artifact records per ADR-006
-- ingest_token UNIQUE prevents duplicate artifacts for the same finalization request
CREATE TABLE artifact_records (
    id           UUID        PRIMARY KEY,
    asset_id     UUID        NOT NULL REFERENCES assets (id),
    kind         TEXT        NOT NULL,
    ingest_token UUID        NOT NULL UNIQUE,
    storage_key  TEXT        NOT NULL,
    content_type TEXT        NOT NULL,
    size_bytes   BIGINT      NOT NULL,
    checksum     TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
