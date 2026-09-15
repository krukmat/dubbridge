-- MVP0-P2P P3: one-viewer/one-device invitation and O3 authorization state.
-- Raw invitation tokens and plaintext content keys are never persisted here.

CREATE TABLE p2p_devices (
    id UUID PRIMARY KEY,
    subject_id UUID NOT NULL,
    key_id TEXT NOT NULL,
    public_key_spki BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    CONSTRAINT p2p_devices_key_id_nonblank CHECK (btrim(key_id) <> ''),
    CONSTRAINT p2p_devices_public_key_nonempty CHECK (octet_length(public_key_spki) > 0),
    CONSTRAINT p2p_devices_subject_key_unique UNIQUE (subject_id, key_id)
);

CREATE UNIQUE INDEX p2p_devices_one_active_per_subject_idx
    ON p2p_devices (subject_id)
    WHERE revoked_at IS NULL;

CREATE TABLE p2p_invitations (
    id UUID PRIMARY KEY,
    asset_id UUID NOT NULL REFERENCES assets(id),
    publication_id UUID NOT NULL,
    lineage_id UUID NOT NULL,
    owner_subject_id UUID NOT NULL,
    token_hash BYTEA NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    claimed_by_subject_id UUID,
    claimed_device_id UUID REFERENCES p2p_devices(id),
    claimed_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT p2p_invitations_publication_lineage_fk
        FOREIGN KEY (publication_id, lineage_id)
        REFERENCES p2p_publications (id, lineage_id),
    CONSTRAINT p2p_invitations_token_hash_sha256 CHECK (octet_length(token_hash) = 32),
    CONSTRAINT p2p_invitations_claim_all_or_none CHECK (
        (claimed_by_subject_id IS NULL AND claimed_device_id IS NULL AND claimed_at IS NULL)
        OR
        (claimed_by_subject_id IS NOT NULL AND claimed_device_id IS NOT NULL AND claimed_at IS NOT NULL)
    )
);

CREATE INDEX p2p_invitations_owner_idx
    ON p2p_invitations (owner_subject_id, created_at DESC);
CREATE INDEX p2p_invitations_viewer_idx
    ON p2p_invitations (claimed_by_subject_id, created_at DESC)
    WHERE claimed_by_subject_id IS NOT NULL;

CREATE TABLE p2p_audience_authorizations (
    id UUID PRIMARY KEY,
    invitation_id UUID NOT NULL UNIQUE REFERENCES p2p_invitations(id),
    asset_id UUID NOT NULL REFERENCES assets(id),
    publication_id UUID NOT NULL,
    lineage_id UUID NOT NULL,
    viewer_subject_id UUID NOT NULL,
    device_id UUID NOT NULL REFERENCES p2p_devices(id),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT p2p_audience_authorizations_publication_lineage_fk
        FOREIGN KEY (publication_id, lineage_id)
        REFERENCES p2p_publications (id, lineage_id)
);

CREATE INDEX p2p_audience_authorizations_viewer_idx
    ON p2p_audience_authorizations (viewer_subject_id, created_at DESC);
