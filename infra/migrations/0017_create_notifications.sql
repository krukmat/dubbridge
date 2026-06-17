-- S-160-T4a: durable notifications + push-token persistence for reviewer workflows.
-- Payloads are reference-only: no asset title, reviewer comment, or freeform body columns.
CREATE TABLE notifications (
    id                   UUID        PRIMARY KEY,
    recipient_subject_id UUID        NOT NULL,
    notification_kind    TEXT        NOT NULL,
    ref_entity_type      TEXT        NOT NULL,
    ref_entity_id        UUID        NOT NULL,
    actor_subject_id     UUID,
    read_at              TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT notifications_kind_check
        CHECK (
            notification_kind IN (
                'review_task_assigned',
                'review_task_decided',
                'review_task_published'
            )
        ),
    CONSTRAINT notifications_ref_entity_type_check
        CHECK (ref_entity_type IN ('review_task'))
);

CREATE INDEX notifications_recipient_created_idx
    ON notifications (recipient_subject_id, created_at DESC, id DESC);

CREATE INDEX notifications_recipient_unread_idx
    ON notifications (recipient_subject_id, read_at, created_at DESC);

CREATE TABLE push_tokens (
    id           UUID        PRIMARY KEY,
    subject_id   UUID        NOT NULL,
    provider     TEXT        NOT NULL,
    device_token TEXT        NOT NULL,
    platform     TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT push_tokens_provider_check
        CHECK (provider IN ('expo')),
    CONSTRAINT push_tokens_platform_check
        CHECK (platform IN ('ios', 'android')),
    CONSTRAINT push_tokens_provider_device_unique
        UNIQUE (provider, device_token)
);

CREATE INDEX push_tokens_subject_idx
    ON push_tokens (subject_id, created_at DESC);
