-- Full refresh of every derived view (= the staleness bound if refreshed in a loop)
\timing on
\echo '== full refresh'
REFRESH MATERIALIZED VIEW mv_scores;
REFRESH MATERIALIZED VIEW mv_indeg;
REFRESH MATERIALIZED VIEW mv_depmin;
REFRESH MATERIALIZED VIEW mv_team_anc;
REFRESH MATERIALIZED VIEW mv_team_rollup;
REFRESH MATERIALIZED VIEW mv_domain_rollup;
REFRESH MATERIALIZED VIEW mv_tuples;
\echo '== permission view (ListObjects for all users), optional'
REFRESH MATERIALIZED VIEW mv_perm;
\timing off
SELECT (SELECT round(avg(passed),1) FROM mv_scores) AS avg_passed,
       (SELECT array_agg(c ORDER BY level DESC) FROM (SELECT level, count(*) c FROM mv_scores GROUP BY level) x) AS levels_g_s_b_none,
       (SELECT count(*) FROM mv_depmin WHERE min_dep_level >= 1) AS deps_ok,
       (SELECT max(dependents) FROM mv_indeg) AS max_indeg,
       (SELECT services FROM mv_team_rollup WHERE team = 0) AS root_rollup,
       (SELECT count(*) FROM mv_tuples) AS tuples,
       (SELECT count(*) FROM mv_perm) AS perm_pairs;
\echo '== concurrent (non-blocking) refresh of the scores view'
CREATE UNIQUE INDEX ON mv_scores(id);
\timing on
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_scores;
\timing off
\echo '== single-entity change, then compute-on-read for its entity page (plain views, no materialisation)'
UPDATE git_frag SET owner_team = 17, tier = 3 WHERE id = :id;
\timing on
SELECT id, passed, level FROM entity_scores WHERE id = :id;
SELECT id, passed, level FROM entity_scores WHERE id = :id;
SELECT min(s.level) AS min_dep_level FROM edges e JOIN entity_scores s ON s.id = e.dst WHERE e.src = :id;
SELECT min(s.level) AS min_dep_level FROM edges e JOIN entity_scores s ON s.id = e.dst WHERE e.src = :id;
SELECT count(*) AS dependents FROM edges WHERE dst = :id;
WITH RECURSIVE anc(t) AS (SELECT owner FROM entities WHERE id = :id
  UNION SELECT t2.parent FROM anc JOIN teams t2 ON t2.id = anc.t WHERE t2.parent IS NOT NULL)
SELECT EXISTS (SELECT 1 FROM members WHERE user_id = 42 AND team IN (SELECT t FROM anc)) AS can_view_details;
\echo '== list query computed on read: all services of team 3 subtree failing silver'
SELECT count(*) FROM entity_scores s JOIN mv_team_anc a ON a.team = s.owner WHERE a.ancestor = 3 AND s.level < 2;
\timing off
