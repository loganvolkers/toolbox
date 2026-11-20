-- Toolbox v1 durable workflows: a column subset of DBOS's system tables (dbos-transact-rust bad825f,
-- migrations/1_initial_dbos_schema.sql), same table and column names, in schema toolbox_durable.
CREATE SCHEMA IF NOT EXISTS toolbox_durable;

CREATE TABLE toolbox_durable.workflow_status (
    workflow_uuid              TEXT PRIMARY KEY,
    status                     TEXT NOT NULL CHECK (status IN ('PENDING','SUCCESS','ERROR','CANCELLED')),
    name                       TEXT NOT NULL,              -- workflow type, e.g. 'action-run'
    authenticated_user         TEXT,                       -- requesting principal
    inputs                     TEXT NOT NULL,              -- JSON
    output                     TEXT,                       -- JSON
    error                      TEXT,                       -- JSON
    executor_id                TEXT NOT NULL,              -- owning instance
    application_version        TEXT NOT NULL,              -- manifest hash + server version
    deduplication_id           TEXT UNIQUE,                -- idempotency key
    recovery_attempts          BIGINT NOT NULL DEFAULT 0,
    workflow_deadline_epoch_ms BIGINT,
    created_at                 BIGINT NOT NULL DEFAULT (EXTRACT(epoch FROM now())::numeric * 1000)::bigint,
    updated_at                 BIGINT NOT NULL DEFAULT (EXTRACT(epoch FROM now())::numeric * 1000)::bigint
);
CREATE INDEX workflow_status_status_index ON toolbox_durable.workflow_status (status);
CREATE INDEX workflow_status_executor_id_index ON toolbox_durable.workflow_status (executor_id);

-- One row per completed step: replaying a workflow returns these instead of re-running the step.
CREATE TABLE toolbox_durable.operation_outputs (
    workflow_uuid TEXT NOT NULL REFERENCES toolbox_durable.workflow_status (workflow_uuid) ON DELETE CASCADE,
    function_id   INT4 NOT NULL,                           -- step sequence number within the workflow
    function_name TEXT NOT NULL,
    output        TEXT,
    error         TEXT,
    PRIMARY KEY (workflow_uuid, function_id)
);

-- Signals to a waiting workflow (approvals, executor results, command completions).
CREATE TABLE toolbox_durable.notifications (
    destination_uuid    TEXT NOT NULL REFERENCES toolbox_durable.workflow_status (workflow_uuid) ON DELETE CASCADE,
    topic               TEXT NOT NULL,
    message             TEXT NOT NULL,
    created_at_epoch_ms BIGINT NOT NULL DEFAULT (EXTRACT(epoch FROM now())::numeric * 1000)::bigint,
    message_uuid        TEXT NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY
);
CREATE INDEX idx_workflow_topic ON toolbox_durable.notifications (destination_uuid, topic);

-- Toolbox addition (not in DBOS): instance liveness, so any instance can recover a dead one's workflows (Q13).
CREATE TABLE toolbox_durable.executors (
    executor_id  TEXT PRIMARY KEY,
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
