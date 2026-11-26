-- Action runs on a served portal (docs/design/federated-actions.md §7): each run's record and
-- the confirm tokens already spent, so a restart keeps both. Expand/contract only.
CREATE TABLE IF NOT EXISTS seikika.runs (
    id          TEXT PRIMARY KEY,
    doc         JSONB NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS seikika.spent_tokens (
    token   TEXT PRIMARY KEY,
    issued  BIGINT NOT NULL
);
