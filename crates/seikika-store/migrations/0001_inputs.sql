-- Seikika v1 inputs and published snapshots (docs/v1-spec.md §8). Expand/contract only.
CREATE SCHEMA IF NOT EXISTS seikika;

CREATE TABLE IF NOT EXISTS seikika.input_versions (
    version     BIGSERIAL PRIMARY KEY,
    connector   TEXT,
    kind        TEXT NOT NULL,              -- 'ingest' | 'native'
    records     INT NOT NULL DEFAULT 0,
    deletes     INT NOT NULL DEFAULT 0,
    command_id  TEXT,
    at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS seikika.raw_records (
    connector    TEXT NOT NULL,
    kind         TEXT NOT NULL,
    external_id  TEXT NOT NULL,
    data         JSONB NOT NULL,
    observed_at  TIMESTAMPTZ,
    version      BIGINT NOT NULL REFERENCES seikika.input_versions (version),
    PRIMARY KEY (connector, kind, external_id)
);

CREATE TABLE IF NOT EXISTS seikika.native_values (
    entity_id  TEXT NOT NULL,
    field      TEXT NOT NULL,
    revision   BIGINT NOT NULL,
    value      JSONB NOT NULL,
    author     TEXT,
    reason     TEXT,
    at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    version    BIGINT NOT NULL REFERENCES seikika.input_versions (version),
    PRIMARY KEY (entity_id, field, revision)
);

CREATE TABLE IF NOT EXISTS seikika.snapshots (
    snapshot_id    TEXT PRIMARY KEY,
    input_version  BIGINT NOT NULL,
    manifest_hash  TEXT NOT NULL,
    published_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    summary        JSONB NOT NULL
);

-- Derived OpenFGA tuples, written only by the publishing instance as deltas (D26, D38).
CREATE TABLE IF NOT EXISTS seikika.derived_tuples (
    object    TEXT NOT NULL,
    relation  TEXT NOT NULL,
    subject   TEXT NOT NULL,
    PRIMARY KEY (object, relation, subject)
);
