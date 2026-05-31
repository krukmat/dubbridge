-- H1-T2: persistence-level governance invariants per ADR-008
-- 1. Append-only rights ledger: block UPDATE and DELETE at the DB level.
-- 2. CHECK constraints on governance enum columns to reject unknown stored values.

-- Append-only enforcement for rights_records.
-- RULEs run in the query planner before execution — no PL/pgSQL overhead.
CREATE RULE rights_no_update AS ON UPDATE TO rights_records DO INSTEAD NOTHING;
CREATE RULE rights_no_delete AS ON DELETE TO rights_records DO INSTEAD NOTHING;

-- Enum constraints on rights_records.
ALTER TABLE rights_records
    ADD CONSTRAINT rights_license_type_check
        CHECK (license_type IN (
            'all_rights_reserved',
            'creative_commons',
            'public_domain',
            'licensed_distribution',
            'internal_only'
        )),
    ADD CONSTRAINT rights_source_type_check
        CHECK (source_type IN (
            'direct_upload',
            'authorized_s3',
            'internal_feed',
            'licensed_source',
            'public_domain_with_proof'
        ));

-- Enum constraint on artifact_records.
ALTER TABLE artifact_records
    ADD CONSTRAINT artifact_kind_check
        CHECK (kind IN ('original_media'));
