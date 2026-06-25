-- S-130-T1: transcription status and artifact kinds
-- Creates asset_transcription_status table and extends artifact_kind_check.
CREATE TABLE asset_transcription_status (
    asset_id     UUID        PRIMARY KEY REFERENCES assets(id),
    status       TEXT        NOT NULL,
    error_detail TEXT,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Extend the artifact_kind_check constraint to include transcript artifact kinds.
-- Migration 0020 set the constraint to original_media + preparation-derived kinds.
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
            'word_alignment'
        ));
