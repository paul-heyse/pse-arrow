SELECT s.gen AS g, type::of(s.gen) AS ty FROM t2:5;
DEFINE FIELD n ON t2 TYPE int; 
UPDATE [attempts:a4, attempts:a1] SET state={phase:'closed', close_generation:9, manifest:'mm'};
SELECT id, state.phase AS p FROM [attempts:a4, attempts:a1];
