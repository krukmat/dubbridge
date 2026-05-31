-- S3-T2: recording session aggregate table per ADR-020/022
-- Must exist before 0009 which adds recording_session_id FK to audit_events.
-- credential_ref is an opaque reference to a secrets store — no plaintext credentials (ADR-022).
CREATE TABLE recording_sessions (
    id                      UUID        PRIMARY KEY,
    owner_id                UUID        NOT NULL,
    source_protocol         TEXT        NOT NULL
                                        CHECK (source_protocol IN ('rtmp', 'srt')),
    source_url              TEXT        NOT NULL,
    credential_ref          TEXT,
    rights_owner            TEXT        NOT NULL,
    rights_license_type     TEXT        NOT NULL,
    rights_source_type      TEXT        NOT NULL,
    rights_proof_reference  TEXT        NOT NULL,
    status                  TEXT        NOT NULL DEFAULT 'requested'
                                        CHECK (status IN (
                                            'requested',
                                            'rights_validated',
                                            'capturing',
                                            'stopping',
                                            'recorded',
                                            'failed',
                                            'rejected_missing_rights'
                                        )),
    asset_id                UUID        REFERENCES assets (id),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
