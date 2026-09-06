-- MVP0-P2P P2.T1b: authoritative publication identity/state + transactional outbox.
-- ADR-044/O4: PostgreSQL is the durable authority. Queue/external reachability
-- never establishes readiness.

CREATE TABLE p2p_publications (
    id                       UUID        PRIMARY KEY,
    asset_id                 UUID        NOT NULL,
    lineage_id               UUID        NOT NULL,
    state                    TEXT        NOT NULL,
    external_publication_id  TEXT,
    confirmed_lineage_id     UUID,
    external_confirmed_at    TIMESTAMPTZ,
    failure_detail           TEXT,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT p2p_publications_asset_fk
        FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    CONSTRAINT p2p_publications_asset_unique
        UNIQUE (asset_id),
    CONSTRAINT p2p_publications_lineage_unique
        UNIQUE (lineage_id),
    CONSTRAINT p2p_publications_id_lineage_unique
        UNIQUE (id, lineage_id),
    CONSTRAINT p2p_publications_state_check
        CHECK (state IN (
            'building',
            'publish_pending',
            'publishing',
            'reconciling',
            'ready',
            'failed'
        )),
    CONSTRAINT p2p_publications_confirmation_all_or_none_check
        CHECK (
            (external_publication_id IS NULL
                AND confirmed_lineage_id IS NULL
                AND external_confirmed_at IS NULL)
            OR
            (external_publication_id IS NOT NULL
                AND confirmed_lineage_id IS NOT NULL
                AND external_confirmed_at IS NOT NULL)
        ),
    CONSTRAINT p2p_publications_confirmation_lineage_check
        CHECK (confirmed_lineage_id IS NULL OR confirmed_lineage_id = lineage_id),
    CONSTRAINT p2p_publications_ready_requires_confirmation_check
        CHECK (
            state <> 'ready'
            OR (
                external_publication_id IS NOT NULL
                AND confirmed_lineage_id = lineage_id
                AND external_confirmed_at IS NOT NULL
            )
        )
);

CREATE TABLE p2p_publication_outbox (
    id              UUID        PRIMARY KEY,
    publication_id  UUID        NOT NULL,
    lineage_id      UUID        NOT NULL,
    delivery_state  TEXT        NOT NULL DEFAULT 'pending',
    attempt_count   INTEGER     NOT NULL DEFAULT 0,
    available_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    claimed_at      TIMESTAMPTZ,
    delivered_at    TIMESTAMPTZ,
    last_error      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT p2p_publication_outbox_publication_unique
        UNIQUE (publication_id),
    CONSTRAINT p2p_publication_outbox_publication_lineage_fk
        FOREIGN KEY (publication_id, lineage_id)
        REFERENCES p2p_publications(id, lineage_id)
        ON DELETE CASCADE,
    CONSTRAINT p2p_publication_outbox_delivery_state_check
        CHECK (delivery_state IN ('pending', 'claimed', 'delivered')),
    CONSTRAINT p2p_publication_outbox_attempt_count_check
        CHECK (attempt_count >= 0),
    CONSTRAINT p2p_publication_outbox_delivered_at_check
        CHECK (
            (delivery_state = 'delivered' AND delivered_at IS NOT NULL)
            OR
            (delivery_state <> 'delivered' AND delivered_at IS NULL)
        )
);

CREATE INDEX p2p_publication_outbox_pending_idx
    ON p2p_publication_outbox (available_at, created_at)
    WHERE delivery_state = 'pending';
