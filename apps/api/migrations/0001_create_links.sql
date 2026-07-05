CREATE TABLE IF NOT EXISTS links (
    id         BIGSERIAL   PRIMARY KEY,
    long_url   TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
