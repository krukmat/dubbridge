-- T2: S1 migration — audit trail per ADR-018
-- asset_id is nullable: rejection events may occur before an asset is persisted
CREATE TABLE audit_events (
    id           UUID        PRIMARY KEY,
    asset_id     UUID        REFERENCES assets (id),
    event_kind   TEXT        NOT NULL,
    ingest_token UUID        NOT NULL,
    detail       TEXT,
    happened_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
