-- S-200-T2a: user_account table for mobile credential auth (ADR-031 §Decision 5).
-- workspace_id references organizations(id) — "workspace" is the domain term; organizations is the table.
CREATE TABLE user_account (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT        UNIQUE NOT NULL,
    password_hash TEXT        NOT NULL,
    workspace_id  UUID        NOT NULL REFERENCES organizations (id),
    status        TEXT        NOT NULL DEFAULT 'active',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_user_account_email ON user_account (email);
