-- S-140-T5b-a: let review tasks optionally carry the exact subtitle artifact identity.
-- Nullable/no-default preserves legacy rows; ON DELETE RESTRICT prevents silent dangling refs.
ALTER TABLE review_tasks
    ADD COLUMN subtitle_artifact_id UUID NULL
        REFERENCES artifact_records (id) ON DELETE RESTRICT;
