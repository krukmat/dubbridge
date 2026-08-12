-- S-150-T2b-i: translation-dispatch outbox table for enqueueing and tracking
-- dispatch state of localization generation work.
--
-- The composite primary key mirrors the localization_generation_claims identity:
-- (operation, project_id, asset_id, target_language_id, generation_request_id),
-- keeping dispatch entries on the same durable generation identity as claims.
--
-- Foreign keys preserve existing ownership boundaries identical to
-- localization_generation_claims: dual FK on (project_id, asset_id) and
-- (target_language_id, project_id).

CREATE TABLE translation_dispatch_outbox (
    operation             TEXT         NOT NULL,
    project_id            UUID         NOT NULL,
    asset_id              UUID         NOT NULL,
    target_language_id    UUID         NOT NULL,
    generation_request_id UUID         NOT NULL,
    delivery_state        TEXT         NOT NULL,
    error_detail          TEXT,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT translation_dispatch_outbox_pk
        PRIMARY KEY (operation, project_id, asset_id, target_language_id, generation_request_id),
    CONSTRAINT translation_dispatch_outbox_operation_check
        CHECK (operation IN ('translation', 'dubbing')),
    CONSTRAINT translation_dispatch_outbox_project_asset_fk
        FOREIGN KEY (project_id, asset_id)
        REFERENCES project_assets (project_id, asset_id)
        ON DELETE CASCADE,
    CONSTRAINT translation_dispatch_outbox_target_language_fk
        FOREIGN KEY (target_language_id, project_id)
        REFERENCES target_languages (id, project_id)
        ON DELETE CASCADE,
    CONSTRAINT translation_dispatch_outbox_delivery_state_check
        CHECK (delivery_state IN (
            'pending',
            'acknowledged',
            'enqueue_failed'
        ))
);
