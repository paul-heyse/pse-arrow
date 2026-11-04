# SurrealDB 4.0 — Comprehensive Technical Capability Reference

Target: SurrealDB 4.0.0 (stable) · Released November 3, 2026 · Research date: November 3, 2026

SurrealDB 4.0 is a major architectural release of the Rust-based, multi-model database. It combines document, relational, graph, vector, full-text, geospatial, and temporal data capabilities with a redesigned query executor, broader SQL interoperability, improved storage engines, and a more capable extension system.

Version 4.0.0 became generally available on November 3, 2026. It introduces a new execution architecture and extends capabilities across the database engine, query language, indexing, storage, and integrations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



The most important conceptual distinction is that SurrealDB is not a collection of independent databases exposed behind a common interface. Its data models operate within one database engine, where records, graph relationships, indexes, and queries can participate in the same transactional operations.

Important release qualification: SurrealDB 4.0 is generally available for new deployments, but upgrading existing 3.x datastores in place remains experimental and irreversible. SurrealDB recommends waiting for 4.1 before migrating existing production deployments. This distinction matters when evaluating 4.0's deployment readiness.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



## 1. Capability map

| Capability family                  | Native mechanisms                                                                   | Architectural significance                                                                               |
| ---------------------------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Multi-model persistence            | Documents, typed records, relational links, graph edges, nested arrays and objects  | Models several related forms of state without requiring separate persistence engines                     |
| Query processing                   | SurrealQL, PostgreSQL/MySQL SQL dialects, ISO GQL, indexes, bytecode execution      | Multiple query interfaces use the same underlying data and transaction machinery                         |
| Schema and semantics               | Schemafull/schemaless tables, named types, constraints, computed fields, migrations | Supports both flexible ingestion and increasingly explicit domain contracts                              |
| Search and retrieval               | B-tree, full-text/BM25, HNSW, DiskANN, spatial indexing                             | Combines structured filtering, semantic retrieval and relationship traversal                             |
| Transactional state                | Snapshot transactions, optional serializable isolation, atomic events, changefeeds  | Coordinates state changes and their derived effects within the database                                  |
| Embedded and distributed execution | Rust embedding, SurrealMX, SurrealKV, SurrealDS, browser persistence                | Similar data/query abstractions across different deployment models, with distinct operational guarantees |
| Extensibility and integration      | Surrealism extensions, custom functions and HTTP APIs, SDKs, wire protocols         | Moves selected logic into database-managed execution while preserving external integration options       |

The important distinction is between logical portability and operational equivalence. A SurrealQL query may be portable between storage backends, but serializable isolation, historical reads, failure recovery, durability and horizontal scaling do not necessarily behave identically.

## 2. Execution architecture: the central change in 4.0

### 2.1 Unified bytecode query executor

Before 4.0, SurrealDB had several planner strategies with different execution paths. Version 4.0 replaces these with a unified executor that compiles statements into cached bytecode.

The architectural implications are substantial:

- More consistent optimization: indexed scans, graph-oriented expressions, subqueries and conventional relational operators are planned through a common execution system.
- Integrated authorization: table-level `SELECT` permission predicates participate in optimization rather than being treated entirely as an additional filtering stage.
- Reduced repeated computation: certain uncorrelated subqueries can be computed once rather than for every candidate record.
- More efficient point access: predicates identifying particular record IDs can use direct key lookups; eligible updates and deletes can exploit indexes.
- Better large-query behavior: parallel selective scans and external-memory execution extend the practical size of sorts, grouping and joins.

This is a meaningful shift from a primarily expressive multi-model query interface toward a more conventional, optimizable database execution architecture.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Source: `https://surrealdb.com/releases/4.0`

### 2.2 Parallelism, memory management and plan inspection

Version 4.0 can partition eligible scans across execution threads, except in browser environments. Operations such as sorting, `GROUP BY`, `DISTINCT` and joins can spill working data to temporary storage when their memory allocation is exceeded.

`SURREAL_MEMORY_LIMIT` provides a node-level control over query memory consumption. This is not a promise that every allocation is spillable: bound variables, retained intermediate values and other operations may have separate limits.

The combination is useful for workloads that mix short transactional queries with less predictable analytical scans. It reduces the need to engineer separate execution paths merely because a result set or aggregation can exceed available memory. It does not turn SurrealDB into a distributed analytical execution engine.

`EXPLAIN FULL` remains the important diagnostic mechanism for determining which indexes and execution operations a query actually uses. Performance assumptions about predicate pushdown, ANN access, scan parallelism or index-assisted sorting should be checked against the resulting plan rather than inferred from query syntax.

A consequential semantic change is that result order is not guaranteed without `ORDER BY`. The new parallel executor makes any previous reliance on incidental scan order especially unsafe.

Source: `https://surrealdb.com/releases/4.0`

### 2.3 Read/write execution and retry semantics

SurrealDB 4.0 improves the handling of write conflicts for standalone statements, including automatic retries in qualifying cases. That does not eliminate transaction conflicts or remove the need for caller-side retry logic around explicit multi-statement transactions.

The distinction matters when database logic performs external side effects. Retrying a database operation is not equivalent to safely retrying an HTTP call, an email, or another irreversible action.

The general architectural advantage is that more application logic can operate against a coherent transactional record state, but external systems still require explicit coordination, idempotency or outbox-style boundaries.

## 3. Data model, identity and schema semantics

### 3.1 Record identity and physical representation

The primary unit of persistence is a record identified by a typed record ID such as `person:alice`. A table groups records, while an individual record can contain scalar values, objects, arrays, record IDs and nested structures.

Records and associated indexes are represented through transactional key-value storage. A graph edge is itself a record with endpoint fields, not an unrelated graph-store object.

Consequently, a record can simultaneously participate in ordinary field predicates, graph traversals, full-text retrieval and vector search without being copied into different native database engines.

SurrealDB retains the familiar hierarchy of root, namespace, database, table and record. Namespaces and databases are useful administrative and isolation boundaries, but they should not be confused with independent distributed transaction coordinators.

Source: `https://surrealdb.com/docs/learn/data-models/architecture`

### 3.2 Schemafull and schemaless modeling

SurrealDB offers two complementary schema modes.

Schemaless tables permit records with variable fields and heterogeneous nested structure, useful when source material or domain shape is genuinely variable.

Schemafull tables constrain fields through `DEFINE FIELD`, type declarations, defaults, assertions, computed expressions, field permissions and related schema definitions.

The distinction is not equivalent to relational versus document storage. A schemafull record can still contain nested objects, arrays, unions and record links. Likewise, a schemaless table can still participate in graph relationships and indexes.

A significant modeling advantage is the ability to constrain stable domain structure without flattening every variable part into separate tables. The corresponding tradeoff is that deeply nested values can be less convenient to index, update selectively or expose through SQL interoperability layers.

### 3.3 Named types, unions and numerical precision

SurrealDB 4.0 adds `DEFINE TYPE`, providing reusable named constraints for scalar types, object shapes and value unions.

For example, a named state type can represent a finite domain vocabulary rather than repeating a string validation expression on every field. Named unions retain declaration order, with supporting `enum::rank`, `enum::min`, `enum::max` and `enum::values` functions. That gives a type both validation semantics and an explicit ordering contract.

This is different from a collection of untyped string literals: the type becomes part of the database schema and can participate in casts and field definitions.

Version 4.0 also introduces narrower numeric representations, including `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `u64`, `f16` and `f32`; `i64` and `f64` correspond to the existing general integer and float types. Arithmetic and conversion rules account for width and overflow rather than silently assuming every value has identical representation.

Additional types include `date`, `time`, `timestamp`, `interval` and `bigdecimal`.

These capabilities matter for interoperability with typed systems, preserving external numeric contracts, representing exact decimal quantities and preventing accidental widening or silent overflow. They do not remove the need to distinguish floating-point approximation from exact decimal arithmetic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Sources:

`https://surrealdb.com/docs/reference/query-language/statements/define/type`

`https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/named-types`

### 3.4 Missing, null and optional values

`NONE` and `NULL` are distinct SurrealQL values.

`NONE` represents absence; `NULL` is an explicit null value. A field typed as `option<string>` may be absent, whereas accepting an explicit null requires the appropriate nullable union.

This distinction propagates into field validation, comparisons, JSON/SQL conversion, filtering and schema evolution. It matters particularly when preserving the difference between “not supplied,” “unknown,” and “explicitly cleared.”

When interacting through PostgreSQL or MySQL protocols, some of SurrealQL's more expressive absence semantics must be mapped onto SQL's column and null model.

### 3.5 Named record-ID components and key-locality design

Version 4.0 supports named components in array-based record IDs, for example a key logically composed of `city` and `time`.

This allows application-level identity to carry structured, typed information that remains addressable in queries. Because the underlying key-value engine maintains ordered keys, suitably arranged composite IDs can support efficient prefix and range access.

That creates a direct connection between domain identity and storage locality. Time-series readings or entity-specific event histories can sometimes use native record-key ordering instead of a separately maintained lookup structure.

The tradeoff is that key structure becomes part of the persistence contract. Changing identity composition can be more consequential than changing an ordinary field.

Source: `https://surrealdb.com/releases/4.0`

### 3.6 Schema evolution with stored-data migration

One of 4.0's most important additions is declarative migration of existing values when a field type changes.

An illustrative SurrealQL statement from the release notes is:

```
ALTER FIELD age ON person TYPE int
    MIGRATE EXISTING ROWS AS age_to_int
    USING <int>$value
    FALLBACK 0;
```

The migration has a stable name, supports a conversion expression and defines fallback behavior. Existing records are converted when read while a background operation rewrites the table.

This materially differs from changing only the type declaration and leaving existing records in their previous representation. It can reduce the period during which readers must tolerate several incompatible shapes.

For newly created 4.0 datastores, schemafull tables enforce the new migration requirements for changes that may invalidate stored records. Datastores upgraded from 3.x retain older compatibility behavior until their compatibility level is explicitly advanced.

Important boundaries remain: the conversion expression must use supported operations, a fallback is a semantic decision rather than automatic validation of business meaning, and background rewriting creates an operational lifecycle to monitor.

Source: `https://surrealdb.com/releases/4.0`

## 4. Graph, record references and relationship modeling

### 4.1 Edges are first-class records

SurrealDB's native graph model uses ordinary records as vertices and relation-table records as edges.

For example:

```
CREATE person:alice SET name = 'Alice';
CREATE company:acme SET name = 'Acme';

RELATE person:alice->works_at->company:acme
    SET since = d'2024-01-15';

SELECT
    name,
    ->works_at->company.name AS employers
FROM person;
```

This is a conceptual example, not an executed query.

The `works_at` edge has its own identity and can store attributes such as effective dates, roles, provenance, confidence, or relationship status.

Consequently, relationships can have an independent lifecycle and carry domain semantics beyond the existence of an association.

Graph traversal uses directional operators such as `->`, `<-` and bidirectional forms. Traversals can include predicates, projections, multiple hops and bounded recursive patterns.

The core distinction from a conventional foreign-key model is that a relationship can be queried and modified as a first-class database entity without first implementing a separate join-table abstraction in application code.

Source: `https://surrealdb.com/docs/learn/data-models/graph/overview`

### 4.2 Record links versus graph edges

SurrealDB also permits a record field to hold another record's ID directly.

These mechanisms are related but not interchangeable:

| Mechanism                      | Semantics                                                            | Typical advantage                                               |
| ------------------------------ | -------------------------------------------------------------------- | --------------------------------------------------------------- |
| Record link                    | A field contains another record ID                                   | Compact representation of a straightforward association         |
| Tracked `REFERENCE`            | A record link with database-maintained reverse-reference information | Efficient inverse navigation and referential behavior           |
| Relation edge                  | Independent record connecting `in` and `out`                         | Attributes, identity and lifecycle for the relationship itself  |
| Computed reference             | A field derived from related records or reference traversal          | Convenient query-time representation                            |
| Materialized reverse reference | Incoming references stored on the referenced record                  | Faster repeated inverse access at additional write/storage cost |

The `REFERENCE` clause adds inverse tracking, accessible with `<~`. It also supports explicit deletion policies including `IGNORE`, `UNSET`, `REJECT` and custom `THEN` behavior.

The default `ON DELETE IGNORE` can leave dangling references, which is important when distinguishing SurrealDB's reference mechanism from a conventional enforced SQL foreign key. Referential behavior must be declared according to the required semantics.

Source: `https://surrealdb.com/docs/reference/query-language/language-primitives/record-references`

### 4.3 Materialized back-references in 4.0

Version 4.0 extends reference tracking with:

`REFERENCE ... MATERIALIZE INTO <table>.<field> [ WHERE <condition> ]`

This maintains a sorted collection of referring record IDs on the referenced record itself, updated in the same transaction.

The value is more than syntactic convenience. Consider a domain where many records point to one parent and the parent is frequently read with all incoming references. A materialized inverse can turn repeated reference-index traversal into direct field access.

The costs are additional stored state, write amplification and potential contention on heavily referenced records. For highly connected vertices, a separate indexed relationship query may be preferable to maintaining an increasingly large inverse collection.

This is a useful example of SurrealDB offering several native physical representations of a logical relationship rather than forcing every association into graph edges.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



### 4.4 ISO GQL and graph-pattern execution

SurrealDB supports a subset of the ISO/IEC 39075 Graph Query Language. It uses constructs including `MATCH`, `OPTIONAL MATCH`, variable-length patterns, shortest-path selectors and graph mutations.

It operates on the same records and relation edges as SurrealQL but has a different language and internal graph-pattern planning surface.

The supported implementation is not the full ISO GQL standard and is not equivalent to accepting arbitrary Neo4j openCypher. Unsupported constructs include various pipeline clauses, recursive data-processing features and some set operations. The documented supported grammar should be consulted for specific patterns.

GQL is available through `/gql`, an RPC method, and selected SQL wire-protocol sessions. GraphQL is a different interface, exposed through `/graphql` with its own schema configuration and query semantics.

For graph-centric workloads, GQL can express path patterns more directly; for queries that combine graph navigation with SurrealQL-specific functions, schema operations or search syntax, SurrealQL often remains the more complete interface.

Sources:

`https://surrealdb.com/docs/learn/querying/gql/overview`

`https://surrealdb.com/docs/learn/querying/graphql/overview`

## 5. Indexing, search and multi-modal retrieval

### 5.1 Conventional indexes

SurrealDB supports standard and unique indexes, including compound fields, for exact lookup, range predicates, indexed filtering and other operations where the optimizer can exploit key ordering.

In 4.0, the planner improves use of these indexes for bound parameters, key lists, eligible indexed counts and ordered top-k operations.

Index lifecycle remains consequential. Definitions, concurrent builds, rebuilds and index readiness are distinct states. In particular, a uniqueness constraint has enforcement implications that a background index build must handle safely.

The new `UNIQUE NULLS NOT DISTINCT` option treats an explicit `NULL` as a value for uniqueness purposes. `NONE` remains separate and is not compared in the same way.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

### 5.2 Vector storage and exact search

Embeddings are stored as numerical arrays in record fields. SurrealDB provides vector distance and similarity functions covering multiple metrics, rather than requiring every similarity computation to pass through an ANN index.

An exact vector scan compares the query vector against the eligible candidate records. It can be preferable when the candidate set is small, when the workload requires exact nearest-neighbor results, or when the overhead of an ANN graph is unjustified.

Exact distance computation and indexed approximate nearest-neighbor search are distinct capabilities. A query using a distance function is not necessarily accelerated by an ANN index, and an ANN query does not automatically guarantee exact global top-k results.

### 5.3 HNSW versus DiskANN

SurrealDB supports two graph-based approximate nearest-neighbor index families.

| Characteristic   | HNSW                                                      | DiskANN                                                            |
| ---------------- | --------------------------------------------------------- | ------------------------------------------------------------------ |
| Primary design   | Low-latency graph search with substantial in-memory state | Larger vector corpora with key-value-backed graph storage          |
| Memory pattern   | Hot graph and vector/cache structures                     | Bounded memory cache over persisted graph/vector data              |
| Tuning           | Construction and search breadth, graph connectivity       | Graph degree, construction/search breadth, cache and beam behavior |
| 4.0 improvements | SIMD-based distance evaluation, improved filtered search  | Quantized traversal, exact final reranking, improved filtering     |
| Main tradeoff    | Memory footprint and index-maintenance cost               | Storage access, cache sensitivity and search-effort tuning         |

Version 4.0 introduces 8-bit quantized traversal for new or rebuilt DiskANN indexes, followed by full-precision final ranking. Quantization accelerates graph exploration but does not, by itself, make approximate retrieval exact.

HNSW and DiskANN also have different supported metrics and vector encodings. Index choice must account for the intended distance metric and numerical representation, not merely dataset size.

An important version-specific change is DiskANN support in WebAssembly, including browser builds. Some older general documentation still describes DiskANN as unavailable on WASM; the 4.0 release notes explicitly state the new support.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



Source: `https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes`

### 5.4 Query planning for vector search

SurrealDB's KNN operators, including `<|K, EF|>`, can route to compatible vector indexes. The second parameter controls search effort, with different practical interpretation for each ANN algorithm.

Version 4.0 also enables eligible ordered-distance queries to use indexes:

```
SELECT id
FROM document
ORDER BY vector::distance::cosine(embedding, $query)
LIMIT 10;
```

This can be easier to compose with ordinary ranking, filtering and result projection than expressing every vector query through a specialized predicate.

The optimizer can also construct candidate prefilters from other indexes. When only a small subset of records is eligible, comparing those candidates directly may be more efficient than traversing an ANN graph over the entire corpus.

The key distinction is between:

Filter first, then rank eligible records, versus search approximately, then test whether returned records satisfy the filter.

Those strategies can produce different performance and recall characteristics. SurrealDB 4.0 expands its ability to select efficient filtered-search strategies, but approximate recall remains workload dependent.

`EXPLAIN FULL` is necessary to establish which route was used.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



### 5.5 Full-text search and ranking

Full-text search consists of an analyzer pipeline, an inverted index and query-time matching/ranking.

`DEFINE ANALYZER` controls tokenization and transformations such as normalization and stemming. `DEFINE INDEX ... FULLTEXT ANALYZER ...` associates the analyzer with a field.

Queries can use `@@`/`MATCHES` and named match references, with functions such as `search::score()` and `search::highlight()`.

BM25 provides relevance ranking based on term and document statistics. Unlike vector similarity, this is principally lexical retrieval, sensitive to the analyzer configuration and document-frequency distribution.

Version 4.0 introduces block-oriented postings and pruning for top-ranked results, along with a per-node cache of frequently accessed term information. Existing indexes may require rebuilding to adopt the improved layout.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Sources:

`https://surrealdb.com/docs/learn/data-models/full-text-search/overview`

`https://surrealdb.com/docs/learn/data-models/full-text-search/scoring-and-ranking`

### 5.6 Hybrid retrieval: structured, lexical, semantic and graph

A distinctive advantage of the multi-model engine is that search results remain ordinary records with relationships and structured attributes.

A query can therefore combine a vector predicate with full-text matching, access permissions and graph projections.

The release announcement demonstrates the general shape:

```
SELECT
    text,
    ->mentions->person.name AS people,
    search::score(1) AS relevance
FROM note
WHERE embedding <|5,40|> $question
    AND text @1@ 'contract renewal'
ORDER BY relevance DESC;
```

This example combines several operations in one query execution path.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



SurrealDB also provides `search::rrf()` for reciprocal-rank fusion, which combines rankings from retrieval methods whose raw scores are not directly comparable.

There are two distinct architectural choices:

Conjunctive retrieval: lexical, vector and structural conditions must all match. This is useful when every condition is mandatory, but can exclude relevant items that match only one modality.

Rank fusion: separate retrieval methods contribute candidates and rankings that are combined. This improves flexibility, but requires decisions about candidate depth, fusion parameters and final filtering.

The benefit of a common engine is avoiding cross-database identity synchronization and separate persistence pipelines for each retrieval mode. It does not eliminate the need to generate embeddings, evaluate retrieval quality, maintain indexes, or reason about permissions.

Source: `https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search`

### 5.7 Spatial and geospatial indexing

SurrealDB supports geometry data and geospatial predicates; 4.0 adds the `SPATIAL` index as a significant new acceleration mechanism.

A spatial index organizes geometry references using cells over geographic space, enabling candidate pruning for predicates such as `INSIDE`, `CONTAINS`, `INTERSECTS`, and supported radius comparisons using `geo::distance`.

It can also combine with conventional filters and vector candidate selection.

This is especially useful for workloads where a semantic query is relevant only within a geographic region, or where graph-connected entities have spatial constraints.

As with vector retrieval, the distinction between a geometric predicate and its indexed execution is important. The query expresses the required spatial semantics; the index accelerates eligible candidate discovery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

## 6. Transactions, isolation and concurrent state

### 6.1 Snapshot isolation is the universal baseline

SurrealDB provides ACID transactions with snapshot isolation and commit-time conflict detection across its storage engines.

A transaction reads a consistent snapshot. Concurrent updates to records written by that transaction can cause its commit to fail. Changes to records, associated indexes and synchronous database-side events participate in the transaction's atomic boundary.

A single statement normally executes as one implicit transaction. Explicit transactions use `BEGIN`, `COMMIT` and `CANCEL`; query blocks and custom function bodies can also provide transaction boundaries.

Snapshot isolation is not full serializability. Two transactions can read a shared condition, modify different records and commit even though the combined result violates an invariant. This is the classic write-skew problem.

### 6.2 Serializable isolation is engine-dependent

Version 4.0 supports:

```
BEGIN ISOLATION LEVEL SERIALIZABLE;
-- Reads and writes whose combined invariant matters
COMMIT;
```

But it is available only on particular backends.

| Storage backend       | Snapshot isolation | Serializable isolation |
| --------------------- | ------------------ | ---------------------- |
| SurrealMX             | Yes                | Yes                    |
| SurrealKV             | Yes                | No                     |
| SurrealDS             | Yes                | Yes                    |
| Separate TiKV backend | Yes                | No                     |

On unsupported engines, requesting serializable isolation fails explicitly instead of silently downgrading the transaction.

`SELECT ... FOR UPDATE` offers another mechanism: it registers selected records for conflict checking even if they are not subsequently written. On SurrealDB this is not simply the same blocking row-lock behavior as in PostgreSQL; conflicting updates are generally detected at commit.

The distinction between them is essential. `FOR UPDATE` protects specific records that have been read; serializable isolation also addresses conflicts arising from predicate and range reads, including cases where the critical condition is the absence of a record.

Serializable transactions are more likely to abort under concurrency and are therefore most valuable where the stronger invariant justifies the additional conflict rate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions`

### 6.3 Read-only and bounded-staleness transactions

Version 4.0 adds explicit `BEGIN READ ONLY` transactions, which reject writes and retain a consistent snapshot.

SurrealDS additionally supports bounded-staleness reads through `MAX STALENESS`, with a strictness option when no sufficiently recent snapshot exists.

This is a useful separation between an operation requiring the newest committed state and one that can tolerate a documented amount of staleness.

Other engines do not acquire SurrealDS's historical published-snapshot behavior merely because they accept read-only transactions.

Serializable mode cannot be combined with a positive staleness allowance, reflecting the different guarantees of those modes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



### 6.4 Historical versioning versus transactional snapshots

SurrealDB also has versioned-read capabilities on applicable persistent engines, allowing `SELECT` queries against retained historical versions when the datastore was configured for versioning.

These must be distinguished from ordinary snapshot transactions:

Transaction snapshot: the consistent state visible during an active transaction.

Historical versioned read: access to a previously committed state preserved according to a storage engine's version-retention configuration.

Changefeed: a retained record of mutations suitable for incremental replay.

Those are three related but different mechanisms. In particular, SurrealMX does not support the historical `VERSION` query functionality that SurrealKV can provide when configured appropriately.

Source: `https://surrealdb.com/docs/running/in-memory`

## 7. Reactive state, computed data and change streams

### 7.1 Synchronous database events

`DEFINE EVENT` associates logic with table mutations. Event bodies can inspect `$before`, `$after` and `$event`, and can execute SurrealQL operations against related records.

A synchronous event participates in the triggering transaction. This allows, for example, maintaining a related status record or audit row atomically with the originating mutation.

This can replace bespoke application callbacks when the invariant belongs to the database's own state.

However, events are not a universal interception mechanism. The official documentation notes that events are not fired during bulk imports, an important distinction when using import mechanisms to reconstruct or load state.

### 7.2 Asynchronous events

An asynchronous event separates subsequent processing from the originating transaction. The initiating write can commit before the event body runs; the later work executes under its own transaction boundary.

This can reduce critical-path work, but it does not preserve the same all-or-nothing guarantee between the original write and the eventual derived effect.

SurrealDB provides controls for asynchronous execution and retry behavior, but applications interacting with external systems still need to account for failure, duplicate effects and recovery.

Source: `https://surrealdb.com/docs/learn/schema-management/events-and-triggers/defining-events`

### 7.3 Computed fields versus stored derived state

A field defined with `COMPUTED` is derived at read time. A field using `VALUE` normalizes or transforms what is stored on write.

These are not interchangeable.

A computed value naturally reflects the source state but adds query-time work. A stored value can be cheaper to read and easier to index, but has to be maintained correctly.

SurrealDB also supports table views and aggregation-oriented derived representations. Together with events and materialized references, these create several native mechanisms for producing denormalized or read-optimized state.

The relevant distinction is when derivation happens, whether the result is stored, and whether updates participate in the original transaction.

Source: `https://surrealdb.com/docs/learn/schema-management/computed-data/computed-fields`

### 7.4 Live queries

`LIVE SELECT` registers a subscription that delivers changes matching the query as records are created, updated or deleted.

Notifications are based on committed state. They can use projections, filtering and record/field permissions; subscribers do not simply receive unrestricted copies of all changed records.

The semantics are designed for live interfaces and subscriptions rather than durable replay. Notifications associated with one client's commits retain that client's commit order, but SurrealDB does not promise a single global notification order across all concurrent writers.

A live query therefore should not be treated as an exactly-once, globally ordered message broker.

Source: `https://surrealdb.com/docs/learn/querying/real-time/live-queries`

### 7.5 Changefeeds and external synchronization

Changefeeds provide a different mechanism: a table or database explicitly retains changes for a configured duration, and clients retrieve them using `SHOW CHANGES`.

A consumer can maintain a replay cursor using the returned versionstamp. This supports incremental export, secondary indexes, cache updates and synchronization to external systems.

A useful pattern is to apply a batch of changes and advance the consumer cursor in the same transaction on the destination. This protects the destination from moving its cursor past unapplied changes.

There are still important boundaries. Retention is finite; falling behind beyond the retention window requires resynchronization. A changefeed is not automatic replication, and the consuming process remains responsible for transferring and applying data.

Source: `https://surrealdb.com/docs/learn/querying/real-time/changefeeds`

## 8. Query interfaces, SQL interoperability and SDKs

### 8.1 SurrealQL is the native general-purpose surface

SurrealQL exposes the database's complete model: record addressing, graph traversal, schema operations, nested values, transaction control, custom functions, vector and full-text predicates, and other engine-specific behavior.

Supported operations extend beyond conventional SQL-style CRUD. They include `RELATE`, `UPSERT`, subqueries, parameter binding, control flow, user functions and direct record-path expressions.

Consequently, SurrealQL is the reference interface for functionality that cannot be represented faithfully through an external SQL dialect.

Source: `https://surrealdb.com/docs/reference/query-language`

### 8.2 PostgreSQL protocol and dialect support in 4.0

This is one of the release's most consequential interoperability additions.

SurrealDB 3.3 already offered a PostgreSQL wire-protocol listener, but clients used SurrealQL over it by default. SurrealDB 4.0 changes that: PostgreSQL connections now run a supported PostgreSQL SQL dialect, with an option to switch to ISO GQL.

The server implements PostgreSQL protocol v3 interactions, including prepared statements, bound parameters, transactions, cancellation, authentication and selected catalog metadata.

Supported SQL functionality includes joins, grouping, aggregations, nonrecursive CTEs, many window functions, schema changes, upserts, savepoints, and text/CSV `COPY` operations.

The same underlying SurrealDB records are visible through this interface and through native SurrealQL.

It is not a PostgreSQL server internally. PostgreSQL system catalogs are emulated to the extent documented, SQL behavior is a defined subset, and PostgreSQL storage internals, native extensions, server administration and all tool-specific catalog queries are not automatically available.

For example, the protocol documentation states that `pg_dump` is not yet supported because the catalog emulation does not satisfy all of its queries.

The SQL listener must be explicitly enabled using `--postgres-bind`. SurrealQL cannot be sent over this listener in 4.0; it remains available through HTTP, RPC and native SDKs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rest-api/postgres-protocol`

### 8.3 MySQL protocol and dialect support

Version 4.0 also adds a MySQL listener, enabled with `--mysql-bind`.

It accepts a documented subset of MySQL 8-style syntax, including joins, aggregation, nonrecursive CTEs, prepared statements, transactions, savepoints and `INSERT ... ON DUPLICATE KEY UPDATE`.

Like the PostgreSQL interface, this is a translation into SurrealDB's own data and execution machinery, not an embedded MySQL database.

Client and ORM compatibility depends on the exact statements the application or framework generates. A successful connection, metadata query or simple `SELECT` does not imply that all schema migrations and advanced SQL expressions will work.

The protocol documentation also records differences in dump tooling, authentication and administrative behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rest-api/mysql-protocol`

### 8.4 Important release-day SQL documentation discrepancy

There is a material inconsistency in the official November 3, 2026 material.

The 4.0 release notes and the detailed PostgreSQL and MySQL protocol references explicitly document window-function support, including functions such as `row_number`, `rank`, `lag` and `lead`, subject to restrictions on certain frame types and expressions.

However, the accompanying launch blog and product landing page state that window functions are not supported.

The more specific release notes and protocol references provide concrete supported syntax and examples, so this overview treats the documented subset as supported while preserving that publication inconsistency. The support should not be extrapolated to unrestricted PostgreSQL or MySQL window semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+3



### 8.5 pgvector compatibility

The PostgreSQL dialect supports pgvector-style column declarations, distance operators and compatible HNSW index creation syntax.

The important boundary is that a `vector` column and `CREATE INDEX ... USING hnsw` map to SurrealDB's vector representation and index implementation; they do not load the PostgreSQL `pgvector` extension.

This can make existing PostgreSQL vector tooling usable with relatively limited query changes, but exact compatibility still depends on data types, operator behavior, supported SQL constructs and the client library's initialization queries.

Source: `https://surrealdb.com/docs/reference/rest-api/postgres-protocol`

### 8.6 MongoDB interoperability through mql.js

The separately distributed mql.js 1.0 library supplies compatibility with parts of the MongoDB Node.js driver API, including aggregation-pipeline operations and Mongoose integration.

This is a client-library compatibility layer, not a native MongoDB wire-protocol listener.

It does not mean that `mongosh`, MongoDB Compass or drivers for other languages can connect to SurrealDB as though it were a MongoDB server. Compatibility must be assessed at the supported driver-method and query-operation level.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Source: `https://surrealdb.com/blog/surrealdb-4-the-database-and-memory-for-every-agent`

### 8.7 Rust SDK, embedding and other client interfaces

The Rust `surrealdb` crate serves both as a database client and, with appropriate feature flags, an embedded engine interface.

Version 4.0 provides asynchronous query execution, typed deserialization, transaction handling, connection management and support for relevant local or remote protocols.

Feature selection changes which mechanisms are compiled in. Significant feature families include `kv-mem`, `kv-surrealkv`, `protocol-ws`, `protocol-http` and `protocol-grpc`. The previous `kv-rocksdb`, `kv-indxdb` and built-in `kv-tikv` features are removed.

Version 4.0 also extends the Rust SDK with `Surreal::api()` for database-defined endpoints, local-engine durability configuration, datastore options and explicit shutdown semantics. The `parse` feature is no longer enabled by default.

The SDK version, server version and storage-engine version are distinct. Other language SDKs have independent release histories and compatibility requirements; the server's 4.0 version number must not be transferred automatically to each client package.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Source: `https://docs.rs/crate/surrealdb/4.0.0`

## 9. Security, database APIs and programmable extensions

### 9.1 Authentication and authorization

SurrealDB separates two identity models.

System users operate at root, namespace or database scope. Their access is controlled through database roles, including `OWNER`, `EDITOR` and `VIEWER`.

Record users are application identities represented as records. Their authentication can be defined using `DEFINE ACCESS ... TYPE RECORD`, with custom signup and signin logic. Authorization is primarily expressed through table- and field-level `PERMISSIONS`.

SurrealDB also supports external JWT-based authentication, allowing identities established by another provider to participate in database authorization.

For record users, table permissions default to denial. Field permissions default to allowing access subject to the enclosing table's rules. System users are controlled by their roles rather than those row-level record permissions.

These distinctions matter because a query's effective identity determines which records and fields are visible, including through subscriptions and relationship traversal.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Sources:

`https://surrealdb.com/docs/learn/security/authentication/overview`

`https://surrealdb.com/docs/learn/security/authorization/permissions-and-row-level-security`

### 9.2 Database-defined HTTP APIs

`DEFINE API` permits database-managed HTTP endpoints with routes, handlers and middleware.

Middleware can be implemented using custom SurrealQL functions and closures, with access to request properties, response transformations and the next handler in the chain.

This provides a middle ground between exposing arbitrary query access and implementing every simple endpoint in a separate application service.

Version 4.0 adds the significant `SECURITY INVOKER` option. It makes the endpoint execute under the caller's permissions, rather than relying on potentially elevated execution authority.

That is an important safety and semantic distinction: a database API can expose a controlled operation without implicitly giving the operation access beyond what its caller possesses. The precise security mode must nevertheless be verified when defining endpoints.

Sources:

`https://surrealdb.com/docs/learn/querying/custom-apis/managing-apis`

`https://surrealdb.com/docs/learn/querying/custom-apis/middleware`

### 9.3 Browser authentication and request scoping

Version 4.0 adds cookie-based authentication through an explicitly enabled server option.

With `--cookie-auth` and appropriate origin restrictions, SurrealDB can issue `HttpOnly` session cookies and support refresh-token workflows for record access definitions configured with refresh support.

This reduces the need for browser application code to handle bearer tokens directly, but introduces the ordinary security obligations of cookie authentication, particularly origin configuration and cross-site request protections.

HTTP endpoints can also express namespace/database scope directly in their paths, for example `/ns/{ns}/db/{db}/sql`. This can simplify routing through proxies and tools that otherwise need SurrealDB-specific headers.

The RPC query interfaces additionally support trace metadata describing relations, references and links encountered by a query. That can be useful for independent inspection of the graph context underlying a result.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



### 9.4 Surrealism: WASM extensions inside the database

Surrealism allows custom Rust functionality compiled to WebAssembly to execute within SurrealDB.

In 4.0, these packages are called extensions, replacing the previous modules terminology.

The corresponding database interface includes `DEFINE EXTENSION`, `ALTER EXTENSION`, `REMOVE EXTENSION` and functions invoked through `ext::`.

The important architectural mechanisms are:

Signed provenance. Extension definitions identify a signer or explicitly declare the archive unsigned. They can also pin an archive digest, allowing the database to associate executable code with an expected artifact.

Capability-scoped execution. Definitions grant specific permissions for network access, functions, queries, scripting, writes and resource limits. An empty capability declaration does not implicitly grant those capabilities.

Per-invocation isolation. Each call starts in a fresh instance. Extensions cannot assume persistent in-memory state between function calls.

Explicit transactional access. The 4.0 Rust ABI changes how an extension receives database transaction access, including read/write privileges.

Compatibility boundary. Extensions built for 3.x must be rebuilt for the 4.0 ABI, and old stored definitions must be re-established with the required signature metadata.

These capabilities make extensions valuable for domain-specific operations that are close to the data and benefit from database-managed execution.

They are not equivalent to arbitrary unrestricted Rust plugins: the WASM boundary, supported host interfaces, capability declarations and transactional semantics determine what extension code can do.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Release-status qualification: The 4.0 extension reference still requires enabling the experimental `surrealism` capability. Some of its examples also mention enabling experimental `files`, although file storage is explicitly stable in 4.0. The extension-system requirements should therefore be read separately from the stale file-storage example.

Sources:

`https://surrealdb.com/docs/reference/query-language/statements/define/extension`

`https://surrealdb.com/releases/4.0`

### 9.5 File storage becomes stable

Version 4.0 promotes file and bucket storage out of experimental status.

Buckets can be defined at root, namespace or database scope with explicit storage backends. Files have their own database values and API operations, rather than requiring an unrelated storage service merely to associate content with records.

New functionality includes presigned upload/download URLs, signature revocation, content-type handling and HTTP bucket routes.

This can simplify systems whose records need to reference source documents, intermediate artifacts or immutable binary objects.

Nevertheless, the underlying bucket backend remains an operational dependency. Transactional changes to database records should not automatically be assumed to have identical atomicity or failure semantics with externally served file transfers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Source: `https://surrealdb.com/releases/4.0`

## 10. Storage engines and deployment architecture

### 10.1 Storage engine capability matrix

SurrealDB's storage engines implement the transactional key-value capabilities required by the query layer, but make different choices about memory use, persistence, concurrency, versioning and distribution.

| Engine / product      | Primary role                           | Notable strength                                                     | Important boundary                                                |
| --------------------- | -------------------------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------- |
| SurrealMX             | In-memory execution                    | Pipelined commit processing and low storage-access overhead          | Dataset must fit in RAM; historical `VERSION` reads unavailable   |
| SurrealKV 1.0         | Single-node persistent storage         | Durable on-disk engine; server, embedded and browser support         | Not a multi-node replicated storage engine                        |
| SurrealDS 1.0         | Enterprise distributed storage         | Replicated commit logs, object-backed persistent data, read scaling  | Enterprise licensing; one sequencer orders writes                 |
| Separate TiKV backend | Alternative distributed KV integration | TiKV-managed persistence and distributed storage characteristics     | No longer compiled into the normal 4.0 binary or Rust SDK         |
| SurrealDB Earth       | Managed serverless deployment          | Per-database isolated instances, on-demand startup and scale-to-zero | Separate hosted product, not a configurable local storage backend |

### 10.2 SurrealMX: in-memory transactions with optional persistence

SurrealMX is the in-memory backend introduced in the SurrealDB 3.x family.

It uses a pipelined commit architecture, allowing transactions to progress through distinct commit stages concurrently.

SurrealMX supports optional persistence through snapshots and append-only logs. Durability depends on the configuration of logging, synchronization and snapshotting.

This is materially different from assuming that an in-memory database automatically becomes equivalent to a conventional durable disk engine when an append-only log is enabled.

The working set must fit in memory, including room for concurrent transaction versions and other overhead. Snapshots also have different cost characteristics from compacted incremental on-disk storage.

SurrealMX supports serializable isolation, but does not support historical `VERSION` queries in the current 4.0 architecture.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/running/in-memory`

### 10.3 SurrealKV 1.0: the standard persistent backend

SurrealDB 4.0 standardizes persistent single-node storage around SurrealKV 1.0.

The former RocksDB and IndexedDB backends have been removed from the normal engine configuration.

On servers and embedded native environments, SurrealKV provides on-disk transactional storage. In browser/WebAssembly environments, it uses the browser's Origin Private File System, with execution through a dedicated worker.

This creates greater commonality between native and browser persistence without requiring the two environments to share identical resource or concurrency characteristics.

Version 4.0 improves SurrealKV's memory management, including treatment of dead cache blocks, write-buffer sizing and isolation of compaction work from query workers.

SurrealKV also supports storage compaction operations and optional versioned reads.

A material difference from SurrealMX is that SurrealKV is the standard choice for persistent data sets that cannot or should not reside entirely in memory.

A material difference from SurrealDS is that SurrealKV does not itself provide multi-node replication, sequencer failover or availability across independent failure zones.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Sources:

`https://surrealdb.com/docs/learn/data-models/architecture`

`https://surrealdb.com/docs/manage/self-hosted/deployment-models`

### 10.4 SurrealDS 1.0: distributed object-backed storage

SurrealDS is the distributed storage engine for SurrealDB Enterprise, introduced in the 4.0 release.

Its architecture separates query execution and cached data access from longer-term object-backed persistence.

A SurrealDS group contains several full database nodes, each with a local commit log and cache. Shared object storage holds durable database pages and log segments.

The nodes have different responsibilities:

Sequencer: establishes the order of writes and coordinates commits.

Acknowledging witness: persists the commit log in another failure zone.

Listener: follows committed state and serves reads, while remaining capable of taking another role when needed.

A write is acknowledged only after the commit has been persisted on disks in two separate zones. Object storage is updated asynchronously and is not on the immediate commit path.

Reads combine the published object-store state with newer log entries, using local memory and disk caches to reduce object-store access.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/manage/enterprise/product/surrealds`

### 10.5 What SurrealDS scales—and what it does not

SurrealDS separates three kinds of scaling.

Storage capacity can grow with object storage. Query and read capacity can increase with additional nodes. Replication and failover allow continued service across appropriate node or zone failures.

Write throughput is not automatically horizontally partitioned across independent sequencers. A group uses one sequencer to establish write order, so simply adding nodes does not multiply write sequencing capacity.

This is an important distinction from distributed databases whose write architecture partitions transactions across multiple independent shards or consensus groups.

SurrealDS requires at least two independent failure zones for its default acknowledged-write behavior. A three-node, three-zone arrangement can tolerate the loss of a single zone while retaining two zones for continued writes.

A SurrealDS group that has expanded beyond one node cannot subsequently be reopened as a single-node group.

The 4.0 release notes identify S3-compatible storage, Azure Blob Storage and Google Cloud Storage support. The detailed deployment documentation additionally specifies requirements such as conditional writes on supported object stores, durable local commit-log disks, inter-node QUIC communication with mutual TLS, and Enterprise licensing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 10.6 SurrealDB Earth and managed Cloud

SurrealDB Earth is a separate hosted serverless offering launched alongside SurrealDB 4.0.

It runs isolated database instances on Cloudflare infrastructure. Each database has dedicated storage and a commit log and can start on demand and scale to zero when idle.

The product is significant for fine-grained per-database isolation and workloads with highly variable activity.

However, its cold-start characteristics, request billing, instance placement and aggregate capacity claims are attributes of the hosted Earth service, not guarantees that automatically apply to self-hosted SurrealDB 4.0.

SurrealDB Cloud also offers managed instance and distributed deployment models; the appropriate scaling and availability characteristics depend on which service and instance type is used.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Source: `https://surrealdb.com/blog/surrealdb-4-the-database-and-memory-for-every-agent`

## 11. Operational tooling, inspection and data movement

### 11.1 CLI and runtime configuration

SurrealDB 4.0 uses the Surreal CLI's database component, with operations for server startup, SQL execution, import/export, validation, readiness and upgrade-related workflows.

The CLI and database server are separate versioned concerns. Legacy command locations and startup options have changed as the newer CLI organization has developed.

The Rust embedded interface additionally exposes storage and durability options through its configuration API.

Examples of important server-level configuration surfaces include query memory limits, temporary file locations, storage persistence settings, experimental capabilities, network permissions, protocol listeners and tracing/telemetry configuration.

These controls affect behavior that cannot be inferred from SurrealQL definitions alone.

### 11.2 Observability and diagnostics

SurrealDB has an OpenTelemetry-oriented observability architecture, with Prometheus-compatible metrics and OTLP export.

The available signals include query, transaction, authentication, RPC, HTTP, session, storage and other runtime activity. Enterprise deployments extend this with distributed storage metrics and dedicated audit/slow-query logging capabilities.

Operational inspection includes health and readiness endpoints, structured logs, execution plans and query tracing.

Version 4.0's improvements to plan consistency and query tracing are particularly valuable for reasoning about how graph relationships, references and indexes are actually being used.

The distinction between Community and Enterprise observability is important: general metrics and traces are not the same as the Enterprise audit-log and slow-query record pipelines.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/manage/observability`

### 11.3 Import, export and schema tooling

SurrealDB offers native import/export operations and supports ingesting data from file-based and external sources.

The database's own schema is introspectable through `INFO` statements, while companion tooling such as SurrealKit supports schema synchronization and managed rollout workflows.

Schema tools and database-engine migration mechanisms should be distinguished. SurrealKit manages definitions and deployment workflows; 4.0's native `MIGRATE EXISTING ROWS` mechanism handles a particular category of stored-field transformation.

Import/export also has semantic limits. Schema definitions, credentials, secrets, generated structures and indexed state are not necessarily reproduced by every format or path in identical fashion.

A full backup-and-restore strategy is therefore broader than collecting a text export, particularly for storage-backed features and security material.

Sources:

`https://surrealdb.com/docs/learn/schema-management`

`https://surrealdb.com/docs/build/migrating`

## 12. Integration boundaries and companion products

Several related technologies are often discussed alongside SurrealDB. Their responsibilities should remain distinct.

| Component              | Relationship to SurrealDB 4.0     | Boundary                                                                                                              |
| ---------------------- | --------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| SurrealMX              | Storage backend                   | Transactional in-memory state, not a general independent replacement for all persistent-engine features               |
| SurrealKV              | Persistent key-value engine       | Storage and transaction primitives; SurrealQL planning and graph semantics belong to the database above it            |
| SurrealDS              | Enterprise distributed storage    | Replication, commit sequencing and object-backed persistence; not part of a standard single-node Community deployment |
| Surrealism             | WebAssembly extension framework   | Custom capability-scoped functions, not unrestricted arbitrary Rust execution                                         |
| SurrealKit             | Schema/deployment tooling         | Manages schema workflows rather than executing the database's query plans                                             |
| mql.js                 | MongoDB API compatibility library | Node.js compatibility, not a MongoDB wire-protocol server                                                             |
| SurrealDB MCP          | Agent-facing tool interface       | Database operations exposed to MCP clients, subject to authentication and permissions                                 |
| SurrealDB Agent Memory | Separate managed memory product   | Higher-level memory extraction, organization and retrieval; not automatically supplied by installing the engine       |
| SurrealDB Earth        | Managed serverless hosting        | Deployment service with its own scaling and billing characteristics                                                   |

### 12.1 MCP integration

SurrealDB offers an MCP interface exposing typed tools for operations including querying, selecting, creating, updating, deleting, relating records and inspecting schema.

The database-local MCP surface and the separately hosted SurrealDB Cloud MCP service are different integration contexts.

The hosted service also offers account, organization, deployment, monitoring and Agent Memory capabilities, governed by Cloud permissions.

The important architectural value is allowing tools and agents to operate against the database through a discoverable, structured interface rather than inventing a new tool server for every basic CRUD or schema-inspection operation.

That convenience does not relax the need to scope credentials, restrict destructive operations and distinguish read-only inspection from write authority.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



Source: `https://surrealdb.com/docs/build/ai-agents/mcp`

### 12.2 Agent Memory versus database-native retrieval

SurrealDB's database engine can maintain entity records, relation graphs, documents, embeddings, histories and retrieval indexes.

The separate Agent Memory service adds higher-level managed workflows around memory capture, organization and recall.

Those higher-level behaviors should not be conflated with native database storage.

The engine can provide the persistence and retrieval mechanisms needed for sophisticated memory systems. It does not automatically infer an ontology, extract entities from arbitrary content or produce embeddings merely because a record or relation table exists.

### 12.3 SurrealML is removed in 4.0

A particularly important compatibility change is the removal of the former SurrealML model-storage and execution interface.

The 4.0 release removes the associated CLI/model operations and HTTP endpoints, and refuses legacy model definitions. An existing datastore containing SurrealML models cannot start under 4.0 until the models are removed through the supported pre-upgrade process.

This is a material change for systems that formerly relied on database-managed model execution. Vector persistence and search remain supported, but the removed SurrealML interface should not be treated as an available native inference subsystem.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



## 13. Version migration and consequential incompatibilities

The following distinctions matter specifically when moving from SurrealDB 3.x to 4.0.

| Area               | 4.0 change                                         | Consequence                                                                    |
| ------------------ | -------------------------------------------------- | ------------------------------------------------------------------------------ |
| Datastore format   | New major record format                            | One-way migration; 3.x cannot reopen a converted datastore                     |
| Migration status   | 3.x in-place upgrade experimental                  | Explicit opt-in required; production migration tooling planned for 4.1         |
| Persistent storage | SurrealKV replaces RocksDB/IndexedDB engines       | Storage conversion required where older backends were used                     |
| SQL wire protocol  | PostgreSQL connections now use PostgreSQL SQL      | Existing clients that sent SurrealQL over this listener must change interfaces |
| Vector indexes     | Changed distance ordering and faster index layouts | Some rebuilds required for correctness; others beneficial for performance      |
| Full-text indexes  | Posting-list improvements and fixes                | Existing affected indexes may require rebuilding for correct scoring           |
| Extension ABI      | Modules become signed/capability-scoped extensions | Rebuild and redefine existing extension archives                               |
| Schema changes     | New migration/compatibility rules                  | Compatibility level controls enforcement on upgraded datastores                |
| SurrealML          | Removed                                            | Legacy model assets must be handled before upgrading                           |
| Authentication     | Stronger bearer-grant validation                   | Previously issued tokens may be rejected                                       |
| Rust embedding     | Removed older KV feature flags                     | Dependency features and local engine initialization may need changes           |

The detailed migration guide is important because it distinguishes index rebuilds needed for correctness from rebuilds that mainly enable 4.0's performance improvements.

Examples of required rebuilds include HNSW indexes using Jaccard or Pearson ranking under the previous ordering, and certain indexes affected by historical update/delete or numeric representation bugs.

Other vector, full-text and B-tree indexes can continue to return correct results but miss the improved 4.0 execution layout until rebuilt.

The migration guide also documents cases where affected DiskANN indexes must be redefined rather than merely rebuilt.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x`

### 13.1 In-place migration requires explicit consent

For an existing 3.x datastore, 4.0 refuses to start by default rather than silently converting it.

The experimental path requires:

```
SURREAL_EXPERIMENTAL_UPGRADE_FROM_3X=true \
  surreal db start surrealkv://path/to/data
```

An embedded Rust engine uses the corresponding datastore option rather than assuming the process environment variable is sufficient.

This conversion is irreversible at the datastore-format level. The reliable route back to 3.x is to restore a pre-upgrade copy or export, not to restart the older binary against the converted files.

The vendor's stated recommendation at this release is to use 4.0 for new deployments and wait for more mature 4.1 migration tooling before upgrading existing production deployments.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 13.2 Licensing considerations

SurrealDB's core database source is distributed under the Business Source License 1.1, with an additional use grant.

The official licensing FAQ permits normal self-hosting, embedding, modification and production use, but reserves certain database-as-a-service offerings in which third parties manage their own database schemas or tables.

SDKs and some supporting components use Apache 2.0 or MIT licenses. Enterprise functionality, including SurrealDS, requires separate commercial licensing.

These are material distinctions for products that expose database management capabilities to external customers, even if they use the same query model and Rust libraries as an internally deployed database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



Source: `https://surrealdb.com/license`

## 14. Architectural synthesis: where 4.0 changes the design space

The most significant architectural value of SurrealDB 4.0 comes from how several capabilities interact.

A unified model for domain state. Typed records, nested structures, first-class relationships, named types and schema migrations can represent both flexible source material and strongly constrained business entities. Record IDs and relation edges provide persistent identity without requiring every access path to be built as a bespoke mapping layer.

Search over canonical records. Full-text, vector, spatial and structured indexes can operate over the same underlying records. Graph traversal then adds relationship context to retrieval. This can eliminate the synchronization machinery otherwise required when a vector database, search index and graph database each maintain separate copies of entity state.

Transactional semantics across models. Updates to records, edge relationships, indexes, reference structures and synchronous event effects can participate in one transactional operation. This is an advantage over loosely coupled services, though it does not extend automatic atomicity to external API calls or separate managed products.

Multiple physical representations for related data. Record links, graph edges, reference indexes, computed fields, materialized reverse references and derived views allow the same logical domain relationship to be represented differently according to lookup frequency, update characteristics and consistency requirements.

Reuse of standard integration infrastructure. PostgreSQL/MySQL protocol support, pgvector-compatible SQL, MCP and SDK interfaces provide several alternatives to writing custom adapters. The interoperability is significant precisely because these interfaces address the same database state, not because they make SurrealDB identical to each underlying ecosystem.

Deployment flexibility with explicit consistency boundaries. SurrealMX, SurrealKV and SurrealDS share higher-level database abstractions while exposing different isolation, history, capacity and durability properties. The ability to choose among them is valuable, provided engine-dependent guarantees are not hidden behind the common API.

The principal limitations are equally important: SQL dialect compatibility is incomplete; ANN results are approximate unless exact search is selected; distributed writes retain a single-sequencer coordination point; extension execution requires explicit capability and ABI management; and an existing 3.x production datastore has a substantially different upgrade-risk profile from a newly created 4.0 datastore.

## 15. Primary reference map

The most useful authoritative resources for further investigation are the following.

| Area                                      | Primary source                                                                              |
| ----------------------------------------- | ------------------------------------------------------------------------------------------- |
| Complete version-specific release details | `https://surrealdb.com/releases/4.0`                                                        |
| Core database architecture                | `https://surrealdb.com/docs/learn/data-models/architecture`                                 |
| SurrealQL language and statements         | `https://surrealdb.com/docs/reference/query-language`                                       |
| Transactions and isolation                | `https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions`                |
| Graph/reference semantics                 | `https://surrealdb.com/docs/reference/query-language/language-primitives/record-references` |
| Vector indexes                            | `https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes`                 |
| Full-text indexing                        | `https://surrealdb.com/docs/learn/data-models/full-text-search/overview`                    |
| PostgreSQL protocol                       | `https://surrealdb.com/docs/reference/rest-api/postgres-protocol`                           |
| MySQL protocol                            | `https://surrealdb.com/docs/reference/rest-api/mysql-protocol`                              |
| WebAssembly extensions                    | `https://surrealdb.com/docs/reference/query-language/statements/define/extension`           |
| SurrealDS architecture                    | `https://surrealdb.com/docs/manage/enterprise/product/surrealds`                            |
| Rust SDK 4.0 crate and features           | `https://docs.rs/crate/surrealdb/4.0.0`                                                     |
| 3.x → 4.0 migration                       | `https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x`           |
| Observability                             | `https://surrealdb.com/docs/manage/observability`                                           |

Overall assessment: SurrealDB 4.0 is significant not merely because it adds more supported syntaxes or index types, but because it strengthens the engine underneath a unified document–relational–graph–search model. The bytecode executor, schema evolution machinery, index improvements, SQL interoperability and clearer transaction capabilities create more opportunities to delegate complex data operations to native database mechanisms. The most consequential engineering decisions remain the representation of relationships, the required isolation guarantees, the execution characteristics of combined retrieval, and the choice of storage backend.

# SurrealDB 4.0 — Technical Capability Addendum

Supplement to the original SurrealDB 4.0 reference · Target: 4.0.0 · Research date: November 3, 2026

This addendum documents the functionality identified as missing or insufficiently developed in the original SurrealDB 4.0 overview. It concentrates on physical graph representations, incremental materialization, query optimization, schema execution semantics, binary and SQL interoperability, database programming, and operational compatibility.

The distinction between new in 4.0 and introduced earlier but supported in 4.0 is maintained throughout. Several particularly important mechanisms, including lightweight graph relations, adjacency caches, sequences and bitmap index fusion, originated in SurrealDB 3.x but remain significant parts of the 4.0 architecture.

## 1. Graph physical representations and execution optimizations

SurrealDB provides several native representations for relationships. Their differences extend beyond query syntax: they determine whether an association has an independent stored record, where its properties reside, how adjacent relationships are retrieved, and which invariants are maintained transactionally.

### 1.1 `LIGHTWEIGHT`: record-less relationships

Introduced in 3.3 · Supported in 4.0

A conventional `TYPE RELATION` table maintains an edge record for each relationship. By contrast, `LIGHTWEIGHT` relations store the connection through adjacency structures without maintaining a separate edge-record payload.

```
DEFINE TABLE follows
    TYPE RELATION IN person OUT person
    LIGHTWEIGHT;

RELATE person:alice->follows->person:bob;
```

The relationship's identity is canonical and derived from its endpoints:

```
follows:[person:alice, person:bob]
```

This has several consequences:

- Set-like identity: Repeating the same `RELATE` is idempotent. A conventional relation can instead create multiple independently identified edges between the same endpoints.
- Reduced storage: There is no independently stored edge record or corresponding per-edge secondary index.
- Synthesized reads: A query can still retrieve the logical `id`, `in` and `out` fields of the relationship.
- Enforced endpoints: `LIGHTWEIGHT` implies `ENFORCED`; both endpoint records must exist.
- Limited mutation: Edge properties, `DEFINE FIELD`, `DEFINE INDEX`, `DEFINE EVENT`, `LIVE SELECT` on the relation table and updates to edge payloads are unavailable.

Lightweight edges can be deleted and traversed. They cannot serve as the endpoint of another relation, since they lack the independently stored record needed by that representation.

The capability is appropriate when the relationship's identity is precisely its ordered endpoint pair and no independent attributes are required. It is not a substitute for a relationship that must carry provenance, effective dates, confidence, multiplicity or other properties.

Architectural significance: Many-to-many membership or connectivity can be represented without manually constructing composite edge IDs, uniqueness indexes and deduplication logic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

### 1.2 `ENFORCED` relation endpoints

`ENFORCED` is independently useful on conventional relation tables:

```
DEFINE TABLE located_in
    TYPE RELATION IN facility OUT region
    ENFORCED;
```

It prevents insertion of a relationship whose endpoints do not exist. Without it, an ordinary relation may refer to missing endpoint records.

This is an insertion-time referential constraint, not a claim that every relation behaves exactly like a SQL foreign key. Endpoint deletion, cascading behavior and import behavior have their own semantics.

In particular, bulk import processing defers endpoint checks because an export may contain relation records before their referenced vertex records.

`LIGHTWEIGHT` automatically adopts enforced endpoint behavior and uses endpoint-based cleanup because there is no conventional edge record to preserve.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 1.3 `INLINE EDGES` and `INLINE REFERENCES`: transactionally maintained adjacency caches

Introduced in 3.3 · Supported in 4.0

Normal graph traversal reads adjacency keys from storage. When a query traverses relationships from many starting records, it may perform one key-range lookup per source record.

`INLINE EDGES` keeps a compact adjacency cache associated with each eligible vertex. `INLINE REFERENCES` provides an equivalent mechanism for inverse record-reference lookup.

```
DEFINE TABLE person
    INLINE EDGES 64
    INLINE REFERENCES 32;
```

The clauses apply to the vertex table, not the relation table.

When a record has no more than the configured number of adjacency entries, the database can retrieve its cached entries through a batched lookup. Writes update the cache within the same transaction as the relationship mutation.

Important behavior:

| Property                | Semantics                                                        |
| ----------------------- | ---------------------------------------------------------------- |
| Capacity                | Configurable from 0 to 256                                       |
| Scope                   | Per record; edge cache applies separately to each direction      |
| Initialization          | Built when relevant edges or references are subsequently written |
| Overflow                | Falls back to normal key-range scanning                          |
| Recovery after overflow | Changing the configured cap resets the cache lifecycle           |
| Write cost              | Updating adjacency rewrites the cached value                     |
| Consistency             | Cache maintenance is transactional                               |
| Diagnostics             | `GraphEdgeScan` / `ReferenceScan` report cache hits and misses   |

Caching is beneficial when many low- or moderate-degree vertices are traversed together. The bounded size avoids requiring every high-degree vertex to maintain a potentially enormous adjacency value.

A central tradeoff is that each mutation can rewrite the entire cached adjacency entry. Larger caches can reduce read I/O while increasing write amplification.

These caches do not accelerate every access path. For example, traversal over a range of edge IDs, historical `VERSION` queries and some predicate-pushdown paths use the underlying keys instead.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

### 1.4 `DEFINE FIELD ... INLINE`: filtering before edge-record retrieval

A separate optimization stores a small edge property directly inside the adjacency entry as well as in the original edge record.

```
DEFINE TABLE likes
    TYPE RELATION IN person OUT article;

DEFINE FIELD score ON likes
    TYPE number
    INLINE;
```

The following expression can evaluate its predicate while scanning adjacency entries:

```
person:alice->(likes WHERE score > 8)->article
```

This differs from filtering the resulting array:

```
person:alice->likes[WHERE score > 8]->article
```

In the first form, the predicate is inside the graph path and can be evaluated against the inline payload. The second form retrieves relationships before applying the array filter.

The database can avoid loading full edge records for candidates rejected by the inline predicate.

The inline payload is limited by `SURREAL_GRAPH_INLINE_PROPS_CAP`, which defaults to 64 bytes. Oversized values remain on the original edge record and require fallback access.

`EXPLAIN` identifies eligible pushed-down predicates with `predicate_scope: payload`.

Architectural significance: Small, frequently filtered edge properties can act as adjacency-local indexes without creating an unrelated materialized structure. The benefit depends on predicate selectivity and the balance between relationship reads and updates.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/field`

### 1.5 Background graph adjacency folding

SurrealDB also compacts adjacency entries independently of the per-record inline caches.

Recently written edge entries can remain in a delta representation. A background task folds these entries into packed adjacency blocks, reducing the number of separate key reads needed for traversal.

The process is controlled through:

- `--graph-fold-interval`
- `SURREAL_GRAPH_FOLD_INTERVAL`

The documented default interval is five seconds, with processing also triggered when queued work is available.

`EXPLAIN ANALYZE` exposes `block_hits` for entries read from packed blocks and `delta_hits` for entries read from the recent-change representation.

This gives the engine two complementary ways to improve graph reads: compacting adjacency storage over time and caching bounded adjacency sets on individual vertices.

Neither requires application-managed graph-cache invalidation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 1.6 Physical relationship choices

| Mechanism                   | Stored independently                | Relationship attributes                    | Main purpose                          |
| --------------------------- | ----------------------------------- | ------------------------------------------ | ------------------------------------- |
| Record link                 | In a record field                   | On the referring record                    | Direct association                    |
| Tracked reference           | Link plus inverse tracking          | On the referring record                    | Efficient inverse reference lookup    |
| Conventional relation       | Yes                                 | Arbitrary edge fields                      | Independently identified relationship |
| `LIGHTWEIGHT` relation      | No separate edge record             | No                                         | Compact, set-like connectivity        |
| `INLINE EDGES`              | Cached adjacency value              | Does not replace edge properties           | Batched neighbor retrieval            |
| Inline relation field       | Edge plus duplicated small property | Yes                                        | Predicate evaluation during traversal |
| Background adjacency blocks | Packed storage entries              | Preserved according to underlying relation | Reduce adjacency key access           |

These mechanisms are composable where their restrictions permit. They represent different physical strategies for relationship identity, retrieval and maintenance—not separate graph databases.

## 2. Incrementally maintained analytical state

### 2.1 `DEFINE TABLE ... AS SELECT` creates a maintained view

SurrealDB's precomputed table views are materially different from ordinary virtual SQL views.

A definition such as:

```
DEFINE TABLE sales_total AS
SELECT
    count() AS sales,
    math::sum(amount) AS total
FROM sale
GROUP ALL;
```

creates a materialized result that is maintained incrementally as source records change.

The initial definition populates the result from existing source records. Subsequent inserts, updates and deletes trigger maintenance of the affected view rows rather than requiring a full recomputation of the entire underlying query.

This functionality can replace bespoke event handlers for many aggregate-maintenance problems, including counters, totals, averages and grouped summaries.

The maintenance work is associated with source-table mutations, making the derived state part of the database's own write-maintenance model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 2.2 `GROUP ALL` preserves an aggregate row for an empty source

A view grouped with `GROUP ALL` retains its single aggregate row, even if the source table has no records.

For the example above, the materialized result is conceptually:

```
[
    {
        id: sales_total:[],
        sales: 0,
        total: 0
    }
]
```

When the last source record is deleted, the view row is reset rather than removed.

This is important for aggregation consumers that need a persistent result identity. Events attached to such a view observe updates to the aggregate row rather than a repeating create/delete lifecycle.

### 2.3 View-row identity can be projected explicitly

Version 4.0 improves the use of a projected `id` as the actual identity of a materialized view row.

```
DEFINE TABLE customer_spend AS
SELECT
    customer AS id,
    math::sum(total) AS spend
FROM purchase
GROUP BY customer;
```

The projected identity can be a string, number, UUID, array or object rather than an already-formed record ID.

SurrealDB converts that value into the view-table identity. A change to the projected identity moves the materialized row to its new key.

An identity expression producing `NONE` fails the originating write instead of silently creating a row with no valid key.

Before 4.0, initial view population and later incremental updates could disagree about the key for a projected identity. The 4.0 behavior closes that inconsistency.

### 2.4 Source dependencies are narrower than arbitrary query dependencies

A crucial limitation is that view maintenance is triggered by writes to its declared source table or tables.

If a view reads related information through a record link, graph traversal or subquery, an update to that indirectly referenced record does not automatically mean the view will refresh.

For example, a review aggregate that projects a product's current name can update when a review changes but retain an old projected product name when only the product record changes.

Thus, incremental materialization should not be mistaken for a general-purpose, automatically inferred dependency graph covering every record touched by a query.

View definitions can depend on other views, but their maintenance order and cycle restrictions remain part of the design semantics.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

### 2.5 `DROP` tables support transient input with persistent aggregates

A `DROP` table disallows persistent record creation or updating in that table while permitting its use as a source for downstream event-driven derived state.

This makes it possible to retain aggregate results without retaining every incoming event record.

The pattern is useful for ingestion workloads where the retained business state is a summary rather than a complete event history.

There is a corresponding information-loss tradeoff: if raw inputs are discarded, the aggregate cannot subsequently be recomputed from that table alone.

### 2.6 Import and rebuild semantics

Bulk import uses different execution semantics from ordinary transactional writes.

Under `OPTION IMPORT`, SurrealDB bypasses normal view maintenance, among other write-path operations. Imported records therefore do not necessarily appear in an existing materialized view immediately.

A documented refresh mechanism is:

```
DEFINE TABLE OVERWRITE sales_total AS
SELECT
    count() AS sales,
    math::sum(amount) AS total
FROM sale
GROUP ALL;
```

This recomputes the materialized contents from the current source table.

Architectural significance: Native incremental views can replace custom aggregate-update code for eligible computations, but correctness depends on understanding source dependencies, import behavior and whether enough underlying data remains to rebuild the view.

Sources:

`https://surrealdb.com/docs/reference/query-language/statements/define/table`

`https://surrealdb.com/docs/learn/data-models/time-series/aggregation-queries`

## 3. Index-only execution, bitmap fusion and query inspection

### 3.1 `COUNT` indexes: materialized cardinality

Introduced in 3.0 · Expanded planner use in 4.0

A `COUNT` index maintains record cardinality rather than storing a conventional mapping from field values to record IDs.

The unconditional form counts an entire table:

```
DEFINE INDEX total_documents ON document COUNT;
```

The filtered form maintains cardinality for a specified predicate:

```
DEFINE INDEX active_documents ON document
    COUNT WHERE status = "active";
```

The engine stores count deltas and compacts them in the background. This is physically different from a B-tree index, which maintains entries for individual indexed records.

The query form is important:

```
SELECT count()
FROM document
WHERE status = "active"
GROUP ALL;
```

A filtered count index is eligible when the query's `WHERE` expression exactly matches the index's stored predicate, the index is ready, and the relevant field permissions permit index-only counting.

The distinction between exact predicate matching and logical equivalence matters. A different but mathematically equivalent predicate is not automatically guaranteed to use the same count index.

SurrealDB also has a `CountScan` path for full-table cardinality that counts record keys without deserializing every record. Therefore, an unconditional `COUNT` index is not necessary for efficient counting in all circumstances.

### 3.2 B-tree index-only counts in 4.0

Version 4.0 extends the ability to answer counts from conventional indexes without retrieving records.

For example:

```
SELECT count()
FROM document
WHERE status IN ["draft", "review"]
GROUP ALL;
```

An eligible B-tree index on `status` can count entries for the specified values and combine the results.

The implementation places restrictions on this optimization: the value list must contain distinct literals, and an `IN` list with more than 32 values does not use this particular fast path.

A normal B-tree index is often preferable when the same field also supports filtering, ordering and record retrieval. A filtered count index has a narrower purpose but can maintain a substantially smaller persistent representation than a B-tree that stores an entry per record.

This distinction avoids treating every repeated aggregate query as a reason to maintain another full secondary index.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

### 3.3 Bitmap index fusion

Introduced in 3.3 · Supported in 4.0's unified executor

SurrealDB can combine multiple index results before fetching the underlying records.

For a query with separate indexed predicates:

```
SELECT id
FROM article
WHERE category = "databases"
    AND published = true;
```

the optimizer may construct a plan equivalent to:

```
BitmapResolve
└── BitmapAnd
    ├── BitmapIndexScan(category_index)
    └── BitmapIndexScan(published_index)
```

Each index returns a candidate set represented as a bitmap. The engine combines candidate sets through set operations before resolving the surviving record IDs.

Relevant operators include:

| Operator             | Purpose                                                                     |
| -------------------- | --------------------------------------------------------------------------- |
| `BitmapIndexScan`    | Read candidate IDs from standard or unique indexes                          |
| `BitmapFullTextScan` | Produce candidates using a full-text index                                  |
| `BitmapGraphScan`    | Produce candidates connected through an eligible one-hop graph relationship |
| `BitmapAnd`          | Intersect candidate sets                                                    |
| `BitmapOr`           | Union candidate sets                                                        |
| `BitmapAndNot`       | Subtract one candidate set from another                                     |
| `BitmapResolve`      | Fetch surviving records                                                     |
| `IndexCountScan`     | Count eligible index results without fetching records                       |

The planner can combine structured predicates, lexical matches and selected graph predicates within one candidate-set evaluation.

For an indexed vector query, bitmap results can become an eligibility filter underneath `KnnScan`. This allows the vector search to operate on a constrained candidate population.

The performance implications depend on selectivity and bitmap size. Combining multiple large candidate sets is not automatically cheaper than choosing one highly selective index.

The planner also imposes work budgets. When a bitmap branch exceeds its budget, the engine can abandon that branch or fall back to a different execution strategy rather than constructing arbitrarily expensive candidate sets.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/clauses/explain`

### 3.4 Index-assisted KNN and exact prefilter tiers

Bitmap prefiltering is particularly valuable for queries that combine semantic similarity with mandatory structured conditions.

`KnnScan` can report one of several execution tiers:

- `exact`: compute distances across an eligible candidate set directly.
- `graph`: perform ANN graph exploration using the indexed prefilter.
- `graph_unboosted`: use a graph strategy without the stronger prefilter optimization.
- `fallback`: use a less optimized execution route.

The planner's choice depends on the available indexes, candidate count and supported predicate structure.

This reveals an important distinction: a vector-index definition does not imply that every vector query will traverse that index's ANN graph. The optimizer may choose exact calculation when a prefilter has already reduced the eligible population sufficiently.

This can be desirable because avoiding approximate graph traversal may improve both execution efficiency and result exactness for small candidate populations.

### 3.5 `EXPLAIN ANALYZE` and structured plans

SurrealDB provides two complementary inspection forms.

The traditional trailing clause:

```
SELECT * FROM article
WHERE category = "databases"
EXPLAIN FULL;
```

The standalone planner/profiler interface:

```
EXPLAIN ANALYZE FORMAT JSON
SELECT * FROM article
WHERE category = "databases";
```

`EXPLAIN ANALYZE` executes the statement and reports operator-level information, including output rows, batches and elapsed time.

The JSON format exposes structured operator information and metrics suitable for analysis.

Useful inspection details include:

- Indexes selected and candidate sets produced.
- Bitmap branches and fallback indicators.
- `KnnScan` prefilter tiers.
- `GraphEdgeScan` cache hits, misses, block hits and delta hits.
- Rows produced by individual operators and where filtering occurs.
- Relative execution cost of scans, computations, grouping and sorting.

The reported plan structure is diagnostic rather than a permanently stable API. The documentation explicitly warns against building tooling that assumes one fixed execution-plan representation across versions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/explain`

### 3.6 Expression indexes, array expansion and conditional uniqueness

Several more advanced conventional-index behaviors deserve attention.

Expression indexes: Index fields can include supported field-path methods and equivalent function calls.

```
DEFINE INDEX normalized_name ON person
    FIELDS name.lowercase() UNIQUE;
```

The query predicate must use a compatible expression for the optimizer to exploit that index. Arbitrary arithmetic expressions are not accepted directly as index columns; a stored field derived with `VALUE` can provide a separate indexing surface.

Multivalue indexes: Array-valued fields can produce one index entry per element. With multiple array fields in a compound index, the index can contain entries for combinations of those elements.

This supports membership lookup but increases index size with the product of the indexed arrays' lengths.

Whole-array identity: The `...` suffix allows an array to be indexed as a single value rather than element by element.

Set semantics: From 4.0, `.*` over a field typed as a set indexes the whole set instead of treating its elements as separate index entries.

Conditional uniqueness through `NONE`: SurrealDB does not provide a general `UNIQUE WHERE` index declaration. However, a normalized field can return `NONE` when a condition is false. Since conventional unique indexes ignore `NONE`, this enables conditionally enforced uniqueness using field derivation.

The distinction between `NONE`, `NULL`, empty arrays, expanded array elements and whole-value indexing is therefore directly consequential for uniqueness guarantees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

## 4. Index construction and maintenance lifecycle

### 4.1 Definition, population and readiness

An index definition is not necessarily ready for query planning or constraint enforcement immediately after it appears in schema metadata.

Index construction involves an initial scan of existing records and processing changes that occur while the initial scan is running.

`INFO FOR INDEX` exposes construction status, including initial progress and pending updates.

The distinction between normal and concurrent construction is important:

Without `CONCURRENTLY`: The `DEFINE INDEX` statement waits for the index build to complete or fail. Its build can use separate transactions, creating restrictions when the definition is inside a larger explicit transaction.

With `CONCURRENTLY`: The statement can return while population continues in the background. The index must reach ready status before it can be relied on as a completed access path.

A failed build remains observable through metadata rather than silently disappearing.

### 4.2 Transaction visibility and conflict behavior

Version 4.0 tightens the rules for index construction inside explicit transactions.

A nonconcurrent definition may be refused if the same transaction has already modified records in the target table, because the separate index builder cannot safely incorporate those uncommitted changes.

During an eligible definition, competing writes can be made to wait, then receive a retryable conflict after the defining transaction commits.

These restrictions prevent an index from reaching an apparently valid state while having missed records that were changed concurrently.

A failed `UNIQUE` index is particularly important. If population detects duplicate values, the index can enter an error state. New writes that would require uniqueness validation can be refused until the index is repaired or removed.

This is different from a conventional lookup index that is merely unavailable for optimization during its build.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 4.3 Idempotent index definition in 4.0

`DEFINE INDEX OVERWRITE` is more efficient in 4.0.

When the index's table, fields, type and parameters are unchanged and the existing index is ready, SurrealDB can retain the existing index data rather than rebuilding it merely because the definition was reissued.

Changing metadata such as a comment can therefore be relatively inexpensive.

Changing a consequential property still requires rebuilding the index. An unchanged index definition also does not necessarily detect an external change to a dependent analyzer or another definition that changes its intended semantics.

The distinction is valuable for declarative schema synchronization: repeating an index definition need not imply repeating its full physical construction.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

## 5. Query execution resource allocation and scheduling

### 5.1 Node-wide versus per-query memory limits

SurrealDB 4.0 introduces a more explicit memory-accounting model for operators that can spill intermediate results to temporary storage.

The key controls are:

| Control                        | Purpose                                                                |
| ------------------------------ | ---------------------------------------------------------------------- |
| `SURREAL_MEMORY_LIMIT`         | Budget shared by eligible query operators on a node                    |
| `SURREAL_QUERY_MEMORY_MINIMUM` | Minimum guaranteed allocation for each query using spillable operators |
| `SURREAL_QUERY_MEMORY_MAX`     | Maximum non-spillable memory held by a query                           |
| `SURREAL_MEMORY_WAIT`          | Time allowed for waiting on memory released by other queries           |
| `SURREAL_SPILL_PART_SIZE`      | Chunk size of spilled intermediate data                                |
| `SURREAL_TEMPORARY_DIRECTORY`  | Location of spill files                                                |
| `SURREAL_MEMORY_THRESHOLD`     | Separate process-level memory guard                                    |

By default, the node query budget is 2 GiB unless `SURREAL_MEMORY_THRESHOLD` is set, in which case the default is half that threshold. The documented minimum per eligible query is 4 MiB.

Sorts, grouped aggregates, `DISTINCT` and eligible join builds can spill to temporary files. Query-bound values and gathered subquery results do not necessarily support spill and remain subject to the per-query non-spillable limit.

When the shared budget is exhausted, the engine can spill, wait for memory to become available and, under the documented conditions, continue beyond the nominal budget.

This is bounded resource accounting with fallback behavior, not a strict guarantee that a query's total resident memory remains under one configured number.

`SURREAL_MEMORY_THRESHOLD` can still reject work because it considers broader tracked process memory, including storage caches.

A further operational boundary is that the temporary spill directory has no built-in disk quota. Failure to initialize that directory can disable spilling and leave the server without the query-memory limit normally enforced through it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/cli/db/environment-variables`

### 5.2 Parallel scan scheduling and OLTP protection

Version 4.0 uses a shared worker pool for eligible parallel scans, index builds and background index compaction.

The principal scheduling controls are:

| Variable                        | Function                                                      |
| ------------------------------- | ------------------------------------------------------------- |
| `SURREAL_MORSEL_PLACES`         | Maximum worker places a statement may occupy                  |
| `SURREAL_MORSEL_OLTP_RESERVE`   | Worker places reserved for shorter operations                 |
| `SURREAL_MORSEL_PLACE_QUANTUM`  | Work quantum before a worker yields to waiting work           |
| `SURREAL_MORSEL_SPLIT_PASS_PPM` | Selectivity threshold for splitting scans into parallel tasks |

The worker pool has one place for each CPU core beyond the task running the original statement.

`SURREAL_MORSEL_PLACES=0` disables additional pooled scan parallelism for statements, but does not disable ordinary asynchronous execution.

The default OLTP reservation is one place. On hosts with two or fewer cores, that reservation can leave no additional place available for shared background or scan work.

The selectivity threshold is another meaningful detail. Parallelizing a scan that spends almost all its time fetching and processing records may increase coordination and contention without improving performance. The engine therefore considers the fraction of records surviving predicate evaluation before splitting some scans.

The interaction with index building is important: long scans and background maintenance share capacity. More parallelism for one task does not necessarily increase aggregate database throughput.

This is a native mechanism for balancing interactive transaction latency with analytical and maintenance work, although its effectiveness remains workload dependent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



## 6. CBOR v2 and binary value preservation

New in 4.0

### 6.1 Why CBOR versioning matters

SurrealDB has a richer value model than ordinary JSON. Record IDs, `NONE`, sets, geometries, decimals, dates and other values carry native types.

The database's CBOR protocol uses tagged values to preserve this information across transport boundaries.

Version 4.0 introduces `CBOR v2` because the previous encoding cannot faithfully represent all newly supported numerical and temporal types.

The interfaces are:

| Transport | Original CBOR      | CBOR v2                 |
| --------- | ------------------ | ----------------------- |
| HTTP      | `application/cbor` | `application/cbor; v=2` |
| WebSocket | `cbor`             | `cbor.2`                |

A client must select the appropriate representation. A server supporting the types internally does not imply that an older client receives them without conversion or loss of type identity.

### 6.2 Sized numerical representations

CBOR v2 reserves the following tags:

| Tag | Native type |
| --- | ----------- |
| 180 | `i8`        |
| 181 | `u8`        |
| 182 | `i16`       |
| 183 | `u16`       |
| 184 | `i32`       |
| 185 | `u32`       |
| 186 | `u64`       |
| 187 | `f16`       |
| 188 | `f32`       |

Original CBOR transports these as ordinary integers or floats, discarding their precise width.

A `u64` larger than the maximum signed 64-bit integer is instead represented as a decimal in the original protocol.

CBOR v2 therefore matters when a field's exact numerical representation is part of the data contract, including typed Rust structures, binary interoperability and round-tripping through external systems.

### 6.3 Extended temporal, decimal and regex types

CBOR v2 additionally defines:

| Tag | Type         | Encoded content                                           |
| --- | ------------ | --------------------------------------------------------- |
| 100 | `date`       | Days relative to the Unix epoch                           |
| 189 | `time`       | Nanoseconds since midnight                                |
| 190 | `timestamp`  | Seconds and nanoseconds since the epoch, without timezone |
| 191 | `interval`   | Months, days and microseconds                             |
| 192 | `bigdecimal` | Decimal text                                              |
| 193 | `regex`      | SurrealQL regex pattern text                              |

Original CBOR maps several of these values to related but different types, such as `date` to a datetime at midnight, or `time` to a duration.

Those conversions can preserve an approximate interpretation while losing the original type.

Two qualifications matter:

First, a `bigdecimal` value fitting the narrower decimal representation may still use the existing decimal tag rather than tag 192.

Second, tag 193 contains a SurrealQL regex pattern. A receiving language cannot assume that its own regex engine interprets the pattern identically.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 6.4 Stricter decoding and legacy client compatibility

Version 4.0 rejects several encodings formerly accepted by the server, including:

- Textual datetime tag 0 in requests.
- Textual UUID tag 9.
- Textual duration tag 13.
- Duplicate map keys.
- Invalid or overflowing signed integer representations.
- Invalid `NONE` tag payloads.
- Certain unsupported or noncanonical tagged values.

Clients should send the supported compact datetime, binary UUID and compact duration encodings.

This creates two independent compatibility questions: whether the client supports the server's version and whether it encodes each value using representations the server accepts.

An older client may connect successfully at the transport layer but still fail when serializing a particular typed value.

The original HTTP JSON interface has a separate limitation: it can flatten values such as record IDs and decimals into strings, sets into arrays, and `NONE` and `NULL` into representations that do not preserve their original distinction.

Architectural significance: CBOR v2 is a native alternative to writing custom type-tagging envelopes around every database value. It improves fidelity only when the selected SDK and transport actually use it.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



Source: `https://surrealdb.com/docs/reference/rest-api/cbor-protocol`

## 7. Schema enforcement, migrations and structured domain contracts

### 7.1 `FLEXIBLE` is scoped to a field

A schemafull table need not require every nested object to have a completely fixed shape.

```
DEFINE TABLE document SCHEMAFULL;

DEFINE FIELD title ON document TYPE string;

DEFINE FIELD metadata ON document
    TYPE object FLEXIBLE;
```

The `metadata` field accepts additional object keys without making the entire `document` table schemaless.

`FLEXIBLE` also applies to object types nested in arrays, optional types, unions and object literals within the declared field.

It is a field-definition modifier, not an independent data type. The field's declared type must contain an object.

This provides a useful middle ground between strict domain attributes and variable source metadata.

The boundary remains explicit: flexibility applies to the declared field subtree, not to every other field or structural constraint in the table.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 7.2 `SQL TYPE`: constraints shared with SQL interfaces

New in 4.0

`SQL TYPE` associates a dialect-specific type with a native SurrealQL field:

```
DEFINE FIELD name ON customer
    TYPE string
    SQL TYPE POSTGRESQL 'varchar(20)';

DEFINE FIELD balance ON customer
    TYPE decimal
    SQL TYPE POSTGRESQL 'numeric(10,2)';
```

SurrealDB enforces the SQL-specific restrictions or normalization when records are written, including through native SurrealQL operations.

The supported dialect identifiers include `POSTGRESQL`, `MYSQL` and `DYNAMODB`.

For example, a numeric declaration can normalize a value to the specified scale, while a length-limited character declaration constrains accepted strings.

This is significant because the SQL listener is not merely allowed to announce a column type to a driver; the field retains compatibility-related validation in the canonical database schema.

The resulting behavior must still be distinguished from complete implementation of the source database's entire type system.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/field`

### 7.3 Field-expression dependencies and validation order

SurrealDB manages dependencies between value-producing field expressions.

A field's `VALUE`, `DEFAULT` or related computed expression can depend on other fields. The engine determines the appropriate evaluation order rather than relying exclusively on field-definition order.

Cyclic dependencies are rejected. Assertions evaluate against the relevant final field values.

The distinction among the principal field clauses is important:

| Clause           | Role                                                        |
| ---------------- | ----------------------------------------------------------- |
| `DEFAULT`        | Supply an absent value during creation                      |
| `DEFAULT ALWAYS` | Also supply a value during updates when the field is absent |
| `VALUE`          | Derive or normalize a stored value on record writes         |
| `ASSERT`         | Validate the value against a condition                      |
| `COMPUTED`       | Derive a field when reading rather than storing the result  |
| `READONLY`       | Restrict subsequent direct changes to the field             |

Adding a field definition does not automatically rewrite every existing record.

For example, a newly defined `VALUE` expression takes effect when an existing record is subsequently written, while a new ordinary `DEFAULT` does not automatically backfill previously stored records.

A computed field does not require stored-value backfilling because it is evaluated when read.

These distinctions determine whether a definition creates a current-data invariant, a future-write invariant, or a read-time projection.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 7.4 Restrictions on effects in schema clauses

Expanded enforcement in 4.0

SurrealDB 4.0 forbids arbitrary database writes from clauses intended to calculate, validate or authorize state.

This includes:

- `VALUE`
- `DEFAULT`
- `ASSERT`
- `COMPUTED`
- Table permission predicates

The restriction propagates through called functions and other execution paths; moving a write into a helper function does not make it permissible.

For example, a `COMPUTED` field cannot call a function that creates an audit record.

The rule distinguishes calculation and authorization from transactional mutation reactions.

`DEFINE EVENT` is the appropriate native mechanism when the desired operation is a write caused by another write.

Certain operations remain permitted in value-producing clauses, including sequence allocation, supported random functions and clock reads. These are explicitly recognized exceptions rather than permission for arbitrary database mutations.

An important compatibility issue is that schema definitions stored under earlier releases may contain now-disallowed effects. Such definitions can fail when evaluated under 4.0 even if their syntax remains valid.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 7.5 `MIGRATE EXISTING ROWS`: named transformations

New in 4.0

The original overview introduced this facility; its correctness guarantees and restrictions require further explanation.

The essential syntax is:

```
MIGRATE EXISTING ROWS AS migration_name
USING expression
FALLBACK value
```

A field migration has a persistent name and an expression transforming its previous `$value`.

The name provides idempotency. Reapplying the same migration through a repeated schema script does not transform values again; reusing the name with a different transformation is rejected.

A migration becomes effective when the schema statement commits. Subsequent reads can present converted values while a background task rewrites the stored records.

The transformation expression is deliberately constrained. It can invoke supported built-in functions, but cannot use arbitrary statements, user-defined functions, scripts, extensions or model invocations.

A fallback is required unless the engine can establish that the expression always succeeds and fits the destination type.

The destination value must also satisfy the new field definition. A fallback should therefore be regarded as an explicit data-conversion rule rather than a guarantee of semantic correctness.

### 7.6 Migration restrictions involving derived structures

A native field migration cannot freely modify a field whose current representation is used by every other database mechanism.

The documentation restricts migration of fields involved in structures such as:

- Secondary indexes.
- Materialized views and relevant view dependencies.
- `REFERENCE` tracking.
- Inline graph fields.
- `COMPUTED` fields.
- Special identity or relationship endpoint fields, including `id`, `in` and `out`.

The restriction prevents an apparently local type change from leaving an index or derived structure based on obsolete values.

This is a consequential boundary: native migration can remove substantial application-side rewrite machinery, but it does not automatically orchestrate every dependent schema structure.

### 7.7 Compatibility levels: 3 versus 4

Database compatibility level is a separate concept from the executable's version.

At compatibility level 4, schemafull tables containing records enforce stricter field-change rules.

A type change is permitted without migration when the new definition accepts every value allowed by the old definition. Widening an optional or union type can satisfy this condition.

Narrowing a type generally requires a named migration, even if the current stored records happen to satisfy the new type. The comparison concerns the definitions and their guarantees, not just a sample of present values.

A newly required field is likewise a narrowing change from previously absent values and may require a migration.

At compatibility level 3, older schema-change permissiveness is retained. Explicit migrations remain available, but the stronger automatic requirements are not enforced in the same way.

This explains why two databases running the same 4.0 executable can behave differently during schema evolution.

### 7.8 Removing undeclared fields

Version 4.0 also supports a named migration for removing fields no longer present in a schemafull definition:

```
ALTER TABLE product
    MIGRATE EXISTING ROWS AS remove_legacy_fields
    REMOVE UNDECLARED FIELDS;
```

This is valuable when a schemafull table has historically retained fields introduced by earlier definitions.

`INFO FOR TABLE` exposes migration history and which fields have a guarantee that stored records conform to their definitions.

Architectural significance: SurrealDB's database schema now includes mechanisms for identifying, transforming and tracking the stored representation of records, rather than leaving all data-shape evolution to application-managed migration code.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/field`

## 8. Native sequences and allocation semantics

Introduced in 3.0 · Supported in 4.0

`DEFINE SEQUENCE` creates a durable source of unique, monotonically increasing numerical values.

```
DEFINE SEQUENCE invoice_number
    BATCH 1000
    START 100
    TIMEOUT 5s;
```

Values are allocated through:

```
sequence::nextval("invoice_number");
```

### 8.1 Batched allocation

Each node reserves ranges of values rather than coordinating a distributed transaction for every individual allocation.

Sequence metadata is persisted in the underlying key-value store, and reserved batches are associated with the requesting node.

This reduces coordination overhead while ensuring that different nodes do not allocate overlapping ranges.

The tradeoff is that values allocated on different nodes need not correspond to one universal transaction-commit order.

### 8.2 Sequences are not transactional counters

Sequence allocation is not rolled back when a transaction fails.

A transaction may allocate a value, perform unrelated writes and then abort; the allocated number is still consumed.

Consequently, a sequence provides unique allocation but not a gap-free numbering of committed records.

A single record whose counter is incremented transactionally has different semantics and different contention characteristics.

This distinction is central for choosing between identity allocation, externally visible serial numbers and counters whose increments must participate in a business transaction.

### 8.3 Export and restart state

`INFO FOR DB STRUCTURE` exposes sequence metadata including `start` and `resume`.

`resume` identifies a safe value at which allocation can resume without reusing numbers previously reserved by nodes. It is not necessarily the next value a currently running node will return, since that node may still hold part of an earlier batch.

Exports preserve this safe resume position so imported sequences do not reuse numbers already associated with exported records.

The same sequence mechanism underlies supported PostgreSQL `serial`/`bigserial` and MySQL `AUTO_INCREMENT` mappings in the SQL interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/sequence`

## 9. Database programming and execution authority

### 9.1 `DEFINE FUNCTION` as a transactional procedure mechanism

SurrealDB user-defined functions are stored SurrealQL programs, not merely expressions evaluated as scalar functions.

They support typed arguments, return types, statements, conditional logic, loops, recursive calls, reads and writes.

A function can encapsulate a multi-record operation:

```
DEFINE FUNCTION fn::transfer(
    $source: record<account>,
    $destination: record<account>,
    $amount: decimal
) -> bool {
    IF $source.balance < $amount {
        THROW "Insufficient balance";
    };

    UPDATE $source SET balance -= $amount;
    UPDATE $destination SET balance += $amount;

    RETURN true;
};
```

This is illustrative code, not an executed example.

When called independently, a function body executes as one transaction. If a statement fails or `THROW` is reached, the function's writes roll back.

When called inside an explicit transaction, the function participates in that transaction instead of creating a separately committed nested transaction.

`BEGIN`, `COMMIT` and `CANCEL` inside a function body do not create nested transaction boundaries or independently preserve changes.

An early `RETURN` is normal completion, not an error, and therefore does not undo prior writes.

These semantics make functions suitable for reusable multi-record operations whose state changes must succeed or fail together.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/function`

### 9.2 Functions execute with caller permissions

A stored function's `PERMISSIONS` clause controls whether it may be invoked.

It does not automatically grant the function extra authority over its underlying tables.

A record user calling a function can read and write only what their effective database permissions allow.

This contrasts with database events and the default security context of database-defined HTTP APIs.

The distinction matters when a stored function appears to provide a privileged data-access abstraction. Merely placing a query inside `fn::` does not bypass row-level security.

### 9.3 Anonymous closures and higher-order operations

SurrealQL also supports typed closures that can be stored in values and passed as function arguments.

These can compose with array operations such as mapping and filtering, as well as custom middleware chains.

Closures offer a different abstraction from stored functions: they are first-class values useful for local computation and higher-order behavior, whereas `DEFINE FUNCTION` creates a named reusable operation in the database schema.

Both remain subject to execution-context and side-effect restrictions.

Source: `https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/closures`

### 9.4 Default `DEFINE API` security is elevated

A database-defined HTTP API has semantics that are easy to misunderstand.

By default, an endpoint uses:

- `PERMISSIONS FULL`, permitting invocation by clients that can reach the endpoint, including unauthenticated clients.
- `SECURITY DEFINER`, allowing its body to execute without ordinary table and field permission checks.

These are separate controls.

Endpoint permissions decide who can call the operation. Execution security decides what data the operation can access.

The default combination is powerful but exposes a significant risk when a handler can read or modify sensitive records.

Version 4.0 adds `SECURITY INVOKER`:

```
DEFINE API "/my-records"
    FOR get
    PERMISSIONS WHERE $auth != NONE
    THEN {
        {
            status: 200,
            body: (SELECT * FROM private_record)
        }
    }
    SECURITY INVOKER;
```

With this mode, handlers and middleware execute under the caller's permissions and cannot exceed the authority of the endpoint's definer.

The query result is still governed by the relevant table and field permissions.

In contrast, a `SECURITY DEFINER` endpoint can deliberately expose a controlled privileged operation even when the caller could not issue its underlying queries directly.

### 9.5 API handler atomicity in 4.0

Version 4.0 strengthens failure handling for database-defined APIs.

If a handler fails, database writes performed by that handler, associated index changes and synchronous event effects are rolled back together.

This corrects earlier behavior under which a handler could return an error while retaining some intermediate mutations.

The rollback applies to the handler's transactional database work. It does not imply rollback of an external HTTP request or another irreversible operation performed outside that transaction.

The distinction among native programming mechanisms is therefore:

| Mechanism                            | Execution authority               | State-change behavior                                                      |
| ------------------------------------ | --------------------------------- | -------------------------------------------------------------------------- |
| Stored `fn::` function               | Caller                            | Transactional procedure                                                    |
| `DEFINE EVENT`                       | Database-managed elevated context | Synchronous transactional effect or separately scheduled asynchronous work |
| `DEFINE API` with `SECURITY DEFINER` | Elevated handler context          | Handler writes roll back on failure                                        |
| `DEFINE API` with `SECURITY INVOKER` | Caller, capped by definer         | Same rollback behavior, with caller's data permissions                     |
| Schema expression                    | Context-specific                  | Arbitrary writes forbidden                                                 |

Together these provide several ways to move computation near canonical state without treating every callable database mechanism as having the same authority or lifecycle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/api`

## 10. SQL compatibility: semantic boundaries beyond wire-protocol support

### 10.1 Shared data does not imply identical SQL behavior

SurrealDB's PostgreSQL and MySQL listeners execute defined subsets of their respective SQL dialects against the same underlying records as SurrealQL.

Their capabilities are not identical.

| Capability                      | PostgreSQL dialect                    | MySQL dialect                               |
| ------------------------------- | ------------------------------------- | ------------------------------------------- |
| Foreign keys                    | Supported through record-link mapping | Not supported                               |
| `SERIALIZABLE`                  | Supported on eligible storage engines | Not established by the compatibility matrix |
| Recursive CTEs                  | Refused                               | Refused                                     |
| Window functions                | Supported subset                      | Supported subset                            |
| `LATERAL`                       | Restricted support                    | Refused                                     |
| Schema-changing `ALTER TABLE`   | Supported subset                      | Not supported                               |
| Savepoints                      | Supported                             | Supported                                   |
| SQL DDL within transactions     | Transactional                         | MySQL-style implicit commit                 |
| `CREATE VIEW`                   | Restricted single-table view          | Not documented in the matrix                |
| Arbitrary PostgreSQL extensions | Not supported                         | Not applicable                              |

The official compatibility matrix distinguishes supported, partially supported, differing, refused and undocumented behavior.

This distinction is particularly important for tools that generate SQL: successful execution of ordinary CRUD statements says relatively little about compatibility with a framework's migration system, introspection queries or advanced relational expressions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rest-api/sql-compatibility`

### 10.2 Relational keys map to SurrealDB record identity

SQL clients see a SurrealDB record's key as a primary-key value.

For example, a SurrealDB record ID:

```
person:123
```

is represented through a PostgreSQL or MySQL primary-key column as the key component `123`.

A PostgreSQL foreign-key definition can map to a typed SurrealDB record link.

This is useful because SQL tools can operate over native record identity without requiring the data to be duplicated into a separate relational store.

It also creates a conversion boundary. The SQL interface exposes values through column-oriented SQL types, while SurrealQL preserves the fuller record-ID representation and richer value model.

Primary-key semantics, type inference, nullable values and declared SQL types therefore determine the fidelity of a round trip.

### 10.3 PostgreSQL catalog compatibility is selective

SurrealDB provides emulated `pg_catalog` and `information_schema` relations.

These expose sufficient metadata for certain tools to discover tables, columns, indexes and constraints.

They do not constitute a complete PostgreSQL system catalog.

Important documented differences include:

- `pg_dump` is not supported.
- Some `psql` introspection commands work, while other detailed table descriptions remain incomplete.
- `CREATE SCHEMA` can return a successful command tag without creating a distinct PostgreSQL-style schema.
- Some generated DDL constructs are accepted but have no effect.
- Native PostgreSQL extensions cannot generally be installed.

For example, `CREATE EXTENSION vector` is accepted as a compatibility statement, but does not install PostgreSQL's pgvector library. The relevant vector functionality is provided by SurrealDB itself.

A SQL client or ORM that assumes PostgreSQL catalog behavior may therefore fail before executing any application query.

### 10.4 SQL view semantics are narrower than SurrealQL views

The PostgreSQL dialect accepts a restricted `CREATE VIEW` over one table, with projections and a filter.

It does not accept arbitrary join-based, grouped or windowed view definitions, and `CREATE MATERIALIZED VIEW` does not parse.

This is especially important because the native SurrealQL materialized-view mechanism described in Section 2 is more expressive in several respects than the exposed PostgreSQL `CREATE VIEW` syntax.

Having a native capability does not mean every compatibility dialect exposes that capability through its own standard syntax.

### 10.5 SQL functions and writes have non-obvious differences

The PostgreSQL and MySQL dialects expose their supported SQL functions, not the entire SurrealQL function namespace.

Native functions such as `fn::...` and `http::get` are not automatically callable through those dialects.

Other examples of semantic differences include:

- PostgreSQL-style array comparison operators treating arrays as sets in supported forms.
- A grouped projection of an ungrouped field returning an array of values in cases where PostgreSQL would reject the query.
- Restricted use of `LIKE` and concatenation operators in PostgreSQL-dialect write statements.
- Limited lateral joins and scalar-subquery forms.
- `CREATE TABLE` and SQL DDL behaviors that differ between the two compatibility dialects.
- MySQL-like implicit commit around DDL through the MySQL listener.

These are not merely unsupported grammar features. Some statements execute but produce results or transactional behavior different from the corresponding database.

### 10.6 PostgreSQL vector compatibility is a translation layer

The supported pgvector syntax maps to SurrealDB native vectors and HNSW indexing.

A PostgreSQL declaration such as `vector(3)` corresponds to a fixed-length native numeric array.

The documented interface requires explicit dimensions and limits vectors to 16,000 dimensions through this compatibility path.

An eligible `CREATE INDEX ... USING hnsw` creates a SurrealDB HNSW index. It does not create an index managed by PostgreSQL's pgvector extension.

This arrangement allows both the PostgreSQL dialect and SurrealQL to query the same vector data, but the available index algorithms, tuning options and execution plans remain those of SurrealDB.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 10.7 Known 4.0 documentation inconsistency

The official PostgreSQL protocol page contains contradictory statements about `UPDATE ... FROM` and `DELETE ... USING`.

Its initial feature table lists these constructs as not yet supported, while later sections document their execution and specific restrictions. The separate SQL compatibility matrix also lists them as supported.

The later detailed descriptions state that these joined writes support selected forms but reject certain `RETURNING` projections and correlated constructs.

The documentation therefore supports identifying an intended implemented subset, but does not justify asserting unrestricted PostgreSQL compatibility. Exact behavior for a consequential query shape should be verified against the tagged 4.0 implementation or integration tests.

This is a documentation inconsistency, not evidence that the entire PostgreSQL dialect is experimental.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



## 11. GraphQL schema generation and query contracts

### 11.1 Native schema generation

SurrealDB provides a GraphQL endpoint with a database-generated schema.

```
DEFINE CONFIG GRAPHQL
    TABLES AUTO
    FUNCTIONS AUTO;
```

The schema can expose queries, filters, aggregates, mutations, function calls and connection-based pagination.

Generated GraphQL interfaces operate against the underlying database rather than requiring a separately implemented resolver for every ordinary table operation.

Version-specific naming conventions are relevant. Generated operations include record-level queries, pluralized table queries, aggregate queries and bulk mutations.

### 11.2 Schema inclusion and exclusion

`DEFINE CONFIG GRAPHQL` controls which tables and functions enter the exposed GraphQL schema.

For example:

```
DEFINE CONFIG GRAPHQL
    TABLES INCLUDE person, company
    FUNCTIONS NONE
    DEPTH 8
    COMPLEXITY 500
    INTROSPECTION NONE;
```

The configuration supports include/exclude rules, depth limits, complexity budgets and disabling introspection.

The default documented depth limit is 16 and the default complexity limit is 1,000. Queries exceeding these bounds are rejected before execution.

GraphQL name generation can produce collisions when two table names resolve to the same operation name. Table aliases can resolve such collisions.

### 11.3 Generated read/write surfaces reflect field semantics

Native table and field definitions influence GraphQL schema generation.

In particular:

- `GRAPHQL_ALIAS` changes exposed names.
- `GRAPHQL_DEPRECATED` marks generated fields or operations as deprecated.
- Materialized view tables expose query operations but not ordinary record-creation or mutation operations.
- `READONLY` and `COMPUTED` fields are excluded from mutation input types.
- Connection queries support cursor-based pagination in addition to offset-style access.

GraphQL's schema therefore reflects some of the database's semantic constraints without an entirely separate schema declaration.

The boundary remains that GraphQL is a generated API interface, not a general replacement for SurrealQL or ISO GQL graph-pattern querying.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



Sources:

`https://surrealdb.com/docs/learn/querying/graphql/overview`

`https://surrealdb.com/docs/reference/query-language/statements/define/config`

## 12. Rust SDK, RPC streaming and session lifecycle

### 12.1 Rust SDK features determine the executable engine boundary

The `surrealdb` Rust crate uses Cargo features to select which local and remote engines are included.

The published 4.0.0 crate exposes 17 feature flags, with three enabled by default: `protocol-ws`, `regex-unicode` and `rustls`.

The most consequential optional features are:

| Feature                | Capability                                                                |
| ---------------------- | ------------------------------------------------------------------------- |
| `kv-mem`               | Embed the in-memory database engine                                       |
| `kv-surrealkv`         | Embed persistent SurrealKV storage                                        |
| `protocol-http`        | Connect over HTTP                                                         |
| `protocol-grpc`        | Connect over gRPC                                                         |
| `protocol-ws`          | Connect over WebSocket                                                    |
| `scripting`            | Enable the embedded scripting functionality supported by the local engine |
| `parse`                | Expose parsing functionality                                              |
| `unstable`             | Access APIs outside the normal SDK stability guarantee                    |
| `allocation-tracking`  | Include local-engine allocation tracking                                  |
| `native-tls`, `rustls` | Select supported TLS integrations                                         |

The difference between embedding and remote access is significant. Enabling a local KV backend compiles substantially more of the database engine into the application than connecting to a standalone server.

The runtime-selectable `engine::any` interface can choose an enabled engine from an endpoint URL. A URL selecting a backend omitted from the Cargo feature set fails at connection time.

Version 4.0 removes the `kv-rocksdb` and `kv-indxdb` features. The corresponding legacy URL schemes can still be recognized as deprecated spellings of SurrealKV paths, with the conversion behavior described in the migration documentation.

Source verification:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Sources:

`https://docs.rs/crate/surrealdb/4.0.0/features`

`https://surrealdb.com/docs/reference/rust/methods/connect`

### 12.2 Streaming query results

A conventional SDK query can buffer its result set before returning it to the caller.

For large results, this requires retaining data that may not be needed simultaneously by the consuming application.

The Rust SDK provides `.stream_items()`, which yields result items incrementally as the database produces them.

Its behavior depends on the execution engine:

| Connection type            | Incremental query-result delivery |
| -------------------------- | --------------------------------- |
| Embedded SurrealMX         | Supported                         |
| Embedded SurrealKV         | Supported                         |
| Remote gRPC                | Supported                         |
| Remote WebSocket SDK query | Results buffered and replayed     |
| Remote HTTP SDK query      | Results buffered and replayed     |

The distinction matters for memory use and latency to the first result. A streaming-looking API does not necessarily eliminate server- or transport-side result buffering.

The gRPC implementation also has explicit message-size limits. The server advertises a limit, and the client can configure its own. Large results may be divided across messages, but an individual record exceeding the server's message limit still fails.

The `.stream_items()` interface and the gRPC transport can therefore reduce large-result materialization without implying unlimited result size or unconstrained memory consumption.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 12.3 WebSocket RPC `query_stream` is a separate protocol facility

SurrealDB's RPC protocol also provides a `query_stream` method over WebSocket.

It returns a sequence of framed messages associated with the request ID.

The stream includes an explicit completion frame. Multiple streams require distinct request IDs, and reusing the ID of an active stream is rejected.

A streaming query can execute inside an interactive transaction. The transaction cannot safely commit while an associated write-capable stream remains active; premature commit can fail and cancel the transaction.

The Rust and JavaScript SDKs do not currently use this WebSocket RPC method for their ordinary streaming query APIs.

Thus, wire-protocol streaming support does not imply that a particular SDK exposes the same streaming behavior over every transport.

This capability is relevant when implementing a custom RPC client or inspecting the protocol independently of the higher-level SDKs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rest-api/rpc-protocol`

### 12.4 Connection ownership and explicit transaction completion

The Rust SDK's `Surreal` handles can share a connection through cloning.

The underlying connection remains alive while a handle referencing it remains alive. The documented connection interface does not expose a general `close()` method; connection lifetime is managed through ownership and dropping handles.

For interactive transactions, explicit `commit()` or `cancel()` remains important because a transaction can hold a snapshot, pending writes and conflict-tracking state independently of an individual query result.

A query timeout is likewise distinct from a transaction-lifetime limit. A transaction may span several requests or operations, each subject to its own timeout.

This is especially relevant when a query is streamed incrementally: completion of the stream and completion of the surrounding transaction are separate lifecycle events.

### 12.5 Durable HTTP RPC sessions

Default behavior changed in 4.0

SurrealDB 4.0 persists attached HTTP RPC sessions in the datastore by default.

An attached session can survive a server restart and, where nodes share the appropriate datastore, resume through another node.

This changes an important assumption about session state. It is no longer necessarily tied to the lifetime of the particular server process that first accepted the session.

Relevant configuration includes:

| Control                               | Documented default           |
| ------------------------------------- | ---------------------------- |
| `SURREAL_DURABLE_SESSION_TTL`         | 24 hours                     |
| `SURREAL_DURABLE_SESSION_GC_INTERVAL` | 1 hour                       |
| `SURREAL_DURABLE_SESSIONS`            | Deprecated; no effect in 4.0 |

Session expiration is based on inactivity, with the documented expiration interval extending to between one and one-and-a-half times the configured TTL.

The persisted session includes authentication state and is stored unencrypted in the datastore. That is an important operational distinction for controlling access to persisted database state.

WebSocket sessions do not acquire the same durability guarantees.

Source: `https://surrealdb.com/docs/reference/cli/db/environment-variables`

### 12.6 Experimental live-query routing

The default live-query implementation uses the `inline` engine.

A separately configurable `router` engine is documented as an experimental rollout:

```
SURREAL_LIVE_QUERY_ENGINE=router
SURREAL_LIVE_QUERY_RETENTION=1h
```

The router separates some subscription processing costs from the number of subscribers attached to writes.

It also maintains retained notification history for subscriber resumption, controlled by the retention setting.

This should not be conflated with ordinary live-query semantics or durable changefeeds. Its experimental status and independent retention behavior matter when discussing replay and reconnection guarantees.

Source: `https://surrealdb.com/docs/reference/cli/db/environment-variables`

## 13. Migration-sensitive data and query semantics

The original reference describes the major datastore upgrade. Additional changes affect applications even when the database files convert successfully.

These are especially important because many involve the meaning of an existing expression or index, not merely whether a feature is available.

### 13.1 Float–decimal equality becomes exact

Version 4.0 changes comparison semantics between floating-point and decimal values.

For example:

```
[
    0.1f = 0.1dec,
    0.5f = 0.5dec,
    1f = 1dec
];
```

The result is:

```
[false, true, true]
```

The reason is that `0.1` has no exact binary floating-point representation, whereas `0.5` and integer values can be represented exactly in both forms.

This can change the results of predicates that compare persisted decimal fields against floating-point literals or bound values.

The distinction is not confined to query syntax. Rust `Number` and `Value` comparisons follow the revised semantics.

### 13.2 Decimal literal precision

A `dec` literal with significant fractional precision beyond 28 places now produces a parse error rather than silently rounding to fit the supported decimal representation.

A conversion from a string through a cast can have different rounding behavior from writing an excessively precise literal.

This distinction matters for generated queries, configuration values and serialization formats that reconstruct decimal numbers as SurrealQL text.

The introduction of `bigdecimal` adds a representation for larger precision requirements, but it does not change the parsing rules of every existing `dec` literal.

### 13.3 Vector arithmetic changes at floating-point precision

Version 4.0 changes the reduction order used by several vector operations over floating-point vectors of eight or more elements.

Affected operations include distance functions, cosine and Jaccard similarity, dot products and exact KNN calculations.

Numerical results can differ in their final digits because floating-point addition is not associative.

When intermediate positive and negative contributions substantially cancel, the relative difference can be larger and may even change the sign of a result close to zero.

Nearest-neighbor order can change for records with nearly identical distances.

This behavior is separate from approximate ANN recall. Even exact calculations can change order when their floating-point reduction algorithm changes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 13.4 Arrays and sets have different traversal/index behavior

Several collection operations change in 4.0.

A range extending past the end of an array now stops at the available elements in cases where the previous behavior returned `NONE`.

Nested arrays supplied to `FROM` are flattened into their elements rather than always being treated as nested values.

Paths such as `s.*` now operate over sets, collecting the results as sets.

The distinction between arrays and sets also affects indexing: a set-valued field indexed with `.*` is treated as a whole set in 4.0, while array expansion can generate separate index entries for elements.

These changes are consequential for expression-driven code, collection normalization and queries relying on previously observed shape or ordering behavior.

### 13.5 `NONE`, `NULL` and unique-index semantics

A standard unique index does not treat `NONE` or `NULL` values as duplicates.

The 4.0 `UNIQUE NULLS NOT DISTINCT` option changes the treatment of explicit `NULL` while continuing to leave absent values (`NONE`) outside the uniqueness comparison.

An empty array can be represented as the `NULL` index value and may therefore collide with an explicit null under the stricter option.

This is a physical-index consequence of the logical distinction between absence and explicit null.

It also affects the conditional-uniqueness technique discussed in Section 3: returning `NONE` from a normalized field has different implications from storing `NULL`.

### 13.6 Index rebuilds: correctness versus performance

Upgrading a datastore to the 4.0 record format does not automatically rebuild all its indexes.

The migration guide distinguishes two categories.

| Index condition                                                             | Rebuild significance                        |
| --------------------------------------------------------------------------- | ------------------------------------------- |
| HNSW using Jaccard or Pearson ordering from the previous version            | Required for correct nearest-first behavior |
| HNSW/DiskANN affected by earlier duplicate-vector bugs                      | Required to recover missing KNN results     |
| HNSW/DiskANN used with selective structured filters                         | Performance improvement                     |
| DiskANN eligible for new 8-bit quantized traversal                          | Performance and physical-layout improvement |
| Earlier B-tree indexes missing information needed for newer optimizer paths | Improved query-plan selection               |
| Full-text indexes affected by documented earlier bugs                       | May require rebuild for correctness         |

For DiskANN, the migration guide notes that a rebuilt `F32` or `F16` index stores an additional one-byte-per-dimension code for faster search traversal, increasing index size.

The distinction is important because an index that exists, loads and produces results is not necessarily operating with the corrected semantics or improved physical representation of the new release.

The migration guide should be treated as the source of truth for which specific pre-upgrade indexes require rebuilding.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 13.7 Record-format compatibility is storage-engine dependent

Version 4.0 introduces a new stored record format, but the timing of switching to that format depends on the engine.

SurrealKV and single-process storage: The first eligible 4.0 startup switches the datastore to the newer format.

TiKV: The separate TiKV build can require an explicitly coordinated feature-level switch after the deployment's nodes have been upgraded.

The relevant setting is:

```
SURREAL_FEATURE_LEVEL=4.0
```

This exists because several processes may concurrently access shared TiKV storage while running different executable versions.

The format switch is guarded against running while older nodes remain active.

A 4.0 datastore may temporarily contain records encoded in both older and newer formats, with the 4.0 engine able to read them. A change in the datastore's compatibility level therefore does not imply that every record was rewritten synchronously.

This differs from a conversion of a RocksDB directory to SurrealKV, which also requires physical storage migration and sufficient temporary disk space.

### 13.8 Client and authentication changes

Additional upgrade-sensitive behavior includes:

SDK version negotiation. Some older clients refuse a 4.0 server even if their query syntax would otherwise remain valid.

CBOR decoding. Some older SDK encoders emit the legacy tags now refused by 4.0.

Bearer tokens. Revocation behavior is strengthened by validating a token against its grant; tokens issued before the upgrade may need to be replaced.

Authentication identity. `$auth.id` can return the authenticated record ID directly from session state, independently of whether that record is visible or still exists. It should not be treated as a proof of current record existence.

Schema definitions. Repeated conflicting clauses in a `DEFINE TABLE` statement now produce errors rather than silently retaining the last occurrence.

These are independent of file-format migration and can change application behavior even after a database successfully starts under 4.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x`

## 14. Additional capabilities and limits worth retaining in the reference

Several more narrowly scoped features deserve coverage because they affect the composition of capabilities described above.

### 14.1 Native capability restrictions apply across interfaces

SurrealDB exposes controls for functions, network access, scripting, RPC methods, HTTP routes, MCP tools and arbitrary query execution.

The same general capability model can restrict operations regardless of whether they are reached through a direct query, stored function, custom API or remote protocol.

For example, enabling a PostgreSQL listener does not necessarily allow it to bypass a server-wide restriction on arbitrary queries.

Likewise, a stored function referencing a disallowed built-in operation is not automatically permitted merely because the function itself exists.

The relevant boundary is the combination of authentication, authorization and runtime capabilities.

Source: `https://surrealdb.com/docs/reference/cli/db/environment-variables`

### 14.2 Bulk imports are not equivalent to ordinary writes

Import processing may defer or bypass several mechanisms associated with the normal write path, including selected checks, events, materialized-view updates and changefeed maintenance.

This is necessary in some cases, such as restoring relationship records before their endpoint records.

However, it means that a logical reconstruction of a database cannot always be treated as a series of ordinary transactions executed in another order.

Index rebuilding, view reconstruction, referential integrity and security definitions can require explicit consideration during restoration.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

### 14.3 Schema comments are available for structural documentation

`COMMENT` can be attached to schema definitions, including tables, fields, indexes and other database objects.

Those comments are exposed through schema introspection and can be used by agent-facing tools.

This is a minor feature technically, but it permits structural metadata such as record-ID conventions, relationship semantics and constraints to live with the definitions they describe.

It does not replace formal constraints or typed schema declarations. Rather, it complements them with human-readable and agent-readable context.

### 14.4 Experimental and development functionality should remain separate

Some current documentation surfaces describe experimental facilities or contain examples written against earlier releases.

Two cases are particularly important.

Surrealism: Version 4.0 changes the extension ABI and capability model, but the extension facility remains subject to its documented experimental enablement and runtime restrictions.

File storage and ISO GQL: Their feature status must be evaluated against the 4.0 documentation rather than older examples requiring experimental flags. In particular, the `gql` experimental capability name is not accepted under the newer configuration.

The official PostgreSQL protocol pages also contain the joined-write documentation inconsistency identified earlier.

The November 3, 2026 release announcement identifies SurrealDB 4 as generally available, while some version-listing and migration search material still contains beta-era descriptions. These should be treated as publication inconsistencies rather than silently resolved through assumptions about unrelated features.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+2



## 15. Additional primary documentation references

The following resources cover the omitted mechanisms and their version-specific behavior. They are supplementary to the source map in the original overview.

| Capability                                                                         | Direct primary source                                                             |
| ---------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Graph relation representations, adjacency caches, views and enforced relationships | `https://surrealdb.com/docs/reference/query-language/statements/define/table`     |
| Inline edge properties, flexible fields and schema migrations                      | `https://surrealdb.com/docs/reference/query-language/statements/define/field`     |
| Count indexes, expression indexes and index lifecycle                              | `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`   |
| Bitmap operators and graph/vector access-plan inspection                           | `https://surrealdb.com/docs/reference/query-language/clauses/explain`             |
| Executed plans and operator metrics                                                | `https://surrealdb.com/docs/reference/query-language/statements/explain`          |
| Query scheduling, memory, RPC sessions and runtime capabilities                    | `https://surrealdb.com/docs/reference/cli/db/environment-variables`               |
| CBOR v2 type encodings                                                             | `https://surrealdb.com/docs/reference/rest-api/cbor-protocol`                     |
| Database-defined functions                                                         | `https://surrealdb.com/docs/reference/query-language/statements/define/function`  |
| HTTP API security and transaction semantics                                        | `https://surrealdb.com/docs/reference/query-language/statements/define/api`       |
| Sequence allocation                                                                | `https://surrealdb.com/docs/reference/query-language/statements/define/sequence`  |
| PostgreSQL/MySQL compatibility matrix                                              | `https://surrealdb.com/docs/reference/rest-api/sql-compatibility`                 |
| GraphQL schema configuration                                                       | `https://surrealdb.com/docs/reference/query-language/statements/define/config`    |
| Rust SDK 4.0 features                                                              | `https://docs.rs/crate/surrealdb/4.0.0/features`                                  |
| Rust connection engines and gRPC behavior                                          | `https://surrealdb.com/docs/reference/rust/methods/connect`                       |
| RPC protocol and streaming                                                         | `https://surrealdb.com/docs/reference/rest-api/rpc-protocol`                      |
| 3.x-to-4.0 migration and compatibility                                             | `https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x` |

The additions clarify an important aspect of SurrealDB's architecture: the database exposes not only multiple data models, but several native ways to physically represent, incrementally maintain, query and validate the same logical state.

Those distinctions—especially lightweight and cached graph representations, incremental views, index-set fusion, typed protocol encoding, schema migrations and transactional procedure semantics—are the principal functionality missing from the original overview.