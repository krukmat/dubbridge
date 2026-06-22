-- S-125-T2a: playback-grant schema (ADR-032, ADR-008, ADR-018)
-- grant_id supplied by caller (PlaybackGrantId::new() in the repo layer).
CREATE TABLE playback_grants (
    grant_id      UUID        PRIMARY KEY,
    asset_id      UUID        NOT NULL REFERENCES assets (id),
    principal_id  UUID        NOT NULL,
    org_id        UUID        NOT NULL REFERENCES organizations (id),
    project_id    UUID        NOT NULL,
    scope         TEXT        NOT NULL
                              CHECK (scope IN ('review')),
    status        TEXT        NOT NULL
                              CHECK (status IN ('active', 'expired', 'revoked')),
    issued_at     TIMESTAMPTZ NOT NULL,
    expires_at    TIMESTAMPTZ NOT NULL,
    CONSTRAINT chk_grant_expiry_after_issued CHECK (expires_at > issued_at)
);

-- Active-grant lookup: resolve_grant_target and get_active_grant both filter on
-- (asset_id, status, expires_at) to avoid a seqscan on large grant tables.
CREATE INDEX idx_playback_grants_active
    ON playback_grants (asset_id, status, expires_at);
