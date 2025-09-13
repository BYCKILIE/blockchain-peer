CREATE TABLE IF NOT EXISTS blocks
(
    hash          TEXT PRIMARY KEY,
    previous_hash TEXT,
    organization  TEXT        NOT NULL,
    payload       TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL
);