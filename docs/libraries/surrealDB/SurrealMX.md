# SurrealMX 1.2 — Comprehensive Rust Library Reference

Research date: November 10, 2026 · Target version: 1.2.0 · Language: Rust · License: Apache 2.0

SurrealMX is an embedded, memory-resident, transactional key-value storage engine developed by SurrealDB. It combines a concurrent adaptive radix tree, multi-version concurrency control (MVCC), optimistic transaction validation, and a circular commit ring to provide high-throughput access to mutable state without requiring a separate database server.

Its significance extends beyond caching: SurrealMX provides a general-purpose transactional state substrate for applications that need consistent snapshots, concurrent writes, atomic multi-key updates, ordered range access, and optional persistence, while retaining direct access to data from Rust.

Unlike SurrealDB itself, SurrealMX is a storage engine rather than a document, graph, or SQL database. It does not independently supply SurrealQL, schemas, indexes over document properties, graph traversal, or distributed database services.

Primary upstream sources:

- Repository: [https://github.com/surrealdb/surrealmx](https://github.com/surrealdb/surrealmx?utm_source=chatgpt.com)
- v1.2.0 release: [https://github.com/surrealdb/surrealmx/releases/tag/v1.2.0](https://github.com/surrealdb/surrealmx/releases/tag/v1.2.0?utm_source=chatgpt.com)
- Version-pinned source: [https://github.com/surrealdb/surrealmx/tree/v1.2.0](https://github.com/surrealdb/surrealmx/tree/v1.2.0?utm_source=chatgpt.com)
- Rust crate: [https://crates.io/crates/surrealmx](https://crates.io/crates/surrealmx?utm_source=chatgpt.com)
- API documentation: [https://docs.rs/surrealmx/latest/surrealmx/](https://docs.rs/surrealmx/latest/surrealmx/?utm_source=chatgpt.com)

The GitHub release and tagged `Cargo.toml` confirm v1.2.0 was released on November 10, 2026, at 17:20 UTC. At the time of research, the publicly indexed docs.rs pages still showed v1.0.0. Accordingly, this reference uses the v1.2.0 Git tag as the version authority, rather than assuming the `latest` documentation alias reflects the new release.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 1. What's new in SurrealMX 1.2

Version 1.2 is primarily a commit-path performance and concurrency refinement, rather than an expansion of the public API.

The v1.2 GitHub release has no descriptive changelog. Comparing the v1.1.0 and v1.2.0 source tags identifies three commits, two containing substantive engine optimizations.

| Change                                                 | Technical significance                                                                      |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| Merge versions derived directly from commit-ring slots | Removes a separate version-allocation and ordering mechanism, reducing synchronization work |
| Elimination of the earlier-version-claim waiting chain | Avoids an additional dependency between concurrent committing transactions                  |
| Commit unwind guards borrow the database               | Avoids unnecessary `Arc` reference-count operations in the commit path                      |
| Simplified abort/version publication                   | Aborted commits publish empty merges so subsequent versions can continue to become visible  |
| Commit-scaling example                                 | Adds a benchmark-style example varying concurrent writers                                   |
| Recovery and ring reseeding adjustments                | Aligns recovered version numbering with the commit ring's new authoritative sequence        |

### 1.1 Commit-ring slots now determine version ordering

Previously, the engine assigned commit-ring slots and merge-version numbers through separate mechanisms. It had to coordinate those mechanisms to ensure transactions became visible in their validated commit order.

In v1.2, a commit's ring sequence number is also its merge version.

This is important because transaction validation depends on the relative ordering of commits. Using a single sequence simplifies the consistency protocol and removes the need to wait for earlier transactions to claim separate versions.

Aborted transactions still occupy sequence positions. Their empty merge entries allow the visibility watermark to advance without exposing writes that did not commit.

This is a consequential architectural simplification: the same ordered sequence now coordinates conflict validation, version identity, and visibility progression.

Source: [https://github.com/surrealdb/surrealmx/commit/419e55d3816af6d43ff5bcdd3bf2416d77ba84f4](https://github.com/surrealdb/surrealmx/commit/419e55d3816af6d43ff5bcdd3bf2416d77ba84f4?utm_source=chatgpt.com)

### 1.2 Reduced reference-counting overhead

The commit path previously cloned the shared database `Arc` into several panic/unwind guards. Version 1.2 changes those guards to borrow the database, avoiding reference-count increments and decrements for these temporary guards.

The improvement matters particularly for high-frequency, small transactions, where synchronization and object-management overhead can represent a substantial fraction of execution time.

Source: [https://github.com/surrealdb/surrealmx/commit/234950672de9798ffe209ce25e80e523b6594360](https://github.com/surrealdb/surrealmx/commit/234950672de9798ffe209ce25e80e523b6594360?utm_source=chatgpt.com)

### 1.3 How much faster is v1.2?

The release includes a new `examples/commit_scaling.rs`, which measures throughput across configurable thread counts, with defaults of 1, 4, 16, 32, 48, and 64 threads.

However, the release does not provide a verified v1.1-versus-v1.2 throughput comparison. Its optimizations should reduce commit-path overhead, but the magnitude of improvement, scaling behavior under contention, and effect on application workloads remain empirical questions.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/examples/commit_scaling.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/examples/commit_scaling.rs?utm_source=chatgpt.com)

## 2. Architecture and execution model

SurrealMX separates transaction ordering, read visibility, and physical application of writes. This separation allows commits to become visible without requiring every write to have been fully merged into the primary data structure first.

Its principal components are:

| Component                            | Function                                                  | Why it matters                                             |
| ------------------------------------ | --------------------------------------------------------- | ---------------------------------------------------------- |
| Concurrent adaptive radix tree (ART) | Ordered key index holding versioned values                | Efficient point lookups and byte-ordered range traversal   |
| Per-key MVCC version chains          | Preserve values visible to active snapshots               | Consistent reads without holding a database-wide read lock |
| Reader registry                      | Tracks active transaction snapshots                       | Determines which versions remain necessary                 |
| Commit ring                          | Orders and validates committing transactions              | Optimistic concurrency without a global transaction mutex  |
| Merge queue and ring                 | Tracks committed writes pending application or retirement | Separates logical visibility from physical tree updates    |
| Transaction-local write set          | Stages uncommitted changes                                | Atomic multi-key updates and read-your-writes              |
| Garbage collector                    | Reclaims obsolete versions and history                    | Controls memory growth under concurrent access             |
| Optional persistence layer           | Append-only logging and snapshots                         | Adds recoverability with configurable durability           |

Source: [https://github.com/surrealdb/surrealmx/tree/v1.2.0/src](https://github.com/surrealdb/surrealmx/tree/v1.2.0/src?utm_source=chatgpt.com)

### 2.1 Concurrent adaptive radix tree

The underlying `artmap` data structure provides an ordered byte-key index using an adaptive radix tree.

An ART indexes successive portions of a key, adapting node representations to the number of children. Compared with a conventional binary search tree, its structure exploits common prefixes and permits ordered traversal without maintaining a separate sorted-key index.

For SurrealMX, this provides several important properties:

- Point lookup through a shared-prefix index.
- Efficient traversal over contiguous lexicographic key ranges.
- Ordered forward and reverse access.
- Concurrent access without a database-wide tree lock.
- Fine-grained synchronization during structural modifications.

The distinction between lock-free reads and lock-free writes is important. The implementation does use fine-grained locking when modifying per-key version chains. The project's broad description of a lock-free engine should not be interpreted as guaranteeing that every writer operation is mathematically lock-free.

### 2.2 Multi-version value storage

Instead of replacing a committed value and immediately destroying the old one, each key can retain multiple committed versions.

A transaction beginning at logical version \\(v\\) reads the newest version whose commit timestamp is no greater than \\(v\\).

Conceptually:

```
Key: customer:1842

Version 108: "active"       ← newest
Version 93:  "pending"
Version 71:  "created"

Transaction at version 95  → "pending"
Transaction at version 110 → "active"
```

These version chains are managed behind atomic, epoch-protected references. Readers can inspect a consistent version without acquiring the writer's key lock.

This mechanism also means the storage engine's maintenance policy is part of its concurrency model. Older versions cannot be reclaimed merely because a newer value exists; they may still be visible to running transactions.

The version history is primarily an internal concurrency mechanism, not a general historical-query feature. SurrealMX 1.2 does not expose a public arbitrary-version time-travel query API.

### 2.3 Reader registration and memory reclamation

Transactions register a reader slot recording the snapshot they need. The cleanup system uses the oldest active snapshot to determine which version-chain entries and commit-history records are safe to reclaim.

The engine also uses epoch-based protection for concurrent memory access, so an object removed from a shared structure is not immediately deallocated while another reader could still reference it.

These mechanisms address different problems:

- Snapshot tracking determines whether an older logical value is still needed.
- Memory reclamation protection determines whether a removed in-memory object can safely be freed.

A long-lived transaction may retain historical data even if it performs little work. Consequently, read duration can affect memory consumption and commit-history retention.

### 2.4 Commit-ring validation and ordered visibility

A writable transaction initially accumulates changes in a transaction-local write set. At commit, SurrealMX places it in the commit ring, validates its conflicts against earlier concurrent commits, and publishes its result in the appropriate logical order.

Version 1.2 makes this ordering simpler by using the commit-ring slot sequence as the merge version.

The high-level progression is:

Snapshot + local writes

Commit-ring slot

Conflict validation

Ordered visibility

Optional log append

Apply to ART → retire merges → reclaim old versions

This is a simplified logical view rather than an exact scheduling diagram. Concurrent commits can overlap in their validation and application phases, and the persistence path depends on configuration.

One useful implication is that readers may need to consult both the ART and committed writes that have not yet been fully retired from the merge queue. SurrealMX handles this overlay internally.

The engine's read and scan APIs therefore do not require applications to manage separate committed-but-unapplied write buffers.

### 2.5 Why this is not simply a concurrent hash map

A concurrent hash map typically provides independently synchronized operations on keys. SurrealMX additionally provides:

- Consistent multi-key snapshots.
- Atomic multi-key commits.
- Optimistic validation of relationships between reads and writes.
- Detection of changes within previously scanned key ranges.
- Ordered range traversal.
- Savepoints and rollback.
- Optional recovery across process restarts.

These are fundamentally transactional capabilities, not just concurrent container operations.

Primary architecture reference: [https://surrealdb.com/static/whitepapers/surrealmx.pdf](https://surrealdb.com/static/whitepapers/surrealmx.pdf?utm_source=chatgpt.com)

The whitepaper describes SurrealMX 1.0, including its original separate commit-slot and merge-version allocation. The v1.2 source supersedes that specific part of the whitepaper's architecture.

## 3. Data model and storage API

### 3.1 Byte-oriented keys and values

SurrealMX stores opaque byte keys and byte values. It does not prescribe a record schema or application-level serialization format.

The `IntoBytes` trait provides two operations:

```
pub trait IntoBytes {
    fn as_slice(&self) -> &[u8];
    fn into_bytes(self) -> ByteSlice;
}
```

This permits borrowed access for comparison or lookup, while allowing the engine to take ownership when storing data.

Supported input types include byte slices and arrays, `Vec<u8>`, `ByteSlice`, `bytes::Bytes`, strings, boxed byte slices, and `Cow<[u8]>`.

`ByteSlice` is also re-exported from the crate and used throughout its retrieval APIs.

A consequential distinction is that keys are ordered by their byte representation, not by a higher-level semantic type. Compound key encodings must therefore preserve the order needed by the application.

For example, an ordered integer index generally needs fixed-width big-endian encoding or another order-preserving representation, rather than an arbitrary textual or little-endian encoding.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/kv.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/kv.rs?utm_source=chatgpt.com)

### 3.2 Direct database operations

`Database` provides lightweight single-operation access without explicitly constructing a transaction.

| API                          | Behavior                                               |
| ---------------------------- | ------------------------------------------------------ |
| `get(key)`                   | Read the latest committed value                        |
| `with_value(key, closure)`   | Inspect committed value bytes through a borrowed slice |
| `exists(key)`                | Test for existence                                     |
| `set(key, value)`            | Unconditional insert or replacement                    |
| `put(key, value)`            | Insert only when absent                                |
| `putc(key, value, expected)` | Conditionally replace based on an expected value       |
| `del(key)`                   | Delete a key                                           |
| `delc(key, expected)`        | Conditionally delete                                   |
| `scan`, `keys`, `total`      | Read an ordered range at a consistent snapshot         |

For direct point reads, `get`, `exists`, and `with_value` avoid transaction-pool checkout and snapshot-reader registration.

These point operations are documented as linearizable individually: they observe a value committed before or during the call. Two independent direct reads, however, are not guaranteed to observe the same multi-key snapshot.

For direct writes, `set` and `del` use auto-committed transactions with internal retries for write-write conflicts. Conditional operations and explicit multi-operation transactions have different retry obligations.

The conditional `putc` and `delc` APIs use an `Option` for the expected value: `None` represents an expected absence, while `Some(bytes)` represents an expected byte-for-byte match.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/db.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/db.rs?utm_source=chatgpt.com)

### 3.3 Transactional operations

A transaction is created through:

```
let mut tx = db.transaction(true);  // writable
let read_tx = db.transaction(false); // read-only
```

A writable transaction supports point access, batched retrieval through `getm`, writes, conditional writes, range operations, savepoints, commit, and cancellation.

An illustrative multi-key operation:

```
use surrealmx::Database;

let db = Database::new();
let mut tx = db.transaction(true);

tx.set("account:A", "updated_a")?;
tx.set("account:B", "updated_b")?;
tx.commit()?;
```

Both writes belong to the same atomic transaction. Their values are not separately committed merely because `set` was invoked twice.

The engine also supports read-your-writes: reads within a transaction account for its own staged modifications.

Transactions are pooled for reuse. Dropping a transaction releases its snapshot registration and returns eligible internal state to the pool, even when the transaction was not explicitly committed.

For clarity, explicitly calling `commit` or `cancel` still makes transaction lifetime and outcome easier to reason about.

## 4. Isolation, conflicts, and conditional coordination

Isolation is one of SurrealMX's most consequential features. It distinguishes the engine from simpler in-memory caches and key-value containers.

### 4.1 Supported isolation modes

`IsolationLevel` exposes two variants:

| Mode                            | Conflict validation                                   | Important consequence                                                            |
| ------------------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------- |
| `SnapshotIsolation`             | Write-write conflicts                                 | High concurrency, but read-dependent write skew can remain possible              |
| `SerializableSnapshotIsolation` | Write-write, read-write, and relevant range conflicts | Prevents write skew and detects phantoms affecting committed update transactions |

Writable transactions default to `SerializableSnapshotIsolation` in v1.2. The default is established in both transaction creation and transaction-pool reset.

A caller can explicitly select the weaker mode:

```
let mut tx = db.transaction(true).with_snapshot_isolation();
```

Or explicitly retain the stronger one:

```
let mut tx = db.transaction(true)
    .with_serializable_snapshot_isolation();
```

Read-only transactions obtain a consistent snapshot and do not perform the writable transaction's commit-conflict validation.

The project describes its default update-transaction guarantee as strict serializability. The implementation establishes this through commit-order validation and ordered publication; it does not rely on blocking every transaction behind a global lock.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)

### 4.2 Preventing write skew

Consider two transactions enforcing an invariant that at least one of two service nodes must remain available.

Both transactions read the initial state and then independently disable different nodes.

Under ordinary snapshot isolation, their write sets do not overlap. Both could commit even though the combined result violates the invariant.

Under SurrealMX's default serializable update mode, the read of a key subsequently modified by the other transaction is part of conflict validation. One transaction can be rejected with `KeyReadConflict`.

The application must then restart that logical operation against a new snapshot.

This is particularly useful for constraints involving multiple records, because correctness does not depend exclusively on overlapping write keys.

### 4.3 Range conflicts and phantom detection

Point-read validation alone is insufficient for predicates over key ranges.

Suppose a transaction scans a range representing all active reservations for a resource, finds none, and attempts to create a reservation.

A concurrent transaction could insert a previously absent key within that range.

SurrealMX tracks scans made by serializable writable transactions and checks them against concurrent writes. This allows the conflict detector to identify inserts or modifications affecting the scanned range, rather than only modifications to keys that originally existed.

The range boundaries and byte encoding therefore influence concurrency semantics. A poorly designed key layout can make otherwise related records difficult to cover with one validated range.

### 4.4 `get_for_update` is optimistic

The method:

```
tx.get_for_update(key)
```

provides stronger validation for a specific key in a writable transaction, even when the transaction is configured for ordinary snapshot isolation.

Despite the name, this is not a conventional blocking row lock.

The key is recorded for conflict validation. If a concurrent transaction changes it after the current snapshot, the eventual commit can fail with `KeyReadConflict`.

The guarantee applies to that key; invoking it does not automatically upgrade all other plain reads and scans to serializable validation.

An easily overlooked capability is that a transaction which has used `get_for_update` can perform meaningful commit validation even if it ultimately contains no writes.

### 4.5 Conflict detection and retry behavior

The major retryable conflict errors are:

- `KeyReadConflict`: a value or tracked read condition became inconsistent with concurrent committed writes.
- `KeyWriteConflict`: a conflicting write, or a commit history that can no longer be validated safely.

The implementation records sorted write keys and uses adaptive filtering to reduce unnecessary comparisons during commit validation.

However, optimistic validation does not make contention disappear. Repeated updates to hot keys can produce frequent aborts, and transactions with long lifetimes may need to check substantial commit histories.

A failed transaction should generally be retried as a new transaction, rerunning the relevant reads and decision logic, rather than replaying only its final writes.

### 4.6 Savepoints and partial rollback

Writable transactions provide stackable savepoints:

```
tx.set_savepoint()?;
tx.set("item:1", "temporary")?;

tx.rollback_to_savepoint()?;
```

Or retain changes made inside a nested scope:

```
tx.set_savepoint()?;
tx.set("item:2", "accepted")?;

tx.release_savepoint()?;
```

Savepoints use an undo journal over the transaction's staged writes.

A subtle but important detail: rolling back to a savepoint does not erase reads and scans performed since it was created. Those accesses remain relevant to conflict detection.

This favors correctness, but it can make transaction validation more conservative: a rolled-back exploratory branch can still contribute to a later retryable conflict.

Savepoints are anonymous and follow a stack discipline. Every creation should be paired with precisely one release or rollback.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.2.0/README.md?utm_source=chatgpt.com)

## 5. Ordered scans, iterators, and cursors

Range access is a major native capability of SurrealMX, not an auxiliary operation implemented by filtering the complete keyspace.

The adaptive radix tree provides ordered access to byte keys. SurrealMX then combines tree entries, any relevant unretired committed writes, and a transaction's local write set to produce the transactionally visible result.

### 5.1 Range operation families

| API family           | Methods                                        | Important properties                                          |
| -------------------- | ---------------------------------------------- | ------------------------------------------------------------- |
| Materialized results | `scan`, `keys`, `scan_reverse`, `keys_reverse` | Returns owned result vectors; forward or reverse traversal    |
| Counting             | `total`                                        | Counts matching visible keys without returning their values   |
| Callback traversal   | `scan_for_each`, `scan_with`, `keys_for_each`  | Avoids allocating a result vector; supports early termination |
| Reusable buffers     | `scan_into`, `keys_into`                       | Reuses caller-owned vector capacity                           |
| Lazy iterators       | `scan_iter`, `keys_iter`, reverse variants     | Integrates with Rust's iterator interface                     |
| Seekable cursor      | `cursor`                                       | Explicit positioning and bidirectional navigation             |

The direct `Database` API includes forward range scans and callback/buffer variants. The explicit `Transaction` API additionally exposes reverse scans, seekable cursors, and lazy iterators.

Ranges are represented using `std::ops::Range<K>`, with inclusive starting and exclusive ending keys.

For example:

```
let tx = db.transaction(false);

let rows = tx.scan(
    "order:1000".."order:2000",
    None,
    Some(100),
)?;
```

This retrieves up to 100 visible key-value pairs whose encoded keys fall within the specified half-open range.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)

### 5.2 Borrowed-value scanning

The most efficient APIs are often those which avoid creating intermediate result objects.

`scan_with` passes each key and borrowed value bytes into a closure:

```
let mut matches = 0usize;

db.scan_with(
    "item:0000".."item:9999",
    None,
    Some(1000),
    |_key, value| {
        if value.starts_with(b"active") {
            matches += 1;
        }
        true
    },
)?;
```

Returning `false` from the closure stops traversal.

The value is supplied as `&[u8]`, allowing inspection without cloning or allocating an owned value. The borrowed reference is valid within the callback; retaining it outside that lifetime requires an owned copy.

There are two performance qualifications.

First, zero-allocation refers to the result-processing path, not an unconditional absence of allocations anywhere in the transaction, index, or merge machinery.

Second, the direct scan implementation has a fast path when the relevant committed merges have already been applied. If an unretired merge remains, or a thread-local scan slot is unavailable, it can fall back to a pooled read transaction.

Thus, the benchmark advantage of closure-based scanning depends partly on the state of the engine's commit pipeline.

### 5.3 Reusing result buffers

For repeated scans with similar result sizes, SurrealMX supplies:

```
let mut buffer = Vec::new();

db.scan_into(
    "item:0000".."item:1000",
    None,
    Some(100),
    &mut buffer,
)?;
```

The engine clears the buffer and reuses its capacity where possible.

This reduces repeated vector allocation, but it is not equivalent to borrowed-value scanning: result keys and values still need owned representations.

For applications that consume rows immediately, callbacks can be more efficient. For applications that need retained result batches, reusable buffers are a useful middle ground.

### 5.4 Bidirectional cursors and seek operations

The `Cursor` API supports:

| Method               | Purpose                                                |
| -------------------- | ------------------------------------------------------ |
| `seek_to_first()`    | Position at the first key in the range                 |
| `seek_to_last()`     | Position at the last key in the range                  |
| `seek(key)`          | Find the first key greater than or equal to the target |
| `seek_for_prev(key)` | Find the last key less than or equal to the target     |
| `next()` / `prev()`  | Advance in either direction                            |
| `valid()`            | Check whether the cursor is positioned on an entry     |
| `key()` / `value()`  | Borrow the current entry                               |
| `exists()`           | Check whether the current entry is not deleted         |

A cursor begins unpositioned.

```
let tx = db.transaction(false);
let mut cursor = tx.cursor("a".."z")?;

cursor.seek(b"m");

while cursor.valid() {
    if let Some(key) = cursor.key() {
        // Process key.
    }
    cursor.next();
}
```

Cursors retain the transaction's snapshot and incorporate committed merge-queue contents relevant to that snapshot.

A particularly useful implementation detail is that SurrealMX retains a persistent merge iterator across sequential navigation. It does not need to rebuild the full merged view for every `next()` call.

Changing direction or performing a new seek can require rebuilding iterator state.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs?utm_source=chatgpt.com)

### 5.5 Prefix indexing and keyset pagination

Ordered byte keys allow applications to implement several indexing patterns directly:

- Group related records under a shared prefix.
- Sort entities by a timestamp embedded in the key.
- Represent composite indexes through encoded tuples.
- Store forward and inverse lookup mappings as separately maintained key families.
- Navigate ordered work queues and event sequences.

For example, a key layout might encode:

```
event:<tenant-id>:<timestamp>:<event-id>
```

With an order-preserving timestamp encoding, a contiguous range can retrieve events for a tenant in chronological order.

This can avoid building a separate in-memory B-tree or sorted-map index.

However, SurrealMX does not automatically maintain secondary indexes. If one logical record is represented by several index keys, their consistency is the application's responsibility. A single transaction can update all of them atomically.

Another practical distinction concerns pagination. `skip` traverses and discards entries, so large offsets are not constant-time. The project's benchmarks demonstrate a substantial throughput difference between short limit scans and scans skipping thousands of entries.

For deep pagination, resuming from an encoded last-seen key can avoid repeatedly traversing earlier pages.

## 6. Advanced concurrency and performance mechanisms

Several of SurrealMX's more distinctive capabilities lie below its public API. They explain why its design differs from either a conventional B-tree database or an ordinary concurrent collection.

### 6.1 Adaptive Bloom-filter conflict detection

The commit queue uses multiple levels of inexpensive rejection before performing exact write-set comparisons.

The implementation uses a 4,096-bit, 512-byte Bloom filter with three derived hashes based on `xxHash3`.

However, it does not allocate one unconditionally for every transaction.

For write sets containing two or fewer keys, the engine uses direct comparisons instead.

For larger write sets, the conflict-detection path can:

1. Reject candidates whose minimum and maximum keys cannot overlap.
2. Use Bloom filters to rule out possible intersections.
3. Fall back to exact key comparisons when the filter indicates possible overlap.

The exact checks preserve correctness: Bloom-filter false positives may increase validation work, but do not by themselves become transaction conflicts.

For write-write intersection, the engine also exploits sorted write sets with a two-pointer merge.

For read-write conflicts, it chooses whether to iterate the read set or write set based on their relative sizes.

These details are valuable because the common case is often not a large transaction with thousands of modifications, but a high volume of small transactions touching a few keys.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/queue.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/queue.rs?utm_source=chatgpt.com)

### 6.2 Fixed-size commit and merge rings

In a normal non-Miri build, both the commit ring and merge ring use a default capacity of 65,536 slots.

Each ring is indexed using a power-of-two mask, avoiding general-purpose modulo division for slot selection.

The commit ring maintains separate contiguous progress watermarks for publication, completion, and retirement.

The merge ring tracks versioned writes that are awaiting physical application or in-order retirement.

Version 1.2 does not eliminate the merge ring; it eliminates the separate version-number allocation protocol.

A slot cannot be unsafely overwritten while an earlier occupant still needs to complete the relevant stage. Under sufficient pressure, ring reuse can therefore involve waiting for outstanding work.

The capacities are internal constants in the tagged implementation, not exposed through `DatabaseOptions`.

Sources:

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/ring.rs?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/merge_ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/merge_ring.rs?utm_source=chatgpt.com)

### 6.3 Out-of-order physical application

SurrealMX permits the logical visibility order and physical tree-application order to differ.

This introduces an important corner case:

1. Transaction A commits an older version of a key.
2. Transaction B commits a newer deletion.
3. B's deletion is applied to the tree before A's write is physically applied.

If the engine immediately discarded the deletion tombstone, the late application of A could incorrectly resurrect the key.

The version-chain implementation therefore supports insertion of older versions into their correct sorted positions and retains relevant tombstones until reclamation is safe.

This is an easily overlooked part of the engine's correctness model. It is necessary precisely because commits and physical application can overlap.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/versions.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/versions.rs?utm_source=chatgpt.com)

### 6.4 Specialized single-version storage

Most keys do not need multiple resident versions indefinitely.

The version-container implementation has three representations:

| Representation            | Internal meaning           |
| ------------------------- | -------------------------- |
| `Empty`                   | No retained version        |
| `Single(Version)`         | One inline version         |
| `Chain(ThinVec<Version>)` | Multiple retained versions |

The `Single` representation avoids allocating a separate version vector. When a reader pins older history, the container can expand into a `ThinVec` chain; after reclamation, it can return to the single-version representation.

The implementation reports a 40-byte `Versions` container and describes the single-version case as its steady-state optimization.

These figures refer to the version container, not the complete memory footprint of an indexed record, including its key, value, ART nodes, and allocator overhead.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/versions.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/versions.rs?utm_source=chatgpt.com)

### 6.5 Cache-line-padded reader registration

SurrealMX's reader registry is designed to reduce contention from multiple threads repeatedly modifying shared snapshot metadata.

Each registered transaction has a slot. Those slots are cache-line padded so neighboring transactions do not unnecessarily contend over the same cache line.

Slots are allocated in fixed chunks and reused after release. Pooled transactions can retain registered, unpinned slots between uses.

The registry is bounded rather than unlimited: the tagged implementation has 1,024 chunks with 1,024 slots each, and registration beyond its maximum panics.

In most workloads, the more immediate concerns are the number of simultaneously active snapshots and their duration, rather than that upper bound.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/readers.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/readers.rs?utm_source=chatgpt.com)

## 7. Persistence, durability, and crash recovery

SurrealMX is memory-first, but persistence is a substantial built-in subsystem rather than an external adapter that merely serializes the entire map on shutdown.

All live data remains memory-resident. Persistence provides recovery, not a disk-backed working set or eviction mechanism.

### 7.1 Persistence configuration

Persistence is configured through `PersistenceOptions` when constructing the database.

```
use std::time::Duration;
use surrealmx::{
    AolMode,
    CompressionMode,
    Database,
    DatabaseOptions,
    FsyncMode,
    PersistenceOptions,
    SnapshotMode,
};

let persistence = PersistenceOptions::new("./database")
    .with_aol_mode(AolMode::SynchronousOnCommit)
    .with_fsync_mode(FsyncMode::EveryAppend)
    .with_snapshot_mode(
        SnapshotMode::Interval(Duration::from_secs(300))
    )
    .with_compression(CompressionMode::Lz4);

let db = Database::new_with_persistence(
    DatabaseOptions::default(),
    persistence,
)?;
```

This example configures synchronous append logging, synchronization of logged writes to storage, periodic snapshots, and LZ4-compressed snapshots.

Persistence construction is native-only; the relevant APIs are conditionally compiled out on `wasm32`.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs?utm_source=chatgpt.com)

### 7.2 Durability configuration matrix

Three independent configuration dimensions shape durability.

| Dimension       | Variants                                                  | Effect                                                            |
| --------------- | --------------------------------------------------------- | ----------------------------------------------------------------- |
| Append-only log | `Never`, `SynchronousOnCommit`, `AsynchronousAfterCommit` | Whether and when individual commits are logged                    |
| Fsync           | `Never`, `EveryAppend`, `Interval(Duration)`              | When operating-system buffered writes are synchronized to storage |
| Snapshots       | `Never`, `Interval(Duration)`                             | Whether full point-in-time images are periodically persisted      |

The defaults for `PersistenceOptions` are `AolMode::Never`, `SnapshotMode::Never`, and `FsyncMode::Never`. Simply supplying a persistence directory without enabling logging or snapshots does not establish durable storage.

The broad durability implications are:

| Configuration                       | Meaning                                                                                   |
| ----------------------------------- | ----------------------------------------------------------------------------------------- |
| Memory only                         | State disappears on process termination                                                   |
| Snapshot only                       | Recovery is limited to completed snapshots                                                |
| Asynchronous AOL                    | Acknowledged commits may be lost before their log write completes                         |
| Synchronous AOL, no fsync           | The log is written before acknowledgement, but OS or power failure can lose buffered data |
| Synchronous AOL, periodic fsync     | Durability follows successful sync points rather than every individual commit             |
| Synchronous AOL, fsync every append | Strongest documented local crash-durability configuration                                 |

The final guarantee assumes the operating system, filesystem, and storage device honor their synchronization contracts.

An especially important distinction: `AsynchronousAfterCommit` with `EveryAppend` does not mean that an acknowledged transaction was already synchronized when its `commit()` returned. The asynchronous submission and eventual persistence completion are separate events.

### 7.3 Group commit

The synchronous AOL path includes a group-commit coordinator.

Group commit is useful because individually synchronizing each small transaction can cause substantial I/O overhead. Coordinating concurrent appends allows the engine to amortize persistence costs over groups of commits.

This is particularly relevant when many threads produce small write transactions. In-memory transaction validation may be highly concurrent, while durable storage must ultimately coordinate writes to a shared log.

The exact benefit depends on transaction arrival rate, write sizes, filesystem behavior, and fsync latency.

### 7.4 Point-in-time snapshots

SurrealMX creates snapshots by capturing a consistent logical cut of the database.

The snapshot procedure:

1. Coordinates with the append-only log to identify an appropriate cutoff.
2. Waits until relevant committed versions have been applied to the data structure.
3. Pins a consistent snapshot version.
4. Streams the visible records to a temporary snapshot file.
5. Completes and synchronizes that file.
6. Atomically renames it into place.
7. Preserves log entries needed for commits occurring after the snapshot cutoff.

The streaming design avoids requiring the entire database to be duplicated into a second in-memory serialization buffer.

The API also exposes:

```
db.persistence()
    .expect("persistence configured")
    .snapshot()?;
```

This allows a manually requested snapshot independently of the periodic schedule.

Snapshots are not free. They must traverse and serialize the visible dataset, and active version pins can affect reclamation while the snapshot proceeds.

### 7.5 Compression

SurrealMX supports two snapshot compression modes:

- `CompressionMode::None`
- `CompressionMode::Lz4`

Compression applies to the snapshot representation, not automatically to every live in-memory value or append-log record.

LZ4 can reduce snapshot size and associated I/O at the expense of compression/decompression work. Whether this accelerates recovery depends on the balance between storage throughput and available CPU capacity.

### 7.6 Recovery and data-integrity behavior

Each transaction is stored in a checksummed append-only-log frame.

Recovery loads the snapshot, replays eligible log records, and discards an incomplete trailing frame if a crash interrupted the last append.

The engine distinguishes that recoverable torn tail from corruption occurring in the middle of the log. Mid-log corruption is reported as an error rather than silently treating all subsequent records as disposable.

Version 1.2 also reseeds the commit ring using the maximum recovered version, ensuring that newly committed transactions continue above the persisted history.

This is important because replayed deletes and older snapshot boundaries must not allow new transactions to reuse sequence numbers that could invalidate version ordering.

The implementation also compacts replay-created version chains down to their necessary live versions before normal transactions begin.

### 7.7 Failure and ambiguous commit outcomes

Persistence failures are not treated as ordinary transaction conflicts.

The engine may enter a poisoned or stopped state after unrecoverable persistence or internal failures, exposed through:

```
db.is_poisoned()
db.check_poisoned()
```

The error `TxCommitNotPersisted` warrants special treatment. The source explicitly acknowledges that a failed append may have partially or fully reached the log, leaving the transaction's recovery outcome uncertain.

A caller should not automatically interpret such a failure as proof that no durable side effect occurred. Recovery and application-level idempotency may be necessary to reconcile ambiguous outcomes.

Primary persistence source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs?utm_source=chatgpt.com)

## 8. Garbage collection, resource controls, and observability

### 8.1 Inline reclamation and tracked sweeps

SurrealMX combines two complementary forms of version reclamation.

Inline reclamation occurs while a commit is already updating the relevant key's version chain. Obsolete versions can be trimmed without scheduling an independent full-database scan.

Tracked reclamation handles cases where a version cannot be removed yet, typically because an active reader still needs it.

The engine tracks those keys as garbage candidates. Background sweeps visit the tracked candidates rather than traversing the entire dataset.

This makes the background cost proportional to the number of keys requiring deferred reclamation, not automatically to every stored key.

The public methods are:

| Method             | Behavior                                                        |
| ------------------ | --------------------------------------------------------------- |
| `run_gc_tracked()` | Sweep tracked candidate keys                                    |
| `run_gc()`         | Perform a full datastore garbage-collection scan                |
| `run_cleanup()`    | Retire commit-history entries no longer required for validation |

A full scan remains available when a deeper cleanup is appropriate, but it is not the ordinary steady-state mechanism.

Long-lived snapshots can increase both retained version memory and the tracked candidate set.

Sources:

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/gc_candidates.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/gc_candidates.rs?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/inner.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/inner.rs?utm_source=chatgpt.com)

### 8.2 Database configuration

`DatabaseOptions` supplies operational controls and presets.

| Option                | Default    | Function                                              |
| --------------------- | ---------- | ----------------------------------------------------- |
| `pool_size`           | 512        | Maximum number of reusable pooled transactions        |
| `enable_gc`           | `true`     | Enable background candidate sweeps                    |
| `gc_interval`         | 10 seconds | Period between garbage-collection sweeps              |
| `enable_cleanup`      | `true`     | Enable background commit-history cleanup              |
| `cleanup_interval`    | 1 second   | Commit queue cleanup interval                         |
| `reset_threshold`     | 100        | Threshold controlling pooled transaction-state resets |
| `max_transaction_age` | `None`     | Optional maximum transaction lifetime                 |
| `memory_limit`        | `None`     | Optional approximate data-memory limit                |

Builders include `with_pool_size`, `with_gc_interval`, `with_cleanup_interval`, `with_max_transaction_age`, `with_memory_limit`, and `with_all_workers_disabled`.

Three convenience presets are available:

- `with_high_performance()` — larger transaction pool and adjusted worker intervals.
- `with_low_resource()` — smaller pool and less frequent background work.
- `with_reduced_memory()` — smaller pool and more frequent reclamation.

These presets encode tradeoffs, not guaranteed universally optimal configurations.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/options.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/options.rs?utm_source=chatgpt.com)

### 8.3 Transaction lifetime limits

`with_max_transaction_age(Duration)` introduces a useful protection against transactions retaining snapshots indefinitely.

When an operation discovers that the configured age has been exceeded, it returns `Error::TxTooOld` and releases the transaction's reader pin.

This is an operation-time check, not a guarantee that an idle transaction is forcibly interrupted at the exact deadline.

The distinction matters for transactions held by stalled tasks or long-running application callbacks.

### 8.4 Memory limits are approximate

`with_memory_limit(bytes)` configures a limit based on tracked approximate data memory.

This should not be interpreted as a strict operating-system resident-set limit.

The tracker is designed around key/value versions and uses sharded atomic counters to minimize contention. It does not encompass every allocator, transaction, index, queue, and thread-related allocation.

Furthermore, concurrent writers can observe approximate counters at overlapping times.

A memory limit can provide useful application-level pressure, but it is not a substitute for external memory budgets where out-of-memory failure must be prevented.

### 8.5 Persistence backpressure

Asynchronous persistence has optional bounds on queue depth and queued bytes:

```
.with_max_async_queue_depth(...)
.with_max_async_queue_bytes(...)
```

When the configured queue is full, write operations can return `Error::AsyncQueueFull`.

This avoids treating the asynchronous log writer as an infinite buffer.

Unlike ordinary commit-conflict errors, the source preserves the transaction as open for some pre-commit resource failures, including memory-limit and asynchronous-queue-capacity checks. Callers can therefore respond to those conditions differently from an already-aborted conflict.

### 8.6 Runtime statistics

`Database::stats()` returns a `DatabaseStats` structure with:

| Metric                          | Interpretation                               |
| ------------------------------- | -------------------------------------------- |
| `live_keys`                     | Number of live indexed keys                  |
| `live_versions`                 | Estimated number of retained versions        |
| `approx_data_bytes`             | Approximate memory used by tracked data      |
| `gc_candidate_keys`             | Number of keys awaiting reclamation work     |
| `oldest_pinned_version`         | Oldest active snapshot, if present           |
| `active_readers`                | Number of active pinned readers              |
| `commit_ring_occupancy`         | Unretired commit-ring entries                |
| `merge_ring_occupancy`          | Unretired merge entries                      |
| `async_queue_depth`             | Outstanding asynchronous persistence batches |
| `async_queue_bytes`             | Estimated queued persistence data            |
| `async_dropped_batches`         | Failed or dropped asynchronous batches       |
| `is_poisoned` / `poison_reason` | Fatal engine-state diagnostics               |

These metrics are especially useful in combination.

For example, rising retained versions together with an old pinned snapshot suggests reclamation is being delayed by a reader. Rising asynchronous queue depth suggests persistence is failing to keep up with incoming commits.

Such interpretations are diagnostic possibilities, not proofs of a single underlying cause.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/stats.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/stats.rs?utm_source=chatgpt.com)

### 8.7 Conflict tracing

The crate exports:

```
surrealmx::LOG_TARGET_CONFLICTS
```

with value `surrealmx::conflicts`.

In debug builds, conflict diagnostics use this tracing target and can identify conflicting keys.

An example filter is:

```
RUST_LOG="info,surrealmx::conflicts=debug"
```

The conflict-key tracing is conditional on `debug_assertions`. It should not be assumed available in an ordinary optimized release build.

## 9. Platform support and integration boundaries

### 9.1 Native Rust applications

SurrealMX is an embedded synchronous Rust library.

It has no standalone server requirement for direct use. Calls operate inside the hosting process, and the database instance can be shared among concurrent Rust tasks or threads through appropriate shared ownership.

The synchronous API is a consequential interface choice. A Rust async application can use the library, but a synchronous operation that performs substantial scans, commit validation, or durable I/O may occupy the calling executor thread. Integrating such operations with an async runtime requires consideration of blocking and task scheduling.

The crate has no native SQL parser, external RPC protocol, or asynchronous query interface in its v1.2 public surface.

### 9.2 WebAssembly

The codebase supports WebAssembly builds.

Core memory-resident transactional behavior is available, while the native persistence implementation is conditionally excluded on `wasm32`.

This should not be confused with SurrealDB's separate browser persistence capability using SurrealKV over the Origin Private File System.

SurrealMX's browser capability is an embedded in-memory engine, not evidence of built-in browser filesystem persistence.

### 9.3 SurrealMX versus SurrealDB

SurrealMX provides a low-level transactional KV foundation. SurrealDB supplies a higher-level database system above its storage backends.

| Capability                            | SurrealMX directly      | Via SurrealDB's higher layers                      |
| ------------------------------------- | ----------------------- | -------------------------------------------------- |
| Byte-key storage and retrieval        | Yes                     | Yes                                                |
| Atomic multi-key transactions         | Yes                     | Yes                                                |
| Ordered key scans                     | Yes                     | Yes                                                |
| MVCC                                  | Yes                     | Yes                                                |
| Document data model                   | No native schema        | Yes                                                |
| SurrealQL                             | No                      | Yes                                                |
| Graph relationships and traversal     | No native graph model   | Yes                                                |
| Property-based secondary indexes      | Not automatically       | Yes                                                |
| Vector indexing and search            | No native vector search | Yes                                                |
| Record permissions and authentication | No                      | Yes                                                |
| Server/network protocol               | No                      | Yes                                                |
| Distributed database operation        | No                      | Separate SurrealDB deployment/storage capabilities |

SurrealDB 4.0 identifies SurrealMX as its in-memory storage backend. Its `surrealdb-kvs-mem` adapter integrates the engine with SurrealDB's internal storage abstraction.

That adapter is explicitly an internal API, not a stable replacement for the public SurrealDB Rust SDK.

At the time of research, the published `surrealdb-kvs-mem` 4.0.0 dependency was declared as `surrealmx ^1.1.0`. Cargo's caret compatibility can permit 1.2, but this alone does not establish that a particular SurrealDB binary has upgraded its locked dependency to v1.2.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Sources:

[https://surrealdb.com/docs/build/embedding/storage-engines](https://surrealdb.com/docs/build/embedding/storage-engines?utm_source=chatgpt.com)

[https://docs.rs/crate/surrealdb-kvs-mem/4.0.0](https://docs.rs/crate/surrealdb-kvs-mem/4.0.0?utm_source=chatgpt.com)

### 9.4 SurrealMX versus SurrealKV

The two engines optimize for different storage models.

| Dimension                    | SurrealMX                          | SurrealKV                                             |
| ---------------------------- | ---------------------------------- | ----------------------------------------------------- |
| Primary storage              | Memory                             | Disk                                                  |
| Design priority              | Low-latency concurrent access      | Durable persistent storage                            |
| Dataset memory requirement   | Working dataset resident in memory | Disk-backed dataset                                   |
| Transactions                 | Yes                                | Yes                                                   |
| Persistence                  | Optional AOL and snapshots         | Core storage function                                 |
| Historical/versioned queries | Not exposed                        | Supported in the appropriate SurrealDB configurations |
| Long-term disk compaction    | Not a disk-oriented LSM            | LSM-oriented maintenance                              |
| Browser persistent storage   | No native SurrealMX persistence    | SurrealKV over OPFS through SurrealDB                 |

SurrealDB's current documentation explicitly states that SurrealMX does not support versioned queries and that an in-memory server requiring them should use a suitable SurrealKV-backed deployment. It also cautions that SurrealMX persistence is a recovery convenience for resident data, not an equivalent replacement for a disk-oriented primary store.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



Sources:

[https://surrealdb.com/docs/running/in-memory](https://surrealdb.com/docs/running/in-memory?utm_source=chatgpt.com)

[https://surrealdb.com/docs/running/file-backed](https://surrealdb.com/docs/running/file-backed?utm_source=chatgpt.com)

### 9.5 Higher-level data structures and integrations

SurrealMX's byte-oriented KV interface can serve as the transactional foundation for more specialized structures.

For example, one can maintain an entity key and several ordered index keys in the same write transaction. Similarly, queues, dependency mappings, graph adjacency lists, counters, and materialized lookup tables can be represented through structured key namespaces.

The advantage is that a native database transaction can replace some custom coordination between concurrent maps and indexes.

The limitation is equally important: the representation, encoding, constraints, index maintenance, and query execution still belong to the surrounding application or a higher-level database library.

SurrealMX does not automatically turn those encoded keys into a complete domain-aware database.

## 10. Performance evidence and interpretation

The most widely published SurrealMX benchmarks originate with its October 2026 v1.0 introduction. They remain useful for understanding the engine's performance characteristics, but they are not measurements of the v1.2 commit-path improvements.

### 10.1 Published baseline throughput

Vendor benchmark · SurrealMX 1.0-era measurements · Operations per second

SurrealMX baseline throughput by operation

06M12M18M24MPoint readCreateUpdateDeleteLimit scan

The published environment was an AMD Ryzen Threadripper 9970X, 128 GB DDR5 RAM, 15 million keys, 48 worker threads, and 128 clients, using `crud-bench`. Each reported value was the best of three runs.

The point-read number used `with_value`, which avoids owning a copied value.

The limit-scan number refers to scan operations per second for a limit-100 workload, not a general rate for arbitrary scans.

The benchmark also reported:

| Scan workload                   | SurrealMX scans/sec |
| ------------------------------- | ------------------- |
| Limit 100 key-value pairs       | 18,031,692          |
| Limit 100 keys only             | 15,524,603          |
| Skip 5,000 then limit 100 pairs | 232,121             |
| Skip 5,000 then limit 100 keys  | 253,039             |
| Full-range count                | 17.3                |

These figures illustrate why it is important to distinguish short, cache-friendly range scans from large-offset or whole-database traversal.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Sources:

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.2.0/README.md?utm_source=chatgpt.com)

[https://github.com/surrealdb/crud-bench](https://github.com/surrealdb/crud-bench?utm_source=chatgpt.com)

### 10.2 Memory and concurrency characteristics

The same published benchmark reported approximately 15.9 GiB resting process memory for SurrealMX with 15 million stored documents, increasing to a 32.0 GiB peak during concurrent work.

This is consistent with an MVCC architecture that retains additional versions while active transactions can still observe them.

The memory profile should not be generalized into a fixed number of bytes per key. Key lengths, values, index structure, transaction lifetimes, concurrency, and allocator behavior all change the result.

### 10.3 What the comparative benchmarks establish

The vendor reported substantial advantages over its selected in-memory LMDB and RocksDB configurations for point reads, writes, and short scans.

However, the comparison has important conditions:

- The engines used workload-specific settings and transaction APIs.
- LMDB and RocksDB were configured for memory-resident operation.
- Durability settings were not equivalent to fully synchronized production storage.
- Some scan workloads favored LMDB, particularly large-offset scans and full counts.
- A single benchmark configuration cannot establish performance across all key sizes, contention patterns, or hardware topologies.

The results are credible evidence of the potential of SurrealMX's design in that workload. They should not be interpreted as universal superiority to every disk-backed or memory-resident database.

### 10.4 Expected v1.2 performance effects

Version 1.2 changes are concentrated on the commit protocol.

The most plausible benefits are reduced synchronization and allocation overhead for frequent commits, especially where many threads perform small, independent transactions.

They are less likely to materially transform operations dominated by full-range traversal, user code inside scan callbacks, large-value serialization, or storage-device fsync latency.

No new controlled measurements were found quantifying those differences for v1.2.0. This conclusion is an architectural inference from the source diff, not an upstream benchmark result.

## 11. Testing, reliability, and engineering guarantees

SurrealMX includes several testing approaches addressing the kinds of bugs that are difficult to reproduce in concurrent storage engines.

### 11.1 Deterministic simulation

The project's published material reports deterministic simulation testing against a reference model across 25 million operations without divergence.

The source tree includes simulation generators, a test harness, and reference-model code.

This provides evidence of deliberate validation of transaction histories rather than relying solely on conventional unit tests.

However, a reported zero-divergence run does not prove correctness across every possible interleaving, platform, or failure mode.

Sources:

[https://github.com/surrealdb/surrealmx/tree/v1.2.0/src/test/sim](https://github.com/surrealdb/surrealmx/tree/v1.2.0/src/test/sim?utm_source=chatgpt.com)

[https://surrealdb.com/static/whitepapers/surrealmx.pdf](https://surrealdb.com/static/whitepapers/surrealmx.pdf?utm_source=chatgpt.com)

### 11.2 Focused integration tests

The tagged repository includes tests covering:

- Concurrent writes, conflicts, and deletes.
- Snapshot consistency and isolation.
- Conditional operations.
- Concurrent range scans and prefix keys.
- Savepoints and transaction lifecycle.
- Garbage collection and bounded memory.
- Append-log recovery and persistence failures.
- Large transactions and edge-case iteration.

Version 1.2 adds direct tests for the new commit-ring sequencing protocol, including aborted commits interleaved with successful commits and ring reseeding across multiple laps.

Source: [https://github.com/surrealdb/surrealmx/tree/v1.2.0/tests](https://github.com/surrealdb/surrealmx/tree/v1.2.0/tests?utm_source=chatgpt.com)

### 11.3 Model checking and unsafe-code discipline

The crate's synchronization module supports a conditional `loom` implementation for exploring concurrent interleavings.

It also has targeted epoch-protected pointer operations and a crate-level lint policy that denies ordinary introduction of `unsafe` outside the explicitly permitted synchronization implementation.

This is more precise than saying the engine contains no unsafe Rust. Some unsafe machinery remains necessary for its epoch-protected pointer handling, and dependencies have their own implementations.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/sync.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/sync.rs?utm_source=chatgpt.com)

## 12. Consequential boundaries and tradeoffs

The following distinctions determine which architectural problems SurrealMX can solve natively.

| Concern                | SurrealMX 1.2 behavior                                                              | Implication                                                                   |
| ---------------------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Dataset size           | Entire active dataset is memory-resident                                            | Not a general solution for data exceeding RAM                                 |
| Historical reads       | MVCC versions retained for active transactions, no public arbitrary-version queries | Not a temporal database                                                       |
| Indexes                | One native ordered byte-key index                                                   | Secondary indexes must be encoded and maintained                              |
| Queries                | Key lookups and ranges                                                              | Joins, aggregation engines, SQL, and graph traversal are outside its core API |
| High contention        | Optimistic aborts and retries                                                       | Hot keys can limit effective throughput                                       |
| Long transactions      | Pin versions and conflict history                                                   | Can increase memory and validation costs                                      |
| Persistence            | Optional log and snapshot system                                                    | Durability settings materially affect performance and recovery guarantees     |
| Distributed systems    | Embedded single-engine concurrency                                                  | No built-in network consensus, replication, or cluster coordination           |
| Asynchronous execution | Synchronous Rust API                                                                | Blocking work needs suitable integration with an async executor               |
| Memory constraints     | Approximate data accounting                                                         | Does not enforce an exact process RSS limit                                   |
| Range pagination       | Ordered scans with skip/limit and seek                                              | Large skips remain potentially expensive                                      |
| Expiration             | No native TTL/expiry API identified in v1.2 exports                                 | Expiration requires another layer or custom coordination                      |
| Change streams         | No general public subscription/changefeed API identified                            | External consumers require another delivery mechanism                         |

The absence statements here describe the published v1.2 public API inspected, not a claim that an application could never implement the capability above SurrealMX.

### Composing native capabilities

The most architecturally interesting opportunity is often not one individual API method, but combining several mechanisms.

For instance, a system with independently synchronized maps for entities, indexes, status flags, and work queues might instead encode those structures into distinct key families, update them through one transaction, and use the ordered scanning APIs for retrieval.

That replaces some bespoke consistency machinery with native optimistic transactions and MVCC snapshots.

Likewise, the combination of `get_for_update`, range-conflict detection, and conditional writes allows applications to express more sophisticated coordination rules than a conventional compare-and-swap map.

The tradeoff is that coordination is not costless. Encoding, transaction retries, conflict history, version retention, and cross-structure range design remain real engineering considerations.

SurrealMX is most distinctive where its transactional substrate can remove enough custom synchronization to justify its additional machinery.

## 13. Source map and version-sensitive reading order

For subsequent technical investigation, the following sources provide the most direct route into the implementation.

| Topic                                    | Primary source                                                                                                                                                        |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Release and identity                     | [https://github.com/surrealdb/surrealmx/releases/tag/v1.2.0](https://github.com/surrealdb/surrealmx/releases/tag/v1.2.0?utm_source=chatgpt.com)                       |
| v1.2 changes                             | [https://github.com/surrealdb/surrealmx/compare/v1.1.0...v1.2.0](https://github.com/surrealdb/surrealmx/compare/v1.1.0...v1.2.0?utm_source=chatgpt.com)               |
| Public crate exports                     | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/lib.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/lib.rs?utm_source=chatgpt.com)                 |
| Database API                             | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/db.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/db.rs?utm_source=chatgpt.com)                   |
| Transaction semantics                    | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)                   |
| Commit conflict filtering                | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/queue.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/queue.rs?utm_source=chatgpt.com)             |
| Commit and merge rings                   | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/ring.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/ring.rs?utm_source=chatgpt.com)               |
| MVCC version representation              | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/versions.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/versions.rs?utm_source=chatgpt.com)       |
| Readers and reclamation                  | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/readers.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/readers.rs?utm_source=chatgpt.com)         |
| Persistence                              | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs?utm_source=chatgpt.com) |
| Runtime options                          | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/options.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/options.rs?utm_source=chatgpt.com)         |
| Statistics                               | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/stats.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/stats.rs?utm_source=chatgpt.com)             |
| Architecture and performance paper, v1.0 | [https://surrealdb.com/static/whitepapers/surrealmx.pdf](https://surrealdb.com/static/whitepapers/surrealmx.pdf?utm_source=chatgpt.com)                               |

## Overall technical assessment

SurrealMX 1.2 represents a fairly sophisticated transactional memory engine with capabilities that go well beyond ordinary key-value caching. Its strongest differentiators are the interaction of MVCC, ordered ART indexing, optimistic serializable transactions, low-overhead point access, and a commit pipeline designed to keep readers and writers highly concurrent.

The v1.2 release is particularly interesting because it simplifies an already performance-oriented implementation: commit-ring slot numbers now serve directly as logical merge versions, eliminating a separate allocation and waiting protocol.

Three distinctions are especially consequential:

First, SurrealMX is a genuine transaction engine. Applications can rely on atomic multi-key updates, consistent snapshots, conflict detection, savepoints, and ordered scans rather than implementing these independently above a concurrent map.

Second, it is explicitly designed around in-memory execution. The optional persistence subsystem is substantial, but it does not turn SurrealMX into an LSM-based disk database or eliminate its requirement for resident working data.

Third, its strongest performance opportunities come from using its native access paths appropriately. Direct point reads, borrowed-value scans, adaptive conflict filtering, inline MVCC reclamation, and lightweight transaction pooling can be valuable individually, but their combined performance depends on workload shape and contention.

Version-status conclusion: SurrealMX 1.2.0 is a released stable version as of November 10, 2026. Its commit-sequencing optimization is verified in the tagged source. The broader functionality documented here is present in that tag; historical benchmark numbers have been deliberately distinguished from unmeasured v1.2 performance claims.

# SurrealMX v1.2.0 — Technical Addendum

Research date: November 10, 2026 · Target: SurrealMX 1.2.0 · Scope: Additional capabilities, implementation details, semantic qualifications, and corrections to the preceding reference

This addendum supplements the original SurrealMX overview without repeating its general architecture, transaction API, MVCC model, or v1.2 release summary.

It concentrates on capabilities and consequential distinctions identified through additional inspection of the v1.2.0 tagged source, integration tests, the `byteslice` dependency, and SurrealDB 4 documentation.

The v1.2.0 release was published on November 10, 2026. The source-level findings below apply specifically to that release, not necessarily to subsequent development commits.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 1. ByteSlice: compact storage, borrowed views, and efficient key representation

SurrealMX's `ByteSlice` representation deserves more attention than the original overview gave it. It is a dedicated storage abstraction that influences the engine's lookup costs, memory allocation patterns, comparison performance, and handling of immutable byte values.

SurrealMX 1.2 depends on `byteslice` 0.2.0, which implements a compact, immutable byte slice with small-string optimization, cached-prefix comparisons, and reference-counted sharing.

### 1.1 Dual representation

On 64-bit platforms, `ByteSlice` occupies 24 bytes and uses different representations according to value length.

| Property        | Short representation             | Long representation                              |
| --------------- | -------------------------------- | ------------------------------------------------ |
| Capacity        | Up to 20 bytes                   | Greater than 20 bytes                            |
| Data location   | Inline in the `ByteSlice` object | Separate allocation or borrowed buffer           |
| Heap allocation | None for inline construction     | Normally required for owned data                 |
| Clone           | Copies the inline representation | Shares the allocation through reference counting |
| Comparison      | Reads inline bytes               | Can compare cached prefix before full buffer     |

On 32-bit systems, the inline capacity is 16 bytes.

The long representation stores a four-byte prefix alongside its pointer and metadata. Comparisons can reject many unequal keys using that prefix without dereferencing the entire underlying byte buffer.

This is particularly relevant for ordered keys with distinguishing bytes near the beginning, because SurrealMX repeatedly performs key comparisons in ART traversal, write-set validation, and merge operations.

The optimization is data-dependent: keys sharing identical leading bytes still require comparisons beyond the cached prefix.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 1.2 Borrowed views versus owned storage

`ByteSlice` provides several construction mechanisms with materially different ownership behavior.

| Mechanism                                  | Semantics                                                           |
| ------------------------------------------ | ------------------------------------------------------------------- |
| `ByteSlice::from_slice(&[u8])`             | Constructs an owned representation; copies source bytes             |
| `ByteSlice::from_static(&'static [u8])`    | Constructs an owned representation from static data                 |
| `ByteSlice::from_bytes(&Bytes)`            | Copies the supplied `bytes::Bytes` contents                         |
| `ByteSlice::with_borrowed(&[u8], closure)` | Provides a temporary borrowed `ByteSlice` view                      |
| `ByteSlice::slice(range)`                  | Creates a sub-slice, sharing an owned heap allocation when eligible |

One easily missed distinction is that `from_bytes` does not automatically share the original `bytes::Bytes` allocation. Its implementation delegates to the copying constructor.

For temporary lookups, `with_borrowed` is often more interesting.

```
use surrealmx::ByteSlice;

let key = b"customer:001234567890";

ByteSlice::with_borrowed(key, |view| {
    // Inspect or compare view without allocating
    // an owned ByteSlice for the lookup.
    assert_eq!(view.as_slice(), key);
});
```

For sufficiently long borrowed values, the temporary view points directly into the original buffer. The closure prevents the borrowed view from escaping its valid lifetime.

The underlying `byteslice` crate also exposes an unsafe `from_borrowed_unchecked` constructor, but it requires the caller to guarantee that the view does not outlive its input. SurrealMX's own implementation uses the safe closure-based mechanism for certain lookup paths.

### 1.3 Efficient sub-slicing

`ByteSlice::slice` can create a new immutable view over a portion of an existing value.

For heap-backed owned data, a sufficiently long sub-slice shares the same allocation rather than copying its bytes. The new view records its adjusted offset and length.

Short sub-slices are copied inline, avoiding heap allocation altogether.

A sub-slice created from a temporary borrowed view must instead produce owned bytes, because its result might outlive the original borrow.

This distinction enables efficient handling of structured binary values, where different components may be accessed independently without repeatedly copying the underlying buffers.

It also introduces a memory-retention tradeoff: keeping a small shared sub-slice can retain the larger allocation from which it originated.

### 1.4 Size limits and serialization boundaries

Lengths and offsets use 32-bit metadata. The constructor consequently rejects byte sequences approaching or exceeding 4 GiB.

SurrealMX's `IntoBytes` trait does not provide arbitrary serialization of Rust domain types. Applications still need an appropriate encoding for structured records.

Important distinctions include:

- UTF-8 is not required for keys or values.
- NUL bytes are valid data, not terminators.
- Empty values are distinct from missing values.
- Immutable byte ownership does not imply automatic schema validation.
- Serialized record order is not necessarily the same as bytewise key order.

The key-comparison and allocation benefits are native, but a separate serialization format may still dominate the cost of complex record retrieval.

Primary source:\
[https://github.com/surrealdb/byteslice/blob/v0.2.0/src/byteslice.rs](https://github.com/surrealdb/byteslice/blob/v0.2.0/src/byteslice.rs?utm_source=chatgpt.com)

## 2. Transaction semantics requiring more precise treatment

### 2.1 Transactions support `Send + Sync`

The v1.2 source contains a compile-time assertion establishing that `Transaction` implements both `Send` and `Sync`.

This enables a transaction to be moved between threads or shared through appropriate synchronization primitives.

However, the distinction between thread safety and concurrent mutation remains important.

Multiple threads can hold shared access to a transaction, but methods requiring `&mut self`, including `set`, `del`, `commit`, and `cancel`, still require exclusive mutable access.

For asynchronous applications, `Send` permits a transaction to cross suitable task boundaries, but SurrealMX operations remain synchronous. The trait guarantees do not convert disk-backed commits or large scans into nonblocking asynchronous operations.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)

### 2.2 Negative reads are tracked under serializable isolation

A serializable write transaction does not need to read an existing value to establish a read dependency.

Reading the absence of a key also participates in conflict detection.

Consider:

```
let mut tx = db.transaction(true);

assert!(tx.get("reservation:42")?.is_none());

// Another transaction inserts reservation:42 here.

tx.set("reservation:index", "modified")?;

// Can return KeyReadConflict.
tx.commit()?;
```

Under `SerializableSnapshotIsolation`, the initial missing-key lookup is recorded.

If another transaction commits an insertion of that key after the first transaction's snapshot, the first transaction's eventual commit can fail.

This also applies to `exists` checks performed against the underlying snapshot.

The mechanism provides a native basis for optimistic validation of decisions that depend on absence, including uniqueness checks, reservations, and conditional state creation.

This is different from automatically enforcing application-level uniqueness constraints: the application must still express the relevant lookup and write within an appropriately isolated transaction.

The tagged isolation tests explicitly cover missing-key reads and existence checks followed by concurrent insertion.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/isolation.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/isolation.rs?utm_source=chatgpt.com)

### 2.3 Batched reads through `getm`

`Transaction::getm` performs multiple point lookups under one transaction snapshot.

Its signature is:

```
pub fn getm<K>(
    &self,
    keys: Vec<K>,
) -> Result<Vec<Option<ByteSlice>>, Error>
where
    K: IntoBytes;
```

Results correspond positionally to the supplied keys, including `None` for absent values.

For writable transactions, each lookup first checks the local write set. Serializable reads of committed data also contribute to conflict tracking.

The implementation performs a sequential loop over the input keys. It does not introduce an independently parallelized multi-get execution engine.

The primary benefit is therefore coordinated snapshot semantics and a convenient bulk interface, not an established SIMD or parallel lookup advantage.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)

### 2.4 Conditional mutation and local write-set states

`put`, `putc`, and `delc` distinguish between the committed snapshot and operations already staged within the transaction.

The distinction becomes important when a transaction modifies the same key repeatedly.

For example:

```
let mut tx = db.transaction(true);

tx.set("item", "original")?;
tx.del("item")?;

// A subsequent put("item", ...) is not treated
// identically to inserting into a fresh absent key.
```

In v1.2, `Transaction::put` rejects a key whenever it is already present in the transaction's write-set map, even when that map entry represents a pending deletion.

Thus, `del` followed by `put` is not guaranteed to behave as a deletion followed by a fresh insertion.

The existing integration test permits either outcome rather than asserting a single semantic contract.

For unconditional replacement, `set` provides a different operation from `put`. Conditional operations are more explicit about the expected state, with `None` representing expected absence and `Some(bytes)` representing an expected byte value.

An empty byte value is not the same as `None`.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/conditional.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/conditional.rs?utm_source=chatgpt.com)

## 3. Source-level API discrepancies in v1.2.0

Three behaviors in the tagged implementation deserve explicit qualification because they affect the correctness of otherwise reasonable application code.

These are source-derived findings, not independently executed defect reproductions. The relevant control flow is sufficiently specific to identify likely outcomes, but a complete verification would require building and exercising the tagged crate.

### 3.1 `Transaction::exists` does not consistently account for local deletion tombstones

`Transaction::del` records a pending deletion as `None` in the transaction's write-set map.

`Transaction::get` interprets this correctly: a key with a local tombstone returns `None`.

However, `Transaction::exists` checks:

```
if self.writeset.contains_key(lookup) {
    true
}
```

It does not inspect whether the associated write-set value represents an insertion/update or a deletion.

Accordingly, the following sequence appears to produce inconsistent answers:

```
let db = Database::new();

let mut tx = db.transaction(true);
tx.set("key", "value")?;
tx.del("key")?;

assert!(tx.get("key")?.is_none());

// Apparently true in v1.2.0 despite the local deletion.
assert!(tx.exists("key")?);
```

This is a meaningful exception to the usual read-your-writes expectation.

The related `getm` implementation does inspect the optional write-set value, so batched retrieval should agree with `get`, rather than `exists`, in this case.

The inconsistency is limited to the observed transaction-local implementation path. It should not be generalized into a claim that direct `Database::exists` is incorrect.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)

### 3.2 `Cursor::seek_for_prev` appears to exclude exact matches

The documented behavior of `seek_for_prev(key)` is to position on the greatest key less than or equal to the requested key.

The v1.2 implementation creates a reverse iterator using the requested key as the exclusive upper bound:

```
self.create_iterator(&beg, &seek_key, Direction::Reverse);
```

That appears to implement a strict predecessor lookup for keys inside the cursor's range.

For example, given:

```
Keys: a, c, e

seek_for_prev("d") → c
seek_for_prev("c") → apparently a
```

The first result is consistent with the method's documentation. The second is not.

The same discrepancy appears relevant when seeking to the beginning of the cursor's range: an inclusive predecessor seek should be capable of returning the starting key, while an exclusive upper bound at that key produces an empty range.

Until verified or corrected, the method's exact-match semantics should not be assumed from its documentation alone.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs?utm_source=chatgpt.com)

### 3.3 `DoubleEndedIterator` does not appear to maintain independent traversal boundaries

`KeyIterator` and `ScanIterator` both implement `DoubleEndedIterator`, enabling Rust operations such as `next_back()` and `.rev()`.

The implementation uses one underlying cursor and changes its direction when `next_back()` is called.

Importantly, the iterator constructors immediately position that cursor at the first or last entry.

In `next_back`, the repositioning logic is conditional on the cursor being unpositioned. For a newly constructed forward iterator, that condition is already false.

Consequently, `next_back()` on a forward iterator appears to consume its current front entry rather than the back entry required by the `DoubleEndedIterator` contract.

The implementation also does not maintain independent front and back boundaries to prevent overlapping consumption when calls are interleaved.

This matters because the Rust trait's semantics are stronger than simply supporting a cursor capable of moving in both directions.

The dedicated `keys_iter_reverse` and `scan_iter_reverse` constructors are separate APIs and do not rely on obtaining reverse traversal by calling `next_back()` on a forward iterator.

The existing iterator integration tests demonstrate normal forward and dedicated reverse traversal, but the examined tests do not establish correct interleaved front/back consumption.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs?utm_source=chatgpt.com)

Tests: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/iterators.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/iterators.rs?utm_source=chatgpt.com)

## 4. Binary-key ordering and prefix-range correctness

SurrealMX orders keys lexicographically by bytes.

This makes compound-key design important not only for lookup performance, but also for defining which records belong to a range and which concurrent writes conflict with a serializable range read.

### 4.1 A prefix is not inherently a range boundary

A common pattern is to construct a range from:

```
start = prefix
end   = prefix + 0xFF
```

This covers many ordinary ASCII-key cases, but does not represent all possible keys beginning with an arbitrary binary prefix.

It excludes keys whose suffix begins with `0xFF`, because those keys compare greater than or equal to the exclusive upper bound.

For example, given the prefix `[0x61]`:

```
[0x61]                Included
[0x61, 0x00]          Included
[0x61, 0xFE]          Included
[0x61, 0xFF]          Excluded
[0x61, 0xFF, 0x01]    Excluded
```

A correct exclusive upper bound is the smallest byte string greater than every key beginning with the prefix.

It can be constructed by incrementing the rightmost byte that is not `0xFF` and truncating the result immediately after that byte.

For example:

```
Prefix: [0x61, 0x62]
Upper:  [0x61, 0x63]

Prefix: [0x61, 0xFF]
Upper:  [0x62]
```

When a nonempty prefix consists entirely of `0xFF` bytes, no finite greater byte-string prefix successor exists.

The public SurrealMX range API uses bounded `std::ops::Range` values; it does not directly expose a general unbounded prefix iterator.

This is a key-encoding consideration rather than a defect in its ordered index.

### 4.2 Serialized numeric ordering

An ordinary native-endian numeric encoding does not necessarily preserve numerical ordering under bytewise comparison.

For example, little-endian integer encoding places the least-significant bytes first, which can produce an order different from the underlying integer sequence.

For ordered numeric fields in composite keys, fixed-width big-endian encodings, order-preserving signed integer transforms, or other explicitly sortable encodings are generally appropriate.

This is especially consequential when a single range represents a business predicate, because the same encoded boundaries can determine both retrieval and serializable conflict validation.

### 4.3 Prefix-sensitive test coverage

The tagged integration suite contains substantial tests for shared prefixes, deep key nesting, prefix boundaries, deletion followed by insertion, and concurrent updates to prefix-sharing keys.

This establishes that the engine's ordered storage behavior receives focused testing.

It does not eliminate the responsibility to construct application-level prefix bounds correctly.

Sources:

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/key_boundaries.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/key_boundaries.rs?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/prefix_keys.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/prefix_keys.rs?utm_source=chatgpt.com)

## 5. Thread-local scan registration and garbage-collection buffering

Two internal mechanisms further explain SurrealMX's low-contention design.

### 5.1 Direct scans reuse per-thread reader registrations

A direct database scan must pin a consistent snapshot while traversing the in-memory index. Otherwise, concurrent garbage collection could reclaim a version that the scan still needs.

SurrealMX avoids repeatedly registering a completely new reader slot for common direct scans.

Its `ThreadSnapshot` implementation maintains thread-local registrations associated with database instances.

When a direct scan begins, it can reuse the corresponding slot, pin a snapshot version, traverse the data, and unpin the slot when complete.

The same thread's subsequent direct scan can reuse that registration.

There are two important fallback cases.

Nested scans: A callback may invoke another direct scan on the same thread while its reader slot is already pinned. The nested scan cannot reuse the same slot concurrently and instead falls back to an ordinary read transaction.

Pending merge application: A direct scan's fast path requires a snapshot that can be served consistently from the applied datastore. If that condition is not met, it falls back to the transaction machinery capable of resolving merge-queue overlays.

This optimization is specific to direct range access. Direct point lookups use their own lightweight lookup paths.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/thread_slot.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/thread_slot.rs?utm_source=chatgpt.com)

### 5.2 Per-thread garbage-candidate buffers

The `GcCandidates` implementation also uses thread-local state.

A commit that cannot fully reclaim a version chain records the affected key in a buffer associated with its thread.

The garbage-collection sweep subsequently drains the registered buffers, sorts the accumulated keys, and deduplicates them before processing candidate chains.

This design reduces contention on a shared candidate queue during ordinary writes.

The distinction matters because a globally synchronized garbage-candidate map could reintroduce a coordination point into the commit hot path, even if actual reclamation was performed asynchronously.

The implementation deliberately separates the relatively frequent operation of reporting a candidate from the less frequent operation of processing the collected candidates.

The tradeoff is that candidate collection and deduplication consume resources at sweep time, and a long-lived reader can cause the same key to remain a candidate across multiple sweeps.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/gc_candidates.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/gc_candidates.rs?utm_source=chatgpt.com)

## 6. Append-only-log format evolution and recovery semantics

Several important persistence behaviors were underexplained in the initial reference.

### 6.1 AOL format version 2

SurrealMX 1.2 writes version-2 framed append-only logs.

A version-2 frame contains a 16-byte header followed by its transaction payload.

| Header field                       | Size     | Purpose                                             |
| ---------------------------------- | -------- | --------------------------------------------------- |
| Payload length                     | 4 bytes  | Number of payload bytes                             |
| Checksum                           | 4 bytes  | Detects damaged transaction data or header metadata |
| Previously synchronized log length | 8 bytes  | Records a known durability boundary                 |
| Transaction payload                | Variable | Encoded key/version/value records                   |

Each frame represents one transaction, allowing recovery to apply the transaction's records as a unit.

The frame format creates an additional size constraint: its encoded payload length must fit in a `u32`.

A sufficiently large transaction therefore can fail with a persistence append error even if the in-memory engine could otherwise accommodate its write set.

This is distinct from the maximum size of an individual `ByteSlice`.

The current format uses a checksum derived from xxHash3 over the payload and relevant synchronization metadata.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs?utm_source=chatgpt.com)

### 6.2 Legacy format compatibility

The persistence reader supports several historical representations.

| Format                                             | Recovery support in v1.2              |
| -------------------------------------------------- | ------------------------------------- |
| Legacy unframed key/version/value records          | Supported                             |
| Version-1 transaction frames                       | Supported                             |
| Version-2 transaction frames                       | Native current format                 |
| Older snapshots without the current version header | Supported by the compatibility reader |

Version-1 frames have an eight-byte header without the synchronized-length field.

When an older framed log is opened, the implementation can rewrite it into the current format.

Legacy unframed records may precede a newer framed section. This allows older persisted data to remain readable while new transactions use the transactional framing mechanism.

However, compatibility is directional. An older executable may not understand logs containing newer framed transactions.

This matters for upgrades and downgrades because persistence-format compatibility is not equivalent to Rust crate API compatibility.

### 6.3 Recovery distinguishes damaged synchronized data from an uncertain tail

The previous overview described corruption handling too simply.

Version-2 recovery uses synchronized-length metadata to help distinguish data that should have been durable from writes whose survival was not guaranteed.

If recovery encounters a damaged frame, it examines subsequent valid metadata to determine whether the damaged region was known to have been synchronized.

The distinction is:

- Damage in an unsynchronized region may be treated as a torn tail, with that region and subsequent data discarded.
- Damage affecting a known synchronized region is treated as corruption rather than silently discarded.

The implementation does not simply classify every damaged frame followed by additional bytes as fatal corruption.

This distinction acknowledges that a power failure can leave storage pages persisted out of order. The surviving physical bytes are not necessarily a clean prefix of attempted writes.

### 6.4 Recovery can rewrite the log atomically

After replay, SurrealMX may replace the log with a repaired copy.

When synchronization is enabled, the implementation can construct a replacement containing the recovered valid prefix, synchronize it, and atomically rename it over the previous file.

This is stronger than simply truncating whatever bytes were found after the last successfully decoded frame.

It is intended to protect against cases where a prior failed synchronization left the filesystem in a state that remains readable but is not guaranteed durable after another crash.

### 6.5 Asynchronous append order and version recovery

A further subtlety is that asynchronous persistence records are not assumed to arrive in increasing logical version order.

The replay implementation therefore tracks the maximum version found across all records, rather than treating the last log entry as the latest logical commit.

Version chains can also accept out-of-order version insertions.

This allows the recovered logical clock to continue above every persisted transaction version, including versions belonging to deletions.

The distinction matters because reusing an earlier version number after recovery could cause future writes to become incorrectly ordered relative to persisted tombstones.

### 6.6 Snapshot compression is detected automatically

SurrealMX's snapshot reader checks the stored data to identify LZ4 compression.

The implementation recognizes the LZ4 frame signature and selects the appropriate decompressor. Otherwise, it reads the snapshot as uncompressed data.

As a result, changing the configured compression mode for future snapshots does not inherently prevent recovery of an older snapshot written using the alternative supported mode.

Compression is therefore a property of the persisted snapshot representation, not merely an assumption applied from the current configuration.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/compression.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/compression.rs?utm_source=chatgpt.com)

## 7. Asynchronous persistence, admission control, and failure lifecycles

### 7.1 Asynchronous append batching

The v1.2 asynchronous persistence worker collects queued operations into batches.

The tagged implementation defines an internal batch-size threshold of 100 operations.

A batch is encoded into a reusable scratch buffer and written through the AOL file handle.

This can amortize system-call and synchronization costs across multiple transactions.

It is important to distinguish the three stages:

| Stage                           | Meaning                                                                 |
| ------------------------------- | ----------------------------------------------------------------------- |
| Transaction commit acknowledged | The transaction has completed the configured in-memory commit path      |
| Async AOL write completed       | The background worker has written the corresponding log data            |
| AOL synchronization completed   | The log data has been synchronized according to the fsync configuration |

These stages are not equivalent in asynchronous mode.

Even `FsyncMode::EveryAppend` does not make a previously acknowledged asynchronous transaction durable at its acknowledgement instant.

The synchronization occurs when the background worker processes its batch.

### 7.2 Graceful shutdown versus process failure

The asynchronous append worker is designed to drain outstanding queued operations during normal database shutdown.

Its shutdown loop checks the queue after receiving the shutdown signal and exits only once it has drained the work available under that protocol.

This reduces the risk of losing acknowledged asynchronous operations during an orderly shutdown.

It does not provide the same guarantee for abrupt process termination or power failure, because the worker may never execute its remaining operations.

The distinction belongs in the persistence contract: graceful lifecycle behavior is a useful capability, but it should not be confused with synchronous crash durability.

### 7.3 Queue statistics include rejected work

A subtle interpretation issue exists with `async_dropped_batches`.

The persistence implementation increments its dropped-batch counter not only after certain worker failures, but also when a capacity precheck rejects a new asynchronous append because queue limits would be exceeded.

Therefore, a nonzero dropped-batch statistic does not necessarily prove that an already acknowledged transaction was silently lost.

It may instead represent a rejected transaction that returned `AsyncQueueFull` before admission.

This is important for observability: the counter combines operationally different failure conditions.

### 7.4 Transaction state after errors

Not all commit errors leave a transaction in the same condition.

| Error or outcome       | Relevant behavior                                                                       |
| ---------------------- | --------------------------------------------------------------------------------------- |
| `KeyReadConflict`      | Conflict validation rejects the transaction; the logical operation needs a new snapshot |
| `KeyWriteConflict`     | Conflicting transaction is aborted; retry the logical operation in a new transaction    |
| `MemoryLimit`          | Pre-commit resource rejection can leave the transaction open                            |
| `AsyncQueueFull`       | Capacity rejection can leave the transaction open                                       |
| `TxTooOld`             | Transaction's snapshot registration is released after age-limit detection               |
| `TxCommitNotPersisted` | Persistence failed; the durable outcome may be ambiguous                                |
| `DatabasePoisoned`     | Database has entered a stopped/error state                                              |
| `TxClosed`             | Operation was attempted after transaction completion or cancellation                    |

The particularly important distinction is between a retryable transaction conflict and a retryable commit-admission failure.

For `MemoryLimit` and `AsyncQueueFull`, the source preserves the transaction as open on the relevant pre-commit error paths, permitting the caller to retry the commit or cancel it.

By contrast, a conflict requires reevaluating the operation against a new snapshot.

A failed persistence append introduces another dimension: the engine may have attempted a durable write whose precise recovery outcome is uncertain. Reexecuting external side effects without reconciliation can therefore create problems even if the original in-memory write was not published.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)

### 7.5 Persistence files are not a shared multi-process database service

The v1.2 persistence constructor opens log and snapshot paths using ordinary filesystem operations and coordinates writes through locks associated with its own persistence instance.

The inspected initialization path does not establish an operating-system-level exclusive-owner lock covering independent database instances.

The concurrent-recovery integration test does not assert that multiple instances can safely write to the same files. It allows different open outcomes and checks only limited recovery behavior.

Accordingly, SurrealMX's persistence support should not be interpreted as multi-process concurrent access to a shared database directory.

Its transaction concurrency guarantees apply to operations coordinated through the same engine instance, not automatically to independent engines accessing common persistence files.

This is a boundary of the verified implementation rather than a reproduced corruption test.

Source: [https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/recovery_edge.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/recovery_edge.rs?utm_source=chatgpt.com)

## 8. Distinguishing SurrealMX's native capabilities from SurrealDB 4

Several features exposed by SurrealDB 4 use SurrealMX capabilities underneath but do not share identical API semantics.

### 8.1 Different default isolation levels

The most significant distinction is the default for writable transactions.

| Context                                                                | Default isolation               |
| ---------------------------------------------------------------------- | ------------------------------- |
| Direct SurrealMX `Database::transaction(true)`                         | `SerializableSnapshotIsolation` |
| SurrealDB 4 query execution using SurrealMX                            | Snapshot isolation              |
| SurrealDB 4 explicit `BEGIN ISOLATION LEVEL SERIALIZABLE` on SurrealMX | Serializable isolation          |

The Rust crate's constructor initializes writable transactions using `SerializableSnapshotIsolation`.

SurrealDB 4, however, documents snapshot isolation as the default across storage engines, including its in-memory SurrealMX backend. Serializable isolation is explicitly available where the backend supports it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



This illustrates an important general boundary: an engine-level capability can exist without being the default behavior of a higher-level integration.

### 8.2 `FOR UPDATE` and serializable isolation are distinct

SurrealDB 4 exposes `SELECT ... FOR UPDATE`, which extends conflict validation to specifically read records.

Serializable isolation instead adds transaction read and range dependencies more broadly.

The conceptual distinction matches SurrealMX's own separation between individually locked reads and serializable read-set validation.

However, these should not be treated as interchangeable interfaces.

In SurrealDB, `FOR UPDATE` is available across its documented engines, whereas full serializable isolation is currently offered by SurrealMX and SurrealDS.

The SQL/query layer determines how those semantics are translated into backend operations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 8.3 Published snapshots and `MAX STALENESS`

SurrealDB 4 supports read-only transactions with a maximum-staleness requirement.

This does not mean SurrealMX publishes historical read snapshots suitable for stale-read optimization.

SurrealDB's storage-engine documentation distinguishes:

| Backend   | Supports serializable isolation | Publishes snapshots for `MAX STALENESS` |
| --------- | ------------------------------- | --------------------------------------- |
| SurrealMX | Yes                             | No                                      |
| SurrealKV | No                              | No                                      |
| TiKV      | No                              | No                                      |
| SurrealDS | Yes                             | Yes                                     |

On an engine without published snapshots, a maximum-staleness read uses the current state rather than selecting an older published snapshot.

This is different from SurrealMX's internal MVCC history. Retaining versions needed by active transactions does not automatically provide a public historical-snapshot selection service.

Source: [https://surrealdb.com/docs/reference/query-language/statements/begin](https://surrealdb.com/docs/reference/query-language/statements/begin?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



## 9. Version-pinned documentation discrepancies

The v1.2 source contains some inconsistencies between its packaged documentation and actual code.

`Cargo.toml` specifies `CARGO.md` as the crate's readme, while the repository also contains a separate `README.md`.

The packaged readme labels full persistence as the default and describes snapshot isolation as the default transaction mode.

Both statements require qualification.

The actual `PersistenceOptions` defaults disable AOL and snapshot persistence, and the direct Rust writable transaction constructor selects serializable snapshot isolation.

The repository's main README describes these defaults more accurately.

This distinction is significant for programmatic documentation consumers because crate metadata and hosted documentation can surface the packaged readme rather than the repository's more accurate introductory text.

For v1.2, the tagged implementation should remain authoritative when documentation statements conflict.

Sources:

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/CARGO.md](https://github.com/surrealdb/surrealmx/blob/v1.2.0/CARGO.md?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/README.md](https://github.com/surrealdb/surrealmx/blob/v1.2.0/README.md?utm_source=chatgpt.com)

[https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/options.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/options.rs?utm_source=chatgpt.com)

## 10. Consolidated technical implications

The additional findings clarify several broader architectural properties of SurrealMX that are easy to miss when evaluating only its documented API names.

The engine optimizes both value access and coordination overhead. `ByteSlice` reduces allocations and comparison costs; thread-local snapshot registrations reduce reader-management work; adaptive conflict filters avoid unnecessary exact comparisons; and per-thread GC buffers reduce shared synchronization. These mechanisms target different costs and can compound under high-frequency operations.

Transactional behavior depends on more than isolation-level selection. Negative reads, tombstones, local write-set inspection, savepoint history, and the precise semantics of conditional operations determine whether a multi-step operation behaves as intended. The identified `exists` discrepancy illustrates why these details deserve explicit verification.

Rust trait availability is not sufficient evidence of behavioral guarantees. The iterator implementations demonstrate that source inspection should examine how traits are implemented, especially when iterator ordering, ownership, or correctness depends on their contracts.

Persistence is more sophisticated than periodic serialization. Versioned framed logs, synchronization-watermark metadata, legacy format migration, recovery-time rewriting, compression detection, and asynchronous batching form a substantial durability subsystem. Nevertheless, asynchronous acknowledgement and cross-process file sharing remain distinct from synchronous, coordinated durable transactions.

SurrealMX and SurrealDB must be documented at their actual abstraction boundaries. SurrealMX implements low-level transactional mechanics; SurrealDB determines the higher-level query semantics, defaults, and supported interfaces.

### Final source references

The following version-pinned files are the principal references for this addendum:

| Subject                               | Direct source                                                                                                                                                                   |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ByteSlice` internals                 | [https://github.com/surrealdb/byteslice/blob/v0.2.0/src/byteslice.rs](https://github.com/surrealdb/byteslice/blob/v0.2.0/src/byteslice.rs?utm_source=chatgpt.com)               |
| Transaction implementation and errors | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/tx.rs?utm_source=chatgpt.com)                             |
| Cursor and iterator implementation    | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/cursor.rs?utm_source=chatgpt.com)                     |
| Direct scan registration              | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/thread_slot.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/thread_slot.rs?utm_source=chatgpt.com)           |
| GC candidate collection               | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/gc_candidates.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/gc_candidates.rs?utm_source=chatgpt.com)       |
| Persistence and recovery              | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/persistence.rs?utm_source=chatgpt.com)           |
| Snapshot compression                  | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/compression.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/src/compression.rs?utm_source=chatgpt.com)           |
| Isolation tests                       | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/isolation.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/isolation.rs?utm_source=chatgpt.com)           |
| Iterator tests                        | [https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/iterators.rs](https://github.com/surrealdb/surrealmx/blob/v1.2.0/tests/iterators.rs?utm_source=chatgpt.com)           |
| SurrealDB transaction semantics       | [https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions?utm_source=chatgpt.com) |

This addendum completes the previously identified capability gaps without expanding the original overview into unrelated application design. The three source-level discrepancies concerning `exists`, `seek_for_prev`, and `DoubleEndedIterator` remain qualifications requiring executable verification, rather than claims of tested defects.