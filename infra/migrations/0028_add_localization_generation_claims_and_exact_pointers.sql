-- S-150-T1c-i: exact current-generation pointers on localization status rows plus
-- normalized immutable generation claims for translation/dubbing.
--
-- Status rows keep the "current generation" identity/pointers nullable so existing
-- rows require no backfill. The added checks are fail-closed:
--   * all current-pointer columns are either entirely NULL, or
--   * a current generation always carries a source artifact, and
--   * dubbed_audio cannot be current without a current manifest.
--
-- The generation-claim table stores the exact source artifact for each claimed
-- `(operation, project_id, asset_id, target_language_id, generation_request_id)`
-- tuple. Repository code in S-150-T1c-ii will enforce cross-asset/kind checks and
-- reserved-request semantics on top of these storage constraints.
--
-- Reversal statements for a future forward migration if needed:
--   ALTER TABLE asset_translation_status DROP CONSTRAINT asset_translation_status_current_pointer_check;
--   ALTER TABLE asset_translation_status DROP CONSTRAINT asset_translation_status_current_translated_fk;
--   ALTER TABLE asset_translation_status DROP CONSTRAINT asset_translation_status_current_source_artifact_fk;
--   ALTER TABLE asset_translation_status DROP COLUMN current_translated_subtitle_artifact_id;
--   ALTER TABLE asset_translation_status DROP COLUMN current_source_artifact_id;
--   ALTER TABLE asset_translation_status DROP COLUMN current_generation_request_id;
--   ALTER TABLE asset_dubbing_status DROP CONSTRAINT asset_dubbing_status_current_pointer_check;
--   ALTER TABLE asset_dubbing_status DROP CONSTRAINT asset_dubbing_status_current_dubbed_audio_artifact_fk;
--   ALTER TABLE asset_dubbing_status DROP CONSTRAINT asset_dubbing_status_current_manifest_artifact_fk;
--   ALTER TABLE asset_dubbing_status DROP CONSTRAINT asset_dubbing_status_current_source_artifact_fk;
--   ALTER TABLE asset_dubbing_status DROP COLUMN current_dubbed_audio_artifact_id;
--   ALTER TABLE asset_dubbing_status DROP COLUMN current_manifest_artifact_id;
--   ALTER TABLE asset_dubbing_status DROP COLUMN current_source_artifact_id;
--   ALTER TABLE asset_dubbing_status DROP COLUMN current_generation_request_id;
--   DROP TABLE localization_generation_claims;

ALTER TABLE asset_translation_status
    ADD COLUMN current_generation_request_id UUID,
    ADD COLUMN current_source_artifact_id UUID,
    ADD COLUMN current_translated_subtitle_artifact_id UUID;

ALTER TABLE asset_translation_status
    ADD CONSTRAINT asset_translation_status_current_source_artifact_fk
        FOREIGN KEY (current_source_artifact_id)
        REFERENCES artifact_records (id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT asset_translation_status_current_translated_fk
        FOREIGN KEY (current_translated_subtitle_artifact_id)
        REFERENCES artifact_records (id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT asset_translation_status_current_pointer_check
        CHECK (
            (
                current_generation_request_id IS NULL
                AND current_source_artifact_id IS NULL
                AND current_translated_subtitle_artifact_id IS NULL
            ) OR (
                current_generation_request_id IS NOT NULL
                AND current_source_artifact_id IS NOT NULL
            )
        );

ALTER TABLE asset_dubbing_status
    ADD COLUMN current_generation_request_id UUID,
    ADD COLUMN current_source_artifact_id UUID,
    ADD COLUMN current_manifest_artifact_id UUID,
    ADD COLUMN current_dubbed_audio_artifact_id UUID;

ALTER TABLE asset_dubbing_status
    ADD CONSTRAINT asset_dubbing_status_current_source_artifact_fk
        FOREIGN KEY (current_source_artifact_id)
        REFERENCES artifact_records (id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT asset_dubbing_status_current_manifest_artifact_fk
        FOREIGN KEY (current_manifest_artifact_id)
        REFERENCES artifact_records (id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT asset_dubbing_status_current_dubbed_audio_artifact_fk
        FOREIGN KEY (current_dubbed_audio_artifact_id)
        REFERENCES artifact_records (id)
        ON DELETE RESTRICT,
    ADD CONSTRAINT asset_dubbing_status_current_pointer_check
        CHECK (
            (
                current_generation_request_id IS NULL
                AND current_source_artifact_id IS NULL
                AND current_manifest_artifact_id IS NULL
                AND current_dubbed_audio_artifact_id IS NULL
            ) OR (
                current_generation_request_id IS NOT NULL
                AND current_source_artifact_id IS NOT NULL
                AND (
                    current_dubbed_audio_artifact_id IS NULL
                    OR current_manifest_artifact_id IS NOT NULL
                )
            )
        );

CREATE TABLE localization_generation_claims (
    operation             TEXT        NOT NULL,
    project_id            UUID        NOT NULL,
    asset_id              UUID        NOT NULL,
    target_language_id    UUID        NOT NULL,
    generation_request_id UUID        NOT NULL,
    source_artifact_id    UUID        NOT NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT localization_generation_claims_pk
        PRIMARY KEY (operation, project_id, asset_id, target_language_id, generation_request_id),
    CONSTRAINT localization_generation_claims_operation_check
        CHECK (operation IN ('translation', 'dubbing')),
    CONSTRAINT localization_generation_claims_project_asset_fk
        FOREIGN KEY (project_id, asset_id)
        REFERENCES project_assets (project_id, asset_id)
        ON DELETE CASCADE,
    CONSTRAINT localization_generation_claims_target_language_fk
        FOREIGN KEY (target_language_id, project_id)
        REFERENCES target_languages (id, project_id)
        ON DELETE CASCADE,
    CONSTRAINT localization_generation_claims_source_artifact_fk
        FOREIGN KEY (source_artifact_id)
        REFERENCES artifact_records (id)
        ON DELETE RESTRICT
);
