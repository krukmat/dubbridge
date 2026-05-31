-- T2: S1 migration — rights ledger per ADR-008
CREATE TABLE rights_records (
    id              UUID        PRIMARY KEY,
    asset_id        UUID        NOT NULL REFERENCES assets (id),
    owner           TEXT        NOT NULL,
    license_type    TEXT        NOT NULL,
    source_type     TEXT        NOT NULL,
    proof_reference TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
