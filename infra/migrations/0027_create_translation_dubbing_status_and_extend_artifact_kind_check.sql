-- S-150-T1b: per-target translation/dubbing status storage + exhaustive artifact_kind_check.
-- Uses the same localization-unit identity as review_tasks: (project_id, asset_id, target_language_id).
-- Rebuilds artifact_kind_check from the full current domain literal set instead of copying 0023's
-- incomplete list, which predates recorded/downloaded media and the S-150 artifact kinds.

CREATE TABLE asset_translation_status (
    project_id         UUID        NOT NULL,
    asset_id           UUID        NOT NULL,
    target_language_id UUID        NOT NULL,
    status             TEXT        NOT NULL,
    error_detail       TEXT,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT asset_translation_status_pk
        PRIMARY KEY (project_id, asset_id, target_language_id),
    CONSTRAINT asset_translation_status_project_asset_fk
        FOREIGN KEY (project_id, asset_id)
        REFERENCES project_assets (project_id, asset_id)
        ON DELETE CASCADE,
    CONSTRAINT asset_translation_status_target_language_fk
        FOREIGN KEY (target_language_id, project_id)
        REFERENCES target_languages (id, project_id)
        ON DELETE CASCADE,
    CONSTRAINT asset_translation_status_check
        CHECK (status IN (
            'pending',
            'in_progress',
            'ready',
            'failed'
        ))
);

CREATE TABLE asset_dubbing_status (
    project_id         UUID        NOT NULL,
    asset_id           UUID        NOT NULL,
    target_language_id UUID        NOT NULL,
    status             TEXT        NOT NULL,
    error_detail       TEXT,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT asset_dubbing_status_pk
        PRIMARY KEY (project_id, asset_id, target_language_id),
    CONSTRAINT asset_dubbing_status_project_asset_fk
        FOREIGN KEY (project_id, asset_id)
        REFERENCES project_assets (project_id, asset_id)
        ON DELETE CASCADE,
    CONSTRAINT asset_dubbing_status_target_language_fk
        FOREIGN KEY (target_language_id, project_id)
        REFERENCES target_languages (id, project_id)
        ON DELETE CASCADE,
    CONSTRAINT asset_dubbing_status_check
        CHECK (status IN (
            'pending',
            'in_progress',
            'ready',
            'failed'
        ))
);

ALTER TABLE artifact_records
    DROP CONSTRAINT artifact_kind_check;

ALTER TABLE artifact_records
    ADD CONSTRAINT artifact_kind_check
        CHECK (kind IN (
            'original_media',
            'recorded_stream_media',
            'downloaded_platform_media',
            'probe_metadata',
            'hls_manifest',
            'hls_segment',
            'transcript_text',
            'word_alignment',
            'subtitle',
            'translated_subtitle',
            'dubbed_audio_segment',
            'dubbing_manifest',
            'dubbed_audio'
        ));
