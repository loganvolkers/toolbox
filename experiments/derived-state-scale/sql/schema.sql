DROP TABLE IF EXISTS git_frag, conn_frag, edges, teams, members, systems CASCADE;
CREATE TABLE git_frag(id int PRIMARY KEY, name text, owner_team int, tier int, lang text, description text,
                      pci bool, runbook bool, system int, deps text[]);
CREATE TABLE conn_frag(id int PRIMARY KEY, owner_team int, tier int NOT NULL, on_call text, coverage int, vulns int,
                       image_age int, slo bool, alerts int);
CREATE TABLE edges(src int NOT NULL, dst int NOT NULL);
CREATE TABLE teams(id int PRIMARY KEY, parent int);
CREATE TABLE members(user_id int NOT NULL, team int NOT NULL);
CREATE TABLE systems(id int PRIMARY KEY, domain int);
\copy git_frag FROM 'git.csv' CSV
\copy conn_frag FROM 'conn.csv' CSV
\copy edges FROM 'edges.csv' CSV
\copy teams FROM 'teams.csv' CSV
\copy members FROM 'members.csv' CSV
\copy systems FROM 'systems.csv' CSV
CREATE INDEX ON edges(src); CREATE INDEX ON edges(dst);
CREATE INDEX ON members(team); CREATE INDEX ON members(user_id);
VACUUM ANALYZE;
