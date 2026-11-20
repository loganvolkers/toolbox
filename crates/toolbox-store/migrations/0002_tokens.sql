-- Personal access tokens (docs/v1-spec.md §9): only sha256 hashes are stored.
CREATE TABLE IF NOT EXISTS toolbox.tokens (
    token_hash  TEXT PRIMARY KEY,
    subject     TEXT NOT NULL,
    label       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ,
    revoked_at  TIMESTAMPTZ
);
