-- MVP0-P2P P2.T6b: close the two early lifecycle audit boundaries without
-- weakening the existing P2 identity/state model.
--
-- Intent audit is emitted by an AFTER INSERT trigger. Because the publication
-- insert and initial outbox obligation are created in one caller transaction,
-- an audit failure aborts that entire transaction.
--
-- Lineage-sealed audit is emitted only when durable package evidence is first
-- attached to an already wrapped K1 lineage. The trigger runs in the same
-- transaction as that evidence update and therefore fails closed with it.

ALTER TABLE p2p_publications
    ADD COLUMN package_ref TEXT;

ALTER TABLE p2p_publications
    ADD CONSTRAINT p2p_publications_package_ref_nonempty_check
    CHECK (package_ref IS NULL OR btrim(package_ref) <> '');

CREATE OR REPLACE FUNCTION audit_p2p_publication_intent_created()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO audit_events (
        id,
        asset_id,
        event_kind,
        ingest_token,
        detail,
        happened_at,
        recording_session_id,
        platform_ingest_session_id,
        correlation_id,
        publication_id,
        lineage_id
    )
    VALUES (
        gen_random_uuid(),
        NEW.asset_id,
        'p2p_publication_intent_created',
        NULL,
        NULL,
        now(),
        NULL,
        NULL,
        NEW.id,
        NEW.id,
        NEW.lineage_id
    );
    RETURN NEW;
END;
$$;

CREATE TRIGGER p2p_publication_intent_audit_trigger
AFTER INSERT ON p2p_publications
FOR EACH ROW
EXECUTE FUNCTION audit_p2p_publication_intent_created();

CREATE OR REPLACE FUNCTION audit_p2p_lineage_sealed()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO audit_events (
        id,
        asset_id,
        event_kind,
        ingest_token,
        detail,
        happened_at,
        recording_session_id,
        platform_ingest_session_id,
        correlation_id,
        publication_id,
        lineage_id
    )
    VALUES (
        gen_random_uuid(),
        NEW.asset_id,
        'p2p_lineage_sealed',
        NULL,
        NULL,
        now(),
        NULL,
        NULL,
        NEW.id,
        NEW.id,
        NEW.lineage_id
    );
    RETURN NEW;
END;
$$;

CREATE TRIGGER p2p_lineage_sealed_audit_trigger
AFTER UPDATE OF manifest_digest_sha256, package_ref ON p2p_publications
FOR EACH ROW
WHEN (
    OLD.package_ref IS NULL
    AND NEW.package_ref IS NOT NULL
    AND NEW.manifest_digest_sha256 IS NOT NULL
    AND NEW.sealed_kek_id IS NOT NULL
    AND NEW.sealed_kek_version IS NOT NULL
    AND NEW.sealed_nonce IS NOT NULL
    AND NEW.sealed_wrapped_ck IS NOT NULL
    AND NEW.sealed_at IS NOT NULL
)
EXECUTE FUNCTION audit_p2p_lineage_sealed();

CREATE UNIQUE INDEX audit_events_p2p_singleton_early_event_uq
    ON audit_events (publication_id, event_kind)
    WHERE event_kind IN (
        'p2p_publication_intent_created',
        'p2p_lineage_sealed'
    );
