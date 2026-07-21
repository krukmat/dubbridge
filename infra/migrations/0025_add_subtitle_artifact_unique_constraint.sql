-- Partial unique index on subtitle rows: ensures no two subtitle artifacts share
-- the same asset_id and parent_artifact_id combination.
--
-- (1) This is a partial unique index scoped to kind='subtitle' so it does not
--     restrict other artifact kinds sharing a parent_artifact_id.
-- (2) Both asset_id and parent_artifact_id are structurally non-NULL for every
--     subtitle row because subtitle rows are always derived (never source), and the
--     artifact_source_or_derived CHECK from migration 0019 guarantees
--     parent_artifact_id IS NOT NULL for derived rows.
-- (3) No existing data can violate this constraint because no repository code writes
--     ArtifactKind::Subtitle rows yet (that ships in a later, separate task).
-- (4) The predicate restates the NOT NULL guarantee explicitly rather than relying
--     solely on the artifact_source_or_derived CHECK from migration 0019, so this
--     index stays fail-closed even if that CHECK is ever relaxed.
--
-- Reversal statement:
--   DROP INDEX artifact_records_subtitle_unique_asset_parent;
CREATE UNIQUE INDEX artifact_records_subtitle_unique_asset_parent
    ON artifact_records(asset_id, parent_artifact_id)
    WHERE kind = 'subtitle' AND asset_id IS NOT NULL AND parent_artifact_id IS NOT NULL;
