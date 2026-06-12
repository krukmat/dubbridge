-- S-100-T1: organizations table and org_members join (ADR-027)
-- Organizations are the tenancy boundary. Assets remain uploader-owned (ADR-023).
CREATE TABLE organizations (
    id         UUID        PRIMARY KEY,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Role is a constrained TEXT column; unknown values are rejected at application layer
-- via parse_org_role (fail-closed, ADR-008 posture). Constraint enforces storage hygiene.
CREATE TABLE org_members (
    org_id     UUID        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    subject_id UUID        NOT NULL,
    role       TEXT        NOT NULL
                           CHECK (role IN ('owner', 'admin', 'editor', 'reviewer', 'viewer')),
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (org_id, subject_id)
);

CREATE INDEX org_members_subject_idx ON org_members (subject_id);
