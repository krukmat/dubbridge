-- S-100-T1: target_languages table (ADR-027)
-- Records localization intent per project: one source language, one target language per row.
-- BCP-47 codes (e.g. "en", "es-ES", "pt-BR") are stored as TEXT; validation is at the
-- application layer. A project may have multiple target_languages rows (one per target).
CREATE TABLE target_languages (
    id           UUID        PRIMARY KEY,
    project_id   UUID        NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    source_lang  TEXT        NOT NULL,
    target_lang  TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, target_lang)
);

CREATE INDEX target_languages_project_idx ON target_languages (project_id);
