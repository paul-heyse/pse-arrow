# SurrealKV 1.0 — Comprehensive Rust Library Overview

Version: 1.0.0 · Research date: October 22, 2026 · Language: Rust · License: Apache-2.0

## 1. Executive orientation

SurrealKV is an embedded, transactional, persistent key-value storage engine implemented in Rust, combining a log-structured merge tree (LSM), multi-version concurrency control (MVCC), snapshot isolation, optimistic concurrency control, and an asynchronous commit pipeline.

Version 1.0 was announced on October 13, 2026, as the successor to RocksDB for persistent storage within SurrealDB. It is also independently usable as a Rust library, without requiring SurrealDB itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+1



SurrealKV's architectural distinction is its combination of:

- Concurrent transactions: snapshot-isolated reads, optimistic write-conflict detection, and batched durable commits.
- High-performance storage: concurrent radix-tree memtables, SSTables, leveled compaction, caching, and optional value-log separation.
- Temporal data: optional timestamped key versions, historical lookups, version iteration, and retention-aware compaction.
- Operational capabilities: crash recovery, durability controls, integrity verification, diagnostics, and storage migration.
- Integration: a native Rust API, standalone embedding, browser persistence through WebAssembly/OPFS, and use as SurrealDB's underlying persistent engine.

The most important conceptual distinction is that SurrealKV is a storage kernel, not a complete database management system. It stores ordered binary keys and values and implements storage-level transactional guarantees. Schema management, secondary indexes, relational/graph semantics, query optimization, access control, and distributed coordination belong to higher layers or applications using the engine.

### Capability map

| Functional area        | Principal mechanisms                                        | Architectural significance                                   |
| ---------------------- | ----------------------------------------------------------- | ------------------------------------------------------------ |
| Storage                | LSM tree, radix-tree memtable, SSTables, WAL                | Persistent, write-oriented KV storage                        |
| Transactions           | MVCC, snapshots, optimistic conflict detection              | Concurrent, atomic multi-key operations                      |
| Commit processing      | Concurrent commit ring, group commit                        | Amortizes synchronization and WAL I/O                        |
| Temporal storage       | Timestamped versions, `get_at`, history, retention          | Native historical state access                               |
| Large values           | Optional value separation                                   | Reduces large-value rewriting during compaction              |
| Reads and scans        | Point reads, ordered iterators, range and key-only scans    | Supports application-defined indexes and ordered data access |
| Persistence            | Durability policies, WAL recovery, checkpoints              | Explicit crash-survival and recovery behavior                |
| Efficiency             | Compression, filters, caching, memory budgeting, autotuning | Configurable read/write/memory tradeoffs                     |
| Security and integrity | Authenticated encryption, checksums, scrubbing              | Storage protection and corruption detection                  |
| Operations             | Telemetry, inspection, `skv` CLI, migration utilities       | Diagnostics, maintenance, and format transition              |

Primary entry points

- Repository: `https://github.com/surrealdb/surrealkv`
- Rust API documentation (1.0.0): `https://docs.rs/surrealkv/1.0.0/surrealkv/`
- Crate registry: `https://crates.io/crates/surrealkv`
- SurrealKV 1.0 announcement: `https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0`

The remaining discussion targets the published 1.0.0 API, rather than assuming older 0.x examples or development-branch APIs are interchangeable.

## 2. Storage architecture and execution model

SurrealKV is organized around a `Tree`, representing an independently configured persistent keyspace. A `TreeBuilder` opens or creates the storage engine; transactions operate against that tree, and background work manages memtable flushing, SSTable compaction, and persistence.

The architecture is described in detail in the official October 2026 whitepaper, A Versioned LSM Engine with Radix-Tree Memtables and a Commit Ring:

`https://surrealdb.com/static/whitepapers/surrealkv.pdf`

### 2.1 End-to-end storage path

Conceptual write/storage path, simplified from the SurrealKV 1.0 whitepaper. Reads combine visible versions across memory and on-disk tables.

### 2.2 Concurrent adaptive radix-tree memtable

SurrealKV uses a concurrent adaptive radix tree rather than the skiplist-based memtables characteristic of RocksDB and Pebble.

The memtable stores lexicographically ordered keys in an arena-backed, prefix-oriented structure. Its relevant properties include compact representation of shared key prefixes, adaptive child representations, and efficient lookup and ordered traversal.

The design separates the structure of the tree from the byte payloads held by its entries. A fixed-size arena provides predictable structural allocation, although this does not impose a hard bound on total value memory.

When a memtable reaches capacity, it becomes immutable and a new active memtable replaces it. A background flush converts the immutable memtable into an L0 SSTable. An oversized batch can be written directly as an L0 table rather than requiring the active memtable to expand indefinitely.

The default memtable capacity described by the 1.0 whitepaper is 100 MiB.

Why this matters: The radix tree is designed to reduce pointer chasing, allocation overhead, and contention on the memory-resident write/read path. The fixed arena also makes the structural memory cost more controllable. However, relative performance depends on key distribution, prefix sharing, allocator behavior, and read/write patterns. The published benchmarks do not isolate the radix tree's individual contribution.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 2.3 Leveled LSM storage

Persistent data is organized into immutable sorted-string tables (SSTables), grouped by level.

An SSTable combines compressed key blocks, index partitions, probabilistic filters, checksums, and metadata. SurrealKV uses a two-level table index, with a top-level index held in memory and subordinate partitions fetched as needed.

The default configuration described in the whitepaper is:

| Mechanism                | 1.0 default                           |
| ------------------------ | ------------------------------------- |
| LSM levels               | 6                                     |
| Memtable                 | 100 MiB                               |
| Data block size          | 64 KiB                                |
| Unified block cache      | 32 MiB                                |
| L0 compaction trigger    | 4 tables or 256 MiB                   |
| Level 1 target           | 256 MiB                               |
| Level size multiplier    | 10×                                   |
| Compaction output target | 64 MiB                                |
| Write stall              | 12 L0 tables or 2 immutable memtables |

Compaction selects work according to level pressure and file priority, merges overlapping sorted runs, and discards obsolete versions when snapshot and history-retention rules permit.

SurrealKV's 1.0 design uses a single active compaction operation, so scaling is not simply a matter of raising the number of background workers.

Consequential tradeoff: Leveled compaction improves lookup efficiency by reducing overlapping files in lower levels, but entails rewriting data. The frequency and cost of rewriting depend on level sizing, overwrite patterns, tombstone density, the amount of retained history, and whether large values are stored inline.

### 2.4 SSTable indexing, caching, and read costs

A point lookup may search the active memtable, immutable memtables, and relevant on-disk tables. It uses metadata and filters to avoid unnecessary data-block reads.

The primary mechanisms are:

- Prefix-compressed data blocks with restart points for reconstruction and seeking.
- Two-level partitioned indexes for reducing resident index memory.
- Partitioned Bloom filters to exclude SSTables or partitions that cannot contain a requested key.
- A sharded, byte-weighted block cache shared by index blocks, filter blocks, data blocks, and value-log values.

These structures can reduce disk accesses for point reads and working sets with locality.

However, the whitepaper identifies several implementation costs: synchronous reads can perform positional file I/O on the calling thread; the implementation does not depend on memory mapping or kernel asynchronous I/O; scan cursors combine multiple sources; and reverse traversal can require reconstruction from block restart points.

Consequently, a fast point-lookup path does not imply that long range scans or reverse scans have the same relative advantage.

Source: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §§5–7.

## 3. Transactions, isolation, and concurrency

The transaction model is one of SurrealKV's most significant capabilities, particularly when used independently of SurrealDB.

### 3.1 Snapshot isolation and MVCC

Each transaction observes a consistent snapshot identified by a published sequence number. Reads select the newest visible version at or below that sequence.

A transaction can perform multiple reads and writes while preserving a consistent view of committed data. Local pending writes are included in the transaction's own reads.

SurrealKV's concurrency model is:

| Property               | Semantics                                                |
| ---------------------- | -------------------------------------------------------- |
| Atomic writes          | A committed transaction's writes become visible together |
| Read consistency       | Snapshot isolation                                       |
| Concurrent readers     | Do not block ordinary writes                             |
| Write conflicts        | First-committer-wins for overlapping written keys        |
| Plain reads            | Not validated for conflicts at commit                    |
| Locked reads           | `get_for_update` registers a key for conflict validation |
| Serializable isolation | Not generally guaranteed                                 |
| Failed commit          | Transaction becomes closed; retry in a new transaction   |

The distinction between snapshot isolation and serializability is essential.

Two transactions can read the same snapshot, make decisions based on different keys, and commit disjoint writes that jointly violate an application-level invariant. This is the classic write-skew anomaly.

`get_for_update` helps by registering a read key for conflict validation, but does not acquire an exclusive mutex. Two transactions can both register the same key and succeed if neither writes it. Range scans also do not provide general predicate-conflict validation.

Accordingly, serializable application behavior may require explicit contention keys or additional invariant enforcement.

### 3.2 Commit ring and group commit

SurrealKV distinguishes itself from more conventional transaction pipelines by separating concurrent validation from ordered persistence.

Its pipeline has three important stages:

1. Concurrent validation: writers enter a bounded commit ring, publish their write sets, and detect possible conflicts using Bloom-filter prechecks followed by exact key comparison.
2. Grouped persistence: a single flusher collects ready commits in ring order, assigns sequence numbers, and appends the group to the WAL.
3. Ordered publication: the flusher applies the group to the memtable, advances the published sequence number, and completes waiting commits.

The reference architecture uses a 65,536-slot ring, permits up to 32,768 admitted commits, and gathers commit groups up to 4 MiB. A group requires at most one WAL synchronization when any member requests immediate durability.

This design reduces the number of expensive synchronization operations relative to processing every concurrent transaction through an independent synchronous WAL append.

It also preserves ordered visibility: readers cannot observe a partially applied commit group.

There are important tradeoffs. The single flusher serializes application to the active memtable, and conservative conflict detection can reject a transaction that would not ultimately have conflicted. The system is optimized for throughput under concurrency rather than guaranteeing a conflict-free or fully parallel commit path.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 3.3 Transaction API and modes

The principal entry points are `Tree::begin`, `begin_with_mode`, `begin_with_opts`, and `view`.

The 1.0 transaction interface supports the following operation families:

| Family               | Key methods                                                    |
| -------------------- | -------------------------------------------------------------- |
| Read                 | `get`, `get_async`, `get_for_update`, `get_at`                 |
| Write                | `set`, `set_at`, `replace`, `delete`, `soft_delete`            |
| Range modification   | `delete_range`                                                 |
| Iteration            | `iter`, `scan`, `range`, `range_with_options`                  |
| Historical iteration | `history`, `history_with_options`                              |
| Transaction control  | `commit`, `rollback`, `set_savepoint`, `rollback_to_savepoint` |
| Durability control   | `set_durability`, `with_durability`                            |

`Mode` and transaction options distinguish read-only, read-write, and write-only behaviors. A write-only transaction does not need to establish a read snapshot; a read-only transaction cannot perform writes or commit-time locked-read validation.

Savepoints provide stackable local rollback points without committing the enclosing transaction. A failed commit attempt terminates the transaction, so retries require reconstructing the intended operation in a fresh transaction.

The implementation exposes both ordinary key reads and conflict-tracked reads rather than silently treating every read as a potential write dependency.

### 3.4 Deletes and version visibility

SurrealKV provides several semantically different forms of modification:

- `delete`: hard deletion, hiding all earlier versions of a key.
- `soft_delete`: adds a timestamped tombstone, retaining temporal deletion information where versioning is enabled.
- `replace`: writes a new value while establishing a barrier against earlier versions.
- `delete_range`: creates a range tombstone over `[start, end)` without individually rewriting every key at deletion time.

The last distinction is useful for clearing ordered subspaces, such as a collection represented by a shared key prefix.

An O(1) range-tombstone insertion should not be confused with O(1) total deletion cost. Reads and compaction still need to interpret the tombstone, and physical reclamation occurs later.

Under snapshot isolation, range-delete conflict detection does not amount to general phantom prevention; the whitepaper specifies that only the range's start key participates in its commit-conflict check.

Sources

- Transaction API: `https://docs.rs/surrealkv/1.0.0/surrealkv/struct.Transaction.html`
- Tree API: `https://docs.rs/surrealkv/1.0.0/surrealkv/struct.Tree.html`
- Commit architecture and isolation: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §4.

## 4. Temporal versioning and historical state

SurrealKV's optional temporal versioning is more than conventional MVCC. Ordinary MVCC retains versions needed by active transactions; SurrealKV can deliberately preserve historical versions beyond transaction lifetimes and expose them to callers.

### 4.1 Two independent notions of time

SurrealKV distinguishes:

1. Commit sequence number: establishes transactional visibility and write ordering.
2. Version timestamp: represents the time associated with a stored value, supplied by the caller or assigned at commit.

A transaction sees only versions permitted by its snapshot sequence. Within that visible set, `get_at(key, timestamp)` selects the version with the greatest timestamp not exceeding the requested time. If timestamps tie, the newer write wins.

This means an application can insert a historical correction after newer data has already been committed. The correction becomes visible to later transactions but can affect what those transactions observe when querying an earlier point in time.

A later commit does not necessarily have a later version timestamp.

### 4.2 Versioning configuration and operations

Versioning is disabled by default. It is enabled through `with_versioning(true, retention_ns)`.

| Operation or setting           | Function                                                                               |
| ------------------------------ | -------------------------------------------------------------------------------------- |
| `set_at(key, value, ts)`       | Write a version carrying an explicit timestamp                                         |
| `set(...)` in a versioned tree | Timestamp at commit                                                                    |
| `get_at(key, ts)`              | Retrieve the visible version as of a timestamp                                         |
| `history(start, end)`          | Iterate historical versions over a key range                                           |
| `history_with_options(...)`    | Filter history by timestamp interval, tombstone inclusion, and result limit            |
| `soft_delete(...)`             | Record a time-bearing deletion event                                                   |
| `retention_ns = 0`             | Retain all history not hidden by hard barriers                                         |
| `retention_ns > 0`             | Retain recent history and the boundary versions necessary for correct historical reads |

These mechanisms do not require enabling the separate value log. The 1.0 implementation stores timestamps within value records and reuses the existing LSM structures for history retention and retrieval.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 4.3 Historical reads and historical iteration

`get_at` inspects visible versions of the requested key and selects by timestamp. It is not a direct timestamp-index lookup.

`history` iterates across keys in the requested range and returns versions for each key in descending timestamp order. It can include tombstones and can traverse in either direction.

This provides two different operations:

- State reconstruction: determine the value that applied at a specified logical time.
- Change examination: enumerate past values and deletions, including multiple versions of the same key.

These are useful primitives for temporal records, historical configuration, change reconstruction, and audit-oriented data models. They do not automatically produce cross-key audit events, transaction-level change feeds, or materialized snapshots of an entire application state.

A point-in-time transaction's snapshot sequence and a `get_at` historical timestamp should not be conflated. The former determines which writes the transaction is allowed to see; the latter determines which timestamped version to select from those writes.

### 4.4 Retention and compaction

Temporal retention is enforced during compaction.

With a nonzero retention horizon, the engine preserves versions within the retention window and boundary versions required to answer reads correctly at the cutoff. Open snapshots can retain additional versions.

This is not a time-based deletion service that immediately removes data when a clock reaches a deadline. Versions are reclaimed as compactions process their keys, subject to snapshot visibility and deletion barriers.

There are three particularly consequential limits:

- A `get_at` read examines every visible version of its key. Its cost therefore increases with retained history.
- A history iterator buffers the versions of one key while processing them, so a single exceptionally version-heavy key can increase memory consumption.
- A hard delete, replace, or range delete hides prior history rather than merely adding another time-stamped event. Soft deletion is the mechanism for preserving a deletion event as historical data.

The 1.0 whitepaper states explicitly that it does not contain measurements for temporal read overhead or versioned compaction overhead.

Source: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §8.

## 5. WAL durability, recovery, checkpoints, and integrity

### 5.1 Durability is a transaction-level choice

SurrealKV exposes two durability policies:

| Policy                  | Meaning                                                                                     | Principal tradeoff                                                                                  |
| ----------------------- | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `Durability::Eventual`  | Commit without waiting for an `fsync`; default                                              | Lower synchronization cost, but an OS crash or power failure can lose recently acknowledged commits |
| `Durability::Immediate` | Wait for the WAL and required value-log data to be synchronized before acknowledging commit | Stronger crash-survival guarantee at the cost of synchronization                                    |

Because transactions are group-committed, one immediately durable transaction can cause the entire group to be synchronized, including other transactions requesting eventual durability.

`Tree::flush_wal(true)` provides a separate synchronization boundary for preceding WAL writes, while `flush_wal(false)` flushes without requesting the same disk synchronization guarantee.

This permits a deliberate distinction between transactional atomicity and isolation and persistence through a power failure. Both modes use the transaction machinery; the difference is when the engine acknowledges the requested durability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 5.2 Crash recovery and explicit salvage behavior

SurrealKV associates WAL segments with memtable generations. On restart, it loads the manifest and referenced table metadata, replays the relevant WAL segments, and resumes logging in a new segment.

The 1.0 implementation does not treat arbitrary WAL corruption as a reason to silently discard data.

| Recovery policy       | Behavior                                                                                                         |
| --------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Default recovery      | Repair a provable torn tail of the newest WAL segment; reject corruption that would discard known intact records |
| `AbsoluteConsistency` | Reject a damaged log without modifying it                                                                        |
| `SalvageValidPrefix`  | Explicitly retain a valid log prefix and discard later records, accepting potential data loss                    |

The default repair path quarantines copies of affected log segments and exposes recovery reporting. `Tree::recovery_report()` supplies information about replayed, trimmed, or discarded records and related repair actions.

Recovery also verifies relevant references to values stored in the separate value log. Depending on recovery policy and the location of the damage, a missing referenced value can prevent the database from opening.

A directory containing database data but lacking a usable manifest is rejected rather than silently reinitialized as an empty database. This matters for preventing accidental apparent data loss when a file or metadata operation fails.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 5.3 Checkpoint and restore

The engine provides native `Tree::create_checkpoint` and `Tree::restore_from_checkpoint` operations.

A checkpoint establishes a consistent on-disk representation by flushing memtables, hard-linking immutable SSTables into the checkpoint, copying the manifest and value-log contents, and creating an empty WAL directory. Completion metadata is written last so incomplete checkpoints are detectable.

A restore validates its source before modifying the destination, coordinates with compaction, and fences in-flight commits from entering the restored state.

Important constraints:

- Checkpoints are not merely an unflushed snapshot of process memory; creating one entails storage work.
- Hard-linking SSTables can reduce local copy cost, but does not itself create an independent off-host backup.
- Restore is not atomic. An interrupted restore leaves a detectable marker and prevents an ordinary open until the interruption is addressed.
- Checkpoint directories cannot overlap the live database directory.
- Encrypted checkpoints require access to the corresponding historical encryption keys.

The resulting capability is a coordinated storage-level backup/restore primitive, not a general incremental backup protocol or cross-machine replication service.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 5.4 Data integrity and corruption detection

SurrealKV applies checksums to SSTable blocks, WAL records, and manifest metadata. SSTable blocks use the IEEE CRC-32 polynomial, not CRC-32C, and are verified when loaded from disk rather than on every hit in the block cache.

Value-log checksum verification is separately configurable. The default does not perform full value-log checksum verification, which matters when enabling value separation for data requiring stronger end-to-end integrity checking.

The operator tool includes an offline `skv scrub` facility for checking SSTable data blocks. Continuous online background scrubbing is not implemented in 1.0.

These mechanisms detect several classes of corruption, but should not be interpreted as comprehensive protection against every hardware, filesystem, or application-level integrity failure.

Sources: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §§12–13; `https://github.com/surrealdb/surrealkv`.

## 6. Value separation, compression, filters, and ordered access

### 6.1 Optional WiscKey-style value log

With `with_enable_vlog(true)`, SurrealKV separates sufficiently large values from the primary LSM tree.

By default, values larger than 1 KiB are appended to a value log; the LSM retains a 25-byte pointer identifying their location. Value-log files rotate at approximately 256 MiB by default.

This changes the cost structure of writes and compaction:

- The LSM continues to compact keys, sequence metadata, and pointers without repeatedly rewriting large payloads.
- Reads of separated values require an additional lookup to the value log unless the value is cached.
- Value-log writes require coordination with WAL durability and recovery.
- Physical space reclamation is governed by references from surviving tables and memtables.

A critical 1.0 limitation is that value-log garbage collection is prefix-only. It does not rewrite live values from partially obsolete files into newer files.

Consequently, a single live value in an old value-log file can prevent deletion of that file and every subsequent file, even if most other values have been overwritten or deleted.

This is a material distinction from value-log systems with rewriting garbage collectors. Value separation can reduce compaction bandwidth while increasing persistent space amplification under certain update and retention patterns.

It is most attractive when large values dominate write amplification and their lifetimes permit effective reclamation. It is less clearly beneficial when values are small, reads are latency-sensitive, or old large values remain live for long periods.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



### 6.2 Compression by LSM level

The engine offers configurable block compression, including Snappy, Zstd, and LZ4 codecs.

Its `Options` API exposes:

- `with_compression_per_level(...)`
- `with_l0_no_compression()`
- `without_compression()`

Compression is disabled by default. A per-level configuration can preserve inexpensive writes and fast reads for recent L0 data while compressing colder data in deeper levels.

Because compression happens at the block level, its effectiveness depends on key/value representation, block size, restart intervals, and data similarity. Compression may reduce persistent bytes and physical I/O while adding CPU work on block construction and cache misses.

The engine also exposes `with_block_size`, `with_block_restart_interval`, `with_index_partition_size`, and `with_block_cache_capacity`, allowing tuning of the balance between lookup granularity, index overhead, compression, and caching.

### 6.3 Configurable comparators and probabilistic filters

Beyond a fixed bytewise key ordering, the public 1.0 API includes `Comparator` and `FilterPolicy` extension traits, along with a `BytewiseComparator` and an exported `RibbonFilter`.

These are more consequential than conventional performance settings:

| Extension point | Purpose                                              | Important constraint                                                                                                |
| --------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `Comparator`    | Define user-key ordering                             | The same ordering must be preserved across storage creation, reopening, and interpretation of SSTables              |
| `FilterPolicy`  | Customize probabilistic membership filtering         | False negatives would violate correctness; format and compatibility matter                                          |
| `RibbonFilter`  | Alternative compact membership-filter implementation | Its tradeoff concerns construction cost, memory, and false positives; it is not the 1.0 default Bloom configuration |

A custom comparator can make particular encoded key types naturally sortable, but changes persistent ordering semantics. Applications should treat it as part of the database format rather than merely a runtime convenience.

By contrast, filters determine which negative lookups can avoid reading blocks. A filter can reject absent keys but cannot establish their presence without checking the underlying data.

These extension points make SurrealKV more adaptable as a storage kernel, although they also introduce persistence-format compatibility responsibilities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



### 6.4 Ordered iteration and key encoding

`Transaction::range`, `scan`, and `iter` expose ordered data access through the `LSMIterator` trait.

The iterator supports seek-first, seek-last, arbitrary seek, forward movement, backward movement, key inspection, and value retrieval. Range boundaries use half-open `[start, end)` semantics.

SurrealKV also exposes borrowed internal key references and optimized byte representations, including `ByteSlice` and `IntoBytes`. Iteration can avoid retrieving or decoding values when only keys are needed.

These features are useful for representing application-defined indexes directly as ordered keyspaces.

For example, an application can store:

- Primary values under one key prefix.
- Secondary index entries under a different prefix, ordered by an encoded attribute.
- A mapping from those index entries to the primary keys.

A range scan then becomes an ordered index traversal.

This is an architectural possibility, not a native secondary-index feature. The caller remains responsible for canonical key encoding, index maintenance, consistent multi-key updates, and any uniqueness constraints. SurrealKV supplies the transactional ordered storage operations underneath those structures.

Sources:

- Tagged 1.0 API definitions: `https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/lib.rs`
- Iterator API: `https://docs.rs/surrealkv/1.0.0/surrealkv/trait.LSMIterator.html`
- Whitepaper: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §§6–8.

## 7. Transparent encryption and key management

SurrealKV 1.0 includes transparent encryption at rest for native storage. It covers the WAL, SSTables, manifest, value log, and checkpoints without changing the key/value transaction API.

### 7.1 Encryption model

An encrypted database uses either AES-256-GCM or XChaCha20-Poly1305.

`CipherPreference::Auto` selects a cipher based on hardware capabilities when the database is created; an explicit `CipherSuite` can also be supplied. The cipher choice is persisted and does not silently change on subsequent openings.

Instead of storing one encryption key for all file contents, the engine derives individual file keys from application-supplied root-key material, file identity, and random salts.

Authenticated encryption binds content to its database, file, and position. This protects against undetected modification or inappropriate relocation of encrypted blocks.

The distinction between encrypted data and encrypted metadata is also explicit: SurrealKV does not hide directory names, file sizes, or file-access timing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



### 7.2 `KeyManager` as an extension point

The library does not persist root keys itself.

It provides `SoftwareKeyManager` and a `KeyManager` trait for integrating external key management.

The trait separates the slow operation of obtaining keys from fast operations used by the engine:

- `prepare(...)`: obtain or initialize required roots before normal operation.
- `active_key_id()`: identify the key for new encrypted files.
- `key_state(...)`: determine whether an ID is active, decrypt-only, or unavailable.
- `root_key(...)`: provide the root material needed to derive a file key.

An implementation backed by a KMS or HSM can fetch or unwrap keys during preparation and retain appropriate key material in memory for operations on the commit path.

The native extension is therefore external key authority with local hot-path key availability, not necessarily remote HSM participation in every encrypt/decrypt operation.

### 7.3 Rotation and lifecycle semantics

Rotation is implemented through key IDs.

A newly active root key is used for newly created files. Earlier files remain encrypted under their original key IDs and remain readable as long as those keys are available in decrypt-only state.

Rotation does not immediately re-encrypt all existing SSTables, WAL segments, checkpoints, or backups.

This has several implications:

- Historical keys must be retained while any required artifact references them.
- Key inventory and backup retention become coupled.
- Removing a key too early makes affected files permanently unreadable.
- A database cannot be retroactively switched from unencrypted to encrypted simply by supplying a key manager.

Missing or incorrect keys produce explicit encryption errors rather than causing the engine to initialize a replacement database.

The `encryption` Cargo feature is enabled by default and can be disabled with `default-features = false`. A build lacking the feature refuses encrypted configuration rather than writing unencrypted files unexpectedly.

Source: `https://github.com/surrealdb/surrealkv`, Transparent Data Encryption section.

## 8. Diagnostics, tuning, and operational integration

### 8.1 Built-in inspection surfaces

The `surrealkv-cli` companion crate supplies the `skv` executable for examining the engine without writing a custom storage inspector.

```
cargo install surrealkv-cli

skv inspect path/to/db
skv manifest path/to/db
skv sst path/to/db/sstables/example.sst --dump-keys --verify-checksum
skv scrub path/to/db
skv get path/to/db "user:001"
skv scan path/to/db --prefix "user:"
skv enc-info path/to/db
```

These cover different operational layers: overall file/storage composition, the level manifest, SSTable contents and checksums, key-level inspection, and encryption metadata.

The Rust API additionally exposes the `inspect` module and explicit recovery and background-error reporting.

For programmatic integration, the distinction between storage metadata and application semantic metadata matters. SurrealKV can expose level structure, file health, WAL recovery, and storage-related failures; it cannot infer whether the serialized value of a key satisfies a higher-level business schema.

### 8.2 Memory and background execution

The public 1.0 crate also exports `autotune`, `memory`, and `telemetry` modules. These are worth examining for hardware-aware configuration, memory accounting/budget coordination, and lower-level operational instrumentation.

They should be distinguished from the engine's fixed structural characteristics. Exposing configuration and telemetry does not eliminate bottlenecks caused by a single commit flusher or a single compaction task.

Several details are especially relevant to memory behavior:

- The memtable's arena limit does not impose a bound on all transaction write buffers, heap-resident values, cached blocks, or background-work allocations.
- The block cache is byte-budgeted, but is only one component of total process memory.
- Large values held inline can cause transient memory use to exceed the nominal memtable capacity.
- Long-lived snapshots keep MVCC versions eligible for later reads and therefore can delay reclamation.

SurrealKV uses write stalls and increasing delays as L0 or immutable-memtable pressure accumulates. These prevent indefinite front-end write progress when the storage-maintenance pipeline is falling behind, but can introduce substantial latency variability.

The source-level API includes `BackgroundError`, `BackgroundErrorHandler`, `BackgroundErrorReason`, `WriteStallReason`, and `WriteStallInfo`, permitting callers to distinguish storage-health failures and write-pressure conditions from ordinary transaction conflicts.

A significant additional limitation is that a long-open mutating transaction can hold back commit-ring retirement. The 1.0 whitepaper notes that the resulting overflow is not bounded. Long-running read-only snapshots have different retention implications and do not pin the same conflict window.

### 8.3 Testing and verification evidence

SurrealKV's testing infrastructure is unusually relevant to the engine's claims.

The project maintains:

- A seeded differential reference-model test, exercising 500 seeds of 5,000 steps per CI run.
- Separate randomized verification of versioned operations, including historical reads and retention behavior.
- Tests that emulate file synchronization failure, truncated WAL records, partial grouped commits, interrupted compactions and restores, and corruption of manifest bytes.
- Property-based tests of encryption behavior.

However, the 1.0 whitepaper explicitly distinguishes the principal differential harness from full deterministic simulation testing: it does not control time, scheduler interleavings, or the filesystem for that test, and its reference-model commits are not concurrent.

The project also reports no coverage-guided fuzzing in the evaluated test suite.

The appropriate interpretation is therefore substantial targeted crash and state-model verification, rather than a claim that every concurrent execution or storage-failure schedule has been exhaustively explored.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+1



Sources:

- CLI, verification, and related crates: `https://github.com/surrealdb/surrealkv`
- 1.0 module exports and operational error types: `https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/lib.rs`
- Verification and stated limitations: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §§13 and 16.

## 9. Browser persistence: WebAssembly and OPFS

SurrealKV 1.0 has a browser-oriented execution mode that is substantially different from its native server mode.

It can be compiled for `wasm32-unknown-unknown` and store database files using the browser's Origin Private File System (OPFS).

The API uses `FileSystem::opfs(...)` and `TreeBuilder::with_file_system(...)`, with asynchronous initialization through `build_async()`.

### 9.1 Runtime architecture

OPFS persistence operates inside a dedicated Web Worker because the implementation depends on synchronous access handles that are not available in the page's main thread.

The browser implementation is single-threaded. It does not start a native Tokio runtime or the native background-thread infrastructure, and building with WebAssembly atomics/threads is explicitly refused.

It supports opening and closing a tree, committing, reading, WAL replay, flushing, compaction, and value-log storage. `FileSystem::memory()` also provides a volatile backend that can run without OPFS persistence.

A complete lifecycle closes the `Tree` and then the `FileSystem`, releasing storage handles for subsequent browser sessions.

### 9.2 Browser-specific constraints

| Area                   | SurrealKV 1.0 browser behavior                                                                |
| ---------------------- | --------------------------------------------------------------------------------------------- |
| Browser validation     | Chrome tested; Firefox and Safari not established                                             |
| Worker model           | Dedicated Web Worker only for OPFS                                                            |
| Concurrency            | One active mount of a database; another tab/worker encounters a database-lock error           |
| Durability             | `Immediate` requests browser flush, but does not establish native OS power-failure guarantees |
| Storage format         | OPFS-backed file pool, not a native SurrealKV directory                                       |
| Memory                 | Limited by browser/WASM address space; large inline values can be significant                 |
| Background work        | Long flushes or compactions can block worker message handling                                 |
| Quota                  | Writes may fail with `StorageFull`; browser eviction remains possible                         |
| Encryption             | Unavailable                                                                                   |
| Checkpoint/restore     | Unavailable                                                                                   |
| Native migration tools | RocksDB and SurrealKV 0.x migrators unavailable                                               |

The browser's `navigator.storage.persist()` mechanism can reduce eviction risk where permission is granted, but it is not equivalent to a durable storage guarantee.

The OPFS implementation also has a fixed file-pool limit. Repeated sessions that write small amounts of data without orderly closure can eventually exhaust that limit, even when the logical key/value dataset remains modest.

### 9.3 IndexedDB migration

The `surrealkv-compat-indxdb` companion supplies `migrate_from_indexeddb`, permitting legacy IndexedDB key/value data to be copied into an OPFS-backed SurrealKV database.

The migration must run before opening the new database, as part of startup coordination. It uses explicit completion markers and reports whether data was imported, whether the destination already exists, or whether no source could be found.

The distinction matters because `IndexedDbMigration::NoSource` can indicate either a genuinely absent database or an environment where IndexedDB is inaccessible. Opening an empty SurrealKV destination too early can prevent a subsequent migration, because later calls detect an existing destination and do not overwrite it.

The source IndexedDB data is not deleted automatically. The separate `delete_indexeddb` operation is destructive and is appropriate only after verifying the migrated data and handling any skipped records.

There is currently no direct export path that makes an OPFS file pool usable as a native SurrealKV database directory.

Source: `https://github.com/surrealdb/surrealkv`, WebAssembly & OPFS and Moving data from IndexedDB sections.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+3



## 10. Migration, companion crates, and SurrealDB integration

### 10.1 Native migration capabilities

SurrealKV 1.0 provides more than an on-disk format compatible only with databases it created itself. It includes migration paths for legacy SurrealKV and SurrealDB storage.

| Source format                                      | Migration path                                | Relevant semantics                                                                     |
| -------------------------------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------- |
| SurrealKV 0.20–0.21                                | Automatic conversion when opened              | Retained versions, sequence numbers, timestamps, and WAL contents are migrated         |
| SurrealDB-created RocksDB databases (RocksDB 8–11) | Automatic conversion or offline `skv migrate` | Pure-Rust reader; supports WAL and blob data, with historical versions where available |
| Legacy IndexedDB (`indxdb://`)                     | Companion library or browser migration API    | Copies key/value records into SurrealKV storage                                        |
| Arbitrary external RocksDB databases               | Not a general compatibility guarantee         | The implemented reader targets databases produced by SurrealDB                         |

For SurrealDB's versioned RocksDB databases, the migration logic converts stored hybrid logical clock timestamps into the temporal representation expected by SurrealKV.

Whether historical versions survive as queryable history also depends on opening the target with versioning enabled and the chosen retention policy.

### 10.2 Migration is a one-way format transition

The migration support is useful, but it should not be mistaken for bidirectional interoperability.

The first open of a legacy database can convert its storage automatically. Older database files are retained under directories such as `_surrealkv_v0_backup/` or `_rocksdb_backup/`.

However, the upstream documentation explicitly warns that these internal backup directories are not reliable rollback points. Some storage components, particularly value-log data and its reclamation, are shared with the converted database.

A database already opened by SurrealKV 1.0 cannot simply be reopened by an older release.

The important general distinction is between a migration's preservation of source-format files and a separately verified, independently restorable backup. They are not equivalent.

### 10.3 Companion libraries and their responsibilities

The upstream workspace contains several components with distinct roles.

| Crate                      | Responsibility                                                 |
| -------------------------- | -------------------------------------------------------------- |
| `surrealkv`                | Core transactional LSM storage engine                          |
| `surrealkv-cli`            | `skv` inspection, diagnostics, integrity checks, and migration |
| `surrealkv-compat-rocksdb` | Pure-Rust reader for supported SurrealDB RocksDB storage       |
| `surrealkv-compat-v1`      | Reader for older SurrealKV LSMV1 storage                       |
| `surrealkv-compat-indxdb`  | Legacy IndexedDB compatibility and migration                   |
| `surrealkv-sim`            | Simulation and differential-testing infrastructure             |

The compatibility crates are principally integration and migration mechanisms. They do not transform SurrealKV into a general multi-engine federation layer.

Source: `https://github.com/surrealdb/surrealkv/tree/v1.0.0/crates`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



### 10.4 SurrealKV versus SurrealDB

SurrealKV is intended to serve as the persistent storage layer beneath SurrealDB, but the two products expose different abstractions.

| Capability                             | SurrealKV                 | SurrealDB                                                           |
| -------------------------------------- | ------------------------- | ------------------------------------------------------------------- |
| Durable binary key/value operations    | Native                    | Via storage backend                                                 |
| Snapshot-isolated storage transactions | Native                    | Builds database transaction semantics on storage                    |
| Temporal key history                   | Native when enabled       | Higher-level temporal functionality depends on database integration |
| SurrealQL query processing             | No                        | Yes                                                                 |
| Schema and record model                | Caller-defined bytes      | Database-level records and schema                                   |
| Secondary indexes and query planning   | Caller-defined structures | Database functionality                                              |
| Graph relations and traversal          | Caller-defined encoding   | Database functionality                                              |
| Authorization and user management      | No                        | Database functionality                                              |
| Distributed database coordination      | No                        | Belongs to SurrealDB deployment architecture                        |

As of the October 13, 2026 announcement, SurrealDB described SurrealKV as the intended sole persistent storage engine for SurrealDB 4. That is a statement about the upcoming SurrealDB release, not evidence that every existing SurrealDB deployment already uses it.

The boundary is consequential: a system can benefit from SurrealKV's transaction and storage machinery without implementing its own WAL, LSM tree, or MVCC engine. But native SurrealKV does not remove the need for query, indexing, domain, or distributed semantics when those are required.

Source: `https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



## 11. Performance characteristics and evidence

SurrealKV's stated performance advantages are associated with identifiable mechanisms rather than a universal claim of superiority.

Its main performance hypothesis is that concurrent commit admission and validation, followed by a single grouped WAL append, substantially reduces the synchronization cost of many small durable writes. Its radix-tree memtable and partitioned SSTable structures are intended to make point reads and in-memory operations efficient.

### 11.1 Published comparison

The official benchmark uses an AMD Ryzen Threadripper 9970X, 128 GiB of RAM, a PCIe 4.0 NVMe SSD, 500,000 keys, 48 worker threads, and 128 clients.

Operations per second, with synchronous durability

| Operation               | SurrealKV | RocksDB | Relative result |
| ----------------------- | --------- | ------- | --------------- |
| Point reads             | 6,804,864 | 855,851 | 7.95×           |
| Creates                 | 229,376   | 8,568   | 26.8×           |
| Updates                 | 221,341   | 36,263  | 6.10×           |
| Deletes                 | 264,089   | 32,043  | 8.24×           |
| Range scans (limit 100) | 905,132   | 852,661 | 1.06×           |

Operations per second, without synchronous durability

| Operation | SurrealKV | RocksDB | Relative result |
| --------- | --------- | ------- | --------------- |
| Creates   | 321,715   | 603,452 | 0.53×           |
| Updates   | 274,172   | 608,494 | 0.45×           |
| Deletes   | 307,034   | 739,357 | 0.42×           |

Published write throughput: SurrealKV versus RocksDB

Operations per second from upstream single-run benchmarks, shown separately for immediate and non-synchronous durability.

SurrealKV

RocksDB

0200K400K600K800KSync createSync updateSync deleteNo-sync createNo-sync updateNo-sync delete

The comparison strongly favors SurrealKV for the benchmark's concurrent, synchronously durable workload. It favors RocksDB for writes that do not wait for synchronous durability.

Other engine comparisons show that SurrealKV is not the fastest in all scan categories. LMDB leads several scan workloads, while libmdbx leads some skip and count workloads in the published results.

### 11.2 What the measurements do and do not establish

The whitepaper discloses important qualifications:

- Each reported number is from a single run, without measured run-to-run variance.
- SurrealKV was configured with a 256 MiB memtable, a cache sized for the test machine, and value-log separation enabled. These are not the engine's ordinary defaults.
- Versioning was disabled, so the numbers do not measure historical reads, version retention, or versioned compaction.
- The dataset was small relative to available RAM. The results say comparatively little about large datasets whose point reads routinely require physical I/O.
- The benchmark does not isolate the performance contribution of the commit ring, memtable, or value separation individually.
- Tail-latency distributions, sustained compaction pressure, long-running transactions, and broad mixed-workload conditions remain unmeasured in that report.

Memory results also require care. The upstream no-sync run reports 221 MiB resting and 1.79 GiB peak memory for SurrealKV, versus 590 MiB resting and 1.02 GiB peak for RocksDB. The engine can therefore have a relatively small idle footprint but higher transient memory consumption under load.

These are published upstream measurements, not independently reproduced results in this overview.

Sources: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §§14–16; `https://github.com/surrealdb/surrealkv`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



## 12. Consequential design boundaries

The library's major limitations are best understood in terms of the guarantees it does not supply or the costs that its architecture cannot eliminate.

| Boundary                                  | Implication                                                                                         |
| ----------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Snapshot isolation, not serializability   | Write skew and predicate anomalies require additional application mechanisms                        |
| Single commit flusher                     | Grouped commits reduce coordination cost, but memtable application is serialized                    |
| Single compaction task                    | Storage maintenance can become a throughput bottleneck under sustained ingestion                    |
| Synchronous point reads and iteration     | Native I/O can block the calling thread; `get_async` is a separate point-read option                |
| Temporal reads inspect retained versions  | Historical lookups can become expensive for version-heavy keys                                      |
| Prefix-only value-log reclamation         | Long-lived values can pin substantial amounts of obsolete disk space                                |
| Long-open mutating transactions           | Commit-ring retention can grow without a defined bound                                              |
| Storage-level transactions only           | No built-in query planner, schema engine, or generalized index maintenance                          |
| No native distributed coordination        | Replication, leader election, consensus, and distributed transactions are external responsibilities |
| Recovery may explicitly refuse corruption | Data-preserving failure can require human intervention rather than silent repair                    |
| Restore is non-atomic                     | Interruption detection exists, but the restore operation is not itself atomic                       |
| Browser support has distinct guarantees   | OPFS format, durability, storage quotas, and feature availability differ from native deployments    |

Two additional points deserve emphasis.

First, a failed commit is not necessarily a proven non-commit. The whitepaper documents that, after certain WAL synchronization failures, a commit that returned an error can still take effect on the next open. That creates an important uncertainty boundary for non-idempotent operations: callers cannot universally interpret every commit error as proof that the operation had no effect.

Second, the 1.0 version number does not imply a frozen storage format. The upstream documentation explicitly warns that its on-disk formats are not yet frozen. This is significant when assessing upgrade compatibility, long-term archive readability, and independently recoverable backups.

Source: `https://surrealdb.com/static/whitepapers/surrealkv.pdf`, §§12 and 16; `https://github.com/surrealdb/surrealkv`, Transparent Data Encryption.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

surrealdb.com

+2



## 13. Rust packaging and minimal API example

The 1.0 crate specifies Rust 1.86 as its minimum supported Rust version and uses the Rust 2021 edition. The default Cargo feature set includes encryption.

A minimal native integration uses the `TreeBuilder` and transactional API directly:

```
use surrealkv::{Durability, TreeBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tree = TreeBuilder::new()
        .with_path("data/store".into())
        .build()?;

    {
        let mut tx = tree.begin()?;
        tx.set(b"item:001", b"payload")?;
        tx.set_durability(Durability::Immediate);
        tx.commit().await?;
    }

    {
        let tx = tree.begin()?;
        let value = tx.get(b"item:001")?;
        assert_eq!(value.as_deref(), Some(b"payload".as_slice()));
    }

    tree.close().await?;
    Ok(())
}
```

This small example captures several design characteristics: the tree is the independently managed storage instance, writes are buffered in transactions, commit is asynchronous, read operations use a transaction snapshot, and durability can be chosen for the transaction.

Source: `https://github.com/surrealdb/surrealkv/blob/v1.0.0/Cargo.toml` and `https://github.com/surrealdb/surrealkv`.

## 14. Primary-source research map

The following are direct URLs, with their principal contents, suitable for use independently of this overview.

| Source                                                                   | Coverage                                                                                       |
| ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| Versioned 1.0 whitepaper                                                 | Architecture, commit ring, MVCC, SSTables, retention, compaction, recovery, limits, benchmarks |
| `https://surrealdb.com/static/whitepapers/surrealkv.pdf`                 |                                                                                                |
| Upstream repository and README                                           | Public feature summary, examples, encryption, CLI, migration, browser support                  |
| `https://github.com/surrealdb/surrealkv`                                 |                                                                                                |
| Tagged 1.0.0 source                                                      | Version-specific implementation for verifying APIs and behavior                                |
| `https://github.com/surrealdb/surrealkv/tree/v1.0.0`                     |                                                                                                |
| Rust API reference                                                       | `Tree`, `Transaction`, `TreeBuilder`, options, iterators, public traits and error types        |
| `https://docs.rs/surrealkv/1.0.0/surrealkv/`                             |                                                                                                |
| Tagged crate root                                                        | Public exports, core options, extension traits, configuration defaults                         |
| `https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/lib.rs`          |                                                                                                |
| Core transaction implementation                                          | Snapshot reads, conflict handling, savepoints and write semantics                              |
| `https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/transaction.rs`  |                                                                                                |
| Storage engine implementation                                            | Tree lifecycle, persistence and background operations                                          |
| `https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/lsm.rs`          |                                                                                                |
| Release announcement                                                     | Design rationale, performance positioning, planned SurrealDB integration                       |
| `https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0` |                                                                                                |
| Published crate                                                          | Dependency metadata, releases and feature flags                                                |
| `https://crates.io/crates/surrealkv`                                     |                                                                                                |

Version-research caution: The repository contains older architectural documentation that describes skiplist memtables, different commit arrangements, older WAL recovery behavior, and VLog mechanisms that do not accurately describe the 1.0 implementation. The tagged 1.0 source and October 2026 whitepaper are preferable where they conflict with those earlier documents. Similarly, search-engine caches of `docs.rs/latest` may still expose 0.21.x documentation; verify the displayed crate version before relying on an API description.

## 15. Architectural synthesis

SurrealKV's fundamental value comes from combining mechanisms that are often implemented separately in storage-backed applications.

Its native transaction model, WAL, ordered LSM storage, snapshot tracking, and historical version retention can support an application-defined persistent data model without requiring the caller to build those foundational mechanisms. The same ordered keyspace can hold primary records, secondary-index entries, transactional metadata, and historical records, with consistency across those structures managed by storage transactions.

The distinction from conventional embedded key/value libraries is especially visible in three combinations:

Concurrent durable writes: Optimistic transactions plus the commit ring plus group synchronization provide atomic multi-key writes while amortizing durability costs across concurrent writers.

Temporal persistent state: MVCC sequence ordering plus value timestamps plus retention-aware compaction permit historical reads without a separate history index or mandatory event-log subsystem.

Adaptable storage efficiency: Leveled compaction plus configurable caching and compression plus optional value separation allow workloads to trade memory, CPU, disk bandwidth, latency, and persistent space against one another.

These benefits are conditional. Snapshot isolation is insufficient for some invariants, temporal lookups become increasingly expensive with large histories, and value separation may have unfavorable garbage-collection behavior. The engine's intentionally constrained background and commit pipelines also define scaling limits that more configuration alone cannot remove.

SurrealKV 1.0 is therefore best understood as a capable, transactionally consistent Rust storage kernel with distinctive strengths in concurrent durable writes and temporal key/value data—not as a general substitute for SurrealDB, a relational engine, or a distributed database.