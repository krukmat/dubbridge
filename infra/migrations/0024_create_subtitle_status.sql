-- S-140-T1b: create asset_subtitle_status table with status CHECK constraint.
-- Deviation from the transcription-table precedent: 0022 had no status constraint,
-- while this table adds one to satisfy EC-2 using SubtitleStatus literals from
-- crates/domain/src/artifact.rs (pending, in_progress, ready, failed).
CREATE TABLE asset_subtitle_status (
    asset_id     UUID        PRIMARY KEY REFERENCES assets(id),
    status       TEXT        NOT NULL,
    error_detail TEXT,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE asset_subtitle_status
    ADD CONSTRAINT subtitle_status_check
        CHECK (status IN (
            'pending',
            'in_progress',
            'ready',
            'failed'
        ));
