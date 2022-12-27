CREATE TABLE nft_tokens
(
    id         TEXT PRIMARY KEY,
    type       TEXT        NOT NULL,
    owner      TEXT        NOT NULL,
    url        TEXT        NOT NULL,
    traits     JSONB       NOT NULL,
    created_at timestamptz NOT NULL
);
