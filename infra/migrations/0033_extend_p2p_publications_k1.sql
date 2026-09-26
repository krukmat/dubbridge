-- MVP0-P2P P2.T2e: additive sealed-K1 metadata (wrapped-CK reference) on the
-- existing p2p_publications table. Does not alter T1's identity/state
-- semantics. Plaintext CK and raw KEK bytes are never stored here; only the
-- opaque wrap outputs produced by crates/p2p::key_wrap (T2d).

ALTER TABLE p2p_publications
    ADD COLUMN sealed_kek_id      TEXT,
    ADD COLUMN sealed_kek_version INTEGER,
    ADD COLUMN sealed_nonce       BYTEA,
    ADD COLUMN sealed_wrapped_ck  BYTEA,
    ADD COLUMN sealed_at          TIMESTAMPTZ;

ALTER TABLE p2p_publications
    ADD CONSTRAINT p2p_publications_sealed_k1_all_or_none_check
    CHECK (
        (sealed_kek_id IS NULL
            AND sealed_kek_version IS NULL
            AND sealed_nonce IS NULL
            AND sealed_wrapped_ck IS NULL
            AND sealed_at IS NULL)
        OR
        (sealed_kek_id IS NOT NULL
            AND sealed_kek_version IS NOT NULL
            AND sealed_nonce IS NOT NULL
            AND sealed_wrapped_ck IS NOT NULL
            AND sealed_at IS NOT NULL)
    );
