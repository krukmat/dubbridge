-- T1-T2: tuning/hardening — add TTL/expiration to pending ingestion sessions
ALTER TABLE pending_ingestions
    ADD COLUMN expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '24 hours');
