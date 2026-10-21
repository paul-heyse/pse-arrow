DEFINE TABLE points SCHEMAFULL;
DEFINE FIELD n ON points TYPE int;
DEFINE TABLE deps TYPE RELATION IN points OUT points ENFORCED SCHEMAFULL;
DEFINE FIELD seq ON deps TYPE int;
-- 200 targets, inserted in a shuffled order so insertion order != id order
FOR $i IN array::shuffle(array::range(0, 200)) { CREATE type::record('points', $i) SET n = $i; };
CREATE points:hub SET n = -1;
-- hub -> each point with edges whose ids are random (default ULID-like/rand) created in shuffled order
FOR $i IN array::shuffle(array::range(0, 200)) { RELATE points:hub->deps->(type::record('points', $i)) SET seq = $i; };
-- each point -> hub with explicit edge ids in reverse order of creation
FOR $i IN array::range(0, 200) { LET $eid = type::record('deps', 1000 - $i); RELATE (type::record('points', $i))->deps->points:hub CONTENT { id: $eid, seq: $i }; };
-- ENFORCED: relate to a non-existent point
RELATE points:hub->deps->points:ghost SET seq = -1;
-- wrong table
CREATE nonpoint:1; RELATE points:hub->deps->nonpoint:1 SET seq = -2;
RETURN count(SELECT * FROM deps);
