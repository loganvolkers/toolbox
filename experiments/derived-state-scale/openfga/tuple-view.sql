-- Replace OpenFGA's `tuple` table with a read-only view over the derived tuples (mv_tuples), so the
-- OpenFGA server (sidecar, Postgres datastore) reads tuples that were never written as source data.
-- :store is the OpenFGA store id. Writes through the OpenFGA API will fail, by design.
ALTER TABLE tuple RENAME TO tuple_unused;
CREATE MATERIALIZED VIEW fga_tuple AS
  SELECT :'store'::text AS store,
         split_part(object, ':', 1) AS object_type, split_part(object, ':', 2) AS object_id,
         relation, subject AS _user,
         CASE WHEN subject LIKE '%#%' THEN 'userset' ELSE 'user' END AS user_type,
         NULL::text AS condition_name, NULL::bytea AS condition_context,
         md5(object || '|' || relation || '|' || subject) AS ulid, now() AS inserted_at
  FROM mv_tuples;
CREATE INDEX ON fga_tuple (store, object_type, object_id, relation, _user);
CREATE INDEX ON fga_tuple (store, object_type, relation, _user);
CREATE VIEW tuple AS SELECT * FROM fga_tuple;
ANALYZE fga_tuple;
