-- CANDIDATE ONLY: B4.W3.2 neutral, opt-in DubBridge -> Fenix dispatch intent.
-- NOT in infra/migrations: neither sqlx migrate nor deploy should apply it
-- without a separate W4 owner/operator approval. This SQL is nonproduction
-- design/source, not permission to activate the HTTP client.
--
-- A "submitted" record means Fenix ADMITTED/reserved an operation, never
-- that a workflow ran or produced a business result.

CREATE TABLE fenix_dispatch_intents (
    id                      UUID PRIMARY KEY,
    requested_by_subject_id UUID NOT NULL,
    workspace_id            UUID NOT NULL REFERENCES organizations(id),
    org_id                  UUID NOT NULL REFERENCES organizations(id),
    project_id              UUID,
    capability_key          TEXT NOT NULL,
    input_schema_version    TEXT NOT NULL,
    request_sha256          BYTEA NOT NULL,
    operation_id            UUID NOT NULL UNIQUE,
    trace_id                UUID NOT NULL UNIQUE,
    delivery_state          TEXT NOT NULL DEFAULT 'prepared',
    send_token              UUID,
    remote_replayed         BOOLEAN,
    last_error_code         TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fenix_dispatch_intents_scope_fk
        FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id)
        ON DELETE RESTRICT,
    CONSTRAINT fenix_dispatch_intents_workspace_scope_check
        CHECK (workspace_id = org_id),
    CONSTRAINT fenix_dispatch_intents_non_nil_id_check
        CHECK (id <> '00000000-0000-0000-0000-000000000000'::uuid
           AND operation_id <> '00000000-0000-0000-0000-000000000000'::uuid
           AND trace_id <> '00000000-0000-0000-0000-000000000000'::uuid),
    CONSTRAINT fenix_dispatch_intents_capability_check
        CHECK (char_length(capability_key) <= 96
           AND capability_key ~ '^[a-z][a-z0-9]*([._-][a-z0-9]+){1,6}$'),
    CONSTRAINT fenix_dispatch_intents_version_check
        CHECK (input_schema_version ~ '^[1-9][0-9]{0,2}$'),
    CONSTRAINT fenix_dispatch_intents_digest_check
        CHECK (octet_length(request_sha256) = 32
           AND request_sha256 <> decode(repeat('00',32),'hex')),
    CONSTRAINT fenix_dispatch_intents_state_check
        CHECK (delivery_state IN ('prepared','sending','submitted','indeterminate','rejected')),
    CONSTRAINT fenix_dispatch_intents_send_state_check
        CHECK ((delivery_state = 'prepared' AND send_token IS NULL)
           OR (delivery_state <> 'prepared' AND send_token IS NOT NULL)),
    CONSTRAINT fenix_dispatch_intents_outcome_check
        CHECK ((delivery_state = 'submitted' AND remote_replayed IS NOT NULL
                  AND last_error_code IS NULL)
           OR (delivery_state = 'indeterminate' AND remote_replayed IS NULL
                  AND last_error_code IN ('AMBIGUOUS_OUTCOME','INVALID_RESPONSE','PROVIDER_UNAVAILABLE'))
           OR (delivery_state = 'rejected' AND remote_replayed IS NULL
                  AND last_error_code IN ('INVALID_REQUEST','REQUEST_DENIED','OPERATION_CONFLICT'))
           OR (delivery_state IN ('prepared','sending') AND remote_replayed IS NULL
                  AND last_error_code IS NULL))
);

CREATE INDEX fenix_dispatch_intents_actor_idx
    ON fenix_dispatch_intents (org_id, requested_by_subject_id, created_at DESC);
CREATE INDEX fenix_dispatch_intents_unresolved_idx
    ON fenix_dispatch_intents (org_id, updated_at)
    WHERE delivery_state IN ('sending','indeterminate');

-- Mutation is limited to a single prepared -> sending -> terminal transition.
-- The source tuple and request fingerprint are permanently immutable.
CREATE FUNCTION reject_unsafe_fenix_dispatch_intent_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'fenix dispatch intent deletion forbidden';
    END IF;

    IF ROW(NEW.id, NEW.requested_by_subject_id, NEW.workspace_id,
           NEW.org_id, NEW.project_id, NEW.capability_key,
           NEW.input_schema_version, NEW.request_sha256,
           NEW.operation_id, NEW.trace_id, NEW.created_at)
       IS DISTINCT FROM
       ROW(OLD.id, OLD.requested_by_subject_id, OLD.workspace_id,
           OLD.org_id, OLD.project_id, OLD.capability_key,
           OLD.input_schema_version, OLD.request_sha256,
           OLD.operation_id, OLD.trace_id, OLD.created_at) THEN
        RAISE EXCEPTION 'fenix dispatch intent immutable identity changed';
    END IF;

    IF NOT ((OLD.delivery_state = 'prepared' AND NEW.delivery_state = 'sending')
            OR (OLD.delivery_state = 'sending'
                AND NEW.delivery_state IN ('submitted','indeterminate','rejected')))
       OR (OLD.send_token IS NOT NULL AND NEW.send_token IS DISTINCT FROM OLD.send_token)
       OR (NEW.updated_at < OLD.updated_at) THEN
        RAISE EXCEPTION 'fenix dispatch intent transition forbidden';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER fenix_dispatch_intents_update_guard
    BEFORE UPDATE ON fenix_dispatch_intents
    FOR EACH ROW EXECUTE FUNCTION reject_unsafe_fenix_dispatch_intent_mutation();

CREATE TRIGGER fenix_dispatch_intents_delete_guard
    BEFORE DELETE ON fenix_dispatch_intents
    FOR EACH ROW EXECUTE FUNCTION reject_unsafe_fenix_dispatch_intent_mutation();
