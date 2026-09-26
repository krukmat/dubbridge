-- MVP0-P2P P2.T5c: durable manifest evidence required by the authoritative
-- p2p-ready-descriptor-v1 read model.
--
-- The digest is populated only from Availability Node evidence that already
-- matched the exact request identity/digest. Existing ready rows remain
-- readable as legacy state but do not produce a ready descriptor until this
-- evidence is present, preserving fail-closed handoff semantics.

ALTER TABLE p2p_publications
    ADD COLUMN manifest_digest_sha256 TEXT;

ALTER TABLE p2p_publications
    ADD CONSTRAINT p2p_publications_manifest_digest_sha256_check
        CHECK (
            manifest_digest_sha256 IS NULL
            OR manifest_digest_sha256 ~ '^[0-9a-f]{64}$'
        );
