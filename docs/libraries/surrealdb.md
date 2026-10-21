# SurrealDB 3.3.4 — Comprehensive Rust Library and Tooling Reference

Research date: October 21, 2026

Scope: SurrealDB database engine, Rust SDK, SurrealQL, graph-query capabilities, RocksDB, SurrealKV, SurrealMX, SurrealKit 1.0 beta, schema management, storage and indexing, transactions, embedded execution, integrations, and operational tooling.

## 1. Executive orientation and version scope

SurrealDB is a Rust-native, multimodel database whose central architectural distinction is that document, relational, graph, key-value, geospatial, full-text, and vector-oriented operations are available through a shared database engine and query language.

Its capabilities are not simply an aggregation of independently operated databases. SurrealDB provides one record and relationship model, one transactional engine, and a common SurrealQL execution environment. This allows operations that would otherwise involve separate document, relational, graph, and vector stores to be expressed within a single database.

The Rust implementation is especially significant because SurrealDB can operate either as a standalone database service or as an embedded database inside a Rust application.

### Verified versions

| Component             | Version covered      | Status                                |
| --------------------- | -------------------- | ------------------------------------- |
| SurrealDB database    | 3.3.4                | Current stable, October 13, 2026      |
| Rust SDK, `surrealdb` | 3.3.4                | Current stable                        |
| SurrealKit            | 1.0.0-beta.6         | Prerelease, October 2, 2026           |
| SurrealKV             | 1.0                  | Released storage-engine library       |
| SurrealMX             | 1.0                  | Released in-memory engine             |
| RocksDB integration   | SurrealDB 3.3.x      | Supported persistent backend          |
| SurrealDB 4           | Future major release | Not treated as released functionality |

Important version distinction: The transition from `0.7` to `1.0.0-beta` belongs to SurrealKit, the schema management and migration toolkit, rather than the main Rust database SDK. I have therefore used SurrealKit 1.0.0-beta.6, not 0.7, throughout this reference, while covering the current `surrealdb` Rust SDK separately.

SurrealKit's published migration documentation confirms that beta.6 includes additional capabilities beyond the earlier beta.1 documentation, notably frozen rollout SQL, catch-up migrations, safer sequence handling, and SurrealDB 3.3 compatibility.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+3



Primary version references:

- SurrealDB release history: [https://surrealdb.com/releases/3.3](https://surrealdb.com/releases/3.3?utm_source=chatgpt.com)
- SurrealDB Rust SDK: [https://surrealdb.com/docs/reference/rust](https://surrealdb.com/docs/reference/rust?utm_source=chatgpt.com)
- SurrealKit beta.6 release: [https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-beta.6](https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-beta.6?utm_source=chatgpt.com)
- SurrealKit beta migration details: [https://github.com/surrealdb/surrealkit/blob/main/MIGRATING.md](https://github.com/surrealdb/surrealkit/blob/main/MIGRATING.md?utm_source=chatgpt.com)

## 2. Architectural model: a database engine, not simply a client library

SurrealDB has three principal layers:

| Layer                        | Responsibility                                                                       | Principal components                               |
| ---------------------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------- |
| Query and database semantics | Parsing, planning, transactions, authorization, indexes, record and graph operations | `surrealdb-core`, SurrealQL and query execution    |
| Persistence and concurrency  | Key-value operations, versioning, durability, storage layout                         | RocksDB, SurrealKV, SurrealMX, distributed storage |
| Integration and tooling      | Embedded execution, remote access, typed values, migrations, administration          | Rust SDK, protocols, SurrealKit, CLI, Studio       |

The database's logical model is largely independent of the selected storage engine. The engine translates higher-level records, index entries and relationships into ordered key-value structures. For example, a graph traversal is ultimately implemented through adjacency lookups, and a time-oriented read can use ordered keys and range scans rather than necessarily examining every record.

That separation is important: RocksDB is not a separate graph or query database that an application integrates with SurrealDB. It is one of the engines SurrealDB uses to persist its own internal data structures. Applications generally interact with SurrealDB rather than calling RocksDB directly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



### Execution topologies

- Embedded Rust: The engine runs inside the application's process, avoiding the database network round trip. This is useful for local applications, test harnesses, offline processing and tools with tightly coupled state.
- Single-node database server: The same engine exposes network protocols to multiple clients, with durable storage supplied by RocksDB or SurrealKV.
- Distributed deployment: Multiple database compute nodes use a distributed transactional storage layer. Production clustering and high availability have edition and deployment-model boundaries; deploying multiple independent RocksDB-backed processes is not equivalent to obtaining a coherent distributed database.
- Browser or edge: Supported embedded backends include browser-oriented IndexedDB, subject to the target platform and feature configuration.

SurrealDB is also organized into namespaces, databases and tables, with schema definitions, permissions and access methods at appropriate levels of that hierarchy.

Primary sources:

`https://surrealdb.com/docs/learn/data-models/architecture`

`https://surrealdb.com/docs/manage/self-hosted/deployment-models`

`https://surrealdb.com/docs/reference/rust/embedding`

## 3. Unified records, documents, relationships and schema

### 3.1 Record identity and structured values

A SurrealDB record has an identifier consisting of a table name and record key, conventionally represented as `table:id`. The identity is a native database value, not merely a string convention.

Keys can support deterministic identities as well as generated identities. Direct record-address operations can eliminate the need for a separate lookup index when the application's natural identifier can be represented appropriately as a record key.

Records support nested objects and arrays, optional values, typed references to other records, and database-native types including decimals, datetimes, durations, UUIDs, bytes, sets and geometries.

Two significant semantic distinctions are:

- `NONE` versus `NULL`: Absence or nonexistence of a value is distinct from an explicitly null value. Schema definitions and Rust deserialization must preserve that distinction where it matters.
- Record IDs versus strings: A field containing a `RecordId` is not interchangeable with its formatted textual representation. This matters especially with the 3.x Rust SDK's stricter typed conversion.

Source: `https://surrealdb.com/docs/reference/query-language/language-primitives/data-types`

Source: `https://surrealdb.com/docs/reference/rust/concepts/working-with-types`

### 3.2 Optional schema with meaningful constraints

SurrealDB supports `SCHEMALESS` and `SCHEMAFULL` tables. A table can be developed with flexible document structures or constrained using field definitions, types, assertions, defaults, and permissions.

Important schema mechanisms include `DEFINE TABLE`, `DEFINE FIELD`, `ASSERT`, `DEFAULT`, `VALUE`, `COMPUTED`, `READONLY`, and typed record references.

The distinction between computed forms is particularly useful: write-time normalization and derived values are different from values calculated when read. Schema-full processing and default behavior should not be assumed identical to PostgreSQL constraints or generated columns.

The database also supports sequences, named parameters, functions, analyzers, access methods, and indexes as schema-managed objects.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/field`

### 3.3 Three forms of relationship

These are not interchangeable modeling constructs.

| Mechanism       | Representation                         | Appropriate when                                                              |
| --------------- | -------------------------------------- | ----------------------------------------------------------------------------- |
| Embedded object | Nested data in a record                | The nested value belongs to its parent and does not need independent identity |
| Record link     | A field containing another record's ID | A direct association is sufficient                                            |
| Graph relation  | A separate edge in a relation table    | The connection has its own meaning, needs traversal, or carries attributes    |

A record link avoids creating an edge record. With a `REFERENCE` declaration, SurrealDB also tracks incoming references, enabling reverse navigation through `<~`.

A graph relation has an `in` endpoint and an `out` endpoint. A conventional relation can store properties of the connection itself.

```
DEFINE TABLE works_at
    TYPE RELATION IN person OUT company ENFORCED;

RELATE person:alice->works_at->company:acme
    SET since = d'2024-01-15', role = "Engineer";

SELECT
    name,
    ->works_at->company.name AS employers
FROM person;
```

`ENFORCED` checks that relationship endpoints exist. This matters because endpoint type restrictions alone should not be treated as complete referential-integrity enforcement.

Record references additionally support configurable deletion actions, including rejecting deletions, cascading, unsetting references, or other declared behavior. The chosen behavior is consequential for cleanup costs and invariants.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Primary sources:

`https://surrealdb.com/docs/learn/data-models/graph/record-links-vs-graph-relations`

`https://surrealdb.com/docs/learn/data-models/graph/creating-relations`

`https://surrealdb.com/docs/reference/query-language/language-primitives/record-references`

### 3.4 Advanced graph storage introduced in 3.3

SurrealDB 3.3 introduces several mechanisms that are more consequential than basic graph syntax.

`LIGHTWEIGHT` relations represent connections without storing independent edge records. Their identities are derived from endpoint pairs, making repeated creation of the same connection idempotent. They are appropriate for simple unweighted relations, but cannot carry arbitrary edge properties, field definitions, indexes or events.

```
DEFINE TABLE follows
    TYPE RELATION IN person OUT person LIGHTWEIGHT;
```

`INLINE EDGES` and `INLINE REFERENCES` maintain bounded adjacency caches on vertex tables. These reduce the number of storage lookups for low-degree records, especially when traversing many vertices. They are transactional and preserve query semantics, but they introduce write-maintenance overhead. Records exceeding the configured cache threshold use the ordinary scan path.

```
DEFINE TABLE person
    INLINE EDGES 64
    INLINE REFERENCES 32;
```

SurrealDB also packs adjacency entries into blocks in the background. These structures can make graph traversal cheaper without application-maintained adjacency caches. Execution diagnostics report cache hits, cache misses, block hits and delta hits, allowing the effect to be examined rather than assumed.

This is an important general architectural opportunity: a system that would otherwise maintain parallel forward and reverse edge indexes, adjacency structures, and duplicate-edge prevention can sometimes obtain those behaviors directly from the database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

## 4. Query languages: SurrealQL, ISO GQL and GraphQL

SurrealDB supports three materially different query surfaces over related underlying data.

| Language  | Principal purpose                                   | Characteristic syntax                          | Interface                 |
| --------- | --------------------------------------------------- | ---------------------------------------------- | ------------------------- |
| SurrealQL | General-purpose database operations                 | `SELECT`, `RELATE`, `DEFINE`, traversal arrows | `/sql`, SDK query methods |
| ISO GQL   | Declarative graph-pattern matching and manipulation | `MATCH (a)-[r]->(b) RETURN ...`                | `/gql`, GQL RPC           |
| GraphQL   | Typed client-facing data APIs                       | `query { ... }`                                | `/graphql`                |

### 4.1 SurrealQL

SurrealQL is the most complete and database-native interface.

Its useful capabilities extend beyond SQL-like CRUD:

- Structured projections over objects, arrays and record links.
- Inline graph traversals using `->` and `<-`.
- Recursive graph navigation, including depth-qualified traversal expressions.
- Aggregation, grouping, sorting, record ranges and subqueries.
- Database-side expressions, array and object transformations, closures and control flow.
- DDL, permissions, events, transactions and administrative definitions.
- Vector-distance expressions, full-text retrieval and geospatial operations.
- Query parameters and server-side functions.

For example, a graph traversal can be embedded in the projection of an ordinary table query:

```
SELECT
    name,
    ->works_at->company.{ id, name } AS companies
FROM person;
```

The practical distinction is that the caller does not necessarily need to retrieve persons, issue separate relationship queries, then perform client-side joins. However, traversal fan-out and result materialization still matter for performance.

Sources:

`https://surrealdb.com/docs/reference/query-language`

`https://surrealdb.com/docs/learn/data-models/graph`

### 4.2 ISO GQL: a genuine graph-pattern language

ISO/IEC 39075 GQL became enabled by default in SurrealDB 3.3. In 3.2 it required an experimental capability flag.

GQL operates on the existing SurrealDB records and relation tables. It is not an independent graph store, and the query is compiled into an internal execution plan rather than mechanically rewritten into SurrealQL text.

It provides graph-pattern variables, predicates, variable-length relationships, optional matches, path selection and mutations.

An illustrative pattern is:

```
MATCH (p:person)-[r:works_at]->(c:company)
WHERE c.name = 'Acme'
RETURN p.name AS person, r.role AS role;
```

The most distinctive advanced capabilities are multi-hop graph patterns and shortest-path-style selections, including `SHORTEST` and `ALL SHORTEST`. These are valuable when a graph problem is more naturally expressed as a pattern of bound vertices and edges than as a nested traversal projection.

There is a critical compatibility boundary: ISO GQL support is not a claim of complete Neo4j Cypher or openCypher compatibility. SurrealDB documents differences in inequality syntax, multi-label matching, variable-length relationship notation, and unsupported constructs. Graph algorithms should also not be inferred to exist merely because graph pattern matching and shortest paths are supported.

GQL is exposed through HTTP, WebSocket RPC, the PostgreSQL wire protocol's GQL dialect, and supported MCP functionality.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/learn/querying/gql/overview`

`https://surrealdb.com/docs/learn/querying/gql/sample-queries`

`https://surrealdb.com/docs/learn/querying/gql/mutations`

### 4.3 GraphQL: schema-driven application API

GraphQL serves a different purpose. It exposes a strongly typed API generated from SurrealDB schema definitions, with query and mutation operations and configurable inclusion of tables and functions.

```
DEFINE CONFIG GRAPHQL AUTO;
```

The generated schema may expose the database to GraphQL clients without writing a separate resolver layer for every basic operation. However, GraphQL's schema, connection/pagination conventions, authorization semantics and generated names should be assessed separately from SurrealQL.

In SurrealDB, `gql` means ISO GQL, not GraphQL. The endpoints and protocols are different.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/learn/querying/graphql/overview`

## 5. Indexing, full-text retrieval, vectors and query optimization

SurrealDB's indexing capabilities are an important part of its multimodel architecture. An indexed predicate, graph traversal and vector search can participate in the same query rather than necessarily requiring application-side coordination.

### 5.1 Conventional indexes and database-enforced uniqueness

`DEFINE INDEX` supports standard, unique, composite and specialized indexes. Particularly useful capabilities include multivalued array indexing through field paths such as `tags.*`, as well as `COUNT` indexes for count-oriented queries.

For example:

```
DEFINE INDEX by_status_time
    ON event FIELDS status, created_at;

DEFINE INDEX by_tag
    ON event FIELDS tags.*;

DEFINE INDEX unique_external_id
    ON event FIELDS external_id UNIQUE;
```

The array distinction is substantive: indexing an array as a single field is not equivalent to indexing its elements for `CONTAINS` or `CONTAINSANY` queries.

### 5.2 Bitmap combination and predicate pushdown

SurrealDB 3.3's query planner can combine indexes through bitmap operators, including:

- `BitmapIndexScan`, `BitmapFullTextScan` and `BitmapGraphScan`.
- `BitmapAnd`, `BitmapOr` and `BitmapAndNot`.
- `BitmapResolve` to fetch the records surviving combined predicates.

These capabilities reduce the need to choose one index and evaluate every remaining predicate against a large intermediate result. The bitmap operators also participate in indexed count queries and can appear below `KnnScan` to pre-filter vector searches.

The key architectural improvement is composability. A vector query constrained to a tenant, category, time range or relationship subset may benefit from ordinary indexes before candidate selection, instead of first retrieving approximate neighbors globally and discarding unwanted ones afterward.

Query planner behavior is version-sensitive. Several 3.3 patch releases corrected indexed query semantics; 3.3.4 specifically addresses incorrect results involving decimal and regex comparisons and HNSW searches during deletion.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

### 5.3 Full-text search and BM25

SurrealDB includes a text-analysis and inverted-index pipeline:

`DEFINE ANALYZER` → `DEFINE INDEX ... FULLTEXT ANALYZER` → `@@` matching → `search::score()` and highlighting.

Analyzers support tokenizers and filters such as case normalization, stemming, n-grams and language-oriented transformations. They can also use a preprocessing function. The 3.3 release extends tokenization support for CJK-oriented workloads.

A lexical search index can use BM25 ranking rather than simple substring matching. A noteworthy 3.x syntax change is that `FULLTEXT ANALYZER` replaces the older `SEARCH ANALYZER` form.

Full-text indexes are defined per indexed field; combining title and body into a single full-text field declaration is not the same as a multicolumn B-tree index.

Sources:

`https://surrealdb.com/docs/learn/data-models/full-text-search/analyzers-and-tokenizers`

`https://surrealdb.com/docs/learn/data-models/full-text-search/search-indexes`

`https://surrealdb.com/docs/reference/query-language/statements/define/indexes`

### 5.4 Vector search: exact, HNSW and DISKANN

SurrealDB supports numeric vector values, distance and similarity functions, and approximate nearest-neighbor indexes.

| Method       | Core mechanism                     | Principal distinction                                                        |
| ------------ | ---------------------------------- | ---------------------------------------------------------------------------- |
| Exact search | Direct vector comparisons          | Exact neighbor ranking for the chosen metric; potentially expensive at scale |
| HNSW         | Navigable proximity graph          | Fast approximate retrieval with an in-memory graph and persistence           |
| DISKANN      | Storage-backed ANN proximity graph | Larger indexes with bounded memory cache; introduced in SurrealDB 3.1        |

A representative HNSW index:

```
DEFINE INDEX semantic_index
    ON document FIELDS embedding
    HNSW DIMENSION 768 DIST COSINE;
```

The index configuration exposes consequential tradeoffs. HNSW has graph-connectivity and construction-search parameters such as `M` and `EFC`; DISKANN exposes `DEGREE`, `L_BUILD`, `ALPHA`, numeric element encodings and other options. Candidate-list sizing and embedding representation affect accuracy, latency, memory requirements and index construction.

DISKANN retains the graph and vectors in the key-value layer and uses a shared bounded cache; it is not available on WASM targets. HNSW retains its proximity graph in memory. Neither should be described as providing exact nearest-neighbor guarantees.

A critical migration distinction: MTREE was deprecated in 2.x and removed in SurrealDB 3.x. HNSW and DISKANN are the relevant maintained ANN index families.

Sources:

`https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes`

`https://surrealdb.com/docs/learn/data-models/vector-search/similarity-search`

### 5.5 Hybrid lexical and semantic retrieval

Full-text and vector search can be composed using built-in functions such as `search::rrf()` for reciprocal rank fusion.

A typical operation combines BM25-ranked literal matches with embedding-ranked semantic candidates, optionally subject to metadata constraints or graph traversal. This can eliminate a custom ranking-combination layer for straightforward hybrid retrieval.

The database manages storage and retrieval of embeddings, but it does not eliminate the need for an embedding model or embedding-generation process. Embedding creation, chunking and model consistency remain responsibilities of upstream components or explicitly configured integrations.

Likewise, reciprocal rank fusion is a ranking-combination mechanism, not a learned cross-encoder reranker.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Source: `https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search`

### 5.6 Explain plans and execution diagnostics

SurrealDB 3.x provides a tree-oriented `EXPLAIN` statement, with `ANALYZE` and JSON output options.

```
EXPLAIN ANALYZE FORMAT JSON
SELECT * FROM document WHERE category = "science";
```

The execution plan can reveal index scans, graph-edge scans, bitmap operations, KNN scans, sorting, aggregation, row counts, batches and elapsed execution time.

Additional useful facilities include indexed `count()` evaluation, top-K sorting, query timeouts, and `TEMPFILES` for operations where spilling to temporary files is preferable to large in-memory intermediates.

Limitation: SurrealDB explicitly treats the `EXPLAIN` output as informational and subject to change. Its JSON structure should not be treated as a permanent compatibility contract for external tooling.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/explain`

## 6. Transactions, state changes and reactive execution

### 6.1 Multi-statement transactions and concurrency

Ordinary statements execute transactionally. Explicit `BEGIN`, `COMMIT` and `CANCEL` group operations into one atomic unit.

The Rust SDK additionally provides client-side transaction objects through `.begin()`, followed by database operations on the transaction and `.commit()` or `.cancel()`.

SurrealDB's underlying engines use multiversion techniques and conflict detection. The database should not, however, be assumed to provide identical isolation semantics under every storage backend, access pattern and deployment model.

SurrealDB 3.3 includes `SELECT ... FOR UPDATE`, which adds read-conflict tracking useful for patterns where correctness depends on a value remaining unchanged between reading it and committing a dependent write.

The relevant performance tradeoff is familiar: larger or longer-running transactions can enforce more extensive invariants, but increase conflict exposure and resource lifetime.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Sources:

`https://surrealdb.com/docs/reference/query-language/language-primitives/transactions`

`https://surrealdb.com/docs/reference/rust/concepts/transaction`

### 6.2 Live queries versus persistent changefeeds

These serve different operational needs.

| Mechanism                     | Primary use                               | Important limitation                                     |
| ----------------------------- | ----------------------------------------- | -------------------------------------------------------- |
| `LIVE SELECT`                 | Push record changes to connected clients  | Subscription lifecycle and connection management matter  |
| `CHANGEFEED` / `SHOW CHANGES` | Read and replay recorded database changes | Retention and cursor management are required             |
| `DEFINE EVENT`                | Execute logic as data changes             | Side effects and transaction coupling must be understood |

Live queries send notifications about matching records changing. They fit interfaces that require current state, but are not a general substitute for a durable, replayable event log.

Changefeeds retain changes for a declared duration and allow consumers to retrieve them by timestamp or versionstamp. They support incremental replication and catch-up patterns without requiring every downstream application to implement its own mutation journal.

`INCLUDE ORIGINAL` changes the information retained about previous record states. Versionstamp semantics and retention windows are important for consumers that need reliable incremental progress.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/learn/querying/real-time/live-queries`

`https://surrealdb.com/docs/reference/query-language/statements/show`

### 6.3 Database events: synchronous and asynchronous

`DEFINE EVENT` provides conditional data-change behavior with access to `$before`, `$after`, `$event`, and the affected record.

Synchronous events execute as part of the originating transaction. They can maintain data invariants or associated state atomically, but their execution cost adds to write latency.

Since 3.0, `ASYNC` events can run outside the originating transaction. Enqueuing is coupled to the original transaction, while processing occurs later in a separate transaction. Retry and maximum-depth controls are provided.

The consequential guarantee is at-least-once execution, not exactly-once execution. Operations such as outbound HTTP calls need idempotency or other external deduplication, because retrying an event cannot roll back an already completed external effect.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/event`

### 6.4 Materialized views and derived data

`DEFINE TABLE ... AS SELECT ...` supports pre-computed table views. These can maintain derived data and aggregate projections in response to underlying writes.

This is especially useful for counters, categorical summaries and time-bucketed aggregations, where repeatedly scanning detailed records would be unnecessary. It may replace custom event handlers and application-maintained rollup tables for supported definitions.

It is not equivalent to having a completely unrestricted, automatic relational materialized-view engine: the supported view expression and update semantics determine which aggregates and transformations are suitable.

Source: `https://surrealdb.com/docs/reference/query-language/statements/define/table`

Source: `https://surrealdb.com/docs/learn/data-models/time-series/aggregation-queries`

## 7. Time-series and geospatial workloads

SurrealDB's time-series capabilities arise from native datetimes, sortable record IDs, range scans, time functions, aggregation and derived tables. `time::floor()` and `time::group()` can establish temporal buckets.

A useful distinction is that SurrealDB provides time-series modeling and computation, but that does not automatically make it interchangeable with a purpose-built columnar telemetry database offering specialized compression, continuous aggregates and retention policies. Such characteristics require workload-specific verification.

Source: `https://surrealdb.com/docs/learn/data-models/time-series/time-buckets-and-windowing`

For geospatial workloads, SurrealDB supports GeoJSON-style points, lines and polygons, together with `geo::` calculations and predicates such as intersection, containment and distance.

Important limitation: SurrealDB does not currently expose a dedicated spatial index type comparable to a PostGIS GiST spatial index. Its documentation suggests geohash-derived keys or ordinary indexes for pruning large spatial search spaces. Coordinate order is longitude, then latitude.

Sources:

`https://surrealdb.com/docs/learn/data-models/geospatial/geometry-types`

`https://surrealdb.com/docs/learn/data-models/geospatial/location-based-patterns`

## 8. Storage engines: RocksDB, SurrealKV and SurrealMX

The October 2026 storage transition is one of the most important version-sensitive aspects of the ecosystem.

### 8.1 Storage-engine comparison

| Engine                            | Implementation                             | Main role in SurrealDB 3.3                     | Key tradeoff                                                                                                   |
| --------------------------------- | ------------------------------------------ | ---------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| RocksDB                           | C++ LSM-tree engine                        | Recommended durable single-node server backend | Mature and tunable; larger native dependency and build surface                                                 |
| SurrealKV                         | Rust-native LSM-based engine               | Beta backend in SurrealDB 3.3                  | Designed for embedded and local workloads; its standalone 1.0 improvements are not yet all integrated into 3.3 |
| SurrealMX                         | Rust-native transactional in-memory engine | Default in-memory backend                      | Low storage overhead and high concurrency; primary dataset is memory-resident                                  |
| IndexedDB                         | Browser storage                            | Embedded browser use                           | Browser and runtime constraints                                                                                |
| Distributed transactional storage | Deployment-specific storage infrastructure | Clustered editions and managed Scale           | Replication, consensus and operational/edition boundaries                                                      |

### 8.2 RocksDB

RocksDB contributes the persistent ordered key-value substrate, including memtables, SSTables, compaction, caches and transactional storage capabilities.

SurrealDB adds the records, indexes, graph structures, query language, permissions and high-level transaction coordination above it. This means RocksDB's column families or native APIs should not be assumed to be exposed as independent application interfaces through SurrealDB.

RocksDB can be selected in a server using a storage path such as:

```
surreal db start --user root --pass secret rocksdb://mydata
```

In an embedded Rust application, it is enabled through the `kv-rocksdb` Cargo feature. Building it introduces a C++ compiler and `libclang` dependency.

RocksDB remains the documented production on-disk server recommendation for SurrealDB 3.3 and earlier. It offers a substantial tuning surface but also places more responsibility on engine-level configuration and native build compatibility.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Source: `https://surrealdb.com/docs/manage/self-hosted/deployment-models`

Source: `https://surrealdb.com/docs/reference/rust/embedding`

### 8.3 SurrealKV 1.0: the incoming native persistent engine

SurrealKV is a separately published Rust-native storage library. Its standalone 1.0 release of October 13, 2026 includes:

- LSM-oriented persistent storage and snapshot-isolated transactions.
- Concurrent adaptive-radix-tree memtables.
- A commit-ring and grouped write-ahead-log flushing.
- Opt-in historical version retention and version-oriented reads.
- Optional separation of large values from the compaction hot path.
- Transparent encryption-at-rest facilities.

Its design aims to reduce synchronization overhead, native dependencies and the complexity of tuning a large general-purpose LSM engine.

The upstream benchmarks report substantial advantages over RocksDB in several synchronous-durability tests, but those measurements use particular hardware, data distributions, worker counts, tuning settings and durability policies. The same benchmark publication reports that RocksDB is faster for several writes when synchronous durability is disabled. Those are storage-library microbenchmarks, not proof of equivalent end-to-end SurrealQL speedups.

Most importantly, SurrealDB 3.3 still uses the beta SurrealKV integration. Upstream documentation states that SurrealDB 3.4 will incorporate SurrealKV 1.0 as its preferred persistent engine, while SurrealDB 4 is planned to make SurrealKV the sole persistent engine. These future integrations should not be mistaken for already released SurrealDB 3.3 functionality.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+1



Sources:

`https://surrealdb.com/blog/we-are-replacing-rocksdb-with-surrealkv-1-0`

`https://github.com/surrealdb/surrealkv`

### 8.4 SurrealMX 1.0: in-memory transactional execution

SurrealMX is the Rust-native in-memory storage engine, with a separate 1.0 release announced October 12, 2026.

Its storage-library architecture uses an adaptive radix tree, multiversion data, concurrency-oriented commit processing, and optimized scanning. The standalone library exposes advanced transactional facilities, including snapshot-isolation options and savepoints.

Its core role is memory-resident, transactional data. It is therefore relevant to disposable datasets, caches, validation workloads, simulations and other situations where maintaining all primary data on disk would introduce unnecessary cost.

Optional persistence mechanisms exist in the standalone engine, but should not be conflated with RocksDB-style durable operation under the normal in-memory SurrealDB configuration. The fastest fully in-memory configuration loses data when the process exits.

Also, SurrealDB 3.3's SurrealMX integration does not support `SELECT ... VERSION` historical queries. This is an explicit backend limitation, not a general absence of temporal querying in SurrealDB.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+1



Sources:

`https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine`

`https://github.com/surrealdb/surrealmx`

### 8.5 Temporal versioning in RocksDB and SurrealKV

Where the datastore supports it and versioning is enabled, SurrealDB can retain prior record states and query them using a `VERSION` clause.

For instance, a version-enabled file-backed engine can answer a query about a record as it existed at an earlier datetime.

This opens useful possibilities for historical inspection, state comparisons and reconstruction of prior views, but it has costs: additional retained versions increase storage and compaction work, and availability depends on retention configuration.

Historical versioning, changefeeds and logical backups are separate mechanisms. A version-enabled datastore does not imply that a standard `surreal db export` includes all previous record versions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/reference/query-language/statements/select`

`https://surrealdb.com/docs/reference/cli/db/commands/start`

## 9. Rust SDK 3.3.4: embedded and remote integration

The official `surrealdb` crate is a substantial integration layer, not merely an HTTP client. It provides embedded database initialization, connection management, typed value conversion, query execution, transactions, authentication, live-query subscriptions and streaming interfaces.

The current 3.3.4 SDK requires Rust 1.95 or later. Its APIs should not be inferred from Rust SDK 2.x examples: the 3.x typed-value model contains significant breaking changes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 9.1 Embedded and network execution

The primary distinction is between an embedded engine and a network transport.

| Configuration      | Typical connection                      | What changes                                                     |
| ------------------ | --------------------------------------- | ---------------------------------------------------------------- |
| In-memory embedded | `Surreal::new::<Mem>(())`               | Database executes in process and retains primary data in memory  |
| RocksDB embedded   | `Surreal::new::<RocksDb>(path)`         | Persistent local database with native RocksDB build requirements |
| SurrealKV embedded | `Surreal::new::<SurrealKv>(path)`       | Persistent local database using the Rust-native backend          |
| Remote WebSocket   | `Surreal::new::<Ws>(address)`           | Database executes in a separate process                          |
| Remote HTTP        | `Surreal::new::<Http>(address)`         | HTTP transport semantics                                         |
| Remote gRPC        | `grpc://` with protocol support enabled | Native RPC transport, including incremental row streaming in 3.3 |

A minimal embedded example:

```
use surrealdb::engine::local::Mem;
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb::Surreal;

#[derive(Debug, SurrealValue)]
struct Person {
    name: String,
    active: bool,
}

#[derive(Debug, SurrealValue)]
struct PersonRecord {
    id: RecordId,
    name: String,
    active: bool,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("main").use_db("main").await?;

    let person: Option<PersonRecord> = db
        .create("person")
        .content(Person {
            name: "Ada".into(),
            active: true,
        })
        .await?;

    println!("{person:?}");
    Ok(())
}
```

The `kv-mem`, `kv-rocksdb` and `kv-surrealkv` Cargo features enable the corresponding embedded backends. Using RocksDB introduces native compilation requirements, whereas SurrealKV is Rust-native.

A particularly important operational limitation is that an embedded RocksDB or SurrealKV datastore is exclusively opened by one database instance. Independent processes cannot safely open the same database directory simultaneously, and creating multiple SDK connections to the same embedded directory can conflict with its lock. Cloning an existing SDK client shares the open datastore instead.

This distinction determines whether embedded execution can replace a separate server: multiple tasks within one process can share an embedded engine, but unrelated processes requiring simultaneous access ordinarily need a server.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rust/embedding`

### 9.2 Native SurrealDB values and Rust type safety

The centerpiece of the 3.x SDK type system is `SurrealValue`, which controls conversion between Rust objects and native SurrealDB values.

It is not simply Serde serialization under another name.

```
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, SurrealValue)]
struct Document {
    id: RecordId,
    title: String,
    parent: Option<RecordId>,
    tags: Vec<String>,
}
```

Important capabilities include:

- `#[derive(SurrealValue)]` for structured conversion.
- `#[surreal(...)]` attributes for renaming, flattening and other transformations.
- `Value` for dynamic database values.
- `RecordId` and `RecordIdKey` for preserving record identity.
- `Kind`, `kind!` and `SurrealValue::kind_of()` for representing corresponding SurrealQL types.
- `ToSql` for obtaining an appropriate SurrealQL textual representation.

The `SurrealValue` trait can also be implemented manually, including for domain-specific types whose storage representation is not identical to their Rust representation.

These facilities open the possibility of sharing type descriptions between application-level Rust types and database field definitions, although the library does not automatically make schema evolution type-safe across deployed versions.

A consequential detail: SurrealKit's type generator does not currently generate Rust structs from the database schema. It generates schema metadata as JSON and supports TypeScript generation. Conversely, the Rust SDK can describe the SurrealQL type corresponding to a Rust type.

The 3.x distinction between native `datetime` and a string containing a formatted timestamp is another reason not to treat Serde JSON round-tripping as semantically equivalent. An application may otherwise store a date as a string and later obtain incorrect type-based query comparisons.

Similarly, a `record<person>` field normally maps to `RecordId`, not a fully materialized `Person` struct. Fetching the linked record is an explicit query operation that changes the result shape.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Sources:

`https://surrealdb.com/docs/reference/rust/concepts/working-with-types`

`https://surrealdb.com/docs/reference/rust/concepts/surreal-value-attributes`

`https://github.com/surrealdb/surrealdb/tree/main/surrealdb/types`

### 9.3 Query composition and result handling

The SDK provides high-level methods for creating, reading, updating, inserting, upserting and deleting records. It also supports raw SurrealQL with parameter binding.

```
let mut response = db
    .query("SELECT * FROM person WHERE active = $active")
    .bind(("active", true))
    .await?;

response.check()?;
let people: Vec<PersonRecord> = response.take(0)?;
```

Parameter binding avoids textual interpolation for values. It also preserves native SurrealDB types that might otherwise be lost in text-based query construction.

Multi-statement result handling deserves special attention. Successful completion of the Rust future does not mean every SurrealQL statement succeeded. The SDK's `IndexedResults` can contain statement-level errors; `.check()`, `.take_errors()` and indexed `.take()` are the relevant methods for examining them.

The SDK also supports per-statement statistics, which allow callers to collect database execution information without parsing console output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rust/methods/query`

### 9.4 Incremental result streaming, new in 3.3

The `stream_items()` API is an especially significant new SDK capability for large query results.

Normal awaited queries produce `IndexedResults`, which materializes the result set before handing it to the caller. By contrast:

```
use futures::StreamExt;
use surrealdb::method::StreamItem;

let mut stream = db
    .query("SELECT * FROM event")
    .stream_items()?;

while let Some(item) = stream.next().await {
    match item? {
        StreamItem::Row { value, .. } => {
            // Process a row.
            let _ = value;
        }
        StreamItem::StatementEnd { result, .. } => {
            result?;
        }
    }
}
```

This distinction can reduce peak memory and allow computation to overlap database execution.

However, its transport semantics are important.

| Execution mode                      | Incremental delivery                          |
| ----------------------------------- | --------------------------------------------- |
| Embedded memory, RocksDB, SurrealKV | Yes                                           |
| Remote gRPC                         | Yes                                           |
| Remote WebSocket                    | No; items are replayed after query completion |
| Remote HTTP                         | No; items are replayed after query completion |

Moreover, a streamed row is provisional until its statement succeeds. A statement can fail after emitting rows, and a transaction can subsequently roll back. A consumer that creates irreversible external effects from provisional rows would need to handle this possibility explicitly.

Dropping the stream also has query-lifecycle consequences, because the database must finalize the transaction held by the streaming execution.

This makes `stream_items()` an important alternative to bespoke pagination or intermediate materialization for supported transports, but not an automatic transactional event stream. It is also distinct from subscribing to `LIVE SELECT` updates.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/rust/methods/query`

### 9.5 Client-side transactions and concurrency

The SDK supports manually controlled transactions with `.begin()`, `.commit()` and `.cancel()`, enabling multiple database operations to be grouped under one transaction context.

The client is designed for concurrent asynchronous use. It can be shared by cloning its handle rather than opening a new datastore or socket for every task.

For remote connections, the SDK exposes `.with_capacity(n)` to configure bounded request-channel capacity. This is a significant backpressure control: a bounded channel can prevent an overloaded caller from accumulating an unlimited backlog of pending requests, at the cost of making producers wait.

There is no automatic implication that increasing connection counts or request capacity increases throughput. Backpressure, database lock contention, query parallelism, index access and the selected transport determine the result.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/reference/rust/concepts/transaction`

`https://surrealdb.com/docs/reference/rust/concepts/concurrency`

### 9.6 Companion crates and integration boundaries

The core Rust ecosystem includes the `surrealdb` client, `surrealdb-types` for native data values, lower-level engine components, and the separately released SurrealKit tooling.

The ability to use database-native `Value`, `RecordId` and `Kind` independently of a live connection is valuable for building structured query and data transformation tooling. But these value types do not themselves implement the full query planner, permission system or storage engine.

The official documentation also provides integration examples for Actix, Axum, Rocket and Egui. These examples demonstrate application integration, not separate persistence backends.

Source: `https://surrealdb.com/docs/reference/rust`

## 10. SurrealKit 1.0.0-beta.6: schema, migration and validation tooling

SurrealKit deserves separate treatment because it supplies operational capabilities that are not intrinsic to the Rust SDK.

The 1.0 beta series transforms SurrealKit into a more comprehensive declarative schema management, controlled migration, metadata tracking and validation framework.

The published documentation landing pages still largely describe 1.0.0-beta.1. The following discussion uses the tagged 1.0.0-beta.6 source and migration notes to distinguish features and fixes actually present in the requested beta line.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



### 10.1 Declarative schema synchronization

The basic model is to maintain the desired database schema in version-controlled `.surql` files.

```
database/
  schema/
  rollouts/
  snapshots/
  seed/
  tests/
  setup.surql

surrealkit.toml
```

`surrealkit sync` compares the local schema files with tracked database state and applies changed definitions.

The schema files remain normal SurrealQL. SurrealKit is the reconciliation and tracking layer, not a separate query language or an ORM.

Key capabilities include content-hash change tracking, dependency-aware ordering, dry runs, automatic removal of deleted managed definitions, and `sync --watch` for repeated development reconciliation.

There are meaningful safety boundaries:

- Sync is destructive reconciliation. Removing a tracked definition from the source files can remove that definition from the database.
- An unexpectedly empty source directory is rejected by default. Explicit `--allow-empty-prune` is required for the relevant exception.
- A wholesale file reorganization can look like removal and re-creation, because the tracked identity includes file paths.
- Reapplying definitions is not always semantically harmless, particularly for sequences and authentication definitions.

For that reason, sync's greatest value is in environments where the declared schema can authoritatively replace the current schema, especially disposable and development databases. It should not be confused with a universally safe production migration algorithm.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/manage/schema-migration/sync`

`https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md`

### 10.2 Controlled expand-and-contract rollouts

SurrealKit's second approach is a staged migration system.

A rollout has an expansion phase for changes that can coexist with the old application, followed by a contract phase that removes obsolete definitions after a separate application cutover.

The significant commands are:

| Operation                   | Command                              | Purpose                                           |
| --------------------------- | ------------------------------------ | ------------------------------------------------- |
| Establish existing baseline | `surrealkit rollout baseline`        | Identify starting schema state                    |
| Plan migration              | `surrealkit rollout plan --name ...` | Produce a reviewable migration manifest           |
| Check migration chain       | `surrealkit rollout lint`            | Detect missing steps, forks and invalid artifacts |
| Start expansion             | `surrealkit rollout start <id>`      | Apply the initial phase                           |
| Advance pending rollouts    | `surrealkit rollout up`              | Bring lagging databases forward                   |
| Complete contraction        | `surrealkit rollout complete <id>`   | Apply deferred destructive changes                |
| Undo an unfinished rollout  | `surrealkit rollout rollback <id>`   | Restore supported prior definitions               |
| Recover interrupted state   | `surrealkit rollout repair <id>`     | Reconcile metadata without replaying SQL          |

The system maintains rollout progress in database metadata, providing resumability and visibility into incomplete migrations.

The most important beta.6 improvement is frozen rollout SQL.

When a rollout is planned, SurrealKit copies the relevant schema files into a rollout-specific directory and records their hashes. Deployment uses these frozen copies, not the latest contents of `database/schema`.

That solves an important historical-migration problem: a database several releases behind can receive the SQL that belonged to each intermediate migration rather than trying to apply every past manifest against the newest schema files.

`rollout up` can advance through pending migrations in order. The newest can remain in the expanded state for application cutover; `up --complete` performs its contract phase. Rollout linting detects forks in the migration history and changes not represented by planned rollouts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com

+1



Source: `https://surrealdb.com/docs/manage/schema-migration/rollouts`

Source: `https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md`

### 10.3 The limits of automatic rollback

Rollout reversibility is not equivalent to automatically reversing arbitrary data changes.

SurrealKit can restore supported schema definitions. However, reversing a changed `TYPE`, `VALUE` or `DEFAULT` definition does not necessarily restore data written or transformed while the new definition was active.

The `--allow-modified` planning option explicitly acknowledges changes that may only be reversible at the definition level.

Custom SQL backfills and externally visible side effects require appropriate migration design and idempotency. Completing a destructive contract phase also changes what can reasonably be recovered without a backup or a separately constructed inverse migration.

This is a useful distinction between schema state rollback, data rollback, and recovery from a backup. They are different guarantees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 10.4 Migration reliability improvements specific to beta.6

Several less visible beta.6 changes are important for production reliability.

Safe sequence handling. Previous behavior could overwrite a `DEFINE SEQUENCE`, resetting its counter and creating the possibility of subsequent record-ID collisions. Beta.6 rewrites an ordinary sequence definition to `IF NOT EXISTS`; explicit `OVERWRITE` remains possible but produces a warning.

Historical rollout compatibility. Frozen files and per-file hashes allow ordered catch-up and detect modifications to already planned SQL. Existing earlier beta manifests can be frozen using the provided upgrade operation, subject to availability of their original files.

Improved parser handling. A more capable SurrealQL statement scanner handles regex literals, comments, escaped identifiers and statement boundaries. Invalid files now fail before schema definitions are applied or pruned, rather than risking a partially interpreted schema.

Incremental metadata reconciliation. The entity catalog is updated in bounded batches rather than cleared and rewritten wholesale. This reduces the risk of leaving the catalog empty after an interrupted operation.

Bounded connection establishment. Connection and sign-in operations have a default timeout, while migration SQL can be given a separate timeout. Long-running steps report progress; `repair` can reconcile certain interrupted rollout states without re-executing their SQL.

SurrealDB 3.3 support. Beta.6 is built against the 3.3 SDK and requires Rust 1.95. It retains tested compatibility with certain 3.2 servers, though server-language differences remain relevant.

The beta.6 source also warns that overwriting a record access definition without a stable JWT signing key can rotate that key and invalidate existing sessions. This is an example where declarative schema reconciliation must account for security state, not just table structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com

+2



### 10.5 Schema modules and database targets

SurrealKit 1.0 supports splitting a schema into modules and managing multiple database targets from one project configuration.

A module can declare dependencies on other schema modules. This allows definitions to be maintained in logical groups without building an entirely bespoke schema dependency system.

Targets define database-specific connection information and can be combined with modules using `sync --all`.

These features support shared definitions across multiple namespaces or databases, but multi-target execution is not a single distributed transaction. Targets are updated individually, and an interrupted operation can leave them at different schema revisions.

Module identity also matters. Renaming an already applied schema module is not automatically a harmless filesystem refactor: its metadata ownership changes, which can cause existing definitions to appear stale.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Sources:

`https://surrealdb.com/docs/manage/schema-migration/modules-and-targets`

`https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md`

### 10.6 Schema embedding inside a Rust binary

SurrealKit offers an `embed_schema!` macro and a library API that can package database schema definitions with an application.

The architectural distinction is that a Rust application can carry the definitions needed to initialize or reconcile its embedded database without requiring an external migration executable and runtime filesystem source tree.

The library also supports embedded seed data through `embed_seed!`.

This is especially useful when the database and application share a release lifecycle, but it does not remove the distinction between application startup initialization and a controlled migration of valuable existing data.

Source: `https://surrealdb.com/docs/manage/schema-migration/embed-schema-macro`

Source: `https://surrealdb.com/docs/manage/schema-migration/library`

### 10.7 Seeding, templates and generated types

SurrealKit's seed subsystem tracks `.surql` files and avoids reapplying unchanged seed files. It supports explicit forced reapplication, which is useful but should not be confused with exactly-once data-mutation semantics.

Schema and seed files can contain template variables resolved from configuration, environment variables or CLI arguments. This supports parameterized definitions, although changes to a variable alone do not necessarily change a schema file's content hash and trigger reapplication.

`surrealkit typegen` introspects database metadata and generates JSON schema descriptions, with optional TypeScript definitions suitable for integration with the JavaScript SDK. Beta.6 additionally supports configurable output filenames and direct paths.

For Rust applications, this is primarily a schema-inspection and cross-language type-generation capability, not a native Rust ORM code generator.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com

+1



Sources:

`https://surrealdb.com/docs/manage/schema-migration/seeding`

`https://surrealdb.com/docs/manage/schema-migration/typegen`

### 10.8 Declarative schema and security testing

SurrealKit 1.0 includes a testing framework with isolated test databases and declarative TOML suites.

Its five documented test categories are:

| Test type            | Capability                                                      |
| -------------------- | --------------------------------------------------------------- |
| `sql_expect`         | Execute SurrealQL and assert values, errors or expected results |
| `permissions_matrix` | Exercise table and record permissions across defined actors     |
| `schema_metadata`    | Assert definitions and metadata                                 |
| `schema_behavior`    | Verify database-enforced behavior                               |
| `api_request`        | Test HTTP API endpoints and responses                           |

The permission-testing capability is particularly valuable: it can evaluate the database's actual authorization behavior rather than relying on static inspection of `PERMISSIONS` declarations.

Suites can use isolated ephemeral namespace/database combinations and run with distinct authentication contexts. The test runner supports filtering, parallel execution, JSON reports and nonzero exit status on failure.

An especially useful 1.0 feature is schema-source parity testing. Suites can be run against schema established through direct sync, through rollout replay, or both. When rollouts are replayed, the tool can compare the resulting schema against direct sync to detect changes that never entered the migration chain.

This turns schema management into a testable artifact rather than a set of migration scripts whose end state is assumed to be correct.

The 1.0 series also corrected an important 0.7 testing problem: assertions against nonexistent JSON paths or response headers no longer silently succeed. Missing paths fail unless absence is explicitly asserted.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com

+1



Source: `https://surrealdb.com/docs/manage/schema-migration/testing`

### 10.9 Other SurrealKit ecosystem integration

The tagged beta.6 repository also includes `vite-plugin-surrealkit`, which integrates schema synchronization into Vite-based development and build workflows. Its versioning is independent of the Rust CLI. The associated migration notes specifically distinguish Vite 8-compatible plugin releases from earlier plugin versions.

A further source-version caveat matters here: the repository's later `main` branch documents a new static SurrealQL analysis feature with commands such as `check`, `generate` and `watch`. I could not verify those commands in the tagged 1.0.0-beta.6 README, so they should not be treated as established beta.6 functionality. They are a separate development-line capability to investigate once explicitly released.

Primary sources for the requested beta version:

`https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-beta.6`

`https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md`

`https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md`

## 11. Database-native authorization, API endpoints and execution capabilities

### 11.1 Authorization integrated with data semantics

SurrealDB's security model includes root, namespace and database users, together with record-based authentication and permissions.

`DEFINE ACCESS ... TYPE RECORD` allows the database to implement record-backed signup and signin logic. Record users can authenticate through a declared access method and receive a session associated with their record identity.

At the data layer, table and field permissions can use SurrealQL predicates to control access to individual records and fields. This makes authorization part of database query execution rather than requiring all filtering to be implemented by a separate application service.

However, permissions depend on the caller's authentication context. Privileged system users are not equivalent to ordinary record users, and testing a query with root credentials does not demonstrate what an authenticated application user may access.

SurrealDB also provides bearer and JWT access mechanisms, configurable token lifetimes and revocation-related functionality. Token management, session lifetimes and signing-key stability remain distinct security concerns.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Sources:

`https://surrealdb.com/docs/reference/query-language/statements/define/access/record`

`https://surrealdb.com/docs/learn/security/authorization/permissions-and-row-level-security`

`https://surrealdb.com/docs/learn/security/authorization/tokens-and-jwts`

### 11.2 Capability restrictions and controlled execution

SurrealDB exposes a capability model for controlling which categories of operations are permitted, including functions, scripting, network access and experimental functionality.

Administrators can configure broad restrictions and specific exceptions. For example, external HTTP functions may be available only for explicitly allowed destinations.

This provides a useful separation between database authorization, which determines which data a caller may access, and execution capabilities, which constrain what code or external operations the database may perform.

The distinction is especially important when database queries can execute functions with side effects, make network calls or invoke extension code.

Source: `https://surrealdb.com/docs/learn/security/authorization/capabilities`

### 11.3 Database-defined HTTP APIs

SurrealDB 3.x supports `DEFINE API`, which creates HTTP endpoints backed by database-defined handlers.

These endpoints live under paths of the form `/api/:namespace/:database/:endpoint_name` and can be used to expose constrained application operations without implementing an equivalent handler in a separate web framework.

The resulting architecture can combine request handling, database access, authorization and transactional updates inside SurrealDB.

This is potentially valuable for compact applications and internal services, but it is not a universal replacement for an application server. Complex external orchestration, sophisticated request middleware, specialized transport behavior and independently versioned APIs may still justify another execution layer.

`DEFINE API` is also distinct from the automatically generated GraphQL interface and from the general `/sql` query endpoint.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/query-language/statements/define/api`

## 12. Extensibility and additional data capabilities

### 12.1 SurrealQL functions, closures and scripting

SurrealDB supports database-defined functions as well as an extensive native function library for arrays, objects, strings, time, mathematics, cryptography, vector operations, geospatial computations and other transformations.

Closures and control-flow constructs allow parts of the data processing logic to remain within the database.

These facilities are useful because a query can transform, validate and relate data without necessarily transferring intermediate values to an application.

There are limits to that approach. Database-side execution consumes database resources, and the capability model may restrict scripting or external function calls. Moving complex logic into a query also creates coupling to SurrealQL's version-specific execution semantics.

Sources:

`https://surrealdb.com/docs/reference/query-language/functions`

`https://surrealdb.com/docs/reference/query-language/statements/define/function`

### 12.2 Surrealism: Rust-authored WebAssembly extensions

Surrealism is SurrealDB's Rust-authored extension system.

It allows functions implemented in Rust to be compiled into sandboxed `.surli` modules and invoked through SurrealDB.

Its architecture includes the `wasm32-wasip2` Rust target, the `#[surrealism]` attribute for exported functions, module initialization, namespaced exports, optional packaged read-only files and resource constraints.

The associated CLI supplies commands such as:

```
surreal db module init
surreal db module build
```

This offers a middle ground between implementing every transformation in SurrealQL and adding a separate external service for each specialized computation.

For example, a specialized parser or domain-specific numeric transformation might be implemented once in a Rust crate and exposed to queries without moving the source data out of the database.

Status qualification: Surrealism remains an experimental capability in SurrealDB 3.3 and requires explicit enablement. Its sandbox and capability boundaries are not the same as running arbitrary native Rust code inside the database server process.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+2



Sources:

`https://surrealdb.com/docs/learn/extensions/plugins/overview`

`https://surrealdb.com/docs/learn/extensions/guides/creating-custom-modules`

`https://surrealdb.com/docs/learn/extensions/guides/module-architecture`

### 12.3 Experimental file storage and external object stores

SurrealDB also has a file-value model based on `DEFINE BUCKET` and `f"bucket:/path"` references.

Files can be manipulated using database-native file functions. Backends include memory and local filesystem storage.

SurrealDB 3.3 adds object-storage backends for S3 and S3-compatible services, Google Cloud Storage and Azure Blob Storage in native builds.

This allows database records and database-managed file references to participate in a shared data model, potentially reducing the need for an independently maintained metadata and object-reference layer.

However, the feature remains experimental and requires the `files` capability. External object-storage operations also introduce network, credential and failure semantics that are different from local key-value transactions. One should not infer full atomicity between arbitrary cloud object operations and record transactions without an explicit documented guarantee.

WASM builds have more limited backend availability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/reference/query-language/statements/define/bucket`

`https://surrealdb.com/docs/learn/schema-management/files/working-with-files`

## 13. Protocols, administration, graphical tools and agent integrations

### 13.1 Network protocols

SurrealDB exposes several network interfaces, each with different behavior.

| Interface                | Purpose                                       | Material distinction                                       |
| ------------------------ | --------------------------------------------- | ---------------------------------------------------------- |
| HTTP query endpoints     | SurrealQL, GQL, GraphQL, administration       | Request/response execution                                 |
| WebSocket RPC            | Persistent SDK connection and subscriptions   | Supports live-query delivery                               |
| gRPC RPC                 | Native RPC transport                          | Supports incremental result streaming in 3.3               |
| PostgreSQL wire protocol | Compatibility with PostgreSQL network clients | Protocol-compatible, not PostgreSQL SQL-compatible         |
| MCP                      | AI-agent-oriented database tools              | Separate authentication and execution-trust considerations |

The PostgreSQL wire protocol, introduced in 3.3, is particularly interesting because it permits clients such as `psql`, JDBC and `tokio-postgres` to communicate with SurrealDB using the PostgreSQL network framing.

It supports simple and extended query flows, prepared statements, cancellation and other protocol features. The listener is opt-in through `--postgres-bind`.

The consequential distinction is that the query dialect is still SurrealQL or ISO GQL, not general ANSI SQL or PostgreSQL SQL. PostgreSQL catalog emulation and `COPY` are among the documented missing capabilities. Therefore, connectivity through a PostgreSQL driver does not imply compatibility with SQLAlchemy's PostgreSQL dialect, PostgreSQL migrations or PostgreSQL-specific database tooling.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/reference/rest-api/rpc-protocol`

`https://surrealdb.com/docs/reference/rest-api/postgres-protocol`

### 13.2 SurrealDB CLI

The main `surreal` executable provides database server and operational commands under `surreal db`.

Major functions include starting a database, opening an interactive SQL shell, validating and importing data, performing logical exports, checking readiness, and managing modules.

Representative commands are:

```
surreal db start --user root --pass secret memory

surreal db sql --endpoint ws://localhost:8000 \
  --namespace main --database main \
  --username root --password secret

surreal db export ...
surreal db import ...
surreal db isready ...
```

The CLI is useful beyond basic server operation. It exposes features such as storage backend selection, experimental capability controls, request limits, authentication configuration and optional PostgreSQL-wire connectivity.

The CLI command structure has evolved substantially, so older instructions using top-level `surreal start` or `surreal sql` should not automatically be assumed to be the preferred syntax for the 3.3 release line.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database

+1



Sources:

`https://surrealdb.com/docs/reference/cli/db/commands/start`

`https://surrealdb.com/docs/reference/cli/db/commands/sql`

### 13.3 SurrealDB Studio

Graphical administration and inspection

SurrealDB Studio is the official graphical application for database exploration and management. It superseded Surrealist in August 2026.

It provides a visual environment for connecting to databases, exploring data and schema, authoring queries, and working with managed or self-hosted instances.

It complements rather than replaces SurrealKit: Studio is principally an interactive inspection and administration surface, while SurrealKit tracks version-controlled schema state and migration history.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Sources:

`https://surrealdb.com/blog/introducing-surrealdb-studio`

`https://studio.surrealdb.com`

### 13.4 Model Context Protocol

SurrealDB includes MCP integration for programming agents and related applications.

The built-in `surreal db mcp` command starts an MCP server using an embedded datastore over standard input/output. A networked SurrealDB server can expose an `/mcp` endpoint.

This provides a database-aware tool interface for agents without requiring each application to implement its own query execution and inspection protocol.

The trust boundary is essential: the embedded stdio mode operates with powerful database access and is intended for trusted local operation. It should not be exposed as an unrestricted multi-user service. MCP also does not replace the need to reason about query permissions, destructive operations or data provenance.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/reference/cli/db/commands/mcp`

### 13.5 AI and external framework integrations

SurrealDB has integrations with several AI and application frameworks, alongside SDKs for additional programming languages.

Its most substantial native contribution to retrieval-oriented applications is the ability to store documents, relationships and vectors together, then combine lexical matches, vector similarity, filters and graph traversal within the database.

External frameworks can provide orchestration, embedding generation, prompts, model execution and answer synthesis. SurrealDB supplies data persistence and retrieval.

This is a useful integration boundary: adopting a database with vector retrieval and graph traversal does not mean that it supplies embedding models, a complete retrieval-augmented generation pipeline or a learned reranking system.

Source: `https://surrealdb.com/docs/build/integrations/ai-frameworks`

## 14. Operational characteristics, durability and constraints

### 14.1 Observability

SurrealDB provides an HTTP `/health` endpoint and exposes runtime metrics for external monitoring.

Its observability tooling includes Prometheus-compatible metrics, OpenTelemetry integration, structured logging and configuration suitable for tracing query and database behavior.

The published monitoring documentation distinguishes Community and Enterprise availability. In particular, specialized audit and slow-query logging pipelines have Enterprise edition boundaries.

For performance analysis, the database's query-plan diagnostics are complementary to infrastructure metrics: a slow query may be dominated by index selection, traversal fan-out, sorting or result materialization, while process-level metrics show CPU, memory, disk and other resource pressures.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/manage/self-hosted/monitoring-and-observability`

### 14.2 Backups and recovery

`surreal db export` produces a logical SurrealQL representation of schema and current data, while `surreal db import` restores it.

Logical exports are portable across supported storage engines, but their portability is not equivalent to preserving every storage-engine-specific historical version, journal or internal state.

Physical or volume-level snapshots can provide another backup mechanism when obtained consistently and in accordance with the storage engine's requirements.

For an embedded database, copying an actively written data directory without a supported snapshot mechanism should not be treated as a reliable online backup.

SurrealDB's documentation recommends validating backups through actual restores and distinguishes logical exports from point-in-time recovery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: `https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery`

### 14.3 Licensing and deployment rights

SurrealDB's core database engine is source-available under the Business Source License 1.1, whereas SDKs, libraries and associated tooling may use Apache 2.0 or MIT.

The published license permits broad self-hosted, embedded and application use, including production applications, while restricting provision of a database service that allows third parties to create or manage their own database schemas or tables without an appropriate agreement.

The distinction matters when evaluating SurrealDB as infrastructure to embed within a distributed product or expose as a managed service. License obligations should be evaluated for the exact component and intended offering; using an Apache-licensed Rust SDK does not change the license governing the database engine it embeds.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB

+1



Source: `https://surrealdb.com/license`

## 15. Capability combinations and important architectural distinctions

The largest opportunities to reduce bespoke infrastructure arise from combining several native capabilities, rather than evaluating each feature independently.

| Problem family                       | Useful native combination                                   | Potentially avoidable custom machinery                                       | Conditions and limitations                                                        |
| ------------------------------------ | ----------------------------------------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Graph-rich domain model              | Record IDs + references + relations + ISO GQL               | Manual adjacency tables, repeated client joins, custom traversal logic       | Relationship cardinality, edge attributes and graph algorithm requirements matter |
| Hybrid retrieval                     | Full-text + HNSW/DISKANN + bitmap filters + graph traversal | Separate lexical/vector stores and some result-merging logic                 | Embedding generation and advanced reranking remain external                       |
| Reactive state                       | Transactions + events + live queries + changefeeds          | Polling loops, some state projections and change-capture adapters            | Live delivery is not durable event replay; async events are at least once         |
| Embedded application storage         | Rust SDK + SurrealMX/SurrealKV + native values              | Separate server, network client, custom serialization and persistence layers | Exclusive datastore ownership and backend-specific durability                     |
| Schema lifecycle                     | SurrealKit sync + rollouts + tests + embedded schema        | Bespoke migration runners, schema tracking, permission test harnesses        | Sync can prune; migration reversibility is limited                                |
| Historical inspection                | Version-enabled storage + temporal queries + changefeeds    | Some application-maintained historical state and audit reconstruction        | Backend support, history retention and backup semantics differ                    |
| Domain-specific database computation | SurrealQL functions + Surrealism                            | Selected application-side transformation services                            | Wasm extension system remains experimental                                        |
| Constrained data APIs                | `DEFINE API` + record authentication + permissions          | Some CRUD endpoints, basic authorization middleware                          | API versioning and external workflow needs may still require a service            |

These possibilities are conditional, not general claims that moving every subsystem into a database is optimal.

### 15.1 Multi-model integration is the central advantage

SurrealDB can model an entity as a document, reference it through native record identities, connect it through typed graph relations, index text and embeddings associated with it, and query across those structures within one transactional system.

That integration can reduce duplicated state, synchronization logic and interservice coordination.

The tradeoff is increased reliance on SurrealDB's query language, semantics, operational maturity and supported client tooling. A multimodel database is not automatically the fastest or most feature-complete implementation of every individual model.

### 15.2 The embedded-versus-server decision changes the system boundary

An embedded engine can remove an entire network and process boundary. It also ties the database's lifecycle, resource consumption and release compatibility more closely to the host application.

A server centralizes access and operational control, but reintroduces connection management, network latency and independent deployment concerns.

Importantly, embedding SurrealDB does not mean using RocksDB directly. The embedded configuration still includes the SurrealDB query and transactional layers.

### 15.3 Native history, migrations and backups solve different problems

Temporal versioning preserves eligible historical data states. Changefeeds expose incremental changes. SurrealKit provides schema evolution. Backups preserve recovery artifacts.

None of these mechanisms is a complete substitute for the others.

A system requiring reconstruction of arbitrary historical application state needs to define the retention, schema compatibility, data consistency and recovery guarantees it actually requires.

### 15.4 Version alignment matters more than package naming

SurrealDB 3.3, SurrealKit beta.6, SurrealKV 1.0 and SurrealMX 1.0 are separate versioned components.

The existence of a capability in one standalone engine does not prove its availability through the current database integration.

Likewise, a Rust SDK method may exist but behave differently across embedded execution, WebSocket, HTTP and gRPC.

For SurrealDB in October 2026, the particularly important boundaries are:

- SurrealDB 3.3.4 is the stable database and Rust SDK release.
- SurrealKit 1.0.0-beta.6 is the relevant prerelease schema-management tool, not 0.7.
- SurrealKV's standalone 1.0 improvements precede their planned full integration into SurrealDB 3.4.
- ISO GQL is enabled by default in 3.3, but is not equivalent to complete Cypher compatibility.
- Incremental Rust SDK row streaming requires a supporting execution transport.
- Files and Surrealism are experimental, not unqualified production-stable interfaces.

## 16. Primary-source reference map

The sections above contain capability-specific URLs. This condensed map identifies the principal reference families for deeper investigation, with all addresses included as plain text.

| Topic                            | Primary source                                                                      |
| -------------------------------- | ----------------------------------------------------------------------------------- |
| SurrealDB 3.3 releases           | `https://surrealdb.com/releases/3.3`                                                |
| Database repository              | `https://github.com/surrealdb/surrealdb`                                            |
| Rust SDK documentation           | `https://surrealdb.com/docs/reference/rust`                                         |
| Rust SDK API reference           | `https://docs.rs/surrealdb/3.3.4/surrealdb/`                                        |
| Rust value types                 | `https://surrealdb.com/docs/reference/rust/concepts/working-with-types`             |
| Embedded execution               | `https://surrealdb.com/docs/reference/rust/embedding`                               |
| SurrealQL reference              | `https://surrealdb.com/docs/reference/query-language`                               |
| Graph model                      | `https://surrealdb.com/docs/learn/data-models/graph`                                |
| ISO GQL                          | `https://surrealdb.com/docs/learn/querying/gql/overview`                            |
| GraphQL                          | `https://surrealdb.com/docs/learn/querying/graphql/overview`                        |
| Index definitions                | `https://surrealdb.com/docs/reference/query-language/statements/define/indexes`     |
| Vector search                    | `https://surrealdb.com/docs/learn/data-models/vector-search/vector-indexes`         |
| Full-text search                 | `https://surrealdb.com/docs/learn/data-models/full-text-search/search-indexes`      |
| Query plans                      | `https://surrealdb.com/docs/reference/query-language/statements/explain`            |
| Storage deployment models        | `https://surrealdb.com/docs/manage/self-hosted/deployment-models`                   |
| SurrealKV source                 | `https://github.com/surrealdb/surrealkv`                                            |
| SurrealMX source                 | `https://github.com/surrealdb/surrealmx`                                            |
| SurrealKit beta.6 release        | `https://github.com/surrealdb/surrealkit/releases/tag/v1.0.0-beta.6`                |
| SurrealKit tagged documentation  | `https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/README.md`    |
| SurrealKit 1.0 migration details | `https://raw.githubusercontent.com/surrealdb/surrealkit/v1.0.0-beta.6/MIGRATING.md` |
| SurrealKit schema rollouts       | `https://surrealdb.com/docs/manage/schema-migration/rollouts`                       |
| SurrealKit testing               | `https://surrealdb.com/docs/manage/schema-migration/testing`                        |
| Surrealism extensions            | `https://surrealdb.com/docs/learn/extensions/plugins/overview`                      |
| MCP integration                  | `https://surrealdb.com/docs/reference/cli/db/commands/mcp`                          |
| PostgreSQL wire compatibility    | `https://surrealdb.com/docs/reference/rest-api/postgres-protocol`                   |
| Monitoring and observability     | `https://surrealdb.com/docs/manage/self-hosted/monitoring-and-observability`        |
| Backups and recovery             | `https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery`                |
| Licensing                        | `https://surrealdb.com/license`                                                     |

### Closing perspective

SurrealDB is most distinctive when viewed as a unified execution and persistence environment, rather than as a document store with an additional graph query interface.

The capabilities that deserve particular attention are its native graph and record-reference model; the coexistence of SurrealQL and ISO GQL; indexed graph, lexical and vector queries; transactional and reactive state mechanisms; Rust-native embedded execution; and the increasingly comprehensive SurrealKit schema lifecycle.

The less obvious features—lightweight relations, inline adjacency caches, bitmap index combination, incremental SDK streaming, database events, temporal reads, schema-source parity tests, and beta.6's frozen migration artifacts—are especially important for understanding what the platform can provide without custom supporting infrastructure.

The major qualification is that SurrealDB 3.3.4 represents a particular point in a rapidly evolving architecture. Keeping core database, storage-engine, Rust SDK and SurrealKit version boundaries explicit is essential to obtaining the intended capabilities and guarantees.