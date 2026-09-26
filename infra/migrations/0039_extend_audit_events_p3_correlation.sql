-- MVP0-P2P P3.T3 certification repair:
-- align the durable audit CHECK with the P3 correlation shapes already frozen
-- in the domain contract. This preserves the strict P2 package correlation and
-- keeps non-P2/P3 events free of generic P2P correlation identifiers.

ALTER TABLE audit_events
    DROP CONSTRAINT audit_events_p2p_correlation_shape_check;

ALTER TABLE audit_events
    ADD CONSTRAINT audit_events_p2p_correlation_shape_check
        CHECK (
            CASE
                WHEN event_kind IN (
                    'p2p_publication_intent_created',
                    'p2p_lineage_sealed',
                    'p2p_publication_confirmed',
                    'p2p_publication_reconciliation_entered',
                    'p2p_publication_ready',
                    'p2p_publication_failed'
                ) THEN
                    asset_id IS NOT NULL
                    AND publication_id IS NOT NULL
                    AND lineage_id IS NOT NULL
                    AND correlation_id = publication_id
                    AND ingest_token IS NULL
                    AND recording_session_id IS NULL
                    AND platform_ingest_session_id IS NULL
                WHEN event_kind IN (
                    'p2p_invitation_created',
                    'p2p_invitation_claimed',
                    'p2p_audience_authorization_issued',
                    'p2p_device_envelope_released'
                ) THEN
                    asset_id IS NOT NULL
                    AND publication_id IS NOT NULL
                    AND lineage_id IS NOT NULL
                    AND correlation_id IS NOT NULL
                    AND ingest_token IS NULL
                    AND recording_session_id IS NULL
                    AND platform_ingest_session_id IS NULL
                WHEN event_kind IN (
                    'p2p_device_registered',
                    'p2p_audience_access_denied'
                ) THEN
                    correlation_id IS NOT NULL
                    AND (
                        (
                            asset_id IS NOT NULL
                            AND publication_id IS NOT NULL
                            AND lineage_id IS NOT NULL
                        )
                        OR (
                            publication_id IS NULL
                            AND lineage_id IS NULL
                        )
                    )
                    AND ingest_token IS NULL
                    AND recording_session_id IS NULL
                    AND platform_ingest_session_id IS NULL
                ELSE
                    publication_id IS NULL
                    AND lineage_id IS NULL
                    AND correlation_id IS NULL
            END
        );
