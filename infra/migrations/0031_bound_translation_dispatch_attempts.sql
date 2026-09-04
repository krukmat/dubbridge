-- X26-T4: bound translation dispatch retries with durable attempt accounting.
-- Existing rows represent an already-created first dispatch attempt, so the
-- backfill/default starts at 1. Terminal exhaustion is persisted as `failed`.

ALTER TABLE translation_dispatch_outbox
    ADD COLUMN attempt_count INTEGER NOT NULL DEFAULT 1;

ALTER TABLE translation_dispatch_outbox
    DROP CONSTRAINT translation_dispatch_outbox_delivery_state_check;

ALTER TABLE translation_dispatch_outbox
    ADD CONSTRAINT translation_dispatch_outbox_delivery_state_check
        CHECK (delivery_state IN (
            'pending',
            'acknowledged',
            'enqueue_failed',
            'failed'
        )),
    ADD CONSTRAINT translation_dispatch_outbox_attempt_count_positive
        CHECK (attempt_count >= 1);
