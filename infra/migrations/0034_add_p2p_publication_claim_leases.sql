-- MVP0-P2P P2.T4b: bounded PostgreSQL claim/lease ownership for publication dispatch.
-- PostgreSQL remains the delivery authority. A lease prevents concurrent dispatch
-- only while it is live; expiry makes the same durable outbox obligation reclaimable.

ALTER TABLE p2p_publication_outbox
    ADD COLUMN claim_token UUID,
    ADD COLUMN lease_expires_at TIMESTAMPTZ;

ALTER TABLE p2p_publication_outbox
    ADD CONSTRAINT p2p_publication_outbox_claim_lease_check
        CHECK (
            (
                delivery_state = 'claimed'
                AND claim_token IS NOT NULL
                AND claimed_at IS NOT NULL
                AND lease_expires_at IS NOT NULL
                AND isfinite(lease_expires_at)
                AND lease_expires_at > claimed_at
            ) OR (
                delivery_state <> 'claimed'
                AND claim_token IS NULL
                AND lease_expires_at IS NULL
            )
        );

CREATE UNIQUE INDEX p2p_publication_outbox_claim_token_unique
    ON p2p_publication_outbox (claim_token)
    WHERE claim_token IS NOT NULL;

CREATE INDEX p2p_publication_outbox_expired_claim_idx
    ON p2p_publication_outbox (lease_expires_at, created_at)
    WHERE delivery_state = 'claimed';
