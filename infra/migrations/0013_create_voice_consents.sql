-- S-110-T1a: voice-consent ledger per ADR-028
-- Append-only: status = latest row (grant/revoke); evidence stored by reference only (ADR-025).
CREATE TABLE voice_consents (
    id           UUID        PRIMARY KEY,
    asset_id     UUID        NOT NULL REFERENCES assets (id),
    scope        TEXT        NOT NULL,
    status       TEXT        NOT NULL,
    evidence_ref TEXT,
    granted_by   UUID        NOT NULL,
    happened_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT voice_consents_status_check
        CHECK (status IN ('grant', 'revoke')),
    CONSTRAINT voice_consents_scope_check
        CHECK (scope IN ('voice_clone', 'tts_synthesis'))
);

-- Append-only enforcement: block UPDATE and DELETE at the DB level (ADR-028, mirrors 0007).
CREATE RULE voice_consents_no_update AS ON UPDATE TO voice_consents DO INSTEAD NOTHING;
CREATE RULE voice_consents_no_delete AS ON DELETE TO voice_consents DO INSTEAD NOTHING;
