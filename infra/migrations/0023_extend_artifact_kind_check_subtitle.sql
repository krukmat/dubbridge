-- S-140-T1b-ii: extend artifact_kind_check to include subtitle artifact kind
-- Extends the artifact_kind_check constraint on artifact_records with 'subtitle'.
ALTER TABLE artifact_records
    DROP CONSTRAINT artifact_kind_check;

ALTER TABLE artifact_records
    ADD CONSTRAINT artifact_kind_check
        CHECK (kind IN (
            'original_media',
            'probe_metadata',
            'hls_manifest',
            'hls_segment',
            'transcript_text',
            'word_alignment',
            'subtitle'
        ));
