-- valid creates
CREATE attempts:a1 SET ident='i1', kind='a', state={phase:'ingesting'};
CREATE attempts:a2 SET ident='i2', kind='b', opt='x', state={phase:'closed', close_generation:3, manifest:'m'};
CREATE attempts:a3 CONTENT {ident:'i3', kind:'a', state:{phase:'terminal', outcome:'completed', completion:<bytes>'abc'}};
INSERT INTO attempts {id: attempts:a4, ident:'i4', kind:'a', state:{phase:'ingesting'}};
-- illegal combos on CREATE / INSERT
CREATE attempts:b1 SET ident='x', kind='a', state={phase:'terminal', completion:<bytes>'z'};
CREATE attempts:b2 SET ident='x', kind='a', state={phase:'terminal', outcome:'weird', completion:<bytes>'z'};
CREATE attempts:b3 SET ident='x', kind='c', state={phase:'ingesting'};
CREATE attempts:b4 SET ident='x', kind='a', opt='z', state={phase:'ingesting'};
CREATE attempts:b5 SET ident='x', kind='a', state={phase:'ingesting', extra:1};
CREATE attempts:b6 SET ident='x', kind='a', state={phase:'closed', close_generation:'3', manifest:'m'};
CREATE attempts:b7 SET ident='x', kind='a';
INSERT INTO attempts {id: attempts:b8, ident:'x', kind:'a', state:{phase:'terminal', outcome:'weird', completion:<bytes>'z'}};
-- illegal combos on UPDATE / UPSERT
UPDATE attempts:a1 SET state={phase:'terminal', completion:<bytes>'z'};
UPSERT attempts:a1 SET state={phase:'terminal', outcome:'weird', completion:<bytes>'z'};
UPDATE attempts:a1 SET state.phase='terminal';
-- legal transition ingesting->closed via UPDATE
UPDATE attempts:a1 SET state={phase:'closed', close_generation:1, manifest:'m1'};
-- illegal transition terminal->ingesting (UPDATE, UPSERT, INSERT ON DUPLICATE)
UPDATE attempts:a3 SET state={phase:'ingesting'};
UPSERT attempts:a3 SET state={phase:'ingesting'};
INSERT INTO attempts {id: attempts:a3, ident:'i3', kind:'a', state:{phase:'ingesting'}} ON DUPLICATE KEY UPDATE state={phase:'ingesting'};
-- illegal skip ingesting->terminal (a4)
UPDATE attempts:a4 SET state={phase:'terminal', outcome:'failed', completion:<bytes>'f'};
-- legal closed->terminal (a2) via UPSERT
UPSERT attempts:a2 SET state={phase:'terminal', outcome:'partial', completion:<bytes>'p'};
-- READONLY
UPDATE attempts:a1 SET ident='changed';
UPSERT attempts:a1 SET ident='changed';
UPDATE attempts:a1 SET ident='i1';
UPDATE attempts:a1 CONTENT {ident:'changed', kind:'a', state:{phase:'closed', close_generation:1, manifest:'m1'}};
-- UPSERT creating a new row (before = NONE)
UPSERT attempts:a5 SET ident='i5', kind='a', state={phase:'terminal', outcome:'cancelled', completion:<bytes>'c'};
-- index use
SELECT id, state.phase AS phase FROM attempts WHERE state.phase = 'terminal' ORDER BY id;
SELECT id FROM attempts WHERE state.phase = 'terminal' EXPLAIN;
SELECT id, state FROM attempts ORDER BY id;
