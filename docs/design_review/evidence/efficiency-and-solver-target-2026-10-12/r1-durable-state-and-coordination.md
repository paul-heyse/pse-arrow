> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R1 (library-research), Part A: durable state, coordination and lifecycle invariants. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R1: durable state, coordination and lifecycle invariants (Part A)

Supporting inquiry for the consolidated efficiency and solver target review. Bounded
contribution: facts, capabilities, obligations and allocations. No verdict, no finding IDs.
Baseline `main` at `6c368c88a`, 2026-10-12. Static reading only; no probes, builds or services.

**Sources and pins.** SurrealDB SDK 3.3.0 (vendored, patched, `protocol-ws` + `rustls` only,
`Cargo.toml` `[workspace.dependencies]`), server binary 3.3.0 launched as
`rocksdb://<state>/database?sync=every&versioned=false` (`scripts/surreal_server.py::5418`);
pinned crate sources under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`
(`surrealdb-core-3.3.0`, `surrealdb-engine-local-3.3.0`, `surrealdb-kvs{,-rocksdb,-surrealkv}-3.3.0`,
`tokio-1.53.2`, `tokio-util-0.7.19`); `neo4j-surrealdb` skill (SurrealDB half: `surrealdb/GUIDE.md`,
capability briefs, `content/catalogs/surrealql.md`, probes `SB###` executed 2026-10-05 against 3.3.0
embedded and the pinned image); Context7 (`/websites/rs_tokio-util`, `/surrealdb/docs.surrealdb.com`).
Probe IDs below are the skill's existing executed run/control probes, reused within their original
scope (embedded mem/SurrealKV/RocksDB and memory-backed remote server unless noted); they are not
fresh product tests.

**Prior evidence reused (not redone).**
`docs/design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md` (P28);
`docs/design_review/evidence/surrealdb-unified-substrate-2026-10-05/` (U);
`git show e145eb7d2:docs/design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/library-capabilities-and-tooling.md` (LCT);
`docs/dev/surreal-substrate.md` "Capability choices and revisit triggers" I1–I7;
`docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/data-and-infrastructure-library-utilization.md` (EFF, notably L2-02, L2-04).
**What changed since them:** nothing in the pins (SDK/server 3.3.0, tokio 1.53.2, tokio-util 0.7.19,
SurrealKit 1.0.0-beta.6 as LCT assessed). New here: embedded-engine cancellation and shutdown contracts
read from `surrealdb-engine-local-3.3.0` (F10), view/event/reference execution placement read from
core `doc/*.rs` (F3–F5), RocksDB `SyncMode` and grouped commit (F1), live notification
backpressure (F6), tokio-util 0.7.19 surface beyond the four primitives in use (F17).

---

## Part 1. Capability surface at the pinned versions (neutral; before current code)

Each family: **does** / **contract** / **limits** / **newer release** / **responsibility it could serve**.

### F1. Transactions, isolation and commit durability

- **Does.** `BEGIN … COMMIT` blocks in one `query()` (one round trip); client transactions
  `Surreal::begin()` → `Transaction::{query,commit,cancel}`; per-statement implicit transactions.
- **Contract.** All-or-nothing per block: one failing statement cancels the block (`SB031`).
  Isolation is **snapshot isolation with optimistic commit-time conflict detection** on every native
  backend (RocksDB `OptimisticTransactionDB` with `set_snapshot(true)` at begin,
  `surrealdb-kvs-rocksdb-3.3.0/src/lib.rs`; SurrealKV and mem likewise). Write–write conflicts fail
  the second committer; **write skew commits** unless the read set is locked with
  `SELECT … FROM <record ids> FOR UPDATE` (`SB091`, `SB110`). `FOR UPDATE` takes **record ids only**,
  not tables/predicates/ranges, and cannot combine with GROUP BY/SPLIT (`SB092`); it registers keys for
  commit validation (RocksDB `get_for_update(exclusive)`, EFF L2-02), including absent ids — it is an
  optimistic conflict marker, not a blocking mutex or predicate lock. **No phantom protection** for range
  scans. **No shared-lock mode.** Conflicts are **not retried** by the SDK; embedded the error is kind
  `Internal` with backend-specific wording, over ws `QueryError::TransactionConflict` (`SB033`, `SB053`,
  `SB089`, `SB090`, `SB109`). `query_timeout` bounds a whole call; `transaction_timeout` bounds each
  statement, not an idle client transaction (`SB096`); statement `TIMEOUT` writes nothing (`SB097`).
  `db.query()` is `Ok` when a statement failed — `.check()` required (`SB013`). A parse error fails the
  whole call before anything runs (`SB049`).
- **Durability.** RocksDB/SurrealKV `SyncMode::{Never, Every (default), Interval(d)}`
  (`surrealdb-kvs-3.3.0/src/config.rs::SyncMode`); `Every` fsyncs the WAL per commit; RocksDB groups
  concurrent commits (`grouped_commit_timeout` default 5 ms, batch cap scaled with memory,
  `kvs-rocksdb/src/cnf.rs`). Power-loss survival beyond WAL-fsync semantics is not established by any
  source read here.
- **Limits.** Optimistic conflicts surface only at commit, so long transactions with wide read sets
  conflict more. Hot single keys (counters, guard records, `GROUP ALL` views — F5) serialize every
  writer that touches them. Isolation differs on IndexedDB (read-set validation, `SB114`) — not a
  deployment candidate here.
- **Newer release.** None material found for 3.3.x transactions in the captured docs; the registry also
  holds `surrealdb-collections-3.3.3`, i.e. a 3.3.3 line exists, with no content inspected (uncertainty).
- **Serves.** Atomic multi-record lifecycle transitions; uniqueness and fencing by unique keys;
  compare-and-set via `FOR UPDATE` on named ids; crash-atomic state change.

### F2. Schema constraints and field computation

- **Does.** `SCHEMAFULL` tables refuse undefined fields (`SB028`); `DEFINE DATABASE … STRICT` refuses
  undefined tables (`SB029`); `TYPE` with coercion rules (`SB011`, `SB047`; non-`option` is required);
  `ASSERT` expressions; `READONLY` fields; `VALUE` (write-time), `DEFAULT`, `COMPUTED` (read-time,
  `SB125`); `DEFINE INDEX … UNIQUE` (composite allowed), `CONCURRENTLY` builds (`SB099`); `COUNT`
  indexes; `DEFINE PARAM`.
- **Contract.** Constraints are checked in the writing transaction. UNIQUE violations are kind
  `Internal`, not `is_already_exists()` (`SB015`); `INSERT IGNORE` skips them silently (`SB048`);
  `DEFINE INDEX UNIQUE` over existing duplicates fails and creates no index (`SB030`). `READONLY`
  rejects changed values but allows same-value UPSERT and does not stop delete/recreate (P28).
- **Limits.** Single-record and single-index scope: no cross-record CHECK, no conditional uniqueness
  except through computed key fields; u64 above `i64::MAX` wraps silently (`SB046`).
- **Serves.** Shape authority at the storage boundary; identity uniqueness; generation fences as
  unique (owner, generation) keys; refusing accidental tables.

### F3. References, relations and graph traversal

- **Does.** Plain record links (dangle after target delete, read `NONE`, `SB023`); `REFERENCE` fields
  with `ON DELETE IGNORE (default) | UNSET | CASCADE | REJECT | THEN {…}` and backward reads with `<~`
  (`SB023`, `SB024`; docs `language-primitives/record-references.mdx`); edge tables via `RELATE` /
  `INSERT RELATION` (`SB027`); `TYPE RELATION IN a OUT b ENFORCED` refuses dangling endpoints
  (`SB022`); deleting a node deletes its edges (`SB021`); duplicate edges unless UNIQUE on `(in,out)`
  (`SB020`); `LIGHTWEIGHT` relations (endpoint pair only); `INLINE EDGES|REFERENCES n` adjacency caches;
  arrows, recursive idioms `.{1..n}`, `+collect`, `+path`, `+shortest` (`SB025`, `SB026`).
- **Contract.** Reference delete actions run inside the deleting transaction (`core/src/doc/purge.rs`).
  `UNSET` on a required field behaves as `REJECT`. Cascades bypass the referencing record's
  permissions but edge cascades use the caller's (docs). `REFERENCE` cannot be nested.
- **Limits.** Only typed `record<…>` values are tracked: string identities, bytes and ids embedded in
  payloads are invisible (LCT). No time-, lease- or reader-aware semantics: a reference says "exists",
  not "is protected until X settles". LIGHTWEIGHT edges carry no fields, indexes, events or
  changefeeds; INLINE caches have caps 0–256 with permanent scan fallback after overflow (LCT).
  No graph algorithms beyond shortest path.
- **Serves.** Referential integrity of retained artifacts (refuse delete while referenced; cascade
  dependent rows); lineage/provenance traversal; reachability for retention.

### F4. Events (`DEFINE EVENT`)

- **Does.** Per-table triggers on CREATE/UPDATE/DELETE with `$event`, `$before`, `$after`, `$input`,
  `$auth`, `WHEN` filter and a `THEN` block.
- **Contract.** **Synchronous** events run in the triggering transaction; `THROW` aborts the write
  (`SB122`). **`ASYNC [RETRY n] [MAXDEPTH d]`** (3.0+): enqueue is transactional with the change; execution
  is a **later, separate transaction** seeing then-current state, polled by a maintenance task every
  `SURREAL_ASYNC_EVENT_PROCESSING_INTERVAL` (default 5 s); "generally" ordered, may run in parallel within
  a batch; MAXDEPTH default 3 (0–16). At-least-once attempts with a finite retry budget; success is not
  guaranteed and the action must be repeat-safe (LCT). Requires maintenance tasks (`SB104`).
- **Limits.** Sync events add work and conflict surface to every triggering write; ASYNC cannot own
  atomic admission or protection (P28, LCT); bulk import/backfill interplay unqualified.
- **Newer release.** Docs state write-conflict attempts stop consuming RETRY only from **3.3.1** (LCT);
  not applicable to the 3.3.0 pin.
- **Serves.** Same-transaction derived bookkeeping (counters, audit rows, backlinks, refusing
  illegal transitions); deferred follow-up work inside the store.

### F5. Derived tables and computed state

- **Does.** `DEFINE TABLE v AS SELECT … FROM t [WHERE] [GROUP BY|ALL]` incremental views (include
  pre-existing rows, follow later writes, refuse direct writes; `SB124`); `DROP` tables (accept writes
  only to trigger events/views); `COMPUTED` fields.
- **Contract.** View maintenance runs **inside the writing transaction** (`core/src/doc/table.rs`
  "within the currently running transaction").
- **Limits (interpretation).** A grouped view row is a single hot key: concurrent writers to the source
  table then conflict on it (F1). Aggregates are restricted to the supported view grammar.
- **Serves.** Always-consistent counts/summaries (e.g. outstanding obligations per owner) without
  application-maintained counters.

### F6. Change observation: LIVE SELECT and CHANGEFEED

- **LIVE SELECT.** Push of Create/Update/Delete to a connected client, embedded or ws, not http
  (`SB041`, `SB051`). Notifications are **buffered and delivered after commit**
  (`core/src/kvs/ds.rs` broker docs); the embedded channel is bounded (15,000) and the sender awaits
  (backpressure, not drop). Subscriptions **do not survive a ws reconnect** and are not re-subscribed
  (`SB101`); dropping a Stream leaves the live query registered until the connection closes (`SB135`);
  `KILL` never fails (`SB134`); table must exist (`SB042`). No replay of missed notifications.
- **CHANGEFEED.** `DEFINE TABLE … CHANGEFEED <duration> [INCLUDE ORIGINAL]`; `SHOW CHANGES FOR TABLE t
  SINCE <versionstamp|datetime> LIMIT n` returns versionstamp-ordered changes; without a declared feed it
  returns `[]` silently (`SB043`). Retention is time-bounded; GC needs maintenance tasks (`SB104`).
  The consumer owns its cursor, idempotent application and gap recovery (LCT).
- **Limits.** Neither is a durable queue or completion authority (P28, I2). Live is lossy across
  disconnects; changefeeds are lossy past retention.
- **Serves.** Wake-ups replacing polling; incremental readers (results, readiness); audit streams.

### F7. Sequences and identity generation

- **Does.** `DEFINE SEQUENCE s [BATCH n] [START m] [TIMEOUT]`, `sequence::nextval`; record-id generators
  (`rand`, `ulid`, `uuid`); complex (array/object) record ids and id ranges (`SB127`: upper bound
  exclusive).
- **Contract.** Sequences are **unique and increasing, not gap-free**; values taken in a cancelled
  transaction are lost (`SB123`).
- **Serves.** Monotone fencing tokens/generations, attempt numbering, ordered cursors without a
  read-modify-write hot key on the application side.

### F8. In-database logic

- **Does.** `DEFINE FUNCTION fn::…` (runs in the caller's transaction, recursion limit, typed args);
  `DEFINE API` HTTP handlers / `api::invoke` (`SB133`, `SB136`); `DEFINE MODULE` Surrealism WASM
  (experimental, unprobed); embedded JS scripting (feature + capability).
- **Limits.** Functions are SurrealQL (no native numeric work); errors surface as statement errors;
  closures inside read-only contexts could not write before 3.3 (docs).
- **Serves.** Single-round-trip guarded state transitions with server-side validation.

### F9. Authentication, permissions and capabilities

- **Does.** `DEFINE USER` (roles OWNER/EDITOR/VIEWER), `DEFINE ACCESS` (record, JWT, bearer), table
  and field `PERMISSIONS`, server/core capabilities (eval, net, scripting, routes).
- **Contract/limits.** VIEWER writes return `[]` with no error (`SB129`); forbidden record-user updates
  return `[]` silently (`SB093`); an **embedded engine without `Config::user` has auth off** (`SB094`).
- **Serves.** Separating a writer role from read-only inspectors; least-privilege tooling (MCP).

### F10. Deployment shapes and SDK features

- **Embedded** (`kv-mem`, `kv-surrealkv`, `kv-rocksdb`; none enabled at the pin): the `Datastore` runs
  in-process; **one process per on-disk directory** (`SB003`); `Surreal` is `Clone`. Query execution runs
  **on the caller's task**; dropping a query future aborts it mid-batch and the engine sets the
  client transaction aside so a later commit reports rather than persists a half-run batch
  (`surrealdb-engine-local-3.3.0/src/engine.rs::AbandonOnCancel`). Closing the last handle closes the
  session channel and shuts the datastore down (`engine-local/src/lib.rs::connect`);
  `Datastore::shutdown` cancels maintenance tasks, waits with a bounded timeout, then closes storage and
  refuses later commits (`core/src/kvs/ds.rs::shutdown`). Maintenance tasks (index compaction, tombstone
  reclaim, changefeed GC, ASYNC events, interrupted index-build resumption) must be running
  (`maintenance_tasks_running`). Cost: builds the core into the product (C++/libclang for RocksDB;
  SurrealKV pure Rust); `surrealdb-core` is explicitly not SemVer-stable (`core.embedding` brief).
- **Remote** ws/http/grpc: same answers (`SB056`); ws reconnect redials every second and replays
  sign-in/ns/db/vars; in-flight queries fail `Connection reset`; queries sent while redialling wait
  (`SB100`). Upstream has **no request-local cancellation or bounded admission**; the vendored patch adds
  them (`vendor/surrealdb/PATCH.md`: request contexts, 32+2 admissions, 64-entry ordered replay, physical
  drain on disconnect; "does not establish that the server … stopped"). gRPC needs `protocol-grpc`
  (not enabled) and streams provisional rows that may be retracted (`SB103`, P28).
- **Contract difference (interpretation).** Embedded: client drop ⇒ in-process abort, single owner,
  no network unknown-outcome. Remote: an unacknowledged write has an unknown outcome that must be
  settled by identity (the reason the project's settlement/receipt machinery exists, §20.6).
- **Serves.** Choice of process topology for the store.

### F11. History and time travel

- **Does.** `SELECT … VERSION <datetime>`, `INFO … VERSION`, subqueries inherit VERSION.
- **Contract.** Only on stores opened **versioned** (SurrealKV or RocksDB); mem dropped versioning in 3.3;
  plain stores refuse (`SB002`, `SB040`); TiKV refuses (`SB111`). Version retention default unlimited when
  enabled (LCT). Disabled today (`versioned=false`).
- **Limits.** Changes write/storage/backup economics; storage-level history, not a scientific record
  (I4, I7).
- **Serves.** Forensic "what did the store say at T"; cheap audit without application history tables.

### F12. Binary payloads at scale

- **Does.** `bytes` values as ordinary record fields; experimental files/buckets (`DEFINE BUCKET`,
  `f"b:/p".put/get`, needs `--allow-experimental=files`, `SB132`, `SB138`); gRPC/embedded streaming.
- **Limits.** One record is one KV value: large blobs inflate WAL/compaction and transaction size;
  the ws frame cap (`SURREAL_WEBSOCKET_MAX_MESSAGE_SIZE`, set by the supervisor) bounds one message, not a
  whole response (P28). Opaque bytes are not queryable. Buckets move backup/reclaim outside the record
  substrate (I4).
- **Serves.** Result blocks, artifact bodies, payload chunks.

### F13. Import, export and backup

- **Does.** SDK/core `export` with config (users, accesses, params, functions, analyzers, `versions`,
  `tables`, `records`; `vendor/surrealdb/src/method/export.rs`), streamed through a bounded channel;
  `import` / `import_stream`.
- **Contract.** Logical SurrealQL dump; **import inserts, not upserts**: colliding records fail with
  `AlreadyExists`, definitions replace (`SB118`). Directory copies are safe only with the store closed (P28).
- **Limits.** No consistent online physical snapshot API found in the SDK surface read (search scope:
  `vendor/surrealdb/src/method/`, core `kvs/ds.rs` export/import; not exhaustive).
- **Serves.** Backup/restore, fixture seeding, migration.

### F14. Physical maintenance and reclaim

- **Does.** Tombstone reaper; `REMOVE DATABASE` enqueues durable deferred cleanup in the same
  transaction; `ALTER TABLE … COMPACT`; index compaction; changefeed GC (LCT; `core/src/kvs/tasks/`).
- **Contract.** Logical removal acknowledgement ≠ physical key reclaim ≠ returned disk space (LCT, I4).
  **No record TTL and no user scheduler** (LCT search: official expiration docs, 3.3.0 DEFINE parser).
- **Serves.** Bulk disposal of disposable scopes (whole database), space recovery after retention.

### F15. Planner and diagnostics

- **Does.** `EXPLAIN` / `EXPLAIN ANALYZE [FORMAT JSON]` (ANALYZE executes), `INFO FOR …`, `WITH INDEX` /
  `NOINDEX` (`SB128`), `TEMPFILES` external sort, slow-query log (`core/src/kvs/slowlog.rs`), server
  metrics/OTel options.
- **Limits.** EXPLAIN shape is informational and unstable (P28); not a receipt format.
- **Serves.** Checking that lifecycle scans (retention, readiness) use indexes; diagnosing contention.

### F16. SurrealKit 1.0.0-beta.6 (standalone CLI; `.config/surrealkit.toml`)

Reused from LCT without change: desired-state sync (prunes managed objects), phased rollouts with frozen
SHA-checked SQL, declarative test suites with isolated ns/db, cooperative locks (TTL 900 s, refresh 300 s;
lost renewal only warns — does not fence ongoing SQL); cleanup discards statement errors. Targets SDK 3.3.0,
Rust 1.95. Schema/migration bookkeeping and test isolation; **not** an operational lifecycle owner.
Only `surrealkit-1.0.0-beta.1` source is in the cargo registry; beta.6 claims rest on LCT's pinned GitHub reads.

### F17. tokio 1.53.2 / tokio-util 0.7.19 structured concurrency

Pinned features: tokio `rt-multi-thread, sync, macros, time, process, io-util`; tokio-util `rt` only
(`Cargo.toml`). Surface (source: `tokio-util-0.7.19/src/{sync,task}`, `tokio-1.53.2/src/{sync,task}`;
Context7 `/websites/rs_tokio-util`):

| Primitive | Contract | Limits |
|---|---|---|
| `CancellationToken` | `cancel`, `cancelled`/`cancelled_owned`, `child_token` (parent cancels children, not vice versa), `drop_guard`/`drop_guard_ref`, `run_until_cancelled[_owned]` | cooperative: signals, never preempts; no reason payload |
| `TaskTracker` | `spawn`/`spawn_blocking`/`spawn_on`, `track_future`, `token()` (counts arbitrary activities, not only tasks), `close`/`reopen`, `wait` = closed **and** empty; guarantees tracked futures' destructors finished; `wait` cancel-safe | does not cancel; does not bound concurrency; no per-task result |
| `JoinSet` | owns tasks, aborts all on drop, `shutdown` (abort + await), `join_next[_with_id]`, `abort_all`, `detach_all` | abort only at `.await` points; `spawn_blocking` tasks cannot be aborted once running |
| `JoinQueue`, `AbortOnDropHandle` | FIFO completion; abort on drop | same abort limits |
| `JoinMap` | keyed tasks | needs `join-map` feature (not enabled) |
| `Semaphore` (owned) | FIFO-fair; `acquire_many_owned`, `try_acquire_owned`, `close` (wakes waiters with error), `add_permits`, `forget_permits` | process-local; no cross-process capacity; no priorities |
| `task_local!`, `Notify`, `watch`, `time::timeout`, `select!` | scoped context, wake-ups, latest-value broadcast, deadlines | `timeout` drops the future (cancellation by drop) |
| `Runtime::shutdown_timeout`/drop | stops scheduling; blocking threads may outlive | native/FFI threads are never interrupted |

Serves: in-process admission (permits), structured cancellation trees, drain (`close`+`wait`),
teardown ordering, deadlines. Does **not** serve: durability, cross-process exclusion, cancellation of
native compute or `spawn_blocking`, unknown-outcome settlement.

### F18. Filesystem primitives for local durability

- `rename` within one filesystem is atomic for the name; durability needs `fsync(file)` then
  `fsync(parent dir)`. `flock`/`fcntl` advisory locks are released automatically when the holder's
  descriptors close, including on crash — a crash-safe single-owner marker but not a fence for writes
  already in flight, and cooperative only. Rust std `File::lock`/`try_lock` (stable since 1.89; pinned
  nightly has it). These give crash-atomic replacement of small manifests and single-owner exclusion of
  a directory; they give no multi-record transactions or queries.

---

## Part 2. Current use, briefly (pointers, no judgment)

| Family | Where today |
|---|---|
| F1 transactions, `FOR UPDATE` | 84 `FOR UPDATE` in `crates/pse-operations/src/generated/surreal.surql` (2,428 lines); single-RPC `BEGIN…COMMIT`; per-problem guard keys touched by `fn::pse_execution_v1::touch` (`surreal.surql:709`, 14 `retention:` sites); conflict retry and unknown-outcome settlement in `canonical.rs`, `canonical_transport.rs`, `canonical_execution.rs`; tests `canonical_conflict_tests.rs` |
| F1 durability | `sync=every`, `versioned=false` (`scripts/surreal_server.py::5418`); RocksDB tuning via `SURREAL_ROCKSDB_*` (`server_environment`, ~line 5085) |
| F2 schema | 57 `SCHEMAFULL` tables, 442 fields, 139 indexes (77 UNIQUE), 102 `ASSERT`, 246 `THROW`; no `STRICT`, `READONLY`, `VALUE`, `DEFAULT`, `COMPUTED`, `DEFINE PARAM`. Declared in `crates/pse-schema/src/catalog/substrate.rs`, lowered by `crates/pse-codegen/src/codegen/surreal.rs` |
| F3 references | 6 `TYPE RELATION`, 5 `ENFORCED`; **no `REFERENCE`**, no LIGHTWEIGHT/INLINE (I1, I5) |
| F4/F5 events, views | none (`DEFINE EVENT` 0, `AS SELECT` 0) — I2 |
| F6 live/changefeeds | none; polling + persisted readiness (`canonical_study_readiness*` tables, `canonical_studies.rs`) — I2 |
| F7 sequences | none; generations are decimal counters in guard records |
| F8 functions | 94 `DEFINE FUNCTION` in namespaces `pse_execution_v1` (32), `pse_history_v1` (20), `pse_study_v1` (18), `pse_artifact_v1` (13), `pse_retention_v1` (6), `pse_analysis_v1` (5) |
| F9 access | owner + read-only selection credentials (`credentials.json`); VIEWER MCP (I6); no `DEFINE ACCESS/USER/PERMISSIONS` in generated DDL |
| F10 deployment | remote ws only, one supervised RocksDB server per state dir (`scripts/surreal_server.py`, 7,513 lines; systemd user unit, cgroup/PID identity); vendored patch `vendor/surrealdb/` (~2.8k changed lines, EFF L2-04); `canonical_lifetime.rs` (Open/Closing/Closed, activities, `task_local CONTINUATION`) |
| F11 history | engine VERSION disabled; explicit history tables (`canonical_history_*`, `canonical_history.rs`) |
| F12 payloads | bytes in bounded blocks (`canonical_payload_blocks`, `canonical_result_blocks`, `canonical_artifact_blocks`); ≤512 KiB/≤1024-row result blocks (EFF) |
| F13 backup | `surreal_server.py` quiesce/stop/backup/restore sections (`docs/dev/surreal-substrate.md` §"Quiesce, stop and backup", §"Restore and validation") |
| F14 reclaim | application retention/reclaim: `canonical_retention.rs`, `canonical_result_retention.rs`, `canonical_maintenance.rs`, `canonical_reclaimed_ranges` table; no COMPACT routine (I4) |
| F15 diagnostics | INFO/EXPLAIN only in diagnosis guidance (I5) |
| F16 SurrealKit | `.config/surrealkit.toml` (tooling-only, beta.6) |
| F17 tokio | `CancellationToken` in 135 files (pse-engine 53, pse-runtime 29, pse-admission 18, pse-relations 11 …); `JoinSet` 10 files (pse-operations 7); `Semaphore` 17 (pse-engine 9, pse-runtime 6); `TaskTracker` only `pse-operations/src/canonical_results.rs` (`releases`, `TaskTrackerToken`); `task_local!` 11 (pse-runtime 6, pse-operations 4); `spawn_blocking` joins native threads (`pse-runtime/src/math/{staged,jobs}.rs`); no `DropGuard`, `AbortOnDropHandle`, `JoinMap` |
| F18 fs | Python supervisor: `os.fsync`, `fcntl.flock` (`scripts/surreal_server.py:150,222`, `scripts/host_admission.py:449`, `scripts/native_operation.py:462`); Rust: `sync_all`+`rename` in `pse-backend-native/build_receipt.rs`, `pse-runtime/src/workflow/connected_results.rs:197` |

Counts are `grep -c` on the generated SurrealQL and `rg -l` over `crates/` excluding generated paths;
they locate, they do not measure.

---

## Part 3. Responsibility questions

### 3.1 What SurrealDB itself could enforce (and under which shape)

Observed library capabilities mapped to obligation kinds. "Shape" = whether the guarantee holds
embedded (E), remote server (R), or both.

| Obligation | Library mechanism | Shape | Strength |
|---|---|---|---|
| Atomic multi-record transition | `BEGIN…COMMIT`, `DEFINE FUNCTION` in caller tx | E, R | full (F1) |
| Uniqueness / single winner | UNIQUE index; unique (owner, generation) keys | E, R | full; violation kind `Internal` |
| Compare-and-set on named records | `FOR UPDATE` on record ids + retry | E, R | optimistic; retry loop is the caller's |
| Monotone fencing token | `DEFINE SEQUENCE` | E, R | unique/increasing, gaps |
| Shape validity | SCHEMAFULL, STRICT, TYPE, ASSERT, READONLY | E, R | per record |
| Reference integrity / refuse delete while referenced | `REFERENCE ON DELETE REJECT`, `ENFORCED` relations | E, R | typed record values only |
| Dependent cleanup (cascade) | `ON DELETE CASCADE/UNSET/THEN`, node→edge delete | E, R | same transaction |
| Illegal-transition refusal | sync `DEFINE EVENT … THROW`, field `ASSERT` with `$before` | E, R | same transaction |
| Consistent derived counts (e.g. obligations outstanding) | `AS SELECT … GROUP BY` views | E, R | same transaction; hot key |
| Readiness notification | LIVE SELECT (post-commit) | E, ws | lossy on disconnect |
| Durable change log / incremental read | CHANGEFEED + versionstamp cursor | E, R | bounded retention; consumer cursor |
| Deferred follow-up effect | ASYNC event | E, R (maintenance tasks) | at-least-once attempt, finite retry |
| History / audit | VERSION (versioned store), changefeeds, events writing audit rows | E, R | storage-level |
| Crash atomicity | WAL + `sync=every` | E, R | per commit |
| Disposal of a whole scope | `REMOVE DATABASE`/`NAMESPACE` | E, R | logical, deferred physical |
| Least privilege for readers | users/roles, PERMISSIONS | R; E only with `Config::user` | silent empty results |
| In-process cancellation of an in-flight query | drop the future (`AbandonOnCancel`) | **E only** | upstream contract |
| Single-owner exclusion of the store | directory lock (`SB003`) | **E only** | one process |

### 3.2 What it cannot enforce, and why

- **Predicate and phantom protection.** SI + record-id `FOR UPDATE` only; a range or "no row exists"
  premise needs a stable named guard key every invalidating writer touches (P28, EFF L2-02). Library
  cannot see the premise.
- **Time-, lease- and liveness-based protection.** No TTL, no scheduler, no lease expiry semantics;
  references express existence, not "protected until reader X releases" (LCT, F14).
- **Process and physical facts.** Whether a native worker, OS process, socket or thread has actually
  stopped; whether disk space returned; whether a server-side statement aborted after the client
  dropped (remote). The store records claims; it cannot observe the host (§20.6, `PATCH.md`).
- **Unknown-outcome settlement over a network.** A lost ack leaves an unknown commit; only
  application identity (idempotent keys, receipts) can settle it. Embedded shape removes the network
  part of this but not crash-during-commit.
- **Retry policy.** Conflicts are surfaced, not retried; backend wording differs (F1).
- **Exactly-once effects outside the store.** ASYNC events and changefeeds are at-least-once
  attempts; external effects (native solves, files) need idempotence owned elsewhere.
- **Scientific validity, provenance meaning, compatibility.** Constraints check shape and existence,
  not that a result belongs to a compatible revision or that a seed is eligible (§20.6, ADR-0173).
- **Untyped dependencies.** String identities, bytes payloads and ids inside Arrow blocks are invisible
  to references and cascades (LCT). Today's schema keys many relations by strings (I1), so native
  references would need typed `record<…>` fields first.
- **Cross-process capacity accounting.** Nothing in the store charges CPU/memory; it can hold a ledger
  row, not admit work.

### 3.3 What tokio / tokio-util already provide for admission, cancellation, drain, teardown

- **Admission.** `Semaphore` (owned permits, `acquire_many`, `close` to stop new admission and wake
  waiters with error) — process-local, fair, cancel-safe on drop of the acquire future.
- **Cancellation.** `CancellationToken` trees (`child_token`), `DropGuard` (cancel-on-scope-exit),
  `run_until_cancelled`; `JoinSet`/`AbortOnDropHandle` abort at await points; embedded SurrealDB extends
  drop-cancellation into the store transaction (F10).
- **Drain.** `TaskTracker::close()+wait()` = "no new tracked activity and all tracked futures,
  including destructors, finished"; `TaskTracker::token()` counts non-task activities (e.g. an open
  reader or a borrowed connection) in the same drain. This is the primitive shape of §20.6's
  "Open → Closing → drained" for in-process activities.
- **Teardown ordering.** Structured ownership (tracker per layer, child tokens) plus
  `Datastore::shutdown` ordering when embedded.
- **Not provided.** Durability; cross-process exclusion; interrupting native/FFI threads or running
  `spawn_blocking`; settlement of remote writes; capacity across processes (R5 territory). Filesystem
  `flock` (F18) covers cross-process single-owner exclusion with crash release; systemd/cgroups cover
  process-tree kill and drain (outside these libraries).

---

## Part 4. Library-shaped allocations

Each sketch is *Proposed*; benefits are not measured. "Bespoke machinery made unnecessary" is at
responsibility level.

### A. "SurrealDB-native lifecycle" — the store's own constraints, references, events and transactions own the invariants

**Shape.** Store remains the coordination point for all lifecycle state. Two deployment variants:
A-R (remote server, as today) and A-E (embedded in one owning process; other processes reach it only
through that process).

**Library discharges.**
- Typed `record<…>` keys replace string identities so that `REFERENCE ON DELETE REJECT` protects
  artifacts/results while any referencing row (obligation, protection, read, membership) exists, and
  `CASCADE`/`UNSET` removes dependents in the deleting transaction.
- `ENFORCED` relations for every endpoint known at insertion; UNIQUE composite indexes for single
  winners (claims, seals, receipts).
- `DEFINE SEQUENCE` for generations/fencing tokens instead of counter records.
- Sync `DEFINE EVENT … THROW` / `ASSERT $before…` for legal state transitions (run/attempt/study state
  machines) so any writer — function, MCP, tooling — is refused identically.
- `AS SELECT … GROUP BY owner` views for outstanding-obligation counts consumed by retention.
- CHANGEFEED on readiness/result tables as the incremental read path, LIVE SELECT as a wake-up hint
  over the changefeed cursor.
- VERSION on a versioned store for forensic history; `REMOVE DATABASE` for disposable scopes.
- `DEFINE ACCESS`/roles for writer vs reader separation.

**Remains the project's.**
- The premise guards for negative/range conditions (named guard keys, partitioned to avoid hot keys).
- Conflict retry loop and, in A-R, unknown-outcome settlement by original identity, request deadlines
  and transport drain (the vendored patch or an upstream equivalent).
- Leases/heartbeats and expiry decisions (no TTL/scheduler), reclaim eligibility, physical reclaim.
- Process lifecycle: worker supervision, native join, host capacity.
- Scientific validity and provenance meaning; the generator that emits the DDL (one authority,
  `pse-schema` → `pse-codegen`).
- Changefeed cursors and gap recovery; idempotence of ASYNC/feed consumers.

**Lost or weakened.**
- Hot keys: grouped views and sync events add every triggering write to their conflict set; contention
  moves from explicit guards into implicit DDL behaviour (interpretation).
- References see only typed ids: payload-embedded dependencies stay application-owned, so two
  integrity mechanisms coexist unless every dependency becomes a typed field.
- Debuggability shifts into SurrealQL triggers whose errors surface as `Internal`/`THROW` strings.
- A-E: the store becomes reachable only through its owning process; managed native workers in other
  processes need that process as their intermediary (IPC), or must run in-process. Gains in-process
  drop-cancellation and removes network unknown-outcome; loses independent server restart and
  multi-client direct access (`SB003`). Server-only facilities (HTTP `/health`, MCP route, Studio) need
  the owning process to expose them.
- Upgrade exposure: more semantics depend on SurrealDB behaviour that changes between releases
  (e.g. ASYNC retry budget 3.3.1; `surrealdb-core` not SemVer-stable for A-E).

**Integration, lifecycle, testing, upgrade cost (qualitative).** Schema generator change (typed
references, sequences, events, views) and a coordinated v-next interpretation; RocksDB-backed tests
for both insertion/deletion orders, concurrent publish/reclaim races and permissions (I1 trigger
conditions); A-E adds a C++ RocksDB build or SurrealKV choice, the core in the product build and a
process-topology change for workers. Each SurrealDB bump re-qualifies trigger/reference semantics.

**Bespoke responsibilities it could make unnecessary.** Application-maintained backlink/obligation
counters and explicit "may delete?" checks for typed dependencies; per-transition guard functions whose
only job is state-machine legality; generation counters as decimal fields; readiness polling loops
(as primary discovery, retaining periodic recovery); explicit audit/history tables where storage-level
history suffices (only if VERSION is enabled). In A-E additionally: the client-side bounded admission,
replay and physical-drain patch for the network transport, and unknown-commit settlement for the
network hop (crash-during-commit remains).

### B. "Minimal durable core" — only what must be durable is in the store; everything else is process-local

**Shape.** The store holds immutable published facts and the few records whose loss would be
observable after process loss: authored revisions, run/attempt identity and terminal outcome, sealed
result sets and their blocks, study definitions and per-point terminal state, retained artifacts and
their protections. Admission, activity tracking, connection lifetimes, readiness, progress, leases
for in-process work, caches and drain live in the process under tokio structure. Variants: B-R (remote
server, workers write terminal facts directly) and B-E (embedded store in a single long-lived owner;
the Python session or a daemon).

**Library discharges.**
- SurrealDB: atomic append of immutable facts (one transaction per publication), UNIQUE identity
  (idempotent re-publication = settlement by identity), `ENFORCED`/`REFERENCE REJECT` for the few
  retained-artifact protections, export/import for backup.
- tokio/tokio-util: `TaskTracker` + `token()` per lifetime layer (caller, activity, physical
  connection — §20.6's three lifetimes map onto three trackers), `CancellationToken` trees with
  `DropGuard` for caller release, owned `Semaphore` for in-process admission, `JoinSet` for workers.
- Filesystem: `flock` for single-owner exclusion of a state directory (crash-released), fsync+rename
  for small manifests.

**Remains the project's.**
- Defining which facts are durable and their identities (the minimal schema itself).
- Unknown-outcome settlement for B-R (smaller surface: immutable appends with natural keys).
- Recovery on restart: scan durable non-terminal records (runs/points without a terminal fact) and
  decide retry/abandon — replaces live-authority fences with restart reconciliation.
- Retention eligibility and reclaim of immutable facts.
- Native-thread non-cancellability and process supervision (unchanged).
- Cross-process coordination only where a product feature needs it (durable studies with managed
  workers across process loss); claim/fence records survive for exactly those.

**Lost or weakened.**
- Durable observation of in-flight state (progress, heartbeats, live activity) after a crash; a second
  process cannot see the first's in-process activities — multi-client concurrent mutation of one
  problem would need either the store-backed fences of A or a single owner (B-E).
- Fine-grained crash continuation: interrupted work restarts from the last durable terminal fact
  rather than resuming a fenced claim (§20.6 "explicit recovery transfer" semantics narrow).
- In B-E, multi-process direct store access (as in A-E).
- Whether this loss matters depends on which product features consume cross-process recovery —
  W-B (a) supplies that inventory; Part B maps it.

**Integration, lifecycle, testing, upgrade cost (qualitative).** Large schema reduction and a new
interpretation; lifecycle tests move from RocksDB race fixtures to in-process structured-concurrency
tests (tokio test runtime, deterministic `TaskTracker`/token drains) plus a small set of restart
reconciliation tests against the store; fewer SurrealDB semantics to re-qualify per upgrade (append,
unique, read). B-E adds the embedded build cost of A-E.

**Bespoke responsibilities it could make unnecessary.** Durable records for transient coordination
(activity/heartbeat/generation touches on every transition, readiness cursors, maintenance
activations, selection receipts used only for live-authority fencing); store-side guard-key protocols
whose premise concerns in-process work; the hand-rolled Open/Closing/Closed activity accounting in
favour of tracker/token composition; in B-E, the WS transport patch and its replay/drain machinery.

### Hybrid note (not a third sketch)

The families compose: B's durable core can use A's native constraints for the few invariants it keeps
(UNIQUE identity, REFERENCE REJECT for retained artifacts, sequences for fences), while tokio owns
everything process-local. The deployment axis (embedded vs remote) is independent of the A/B axis and
is decided by whether more than one OS process must write the store directly (`SB003`).

---

## Material uncertainties and limits

- Probe evidence is the skill's 2026-10-05 runs (mostly mem/embedded; remote parity on a memory-backed
  server). RocksDB-server behaviour under the project's profile, crash/power-loss durability and
  contention throughput are **not** established here.
- 3.3.x patch releases after 3.3.0 (a 3.3.3 line exists in the registry) were not inspected for
  changes; only the 3.3.1 ASYNC RETRY note (via LCT) is known.
- SurrealKit beta.6 source not present locally; claims are LCT's Interface-checked GitHub reads.
- Embedded-engine cancellation/shutdown contracts are read from `surrealdb-engine-local-3.3.0` source
  (Interface-checked), not executed.
- Absence claims (no TTL/scheduler; no online snapshot API) rest on the stated search scopes only.
- Grouped-view and sync-event hot-key contention is interpretation from transaction placement, not
  measured.

Files written: this file only.
