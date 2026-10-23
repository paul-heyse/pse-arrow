# SurrealMX 1.0 — Comprehensive Rust Library Overview

Version: 1.0.0 · Research date: October 22, 2026 · Language: Rust · License: Apache-2.0

## 1. Executive orientation

SurrealMX is an embedded, memory-first, transactional key-value storage engine written in Rust, developed by SurrealDB. Its architecture combines a concurrent adaptive radix tree, multi-version concurrency control (MVCC), lock-free reads, concurrent transaction processing, and optional crash-recoverable persistence.

SurrealMX is designed for workloads where extremely low latency, high concurrency, efficient ordered access, and transactional consistency are important, and where the active dataset can remain in memory.

Version 1.0.0 was released on October 12, 2026. This is a consequential stabilization release, not merely a version-number change. It introduces or strengthens serializability, consistent concurrent snapshots, crash-safe transaction logging, bounded resource consumption, fail-stop error handling, and several substantial concurrency and indexing optimizations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



Primary sources:

- Release: [https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0?utm_source=chatgpt.com)
- Repository: [https://github.com/surrealdb/surrealmx](https://github.com/surrealdb/surrealmx?utm_source=chatgpt.com)
- Rust crate: [https://crates.io/crates/surrealmx](https://crates.io/crates/surrealmx?utm_source=chatgpt.com)
- Tagged v1.0.0 source: [https://github.com/surrealdb/surrealmx/tree/v1.0.0](https://github.com/surrealdb/surrealmx/tree/v1.0.0?utm_source=chatgpt.com)

### 1.1 Capability map

| Capability family             | Principal mechanisms                                                                                                            |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Storage and indexing          | In-memory ordered key-value storage; concurrent adaptive radix tree; byte-oriented keys and values                              |
| Transaction processing        | MVCC, snapshot isolation, serializable transactions, optimistic conflict detection                                              |
| Concurrency                   | Lock-free version-chain reads, concurrent readers and writers, commit pipeline and circular commit ring                         |
| Data access                   | Point reads/writes, deletion, ordered range scans, bidirectional cursors, iteration and counting                                |
| Transaction composition       | Savepoints, partial rollback, nested scopes, commit/cancel semantics                                                            |
| Persistence                   | Append-only transaction log, consistent-cut snapshots, crash recovery, configurable fsync and compression                       |
| Resource management           | Transaction-age limits, memory budgets, asynchronous queue bounds, garbage collection                                           |
| Observability and reliability | Database statistics, active-reader tracking, pinned-version diagnostics, poison-state errors, deterministic concurrency testing |
| Integration                   | Direct Rust embedding, SurrealDB storage backend, WebAssembly and other platform targets                                        |

### 1.2 The important architectural distinction

SurrealMX is not a standalone document, graph, relational, vector-search, or SQL query engine. Those capabilities come from SurrealDB when SurrealMX is used underneath it.

Application / SurrealDB SDK

Application logic, typed records, API interactions

SurrealDB query and data-model layer

SurrealQL, documents, graphs, indexes, permissions, query execution

SurrealMX 1.0

Ordered KV operations · MVCC · Transactions · Persistence · GC

Alternatively, Rust applications can access SurrealMX directly without the SurrealDB query layer.

This separation produces two distinct use cases:

1. Direct embedded storage: A Rust application uses SurrealMX as a transactional, ordered key-value database, controlling its own encoding, key schema and higher-level data model.
2. SurrealDB storage backend: SurrealDB handles documents, relationships, indexes, query planning, authentication and permissions while SurrealMX handles the transactional storage primitives.

SurrealDB's architectural documentation explicitly describes documents, graph edges, and index entries as data maintained on its underlying transactional key-value engine. Consequently, its higher-level data models can benefit from SurrealMX's storage characteristics without SurrealMX itself implementing those models.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Architecture reference: [https://surrealdb.com/docs/learn/data-models/architecture](https://surrealdb.com/docs/learn/data-models/architecture?utm_source=chatgpt.com)

## 2. Storage architecture and data representation

### 2.1 Concurrent adaptive radix tree (ART)

SurrealMX 1.0 uses the `artmap` Rust library to implement its primary ordered key index, replacing the concurrent skiplist used in previous releases.

An adaptive radix tree is a compressed trie organized around key bytes. Rather than comparing complete keys at every balanced-tree node, its structure exploits common key prefixes and adapts node representations to their child cardinality.

This gives SurrealMX three complementary properties:

- Fast point lookup: Traverses a key's byte representation through a compact prefix-oriented index.
- Ordered traversal: Provides lexicographically ordered access suitable for bounded key ranges, prefixes and cursor positioning.
- Concurrent mutation: The `artmap` implementation permits concurrent operations without introducing a global datastore read/write lock.

The ART is the key index, not the entire storage representation. Each indexed key refers to a versioned value structure managed by SurrealMX's MVCC machinery.

Consequently, an ordered lookup may involve both locating the key in the ART and resolving which version of its value is visible to the reader. The indexed key may also have a deletion tombstone rather than a currently visible value.

Important v1.0 distinction: Documentation referring to SurrealMX's primary B+ tree or skiplist is outdated for the tagged 1.0 implementation.

Sources:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/inner.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/inner.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/versions.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/versions.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/Cargo.toml](https://github.com/surrealdb/surrealmx/blob/v1.0.0/Cargo.toml?utm_source=chatgpt.com)

### 2.2 MVCC value chains

SurrealMX maintains multiple committed versions of a key when necessary to preserve the views of concurrent transactions.

Conceptually:

ART key: `order:42`

Version chain

v103

Latest committed

v97

Older visible

v89

Reclaimable

Conceptual illustration of version visibility, not the exact internal memory layout.

A transaction receives a logical read version at creation. Reads resolve the newest committed value visible at that version, including the transaction's own uncommitted changes where applicable.

This yields:

- Consistent multi-key snapshots without copying the entire database.
- Readers that continue using an older view while concurrent writers commit newer versions.
- Transaction-local write sets that can be validated before publication.
- Version reclamation as soon as no active reader requires obsolete values.

SurrealMX uses logical versions primarily for concurrency isolation. Its 1.0 public API does not expose general arbitrary historical-version querying. An application needing historical state across completed transactions must distinguish that requirement from ordinary MVCC snapshot consistency.

The same distinction applies when using SurrealMX through SurrealDB: the in-memory backend does not support SurrealDB's versioned queries from SurrealDB 3.3.0 onward.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 2.3 Byte-oriented keys and values

The fundamental storage abstraction is an ordered mapping from byte sequences to byte sequences.

Relevant types include:

- `IntoBytes`: Optimized conversion abstraction for supported borrowed and owned byte representations.
- `ByteSlice`: Shared, slice-oriented byte storage used throughout the key/value APIs.
- `Bytes`: Re-exported from the Rust `bytes` crate.

The implementation accepts forms including byte slices, arrays, vectors, strings, and shared byte buffers. Its conversion trait separates borrowing an existing byte representation through `as_slice()` from producing an owned `ByteSlice` through `into_bytes()`.

This distinction matters particularly for point reads and range operations: a lookup does not necessarily need to allocate another owned key.

SurrealMX does not impose an application-level record format. Values might hold serialized Rust structures, compact binary records, index entries, counters encoded as bytes, or opaque application state.

That flexibility also means it does not supply application-level schema validation, serialization evolution, secondary indexing, or referential integrity by itself.

### 2.4 Key schema as an architectural capability

The ordered nature of the key space makes composite and prefix-oriented keys especially useful.

For example, a conceptual key layout could be:

```
entity/<type>/<id>
entity-by-owner/<owner>/<type>/<id>
events/<entity-id>/<ordered-event-id>
```

Contiguous ranges can then represent logical subsets of a larger datastore.

The important mechanism is not the textual format, but the preservation of useful ordering relationships in the encoded bytes. Binary encodings can provide more compact and predictable layouts.

Applications must still define how components are delimited, how numeric values sort, how a prefix's upper bound is calculated, and how encoding versions evolve. SurrealMX offers the ordered access kernel, not a schema-definition language for those conventions.

## 3. Transactions, isolation and concurrency

This is SurrealMX's most architecturally significant capability family.

### 3.1 Transaction model

`Database::transaction(write: bool)` creates a transaction with a consistent snapshot.

- `transaction(false)` creates a read-only transaction.
- `transaction(true)` creates a writable transaction.
- `commit()` finalizes a transaction.
- `cancel()` discards pending modifications.
- Dropping a transaction releases its reader registration and returns reusable transaction state to the pool.

The implementation's default isolation mode is `SerializableSnapshotIsolation`, but read-only transactions do not generate modifications and therefore primarily exercise snapshot-read behavior.

Writable transactions also support an explicit downgrade using `with_snapshot_isolation()`.

### 3.2 Isolation semantics

| Property                                     | Snapshot isolation          | Serializable snapshot isolation                      |
| -------------------------------------------- | --------------------------- | ---------------------------------------------------- |
| Consistent initial snapshot                  | Yes                         | Yes                                                  |
| Read-your-own-writes                         | Yes                         | Yes                                                  |
| Write-write conflict detection               | Yes                         | Yes                                                  |
| Ordinary read-write dependency validation    | No                          | Yes                                                  |
| Range/phantom conflict protection for writes | Not equivalent to SSI       | Yes                                                  |
| Write-skew prevention                        | Not generally guaranteed    | Yes                                                  |
| Primary tradeoff                             | Reduced tracking and aborts | Stronger consistency with conflict-tracking overhead |

Under v1.0's default writable transaction mode, SurrealMX tracks dependencies that could invalidate a transaction's serializable execution.

This includes:

1. Write-write conflicts: Concurrent transactions attempting incompatible modifications to the same key.
2. Read-write conflicts: A transaction reads a value that another transaction changes before the first commits.
3. Range conflicts: A concurrent transaction inserts or changes keys within a range whose contents affected another transaction.

The third case is particularly important. Point-read conflict detection alone cannot fully protect predicates such as "all entries in this range satisfy a condition." Range-dependency tracking is what permits serializability across ordered scans.

The tagged v1.0 README describes update transactions as strictly serializable, and the release identifies write-skew rejection and serializable execution among its stabilization work.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



Tagged implementation:

[https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/tx.rs?utm_source=chatgpt.com)

### 3.3 Optimistic validation instead of global serialization

SurrealMX's concurrency strategy is optimistic.

Transactions accumulate reads and writes without holding a datastore-wide transaction lock. On commit, a writable transaction validates its dependencies against the relevant concurrent commits.

This allows independent operations to progress concurrently while rejecting combinations that would violate isolation guarantees.

The practical distinction is between conflict detection and conflict avoidance.

SurrealMX does not generally prevent another writer from attempting to modify data your transaction has read. It detects an incompatible execution and reports an error when necessary.

In other words, serializability does not mean that every transaction will successfully commit.

The principal retryable conflict errors are:

- `Error::KeyReadConflict`
- `Error::KeyWriteConflict`

A retry should generally restart the entire logical transaction against a fresh snapshot, rather than repeat the failed commit against stale dependencies.

### 3.4 Circular commit ring and commit pipeline

Version 1.0 uses a circular commit ring to coordinate concurrent transactions.

The overall mechanism separates work that traditional transaction managers might serialize under a larger critical section:

- A transaction publishes its commit intent.
- Conflicts are checked against relevant concurrent commits.
- A successful transaction obtains an ordered commit version.
- Modifications become visible through the merge/publication machinery.
- Completed commit history is eventually retired when older transactions no longer need it.

The implementation also uses a distinct merge queue and merge ring.

These structures matter because a commit may be logically visible before every value has finished moving into its final ART-associated version chain. Readers must therefore account for committed changes that remain in the merge machinery.

A useful way to understand this is that commit visibility and physical incorporation into the primary index are coordinated but distinct steps.

The pipeline supports concurrent progress without letting readers observe half-committed transactions.

Other internal mechanisms include bloom filters for accelerated conflict-set checks, logical version clocks, sharded reader registration, epoch-protected memory access, and per-key version synchronization.

These are complementary mechanisms, not separate user-facing storage modes.

Implementation references:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/ring.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/merge_ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/merge_ring.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/queue.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/queue.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/bloom.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/bloom.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/sync.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/sync.rs?utm_source=chatgpt.com)

### 3.5 Read-for-update

`Transaction::get_for_update()` registers a key as requiring commit-time validation.

Unlike a conventional blocking row lock, this should be understood as an optimistic protection mechanism: it tracks a key whose value must not be invalidated by an intervening incompatible commit.

An important distinction is that locked-key validation remains relevant even if the transaction eventually performs no writes, and remains active when intervening writes are undone through savepoints.

This supports read/validate/update operations without introducing a separate application-level lock manager for every key.

## 4. Data operations and ordered access

### 4.1 Direct operations versus transaction-scoped operations

SurrealMX offers a surprisingly useful distinction between operations performed directly on `Database` and those performed through a `Transaction`.

| Operation                     | API                                                | Consequential behavior                                          |
| ----------------------------- | -------------------------------------------------- | --------------------------------------------------------------- |
| Direct point lookup           | `db.get`, `db.exists`                              | Linearizable individual lookup without allocating a transaction |
| Direct borrowed lookup        | `db.with_value`                                    | Inspect value bytes in a closure without cloning the value      |
| Direct blind write            | `db.set`, `db.del`                                 | Auto-commit with automatic retries for write-write conflicts    |
| Direct conditional write      | `db.put`, `db.putc`, `db.delc`                     | Auto-committed insert or conditional mutation                   |
| Transactional point reads     | `tx.get`, `tx.exists`, `tx.getm`                   | Consistent multi-operation snapshot and dependency tracking     |
| Transactional writes          | `tx.set`, `tx.put`, `tx.putc`, `tx.del`, `tx.delc` | Compose multiple modifications atomically                       |
| Transactional read-for-update | `tx.get_for_update`                                | Explicit conflicting-key validation                             |

`set` overwrites an existing key; `put` requires it to be absent. `putc` and `delc` compare the observed value with an expected value, including an expected missing value represented by `None`.

The convenience methods do not all have identical retry policies. In particular, direct `set` and `del` automatically retry write-write conflicts because they are blind writes. A multi-step transaction may require the caller to retry the complete operation.

These distinctions are useful for separating high-throughput independent operations from transactions that enforce cross-key invariants.

### 4.2 Range scans, counting and pagination

SurrealMX supports lexicographically ordered range access using half-open key ranges, with APIs including:

- `keys`, `keys_reverse`
- `scan`, `scan_reverse`
- `total`
- `scan_with`, `scan_for_each`, `keys_for_each`
- `scan_into`, `keys_into`
- `scan_iter`, `scan_iter_reverse`
- `keys_iter`, `keys_iter_reverse`

The collection-returning methods provide convenient materialized results. Iterator, closure, and caller-buffer methods avoid different categories of allocation overhead.

Range scans support `skip` and `limit`, but offset-style pagination still requires traversing skipped entries. This distinction becomes important for large ordered datasets.

The benchmark suite, for example, records substantially different throughput for a first-100-key scan versus a skip-5,000-then-return-100 scan. Those results are discussed later.

### 4.3 Allocation-aware scanning

Four important APIs deserve separate attention.

`scan_with` passes borrowed key/value data to a closure, avoiding materializing a vector of cloned entries.

`keys_for_each` follows the same approach when only keys are required.

`scan_into` and `keys_into` reuse caller-owned vector capacity, which can reduce repeated allocation costs across successive queries.

At the database level, these scans pin a consistent snapshot for the duration of the operation. The implementation can scan the index directly when safe, and otherwise falls back to a pooled read transaction while a concurrent merge is in progress or for certain reentrant scans.

These mechanisms are valuable when results are consumed immediately, reduced into counters or aggregates, filtered by application code, or written into another data structure.

Their benefits depend on avoiding equivalent copies and allocations elsewhere in the pipeline.

### 4.4 Bidirectional cursors

`Cursor` is a lower-level navigation abstraction over a transactional range.

Its key operations include:

- `seek_to_first()` and `seek_to_last()`
- `seek(key)` and `seek_for_prev(key)`
- `next()` and `prev()`
- `valid()`, `key()`, `value()` and `exists()`

Unlike a forward-only iterator, a cursor can reposition and reverse its direction without rebuilding an entire materialized result set.

Internally, cursor traversal merges the indexed data, relevant committed-but-not-yet-merged writes, and the current transaction's own write set.

This opens useful possibilities for ordered indexes, bounded navigation, neighboring-key queries and interactive traversal, although higher-level indexing semantics remain the application's responsibility.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/cursor.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/cursor.rs?utm_source=chatgpt.com)

## 5. Savepoints and transactional composition

Savepoints are a native capability, rather than something an application must emulate with independent temporary write sets.

The relevant methods are:

- `set_savepoint()`
- `rollback_to_savepoint()`
- `release_savepoint()`

Savepoints are stackable, enabling nested modification scopes within one transaction.

For example, a transaction can create a savepoint before attempting a group of related updates. If a condition fails, it can roll back only that group's writes and continue with other modifications.

Implementation details include:

- Savepoint creation records a position in an undo journal, making creation inexpensive.
- Rollback walks the journal backward to reconstruct the earlier write-set state.
- Releasing a nested savepoint retains its writes, but does not protect them from rollback of an enclosing savepoint.
- Savepoints are anonymous and use stack discipline.
- They operate on writable transactions, not independent committed subtransactions.

Crucially, rolling back writes does not remove the corresponding recorded reads and scanned ranges. Keeping those dependencies produces conservative conflict validation and prevents a rollback from inadvertently weakening serializability.

This makes savepoints suitable for structured partial rollback, but they are not a mechanism for discarding arbitrary conflict history.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/tx.rs?utm_source=chatgpt.com)

## 6. Persistence, durability and recovery

SurrealMX's storage hierarchy is memory-first with optional persistence, not a disk-backed tree with an in-memory cache.

The ART and active version chains reside in memory. Persistence maintains recoverable copies of committed data through an append-only log, snapshots, or both.

### 6.1 Persistence modes

`Database::new()` creates a purely in-memory database. Persistence is explicitly enabled through `Database::new_with_persistence()`.

The most important configuration dimensions are:

| Configuration      | Values in v1.0                                            | Implication                                                                            |
| ------------------ | --------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `AolMode`          | `Never`, `SynchronousOnCommit`, `AsynchronousAfterCommit` | Determines whether transaction changes are logged and whether commit waits for logging |
| `SnapshotMode`     | `Never`, `Interval(Duration)`                             | Determines whether recoverable point-in-time images are produced                       |
| `FsyncMode`        | `Never`, `EveryAppend`, `Interval(Duration)`              | Governs when logged data is forced toward durable storage                              |
| `CompressionMode`  | `None`, `Lz4`                                             | Controls snapshot compression                                                          |
| Paths              | Base path, optional AOL and snapshot paths                | Determines persistence-file placement                                                  |
| Async queue limits | Maximum batches and maximum bytes                         | Provides backpressure when asynchronous logging falls behind                           |

All persistence modes default to `Never`, and compression defaults to `None`.

A particularly useful distinction is between enabling persistence and enabling durability at a specific commit boundary. These are not equivalent.

| Storage arrangement             | Commit behavior                    | Recovery boundary                                                     |
| ------------------------------- | ---------------------------------- | --------------------------------------------------------------------- |
| In-memory only                  | Commit updates memory              | No recovery after process loss                                        |
| Snapshot only                   | Commit updates memory              | Last successfully completed snapshot                                  |
| Asynchronous AOL                | Commit may return before log write | Depends on which queued records reached persistent storage            |
| Synchronous AOL, no fsync       | Commit waits for log append        | Process-crash recovery possible; power-loss durability not guaranteed |
| Synchronous AOL, interval fsync | Commit waits for log append        | More recent commits can remain vulnerable to power loss               |
| Synchronous AOL, `EveryAppend`  | Commit waits for durable log flush | Strongest documented per-commit durability mode                       |

The exact durability guarantee must include both the append mode and the fsync policy. An asynchronous append remains asynchronous even if its background worker eventually performs an fsync.

Tagged references:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/persistence.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/compression.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/compression.rs?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md?utm_source=chatgpt.com)

### 6.2 Synchronous logging with group commit

An advanced v1.0 capability is its `GroupCommitter`.

When `AolMode::SynchronousOnCommit` is combined with `FsyncMode::EveryAppend`, concurrent commits can be accumulated into a group. A flusher appends their encoded transaction frames and synchronizes the log in a coordinated operation.

This differs materially from a naive implementation in which every concurrent transaction must independently acquire a file lock and issue its own filesystem synchronization operation.

Group commit can amortize sync overhead across transactions while retaining the synchronous durability contract for each participating commit.

The implementation uses synchronization slots, a shared pending queue, thread parking and wake-up, and a single active flusher to coordinate the batch.

The performance opportunity depends on the degree of concurrent write activity. Under a low-concurrency workload, little batching may be available.

### 6.3 Transaction-atomic append-only log

SurrealMX 1.0 introduces a framed transaction log.

Each frame contains a transaction's key-value modifications, together with a length and checksum. In the tagged implementation, frame headers contain a 32-bit payload length and a 32-bit checksum derived from XXH3.

This provides a useful transaction-level recovery property: a log record can be identified as a complete committed unit instead of treating each individual key modification as independently durable.

On recovery:

- Complete, valid frames are replayed.
- An incomplete trailing frame can be truncated.
- Corruption in the middle of the log is treated as an error instead of discarding later apparently valid transactions.
- The database's logical version clock is advanced beyond persisted versions to avoid reusing an earlier version identifier.

The framing also introduces a maximum encoded transaction-frame payload of approximately 4 GiB. A larger single transaction fails with `PersistenceError::AppendFailed` rather than being silently split into independently recoverable fragments.

The checksum detects accidental corruption; it is not a cryptographic authentication mechanism.

### 6.4 Consistent-cut snapshots

Snapshot generation is more sophisticated than simply enumerating the live ART while transactions continue to mutate it.

The implementation:

1. Records an appropriate append-only-log cutoff.
2. Pins a consistent MVCC snapshot version.
3. Streams the values visible at that version to a temporary snapshot file.
4. Finalizes and synchronizes the snapshot file.
5. Atomically replaces the previous snapshot.
6. Removes only the log prefix safely represented by the snapshot, retaining later log records.

This enables online snapshotting with concurrent writes while avoiding two important failure modes: inconsistent snapshots containing pieces of transactions from different points in time, and lost transactions committed during the snapshot operation.

The snapshot contains the latest relevant state at the pinned cut rather than an archive of every historical MVCC value. Restoring a snapshot does not provide arbitrary historical reads into the pre-snapshot database.

`Persistence::snapshot()` is also exposed for explicitly triggered snapshot creation.

### 6.5 Snapshot compression and recovery

The compression choices are `None` and `Lz4`. Snapshot recovery detects the compression format from the file header, so the reader can recognize a compressed snapshot without treating the runtime's current compression selection as a fixed decoding assumption.

Compression reduces snapshot storage and potentially I/O at the expense of CPU work. The actual result depends on the compressibility of encoded values and available storage bandwidth.

The implementation uses `bincode` for persisted serialization and `lz4` for compressed snapshots. These are internal persistence-format choices, not application-level schema-management tools.

### 6.6 Persistence compatibility and failure behavior

Version 1.0 can read earlier unframed AOL records, and writes newly framed transaction records using a format marker.

The compatibility is one-way: an older SurrealMX release cannot necessarily reopen a log after v1.0 has appended framed records. This makes rollback of the executable distinct from rollback of the persisted data format.

The other consequential distinction is orderly shutdown versus crash recovery. SurrealMX drains queued asynchronous persistence work during normal shutdown, but queued commits can still be lost during abrupt process termination or machine failure.

Failures during the persistence stage can also occur after a transaction's modifications have already entered the in-memory datastore. SurrealMX therefore reports `TxCommitNotPersisted` and poisons the database instead of pretending that the transaction cleanly rolled back.

That is an important boundary for failure handling: a failed persistence acknowledgment does not necessarily imply that the attempted write was never visible in memory.

## 7. Memory management, reclamation and bounded resources

### 7.1 Inline reclamation and garbage-candidate tracking

MVCC introduces a fundamental tension: old versions are needed for snapshot isolation, but retaining every version indefinitely would make memory growth unbounded.

SurrealMX 1.0 addresses this through a combination of inline version reclamation and targeted background garbage collection.

During a successful commit, the engine attempts to reclaim superseded values in the chains it has just modified.

If an older reader still requires a previous version, the affected key is registered as a garbage-collection candidate.

A subsequent background sweep visits only those candidate keys, rather than scanning every entry in the ART.

This is a consequential improvement over a full-database periodic sweep:

- Ordinary updates can reclaim obsolete memory immediately.
- Reclamation deferred by pinned readers is localized to the affected keys.
- The background cost can scale with outstanding version garbage rather than the entire key count.
- Delete tombstones can be collapsed and eventually removed from the index when safe.

SurrealMX still exposes both `run_gc_tracked()` and `run_gc()`.

The first performs targeted candidate reclamation; the second performs a full scan. `run_cleanup()` independently handles obsolete commit-queue entries.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/db.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/db.rs?utm_source=chatgpt.com)

### 7.2 Reader registration and version retention

Each active transaction registers a reader slot. Snapshot versions are pinned so that concurrent reclamation does not free data still visible to an active reader.

The registry also accounts for a transaction in the middle of acquiring its snapshot. In that situation, garbage collection conservatively defers reclamation rather than assuming a potentially unsafe minimum version.

The principal architectural consequence is that long-lived snapshots directly affect memory consumption.

A single transaction that remains open while many keys are repeatedly updated can keep previous versions alive even if it never reads those keys individually.

This means snapshot lifetime is a resource-management dimension, not merely an application-level transaction convention.

### 7.3 Transaction age and memory limits

`DatabaseOptions` adds two significant resource controls:

- `with_max_transaction_age(Duration)` limits how long a transaction may remain active before later operations fail with `Error::TxTooOld`.
- `with_memory_limit(usize)` prevents writes from proceeding when the engine's estimated accounted data size would exceed the configured limit.

The age check occurs when transaction operations are attempted; it is not a continuously running cancellation mechanism. The implementation unpins the timed-out transaction's snapshot so that reclamation can progress.

The memory limit should not be interpreted as a hard maximum resident-set-size limit. It relies on approximate accounting of the engine's stored data and pending write footprint, rather than all allocator overhead, thread stacks, persistence buffers and other process allocations.

The pre-commit estimate can also be conservative for overwrites, as it counts the incoming write footprint before evaluating the eventual memory reclaimed from previous versions.

### 7.4 Persistence queue backpressure

Asynchronous logging can otherwise create an expanding queue when commit production exceeds disk throughput.

SurrealMX offers two optional bounds:

- `with_max_async_queue_depth(usize)`
- `with_max_async_queue_bytes(usize)`

An attempted commit exceeding the configured capacity returns `Error::AsyncQueueFull`.

Importantly, the check occurs before the transaction is applied to the datastore. In the tagged implementation, `MemoryLimit` and `AsyncQueueFull` leave the transaction open so the caller can retry later or cancel it.

This differs from a read/write conflict, where the relevant transaction is ordinarily aborted and must be recreated.

The queue limits are useful for bounded-memory asynchronous persistence, but the limits default to disabled, so production backpressure is not automatic simply because asynchronous logging is selected.

### 7.5 Configuration profiles

Version 1.0 exposes several convenience profiles, implemented as `DatabaseOptions` modifiers.

| Profile                       | Main changes                                                              | Tradeoff                                                     |
| ----------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `with_high_performance()`     | Larger transaction pool, later tracked-GC sweep, adjusted cleanup cadence | More retained memory to reduce repeated allocation work      |
| `with_low_resource()`         | Smaller transaction pool, less-frequent background activity               | Lower ongoing resource use at the cost of slower reclamation |
| `with_reduced_memory()`       | Smaller transaction pool, more-frequent tracked-GC sweep                  | Faster reclamation after pinned readers depart               |
| `with_all_workers_disabled()` | Disables automatic GC and commit cleanup threads                          | Explicit lifecycle and maintenance obligations               |

Defaults in the tagged source are a transaction pool of 512, background GC every 10 seconds, commit-queue cleanup every 1 second, and a transaction-state reset threshold of 100.

These profiles affect resource-management behavior; they do not change the fundamental isolation model or convert SurrealMX into a disk-resident engine.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs?utm_source=chatgpt.com)

## 8. Reliability, observability and platform support

### 8.1 Database statistics

SurrealMX 1.0 exposes a structured `DatabaseStats` snapshot through `Database::stats()`.

The fields provide visibility into several otherwise difficult-to-observe parts of a concurrent MVCC engine.

| Field                   | What it reveals                                           |
| ----------------------- | --------------------------------------------------------- |
| `live_keys`             | Current live key population                               |
| `live_versions`         | Estimated number of retained committed versions           |
| `approx_data_bytes`     | Estimated memory attributable to stored key/value data    |
| `gc_candidate_keys`     | Keys awaiting possible version reclamation                |
| `oldest_pinned_version` | Oldest snapshot currently restricting reclamation         |
| `active_readers`        | Number of readers holding snapshot pins                   |
| `commit_ring_occupancy` | Commit entries not yet retired                            |
| `merge_ring_occupancy`  | Pending merge entries                                     |
| `async_queue_depth`     | Outstanding asynchronous persistence batches              |
| `async_queue_bytes`     | Estimated bytes awaiting asynchronous persistence         |
| `async_dropped_batches` | Batches rejected by asynchronous persistence backpressure |
| `is_poisoned`           | Whether the engine has entered a terminal failure state   |
| `poison_reason`         | Recorded reason for the terminal failure                  |

These fields are especially useful in combination.

For example, growing `live_versions` together with an old `oldest_pinned_version` can indicate that a long-lived reader is preventing reclamation.

A growing persistence queue, by contrast, points to a logging-throughput mismatch rather than necessarily to transaction snapshot retention.

Similarly, persistent commit or merge ring occupancy may suggest that transaction completion, physical merge retirement, or queue cleanup is lagging.

These are diagnostic interpretations, not independently measured guarantees that any particular metric always identifies a single cause.

The counters are intended for operational insight rather than exact process-memory accounting.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs?utm_source=chatgpt.com)

### 8.2 Conflict diagnostics through `tracing`

SurrealMX integrates with Rust's `tracing` ecosystem.

Version 1.0 exposes the constant:

`LOG_TARGET_CONFLICTS = "surrealmx::conflicts"`

In debug builds, the engine can emit conflict-key information when transactions encounter read or write conflicts. A `tracing-subscriber` filter can selectively enable those diagnostics.

For example:

```
RUST_LOG="info,surrealmx::conflicts=debug"
```

The debug-build restriction matters: this conflict logging should not be assumed to provide complete key-level explanations in an optimized release deployment.

The broader `tracing` instrumentation can be incorporated into an application's existing tracing setup without adding a separate SurrealMX-specific logging system.

Sources:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md?utm_source=chatgpt.com)
- [https://docs.rs/tracing/](https://docs.rs/tracing/?utm_source=chatgpt.com)
- [https://docs.rs/tracing-subscriber/](https://docs.rs/tracing-subscriber/?utm_source=chatgpt.com)

### 8.3 Fail-stop and error semantics

Version 1.0 introduces explicit terminal failure handling.

`Database::is_poisoned()` and `Database::check_poisoned()` expose the stopped state, with `DatabaseStats` carrying additional diagnostic information.

The public error model distinguishes several failure classes:

| Error                                  | Interpretation                                                 |
| -------------------------------------- | -------------------------------------------------------------- |
| `KeyReadConflict` / `KeyWriteConflict` | Transaction conflict; retrying a fresh transaction may succeed |
| `KeyAlreadyExists`                     | Insert-only precondition failed                                |
| `ValNotExpectedValue`                  | Conditional mutation precondition failed                       |
| `TxTooOld`                             | Transaction exceeded its permitted lifetime                    |
| `MemoryLimit`                          | Estimated memory budget prevents the commit                    |
| `AsyncQueueFull`                       | Asynchronous persistence capacity prevents the commit          |
| `TxCommitNotPersisted`                 | Transaction changes were applied but persistence failed        |
| `DatabasePoisoned(String)`             | Database entered a terminal stopped state                      |
| `TxClosed` / `TxNotWritable`           | Invalid transaction lifecycle or operation                     |
| `NoSavepoint`                          | Savepoint-stack misuse                                         |

The distinction between retryable transaction failures and terminal database failures is essential.

An ordinary conflict is part of optimistic concurrency control. A persistence failure after in-memory application is not safely handled by treating the transaction as though it never existed.

SurrealMX's fail-stop handling makes unrecoverable failures explicit rather than continuing to accept operations against potentially inconsistent storage state.

### 8.4 Deterministic concurrency testing

SurrealMX's quality-assurance strategy includes mechanisms particularly relevant to a lock-free transactional engine.

- Deterministic simulation testing: The project's CI defines 5,000 seeds with 5,000 operations each, totaling 25 million operations, against a reference model.
- Loom model checking: The commit-ring protocol is tested under explored thread interleavings with a bounded preemption setting.
- Miri: Selected engine and direct-operation tests run with Rust's interpreter-based memory/aliasing checks.
- Integration tests: The tagged test suite covers isolation, concurrent range access, snapshots, log recovery, conditional mutations, savepoints, memory bounds, large transactions and garbage collection.

The upstream project reports zero divergences in its deterministic simulation campaign.

This is meaningful evidence of systematic concurrency testing, but it is not a formal proof that every possible interleaving, platform behavior or filesystem failure has been exhaustively checked.

The tagged manifest also restricts unsafe Rust within the engine, with an explicitly documented exception for the epoch-protected `ArcCell` implementation in `sync.rs`.

Sources:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/.github/workflows/ci.yml](https://github.com/surrealdb/surrealmx/blob/v1.0.0/.github/workflows/ci.yml?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/tree/v1.0.0/src/test/sim](https://github.com/surrealdb/surrealmx/tree/v1.0.0/src/test/sim?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/tree/v1.0.0/tests](https://github.com/surrealdb/surrealmx/tree/v1.0.0/tests?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/sync.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/sync.rs?utm_source=chatgpt.com)

### 8.5 Platform support

The v1.0 build matrix covers Linux, Windows and macOS on relevant x86-64 and ARM64 targets. It also includes build coverage for Android and several Apple device platforms, plus WebAssembly testing in browser and Node.js environments.

There is a crucial distinction between the in-memory core and persistence support:

- The core transactional database is designed to compile for `wasm32`.
- Native filesystem persistence modules are excluded on `wasm32`.
- Background thread-based GC and cleanup workers are unavailable in the same way on WebAssembly.
- Manual cleanup and GC operations remain relevant when automatic workers are unavailable.

SurrealMX itself exposes a synchronous Rust programming interface. Applications using async runtimes must distinguish in-memory operations from potentially blocking operations such as synchronous persistent commits and snapshots.

The presence of WebAssembly support does not imply browser-managed durable storage equivalent to IndexedDB. That persistence model belongs to different supporting infrastructure.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/lib.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/lib.rs?utm_source=chatgpt.com)

## 9. Performance characteristics and benchmark interpretation

SurrealMX's performance is a major part of its design, but the most useful understanding comes from identifying which operations and workload characteristics make its architecture advantageous.

### 9.1 Mechanisms responsible for its performance profile

The combination of ART indexing, lock-free reads and optimistic commits reduces several common costs:

- Point reads avoid global datastore read locks and can inspect existing byte buffers directly.
- Independent writers can progress through the commit pipeline concurrently.
- Transaction pooling reduces repeated allocation and initialization of transaction state.
- Ordered scans avoid building a separate sorted representation of the datastore.
- Version reclamation happens substantially inline rather than requiring frequent full-database compaction.
- Borrowed-value inspection avoids creating result objects when immediate processing is sufficient.
- Group commit reduces redundant synchronization operations when multiple durable commits overlap.

None of these mechanisms eliminates the cost of memory allocation, cache misses, conflict tracking, version maintenance or persistence I/O.

The total benefit therefore depends on the actual mix of read, write, scan, and durability operations.

### 9.2 Published v1.0 benchmark

The v1.0 README reports a comparative benchmark using:

- AMD Ryzen Threadripper 9970X, 32 cores / 64 threads
- 128 GiB DDR5 memory
- 15 million keys
- 48 worker threads and 128 clients
- The upstream `crud-bench` harness
- Best of three runs

Reported operations per second, in millions

SurrealMX

LMDB

RocksDB

06121824Point readCreateUpdateDeleteScan, 100 keys

Published SurrealMX v1.0 benchmark; in-memory LMDB and RocksDB comparisons. Operations and benchmark methods differ by category.

The headline results are substantial: approximately 21.5 million point reads per second and 18 million limited scans per second under the reported conditions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



However, the read figure uses the optimized borrowed-value `with_value` API, and the scan result is for retrieving the first 100 entries. These results should not be transferred to transaction-heavy, durable, remote, document-query or arbitrary range workloads.

### 9.3 The overlooked benchmark distinctions

The same upstream benchmark reports materially less favorable results for some other scan patterns.

| Operation              | SurrealMX | LMDB      | RocksDB  |
| ---------------------- | --------- | --------- | -------- |
| Skip 5,000, return 100 | 232,121/s | 552,885/s | 47,671/s |
| Full-range count       | 17.3/s    | 24.2/s    | 5.0/s    |

This is important evidence that optimizing the first bounded range results does not imply best-in-class performance for large offsets or full-dataset scans.

The difference also illustrates why ordered key access is not equivalent to an indexed analytical aggregation engine.

The published memory results are similarly nuanced:

| Memory measurement | SurrealMX | LMDB     | RocksDB  |
| ------------------ | --------- | -------- | -------- |
| Resting            | 15.9 GiB  | 19.8 GiB | 26.3 GiB |
| Peak               | 32.0 GiB  | 20.1 GiB | 96.1 GiB |

SurrealMX had the lowest resting footprint in this benchmark but a larger peak than LMDB. Concurrent MVCC updates and retained versions contribute to that distinction.

These figures are upstream vendor-published results, not independently reproduced measurements. They also compare particular in-memory configurations; they do not establish equivalent durability guarantees across engines.

Benchmark sources:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md?utm_source=chatgpt.com)
- [https://github.com/surrealdb/crud-bench](https://github.com/surrealdb/crud-bench?utm_source=chatgpt.com)
- [https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine](https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine?utm_source=chatgpt.com)

## 10. Integration boundaries and the SurrealDB version distinction

### 10.1 Direct SurrealMX versus SurrealDB

The two integration surfaces have different responsibilities.

| Requirement                              | Direct SurrealMX 1.0        | Through SurrealDB                                 |
| ---------------------------------------- | --------------------------- | ------------------------------------------------- |
| Ordered binary KV storage                | Native                      | Storage-engine implementation                     |
| Transactions and MVCC                    | Native                      | Exposed through SurrealDB's datastore abstraction |
| Conditional key mutations                | Native                      | Through higher-level data operations              |
| Document schemas and record types        | Caller responsibility       | SurrealDB                                         |
| Graph relationships                      | Caller responsibility       | SurrealDB                                         |
| Secondary and specialized indexes        | Caller responsibility       | SurrealDB query/index layer                       |
| SurrealQL                                | Not included                | SurrealDB                                         |
| Authentication and permissions           | Not included                | SurrealDB                                         |
| Record validation and assertions         | Caller responsibility       | SurrealDB                                         |
| Snapshot/AOL durability                  | Native on supported targets | Configured through SurrealDB's storage backend    |
| Distributed query/service infrastructure | Not included                | SurrealDB deployments and related services        |

The direct integration path offers more control over byte representation, transaction boundaries, ordered keys and memory-oriented data access.

The SurrealDB path offers a substantially richer data/query model at the cost of an additional interpretation, serialization and execution layer.

A particularly valuable boundary is the ability to replace bespoke transactional storage machinery. SurrealMX can supply ordered indexing, consistency, version lifetime, concurrent commits and recovery together, while leaving domain structure and query semantics to the caller.

### 10.2 SurrealDB 3.3.4 does not embed SurrealMX 1.0

This is a version-specific issue worth making explicit.

I verified the tagged `v3.3.4` SurrealDB workspace manifest. It declares:

```
surrealmx = "0.27.0"
```

It does not declare `surrealmx = "1.0.0"`.

In Cargo's version-resolution rules, `"0.27.0"` is a compatible `0.27.x` requirement, not a minimum that automatically accepts the new 1.0 major release.

Therefore, the presence of an independently released SurrealMX 1.0 crate does not establish that SurrealDB 3.3.4 incorporates it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/surrealdb/surrealdb/blob/v3.3.4/Cargo.toml](https://github.com/surrealdb/surrealdb/blob/v3.3.4/Cargo.toml?utm_source=chatgpt.com)

This matters for performance improvements, public APIs, changes to durability handling and any new engine-level guarantees.

A future SurrealDB release or a custom integration may adopt SurrealMX 1.0, but those changes must be verified against the actual dependency and adapter implementation rather than assumed from SurrealMX's release date.

### 10.3 SurrealDB explicitly selects snapshot isolation

There is an additional architectural distinction.

The tagged SurrealDB 3.3.4 memory backend creates its engine transactions as follows:

```
let txn = self.db.transaction(write).with_snapshot_isolation();
```

This is an explicit request for SurrealMX's snapshot-isolation mode.

Consequently, two statements that appear contradictory can both be correct:

1. Native SurrealMX 1.0 writable transactions default to serializable snapshot isolation.
2. SurrealDB 3.3.4's SurrealMX adapter explicitly chooses snapshot isolation.

The second behavior is verified in SurrealDB's `surrealdb-kvs-mem` implementation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs?utm_source=chatgpt.com)

It would therefore be incorrect to infer SurrealDB's complete transaction guarantees solely from SurrealMX 1.0's native defaults. Those guarantees depend on the specific adapter version and any consistency mechanisms provided above it.

The adapter is also significant for another reason: it translates SurrealMX's internal errors and transactions into SurrealDB's generic asynchronous key-value interface. That layer, rather than the direct SurrealMX API, is where engine-specific behavior is adapted to SurrealDB's query execution model.

### 10.4 Time-travel and versioned queries

SurrealMX's MVCC is sometimes confused with long-term version-history storage.

Current SurrealDB documentation is explicit that its SurrealMX memory backend does not support versioned queries from SurrealDB 3.3.0 onward. A server started with `versioned=true` or nonzero `retention` fails to start for that backend.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



This corrects older SurrealDB documentation and articles describing time-travel support in the memory engine.

For historical `VERSION` queries, the documented alternatives are versioned SurrealKV or RocksDB configurations.

The distinction is:

- MVCC snapshots: Protect transaction consistency while a transaction is active.
- Persistent historical queries: Preserve versions intentionally for later retrieval at historical timestamps.

The second capability does not follow automatically from the first.

Sources:

- [https://surrealdb.com/docs/running/in-memory](https://surrealdb.com/docs/running/in-memory?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/reference/cli/db/commands/start](https://surrealdb.com/docs/reference/cli/db/commands/start?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/build/embedding/storage-engines](https://surrealdb.com/docs/build/embedding/storage-engines?utm_source=chatgpt.com)

### 10.5 Internal engine dependencies and related tooling

Several Rust crates contribute important mechanisms to SurrealMX 1.0, but should not be confused with its public interface.

| Library or tool                       | Role                                                            |
| ------------------------------------- | --------------------------------------------------------------- |
| `artmap`                              | Concurrent adaptive radix tree underlying the ordered datastore |
| `byteslice` / `bytes`                 | Shared byte-oriented data representation                        |
| `papaya`                              | Concurrent hash-map data structures used internally             |
| `crossbeam-epoch`                     | Epoch-protected memory reclamation                              |
| `crossbeam-deque` / `crossbeam-queue` | Concurrent work and queue primitives                            |
| `parking_lot`                         | Lightweight synchronization primitives                          |
| `bincode` / `serde`                   | Persistence serialization                                       |
| `lz4`                                 | Snapshot compression                                            |
| `xxhash-rust`                         | Checksums for log frames                                        |
| `tracing`                             | Instrumentation and conflict diagnostics                        |
| `loom`                                | Concurrency model-checking during testing                       |
| `criterion`                           | Engine-level benchmarking                                       |

These dependencies are relevant to understanding how SurrealMX works and investigating performance or concurrency behavior. Most are implementation details, not independent application-level configuration or extension points.

For higher-level integration, the relevant companion projects are SurrealDB itself and its storage abstraction crates. SurrealKV and RocksDB are alternative backend choices, not additional layers that SurrealMX requires.

Sources:

- [https://github.com/surrealdb/surrealmx/blob/v1.0.0/Cargo.toml](https://github.com/surrealdb/surrealmx/blob/v1.0.0/Cargo.toml?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealdb/tree/v3.3.4/surrealdb/kvs-mem](https://github.com/surrealdb/surrealdb/tree/v3.3.4/surrealdb/kvs-mem?utm_source=chatgpt.com)
- [https://github.com/surrealdb/surrealdb/tree/v3.3.4/surrealdb/kvs](https://github.com/surrealdb/surrealdb/tree/v3.3.4/surrealdb/kvs?utm_source=chatgpt.com)

## 11. A compact native Rust example

The following illustrates the tagged v1.0 API and the combination of transaction-scoped consistency, partial rollback, and a range query. It is illustrative code based on the verified source, not an executed example.

```
use surrealmx::{Database, DatabaseOptions, Error};
use std::time::Duration;

fn main() -> Result<(), Error> {
    let options = DatabaseOptions::new()
        .with_max_transaction_age(Duration::from_secs(30))
        .with_memory_limit(512 * 1024 * 1024);

    let db = Database::new_with_options(options);

    // Native writable transactions default to serializable isolation.
    let mut tx = db.transaction(true);

    tx.set("account:001", "active")?;

    // A nested modification scope.
    tx.set_savepoint()?;
    tx.set("account:002", "temporary")?;

    // Revert only the writes made in the nested scope.
    tx.rollback_to_savepoint()?;

    // Ordered access within the transaction's consistent snapshot.
    let entries = tx.scan("account:000".."account:999", None, None)?;

    println!("Visible entries: {}", entries.len());

    tx.commit()?;

    // Low-overhead standalone point lookup.
    let value = db.get("account:001")?;
    assert!(value.is_some());

    // Runtime accounting and reclamation diagnostics.
    let stats = db.stats();
    println!("Keys: {}", stats.live_keys);

    Ok(())
}
```

For a single-process in-memory store, this avoids introducing a separate lock table, custom transaction journal, snapshot manager, ordered index and basic garbage collector.

It does not eliminate the need for application-level key encoding, data modeling, error handling, or higher-level domain consistency rules.

## 12. Important limitations and v1.0 documentation traps

Several distinctions are particularly consequential when evaluating the released version.

| Area                          | Correct interpretation                                                                                       |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Memory residency              | The active datastore is memory-resident, even when persistence is enabled                                    |
| Transaction isolation         | Native writable transactions default to serializable snapshot isolation; adapters may select another mode    |
| Historical data               | MVCC is not a general time-travel database or indefinite version archive                                     |
| Multi-process access          | SurrealMX is an embedded engine, not a standalone multi-process coordination or distributed database service |
| Ordering                      | Key ordering is byte-lexicographic; domain-aware ordering requires appropriate encoding                      |
| Secondary indexes             | Direct users implement their own additional key layouts or use SurrealDB's query/index layer                 |
| Durability                    | In-memory is the default; persistence and exact fsync behavior are explicit choices                          |
| Memory limit                  | Approximate engine-data budget, not a guaranteed maximum process RSS                                         |
| Async persistence             | Normal shutdown drains queued changes; crashes can still lose unpersisted commits                            |
| Recovery errors               | A failed append may occur after in-memory changes were applied                                               |
| Snapshot files                | Recoverable current state, not complete historical version storage                                           |
| Encryption and access control | Not provided as standalone SurrealMX transactional KV features                                               |
| WebAssembly                   | In-memory operation is supported; native filesystem persistence is excluded                                  |
| v1.0 compatibility            | New framed AOL records are not transparently readable by older releases                                      |

Three documentation inconsistencies deserve special attention:

First, the default isolation level. Some older pages describe snapshot isolation as the default for writable transactions. The v1.0 source initializes native transactions with `SerializableSnapshotIsolation`. Explicit `.with_snapshot_isolation()` remains available.

Second, the default persistence mode. Some older README/Cargo documentation labels full persistence as the default. The tagged v1.0 configuration actually defaults to `AolMode::Never` and `SnapshotMode::Never`; `Database::new()` does not persist anything.

Third, the poisoned error variant. The v1.0 release notes refer to `Error::Poisoned`, but the actual tagged `src/err.rs` defines:

`Error::DatabasePoisoned(String)`

The tagged source should take precedence when implementing against the exact published version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 13. Primary-source research map

The following resources collectively provide the most useful routes into SurrealMX 1.0's architecture, capabilities and implementation. All URLs are provided as plain text.

| Topic                                                        | Source                                                                                                                                                                                                      |
| ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Version 1.0 release notes, changes, and compatibility        | [https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0?utm_source=chatgpt.com)                                                             |
| Complete, version-locked repository                          | [https://github.com/surrealdb/surrealmx/tree/v1.0.0](https://github.com/surrealdb/surrealmx/tree/v1.0.0?utm_source=chatgpt.com)                                                                             |
| Overview, persistence guide, benchmark results and examples  | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.0.0/README.md?utm_source=chatgpt.com)                                                         |
| Package metadata and Rust dependencies                       | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/Cargo.toml](https://github.com/surrealdb/surrealmx/blob/v1.0.0/Cargo.toml?utm_source=chatgpt.com)                                                       |
| Public types and platform-gated exports                      | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/lib.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/lib.rs?utm_source=chatgpt.com)                                                       |
| Transaction lifecycle, validation, write sets and savepoints | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/tx.rs?utm_source=chatgpt.com)                                                         |
| Core database operations, GC and worker lifecycle            | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/db.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/db.rs?utm_source=chatgpt.com)                                                         |
| Datastore architecture, logical versioning and coordination  | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/inner.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/inner.rs?utm_source=chatgpt.com)                                                   |
| MVCC version-chain representation                            | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/versions.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/versions.rs?utm_source=chatgpt.com)                                             |
| Commit-ring implementation                                   | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/ring.rs?utm_source=chatgpt.com)                                                     |
| Merge coordination                                           | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/merge_ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/merge_ring.rs?utm_source=chatgpt.com)                                         |
| Ordered cursors and seeking                                  | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/cursor.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/cursor.rs?utm_source=chatgpt.com)                                                 |
| Persistence, frame encoding, recovery and snapshots          | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/persistence.rs?utm_source=chatgpt.com)                                       |
| Runtime options and tuning profiles                          | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs?utm_source=chatgpt.com)                                               |
| Runtime statistics                                           | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs?utm_source=chatgpt.com)                                                   |
| Error model                                                  | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/err.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/err.rs?utm_source=chatgpt.com)                                                       |
| Garbage-candidate reclamation                                | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/gc_candidates.rs](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/gc_candidates.rs?utm_source=chatgpt.com)                                   |
| CI, Loom, Miri, WASM, DST and platform coverage              | [https://github.com/surrealdb/surrealmx/blob/v1.0.0/.github/workflows/ci.yml](https://github.com/surrealdb/surrealmx/blob/v1.0.0/.github/workflows/ci.yml?utm_source=chatgpt.com)                           |
| Integration and concurrency tests                            | [https://github.com/surrealdb/surrealmx/tree/v1.0.0/tests](https://github.com/surrealdb/surrealmx/tree/v1.0.0/tests?utm_source=chatgpt.com)                                                                 |
| Upstream performance benchmark                               | [https://github.com/surrealdb/crud-bench](https://github.com/surrealdb/crud-bench?utm_source=chatgpt.com)                                                                                                   |
| Official v1.0 engineering announcement                       | [https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine](https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine?utm_source=chatgpt.com) |
| SurrealDB storage-engine integration                         | [https://surrealdb.com/docs/build/embedding/storage-engines](https://surrealdb.com/docs/build/embedding/storage-engines?utm_source=chatgpt.com)                                                             |
| SurrealDB in-memory backend and versioning restrictions      | [https://surrealdb.com/docs/running/in-memory](https://surrealdb.com/docs/running/in-memory?utm_source=chatgpt.com)                                                                                         |
| SurrealDB v3.3.4 exact SurrealMX dependency                  | [https://github.com/surrealdb/surrealdb/blob/v3.3.4/Cargo.toml](https://github.com/surrealdb/surrealdb/blob/v3.3.4/Cargo.toml?utm_source=chatgpt.com)                                                       |
| SurrealDB v3.3.4 adapter and selected isolation mode         | [https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs?utm_source=chatgpt.com)                   |

## 14. Architectural synthesis

SurrealMX 1.0's most distinctive contribution is not any single data structure or isolated optimization. It is the integration of ordered in-memory indexing, optimistic serializable transactions, MVCC snapshot management, concurrent commit publication, low-allocation access, garbage reclamation and optional crash-recoverable persistence into one embedded Rust engine.

The combinations are what make it architecturally interesting:

- ART + byte-oriented composite keys + cursors can provide high-performance ordered data structures and index-like access without introducing a separate indexing service.
- MVCC + serializable conflict detection + savepoints can express cross-key transactional operations without a bespoke locking, validation and rollback framework.
- Lock-free reads + transaction pooling + borrowed-value inspection can reduce synchronization and allocation costs on hot paths.
- Inline GC + pinned reader tracking + transaction age limits makes snapshot consistency and resource lifetime part of one coordinated system.
- Framed AOL + consistent-cut snapshots + group commit + poison-state handling provides a native recovery and durability foundation with explicit failure semantics.
- Direct Rust embedding versus SurrealDB integration allows the same category of storage-engine functionality to serve either a low-level KV-oriented application or a richer database model, though exact capabilities and isolation choices depend on the adapter version.

The fundamental tradeoff remains clear: SurrealMX deliberately favors fast, concurrent, memory-resident transactional access. It is not a replacement for a persistent disk-oriented database when the working dataset is substantially larger than memory, nor does it provide a complete document or relational query model independently.

For v1.0 specifically, the two most consequential details to retain are its native serializable transaction default and its new crash-consistent persistence implementation. Both are verified in the tagged source, and neither should be assumed to describe SurrealDB 3.3.4's existing `0.27.0` adapter without further version-specific integration work.