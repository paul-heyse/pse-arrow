DEFINE TABLE t2 SCHEMAFULL;
DEFINE FIELD s ON t2 TYPE { phase:'closed', gen:int, manifest: option<string> } | { phase:'open' };
CREATE t2:1 SET s={phase:'closed', gen:1};
CREATE t2:2 SET s={phase:'closed', gen:1, manifest:NONE};
CREATE t2:3 SET s={phase:'closed', gen:1, manifest:NULL};
CREATE t2:4 SET s={phase:'closed', gen:'1'};
CREATE t2:5 SET s={phase:'closed', gen:1.0};
SELECT * FROM t2 ORDER BY id;
-- multi-row UPDATE where one row violates the transition: statement-level atomicity
UPDATE attempts SET state={phase:'closed', close_generation:9, manifest:'mm'} WHERE state.phase IN ['ingesting','terminal'];
SELECT id, state.phase AS p FROM attempts ORDER BY id;
