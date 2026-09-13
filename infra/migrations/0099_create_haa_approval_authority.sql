-- ADR-047: Human Approval Authority persistence.
-- Biometric images/templates/minutiae MUST NOT be stored in these tables.

CREATE TABLE IF NOT EXISTS haa_authenticators (
    id                  TEXT PRIMARY KEY,
    principal_kind      TEXT NOT NULL,
    principal_id        TEXT NOT NULL,
    kind                TEXT NOT NULL,
    public_key_json     TEXT NOT NULL,
    active              BOOLEAN NOT NULL DEFAULT TRUE,
    enrolled_at         TIMESTAMPTZ NOT NULL,
    revoked_at          TIMESTAMPTZ,
    CONSTRAINT haa_authenticator_revocation_consistent
        CHECK ((active AND revoked_at IS NULL) OR (NOT active))
);

CREATE INDEX IF NOT EXISTS idx_haa_authenticators_principal
    ON haa_authenticators (principal_kind, principal_id)
    WHERE active;

CREATE TABLE IF NOT EXISTS haa_approval_requests (
    id                  UUID PRIMARY KEY,
    protocol_version    TEXT NOT NULL,
    action_schema       TEXT NOT NULL,
    action_type         TEXT NOT NULL,
    resource            TEXT NOT NULL,
    environment         TEXT,
    canonical_action    TEXT NOT NULL,
    canonical_intent    TEXT NOT NULL,
    action_digest       TEXT NOT NULL,
    intent_digest       TEXT NOT NULL,
    requester_kind      TEXT NOT NULL,
    requester_id        TEXT NOT NULL,
    audience_kind       TEXT NOT NULL,
    audience_id         TEXT NOT NULL,
    approver_kind       TEXT NOT NULL,
    approver_id         TEXT NOT NULL,
    policy_id           TEXT NOT NULL,
    policy_version      TEXT NOT NULL,
    state               TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL,
    updated_at          TIMESTAMPTZ NOT NULL,
    expires_at          TIMESTAMPTZ NOT NULL,
    consumed_at         TIMESTAMPTZ,
    execution_id        UUID UNIQUE,
    CONSTRAINT haa_request_state_known CHECK (
        state IN ('requested','pending','approved','rejected','expired','revoked','consumed')
    ),
    CONSTRAINT haa_request_consumption_consistent CHECK (
        (state = 'consumed' AND consumed_at IS NOT NULL AND execution_id IS NOT NULL)
        OR (state <> 'consumed' AND consumed_at IS NULL AND execution_id IS NULL)
    ),
    CONSTRAINT haa_action_digest_shape CHECK (action_digest ~ '^[0-9a-f]{64}$'),
    CONSTRAINT haa_intent_digest_shape CHECK (intent_digest ~ '^[0-9a-f]{64}$')
);

CREATE INDEX IF NOT EXISTS idx_haa_requests_pending
    ON haa_approval_requests (approver_kind, approver_id, created_at)
    WHERE state IN ('requested','pending');

CREATE TABLE IF NOT EXISTS haa_challenges (
    request_id          UUID PRIMARY KEY REFERENCES haa_approval_requests(id) ON DELETE CASCADE,
    nonce               TEXT NOT NULL UNIQUE,
    challenge_digest    TEXT NOT NULL UNIQUE,
    package_json        TEXT NOT NULL,
    issued_at           TIMESTAMPTZ NOT NULL,
    expires_at          TIMESTAMPTZ NOT NULL,
    satisfied_at        TIMESTAMPTZ,
    CONSTRAINT haa_challenge_digest_shape CHECK (challenge_digest ~ '^[0-9a-f]{64}$')
);

CREATE TABLE IF NOT EXISTS haa_approval_evidence (
    id                  UUID PRIMARY KEY,
    request_id          UUID NOT NULL REFERENCES haa_approval_requests(id) ON DELETE CASCADE,
    authenticator_id    TEXT NOT NULL REFERENCES haa_authenticators(id),
    challenge_digest    TEXT NOT NULL,
    kind                TEXT NOT NULL,
    evidence_json       TEXT NOT NULL,
    verified_at         TIMESTAMPTZ NOT NULL,
    UNIQUE (request_id, authenticator_id, challenge_digest)
);

CREATE TABLE IF NOT EXISTS haa_approval_receipts (
    id                  UUID PRIMARY KEY,
    request_id          UUID NOT NULL UNIQUE REFERENCES haa_approval_requests(id) ON DELETE CASCADE,
    receipt_json        TEXT NOT NULL,
    issued_at           TIMESTAMPTZ NOT NULL,
    expires_at          TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS haa_execution_grants (
    execution_id        UUID PRIMARY KEY,
    request_id          UUID NOT NULL UNIQUE REFERENCES haa_approval_requests(id) ON DELETE CASCADE,
    action_digest       TEXT NOT NULL,
    audience_kind       TEXT NOT NULL,
    audience_id         TEXT NOT NULL,
    grant_json          TEXT NOT NULL,
    issued_at           TIMESTAMPTZ NOT NULL,
    expires_at          TIMESTAMPTZ NOT NULL,
    CONSTRAINT haa_grant_digest_shape CHECK (action_digest ~ '^[0-9a-f]{64}$')
);
