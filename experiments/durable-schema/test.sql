\set ON_ERROR_STOP on
-- instance A starts a run and completes step 1, then dies
INSERT INTO toolbox_durable.executors VALUES ('inst-a', now() - interval '5 minutes'), ('inst-b', now());
INSERT INTO toolbox_durable.workflow_status (workflow_uuid,status,name,inputs,executor_id,application_version,deduplication_id)
  VALUES ('run_1','PENDING','action-run','{}','inst-a','m:abc','install-approved-app:repo:snyk');
INSERT INTO toolbox_durable.operation_outputs VALUES ('run_1',1,'preview','{"ok":true}',NULL);
-- the same idempotency key cannot start a second workflow
DO $$ BEGIN
  INSERT INTO toolbox_durable.workflow_status (workflow_uuid,status,name,inputs,executor_id,application_version,deduplication_id)
    VALUES ('run_2','PENDING','action-run','{}','inst-b','m:abc','install-approved-app:repo:snyk');
  RAISE EXCEPTION 'duplicate idempotency key accepted';
EXCEPTION WHEN unique_violation THEN RAISE NOTICE 'ok: duplicate idempotency key rejected';
END $$;
-- a step that already completed is not re-inserted (replay returns the stored output)
INSERT INTO toolbox_durable.operation_outputs VALUES ('run_1',1,'preview','{"ok":false}',NULL) ON CONFLICT DO NOTHING;
SELECT 'stored step output: ' || output FROM toolbox_durable.operation_outputs WHERE workflow_uuid='run_1' AND function_id=1;
-- instance B recovers workflows owned by instances with a stale heartbeat
UPDATE toolbox_durable.workflow_status w SET executor_id='inst-b', recovery_attempts=recovery_attempts+1
  FROM toolbox_durable.executors e
  WHERE w.executor_id=e.executor_id AND w.status='PENDING' AND e.heartbeat_at < now() - interval '60 seconds'
  RETURNING 'recovered ' || w.workflow_uuid || ' by ' || w.executor_id;
-- an approval arrives as a notification and is consumed once
INSERT INTO toolbox_durable.notifications (destination_uuid,topic,message) VALUES ('run_1','approval','{"approved":true,"by":"user:carol"}');
DELETE FROM toolbox_durable.notifications WHERE message_uuid = (
  SELECT message_uuid FROM toolbox_durable.notifications WHERE destination_uuid='run_1' AND topic='approval'
  ORDER BY created_at_epoch_ms LIMIT 1 FOR UPDATE SKIP LOCKED) RETURNING 'consumed ' || message;
