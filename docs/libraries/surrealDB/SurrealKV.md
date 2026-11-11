# SurrealKV (Rust): Comprehensive Technical Reference

Research date: November 10, 2026 | Target release: SurrealKV 1.3.0 (November 6, 2026) | Language: Rust | License: Apache 2.0

SurrealKV is a high-performance, embedded, transactional key-value storage engine developed by SurrealDB. Its architecture combines a log-structured merge tree (LSM), multi-version concurrency control (MVCC), an asynchronous commit pipeline, optional historical version retention, and a configurable storage subsystem.

The 1.0 release on October 13, 2026 represented an important transition: SurrealKV became a standalone, production-oriented storage engine intended to replace RocksDB as SurrealDB's persistent storage foundation. Subsequent 1.2 and 1.3 releases improved recovery, concurrency, write stalls, and batched reads.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



Upstream repository: `https://github.com/surrealdb/surrealkv`

Rust crate: `https://crates.io/crates/surrealkv`

Release history: `https://github.com/surrealdb/surrealkv/releases`

## 1. Executive orientation

SurrealKV is best understood as a transactional storage kernel, rather than a complete database management system.

It manages durable byte-oriented records, concurrency, transaction isolation, historical versions, on-disk indexing, compaction, and recovery. Higher-level facilities such as schemas, declarative queries, relationships, secondary indexes, and application-specific validation belong to software built above it.

| Capability family    | What SurrealKV provides                                                               |
| -------------------- | ------------------------------------------------------------------------------------- |
| Storage engine       | Persistent LSM tree, sorted immutable SSTables, memtables, write-ahead logging        |
| Transactions         | Atomic multi-key updates, snapshot isolation, optimistic concurrency control          |
| Reads                | Point lookups, ordered key-range iteration, batched point reads                       |
| Concurrency          | Multiple concurrent readers and writers, asynchronous commit processing               |
| Temporal storage     | Timestamped versions, point-in-time reads, historical iteration, retention policies   |
| Durability           | WAL-based recovery, immediate/eventual durability, integrity checks                   |
| Storage optimization | Compression, Bloom filters, block caching, compaction, optional value separation      |
| Operations           | Checkpointing, restoration, storage inspection, migration utilities                   |
| Deployment           | Native embedded Rust applications, SurrealDB storage backend, browser OPFS            |
| Extensibility        | Storage configuration, comparison and filtering policies, lower-level instrumentation |

The most consequential architectural distinction is that SurrealKV provides both concurrency isolation and optional persistent historical data, but these are different mechanisms. Snapshot isolation protects active transactions; persistent versioning enables historical queries after those transactions have completed.

Similarly, the asynchronous commit pipeline does not make every database operation asynchronous: ordinary point reads and range iterators remain synchronous, while an opt-in asynchronous point-read API exists.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



## 2. Architecture and execution model

### 2.1 Storage architecture

SurrealKV uses a leveled LSM architecture with several cooperating components:

Rust application / SurrealDB

Transaction API · Key/value reads and writes · Ordered scans

Transaction and commit coordination

MVCC snapshots · OCC validation · Commit ring · Group commit

Write-ahead log

Recovery · Durability · Ordered commits

Adaptive radix memtable

Concurrent mutable state · Snapshot-visible versions

Leveled LSM storage

Immutable SSTables · Sorted keys · Block indexes · Filters · Compaction

Optional value log

Large-value separation

Manifest and files

Level metadata · Recovery state

Conceptual data path; reads combine relevant memtables, SSTables, and value-log references rather than traversing every layer sequentially.

The key architectural elements are:

Adaptive radix tree memtables. Incoming writes are accumulated in memory using an adaptive radix tree backed by arena allocation. Compared with pointer-heavy skiplist approaches, this design can improve key locality and reduce allocation overhead. It is particularly relevant to high-frequency mutation workloads.

Write-ahead logging. WAL records provide a durable recovery source independently of SSTable flushes. The engine can acknowledge a transaction before its records have been incorporated into immutable SSTables.

Immutable SSTables. When memtables are flushed, their records become sorted disk-resident tables. Point lookups use table indexes and membership filters to avoid unnecessary disk reads.

Leveled compaction. Background operations merge overlapping tables, control read amplification, and eventually reclaim overwritten values and tombstones. Compaction also enforces historical retention policies.

Block caching. A unified cache serves SSTable data and index blocks and, when value separation is enabled, value-log data.

Manifest management. Persistent metadata records the active organization of tables, supporting recovery and atomic changes to the storage structure.

SurrealDB's engineering explanation specifically identifies the radix-tree memtable, commit ring, and inexpensive snapshot reads as principal performance mechanisms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



### 2.2 The asynchronous commit pipeline

A distinctive part of SurrealKV 1.x is its commit-ring architecture.

A simplified commit sequence is:

1. A transaction assembles its pending mutations.
2. Optimistic concurrency validation checks for conflicts with relevant committed or in-flight writes.
3. The commit pipeline orders accepted transactions and assigns sequence numbers.
4. A flusher groups multiple commits into WAL operations.
5. Data is applied to the mutable storage structure and becomes visible according to the pipeline's publication ordering.
6. Each transaction receives its result once the required commit and durability conditions have been met.

The design combines parallel validation with ordered publication and group commit. Multiple transactions can therefore share the cost of WAL operations and, where requested, synchronization to persistent storage.

The important distinction is between asynchronous coordination and asynchronous disk I/O. SurrealKV does not universally use nonblocking kernel I/O. Its current implementation routes selected operations through `affinitypool` and otherwise uses synchronous file access.

In particular:

- WAL append/synchronization, asynchronous point reads, and parallel WAL recovery can use an application-installed worker pool.
- Ordinary `get` calls and iterators use synchronous reads.
- Without an installed worker pool, the relevant work may execute inline.
- The upstream implementation explicitly does not use Linux `io_uring`.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

  GitHub



This gives callers a useful distinction between avoiding caller-side coordination overhead and avoiding blocking execution threads.

### 2.3 Snapshot management and MVCC

MVCC associates stored changes with internal sequence numbers and retains sufficient versions for currently active readers.

A transaction reads against a consistent snapshot, rather than continually observing the latest committed state. This allows writers to continue while older read transactions retain their logical view.

SurrealKV also distinguishes between:

- Sequence-number visibility: which committed writes belong to a transaction's snapshot.
- Application-visible timestamps: which historical value applies to a particular requested time.
- Physical retention: which versions must remain on disk for active snapshots or configured history.

These mechanisms interact during compaction but are not interchangeable.

An old snapshot can require retaining data that an ordinary current read would no longer return. Conversely, enabling persistent historical retention does not make an active transaction read every historical version.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/snapshot.rs`

## 3. Transactions, isolation, and concurrency control

### 3.1 Transaction modes

SurrealKV exposes three transaction modes through `Mode` and `TransactionOptions`.

| Mode        | Behavior                                   | Important distinction                                                                                         |
| ----------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------- |
| `ReadWrite` | Reads and writes within a transaction      | Default mode; permits read-dependent updates and explicit conflict-tracked reads                              |
| `ReadOnly`  | Consistent snapshot reads without mutation | Does not enter the writable commit path                                                                       |
| `WriteOnly` | Mutations without reads                    | Avoids constructing a read snapshot and suits data ingestion where decisions do not depend on existing values |

The principal entry points are `Tree::begin`, `begin_with_mode`, and `begin_with_opts`.

`Tree::view` also provides a closure-based convenience method for a read-only snapshot operation.

### 3.2 Optimistic concurrency and conflict detection

SurrealKV uses optimistic concurrency control rather than a general-purpose pessimistic locking system.

Transactions can construct their updates concurrently. At commit, the storage engine checks whether relevant intervening transactions have changed keys that require validation.

A conflict can produce `Error::TransactionWriteConflict`. An old transaction whose necessary conflict history has been retired can produce `Error::TransactionRetry`.

This is a meaningful distinction from simply serializing all writers through a global lock: transactions can perform useful work concurrently, with conflicts resolved at the commit boundary.

Snapshot isolation is not equivalent to full serializability.

Consider an invariant that depends on two separate keys. Two transactions may read the same old state and write different keys, each preserving the invariant according to its own snapshot, but violating it in combination. This is the classic write-skew problem.

SurrealKV does not automatically prevent every such anomaly.

### 3.3 `get_for_update`: conflict-tracked reads

The API includes an especially useful operation:

`Transaction::get_for_update(key)`

It returns the same value as `get`, but records the key for validation at commit. If another transaction has committed a write to that key since the transaction's snapshot, the commit fails.

This is valuable for read-modify-write operations and conditional updates, especially where the key being read is not otherwise part of the transaction's write set.

However, its semantics are precise:

- It does not obtain an exclusive lock on the key.
- Multiple transactions can read the same key for update concurrently.
- A read-only transaction cannot use it.
- Transactions that only perform tracked reads do not necessarily conflict with each other.
- A caller requiring competing transactions to conflict over an invariant may need a common record that they actually write.

Thus `get_for_update` provides snapshot conflict validation, not a general serializable predicate-locking mechanism.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/transaction.rs`

### 3.4 Read-your-own-writes

Within a transaction, pending mutations are consulted before the stored snapshot.

For example, a transaction that changes a value and then reads the same key sees its pending value rather than the value at the transaction's original snapshot.

The same principle applies to deletions and range tombstones, subject to their ordering within the transaction.

This removes the need for a separate application-side overlay of uncommitted changes.

### 3.5 Savepoints and partial rollback

SurrealKV supports:

- `set_savepoint()`
- `rollback_to_savepoint()`
- `rollback()`

Savepoints are stackable and allow a transaction to discard changes made since a particular savepoint without abandoning its earlier operations.

Rollback to a savepoint also releases conflict-tracked reads first acquired after that savepoint.

This can simplify atomic multi-step operations that need to revise provisional changes before commit.

Savepoints are local transaction state. They are not independent durable subtransactions.

### 3.6 Transaction lifecycle and retry behavior

A successful or failed commit attempt terminates the transaction.

That matters operationally: a transaction returning a conflict cannot simply call `commit()` again. Its reads and mutations must be reconstructed in a new transaction.

An application retry loop should distinguish conflicts from failures involving uncertain durability or partial pipeline application.

In particular, certain serious commit-group failures can poison the database. The error types distinguish operations known not to have started from operations whose final committed outcome must be resolved during recovery.

This makes blind retrying of non-idempotent external operations unsafe without additional application-level safeguards.

## 4. Key-value operations and mutation semantics

SurrealKV stores byte-oriented keys and values. It does not impose a schema or typed record format.

### 4.1 Core mutation operations

| Operation                       | Semantics                                                        |
| ------------------------------- | ---------------------------------------------------------------- |
| `set(key, value)`               | Insert or update a key                                           |
| `set_at(key, value, timestamp)` | Write a version with an explicitly supplied timestamp            |
| `set_with_options`              | Write with operation-specific options                            |
| `delete(key)`                   | Hard delete, logically removing all prior versions of the key    |
| `soft_delete(key)`              | Add a timestamped deletion marker, retaining eligible history    |
| `replace(key, value)`           | Replace a value while discarding prior version history logically |
| `delete_range(start, end)`      | Delete keys within a half-open range using a range tombstone     |

The distinction between ordinary, soft, and hard deletion is particularly important when historical retention is enabled.

Soft deletion preserves a meaningful historical state: a query for an earlier time can still find a value, provided its version remains retained.

Hard deletion defines a history-erasing boundary. The application should not expect earlier values to remain queryable after the deletion.

Replacement similarly establishes a new value while invalidating earlier history, unlike an ordinary versioned `set`.

Physical reclamation still depends on compaction and active-reader visibility; these operations should not be interpreted as guarantees of immediate secure erasure of every storage copy.

### 4.2 Range deletion

`delete_range(start, end)` is implemented as an O(1) logical range-tombstone insertion, rather than individually enumerating and deleting every matching key.

This is a consequential feature for deleting whole namespaces, partitions, or ranges identified by a key prefix.

For example, a key schema that groups records under `session:<escape><id></escape>:` can make a whole session removable through an ordered range operation.

The O(1) claim concerns creation of the logical deletion marker. Subsequent reads and compaction still have work to perform, and actual disk reclamation is deferred.

### 4.3 Key ordering and encoding

The default comparator is `BytewiseComparator`, which orders keys lexicographically.

Consequently, key serialization is part of database design.

A key consisting of multiple components can support efficient prefix and range queries, but only if its byte encoding preserves the desired ordering. Fixed-width or order-preserving encodings may be appropriate for numeric identifiers, timestamps, and composite keys.

SurrealKV also exposes a `Comparator` abstraction, permitting alternative key-ordering behavior.

This can eliminate some application-side sorting, but custom comparison behavior should be treated as a persistent storage-format decision rather than an interchangeable runtime preference.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/comparator.rs`

## 5. Reading, scanning, and historical queries

### 5.1 Point reads and batched retrieval

The ordinary point-read API is `Transaction::get(key)`, returning `Result<Option<Value>>`.

SurrealKV 1.3.0 added two particularly useful batch operations:

- `get_many(keys)` — reads multiple keys using one captured view of the memtables and SSTables.
- `get_many_at(keys, timestamp)` — performs the equivalent batched lookup against historical versions.

Both preserve the input key order and return one optional value per requested key.

The optimization is not a database-side relational join or a general bulk query planner. It avoids repeatedly acquiring tree-state information for individual point lookups, while still performing the necessary searches for each key.

For workloads that repeatedly retrieve a known set of related records, this can reduce storage coordination overhead without requiring an application-defined batching layer.

### 5.2 Borrowed-value callbacks

The 1.3 API additionally exposes `get_with` and `get_many_with`.

These invoke a caller-provided callback with a borrowed value slice:

```
tx.get_many_with(&keys, |index, value| {
    // The borrowed value is valid only during this callback.
})?;
```

This can avoid copying values into a newly allocated `Vec<u8>` when the engine can supply them from its underlying storage representation.

There are two important limitations. The borrowed slice cannot safely be retained beyond the callback, and reading a value from disk or resolving a compressed or value-log-backed representation can still require internal buffers or allocations.

Historical callback variants (`get_at_with`, `get_many_at_with`) exist, but the upstream implementation notes that historical iterators already produce owned values in some paths, so they do not offer the same copy-avoidance guarantee.

### 5.3 Ordered scans and bidirectional cursors

SurrealKV provides `iter`, `scan`, `range`, and `range_with_options`.

The iterator contract supports:

| Operation                      | Purpose                                                |
| ------------------------------ | ------------------------------------------------------ |
| `seek_first()` / `seek_last()` | Begin a forward or reverse traversal                   |
| `next()` / `prev()`            | Advance or reverse a cursor                            |
| `valid()`                      | Determine whether the cursor references an entry       |
| `key()`                        | Access the current key and internal version metadata   |
| `value()`                      | Retrieve the current value                             |
| `value_ref()` / `value_into()` | Control value copying and buffer reuse where supported |

`scan` accepts Rust range expressions, including inclusive and exclusive bounds. `range(start, end)` uses an inclusive lower bound and exclusive upper bound.

The engine merges the relevant in-memory and persistent data sources into a logical ordered view. A caller does not need to merge memtable and SSTable iterators independently.

Prefix scanning is a consequence of key ordering, not a separate full-text indexing mechanism. For a prefix namespace, correctly constructed lower and upper byte boundaries allow the scan to be restricted to that namespace.

A consequential subtlety is that the lower-level `LSMIterator::seek` contract refers to encoded internal keys. Callers should use the appropriate key-encoding helpers or start with the documented range/seek-first patterns rather than assuming an arbitrary user-key byte slice is always a valid internal seek target.

### 5.4 Persistent historical versioning

Historical queries are enabled explicitly:

```
let options = surrealkv::Options::new()
    .with_versioning(true, 0);
```

The second argument is a retention duration in nanoseconds. Zero means that historical versions have no configured age-based expiry.

A versioned tree stamps writes with either their explicitly supplied timestamp or the commit timestamp.

Relevant APIs are:

| API                      | Role                                                         |
| ------------------------ | ------------------------------------------------------------ |
| `set_at(key, value, ts)` | Insert a version with an explicit timestamp                  |
| `get_at(key, ts)`        | Return the value applicable at a given timestamp             |
| `get_many_at(keys, ts)`  | Retrieve several historical values                           |
| `history(start, end)`    | Iterate stored versions within a key range                   |
| `history_with_options`   | Add timestamp bounds, tombstone inclusion, and result limits |

Historical iteration orders keys by key order and versions within each key by descending timestamp.

`HistoryOptions` can filter to an inclusive timestamp interval, include soft-deletion tombstones, and limit the number of results.

For a simple key with versions written at times 100, 200, and 300, a lookup at 250 returns the version from 200, assuming that version has been retained and is visible to the transaction.

Backdated inserts are supported. A value may be written with a timestamp earlier than already recorded versions. Consequently, timestamp order and commit order need not match.

This enables temporal correction and late-arriving observations, but applications must define whether an explicit timestamp means event time, validity time, or something else. SurrealKV supplies timestamped storage, not a complete bitemporal database model.

### 5.5 Retention is not simply a TTL

Historical retention is enforced through compaction. When old versions become eligible for removal, the engine also needs to preserve sufficient preceding state for a query at the beginning of the retained interval.

Retention is therefore better understood as a compaction policy over historical visibility, rather than as a promise that every expired version disappears immediately.

Active snapshots can additionally prevent reclamation of otherwise obsolete versions.

The resulting distinctions are important:

- Setting retention to zero retains history indefinitely, potentially increasing persistent storage.
- Setting finite retention does not guarantee immediate physical deletion when the duration expires.
- A historical value outside the retained interval may no longer be available.
- A hard delete or replacement can invalidate historical versions independently of age-based retention.

Unlike SurrealKV 0.21, SurrealKV 1.3 does not require an enabled value log or a separate versioned B+tree index to support versioning. Version metadata is stored with values in the LSM representation.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/transaction.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lib.rs`

## 6. Durability, write-ahead logging, and recovery

### 6.1 Transaction durability options

The `Durability` enum defines two public durability levels.

| Mode                 | Commit behavior                                         | Consequence                                                                 |
| -------------------- | ------------------------------------------------------- | --------------------------------------------------------------------------- |
| `Eventual` (default) | Does not require an `fsync` before returning            | Recent committed data can be lost following a power failure                 |
| `Immediate`          | Waits for required WAL synchronization before returning | Stronger persistence guarantee, subject to filesystem and hardware behavior |

These modes do not change transaction isolation. They change what the commit promises about persistence.

In particular, an atomic transaction with eventual durability is still atomic in the storage engine's logical commit model, but an acknowledged commit may not survive a system crash.

The API allows per-transaction selection:

```
let mut tx = tree.begin()?;
tx.set_durability(surrealkv::Durability::Immediate);
tx.set(b"critical:key", b"value")?;
tx.commit().await?;
```

`Tree::flush_wal(sync)` can also flush previously committed WAL and value-log data, optionally forcing synchronization when `sync = true`.

This provides a mechanism for separating transaction-level acknowledgment policy from explicit storage synchronization.

### 6.2 WAL segment management

The write-ahead log is segmented and supports rotation, replay, and recovery of changes not yet incorporated into persistent SSTables.

Because SurrealKV may use a separate value log, recovery must consider references to values as well as the WAL record itself.

An otherwise valid WAL record referencing a value that was not durably written to the value log cannot simply be replayed as if it were complete.

SurrealKV 1.x contains recovery logic that evaluates these dependencies and can reject inconsistent states rather than returning incomplete values.

### 6.3 Recovery modes

The current `WalRecoveryMode` enum has three substantive policies:

| Mode                  | Behavior                                                                                                          |
| --------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `TolerateTornTail`    | Default. Repairs a provably torn final-log tail, but refuses corruption that cannot safely be classified that way |
| `AbsoluteConsistency` | Fails recovery on WAL corruption rather than trimming or repairing it                                             |
| `SalvageValidPrefix`  | Salvages the valid prefix before corruption, potentially discarding subsequent records                            |

`TolerateCorruptedWithRepair` remains available as a backward-compatible alias for `SalvageValidPrefix`.

This distinction matters because a permissive recovery policy can trade data completeness for availability. A valid prefix is not necessarily the full set of acknowledged operations.

Applications with strict recovery requirements should distinguish the default torn-tail policy from deliberate prefix salvage.

### 6.4 WAL preallocation

Version 1.2 introduced `with_wal_preallocation(true)`.

On supported native filesystems, this writes zero-filled capacity ahead of the active WAL's logical end. Subsequent appended records can overwrite that space rather than continually extending the file.

The intended benefit is to reduce filesystem metadata work associated with durable appends. On filesystems such as ext4 or XFS, avoiding repeated size updates can reduce the cost of synchronization.

This optimization is opt-in, disabled by default, and restricted to the supported native plaintext WAL path.

It should not be interpreted as a universal write-performance optimization across encrypted databases, browser storage, or every filesystem.

### 6.5 File integrity and corruption detection

SurrealKV implements integrity checks at several layers:

- WAL records have checksums and recovery validation.
- SSTable blocks have checksums checked during reads.
- Value-log checksum verification can be configured.
- Encrypted files use authenticated encryption, which detects tampering or incorrect keys.
- Offline scrubbing can explicitly examine SSTable blocks that normal workloads may not read.

The distinction between checksums and authenticated encryption is important. CRC verification detects corruption but is not a security mechanism against deliberate modification. Authenticated encryption provides stronger tamper detection for data protected by the relevant keys.

The `VLogChecksumLevel` options are `Disabled` and `Full`; checksum verification for the value log is not enabled by default.

Continuous background integrity scrubbing is not implemented in the examined release, despite the presence of some scrubber-related internal structures and counters.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/wal/recovery.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/error.rs`

Release details: `https://github.com/surrealdb/surrealkv/releases/tag/v1.2.0`

## 7. Storage layout, compaction, and performance controls

### 7.1 Memtable sizing and write stalls

Incoming writes accumulate in a mutable memtable. When a memtable reaches its configured threshold, it is rotated into an immutable memtable pending disk flush.

If background flushing or compaction cannot keep up with incoming writes, SurrealKV applies backpressure.

The default native settings in version 1.3 are:

| Parameter                                | Default |
| ---------------------------------------- | ------- |
| Memtable arena target                    | 100 MiB |
| Maximum immutable memtables before stall | 2       |
| L0 tables triggering compaction          | 4       |
| L0 tables triggering write stall         | 12      |
| SSTable target file size                 | 64 MiB  |
| Number of levels                         | 6       |
| Base level size                          | 256 MiB |
| Level size multiplier                    | 10×     |

These are default configuration values, not guaranteed memory or disk utilization bounds.

In particular, the memtable arena limit counts its internal key structures; heap-resident values are not necessarily included in that limit. Workloads with large values can therefore consume substantially more memory than their nominal memtable target suggests.

### 7.2 Manual compaction

`Tree::compact_range` allows a caller to request compaction over a particular key interval.

`CompactRangeOptions` controls whether to flush memtables first, the target LSM level, and whether to rewrite the bottommost level.

Manual compaction can be useful when a large namespace has been deleted, when a historical retention policy has made old versions eligible for removal, or when an operation would benefit from consolidating a known range.

The engine preserves versions still needed by active snapshots. Manual compaction does not override transaction visibility rules.

It is a blocking I/O operation and should be kept off latency-sensitive asynchronous executor threads.

### 7.3 Value separation with the value log

SurrealKV optionally supports WiscKey-style value separation.

When enabled, sufficiently large values are placed in a value log and the LSM records hold references to them.

The relevant options are:

- `with_enable_vlog(true)`
- `with_vlog_value_threshold(bytes)`
- `with_vlog_max_file_size(bytes)`
- `with_vlog_checksum_verification(level)`

The default threshold is 1 KiB, but value separation itself is disabled by default.

Why it matters: A leveled LSM may rewrite the same value repeatedly as keys move between SSTables. Separating large values allows many compaction operations to move comparatively small references instead.

This can reduce write amplification for substantial record payloads.

However, it adds value-log reads, reference management, and garbage-collection obligations. For small values, or workloads dominated by random reads where values are not resident in cache, the additional indirection can outweigh some compaction benefits.

Value-log garbage collection is coordinated with the oldest referenced files in memtables, SSTables, and in-flight commit groups. This prevents reclamation of segments still needed by live data.

### 7.4 Compression by LSM level

SurrealKV's public `CompressionType` supports:

- `None`
- `SnappyCompression`
- `ZstdCompression`

Compression can be selected by level through `with_compression_per_level`. A convenience option, `with_l0_no_compression`, disables compression at level zero and uses Snappy at higher levels.

This makes it possible to apply different CPU-versus-space tradeoffs to freshly written data and longer-lived compacted data.

Compression affects storage footprint, disk movement, and decompression cost. It does not replace an application-level decision about how to serialize or compress logical records.

### 7.5 Bloom and cache-line filters

SurrealKV uses membership filters to rule out SSTables that cannot contain a requested key.

The default policy is a LevelDB-style Bloom filter. The code also exposes `RibbonFilter`, implemented as a cache-line-oriented blocked filter.

The intent of the blocked design is to concentrate probes in a small region of memory rather than performing numerous unrelated cache-line accesses.

The public `FilterPolicy` trait permits alternative membership-filter implementations.

A custom filter must preserve the essential property that false negatives are not allowed. A filter may report that a missing key might exist, but it must not rule out a key that is actually present.

Despite its name, the exposed `RibbonFilter` should not automatically be treated as equivalent to RocksDB's particular Ribbon-filter implementation. The verified source implements a cache-line-blocked bit-probe approach.

### 7.6 Block caching and indexes

The engine maintains a unified block cache for SSTable data and index blocks, and value-log data when enabled.

The cache is capacity-limited and sharded. The default native allocation is 32 MiB, with a smaller browser default.

The SSTable format supports partitioned indexes and filters, avoiding the need to load an entire large table index into memory.

`with_index_partition_size`, `with_block_size`, and `with_block_cache_capacity` provide useful control over index granularity, block I/O, and memory use.

Larger blocks can improve sequential access efficiency but increase wasted read/decompression work for small random lookups. Smaller blocks can reduce point-read amplification while increasing index metadata and lookup overhead.

### 7.7 Hardware-aware auto-tuning

The public `autotune` module offers:

- `HardwareProfile::detect()`
- `HardwareProfile::tune_options(options)`
- `auto_tune_options(options)`

It estimates an engine memory budget, then distributes capacity between memtables and block caching.

A significant caveat: the native hardware profile does not currently measure actual available RAM. It estimates memory from the number of available CPU cores, using a minimum estimate of 4 GiB.

The tuning algorithm then derives its memory allocations from that estimate.

As a result, this is a heuristic configuration helper, not an adaptive resource controller that continuously discovers the machine's actual free memory.

The separate `memory` module provides atomic memory accounting and budget calculations, but the presence of those APIs should not be taken as a guarantee of strict process-wide memory containment.

Configuration source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lib.rs`

Compaction source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/compaction/manual.rs`

Auto-tuning source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/autotune.rs`

A version-specific clarification to §7.1: SurrealKV 1.2 added value-size accounting to memtable capacity enforcement. In 1.3, `MemTable::used()` includes both arena allocation and heap-held values, and batches are capacity-checked before insertion. Thus large values do not simply accumulate without regard to the configured limit, although total process memory can still exceed one memtable's target because of other caches, buffers, immutable memtables, and in-flight work. Oversized batches have a separate direct-to-L0 path.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 8. Transparent encryption at rest

SurrealKV 1.x includes encryption at the storage-engine level rather than requiring callers to encrypt individual record values.

### 8.1 Supported encryption mechanisms

The two supported cipher suites are:

- AES-256-GCM
- XChaCha20-Poly1305

`CipherPreference::Auto` selects AES-256-GCM where the processor has suitable AES support, otherwise XChaCha20-Poly1305.

The choice is made when an encrypted database is created and persisted as part of its storage identity.

Encryption covers the manifest, SSTables, WAL, and value log. Checkpoints of encrypted databases likewise contain encrypted storage files.

Per-file encryption keys are derived from a root key, a random salt, and file-specific identity information. Authenticated encryption binds blocks to their database, file, and position, preventing incorrectly substituted blocks from being returned as valid data.

The Rust crate includes the `encryption` Cargo feature by default, but databases are not encrypted by default. Encryption must be configured explicitly when the database is created.

### 8.2 `KeyManager` extension point

The important extension mechanism is `KeyManager`, which separates storage encryption from the source and lifecycle of encryption keys.

Its principal operations are:

| Method            | Responsibility                                                      |
| ----------------- | ------------------------------------------------------------------- |
| `active_key_id()` | Identify the root key for new files                                 |
| `key_state(id)`   | Indicate whether a key is active, decrypt-only, retired, or unknown |
| `root_key(id)`    | Return the required 32-byte root key                                |
| `prepare(ids)`    | Make required keys available before storage operations need them    |

SurrealKV supplies `SoftwareKeyManager`, including support for reading root keys from protected files.

Custom implementations can integrate key-management services or hardware security modules.

A critical performance requirement is that `root_key`, `active_key_id`, and `key_state` must not perform slow remote operations. The engine can call them on its storage write path. Remote key retrieval belongs in `prepare`, with the results cached for subsequent use.

This lets an application integrate external key custody without putting network round trips into individual SSTable or WAL file creation.

### 8.3 Rotation and immutable-file lifecycle

Rotation changes which root key is active for newly created storage files.

Previously generated files remain encrypted under keys derived from the roots that were active when they were created. Therefore an old root key must remain available in decrypt-only state for as long as relevant files or checkpoints need it.

The engine exposes encryption-status reporting that identifies which key IDs cover which files and quantities of stored data.

There is no general promise that changing the active key immediately re-encrypts the complete existing database.

Two further constraints materially affect use:

- Encryption status is fixed at database creation. An existing plaintext database cannot simply be opened with encryption enabled, and the reverse is also prohibited.
- The encryption API is present in the public exports, but several related types and inspection methods are marked `#[doc(hidden)]` because their shape is not yet settled. They warrant additional compatibility scrutiny.

Finally, encryption at rest does not hide filenames, file sizes, or timing of filesystem activity.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/tde/keys.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/tde/context.rs`

## 9. Checkpointing, restoration, and lifecycle management

### 9.1 Consistent checkpoints

`Tree::create_checkpoint(path)` creates a consistent storage checkpoint.

The operation captures the required persistent structure, including SSTables, level metadata, WAL state, and value-log files when enabled.

It returns `CheckpointMetadata` describing the checkpoint.

Unlike merely copying a directory while commits are occurring, the engine coordinates checkpoint creation with mutable storage and background processing to produce a coherent snapshot.

This can eliminate substantial custom machinery for making internally consistent local backup copies.

However, a checkpoint is not a remote backup service or incremental replication protocol. Retaining, transferring, validating, and recovering independent backup copies remains a separate responsibility.

### 9.2 Restoring a checkpoint

`restore_from_checkpoint(path)` restores an existing tree to a checkpoint's state.

The implementation coordinates with commits, compaction, file rotation, active readers, and checkpoint activity, replacing the persistent and in-memory state.

This is deliberately more invasive than simply opening a historical read transaction. Restore changes the database's active state, potentially discarding changes that occurred after the checkpoint.

Checkpoint and restore operations use blocking filesystem I/O. Both are supported through the native filesystem path rather than the browser OPFS backend.

Checkpoint directories also cannot overlap the live database directory through direct or resolved paths.

### 9.3 Shutdown and error visibility

`Tree::close().await` provides a coordinated shutdown path.

The shutdown logic completes admitted commits and relevant ongoing maintenance before finishing. It is preferable to relying on the object being dropped, where graceful asynchronous close may not be possible.

SurrealKV exposes several operational health methods:

- `recovery_report()` — information about recovery performed when opening the database.
- `background_error()` — the latest reported background error.
- `background_error_count()` — number of background errors.
- `is_stopped()` — whether a fatal background failure has stopped the database.

These can be integrated into application-level health and fault-reporting facilities.

A serious storage failure may mean the database must be closed and reopened to establish the results of in-flight operations. Merely retrying a failed call on the same `Tree` is not always sufficient.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lsm.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/checkpoint.rs`

## 10. Inspection, diagnostics, telemetry, and tooling

### 10.1 Programmatic inspection

The `surrealkv::inspect` module exposes native filesystem inspection functions.

| Function family                          | Information available                                                                                   |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `inspect_db` / `inspect_db_with_options` | Storage layout, manifest version, last sequence, table counts and sizes, level distribution, lock state |
| `dump_manifest`                          | Level hierarchy, SSTable metadata, recorded snapshot information                                        |
| `dump_sst`                               | Table format, keys, blocks, sizes, checksums, and optional entry details                                |
| `dump_wal`                               | WAL segment information and decoded content where supported                                             |
| `scrub_db`                               | Integrity verification across SSTable blocks                                                            |
| `encryption_report`                      | File-level encryption identities, suites, and key usage                                                 |

The `_with_options` variants allow relevant configuration, including key managers, for inspecting protected databases.

Scrubbing distinguishes damaged blocks from blocks that could not be verified, for instance because an encrypted table could not be authenticated without the required key.

This is a meaningful capability for building storage diagnostics without manually decoding the on-disk structures.

### 10.2 `skv` command-line utility

The companion `surrealkv-cli` crate installs the `skv` executable.

```
cargo install surrealkv-cli

skv inspect path/to/db
skv manifest path/to/db
skv wal path/to/db

skv sst path/to/table.sst --verify-checksum
skv scrub path/to/db

skv get path/to/db "user:001"
skv scan path/to/db --prefix "user:"

skv enc-info path/to/db
skv migrate path/to/old-db path/to/new-db
skv bench path/to/test-db --keys 50000 --concurrency 8
```

The executable also supports `put` and `delete`, hexadecimal key and value representation, scan boundaries, keys-only scans, and limits.

For encrypted databases, `--key-file` or the `SKV_KEY_FILE` environment variable can supply the path to a key file. `enc-info` can report key coverage from storage headers without possessing the keys themselves.

These functions are useful for debugging data layout, diagnosing unexpected storage growth, checking corruption, and inspecting migration outcomes.

They are storage-engine tools, not a SurrealQL client, relational query debugger, or general SurrealDB administrative interface.

### 10.3 Telemetry and memory accounting

The public `telemetry` module contains `LatencyHistogram` and `FlightRecorder`.

Its primitives use atomic counters and exponential latency buckets for lightweight recording of events such as:

- Commit and point-read latency.
- WAL flush latency.
- WAL bytes written and SSTable bytes read.
- Block-cache hits and misses.

These are lower-level instrumentation primitives. Their presence should not be interpreted as a complete automatically integrated metrics service with an exporter, dashboards, or Prometheus endpoint.

Similarly, the `memory` module provides `MemoryController` for tracking portions of memory consumed by active and immutable memtables, caching, and ring buffers.

Operational integrations can use these structures or the library's `tracing` instrumentation, but the degree of automatic collection and externally accessible engine state is not uniform across all structures.

### 10.4 Deterministic simulation testing

SurrealKV includes a companion `surrealkv-sim` crate used for deterministic simulation and differential testing.

Its purpose is to compare storage-engine behavior with a simpler in-memory reference model under generated sequences of operations.

The upstream project reports a CI run involving 500 seeds and 5,000 steps per seed: 2.5 million modeled operations.

Combined with crash simulation of the in-memory filesystem, this testing approach can reveal concurrency, recovery, and state-transition errors that ordinary CRUD tests might miss.

It is an important indication of engineering methodology, but not proof that every crash, power-loss, filesystem, or concurrency failure mode has been exhaustively verified.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Sources:

`https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/inspect.rs`

`https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/telemetry.rs`

`https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/memory.rs`

`https://github.com/surrealdb/surrealkv/tree/v1.3.0/crates/surrealkv-sim`

## 11. Migration and compatibility

SurrealKV 1.0 introduced a substantially redesigned storage engine, making migration a consequential part of the current library.

### 11.1 SurrealKV 0.20/0.21 migration

The modern engine supports conversion of older SurrealKV databases, including versions through 0.21.4.

Migration preserves eligible key/value records, historical versions and timestamps, and data represented in unflushed WAL records.

Legacy files are moved into a backup subdirectory such as `_surrealkv_v0_backup/`.

An important limitation is that this is not a reversible format switch. The old backup files are not necessarily a complete rollback point, because storage components such as the value log may be shared or subsequently reclaimed.

A separate copy of the full database directory is required if rollback to an earlier engine is important.

### 11.2 RocksDB migration

The companion `surrealkv-compat-rocksdb` crate implements a Rust-based reader for RocksDB databases created by SurrealDB.

The documented migration supports RocksDB versions 8–11, including relevant WAL and blob-file structures.

For SurrealDB databases that used versioned storage, the conversion can preserve historical records and translate their hybrid logical clock timestamps to the new timestamp representation. For unversioned databases, it retains the latest key state.

The compatibility reader does not turn SurrealKV into a general-purpose RocksDB API replacement. Its migration support is targeted at the storage formats produced by SurrealDB.

RocksDB must not have the source database open while migration is performed.

### 11.3 Browser IndexedDB migration

The separate `surrealkv-compat-indxdb` integration supports migration of the legacy SurrealDB IndexedDB storage format into browser OPFS.

The browser API `migrate_from_indexeddb` reads legacy records and writes them into a new SurrealKV store.

The migration has several noteworthy behaviors:

- It processes data in bounded chunks rather than requiring the complete IndexedDB contents in memory.
- It checks the copied database before reporting success.
- Interrupted imports are detected using a marker and can be restarted.
- IndexedDB is left unchanged unless the application separately invokes `delete_indexeddb`.
- Records with empty keys are skipped and counted, because SurrealKV rejects empty keys.
- A destination database that already exists is not silently overwritten.

This is a specialized compatibility mechanism, not a general-purpose bridge between arbitrary IndexedDB object stores and SurrealKV.

### 11.4 Migration-related crates

| Package                    | Scope                                                  |
| -------------------------- | ------------------------------------------------------ |
| `surrealkv-compat-v1`      | Reader for legacy SurrealKV storage                    |
| `surrealkv-compat-rocksdb` | Reader for SurrealDB's older RocksDB databases         |
| `surrealkv-compat-indxdb`  | Reader and migration support for legacy IndexedDB data |
| `surrealkv-cli`            | Explicit migration commands and diagnostics            |

The latest release bundles these compatibility components at version 0.2.3.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/README.md`

Release: `https://github.com/surrealdb/surrealkv/releases/tag/v1.3.0`

## 12. Deployment targets and filesystem abstraction

### 12.1 Native deployments

SurrealKV is an embedded library. It operates within the application's process and uses a local persistent storage directory, rather than requiring a separate database daemon.

The current upstream support matrix is:

| Platform                             | Status                                        |
| ------------------------------------ | --------------------------------------------- |
| Linux x86-64                         | Tier 1, built and tested in CI                |
| macOS ARM64                          | Tier 1, built and tested in CI                |
| Linux ARM64                          | Tier 2                                        |
| macOS x86-64                         | Tier 2                                        |
| Windows x86-64                       | Tier 2                                        |
| WebAssembly `wasm32-unknown-unknown` | Tier 2, with dedicated Chrome browser testing |

The published Rust minimum supported version for SurrealKV 1.3.0 is Rust 1.86, according to its crate manifest.

The library depends on Tokio for parts of its runtime, but it is not a uniformly asynchronous key-value interface. Synchronous reads, scans, compaction, and checkpoint operations can perform blocking work.

Consequently, a Rust application's execution architecture still matters. The library does not automatically turn all filesystem work into nonblocking tasks.

### 12.2 Provided filesystem implementations

The public `FileSystem` abstraction supports:

| Backend                        | Primary use                                                  |
| ------------------------------ | ------------------------------------------------------------ |
| `FileSystem::std()`            | Native filesystem persistence                                |
| `FileSystem::memory()`         | In-memory database storage and testing                       |
| `FileSystem::opfs(name).await` | Persistent storage in a browser's Origin Private File System |

These are provided implementations. Although SurrealKV contains a substantial internal filesystem abstraction, its underlying `Fs` trait is not exposed as a stable general-purpose external plug-in interface.

Thus its current public API should not be interpreted as supporting arbitrary third-party object stores, distributed filesystems, S3 storage, or custom Rust filesystem adapters merely by implementing a public trait.

### 12.3 WebAssembly and OPFS

Browser support is an important newer capability.

A SurrealKV database can run in a dedicated Web Worker using OPFS as its storage backend. It retains the same broad LSM architecture: WAL, SSTables, manifest, compaction, and optional value-log separation.

A typical initialization sequence uses:

```
use surrealkv::{FileSystem, TreeBuilder};

async fn open_browser_tree() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystem::opfs("example").await?;

    let tree = TreeBuilder::new()
        .with_file_system(fs.clone())
        .with_path("db".into())
        .build_async()
        .await?;

    // Perform transactional operations here.

    tree.close().await?;
    fs.close().await?;
    Ok(())
}
```

Illustrative 1.3 API usage, not an executed example. Must run in a supported dedicated-worker environment.

This makes it possible to embed a transactional Rust storage engine directly into certain browser applications without routing every storage operation through IndexedDB.

However, browser support has substantial constraints:

Single-threaded execution. OPFS requires a dedicated worker. The library's browser execution is single-threaded, and synchronous maintenance operations can block that worker.

Exclusive ownership. A database can only be mounted by one worker or tab at a time. The `opfs_with_lock_wait` API provides control over waiting for the previous owner to release its lock.

Limited browser verification. Chrome is the tested browser. Firefox and Safari are not currently covered by the same upstream guarantees.

Durability differences. `Durability::Immediate` asks the browser to flush its storage handles, but the browser specification does not establish the same power-loss guarantee as an ordinary native filesystem with reliable synchronization.

Restricted functionality. Encryption, native checkpoint/restore, path-based inspection tools, and legacy RocksDB/SurrealKV migration are unavailable through OPFS.

Different physical format. The OPFS filesystem implementation uses a file-pool representation. Its files cannot currently be opened as an ordinary native SurrealKV database directory, and no general export mechanism is provided.

Storage limits and lifecycle. Browser quotas, origin eviction, fixed file-pool capacity, and worker shutdown behavior can all affect database availability. Applications should coordinate graceful `Tree` and `FileSystem` closure rather than depending on page-unload callbacks.

The source explicitly distinguishes tested worker termination from operating-system or power failure. Successful worker-restart tests do not prove power-loss durability in a browser.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/vfs/mod.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/README.md`

## 13. Performance characteristics and evidence

The most useful performance evidence is the upstream comparison published with SurrealKV 1.0. It measured several embedded storage engines using a common CRUD benchmark harness.

The hardware was an AMD Ryzen Threadripper 9970X, 128 GB RAM, and a 4 TB NVMe SSD, with 500,000 keys, 48 worker threads, and 128 clients. The SurrealKV test enabled value separation and used a larger-than-default memtable. Each reported figure was one run.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



### 13.1 Synchronous-durability comparison

Operations per second, as published by SurrealDB; measurements of 1.0, not a fresh 1.3 benchmark

| Operation   | SurrealKV | RocksDB | Relative performance |
| ----------- | --------- | ------- | -------------------- |
| Point reads | 6,804,864 | 855,851 | 7.95×                |
| Creates     | 229,376   | 8,568   | 26.8×                |
| Updates     | 221,341   | 36,263  | 6.10×                |
| Deletes     | 264,089   | 32,043  | 8.24×                |
| Range scans | 905,132   | 852,661 | 1.06×                |

Point-read throughput, synchronous durability

Published October 2026 benchmark, millions of operations per second

02468SurrealKVRocksDBSlateDB

The comparison shows that the performance advantage varies greatly by operation. In particular, the nearly eightfold point-read difference does not imply an eightfold benefit for every sequential scan or higher-level database query.

The upstream authors also explicitly reported that RocksDB's create results with synchronous durability were unusually low and not fully explained. That is an important qualification to the largest ratio.

### 13.2 Results without synchronous durability

When the benchmark did not require an `fsync` before acknowledgment, RocksDB was faster on writes:

| Operation | SurrealKV | RocksDB |
| --------- | --------- | ------- |
| Creates   | 321,715   | 603,452 |
| Updates   | 274,172   | 608,494 |
| Deletes   | 307,034   | 739,357 |

Point reads remained faster in the published SurrealKV benchmark, while LMDB led some sequential scan and full-count measurements.

The implication is that performance rankings depend materially on durability policy and workload shape.

### 13.3 Memory profile

In the publisher's non-synchronous benchmark, SurrealKV used approximately 221 MiB of resting memory and reached a peak of 1.79 GiB.

RocksDB used approximately 590 MiB resting and 1.02 GiB peak.

Thus SurrealKV had a smaller resting footprint but a larger observed peak under that workload.

This reinforces the need to distinguish per-component storage configuration from actual whole-process memory use under high concurrency.

### 13.4 Architectural explanation

Several performance mechanisms can account for advantages in appropriate workloads:

Commit grouping. Concurrent commits amortize WAL append and synchronization overhead.

Radix-tree memtables. Ordered key storage and arena allocation can reduce pointer chasing and allocation costs.

MVCC reads. Readers can capture a sequence-based snapshot without blocking writers.

Batch point reads. Version 1.3 avoids repeated tree-state capture for multiple related lookups.

Value separation. Large values need not be rewritten during every compaction.

Partitioned indexes and filters. Negative lookups can avoid unnecessary SSTable block reads.

The expected benefits depend on what dominates a workload: synchronization, memory access, compression, random I/O, compaction, or data movement.

These are architectural explanations, not guarantees of application-level speedup.

Benchmark methodology and results: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/README.md`

Engineering analysis: `https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0`

## 14. Architectural roles and integration boundaries

### 14.1 Direct embedded storage

SurrealKV can act as the primary persistent state layer of a Rust application when the application needs durable byte records, transactional updates, ordered access, and concurrent reads.

It can be especially valuable for state machines, local persistent caches, durable indexes, workflow state, or storage-backed metadata.

Its multi-key transactions can eliminate a separate custom write-ahead log, an application-side atomic batch protocol, and some snapshot-management machinery.

Its range tombstones, savepoints, and historical reads can further reduce application code.

However, the caller remains responsible for the semantic model of its records and keyspaces.

### 14.2 Underlying engine for SurrealDB

SurrealDB 4 uses SurrealKV as its persistent file-backed engine. The documented endpoint scheme is `surrealkv://`.

The older `rocksdb://` spelling remains accepted as a deprecated alias, with the database opened through SurrealKV and legacy data converted as necessary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The boundary between the two is critical:

| Capability                              | SurrealKV directly                  | SurrealDB layer                               |
| --------------------------------------- | ----------------------------------- | --------------------------------------------- |
| Durable byte records                    | Yes                                 | Uses underlying storage                       |
| Ordered key ranges                      | Yes                                 | Builds higher-level access paths              |
| Multi-key atomic transactions           | Yes                                 | Adds database-level semantics                 |
| Historical key versions                 | Optional native capability          | May expose higher-level temporal models       |
| Document schemas and validation         | No native schema engine             | Yes                                           |
| SurrealQL                               | No                                  | Yes                                           |
| Graph relationships and graph traversal | No native query engine              | Yes                                           |
| Secondary and composite indexes         | No native declarative index DDL     | Yes                                           |
| Full-text and vector search             | No native query indexes             | Yes                                           |
| Authentication and permissions          | No database user/role system        | Yes                                           |
| Network database protocol               | No                                  | Yes                                           |
| Distributed multi-node coordination     | Not a standalone distributed engine | Provided by appropriate SurrealDB deployments |

SurrealKV's sorted SSTable indexes and membership filters are internal storage indexes. They are not substitutes for SurrealDB's declarative secondary indexes or its full-text and vector indexing facilities.

Similarly, SurrealKV's historical versioning does not automatically provide SurrealDB record history, event sourcing, audit semantics, or a complete temporal query language.

The storage engine provides primitives from which some of those facilities can be built.

### 14.3 Possible native capability combinations

Several combinations are particularly interesting at the architectural level.

Transactional state with history. MVCC, explicit historical timestamps, and atomic multi-key writes can support records with an auditable sequence of prior states, provided retention and deletion policies are selected to preserve the required information.

Range-oriented lifecycle management. Ordered composite keys, prefix scans, and range tombstones can simplify partition cleanup, namespace expiration, and other bulk state transitions.

Read-dependent state updates. `get_for_update`, ordinary writes, savepoints, and conflict retries can support conditional updates without requiring an external transaction coordinator, provided the application's invariants fit the engine's snapshot-validation model.

Large-value storage. Value separation, compression, and caching can support stores where metadata keys are frequently scanned but larger payloads are read selectively.

Consistent local recovery. WAL replay, checkpoint creation, restore coordination, and diagnostics can reduce the amount of custom infrastructure needed to make a local embedded state store recoverable.

These compositions offer useful options, but none changes the fundamental single-engine scope. Replication, cross-process transaction coordination, sophisticated query planning, and distributed consensus are not supplied by combining low-level storage APIs.

### 14.4 Important constraints

SurrealKV is less suitable as a direct solution when the dominant requirement is:

- Serializable execution across arbitrary read predicates rather than snapshot isolation.
- Concurrent writes from multiple independent processes into the same storage directory.
- Native distributed transactions or network replication.
- Declarative relational, graph, full-text, or vector queries without an additional data-management layer.
- Browser persistence with native-equivalent power-loss guarantees.
- Arbitrary filesystem-provider extensibility through a stable public Rust trait.
- Stable, independently portable on-disk formats across all environments and versions.

These limitations reflect the library's intended scope, rather than missing conveniences that can be assumed to exist somewhere in its internal modules.

## 15. Version history and current maturity

| Release       | Date                | Significant development                                                                                                                     |
| ------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.21.x        | Before October 2026 | Earlier storage-engine architecture, including mechanisms no longer representative of 1.x                                                   |
| 1.0.0         | October 13, 2026    | New LSM engine, commit ring, snapshot isolation, temporal storage, encryption, migration                                                    |
| 1.1.0 / 1.1.1 | October 26, 2026    | Intermediate 1.x maintenance releases                                                                                                       |
| 1.2.0         | November 2, 2026    | Recovery and write-stall hardening, value-aware memtable bounds, cache cleanup, commit-validation optimizations, optional WAL preallocation |
| 1.3.0         | November 6, 2026    | Batched point reads and fixes to pooled-filesystem durability behavior                                                                      |

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

A substantial documentation issue remains: older Context7 and indexed Rustdoc material describes the 0.21 architecture, including historical-version mechanisms that should not be carried forward uncritically into 1.3.

For current work, use the tagged 1.3 source and release notes wherever they disagree with material written for 0.21.

There are also meaningful maturity distinctions within 1.3 itself:

- Native transactional storage, WAL recovery, compaction, and historical reads are released capabilities.
- Browser OPFS storage is implemented and tested in Chrome, but remains subject to the stated platform restrictions.
- Encryption exists as released functionality, but related APIs and on-disk formats carry explicit stability cautions.
- Offline scrubbing is implemented; continuous background scrubbing is not.
- Some public modules expose instrumentation or internal mechanisms without implying a complete externally supported management interface.
- The project reports extensive deterministic simulation testing, while the published performance benchmarks remain upstream, workload-specific measurements.

The on-disk encryption documentation explicitly warns that formats are not fully frozen. That caution is more important than treating the 1.x version number as a universal compatibility guarantee.

## 16. Practical Rust API illustration

The following concise example demonstrates ordinary writes, a transaction with immediate durability, ordered scanning, and historical access. It is illustrative and has not been compiled or executed here.

```

use surrealkv::{
    Durability, HistoryOptions, LSMIterator,
    Mode, TreeBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tree = TreeBuilder::new()
        .with_path("./surrealkv_data".into())
        .with_versioning(true, 0)
        .build()?;

    // Atomic write with immediate durability
    let mut write = tree.begin()?;
    write.set_durability(Durability::Immediate);
    write.set_at(b"observation:001", b"first", 1_000)?;
    write.set_at(b"observation:002", b"second", 2_000)?;
    write.commit().await?;

    // Consistent read-only snapshot
    let read = tree.begin_with_mode(Mode::ReadOnly)?;

    // Batched point reads (introduced in 1.3)
    let keys = [
        b"observation:001".as_slice(),
        b"observation:002".as_slice(),
    ];
    let values = read.get_many(&keys)?;
    assert_eq!(values.len(), 2);

    // Ordered range scan
    let mut cursor = read.range(
        b"observation:000",
        b"observation:999",
    )?;

    cursor.seek_first()?;
    while cursor.valid() {
        let key = cursor.key().user_key().to_vec();
        let value = cursor.value()?;
        println!("{key:?} = {value:?}");
        cursor.next()?;
    }

    // Retrieve historical versions
    let options = HistoryOptions::new()
        .with_tombstones(true)
        .with_ts_range(0, 3_000)
        .with_limit(100);

    let mut history = read.history_with_options(
        b"observation:000",
        b"observation:999",
        &options,
    )?;

    history.seek_first()?;
    while history.valid() {
        println!(
            "timestamp = {}",
            history.key().timestamp()
        );
        history.next()?;
    }

    drop(history);
    drop(cursor);
    drop(read);

    tree.close().await?;
    Ok(())
}

```

This example intentionally leaves out application-level key encoding, serialization, retries, backup management, and operational instrumentation. Those are separate responsibilities rather than intrinsic requirements of using the basic API.

## 17. Primary source reference map

The following references are especially useful for further technical investigation.

| Area                                           | Primary source                                                            |
| ---------------------------------------------- | ------------------------------------------------------------------------- |
| Repository, features, benchmarks               | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/README.md`            |
| Current releases                               | `https://github.com/surrealdb/surrealkv/releases`                         |
| Crate registration                             | `https://crates.io/crates/surrealkv`                                      |
| Rust public exports and configuration          | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lib.rs`           |
| Transactions, reads, history, savepoints       | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/transaction.rs`   |
| Tree lifecycle, compaction, checkpoints        | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lsm.rs`           |
| Commit pipeline                                | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/ring/pipeline.rs` |
| Memtable architecture                          | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/memtable/mod.rs`  |
| Storage inspection                             | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/inspect.rs`       |
| Encryption key management                      | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/tde/keys.rs`      |
| Filesystem and browser integration             | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/vfs/mod.rs`       |
| Error semantics                                | `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/error.rs`         |
| Official architecture and benchmark discussion | `https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0`  |
| SurrealDB persistent-storage integration       | `https://surrealdb.com/docs/running/file-backed`                          |

## Overall assessment

SurrealKV 1.3.0 is materially more capable than a simple embedded key-value map with disk persistence.

Its most significant strengths are the combination of snapshot-isolated multi-key transactions, asynchronous group commit, optional persistent time-travel history, efficient ordered iteration, range deletion, value separation, configurable durability, and native checkpoint/recovery mechanisms.

These provide a substantial set of infrastructure primitives for building persistent stateful Rust systems without implementing a transaction manager, WAL, storage compactor, historical-version index, or crash-recovery protocol independently.

Its principal architectural tradeoff is that the storage engine remains deliberately low-level. It provides strong mechanisms for representing and updating state, but does not supply the semantic data model, distributed execution, query planner, or higher-level indexing functionality of a complete database.

The transition from 0.21 to 1.0, followed rapidly by the 1.2 and 1.3 durability and concurrency improvements, also means that version-pinned source verification is unusually important. Current 1.x behavior should not be inferred from older SurrealKV documentation, and storage-format compatibility deserves particular attention.

For workloads that primarily need local transactional persistence, ordered keyspace operations, and controllable history, SurrealKV provides a rich native foundation. For workloads whose complexity lies primarily in data relationships, queries, secondary indexing, or distributed coordination, its greatest value is as an underlying engine rather than as a replacement for those higher-level facilities.

# SurrealKV 1.3.0 — Technical Addendum

Research date: November 10, 2026 | Target: SurrealKV 1.3.0 | Companion to the comprehensive library overview

This addendum documents advanced SurrealKV functionality, implementation details, and semantic distinctions identified in the subsequent gap analysis. It also corrects several points in the original reference that could lead to incorrect API usage or architectural assumptions.

The scope is the released SurrealKV 1.3.0 implementation, confirmed as the latest published release on November 10, 2026. The upstream source tagged `v1.3.0`, its tests, and release notes take precedence over older Context7 material describing the 0.21 storage engine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 1. Transaction execution beyond ordinary memtable writes

### 1.1 Direct-to-L0 writes for oversized transactions

SurrealKV has a specialized write path for transactions whose encoded batch is too large to fit into a memtable.

The normal commit path accumulates mutations in the transaction write set, validates them, records the transaction in the WAL, and applies the accepted batch to a mutable memtable. Once a memtable fills, it is rotated and subsequently flushed to an SSTable.

An oversized batch creates a different problem: if the batch cannot fit in an empty memtable, rotating memtables cannot resolve the condition.

SurrealKV handles this by allowing the commit pipeline to write the batch directly into a Level 0 SSTable.

Relevant internal mechanisms include:

- `CommitPipeline::write_direct_to_l0`
- `CoreInner::write_batch_direct_to_l0_sst`
- `ApplyStop::Oversized`
- `ApplyStop::ArenaFull { empty: true }`

The commit pipeline distinguishes a memtable that merely lacks remaining capacity from a batch that would not fit even after rotation.

In the former case, it rotates the memtable and retries insertion. In the latter, it takes the direct-to-L0 path.

This is more than a large-value optimization. It allows the engine to accommodate certain large atomic write batches without imposing the configured memtable size as a hard maximum transaction size.

The path remains part of the coordinated commit and recovery machinery. It does not bypass the engine's transaction semantics simply because the batch is written directly into persistent table storage.

There are several important limits:

WAL representation remains bounded. `Error::BatchTooLarge` is returned when an encoded transaction batch exceeds the WAL record representation's `u32::MAX` size limit. Memtable bypass does not remove that limit.

Direct insertion has different write costs. Constructing an SSTable requires sorted table generation, filesystem operations, and manifest changes. This avoids holding an oversized batch in a memtable but is not necessarily faster than ordinary buffered writes.

The path changes compaction pressure. Direct insertion adds L0 storage, potentially increasing overlap and triggering subsequent compaction.

Memory requirements are not eliminated. The transaction write set, encoded batch, and table-construction machinery can still consume substantial memory.

The capability is particularly useful for understanding why SurrealKV can accept batches larger than its nominal memtable capacity, and why write performance may change discontinuously for unusually large transactions.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/ring/pipeline.rs`

### 1.2 Commit-group sizing and write amplification

SurrealKV also limits how many accepted transactions its flusher gathers into one commit group.

The internal `MAX_GROUP_BYTES` constant is 4 MiB in version 1.3. This is a commit-group aggregation target, not a maximum individual transaction size. An individual large batch can be handled separately.

The flusher additionally accounts for estimated memtable consumption through `group_est_budget(max_memtable_size)`.

This budget has a consequential recovery and write-amplification purpose.

When a commit group spans several memtable rotations, its transactions may need to be reconciled with WAL segment changes. Without bounding the group's estimated memory requirements, repeated rotation and re-logging of outstanding batches can cause unexpectedly high WAL write amplification.

The implementation's comments describe an earlier pathological case where accumulated small commits resulted in repeated rotations and approximately 40-fold WAL rewriting.

The group budget helps constrain the number of rotations and the amount of re-logging that can arise from one group.

Thus the commit pipeline manages several different resource quantities:

| Quantity                         | What it controls                                               |
| -------------------------------- | -------------------------------------------------------------- |
| Encoded WAL batch size           | Whether an individual transaction can be represented           |
| Commit-group byte budget         | How many transactions can be aggregated                        |
| Memtable size estimate           | Whether batches can fit into mutable storage                   |
| Direct-to-L0 path                | How to handle batches exceeding memtable capacity              |
| WAL segment availability         | Whether accepted operations can be safely logged and recovered |
| L0 and immutable-memtable counts | Whether writers must be paced or stalled                       |

This is a useful distinction when analyzing high-concurrency write performance. A single configuration such as `max_memtable_size` does not govern all ingestion limits or buffering decisions.

### 1.3 Proactive pacing and hard write stalls

The original overview described write stalls as backpressure that occurs when the number of immutable memtables or L0 SSTables reaches a threshold.

The implementation also includes proactive write pacing before certain hard thresholds are reached.

`WriteStallController` maintains separate soft and hard thresholds for immutable memtables and L0 files.

When the storage system approaches a hard threshold, it can introduce short asynchronous delays proportional to the excess above the soft threshold.

At the hard threshold, the writer waits for flushing or compaction to reduce pressure.

This creates a two-stage mechanism:

1. Gradually moderate write throughput while storage pressure is increasing.
2. Stop admitting further writes at the relevant hard limit until maintenance makes progress.

The actual soft thresholds depend on configured hard limits. With the default memtable stall limit of two, there is effectively no separate pre-stall interval for immutable memtables; the L0 threshold provides a more meaningful pacing interval.

The controller also requests relevant background work when entering a hard stall. This avoids relying solely on a previously issued flush or compaction notification.

The result is a coordinated feedback mechanism rather than a simple global writer lock.

The engine exposes `WriteStallInfo`, including the reason, triggering count, threshold, and duration. The controller also defines `WriteStallCounts`-related structures and interfaces internally, although these should not be confused with a fully exposed runtime policy plug-in.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/stall.rs`

## 2. Temporal storage and history semantics

### 2.1 Range tombstones establish history-erasing boundaries

An especially important property of `Transaction::delete_range(start, end)` is that it does more than hide currently visible values.

In a versioned database, a committed range deletion logically erases the prior history of keys within the half-open interval `[start, end)`.

This is consistent with the semantics of an individual hard delete, rather than a collection of soft deletes.

Consider these committed versions:

| Key | Timestamp | Value |
| --- | --------- | ----- |
| `a` | 100       | `a1`  |
| `b` | 100       | `b1`  |
| `b` | 200       | `b2`  |
| `c` | 100       | `c1`  |
| `d` | 100       | `d1`  |

Suppose a transaction commits:

```
tx.delete_range(b"b", b"d")?;
tx.commit().await?;
```

Afterwards, `get_at(b"b", 150)` and `get_at(b"b", 250)` both return `None`, even with unlimited historical retention.

The history of `c` is similarly erased, while `a` and `d` remain unaffected.

A subsequent write to `c` establishes a new readable version, without reinstating the earlier history.

The upstream tests verify this behavior across unflushed memtables, SSTables, and compaction.

This gives SurrealKV two distinct temporal-deletion models:

- Soft delete: Creates a timestamped state transition while retaining eligible earlier versions.
- Hard delete or range delete: Invalidates previous history for the affected key or interval.

For workloads requiring historical reconstruction, this distinction is fundamental. A range tombstone should not be treated as equivalent to individually issuing soft deletes for every key.

The operation remains O(1) at the logical insertion stage, but preserving correct visibility and eventually reclaiming underlying storage requires additional work during reads and compaction.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/test/versioning_tests.rs`

### 2.2 Point-in-time lookup is not point-in-time range scanning

The historical API is asymmetric.

`get_at(key, timestamp)` retrieves the applicable value for one key at a requested time. `get_many_at(keys, timestamp)` extends that operation to a known collection of keys.

`history(start, end)` and `history_with_options` instead enumerate historical versions in key order, including potentially multiple versions of each key.

There is no separate public `scan_at` or `range_at` method in version 1.3 that directly returns one historically applicable value per key over an arbitrary range.

This matters because two tasks that might appear identical at the query level require different operations:

Historical point lookup: The engine selects the applicable version internally.

Historical range reconstruction: The caller must inspect historical versions and select the applicable state for each key.

A history iterator with timestamp filtering is not automatically equivalent to a snapshot at the end of that interval.

For example, requesting only versions with timestamps between 200 and 300 can omit a value written at 100 that remained valid throughout that period.

To reconstruct state at time 300, the relevant earlier version may be necessary even though its timestamp lies outside the requested interval.

Likewise, soft-deletion tombstones must be considered when determining whether a key exists at the selected time.

The historical-iteration interface supports these operations at the storage level, but does not provide a declarative temporal range-query planner.

### 2.3 Historical ordering, commit order, and equal timestamps

SurrealKV's temporal model distinguishes internal sequence numbers from application-visible timestamps.

Internal sequence numbers determine transaction snapshot visibility and the ordering of committed changes.

Historical timestamps determine which version applies to a requested time.

These orderings may differ because `set_at` permits an explicitly supplied timestamp, including one earlier than previously committed versions.

Within the history iterator, keys are ordered by key, and versions for a key are ordered by descending timestamp. Where multiple writes have the same timestamp, newer writes take precedence.

The implementation also handles uncommitted writes that have not yet received their commit timestamp. These are represented at `u64::MAX` during history iteration until committed.

An explicit timestamp of zero is a sentinel requesting the commit timestamp; it is not interpreted as a literal historical event at Unix time zero.

These details affect whether history can be interpreted as a business-event chronology, a transaction chronology, or a correction history. SurrealKV supplies the versioning machinery, but it does not enforce a particular interpretation of application-supplied times.

### 2.4 Logical-clock implementation

The internal `DefaultLogicalClock` is designed to produce monotonically increasing `u64` timestamps while limiting the cost of obtaining the current time.

On native platforms, it combines an estimated Unix timestamp with a monotonic elapsed-time source, periodically resynchronizing the reference to system time.

An atomic compare-and-swap loop ensures that a timestamp returned by the clock is greater than its previously issued value, even when the current wall-clock estimate has not advanced sufficiently.

Browser builds use a different path because they cannot rely on the same background clock thread.

The relevant distinction is monotonicity within the clock implementation, not a documented globally synchronized timestamp oracle across independent databases or machines.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/clock.rs`

## 3. Concurrency, commit failure, and transaction lifecycle

### 3.1 Snapshot validation is not predicate locking

SurrealKV's optimistic concurrency control primarily validates conflicts involving concrete keys.

`Transaction::get_for_update` adds a key to the commit-time validation set. A concurrent transaction that commits a write to that key after the reader's snapshot can cause the reader's commit to fail with `TransactionWriteConflict`.

However, this mechanism does not establish exclusive ownership of the key. Two transactions can both call `get_for_update` on the same key without conflicting merely because they read it.

Furthermore, key-based validation is not equivalent to predicate locking.

Consider two transactions enforcing a condition over every key within a range. They might independently read the same snapshot, then write different keys. Without a shared written record or equivalent conflict dependency, they can both succeed while violating a cross-key invariant.

The same caution applies to assumptions about concurrent range deletions and writes. The existence of range tombstones does not establish a general serializable range-locking facility.

The important architectural boundary is that SurrealKV provides snapshot isolation plus explicit key-level validation. It does not expose a generalized serializable predicate transaction mode.

### 3.2 A commit attempt is terminal

`Transaction::commit()` permanently closes the transaction when an attempt begins, before awaiting the commit pipeline.

The implementation does this deliberately so that a cancelled commit future cannot simply be resumed or submitted again with the same transaction state.

After a successful or failed commit attempt, subsequent operations on that transaction return `TransactionClosed`.

This creates a distinction between retrying an operation and retrying the same transaction object.

A write conflict generally requires starting a new transaction, rereading relevant state, reconstructing changes, and attempting another commit.

A commit whose outcome is indeterminate requires a different response: the caller cannot necessarily assume that the transaction had no effect.

### 3.3 Commit failure categories

SurrealKV's error types distinguish several materially different conditions.

| Error                      | Meaning                                                                                    | Consequence                                                              |
| -------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| `TransactionWriteConflict` | Conflicting write detected during optimistic validation                                    | A new transaction can attempt the logical operation again                |
| `TransactionRetry`         | Conflict history is insufficient to validate an older transaction                          | Reconstruct the operation using a fresh snapshot                         |
| `PipelineStall`            | The commit pipeline cannot currently proceed                                               | Does not by itself establish that an interrupted operation had no effect |
| `DatabaseStopped`          | A serious failure stopped the database, potentially after partial commit-group application | Affected commit outcomes may require reopening and recovery to establish |
| `DatabasePoisoned`         | The engine has already been stopped by an earlier failure                                  | The new operation was not started                                        |
| `ManifestWriteUncertain`   | Manifest replacement reached a stage where the durable outcome is uncertain                | Storage state cannot be inferred solely from the returned error          |
| `RestoreInterrupted`       | A restore began modifying the database but did not complete                                | The directory may contain an incomplete replacement state                |

These distinctions matter when a storage transaction has consequences outside SurrealKV.

For example, an application may issue a transaction that records completion of an external action. If the commit returns an indeterminate error, blindly repeating that external action can create duplicates.

SurrealKV supplies detailed storage-state errors, but it cannot resolve the correctness of external side effects. Applications may need operation identifiers, idempotency, or reconciliation when an effect can outlive the task that initiated it.

### 3.4 Commit-ring retirement and conflict-history garbage collection

An advanced part of the commit pipeline is its conflict-history retirement mechanism.

Each writable transaction registers an active-transaction guard and captures a commit window. The commit ring retains information needed to determine whether intervening transactions wrote overlapping keys.

The pipeline can eventually retire completed entries that no longer need to be examined by active transactions.

This involves several internal mechanisms, including active-transaction tracking, ring sequence numbers, retirement watermarks, overflow entries, and incremental reclamation of retired entries.

The design balances two requirements:

Correctness: A transaction must not lose access to the conflict information required to validate its snapshot.

Bounded resource use: The engine should not retain every completed transaction's conflict metadata indefinitely.

If a transaction becomes too old relative to the available validation history, SurrealKV can return `TransactionRetry` rather than guessing that the transaction is conflict-free.

This is distinct from MVCC's retention of historical key values. The commit ring retains information for conflict detection, while the LSM retains data needed by readers and historical queries.

The two mechanisms have different reclamation conditions.

### 3.5 WAL recovery can decide the outcome of failed commits

The commit pipeline takes care not to publish a partially applied commit group as successfully visible.

When a group fails after some data has been applied, SurrealKV can stop further commits and preserve the WAL records needed for recovery.

On reopening, recovery determines which committed records are restored and which incomplete operations are discarded.

In certain failure scenarios, a commit that returned an error may nevertheless become effective following recovery if its records reached the WAL. Conversely, a failure before a batch was accepted may have no durable effects.

This distinction is why the engine cannot promise that every failed commit is equivalent to a rollback.

The WAL recovery and commit-group healing code also coordinates segment replacement, preservation of segments needed by outstanding batches, and cleanup of unapplied groups.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/ring/pipeline.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/error.rs`

## 4. Read-path and iterator API distinctions

### 4.1 Asynchronous point reads are an execution option, not a different consistency model

`Transaction::get_async` provides a point-read operation whose result follows the same transaction visibility rules as `get`.

It checks the transaction's pending writes and range deletions before consulting the stored snapshot.

The primary distinction is how blocking storage work can be scheduled.

SurrealKV uses `affinitypool` for selected I/O operations when a suitable worker pool has been installed by the hosting application.

Without that integration, the relevant work may execute inline.

The library does not use Linux `io_uring` for this path.

Consequently, `get_async` should not automatically be assumed to provide nonblocking operating-system I/O or lower latency than `get`.

It can instead be valuable for isolating blocking point reads from an application's asynchronous task execution.

This also differs from `get_many`: batched reads reduce repeated capture of tree state, whereas `get_async` principally changes the execution context for individual reads.

No general-purpose asynchronous range iterator is exposed by the version 1.3 public API.

### 4.2 Correction: transaction-level `seek` accepts raw user keys

The previous overview incorrectly generalized the generic `LSMIterator` trait's encoded-key requirement to transaction iterators.

At the generic trait level, `seek` is documented in terms of encoded internal keys.

However, `TransactionRangeIterator::seek` and `TransactionHistoryIterator::seek` accept raw user-key bytes and construct the required internal seek representation themselves.

For example:

```
use surrealkv::LSMIterator;

let mut iter = tx.range(b"a", b"z")?;

iter.seek(b"m")?;

while iter.valid() {
    let key = iter.key().user_key();
    let value = iter.value()?;

    // Process key and value.

    iter.next()?;
}
```

The transaction iterator handles the internal sequence-number trailer required for positioning.

The distinction is between a generic low-level iterator contract and the behavior of the particular public iterator implementation returned by transaction operations.

This correction should replace the earlier guidance suggesting that callers must encode user keys before seeking these transaction iterators.

### 4.3 Encoded values versus resolved values

The `LSMIterator` interface exposes several ways to access a value, but they are not semantically identical.

| Method                     | Behavior                                                                         |
| -------------------------- | -------------------------------------------------------------------------------- |
| `value_encoded()`          | Retrieves raw stored bytes, which can contain a value-log pointer representation |
| `value_ref()`              | Resolves the logical value and returns `Cow<[u8]>`                               |
| `value_into(&mut Vec<u8>)` | Copies the resolved value into a caller-supplied reusable buffer                 |
| `value()`                  | Returns an owned logical value                                                   |

`value_ref()` can return borrowed bytes for inline values or values already held in the transaction's write set.

When a value must be retrieved from the value log, it may return owned bytes instead.

This distinction is particularly important when optimizing scans involving many values.

A borrowed encoded value is not necessarily equivalent to the application's actual value. Code operating on `value_encoded()` must understand whether the storage representation includes metadata or an external value reference.

`value_into()` provides an alternative when the caller needs owned data but wants to reuse allocation capacity across iterator steps.

Thus SurrealKV exposes several opportunities to control copying without forcing all scans through the same owned-buffer interface.

### 4.4 Iterator direction changes and re-anchoring

The lower-level iterator trait includes two unusual methods:

`reanchor_first()` and `reanchor_last()`

These exist because a merged iterator can contain multiple child iterators, and changing traversal direction may require repositioning a child that is no longer valid.

A normal `seek_first` or `seek_last` expresses a new traversal. By contrast, re-anchoring expresses continuation of the traversal already in progress.

This distinction is consequential when the iterator maintains state beyond its current position.

For example, a history iterator with a result limit must preserve the number of entries it has already returned when an internal direction change requires repositioning.

If repositioning incorrectly resets that limit, the traversal could return more entries than allowed.

SurrealKV's `HistoryIterator` overrides re-anchoring behavior to preserve its result budget.

This is primarily an internal correctness mechanism rather than an application feature, but it explains why the iterator abstraction is more sophisticated than an ordinary bidirectional cursor.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lib.rs`

### 4.5 `get_with_options` currently ignores `ReadOptions`

SurrealKV exposes:

```
tx.get_with_options(key, &options)
```

In version 1.3, the parameter is named `_options` in the implementation and is not used.

The method behaves as an ordinary point read; supplying lower and upper iteration bounds does not change the lookup.

Those bounds are meaningful in `range_with_options` and related range-iteration operations.

This is an example of an API whose signature suggests greater functionality than its current implementation provides.

The documentation should not imply that `get_with_options` currently supports configurable point-read policies, caching modes, timestamp selection, or other read behavior.

### 4.6 Genuine prefix scans require bounded key intervals

A range expression such as:

```
tx.scan(b"order:"..)?
```

starts at `order:` but continues through every subsequent key in the comparator's order.

It is not automatically constrained to keys beginning with `order:`.

For a true bytewise prefix scan, an exclusive upper bound can be constructed by incrementing the last non-`0xFF` byte of the prefix and removing the following bytes.

For example, the bytewise prefix `order:` has an exclusive upper boundary of `order;`.

The scan can then operate over the half-open range `[b"order:", b"order;")`.

When the prefix consists entirely of `0xFF` bytes, no finite higher bytewise-prefix boundary exists, and the scan must use an unbounded upper range or another termination condition.

Custom comparators can change the relationship between byte prefixes and key order. Any prefix-boundary algorithm must therefore be consistent with the configured comparator.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/transaction.rs`

## 5. SSTable encoding and extensibility

### 5.1 Prefix compression within SSTable blocks

In addition to optional Snappy or Zstandard block compression, SurrealKV uses key-prefix encoding within SSTable data blocks.

An entry stores the number of key bytes shared with the preceding entry, its remaining key suffix, and its value.

This reduces repeated storage of common key prefixes.

For example, consecutive keys such as `customer:000001`, `customer:000002`, and `customer:000003` share most of their bytes.

Instead of storing the complete key every time, the block can encode the shared prefix length and only the changing suffix.

The relevant entry representation is conceptually:

```
shared_prefix_length
unshared_suffix_length
value_length
key_suffix
value
```

However, prefix encoding introduces a problem for random seeks: decoding a key can depend on the preceding key.

SurrealKV addresses this with restart points.

At a restart point, the complete key is stored without reference to the preceding entry. A restart-point index allows an iterator to locate a nearby starting position and reconstruct only a limited sequence of keys.

The public configuration option is:

`Options::with_block_restart_interval(interval)`

The default is 16 entries between restarts.

Smaller intervals increase the frequency of complete keys, which can increase block size but reduce reconstruction work.

Larger intervals increase opportunities for prefix compression while potentially requiring more sequential decoding within a block.

This mechanism is particularly relevant when the workload uses long structured keys with repeated prefixes.

It operates independently of general-purpose block compression; both mechanisms can reduce stored bytes, but through different techniques.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/sstable/block.rs`

### 5.2 The full custom comparator contract

The `Comparator` trait exposes four operations:

| Method                | Responsibility                                     |
| --------------------- | -------------------------------------------------- |
| `compare(a, b)`       | Define ordering of user keys                       |
| `separator(from, to)` | Construct a valid separator between key boundaries |
| `successor(key)`      | Construct a key greater than the supplied key      |
| `name()`              | Identify the comparator implementation             |

A custom comparator can make a keyspace follow an ordering other than the default bytewise order.

The advanced requirement is that separator and successor operations must remain consistent with the comparator's ordering.

For example, a separator must not cause an SSTable index to omit keys that would otherwise fall within a lookup interval.

Similarly, changing the comparator used to interpret an existing on-disk database can invalidate assumptions embedded in SSTable ordering and index boundaries.

A comparator should therefore be treated as part of the persistent keyspace design, not a casually interchangeable runtime policy.

The implementation's `name()` identifies the comparator, but the presence of that method should not be treated as proof that every comparator change is detected or automatically migrated.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/comparator.rs`

### 5.3 Hash-based membership-filter construction

The `FilterPolicy` extension interface supports more than building a filter from complete keys.

It provides `key_hash` and `create_filter_from_hashes`, allowing an SSTable builder to retain compact hash values instead of keeping complete raw keys for the entire filter partition.

This can reduce temporary memory consumption during filter construction, particularly when keys are large.

The contract is strict: creating a filter from the hash of each key must produce the same encoded filter as creating it directly from those keys.

The `filter_keys_per_partition` method additionally estimates how many keys can fit within a target filter-partition size.

That helps the table builder partition membership filters without using one constant memory allocation strategy for every possible filter implementation.

A custom policy must also preserve the usual membership-filter guarantee: false positives are permitted, but false negatives are not.

A false negative can cause the storage engine to skip an SSTable containing the requested key, potentially returning an incorrect result.

Finally, SurrealKV's `RibbonFilter` implementation uses cache-line-blocked membership probes. Despite its name, it should not be assumed to implement the precise same algorithm or representation as RocksDB's Ribbon filter.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/sstable/ribbon.rs`

### 5.4 Compression operates at multiple storage layers

The original overview discussed level-specific SSTable compression. A further distinction concerns compression of other persistent structures.

SurrealKV's public SSTable `CompressionType` exposes `None`, `SnappyCompression`, and `ZstdCompression`.

The WAL subsystem has its own compression representation and contains LZ4 support.

These are different storage components with different formats and configuration surfaces.

In particular, the WAL's internal LZ4 compression support should not be interpreted as a public `Options` method that enables LZ4 compression throughout the database.

For SSTables, `with_compression_per_level` also has an important inheritance rule: if the supplied vector contains fewer entries than the number of LSM levels, the final specified compression type is used for subsequent levels.

This permits concise policies such as uncompressed L0 storage with Snappy compression at every deeper level.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/wal/mod.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lib.rs`

## 6. Compaction scheduling and value-log reclamation

### 6.1 Background compaction scoring

SurrealKV's automatic leveled compaction strategy assigns scores to storage levels.

For L0, the score considers both the number of files relative to the compaction trigger and the total bytes relative to the base-level size.

For deeper levels, it considers the level's total bytes relative to its configured target size.

The engine then selects a level requiring compaction based on these scores.

There is an important exception: when L0 reaches its hard write-stall threshold, its compaction score is elevated to ensure that L0 compaction takes precedence over ordinary lower-level maintenance.

This prevents the compactor from repeatedly choosing a different overfull level while writers remain stalled waiting for L0 space.

The internal compaction strategy also supports several file-selection priorities:

`OldestSmallestSeqFirst`, `OldestLargestSeqFirst`, and `ByCompensatedSize`.

The default is `ByCompensatedSize`.

That strategy considers file size adjusted for the number of deletion records, prioritizing files where compaction may reclaim obsolete data.

However, a significant distinction must be preserved: although `CompactionPriority` is exported, the constructor for selecting a custom priority is internal and test-only in version 1.3.

There is no corresponding public builder option for arbitrary background compaction-priority selection.

Thus these variants describe implemented strategies, not three generally available application-level tuning modes.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/compaction/leveled.rs`

### 6.2 Reference floors protect value-log files

Value-log garbage collection must coordinate with the LSM's changing set of references.

SurrealKV tracks the oldest value-log file still referenced by persistent tables, active memtables, and immutable memtables awaiting flush.

It also accounts for in-flight commit groups whose values have been written to the value log but have not yet been registered in a memtable or SSTable.

This last detail is particularly important.

Without a protection mechanism for those unregistered values, a concurrent cleanup could remove a value-log segment while a commit was still in the process of publishing references to it.

SurrealKV maintains a separate unregistered reference floor associated with in-flight groups and combines it with the floor derived from registered storage structures.

A value-log file is eligible for reclamation only when the relevant references and pins no longer require it.

This is a useful example of native mechanisms that eliminate the need for applications to coordinate ordinary value-log segment lifetimes themselves.

It does not guarantee immediate reclamation when a large value is deleted. Obsolete values can remain until compaction and value-log cleanup establish that their containing files are no longer referenced.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lsm.rs`

## 7. Encryption key lifecycle

### 7.1 `Tree::prepare_keys()` and external key providers

The original overview covered the `KeyManager` abstraction but did not explain the engine-level method:

`Tree::prepare_keys()`

The method requests preparation of the active root key and other root keys needed by the database.

This matters when using an external key-management service or hardware-backed key provider.

The `KeyManager` contract separates potentially slow key preparation from critical-path key lookup.

`prepare()` can retrieve and cache the required root keys, while `root_key()`, `active_key_id()`, and `key_state()` are expected to respond without waiting for network or disk operations.

When the active key changes, `Tree::prepare_keys()` allows the database to prepare the new root before a subsequent storage operation needs it.

The same mechanism can help after a transient key-provider availability failure.

For example, a provider-backed manager might change its active key ID, retrieve the associated root from a remote KMS, cache it, and then permit the engine to create new encrypted storage files using that root.

This is different from requiring each storage-file creation to synchronously contact the external key service.

The method is implemented in 1.3 but marked `#[doc(hidden)]`, with upstream documentation explicitly indicating that its interface has not yet been finalized.

It is therefore a real capability but not one that should be presented as a fully stable, long-term public API.

### 7.2 Key rotation is not immediate database-wide re-encryption

A related distinction is that changing the active encryption key affects newly written files.

Existing SSTables, WAL segments, value-log files, and checkpoints remain dependent on the root keys under which they were created until their contents are rewritten or those files are removed.

This is why the `KeyState::DecryptOnly` state exists.

An old key cannot safely be retired merely because a newer key has become active.

`Tree::encryption_status()` and `inspect::encryption_report()` provide information about file coverage by key ID, supporting assessment of which historical keys are still required.

The `Tree` method and related types are also marked as not having fully settled public shapes.

This makes SurrealKV's encryption model more advanced than a single immutable encryption key, while retaining important key-lifecycle responsibilities outside the storage engine.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/tde/keys.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lsm.rs`

## 8. Memory, configuration, and observability boundaries

### 8.1 Hardware auto-tuning is an explicit heuristic

`autotune::auto_tune_options` and `HardwareProfile::tune_options` provide helpers for calculating initial memory-related configuration.

On native platforms, `HardwareProfile::detect` uses available CPU parallelism and estimates memory as approximately 1 GiB per detected core, with a 4 GiB minimum.

It does not discover the machine's actual installed or currently available physical RAM.

The resulting memory estimate is used to derive a proposed memtable size and block-cache capacity.

The tuning function is explicitly invoked with an `Options` value. It is not a continuously running adaptive storage controller.

This matters when the runtime environment has memory limits that differ substantially from the inferred core-based capacity, such as containers with constrained RAM allocations.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/autotune.rs`

### 8.2 `MemoryController` does not imply strict global memory enforcement

SurrealKV exposes a `MemoryController` with atomic counters for several memory categories, including active memtables, immutable memtables, the block cache, and commit-ring buffers.

It also provides `dynamic_cache_target`, which calculates a block-cache target based on remaining accounting budget.

However, inspection of the tagged 1.3 implementation did not identify calls to `dynamic_cache_target` outside its module.

The method's existence therefore does not establish that SurrealKV continuously resizes the live block cache whenever its write buffers grow.

Likewise, `MemoryController::total_used()` is a sum of tracked categories. It is not necessarily equivalent to the process's total resident memory.

The implementation's comments describe stronger memory-budget goals than should be inferred from the verified integration.

It is more accurate to describe SurrealKV as having memory-accounting machinery, configurable memtable and cache budgets, and storage backpressure—not a universally enforced, adaptive process-memory cap.

### 8.3 Telemetry primitives versus automatically populated metrics

`telemetry::FlightRecorder` and `LatencyHistogram` are publicly accessible types.

They provide atomic counters and exponential latency buckets intended for low-overhead instrumentation.

However, the tagged source does not show `FlightRecorder` being constructed and populated as a universally integrated component of the running engine.

The types are implemented, but that does not establish the existence of a fully populated engine metrics registry, automatic latency collection, or a native metrics-export service.

The more reliable distinction is between:

| Mechanism                                                          | Availability in 1.3                      |
| ------------------------------------------------------------------ | ---------------------------------------- |
| `tracing` instrumentation in storage paths                         | Implemented                              |
| Public atomic telemetry structures                                 | Implemented                              |
| Engine background-error reporting                                  | Implemented                              |
| Programmatic storage inspection                                    | Implemented                              |
| Automatically populated `FlightRecorder` exposed from every `Tree` | Not established by the inspected source  |
| Native Prometheus or OpenTelemetry exporter                        | Not part of the documented SurrealKV API |

This distinction avoids mistaking available instrumentation building blocks for an end-to-end observability integration.

Sources:

`https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/memory.rs`

`https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/telemetry.rs`

### 8.4 Configuration validation and default-value documentation

`Options::validate()` checks several configuration invariants before the engine opens storage.

It rejects a zero LSM level count, an immutable-memtable stall threshold below two, and an L0 stall threshold smaller than the configured L0 compaction trigger.

It also checks address-space limitations on memtable arena allocation, notably for 32-bit targets.

The public options provide flexibility, but not every numeric setting represents a valid or safely allocatable configuration.

One documentation inconsistency is worth noting.

The comment associated with `with_flush_on_close` describes a default of `false`, while the actual `Options::default()` implementation sets `flush_on_close: true`.

For version 1.3, the default implementation is the authoritative behavior.

The `TreeBuilder::build_with_options` and `build_with_options_async` variants can also return the supplied configuration alongside the opened tree.

For encrypted databases, the builder documentation clarifies that these are the caller's supplied options rather than necessarily the internal resolved encryption context associated with the opened database.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/lib.rs`

## 9. Crash simulation, filesystem durability, and restoration

### 9.1 In-memory crash simulation

The in-memory filesystem is particularly interesting because it reuses the pooled-filesystem design employed by browser storage.

Rather than being merely a volatile map of files, the testing implementation distinguishes written data from synchronized data.

`FileSystem::simulate_crash(mode)` can construct a new in-memory filesystem representing the state that would remain after a simulated crash.

This enables deterministic tests of storage recovery without repeatedly terminating processes or manipulating actual disk files.

The implementation also includes hidden testing methods for truncating files, appending data, inspecting file names, and obtaining pool-mount reports.

These allow tests to construct specific failure conditions, including incomplete records and partially persisted filesystem metadata.

The distinction between test support and public product functionality is important: `simulate_crash` and several associated types are marked `#[doc(hidden)]`.

They are implemented in version 1.3 but should not be treated as guaranteed stable APIs for application use.

### 9.2 Pooled-filesystem durability fixes in version 1.3

The 1.3 release fixed two problems in its pooled filesystem.

The first concerned a rename operation whose metadata-header write failed. Before the fix, a later synchronization could make the attempted new name durable despite the earlier operation having failed.

The second concerned a superseded file slot. Its free-header state was not always flushed when the slot was released, allowing an obsolete file to reappear after a crash as though it were the replacement's valid content.

Both are examples of why durability involves more than flushing the bytes of an individual value.

File identity, renaming, allocation metadata, and the ordering of persistent updates also determine whether recovery reconstructs the intended database state.

These changes are especially relevant to the OPFS and in-memory pooled-filesystem implementations. They should not be generalized into claims about unrelated native filesystem behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 9.3 Interrupted checkpoint restoration

Checkpoint restoration is a multi-stage state replacement.

The restore path coordinates with ongoing commits, memtable flushes, compaction, WAL rotation, and value-log state.

If restoration fails after beginning to replace database files, the directory may no longer represent either the original database or the complete checkpoint.

The `RestoreInterrupted` error makes this condition explicit.

An important consequence is that a restore cannot always be treated as an atomic file-pointer swap whose failure leaves the previous state untouched.

The library includes recovery markers and validation intended to prevent an incomplete restore from being silently interpreted as an ordinary, healthy database.

This is a separate failure category from a normal transaction conflict or an unsuccessful read.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/checkpoint.rs`

Source: `https://github.com/surrealdb/surrealkv/blob/v1.3.0/src/vfs/mod.rs`

## 10. Security advisory: internal-key decoding

A relevant security advisory was issued on November 6, 2026:

RUSTSEC-2026-0452 — Out-of-bounds read when decoding internal keys shorter than the required encoded representation.

The advisory describes a memory-safety vulnerability involving `InternalKeyRef::from_encoded`.

Earlier versions exposed a safe constructor whose accessors assumed that the supplied encoded key had sufficient length.

For malformed short input, unchecked length arithmetic could produce invalid offsets and out-of-bounds memory reads in release builds.

The issue could also be reached through malformed or corrupted SSTable data reaching the decoding path.

Affected package: `surrealkv`

Fixed in 1.2.0+

RustSec severity: High, CVSS 7.9. The fix validates encoded-key lengths and bounds-checks relevant accessors.

SurrealKV 1.3.0 includes this fix. The advisory is not evidence of an unresolved vulnerability in the audited release.

The architectural lesson is that on-disk checksums and storage-format validation are related but separate concerns. Integrity checks can identify corrupted blocks, while safe decoding must remain robust even when unexpected or malformed bytes reach an internal parser.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustsec.org\&sz=32)

rustsec.org



Advisory: `https://rustsec.org/advisories/RUSTSEC-2026-0452`

Upstream fix: `https://github.com/surrealdb/surrealkv/commit/ac642f945d63c9c9f7ad35f480995fd516f83fb6`

## 11. Synthesis: what these additions change

The most consequential additions refine three aspects of SurrealKV's capability model.

First, the write path is more adaptive than a conventional buffered LSM description suggests. SurrealKV coordinates commit grouping, memtable capacity estimation, large-batch direct-to-L0 insertion, proactive pacing, WAL segment management, and recovery protection. These mechanisms collectively address workloads with highly variable transaction sizes and concurrent write pressure. They do not remove resource limits, but they significantly expand the engine's behavior beyond a fixed memtable-to-SSTable pipeline.

Second, the temporal and concurrency models require precise interpretation. Historical point lookup, version enumeration, snapshot isolation, commit validation, range tombstones, and retention are separate facilities. Their interactions determine whether previous states remain reconstructible and whether concurrent operations can preserve application invariants. In particular, range deletion's history-erasing behavior and the absence of a dedicated historical range-snapshot API are important distinctions.

Third, SurrealKV exposes advanced integration mechanisms at different stability levels. Custom comparators, membership-filter policies, encryption key management, inspection utilities, and manual compaction provide genuine extension and operational capabilities. Other mechanisms—such as simulated crashes, certain encryption-status methods, compaction-priority variants, memory-control helpers, and telemetry structures—are implemented without necessarily being fully integrated or supported as stable public configuration surfaces.

Taken together, these findings strengthen the original overview in two ways: they identify additional native mechanisms that can replace application-specific storage infrastructure, and they establish clearer boundaries around what SurrealKV guarantees versus what a caller must implement or verify.

The original guide's principal capability map remains valid. With these additions and corrections incorporated, it offers a substantially more complete reference to SurrealKV 1.3's advanced Rust functionality without needing to enumerate every internal API.