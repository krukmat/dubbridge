-- MVP0-P2P P2.T6a: backward-compatible ADR-018 correlation for P2 lifecycle events.
-- Existing ingestion/recording/platform/workspace audit rows remain valid. P2 rows
-- use publication_id as correlation_id and bind the exact publication lineage.

ALTER TABLE audit_events
    ADD COLUMN correlation_id UUID,
    ADD COLUMN publication_id UUID,
    ADD COLUMN lineage_id UUID;

ALTER TABLE audit_events
    ADD CONSTRAINT audit_events_p2p_publication_lineage_fk
        FOREIGN KEY (publication_id, lineage_id)
        REFERENCES p2p_publications (id, lineage_id),
    ADD CONSTRAINT audit_events_p2p_correlation_shape_check
        CHECK (
            (
                publication_id IS NULL
                AND lineage_id IS NULL
                AND correlation_id IS NULL
            ) OR (
                publication_id IS NOT NULL
                AND lineage_id IS NOT NULL
                AND correlation_id = publication_id
                AND ingest_token IS NULL
                AND recording_session_id IS NULL
                AND platform_ingest_session_id IS NULL
            )
        );

CREATE INDEX audit_events_p2p_correlation_idx
    ON audit_events (publication_id, lineage_id, happened_at)
    WHERE publication_id IS NOT NULL;
