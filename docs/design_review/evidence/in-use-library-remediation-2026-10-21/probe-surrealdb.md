# SurrealDB 3.3.0 probe: Q1–Q4

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Probe report Q1–Q4 on a scratch in-memory SurrealDB 3.3.0. Scripts under probe-artifacts/surreal/.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

All four questions now have answers from running a real server. Three results change what the inventory assumed:

- **UPSERT can report the wrong error.** When an `UPSERT <record-id> SET` or `MERGE` on an existing row is refused, the server names a different field than the one that failed. The refusal itself still holds.
- **`FOR UPDATE` alone does not stop a dangling reference.** A delete can still leave one behind despite `ON DELETE REJECT`. What prevents it is a write to the target record that actually changes a value; an `UPDATE` that changes nothing is not written.
- **Paging a traversal with outer `LIMIT/START` is slow.** It loads every edge record before it pages. Paging inside the traversal is about 100× faster.

## Setup
- **Server:** `/home/paul/.local/bin/surreal version` gives `3.3.0 for linux on x86_64`. I started it with `/home/paul/pse-arrow/scripts/pse-env -- /home/paul/.local/bin/surreal start memory --bind 127.0.0.1:39539 --user root --pass root --no-banner`. The log says `Running 3.3.0`, `Storage mode: in-memory only`.
- **Isolation:** namespace `probe`, databases `q1`–`q4`. I did not use `scripts/surreal_server.py`, `PSE_SURREAL_STATE`, the managed primary or anything under the repository.
- **Clients:**
  - Mostly HTTP `POST /sql` with `Accept: application/json` (helper `q.py`). It returns each statement's `status`, `kind`, `details` and `result` text, which is the wire form the SDK decodes.
  - The `surreal sql --json` CLI as a second renderer (`CLI.sh`).
  - Concurrency: two Python threads, each sending one HTTP request that holds a text `BEGIN…COMMIT` with `sleep(2s)` inside (`race_lib.py`, `q3_race*.py`, `ww.py`).
  - Scripts and `.surql` files are in `/tmp/claude-1000/-home-paul-pse-arrow/15c6fe71-a184-4329-a417-2fc2e56d8c52/scratchpad/surreal-probe/`.
- **Termination:** the server was stopped. Nothing answers on 39539, and no process with that bind is left. The two other `surreal` processes (library-context, ports 33385 and 54085) were not mine and were not touched.

---

## Q1: typed lifecycle

```sql
DEFINE TABLE attempts SCHEMAFULL;
DEFINE FIELD ident ON attempts TYPE string READONLY;
DEFINE FIELD kind  ON attempts TYPE 'a' | 'b';
DEFINE FIELD opt   ON attempts TYPE option<'x' | 'y'>;
DEFINE FIELD state ON attempts TYPE
    { phase: 'ingesting' }
  | { phase: 'closed', close_generation: int, manifest: string }
  | { phase: 'terminal', outcome: 'completed'|'partial'|'failed'|'cancelled', completion: bytes }
  ASSERT $before = NONE OR $before.phase = $value.phase
      OR [$before.phase, $value.phase] IN [['ingesting','closed'], ['closed','terminal']];
DEFINE INDEX attempts_phase ON attempts FIELDS state.phase;
```

The server stores `option<'x'|'y'>` as `none | 'x' | 'y'`, and `IN` as `INSIDE`. Inside a field `ASSERT`, `$before` is the field's previous value, not the whole document: `$before.phase` worked.

Raw refusals (trimmed). Every one comes back as `kind:"Internal"` with no `details`:
```
Couldn't coerce value for field `state` of `attempts:b2`: Expected `{ phase: 'ingesting' } | { close_generation: int, manifest: string, phase: 'closed' } | { completion: bytes, outcome: 'completed' | 'partial' | 'failed' | 'cancelled', phase: 'terminal' }` but found `{ completion: b"7A", outcome: 'weird', phase: 'terminal' }`
Found { phase: 'ingesting' } for field `state`, with record `attempts:a3`, but field must conform to: $before = NONE OR ... INSIDE [['ingesting', 'closed'], ['closed', 'terminal']]
Found changed value for field `ident`, with record `attempts:a1`, but field is readonly
```
The CLI prints the same text as a bare JSON string, e.g. `["Found changed value for field `ident` ... but field is readonly"]`.

| Case | CREATE | INSERT | UPDATE | UPSERT … SET/MERGE (existing row) | UPSERT … CONTENT / UPSERT table WHERE id= |
|---|---|---|---|---|---|
| Valid union member | OK | OK | OK | OK | OK |
| terminal with no `outcome` | refused, coerce | n/a | refused, coerce | refused, **wrong field named** (`ident … found NONE`) | n/a |
| Unknown `outcome:'weird'` | refused, coerce | refused, coerce | n/a | refused, **wrong field named** | n/a |
| Partial edit `SET state.phase='terminal'` | n/a | n/a | refused, coerce (`found { phase: 'terminal' }`) | n/a | n/a |
| Extra key `{phase:'ingesting', extra:1}` | refused, coerce | | | | |
| `close_generation:'3'` (string) | refused, coerce | | | | |
| `state` missing | refused (`found NONE`) | | | | |
| `kind='c'` / `opt='z'` | refused, coerce | | | refused, **wrong field named** | refused, coerce (correct field) |
| terminal→ingesting | n/a | refused, ASSERT (`ON DUPLICATE KEY UPDATE`) | refused, ASSERT | refused, **wrong field named** | refused, ASSERT |
| ingesting→terminal (skipping closed) | | | refused, ASSERT | | |
| ingesting→closed and closed→terminal | | | OK | OK | OK |
| Change the `READONLY` field | | | refused, readonly | refused, **wrong field named** (`kind … found NONE`) | refused, readonly |
| Rewrite `READONLY` with the same value | | | OK | | |
| New row via UPSERT (`$before = NONE`) | | | | OK, any phase | |
| Multi-row UPDATE where the 2nd row is illegal | | | whole statement rolled back (the legal 1st row is unchanged) | | |
| `WHERE state.phase = 'terminal'` | | | `EXPLAIN` gives `IndexScan index:attempts_phase access:= 'terminal'`; 3 correct rows | | |

What the data shows:
- **Enforcement holds on every write path.** The tagged union, the literal unions, `option<literal>`, `READONLY` and the `$before` transition rule were enforced on CREATE, INSERT, `INSERT … ON DUPLICATE KEY UPDATE`, UPDATE and UPSERT. An index on `state.phase` is defined and used.
- **The UPSERT error text is unreliable.** On an existing record, an `UPSERT <id> SET/MERGE` that fails reports a "fresh record" error, e.g. `Couldn't coerce value for field `ident` … found NONE`, instead of the real violation. I reproduced this 6 times. The row stays unchanged. A successful `UPSERT … SET` updates the existing row correctly. `UPSERT … CONTENT` and `UPSERT table SET … WHERE id = …` report the real cause. Anything that decodes schema refusals from UPSERT text gets the wrong answer.
- **No typed error.** Schema refusals arrive with `kind:"Internal"`, prose only and no `details`.
- **`option` inside an object literal works.** For a variant member `manifest: option<string>`, both leaving it out and setting it to `NONE` are accepted. `NULL` is refused.
- **`int` inside an object literal accepts but does not convert.** A whole-valued float or decimal passes the check and is stored as it was given: `gen:1.0` is stored as a `float`, `2dec` as a `decimal`. `1.5` is refused. A top-level `TYPE int` field converts `1.0` to the int `1`.
- **First-use oddity:** the first `DEFINE TABLE` against a database never used before failed with `NotFound{Database}`. The statements after it in the same request succeeded. To be safe, define the database first.

## Q2: THROW stability

```sql
DEFINE FUNCTION fn::p::top($m: string) { IF $m = 'obj' { THROW {refusal:'study_decision_generation_changed', class:'stale_premise', n:1} } ELSE { THROW 'pse:study_decision_generation_changed' } };
DEFINE FUNCTION fn::p::inif($m: string, $c: bool) { IF $c { IF $m='obj' {THROW {...}} ELSE {THROW 'pse:...'} }; RETURN 'no' };
DEFINE FUNCTION fn::p::infor($m: string) { FOR $i IN [1,2,3] { IF $i = 2 { …same two THROWs… } }; RETURN 'no' };
DEFINE FUNCTION fn::p::nested($m: string)  { LET $x = fn::p::infor($m); RETURN $x };
DEFINE FUNCTION fn::p::nested2($m: string) { FOR $j IN [1] { LET $x = fn::p::nested($m) }; RETURN 'no' };
-- callers: each fn at top level; bare THROW; BEGIN; CREATE side:1; LET $r = fn::p::nested('obj'); CREATE side:2; COMMIT;
-- (and the 'str' variant, a bare THROW in a txn, IF-in-txn, and CREATE side:9 SET v = fn::p::top('obj'))
```

Every thrown error, whatever the nesting, has wire `kind:"Thrown"`, no `details`, and one of exactly two `result` strings:
```
An error occurred: { class: 'stale_premise', n: 1, refusal: 'study_decision_generation_changed' }   (9 occurrences)
An error occurred: pse:study_decision_generation_changed                                             (8 occurrences)
```

| Nesting | Object THROW | String THROW |
|---|---|---|
| Top-level statement | identical | identical |
| fn body, top-level IF | identical | identical |
| fn, nested IF | identical | identical |
| fn, FOR → IF | identical | identical |
| fn → fn (LET) | identical | identical |
| fn → FOR → fn → fn → FOR | identical | identical |
| Inside `BEGIN…COMMIT` | identical | identical |
| Inside `CREATE … SET v = fn()` | identical | (not run) |
| CLI rendering | the same string, bare | the same string, bare |

In a transaction `BEGIN; CREATE a; <throw>; CREATE b; COMMIT;` the statements report:

| Statement | Result |
|---|---|
| BEGIN | OK |
| CREATE before the throw | `Query/NotExecuted` "The query was not executed due to a failed transaction" (reported this way even though it ran) |
| The THROW | `Thrown` |
| CREATE after the throw | **`Query/Cancelled`** "…due to a cancelled transaction" |
| COMMIT | `Query/NotExecuted` "Cannot COMMIT: the transaction was aborted due to a prior error" |

All writes were rolled back (`SELECT * FROM side` gave `[]`).

Other observations:
- **The object form is canonical text.** Keys come out sorted, values in SurrealQL literal syntax. A complex object (`1.5f`, `1dec`, `side:1`, `d'…'`, `NONE`/`NULL`, escaped quotes) parsed back equal to the original (`RETURN <rendered> == <original>` gave `true`).
- **Multi-line strings pass through unchanged** (`"An error occurred: multi\nline 'quoted'"`).
- **Siblings after the failure** report `Cancelled`, not `NotExecuted`.

## Q3: record links with REFERENCE

```sql
DEFINE TABLE studies SCHEMAFULL;  DEFINE FIELD retired ON studies  TYPE bool DEFAULT false;
DEFINE TABLE analyses SCHEMAFULL; DEFINE FIELD retired ON analyses TYPE bool DEFAULT false;
DEFINE TABLE obligations SCHEMAFULL;
DEFINE FIELD owner ON obligations TYPE option<record<studies|analyses>> REFERENCE ON DELETE REJECT;
```

| Test | Result |
|---|---|
| `owner = studies:s1` / `analyses:x1` / not set | OK |
| `owner = other:o1` | refused, coerce (`Expected none | record<studies | analyses>`) |
| `owner = 'studies:s1'` (string) | refused, coerce |
| `owner = studies:ghost` (target does not exist) | **accepted** |
| `$o.owner.retired` on an analyses owner | `false` (dereferences across both tables) |
| `WHERE owner != NONE AND !owner.retired` | includes **`obligations:6` (ghost owner)**: `owner.retired` is NONE, and `!NONE` is true |
| `<~obligations` on a target | lists the referrers |
| `DELETE studies:s1` (referenced) | refused, `kind:Internal` "Cannot delete `studies:s1` as it is referenced by `obligations:1` with an ON DELETE REJECT clause" (the same for analyses, and for `DELETE … WHERE`) |
| Unset the reference, then delete | OK |

Concurrent races. "Dangling" means the final state has the reference row and no target:

| # | Interleaving (T1 starts first, T2 starts 0.5 s later) | Who commits first | Outcome |
|---|---|---|---|
| A | T1 create-ref + sleep; T2 delete | T2 (delete) | **Dangling**, both commit |
| B2 | T1 delete + sleep; T2 create-ref | T2 (ref) | **Dangling** |
| C | T1 `SELECT target FOR UPDATE` + create-ref + sleep; T2 delete | T2 (delete) | T1 fails with `TransactionConflict` "Read conflict"; safe |
| D | T1 delete + sleep; T2 `FOR UPDATE` + create-ref | T2 (ref) | **Dangling** |
| G | Both sides `FOR UPDATE` on the target; T1 delete + sleep | T2 (ref) | **Dangling** |
| E / F | Referrer does `UPDATE target SET retired = retired` (no change), in either order | either | **Dangling** |
| E2 | T1 delete + sleep; T2 `UPDATE target SET touch += 1` + create-ref | T2 | T1 fails with `TransactionConflict` "Write conflict"; target and ref both survive; safe |
| F2 | T1 `touch += 1` + create-ref + sleep; T2 delete | T2 | T1 fails with "Write conflict"; nothing dangles; safe |
| N | `UPDATE … SET retired = retired` vs a concurrent real UPDATE | T2 | no conflict, T2's value persists: an unchanged UPDATE is not written |

What the data shows:
- **REJECT is checked only at delete time.** The check does not catch a racing reference insert. The deleter's scan of referrers is not in its conflict set (A, B2, D, G).
- **`FOR UPDATE` on the referrer side helps in one order only.** It protects when the deleter commits first (C), but not when the deleter commits last (D, G).
- **A write that changes a value is safe in both orders** (E2, F2). A no-op update gives no protection (E, F, N).
- **A dangling owner reads as "not retired."** `!owner.retired` treats a missing target the same as a live, unretired one.

## Q4: traversal paging

```sql
DEFINE TABLE points SCHEMAFULL; DEFINE FIELD n ON points TYPE int;
DEFINE TABLE deps TYPE RELATION IN points OUT points ENFORCED SCHEMAFULL; DEFINE FIELD seq ON deps TYPE int;
FOR $i IN array::shuffle(array::range(0,200)) { CREATE type::record('points',$i) SET n=$i; };
FOR $i IN array::shuffle(array::range(0,200)) { RELATE points:hub->deps->(type::record('points',$i)) SET seq=$i; };   -- random edge ids
FOR $i IN array::range(0,200) { RELATE (type::record('points',$i))->deps->points:hub CONTENT {id: type::record('deps',1000-$i), seq:$i}; };
```

`ENFORCED` checks: relating to `points:ghost` gives `NotFound{Record}` "The record 'points:ghost' does not exist"; relating to `nonpoint:1` gives an Internal coerce error.

| Query | Order and paging result |
|---|---|
| `SELECT ->deps->points AS o FROM points:hub LIMIT 64 START n` | LIMIT/START apply to the **outer row** (one row holding all 200 items; `START 64` gives `[]`). This form cannot page. |
| `SELECT VALUE ->deps->points FROM ONLY points:hub` | **edge-id order** (not target-id order, not creation order) |
| `SELECT VALUE id FROM points:hub->deps->points LIMIT 64 START 0/64/128/192` | pages 64/64/64/8, 200 unique; joined they equal edge-id order; 5 repeats identical |
| `points:hub<-deps<-points` (edge ids deps:801..1000, created in descending id order) | edge-id ascending (`points:199` first); paging joins exactly |
| `RETURN points:hub->(deps LIMIT 64 START n)->points` | pages join to the full order (forward and inverse) |
| `RETURN points:hub->(deps WHERE id > $cursor LIMIT 64)` | keyset paging joins to the full edge-id order |
| `SELECT … FROM deps WHERE in = points:hub` (no index) | `EXPLAIN` gives `TableScan` with a predicate, i.e. a full table scan |

Server-reported times at 20,000 edges on one hub. Single machine, in-memory engine, 3–5 repeats each; indicative only.

| Form | Time |
|---|---|
| `SELECT VALUE id FROM points:hub2->deps LIMIT 64 START {0, 10000, 19936}` | ≈265–355 ms, whatever START is. `EXPLAIN` is `Limit` over `SourceExpr(GraphEdgeScan)`, so it loads every edge record first. |
| `SELECT … FROM points:hub2->deps WHERE id > … LIMIT 64` | ≈300–330 ms |
| `RETURN points:hub2->(deps LIMIT 64 START 10000)` | ≈2.5–6 ms |
| `RETURN points:hub2->(deps WHERE id > deps:5 LIMIT 64)` | ≈0.35 ms |
| `RETURN array::slice(points:hub2->deps, 10000, 10064)` | ≈3 ms |
| Bare `RETURN points:hub2->deps->points` (all 20k) | ≈7 ms |
| `deps WHERE in = …` without an index, `START 10000` | ≈9–12 ms |
| The same with `DEFINE INDEX … FIELDS in`, `START 0` / `START 19900` | ≈1.3 ms / ≈18 ms; same edge-id order as the traversal |

What the data shows:
- **Traversal order is deterministic** and follows edge id in both directions.
- **Pageable forms:** `FROM <traversal> LIMIT/START` pages correctly but is slow. The fast forms are the in-idiom `->(deps LIMIT n START m)` and keyset `->(deps WHERE id > $c LIMIT n)`.

## Caveats
- **Engine:** in-memory only. Isolation, the order in which conflicts are detected, and timings may differ on RocksDB or the server engine the repository runs. Q3's results are about this engine.
- **Client:** I used HTTP `/sql` and the CLI, not the Rust SDK. The `kind`/`details` shapes are the JSON wire form. I did not check the SDK's `message()` or `is_thrown()` directly, and I did not test `surrealdb_syn::value`. The round-trip was checked only by the server parsing the rendered literal back.
- **Concurrency method:** text `BEGIN…COMMIT` plus `sleep(2s)` and a 0.5 s start offset, not SDK interactive transactions. Each scenario ran once. In B, both requests started together, so their internal order was not controlled; B2 repeats it with the order controlled.
- **Not run:** INSERT for the READONLY and partial-edit cases; RELATE edges carrying a union field; `INLINE REFERENCES`.
- **Scale:** Q4 timings come from one hub and a single repeat set.

*Tested:* Q1–Q4 as tabulated, on SurrealDB 3.3.0 (`surreal start memory`, Linux x86_64), over HTTP `/sql` and the `surreal sql --json` CLI, on 2026-10-21. *Measured (indicative):* the Q4 server-reported query times at 20k edges, under the same conditions.