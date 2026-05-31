-- S3-T2: generalize audit_events for recording lifecycle events per ADR-018 (F2).
-- ingest_token becomes nullable so recording events (which have no ingest_token yet)
-- can be persisted. recording_session_id added as optional correlation FK.
-- Order dependency: 0008_create_recording_sessions must be applied first.
ALTER TABLE audit_events
    ALTER COLUMN ingest_token DROP NOT NULL;

ALTER TABLE audit_events
    ADD COLUMN recording_session_id UUID REFERENCES recording_sessions (id);
