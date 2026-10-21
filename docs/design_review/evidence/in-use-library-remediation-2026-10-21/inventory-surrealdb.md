# SurrealDB 3.3.0: capability-gap inventory for the canonical store

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Capability-gap inventory: SurrealDB 3.3.0.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

I examined commit `a791c533e` (HEAD `0530dcaa1`). No files were written, and nothing was built, run or tested.

Abbreviations used below:
- **core** = `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/surrealdb-core-3.3.0/src`
- **sdk** = `…/surrealdb-3.3.0/src`
- **types** = `…/surrealdb-types-3.3.0/src`
- **sql** = `…/surrealdb-sql-3.3.0/src`
- **surql** = `crates/pse-operations/src/generated/surreal.surql`

Evidence strength: claims about 3.3.0 sources are *Interface-checked*, claims about how the repository uses them are *Implemented*, and every recommendation is *Proposed*. Nothing here is *Tested* or *Measured*. I applied the maintainer's clarification: anything that would weaken lane parallelism (P1), daemonless durability (P2), DP-19 truthfulness, value rebind or typed outcomes is marked **remedy open**, not recommended.

---

## 1. Capability surface at 3.3.0 that matters here

| Area | 3.3.0 capability (source) | Semantics and limits that matter |
|---|---|---|
| **Field typing** | `DEFINE FIELD … TYPE` accepts literal kinds (string, int, bool, array, **object** literals) and unions. `KindLiteral::{String,Integer,…,Object(BTreeMap<String,Kind>),Bool}` at types `kind/literal.rs:17-34`. `Kind::{Either, Record(Vec<Table>), Literal, Set, Array}` at types `kind/mod.rs:51-78`. | A union of object literals gives a **tagged union per row**: `{phase:'ingesting'} \| {phase:'terminal', outcome:'completed'\|…, completion:bytes}`. `record<a\|b>` checks the table only, not that the target exists. |
| **Field clauses** | `DefineFieldStatement {readonly, value, assert, computed, default, reference, flexible, inline}` at sql `statements/define/field.rs:17-38`. `ReferenceDeleteStrategy::{Reject,Ignore,Cascade,Unset,Custom}` at sql `reference.rs:33-39`. | `ASSERT` sees `$before`, `$after`, `$input`, `$value`, so it can **guard transitions** (core `doc/field.rs:734-759`). `READONLY` refuses changed values only. |
| **Refusal kinds on the wire** | Schema refusals arrive as `kind:'Internal'` with prose: `FieldValue` (an ASSERT), `FieldReadonly`, `FieldCoerce`, `DeleteRejectedByReference`, `IndexExists`. `RecordExists` is typed as `AlreadyExists{Record{id}}` (core `err/wire_snapshot.txt:57,77,110,118,119,122,124`). | Moving a domain refusal from `THROW` into schema clauses would make it **less** typed. Schema clauses can only be a backstop. |
| **THROW** | `THROW <expr>` evaluates its argument to `value.to_raw_string()` and raises `ExecError::Thrown(String)` (core `exec/physical_expr/control_flow.rs:73-77`; `legacy/expr/expression.rs:125-127`). The display form is `"An error occurred: {0}"` (core `exec/error.rs:27`). It maps to `types::Error::thrown`, with `ErrorDetails::Thrown` and no detail payload (types `error.rs:172-179,569-570`; core `exec/error.rs:414`). The wire form is pinned (core `err/wire_snapshot.txt:200`). | A thrown **object** reaches the client as its rendered SurrealQL text. `surrealdb_syn::value(&str) -> Value` (surrealdb-syn `lib.rs:232`) parses it back. The cause is a typed kind plus a stable string; there is no structured detail channel. |
| **Typed query errors** | `QueryError::{NotExecuted, TimedOut{..}, Cancelled, TransactionConflict}` (types `error.rs:183-200`). | The repository already uses these (`canonical.rs:173`). |
| **Isolation** | Snapshot isolation, conflicts detected on commit. `SELECT … FOR UPDATE` adds read keys to the conflict set (core `dbs/statement.rs:166-208`; test `kvs/tests/select_for_update_test.rs`). | `FOR UPDATE` needs record-id targets and refuses VERSION, GROUP BY and SPLIT. Predicate reads, and therefore phantoms, are not protected. Guard-row writes or UNIQUE entries remain the tool for those. |
| **Transactions** | Text `BEGIN…COMMIT` inside one query. Interactive transactions via `Surreal::begin` → `Transaction{query,select,…,commit,cancel}` (sdk `method/transaction.rs:15-70`). | An interactive transaction holds its snapshot open across client round trips. |
| **Events** | `DEFINE EVENT … [ASYNC [RETRY n] [MAXDEPTH d]] WHEN … THEN …`. Synchronous events run inside the writing transaction. Asynchronous events are queued in that transaction and run by a server background task (core `doc/event.rs:116-172`; `kvs/tasks/mod.rs:380-425`). | When asynchronous retries are exhausted, **the queued event is deleted and a `warn!` is logged** (core `doc/event.rs:483-503`). Retries apply no backoff. |
| **Record IDs** | Array and object IDs. Range scans `t:[a,b]..[a,c]` read only that key range, without a secondary index (Context7: reference `statements/select`). | Key-order encoding of decimal components is unverified (§5, Q7). |
| **Graph** | `TYPE RELATION IN … OUT … ENFORCED`, with `->edge->` / `<-edge<-` traversal over adjacency keys. `INLINE` adjacency caches are opt-in. | |
| **Indexes** | `Index::{Idx, Uniq, Hnsw, DiskAnn, FullText, Count(Option<Cond>)}` (sql `index.rs:10-23`). | `COUNT` counts a whole table (with an optional WHERE) and cannot group by owner. |
| **Sequences** | `DEFINE SEQUENCE … BATCH START TIMEOUT` and `sequence::nextval` (Context7: `define/sequence`). | Values are not gap-free, are allocated outside the transaction, and are not in commit order. Range is i64. |
| **LIVE / CHANGEFEED** | `LIVE SELECT … WHERE`, and `CHANGEFEED <dur> [INCLUDE ORIGINAL]` with `SHOW CHANGES`. | LIVE notifications are not durable: they die with the session. A changefeed is retention-GC'd, so it cannot be a permanent journal. |
| **Reference mechanics** | Each reference writes a key unique to its (target, referencing record) pair (core `doc/field.rs:906-960`). The per-target inline cache, which would be a hot key, is used only when a table opts into `INLINE REFERENCES` (surrealdb-idx `inline_cache/mod.rs:79-81,270-300`). | |
| **Engines** | SDK features (vendored `Cargo.toml` `[features]`): `kv-mem`, `kv-rocksdb`, `kv-surrealkv`, `kv-tikv`, `kv-indxdb`, `protocol-http`, `protocol-ws`, **`protocol-grpc`**, `scripting`, `jwks`. | Embedded RocksDB is one process per directory because of the RocksDB LOCK. SurrealKV is believed to be single-process too (unverified, Q10). Only the server engine gives P2 multi-process. |
| **gRPC (3.3.0)** | Engine at sdk `engine/remote/grpc.rs`. HTTP/2 multiplexing, one RPC per call, no consumer task (`:1-18`). Each request carries a `RequestContext{session,transaction,timeout}` (`:499-505`). `GrpcConfig::max_message_size` is negotiated against the server's advertised limit (sdk `opt/grpc.rs:26-70`). The service has `attach_session/detach_session/begin/commit/cancel_transaction/query/subscribe/kill/export_*` (surrealdb-protocol `gen/rust/proto/…/surrealdb.protocol.rpc.v1.tonic.rs`). | Per Context7 (reference `rust/methods/connect`, `cli/db/environment-variables`): the server serves gRPC on the same port; sessions are attached at server scope and survive transport reconnects (`SURREAL_GRPC_MAX_ATTACHED_SESSIONS`); results stream incrementally; the LIVE buffer is bounded (`SURREAL_GRPC_NOTIFICATION_BUFFER`, `RESOURCE_EXHAUSTED`). Timeout is per connection, not per request. |
| **Export / backup** | SDK `export`/`import` (sdk `method/export.rs`, `import.rs`); CLI `surreal export/import`; gRPC `export_directory`. | |
| **Auth** | `DEFINE USER … ROLES`, `DEFINE ACCESS`, `PERMISSIONS`. | Table and function permissions bind record users only. There is no "writes only through functions" mode for system users (unresolved, Q11). |

## 2. What the repository uses

I use the coordinator's census (`…/scratchpad/review2/census.md` §1) and verified the consequential points.

**SDK and server**
- The vendored SDK enables `protocol-ws` and `rustls` only (`Cargo.toml:158-161`).
- The server is the official v3.3.0 binary on `rocksdb://…?sync=every&versioned=false` (`scripts/surreal_server.py:1457,5517-5531,6221`), started with `--query-timeout=90s --transaction-timeout=90s --deny-scripting`.
- Transactions are text `BEGIN…COMMIT`.
- Typed `QueryError` is used.
- Thrown errors are recognised by `is_thrown()` and then by prose.

**surql (verified counts)**
- 62 `SCHEMAFULL` tables (6 relation tables), 500 fields, 116 functions, 151 indexes (83 UNIQUE).
- 306 `THROW` sites, 294 distinct messages.
- 124 `FOR UPDATE`, 405 `type::record(`.
- 0 of each: `READONLY`, `REFERENCE`, literal types, `DEFINE EVENT`, `SEQUENCE`, `LIVE`, `CHANGEFEED`, `RELATE`, `->`/`<-` traversal.

**Points the census does not state**
1. **Every table has `key` plus `DEFINE INDEX <t>_key … UNIQUE`** (62 of the 83 UNIQUE indexes), emitted at `crates/pse-codegen/src/codegen/surreal.rs:124-127`. Yet every row's id is `type::record(table,key)`, and the decoder checks exactly that (`surreal.rs:136-140`). The index duplicates the primary key.
2. **Relation tables are queried through secondary indexes that duplicate graph adjacency.** Examples: `study_dependencies`/`study_dependents`, `analysis_edge_source/target`, `problem_run_edge`, `run_source_edge` (surql `:738-776`). There are zero traversals.
3. **The registry lifecycle vocabulary differs from the store's.**
   - Store outcomes are `['succeeded','failed','partial','cancelled']` (surql `:1316`, `:1933`).
   - Registry `AttemptState` is `planned…completed…superseded` (`crates/pse-schema/src/catalog/operations.rs:9-22`).
   - The generator maps Arrow `DataType` to SurrealQL and never consults `FieldContract::enum_name_of` (`crates/pse-schema/src/model/field.rs:375`; `surreal.rs:68-80`). That is why `substrate.rs` uses no `enumeration` columns.
4. **Per-problem serialization.**
   - `begin_run` touches `canonical_guards:'execution-sequence:'+problem` (surql `:947`). Every run of a problem writes that key.
   - On the client, `protected_query(&row.problem, Mutation, …)` (`canonical_execution.rs:3346`) takes an exclusive per-problem `tokio::RwLock` in `ProblemTurns` (`canonical.rs:722-786`).
   - On conflict it retries up to `GUARDED_ATTEMPTS = 32` (`:45`), then the failure becomes `Configuration("guarded operation retries exhausted")` (`:2110`).
   - Durable study points call `begin_run_with_study_admission` once per point (`workflow/study.rs:981-984`). This is a per-problem serialization point under P1.
5. **Waiting.**
   - Worker lanes poll every 500 ms (`worker.rs:117,442,527`).
   - `StudyHandle::wait(poll)` sleeps in a loop (`study.rs:2322-2334`).
   - Lanes re-read study points with `SELECT * FROM ONLY …` after guarded functions that already return the row (`canonical_studies.rs:981-992`, callers `:824,:1462,:1587`).
6. **Prose matching in production.**
   - `worker.rs:1697`: three literal message strings.
   - `canonical_files.rs:1470`: `message().ends_with("run results have protected readers")`.
7. **The vendored patch** (`vendor/surrealdb/PATCH.md`) adds to the WebSocket engine:
   - request deadlines and cancellation;
   - bounded admission (32 + 2 control);
   - checkpointed setup replay;
   - FlatBuffers response limits on nodes, depth and strings;
   - activity ownership.
   
   `diff -r` against upstream is about 34k lines (sdk) plus 2.6k (engine-api), tests included.

## 3. Unused or under-used capabilities, mapped to the audit

Verdicts: **Clear win** (no tradeoff against P1, P2 or truthfulness), **Candidate** (gated on a stated probe), **Remedy open** (only available with a regression).

### U1. Registry-generated literal and tagged-union lifecycle columns, `READONLY`, and `$before` transition asserts

**Clear win** for typing; the transition-assert part is a **Candidate** (Q1).

**Mechanism.** Teach the existing generator (`surreal.rs:68-114`) to emit, for any column carrying a registry enum (`enum_name_of`):
- `TYPE 'a'|'b'|…`, or `option<…>` when nullable;
- `READONLY` for identity columns (`key`, `run`, `generation`, `claim_operation`, `request`, `attempt`, owner columns).

Replace independent flag bags with one tagged-union field, for example on attempts:

```
TYPE { phase:'ingesting' }
   | { phase:'closed', close_generation:int, manifest:string }
   | { phase:'terminal', outcome:'completed'|'partial'|'failed'|'cancelled', completion:bytes }
```

Do the same for artifacts (`'active'|'retiring'|…`) and readiness (`dirty`/`notify`/`rebuild_pending`). Declare the allowed transitions once in the registry and emit them as `ASSERT $before = NONE OR [$before.phase.phase, $value.phase] IN [...]` (`$before` is available: core `doc/field.rs:756-759`). Then retarget the registry's `AttemptState` and `StudyPointState` at the store columns, so one vocabulary governs both, and generate the Rust enums from the same declaration.

**Owns.**
- DA-007 completely for value sets and combinations: no `terminal ∧ ingestion_open`, no `active ∧ retiring`, no wildcard projections once the Rust enums are generated.
- The store side of DA-004.
- Part of DA-019: every writer, including the f-string SQL in `scripts/surreal_server.py`, is refused at the table.
- The store side of partiality (DA-007's `Completed ∧ !usable` convention), via `outcome:'partial'`.

**Removes.**
- The hand enums with `as_str` (`canonical_execution.rs:2100-2121`).
- The Rust string sets (`canonical_files.rs:634`).
- The template literal lists (`surreal_execution.rs:566`, `surreal_studies.rs:370-371`, surql `:2478`).
- The 14 `ASSERT … IN [...]` lines.

**Fit and limits.** Schema refusals arrive as `Internal` prose (§1). Keep the domain pre-checks in the functions (U2) and treat a schema refusal as an invariant breach, classified internal. That classification is correct, because only a bypassing writer or a bug can trigger it. Q1 covers whether a union of object literals on a SCHEMAFULL table supports nested indexes and the nested `option` coercion that existing rows need.

**Reverse check.** No additional keys are written, so P1 is unchanged. P2 is unchanged. Truthfulness improves.

### U2. Typed refusal codes carried through `THROW`

**Clear win.**

**Mechanism.**
- Declare a registry enumeration such as `CanonicalRefusal` (about 290 members today; many collapse into classes).
- Templates name refusals by generated constant, and the generator emits `THROW {refusal:'study_decision_generation_changed', class:'stale_premise', …}` or a stable code-prefixed string.
- Rust decodes `is_thrown()` + `message().strip_prefix("An error occurred: ")` once, through `surrealdb_syn::value` or by code prefix, into the generated enum. That module is behind `canonical-maintenance` today and would move into the default build.
- `CanonicalError` gains `Refused{refusal, class}`, and `TypedDiagnostic` maps classes to distinct §23.2 codes.
- Lease expiry (`'execution attempt lease expired before commit'`, surql `:785`) and the fence refusals become a `WorkerAuthorityLost` class.

**Owns.** The DA-008 store-boundary bullets completely:
- prose refusals;
- the collapse into `RuntimeInfrastructure` (`canonical.rs:176-183`);
- message-matched retry (`worker.rs:1697`, `canonical_files.rs:1470`);
- worker loss encoded as `Configuration` (`durable.rs:505-538`) and recovery inferring worker loss from incidental fields.

Attaching the stage and the `effect` field remain Rust work.

**Fit and limits.** Thrown detail is a string, so the encoding must be canonical. The display prefix is pinned by core's wire snapshot. Q2 covers whether wrapping by FOR, nested `fn::` and transactions preserves the raw text. Sibling statements of a failed transaction report `NotExecuted`, which is already handled.

**Reverse check.** Neutral for P1 and P2; DP-19 improves.

### U3. Record links with `REFERENCE` for owners and cross-table references

**Candidate** (Q3).

**Mechanism.**
- Replace string foreign keys (`attempts.run`, `study_points.run/attempt`, `artifact_obligations.owner` + `owner_kind`, `result_batches.file_instance`, …) with `record<canonical_runs>` and similar.
- For polymorphic owners, use `holder: option<record<canonical_studies|canonical_analyses>>` plus `principal: option<'user'|'project'|'audit'>`. The table name becomes the owner kind.
- Add `REFERENCE ON DELETE REJECT` where deleting the target must be refused.
- The release guard becomes uniform (`IF $o.holder != NONE AND !$o.holder.retired THROW …`), which closes the unguarded `analysis` release (surql `:2487-2499`) by construction.
- Link fetch (`$a.run.problem`) replaces many of the 405 `type::record(...)` re-selects.

**Owns.**
- DA-007's owner-kind vocabulary and its release gap.
- Part of DA-025: logical references for file instances. Physical reader protection stays external, because readers are transient processes.

**Fit and limits.**
- `record<t>` checks the table, not existence. `REFERENCE` checks at delete time.
- Under snapshot isolation, a concurrent insert-reference plus delete-target is a phantom pattern; the existing `FOR UPDATE` and guard rows stay. Q3.
- REFERENCE failures arrive as `Internal`, so U2 pre-checks are still needed.

**Reverse check.** References write a unique key per pair. The per-target hot key exists only with `INLINE REFERENCES`, which must not be enabled. P1 holds provided no `INLINE` is used.

### U4. Use the primary key and graph adjacency that already exist

**Clear win** for removing `<t>_key`; a **Candidate** for traversal-ordered paging (Q4).

**Mechanism.** Drop the 62 `<t>_key UNIQUE` indexes: the record id is already unique, and a duplicate gives a typed `AlreadyExists{Record}` instead of `IndexExists`/`Internal`. Replace direction indexes on relation tables with `<-canonical_study_dependencies<-…` and `->…->` traversal over the adjacency that `INSERT RELATION` already writes. Mark `canonical_study_dependencies` `ENFORCED`.

**Owns.**
- Removes 62 index entries per row written, plus 1-2 per edge.
- Readiness reads become adjacency scans (surql `:1579`, `:1617`).
- Index entries under each guarded transaction, and therefore conflict keys, shrink.

**Reverse check.** Fewer keys per transaction, so P1 improves. P2 is unchanged.

### U5. Array record IDs with range scans for result locators

**Candidate**, secondary. The preferred remedy for DA-026 is elsewhere.

If any per-row locators survive, give `canonical_result_rows` the id `[result_set, output, partition, row]`. Range scans then replace `result_row_selection` and `_key`, and the comment-documented compound-index range workaround (`canonical_results.rs:645-650`) disappears. The full DA-026 remedy is to remove `canonical_result_rows`:
- keep block-level descriptors and zone maps in SurrealDB, which they already are (`canonical_result_blocks`, `…_block_outputs`);
- let DataFusion and Arrow find rows inside the tactical IPC payload (P3).

That second part belongs to the DataFusion review. **Reverse check:** removing the locators shrinks append transactions, so P1 improves.

### U6. The gRPC engine of the same SDK version (`protocol-grpc`)

**Candidate**, heavily gated (Q5). If any patch guarantee cannot be met, that part is **remedy open**.

**Mechanism.**
- HTTP/2 multiplexing: each request is its own RPC.
- Dropping a request future resets the stream.
- Message size limits are negotiated with the server.
- Sessions live at server scope and survive transport reconnects, so no client-side setup replay is needed after a dropped socket.
- Results stream (`stream_items`).
- LIVE subscriptions are bounded, with explicit `RESOURCE_EXHAUSTED`.

Per-request deadlines are already enforced inside the store (`fn::pse_execution_v1::deadline`, `$rpc_expiry`), together with the server's `--query-timeout`.

**Owns (if the probes pass).** Most of DA-025's patched SDK: the replay checkpoints, the setup-restoration deadlines and the FlatBuffers response-node limits. Protobuf decode is linear in message bytes, so a byte limit bounds allocation; that needs confirming.

**Limits.**
- Bounded admission (32+2) and activity ownership would need tonic concurrency limits that the SDK may not expose.
- The timeout is per connection.
- Server cancellation on stream reset is unverified; uncertain completion is still handled by receipts.

**Reverse check.** 16 lanes multiplex over HTTP/2 as they do over WebSocket. P2 is unchanged. Must not weaken the "no success inferred" rule.

### U7. LIVE SELECT as a wake-up hint

**Candidate**, a minor improvement.

A `LIVE SELECT` on claimable readiness rows (worker loop) and on the study row (`StudyHandle::wait`) wakes lanes early, with polling kept as the backstop at a longer interval. Claim authority stays in the guarded functions. It improves claim latency and idle query load; nothing in the audit names it.

**Limits.** Notifications are lost on disconnect, which is why polling must stay. The patch's LIVE decoder lacks `KILLED` (`PATCH.md`).

**Reverse check.** P1 improves (latency). P2 is unaffected because LIVE is never an authority.

### U8. `DEFINE EVENT` for readiness propagation (dirty/notify cursors)

**Remedy open.**
- **Asynchronous events** queue the work durably, but on retry exhaustion delete it with only a warning (core `doc/event.rs:493-502`). Under 16-lane conflicts without backoff, readiness could be silently lost, which violates DP-19.
- **Synchronous events** would put dependent fan-out writes into every settle transaction. Siblings sharing a dependent would then conflict, which weakens P1.

The client-cursor protocol (`readiness_notify_advance`/`scan_advance`, surql `:1635-1662`) stays unless a probe shows conflict-free fan-out. Only `pending` counts could move into a synchronous event when fan-out is small. The policy-bearing `premises`/`eligible` must stay in Rust.

### U9. `DEFINE SEQUENCE` for the per-problem run sequence

**Remedy open.**

`sequence::nextval` would remove the per-problem hot key and the `ProblemTurns` pacing of `begin_run`. But it:
- allocates out of commit order, which breaks keyset paging `sequence>$after ORDER BY sequence` (`canonical_execution.rs:3415`);
- is i64, while the registry declares u64 and a test asserts `2^63+1` (`:1920`).

It is adoptable only if run paging is re-based on a commit-ordered key or consumers tolerate skip-free re-reads. Probe Q6 first measures whether this key actually throttles 16 lanes.

### U10. Fewer round trips per point using existing SurrealQL

**Candidate**, minor.

- Use the row the guarded function already returns; the claim, settle and started functions return `$rpc_result`. This drops the re-reads at `canonical_studies.rs:824,1462,1587`.
- Project fields instead of `SELECT *`.
- Consider composing `begin_run_with_study_admission` and `admission_claim` into one guarded transaction, if their fences allow it.

Improves P1 throughput. Probe: count RPCs per durable point.

### U11. `kv-mem` for testing store functions in process

**Candidate**; this is test infrastructure, not product.

The embedded `kv-mem` engine of the same SDK could run surql function and invariant unit tests without a server, reducing shared-machine load. It relates to DA-021 (policy tested through the full runtime). It bypasses the WebSocket patch path, so transport tests stay on the server. Probe: does the vendored SDK compile with `kv-mem`?

### U12. RocksDB's exclusive lock as primary exclusion

**Lead** (Q9).

A directly launched `surreal start rocksdb://…` cannot open a directory another server holds. That is library-owned single-primary exclusion, which could shrink the supervisor's owner, PID and boot-id election machinery (DA-019/DA-025). Process placement through systemd and cgroups is outside SurrealDB. P2 is kept: the server is activated on demand and outlives its caller.

### Considered and not recommended

- **Interactive transactions.** They would hold snapshots across native or client work. Keep single-request text transactions.
- **Embedded engines for the product.** Single-process only, so they break P2's multi-process workspaces.
- **CHANGEFEED as a history authority.** Retention GC makes it unfit for permanent history. It could replace notify cursors, but that runs into the same freshness and backstop question as U8.
- **`COUNT` indexes for obligation counts.** Table-level only, so they cannot replace `canonical_history_obligation_counts`.
- **`PERMISSIONS` or `DEFINE ACCESS` to force writes through functions.** System users bypass table permissions (Q11). U1 and U3 constraints are the enforceable route.

## 4. Improvements no audit finding names

1. **Per-problem hot keys in the 16-lane path.**
   - `'execution-sequence:'+problem` is written by every `begin_run` (surql `:947`).
   - `register('retention:'+problem)` reads it with `FOR UPDATE` (surql `:1904`, `:1935`).
   - The client-side exclusive `ProblemTurns` lock serializes per-problem mutations within one process (`canonical.rs:722-786`). Across processes they meet on the server as conflicts, retried up to 32 times with backoff and then failing as `Configuration`.
   
   This is a P1 throughput and robustness issue in its own right (fix via U9 if paging is re-based, or by moving sequence allocation off the per-point path). When retries run out, the cause is mislabelled as configuration, which also bears on DA-008.
2. **U4's redundant primary-key and direction indexes:** write amplification on every canonical row and edge.
3. **`AlreadyExists{Record}` is the only typed constraint error.** With U4, idempotent `CREATE` paths can classify duplicates without prose.
4. **Unsigned 64-bit values stored as `decimal`** with three-part asserts (71 fields). Where a value provably fits in i64 (generations, revisions, ordinals), `int` with `ASSERT $value >= 0` is cheaper to compare and index. That needs a registry range decision.
5. **`canonical_study_dependencies` is not `ENFORCED`** (census), unlike the other relation tables.
6. **gRPC result streaming** (U6) would let large selections stream, instead of the bounded paging built around WebSocket response buffering.

## 5. Findings with no SurrealDB-owned remedy

- **DA-012 (lifecycle):** the store's run lifecycle functions (`begin_run`, `claim`, `seal`) are already operation-kind-agnostic, with the request as opaque bytes. A single Rust supervisor needs no store change. The horizon cancellation link is a Rust fix.
- **DA-014:** the study schema is kind-agnostic (`descriptor`/`policy` bytes), so no store gap.
- **DA-011:** keep versioned documents as `bytes` to preserve digest identity. Promote to typed columns only the lifecycle facts U1 needs.
- **DA-024, DA-032 and the Python side of DA-004** are outside this library.
- **DA-025's owner-death read tickets:** no in-use SurrealDB capability reports client process liveness. Leases are the library-neutral pattern; this is **remedy open** as far as SurrealDB is concerned.

## 6. Unresolved questions and probes

These are described only; none were run. Each would be run against a scratch `surreal start memory` or `rocksdb` at v3.3.0.

| # | Question | Probe and expected observation | Settles |
|---|---|---|---|
| Q1 | Does a tagged union of object literals on a SCHEMAFULL field enforce, coerce `option` members, and allow nested indexes? Do `$before` transition asserts fire on UPDATE and UPSERT? | Define the attempt union; `CREATE` a valid row; `UPDATE` to an illegal combination and to an illegal transition; `DEFINE INDEX … FIELDS phase.phase`. Expect `FieldCoerce`/`FieldValue` refusals and a working index. | U1's shape, and whether flags collapse into one field or stay as several literal columns |
| Q2 | Is THROW text byte-stable through nested `fn::`, `FOR`, `IF` and `BEGIN…COMMIT`, and with object values? | `THROW {refusal:'x', n:1}` at each nesting level; compare `message()` across them; round-trip through `surrealdb_syn::value`. | U2's encoding: object or code prefix |
| Q3 | Under snapshot isolation, can an insert-reference race with a delete of the target and leave a dangling reference despite `ON DELETE REJECT`? Does a `record<a\|b>` field dereference polymorphically? | Two interleaved interactive transactions (the core test pattern in `select_for_update_test.rs`); `$o.holder.retired`. | Whether U3 needs the existing `FOR UPDATE` guards kept (assumed yes) |
| Q4 | Are graph traversals ordered by edge id, and pageable with `LIMIT`/`START` the way `key` paging is today? | Insert edges, run `SELECT … FROM $p<-dep<-point LIMIT 64 START n`. | Removing U4's direction indexes |
| Q5 | Over gRPC at v3.3.0: does dropping a request stop server execution and roll back uncommitted transactions? Can concurrency and admission be bounded? Does a message-byte limit bound decoded allocation? Do sessions survive a socket loss without replay? | Patched-SDK comparison suite run on `protocol-grpc`; kill the socket mid-query; oversized result. | U6's scope and which patch parts stay (remedy open if bounded admission is lost) |
| Q6 | Does the per-problem sequence/retention guard throttle 16 lanes? | Run a 16-lane study with `QueryStatisticsObserver` on; count `TransactionConflict` retries and `ProblemTurns` waiting time per point (existing probe: `canonical_joined_publication_progress_and_ordinary_cost_probe`, `durable.rs:4309`). | Priority of §4.1 and U9 |
| Q7 | Do decimal and int components in array ids sort numerically in range scans? | Insert `t:[‘a’, 9dec]`, `[‘a’,10dec]`; range-scan. | U5 |
| Q8 | How do asynchronous events behave under conflict load (attempts, backoff, loss)? | Run 16 concurrent writers against a `RETRY 3` event; count executed against queued. | U8 stays remedy open unless loss is zero |
| Q9 | Can a second `surreal start` on a locked RocksDB directory be distinguished, by exit status or error text, from other failures? | Start two servers on one directory. | U12 |
| Q10 | Is `kv-surrealkv` multi-process? | Open from two processes. | Engine table in §1 (believed single-process) |
| Q11 | Can a system user be confined to `fn::` calls? | `DEFINE USER … ROLES EDITOR`; attempt a direct `UPDATE`. | Confirms the U1/U3-only route for DA-019 |

**Search scope and limits.** I covered the vendored and registry SDK, core, sql, types, syn, idx and protocol 3.3.0 sources, Context7 (`/surrealdb/docs.surrealdb.com`), the generated surql and its generator, `pse-operations` (canonical, transport, studies, execution, results, files) and the relevant `pse-runtime` worker and study sites. The official server crate is not installed, so server-side gRPC behaviour rests on Context7 documentation only. Absence claims about the repository come from grep over `crates/` and `scripts/surreal_server.py`.