-- X26-T3c-d: persist platform-ingest correlation per ADR-018/ADR-025.
-- The domain has always carried this identifier; make the durable audit row
-- preserve it before enforcing the family-specific audit-boundary invariant.
ALTER TABLE audit_events
    ADD COLUMN platform_ingest_session_id UUID;
