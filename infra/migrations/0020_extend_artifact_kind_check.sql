-- S-120-T2: extend artifact_kind_check to allow derived artifact kinds
-- Migration 0007 added the constraint with only 'original_media'.
-- Preparation introduces probe_metadata, hls_manifest, hls_segment.
ALTER TABLE artifact_records
    DROP CONSTRAINT artifact_kind_check;

ALTER TABLE artifact_records
    ADD CONSTRAINT artifact_kind_check
        CHECK (kind IN (
            'original_media',
            'probe_metadata',
            'hls_manifest',
            'hls_segment'
        ));
