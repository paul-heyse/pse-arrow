# SurrealQL — Comprehensive Technical Reference for Rust

Research date: November 10, 2026 | Target: SurrealDB 4.0.1 / Rust SDK 4.0.1

## 1. Overview and architectural positioning

SurrealQL is SurrealDB's native declarative query and database programming language. It extends SQL-style querying with document navigation, graph traversal, typed records, vector similarity, full-text search, procedural control flow, schema definitions, access control, and server-side functions.

The central distinction for Rust developers is that SurrealQL is a language, not a standalone Rust execution library. It is implemented within SurrealDB's Rust ecosystem and exposed primarily through the `surrealdb` SDK, embedded database engine, and internal parsing and expression crates.

The separately published `surrealql` crate is version `0.0.0` and does not provide the functional implementation discussed here.

This reference covers both the SurrealQL language and the Rust components through which it can be executed, analyzed, extended, or integrated.

Primary references:

- SurrealQL language: [https://surrealdb.com/docs/reference/query-language](https://surrealdb.com/docs/reference/query-language?utm_source=chatgpt.com)
- Rust SDK: [https://surrealdb.com/docs/reference/rust](https://surrealdb.com/docs/reference/rust?utm_source=chatgpt.com)
- Upstream repository: [https://github.com/surrealdb/surrealdb](https://github.com/surrealdb/surrealdb?utm_source=chatgpt.com)
- `surrealql` crate: [https://docs.rs/surrealql/latest/surrealql/](https://docs.rs/surrealql/latest/surrealql/?utm_source=chatgpt.com)

SurrealDB 4.0 became generally available on November 3, 2026, followed by version 4.0.1 on November 9. The 4.0 release is particularly consequential because it replaces multiple query-planning paths with a unified bytecode-based query executor and introduces substantial changes to query optimization, transactions, types, search, and extension support.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+2



### Capability map

| Area                 | Core capabilities                                                                                               |
| -------------------- | --------------------------------------------------------------------------------------------------------------- |
| Querying             | SQL-style selection, projections, grouping, ordering, subqueries, array processing, joins through relationships |
| Data modeling        | Documents, typed records, graph edges, embedded structures, sets, geometries, vectors, named types              |
| Graph operations     | Directed edge traversal, record links, back-references, multi-hop traversal                                     |
| Search               | Full-text indexing, vector similarity, HNSW, DiskANN, spatial indexing, hybrid retrieval                        |
| Database programming | Expressions, closures, functions, variables, loops, conditionals, transactions                                  |
| Schema and integrity | Schemafull/schemaflexible tables, typed fields, assertions, computed fields, indexes, references, events        |
| Security and APIs    | Row/field permissions, authentication, user-defined API endpoints, scoped execution                             |
| Extensibility        | Native built-in functions, JavaScript scripting, Rust-authored WebAssembly extensions                           |
| Execution            | Query planning, compiled execution, index selection, concurrent scans, memory spilling                          |
| Rust integration     | Embedded or remote execution, parameter binding, strongly typed values, query results, subscriptions            |
| Language tooling     | Parser, lexer, AST, formatter, diagnostics, schema analysis, language server                                    |

The most important architectural capability is composability: one SurrealQL operation can combine graph navigation, document projections, indexed search, data transformation, and permission enforcement without requiring separate database systems or application-side orchestration.

That consolidation can eliminate substantial data-access code, although its performance and suitability depend on the query plan, selected storage engine, transaction requirements, and the amount of computation placed inside the database.

## 2. Language model, values, and expressions

SurrealQL treats a query as an executable sequence of statements and expressions. Unlike conventional SQL, expressions can produce complex values, access related records, invoke functions, and form part of procedural blocks.

### 2.1 First-class value types

SurrealQL's type system is broader than JSON or traditional SQL.

| Type family       | Capabilities and distinctions                                            |
| ----------------- | ------------------------------------------------------------------------ |
| Scalars           | Strings, booleans, integers, floats, decimals, bigdecimals, bytes, UUIDs |
| Temporal          | Datetimes, durations, dates, times, timestamps, intervals                |
| Collections       | Arrays, ordered sets, heterogeneous objects, typed collections           |
| Identity          | Record IDs, typed record references, record ranges                       |
| Spatial           | Points, lines, polygons, and other geometry types                        |
| Executable values | Closures, futures, and expressions                                       |
| Specialized       | Regular expressions, file values, named and literal types                |
| Absence           | Distinct `NONE` and `NULL` representations                               |

These types can appear as literals, expression results, function arguments, return values, and persisted fields where supported.

Two distinctions are particularly consequential.

`NONE` versus `NULL`:

- `NONE` denotes absence. Writing a field as `NONE` removes it from the stored record.
- `NULL` represents an explicitly present field with a null value.
- `option<T>` permits `NONE` in a type definition; it is not equivalent to making every field nullable in the conventional SQL sense.

The distinction affects partial updates, defaults, indexes, parameter bindings, and Rust serialization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



Arrays versus sets:

- Arrays preserve sequence and can contain duplicates.
- Sets deduplicate automatically and maintain value ordering.
- Indexing into a set follows its sorted order, not insertion order.

A set is therefore useful for canonical membership collections, but it cannot substitute for an array when positional identity matters.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/sets](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/sets?utm_source=chatgpt.com)

### 2.2 Record IDs as structured keys

A SurrealDB record ID combines table identity with a key. The key need not be a simple string or integer.

Examples:

```
person:alice
measurement:['sensor-7', d'2026-11-10T12:00:00Z']
document:uuid()
```

Record IDs can use strings, integers, arrays, objects, UUIDs, or generated identifiers.

This creates an important native alternative to separate lookup indexes: structured record keys can encode meaningful lookup dimensions directly in the primary key space.

For example, composite IDs consisting of an entity identifier and timestamp support bounded key-range scans.

SurrealDB 4.0 introduces named components for array-based IDs:

```
DEFINE FIELD id ON reading
    TYPE [sensor: string, timestamp: datetime];
```

Those components become queryable, read-only fields, allowing appropriately constrained predicates to be planned as key-range scans.

This can reduce the need for secondary indexes where the access pattern aligns with the key's ordering. It is less suitable when records must support many unrelated access dimensions, because one key order cannot efficiently serve every query shape.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/record-ids](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/record-ids?utm_source=chatgpt.com)

### 2.3 Named types and constraints

New in SurrealDB 4.0

`DEFINE TYPE` enables reusable schema-level types.

A named type can represent a constrained scalar, structured object, or value union:

```
DEFINE TYPE confidence AS float
    CONSTRAINT valid_range
    ASSERT $value >= 0 AND $value <= 1;

DEFINE TYPE status AS
    'pending' | 'running' | 'complete' | 'failed';

DEFINE TYPE address AS {
    street: string,
    city: string,
    postcode: option<string>
};
```

Named types can be used by fields, function signatures, casts, and nested type expressions.

Important advanced behavior includes:

- Named constraints with identifiable failure messages.
- Defaults inherited from type definitions.
- Named unions whose string values have declaration-defined ordering.
- `ALTER TYPE` operations for modifying unions.
- Dependency checks when attempting to remove a referenced type.

A type definition applies runtime coercion and validation. It is not equivalent to a Rust compiler proving that arbitrary SurrealQL statements are type-correct before execution.

The language also supports explicitly sized numeric types, including `u8`, `u16`, and `f32`. Sized arithmetic rejects overflow rather than silently wrapping.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/named-types](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/named-types?utm_source=chatgpt.com)

### 2.4 Idioms: a composable data-navigation language

An idiom is SurrealQL's generalized path expression.

Idioms support far more than ordinary dotted field access:

| Mechanism                | Capability                                            |
| ------------------------ | ----------------------------------------------------- |
| `object.field`           | Nested object navigation                              |
| `array[0]`               | Positional access                                     |
| `array[$]`               | Last-element access                                   |
| `array[WHERE condition]` | Filter elements in place                              |
| `value.method()`         | Invoke supported value functions                      |
| `object.{a, b}`          | Structural projection                                 |
| `object.?.field`         | Optional navigation                                   |
| `->edge->record`         | Directed graph traversal                              |
| `<~table`                | Incoming tracked references                           |
| Recursive idioms         | Repeated navigation, collection, and path exploration |

These mechanisms compose directly within `SELECT`, `WHERE`, `RETURN`, field definitions, and certain indexes.

For example, a nested projection can preserve document structure without returning the entire stored object:

```
SELECT
    name,
    metadata.{source, confidence},
    observations[WHERE valid = true]
FROM entity;
```

This is useful because the database can produce an application-specific document shape directly, avoiding custom Rust transformations for many common cases.

#### Method chaining

Functions can frequently be called as methods:

```
RETURN ['  ALPHA  ', ' BETA ']
    .map(|$v| $v.trim().lowercase());
```

The method notation is an alternative dispatch syntax for supported built-in functions, not a general mechanism for invoking arbitrary Rust methods.

Optional idiom segments and nested projections make it possible to express conditional navigation without extensive application-side checks.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/idioms](https://surrealdb.com/docs/reference/query-language/language-primitives/idioms?utm_source=chatgpt.com)

### 2.5 Closures and higher-order collection operations

SurrealQL supports anonymous closures using `|...|` syntax.

They can be passed to higher-order functions, including collection mapping, filtering, folding, and related transformations.

```
LET $values = [1, 2, 3, 4, 5];

RETURN $values
    .filter(|$v| $v > 2)
    .map(|$v| $v * $v);
```

Closures support expression composition, typed parameters, and block bodies.

A consequential distinction introduced in SurrealDB 3.3 is that closures can perform writes when invoked in write-capable contexts. This means a collection transformation is not necessarily pure.

That enables sophisticated server-side workflows but requires care when evaluating side effects, repeated invocation, execution order, and transaction behavior.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/closures](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/closures?utm_source=chatgpt.com)

## 3. Native graph and relationship capabilities

SurrealQL offers three distinct relationship representations. They should not be treated as interchangeable.

| Relationship mechanism | Representation                                 | Appropriate use                                         |
| ---------------------- | ---------------------------------------------- | ------------------------------------------------------- |
| Record link            | A field containing another record's ID         | Direct known relationships                              |
| Record reference       | A tracked link with reverse lookup             | Bidirectional navigation and reference management       |
| Graph edge             | A relationship record connecting two endpoints | First-class relationships with attributes and direction |

### 3.1 First-class graph edges

`RELATE` creates directed edge records:

```
RELATE person:alice->authored->document:report
    SET created_at = time::now();
```

The relationship is itself a record and can carry attributes.

Traversal follows the edge direction:

```
SELECT
    *,
    ->authored->document AS documents
FROM person:alice;
```

Incoming and bidirectional traversal forms are also available.

Unlike application-side graph traversal over ordinary foreign keys, SurrealQL integrates traversal into its expression language. Graph relationships can participate in filtering, projection, aggregation, and further traversal.

The distinction is especially valuable when a workload combines graph structure with substantial document attributes.

### 3.2 Record links and tracked references

A conventional record link stores a typed record ID.

A tracked reference adds `REFERENCE` to a supported record-valued field definition:

```
DEFINE FIELD documents ON person
    TYPE option<array<record<document>>>
    REFERENCE;
```

The target can then discover incoming links using `<~` navigation.

This avoids maintaining a separate reverse-reference lookup manually. Reference definitions also provide mechanisms for controlling behavior when referenced records are deleted.

Tracked references have schema constraints: the documented `REFERENCE` declaration applies to top-level fields with supported record or record-array types, not arbitrary deeply nested fields.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/record-references](https://surrealdb.com/docs/reference/query-language/language-primitives/record-references?utm_source=chatgpt.com)

### 3.3 Recursive traversal and graph algorithms

One of SurrealQL's most easily overlooked capabilities is recursive idiom evaluation.

Rather than requiring the application to repeatedly submit traversals, a recursive idiom describes the repeated path operation and its depth bounds.

For example:

```
RETURN person:alice.{1..3}(->knows->person).name;
```

SurrealQL also provides specialized recursive path modes:

| Mode                      | Functionality                                   |
| ------------------------- | ----------------------------------------------- |
| `{..+path}`               | Collect traversed paths                         |
| `{..+collect}`            | Collect unique visited nodes                    |
| `{..+shortest=record:id}` | Find a shortest path to a target                |
| `+inclusive`              | Include the starting node in applicable results |

These are significant capabilities because they can replace relatively complex custom graph-walking loops.

However, they do not constitute a general-purpose graph analytics framework. Algorithms such as arbitrary weighted shortest paths, PageRank, community detection, and specialized graph optimization should not be assumed to be natively implemented merely because recursive traversal is available.

Recursive execution also has specific termination and value-shape rules. Traversal steps must produce records or arrays of records, and paths that encounter incompatible intermediate values can fail rather than silently skip them.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/idioms](https://surrealdb.com/docs/reference/query-language/language-primitives/idioms?utm_source=chatgpt.com)

### 3.4 Materialized reverse relationships

SurrealDB 4.0 additionally introduces `MATERIALIZE INTO` for maintained reverse-reference data.

Unlike a computed reverse-reference expression evaluated on demand, materialization maintains the reverse links on the target record, potentially making common reverse lookups a direct field read.

The tradeoff is additional write-path maintenance and storage.

This broadens the design space between conventional foreign keys, graph edges, dynamically resolved back-references, and denormalized relationship collections.

Reference: [https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)

## 4. Schema, integrity, and database-maintained structures

SurrealQL's schema layer is more than a declaration of field names and types. It defines write transformations, computed values, relationship behavior, indexes, permissions, events, and derived tables.

### 4.1 Schemafull and schemaless tables

`DEFINE TABLE` supports both `SCHEMAFULL` and `SCHEMALESS` modes.

In a schemafull table, fields must be explicitly defined. Since SurrealDB 3.0, writes containing unknown fields are rejected rather than silently discarding them.

A schemaless table permits additional fields, but fields that have explicit definitions still undergo their declared validation.

A relationship table can additionally specify `TYPE RELATION` and constrain its endpoint types.

```
DEFINE TABLE measurement SCHEMAFULL;

DEFINE FIELD instrument ON measurement
    TYPE record<instrument>;

DEFINE FIELD value ON measurement
    TYPE float
    ASSERT $value >= 0;

DEFINE FIELD observed_at ON measurement
    TYPE datetime;

DEFINE INDEX by_instrument_time
    ON measurement FIELDS instrument, observed_at;
```

This schema can replace separate application-level validation for the relevant stored invariants, but only if all writers use paths that enforce those definitions.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/table](https://surrealdb.com/docs/reference/query-language/statements/define/table?utm_source=chatgpt.com)

### 4.2 Defaults, transformations, assertions, and computed fields

`DEFINE FIELD` provides several distinct mechanisms.

| Mechanism        | Evaluation and behavior                                   |
| ---------------- | --------------------------------------------------------- |
| `TYPE`           | Coerces or checks field values against a declared type    |
| `DEFAULT`        | Provides a value on creation when absent                  |
| `DEFAULT ALWAYS` | Also fills an absent value on subsequent updates          |
| `VALUE`          | Computes a value during a record write                    |
| `ASSERT`         | Rejects writes violating a condition                      |
| `READONLY`       | Prevents ordinary direct modification                     |
| `COMPUTED`       | Evaluates an expression when the field is read            |
| `PERMISSIONS`    | Controls field-level reading and modification             |
| `REFERENCE`      | Maintains tracked incoming references                     |
| `INLINE`         | Copies selected edge properties into graph adjacency data |

Since 3.3, fields with dependencies on other fields are evaluated in dependency order rather than merely by field name.

`COMPUTED` deserves special attention: it is not a stored generated column. Its expression is evaluated on each read, inside that statement's transaction. Its execution cannot have database-changing side effects and cannot grant permissions beyond those of its caller or definer.

Conversely, `VALUE` may persist a derived value on the write path, trading storage and write work for simpler reads.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/field](https://surrealdb.com/docs/reference/query-language/statements/define/field?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 4.3 Schema evolution and built-in data migrations

SurrealDB 4.0

`MIGRATE EXISTING ROWS` introduces a native mechanism for changing the values already stored under a field definition:

```
DEFINE FIELD OVERWRITE stock ON product TYPE int
    MIGRATE EXISTING ROWS AS stock_to_int
    USING <int> $value FALLBACK 0;
```

Its behavior is unusually important:

- The named migration is recorded and applied once.
- Values are converted on first read, while a background job rewrites the table.
- `FALLBACK` handles values that cannot be converted or validated.
- Migration expressions are restricted to an approved, effect-free subset; arbitrary custom functions, scripts, and queries are not allowed.
- Fields participating in certain derived structures, indexes, or references cannot simply be migrated in place.

The database also distinguishes compatibility levels 3 and 4. At compatibility level 4, some schema-narrowing changes on populated schemafull tables require explicit migrations.

This creates a valuable alternative to custom Rust data-migration loops for supported field transformations. It is not a complete replacement for migration orchestration: schema dependencies, rollbacks, multi-table changes, and deployment coordination remain separate concerns.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/field](https://surrealdb.com/docs/reference/query-language/statements/define/field?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 4.4 Materialized table views

SurrealQL supports precomputed table views:

```
DEFINE TABLE readings_by_sensor AS
SELECT
    sensor,
    count() AS samples,
    math::mean(value) AS mean_value
FROM reading
GROUP BY sensor;
```

These are event-maintained, materialized results rather than conventional virtual SQL views executed from scratch on each read.

Consequently, they can serve as a native mechanism for maintaining derived aggregate records. Their maintenance has write-side costs, and the definition syntax is substantially narrower than arbitrary `SELECT` syntax.

Complex transformations, arbitrary joins, or independently scheduled refresh procedures should not be assumed to fit the view mechanism.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/table](https://surrealdb.com/docs/reference/query-language/statements/define/table?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 4.5 Lightweight graph edges and adjacency optimizations

Introduced in SurrealDB 3.3, available in 4.0

For relationships that do not need properties, `LIGHTWEIGHT` provides a recordless edge representation:

```
DEFINE TABLE follows
    TYPE RELATION IN person OUT person
    LIGHTWEIGHT;
```

A lightweight edge's identity is its endpoint pair. Repeated `RELATE` operations for the same pair are therefore idempotent.

The storage engine synthesizes the normal-looking edge representation when it is read, but there is no independent edge record to hold properties.

That has important constraints: no edge fields, indexes, events, mutable properties, changefeeds, or live subscriptions on the lightweight relation table.

For conventional property-bearing edges, other optimizations are available:

- `INLINE EDGES n` caches small adjacency lists alongside source records.
- `INLINE REFERENCES n` similarly optimizes incoming tracked references.
- `INLINE` on a relation field can duplicate a small edge property into adjacency data, allowing supported traversal predicates to evaluate without reading the complete edge record.

These are especially consequential in high-degree or high-volume graph traversal workloads. Their purpose is to reduce key-value lookups, not to change graph semantics.

The cost is additional data duplication and write-path maintenance. Large values or predicates written outside the optimizable traversal form may not benefit.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/table](https://surrealdb.com/docs/reference/query-language/statements/define/table?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 5. Query composition and data manipulation

### 5.1 Selection is not limited to tables

A SurrealQL `SELECT` can consume:

- Tables or individual record IDs.
- Multiple input targets.
- Graph traversals.
- Subqueries.
- Parameters containing records or collections.
- Arrays, objects, and other expression values.

Its result shaping includes `VALUE`, nested projections, `OMIT`, `SPLIT`, `GROUP`, `ORDER`, `FETCH`, `LIMIT`, and `START`.

Several of these have meanings that differ from traditional SQL.

`SELECT VALUE` returns the selected value directly rather than an object containing the named column.

`FETCH` expands record links into fetched records in the result.

`SPLIT` expands values from an array field into separate output rows.

`GROUP ALL` performs aggregation over the entire result rather than grouping by a key.

`ONLY` requests a single-record result shape, with corresponding constraints on the source.

Because these mechanisms compose with graph idioms and nested objects, SurrealQL can often return exactly the nested representation required by a consumer.

### 5.2 Native record mutation

SurrealQL supports `CREATE`, `INSERT`, `UPDATE`, `UPSERT`, `DELETE`, and `RELATE`, with different semantics.

`CREATE` creates records and ordinarily reports a collision when a specified ID already exists.

`INSERT` supports bulk insertion and conflict-handling forms such as `ON DUPLICATE KEY UPDATE`.

`UPDATE` modifies existing targets; `UPSERT` can create a target if needed.

`RELATE` creates directed graph relationships.

Mutation statements offer different ways to specify changed data, including `SET`, `CONTENT`, `MERGE`, and `PATCH`, depending on the statement.

Result clauses such as `RETURN BEFORE`, `RETURN AFTER`, `RETURN DIFF`, and `RETURN NONE` allow the database to control how much changed state is returned.

These return forms matter for performance: suppressing a large mutation result avoids unnecessary serialization and network transfer.

Primary references:

- [https://surrealdb.com/docs/reference/query-language/statements/create](https://surrealdb.com/docs/reference/query-language/statements/create?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/reference/query-language/statements/update](https://surrealdb.com/docs/reference/query-language/statements/update?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/reference/query-language/statements/upsert](https://surrealdb.com/docs/reference/query-language/statements/upsert?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/reference/query-language/statements/insert](https://surrealdb.com/docs/reference/query-language/statements/insert?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/reference/query-language/statements/relate](https://surrealdb.com/docs/reference/query-language/statements/relate?utm_source=chatgpt.com)

### 5.3 Query execution and optimization controls

SurrealQL exposes several controls that materially affect execution:

| Mechanism                     | Purpose                                             |
| ----------------------------- | --------------------------------------------------- |
| `WITH INDEX`                  | Restrict the planner to specified indexes           |
| `WITH NOINDEX`                | Force a table scan                                  |
| `TIMEOUT`                     | Bound statement execution time                      |
| `FOR UPDATE`                  | Request update-oriented read behavior               |
| `TEMPFILES`                   | Explicitly use temporary files for large operations |
| `EXPLAIN`                     | Inspect planned query operators                     |
| `EXPLAIN FULL`                | Execute and include operator metrics                |
| `EXPLAIN ANALYZE FORMAT JSON` | Obtain structured execution and timing information  |

The final form is the standalone `EXPLAIN` statement introduced in 3.0; the older appended `EXPLAIN [FULL]` clause remains documented.

Execution-plan output is explicitly described as informational and subject to change. Tooling should not treat its current JSON operator structure as a stable serialization contract.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/explain](https://surrealdb.com/docs/reference/query-language/statements/explain?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 5.4 Unified bytecode execution in 4.0

SurrealDB 4.0 consolidates previously separate execution strategies into a single query executor that plans statements, compiles them into bytecode, and caches compiled plans.

Other relevant changes include:

- Integrating table-level selection permissions into query planning.
- Executing uncorrelated subqueries once instead of repeatedly per input record.
- Turning appropriate `id IN [...]` expressions into direct key lookups.
- Parallelizing qualifying table scans outside browser builds.
- Spilling large sorts, joins, distinct operations, and grouped aggregates to disk.
- Enforcing memory budgets through configuration such as `SURREAL_MEMORY_LIMIT`.

This is a major architectural change in the database engine, not a Rust API for compiling arbitrary application code into SurrealQL bytecode.

A significant behavioral consequence is that result ordering without `ORDER BY` is unspecified. `LIMIT` without an explicit order must not be treated as stable pagination.

Reference: [https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+2



## 6. Indexes, search, and retrieval

SurrealQL exposes multiple specialized index families through one declarative schema language.

| Index family       | Principal functionality                            | Important tradeoff                                |
| ------------------ | -------------------------------------------------- | ------------------------------------------------- |
| Standard/composite | Equality, range, ordered access                    | Key ordering governs which predicates benefit     |
| Unique             | Enforce key uniqueness                             | Uniqueness semantics for `NULL` and `NONE` differ |
| Count              | Maintain a table count or condition-specific count | Incremental maintenance on writes                 |
| Full-text          | Tokenized text search, BM25 scoring, highlights    | Requires analyzer configuration                   |
| HNSW               | Approximate nearest-neighbor vector retrieval      | Graph primarily resident in memory                |
| DiskANN            | Disk-oriented approximate vector retrieval         | More storage I/O, different index parameters      |
| Spatial            | Indexed geometric predicates                       | Depends on geometry and supported predicate forms |

### 6.1 Composite, expression, and multivalue indexes

Ordinary indexes support composite key definitions, and the planner can take advantage of leading-key equality or range conditions.

SurrealQL also permits indexed expressions and expansion of array elements, such as `tags.*`.

This supports more sophisticated access patterns than simple one-field indexes, but the exact predicate matters. A query using an operator that is incompatible with an index does not gain an index scan merely because that index was named in `WITH INDEX`.

From 4.0, a unique index can specify `UNIQUE NULLS NOT DISTINCT`, making explicit `NULL` values participate in uniqueness. `NONE` remains a separate absence case.

### 6.2 Incremental count indexes

`COUNT` and `COUNT WHERE condition` indexes support counts without repeatedly materializing and scanning entire records.

A conditional count index is usable when the query condition matches its declared condition appropriately.

SurrealDB also has a fast `CountScan` path that counts storage keys, so defining a count index is not automatically necessary for every count query.

### 6.3 Full-text search

`DEFINE ANALYZER` controls text processing, including tokenization and filters.

A full-text index may be defined using:

```
DEFINE ANALYZER text_analyzer
    TOKENIZERS blank
    FILTERS lowercase;

DEFINE INDEX document_text
    ON document FIELDS content
    FULLTEXT ANALYZER text_analyzer
    BM25;
```

The `@@` operator performs indexed text matching, and search functions expose relevance scores and highlighting facilities.

The engine can therefore handle tokenization, inverted-index retrieval, scoring, and result combination natively.

SurrealDB 4.0 improves ranked retrieval with blocked postings and pruning of candidates that cannot enter the highest-scoring results.

That is a full-text retrieval mechanism, not a semantic embedding model or neural reranker.

### 6.4 Vector search: HNSW, DiskANN, and exact distance

SurrealQL supports approximate vector search through HNSW and DiskANN indexes, alongside exact distance computations.

An illustrative index definition is:

```
DEFINE INDEX embedding_hnsw
    ON document FIELDS embedding
    HNSW DIMENSION 768
    TYPE F32
    DIST COSINE;
```

The language supports nearest-neighbor operators and vector distance functions.

Version 4.0 can also recognize supported nearest-neighbor queries of the form:

```
SELECT id
FROM document
ORDER BY vector::distance::cosine(embedding, $query_vector)
LIMIT 10;
```

and execute them through an appropriate vector index rather than necessarily evaluating every stored vector.

The architectural distinctions are important:

- Exact search evaluates vector distances over the eligible corpus; it can provide exact ranking but may be expensive.
- HNSW maintains an approximate-neighbor graph in memory, subject to memory pressure and search/build tuning.
- DiskANN persists the graph in the key-value engine and uses bounded caching, better suited to a graph larger than available memory.
- Metadata prefiltering may reduce the candidate set searched by the vector index. Postfiltering after an approximate top-k search has different recall and result-count behavior.
- Approximate indexing does not promise exact nearest-neighbor recall. Correctness of index maintenance is distinct from the approximate ranking guarantee.

SurrealDB 4.0 adds SIMD-based HNSW distance evaluation and quantized DiskANN candidate search. DiskANN is also supported in WebAssembly browser builds in 4.0, superseding older documentation that restricted it to native/server environments.

### 6.5 Spatial indexing

The `SPATIAL` index introduced in 4.0 supports geometric predicates such as containment and intersection, as well as appropriate distance-bound queries.

Its integration with vector prefiltering is especially interesting: a geometric eligibility condition can narrow the vector search space without requiring the application to manually calculate all eligible record IDs.

Reference for the index families and their detailed constraints:

[https://surrealdb.com/docs/reference/query-language/statements/define/indexes](https://surrealdb.com/docs/reference/query-language/statements/define/indexes?utm_source=chatgpt.com)

4.0 execution changes:

[https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 7. Procedural programming and reusable database logic

SurrealQL is not restricted to declarative data retrieval. Its procedural facilities allow a query to perform conditional operations, maintain local values, and execute a sequence of database mutations.

### 7.1 Control flow and scope

The language supports:

- `LET` for locally scoped variables, including typed declarations.
- `IF`, `ELSE IF`, and `ELSE` for conditional evaluation.
- `FOR` loops over collections.
- `BREAK` and `CONTINUE` within loops.
- `RETURN` for producing values from blocks and functions.
- `THROW` for explicit failure and transaction cancellation.
- Anonymous closures for higher-order operations.

A SurrealQL block can perform several operations while remaining part of the enclosing statement and transaction.

This allows some multi-step database operations to be expressed within one server-side execution rather than requiring successive Rust requests.

However, SurrealQL is not intended to replace a general-purpose Rust runtime. It does not provide Rust's ownership model, trait system, type inference, arbitrary crates, or native access to the host process.

### 7.2 The built-in function ecosystem

SurrealQL contains a substantial set of database-native functions. The most relevant families are:

| Family                               | Capabilities                                                                |
| ------------------------------------ | --------------------------------------------------------------------------- |
| `array::`, `set::`                   | Transformations, filtering, aggregation, set operations                     |
| `object::`, `value::`                | Document manipulation, object inspection, value operations                  |
| `string::`, `parse::`                | Normalization, tokenization, parsing, formatting                            |
| `math::`                             | Aggregation, statistical and mathematical operations                        |
| `time::`, `duration::`, `interval::` | Temporal construction, transformations, calculations                        |
| `record::`                           | Record identity and existence operations                                    |
| `vector::`                           | Vector arithmetic, similarity and distance measures                         |
| `geo::`                              | Geometric and geographic calculations                                       |
| `search::`                           | Indexed full-text scoring and highlighting                                  |
| `crypto::`, `encoding::`             | Hashing, password-related operations, encodings                             |
| `type::`                             | Type conversion, dynamic table/record construction, dynamic field selection |
| `sequence::`                         | Database-managed sequence values                                            |
| `file::`                             | Bucket-backed file operations                                               |
| `http::`                             | Outbound HTTP operations, subject to capabilities                           |
| `api::`                              | Invocation of defined API endpoints                                         |
| `rand::`, `session::`, `enum::`      | Random values, session context, named-enum operations                       |

The function catalog matters architecturally because it determines how much computation can be pushed into the database without implementing a custom extension.

Nevertheless, the presence of mathematical and vector operations does not make SurrealQL a general symbolic algebra system, GPU numerical framework, or mathematical optimization solver.

Reference: [https://surrealdb.com/docs/reference/query-language](https://surrealdb.com/docs/reference/query-language?utm_source=chatgpt.com)

### 7.3 User-defined SurrealQL functions

`DEFINE FUNCTION` creates reusable functions with typed arguments and optional return types.

```
DEFINE FUNCTION fn::is_high_confidence(
    $confidence: float,
    $threshold: float
) -> bool {
    RETURN $confidence >= $threshold;
};
```

Functions may contain procedural statements and database queries. They can be called from other statements, functions, and supported schema expressions.

Crucially, function permissions govern the ability to call a function, not the privilege level of its body. Its body executes with the caller's permissions.

Since 4.0, effect restrictions on schema expressions also follow calls into user-defined functions. A function that performs a write cannot be used to smuggle that write into an `ASSERT`, `COMPUTED`, or permission predicate that prohibits side effects.

This makes functions useful for centralizing reusable database logic while preserving the execution context's authorization boundary.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/function](https://surrealdb.com/docs/reference/query-language/statements/define/function?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 8. Transactions, consistency, and time

### 8.1 Transaction composition

Each SurrealQL statement normally executes in its own transaction. Explicit transactions group several statements:

```
BEGIN TRANSACTION;

UPDATE account:source SET balance -= $amount;
UPDATE account:destination SET balance += $amount;

COMMIT TRANSACTION;
```

On failure, database modifications in the transaction are rolled back.

An important syntax limitation is that `BEGIN`, `COMMIT`, and `CANCEL` take effect only as top-level query statements. Placing them inside an `IF`, `FOR`, ordinary block, or function body does not create a nested transaction; those statements are ignored in that context.

`THROW` can instead abort the enclosing transaction when a condition fails.

There is no general savepoint mechanism implied by SurrealQL's support for nested blocks.

Reference: [https://surrealdb.com/docs/reference/query-language/language-primitives/transactions](https://surrealdb.com/docs/reference/query-language/language-primitives/transactions?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 8.2 Isolation levels

SurrealDB 4.0 provides explicit transaction-isolation selection.

| Mode                    | Guarantee and availability                                 |
| ----------------------- | ---------------------------------------------------------- |
| Snapshot isolation      | Default transaction behavior                               |
| Serializable isolation  | Supported by the in-memory engine and Enterprise SurrealDS |
| Read-only transactions  | Explicitly reject writes                                   |
| Bounded-staleness reads | Available through published snapshots on SurrealDS         |

The storage-engine distinction is fundamental: SurrealKV does not support the new serializable isolation mode in 4.0.

For an invariant involving several records, snapshot isolation alone may permit write skew. Such an invariant must instead be protected through a suitable uniqueness constraint, transactional write conflict, serialized execution, or an engine offering the required isolation.

A failed commit can also return an unknown outcome, meaning the client cannot tell whether the database applied the changes. Such an error is not automatically safe to retry.

Reference: [https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



### 8.3 Historical queries and versioned storage

`SELECT ... VERSION` allows reads against a historical database state when the underlying storage engine is configured to preserve the required versions.

SurrealKV supports time-travel querying when its versioning functionality is enabled. SurrealDS also supports historical reads within its own storage semantics.

This should be distinguished from application-level event sourcing:

- A versioned datastore can answer historical state queries within its retention and storage guarantees.
- A changefeed exposes a sequence of data changes.
- An application event log records business events with application-defined meaning.

These structures can complement one another, but one does not automatically provide all guarantees of the others.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/select](https://surrealdb.com/docs/reference/query-language/statements/select?utm_source=chatgpt.com)

## 9. Events, subscriptions, and change data capture

SurrealQL provides three different mechanisms for reacting to database changes.

| Mechanism                             | Execution model                                       | Appropriate role                               |
| ------------------------------------- | ----------------------------------------------------- | ---------------------------------------------- |
| Synchronous `DEFINE EVENT`            | Runs in the originating transaction                   | Enforce or maintain transactional side effects |
| Asynchronous `DEFINE EVENT ... ASYNC` | Queued transactionally and later executed by a worker | Deferred database processing                   |
| `LIVE SELECT`                         | Session-bound notifications to connected clients      | Real-time subscriptions                        |
| `CHANGEFEED` / `SHOW CHANGES`         | Retained sequence of mutations                        | Replayable change synchronization              |

### 9.1 Transactional events

A table event can execute on create, update, or delete, with access to `$before`, `$after`, and `$event`.

Synchronous events run within the transaction that performs the originating write. An event failure can therefore cause that write to roll back.

This makes events capable of maintaining dependent records or enforcing certain workflows without requiring an application request to orchestrate every dependent write.

### 9.2 Asynchronous events and at-least-once processing

With `ASYNC`, the event is enqueued atomically with the originating write, but it executes later in a separate transaction.

Documented behavior includes:

- A worker polls for pending events; the default processing interval is five seconds.
- Delivery is at least once, not exactly once.
- Conflicts and retries can result in repeated execution.
- `RETRY` controls retry behavior.
- `MAXDEPTH` limits recursively triggered event chains; its default is 3 and supported range is 0–16.
- External side effects, such as HTTP requests, are not undone by database rollback.

These semantics support durable deferred processing, but non-idempotent external effects require additional safeguards.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/event](https://surrealdb.com/docs/reference/query-language/statements/define/event?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 9.3 Live queries

`LIVE SELECT` subscribes to changes in records matching a query and pushes notifications containing selected data or differences.

The subscription has a lifecycle: its owning session must remain valid, and it may be terminated using `KILL`.

Live queries are not equivalent to an independently replayable change log. In Community Edition, notifications are local to the node holding the subscription. Distributed live queries across cluster nodes are an Enterprise capability.

Additionally, an error when evaluating a live query's filter or projection may skip the notification without rolling back the write that caused it.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/live-select](https://surrealdb.com/docs/reference/query-language/statements/live-select?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 9.4 Changefeeds

A table or database can enable change tracking using a `CHANGEFEED` definition.

`SHOW CHANGES` retrieves changes by versionstamp or timestamp, subject to the configured retention interval.

`INCLUDE ORIGINAL` enriches certain mutations with information about the previous record state. For updates, the documented representation can include reverse differences.

Changefeeds provide a more appropriate foundation than live subscriptions for consumers that need to resume from a checkpoint after disconnection.

However, retention is finite unless otherwise managed, and permission requirements differ from ordinary record reads: the documented `SHOW CHANGES` statement requires a system user with sufficient database-level privileges.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/show](https://surrealdb.com/docs/reference/query-language/statements/show?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



## 10. Authentication, permissions, and API endpoints

### 10.1 Two different authorization levels

SurrealDB distinguishes system identities from record users.

System users operate at root, namespace, or database scope with roles such as Owner, Editor, and Viewer.

Record users represent application identities. They are the principal type to which table- and field-level permissions apply.

A database can define authentication using `DEFINE ACCESS`, including:

- Record signup and signin expressions.
- JWT verification with keys or remote JWKS.
- Bearer access grants.
- Refresh-token issuance and revocation.
- Authentication-time expressions.
- Session context derived using `CONTEXT`, available since 3.3.

Permission predicates can inspect context such as `$auth`, `$token`, and `$session.data`.

A consequential distinction is that a database-level JWT system user is not the same as a JWT-authenticated record user. Giving application clients system-user credentials may bypass the fine-grained record permissions that were intended to restrict their access.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/access](https://surrealdb.com/docs/reference/query-language/statements/define/access?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



### 10.2 Fine-grained authorization

Table permissions can be expressed separately for selecting, creating, updating, and deleting data. Fields can have additional permissions controlling visibility and mutation.

This allows authorization predicates to incorporate graph relationships, record attributes, or membership checks.

In SurrealDB 4.0, table-selection permission predicates participate directly in query planning, making it possible for the optimizer to handle some permission subqueries as joins or shared computations.

Such predicates remain part of the query's execution cost. The execution model reduces some overhead; it does not make arbitrarily complicated access-control expressions free.

### 10.3 `DEFINE API`: database-hosted HTTP handlers

Available since 3.0, `DEFINE API` creates custom HTTP endpoints whose handlers are defined in SurrealQL.

An endpoint can provide HTTP methods, middleware, route parameters, request-body access, and structured responses containing status, headers, and a body.

The language also supports dynamic and typed route segments, route matching by specificity, and invocation through `api::invoke`.

This can replace a small external API layer when endpoint logic is predominantly database-oriented.

Security semantics are critical.

| Mode               | Execution behavior                                                                        |
| ------------------ | ----------------------------------------------------------------------------------------- |
| `SECURITY DEFINER` | Default. Handler may access data without the caller's table/field permission restrictions |
| `SECURITY INVOKER` | Introduced in 4.0. Handler operates under the caller's permissions                        |

An endpoint's own `PERMISSIONS` clause determines whether it may be called. If omitted, the documented default is `FULL`, potentially allowing unauthenticated callers.

Because the default handler mode is privileged relative to record permissions, custom endpoints must not be assumed safe merely because the underlying tables have restrictive permissions.

This is a substantial architectural feature, but it shifts the security boundary from application middleware into database definitions.

Reference: [https://surrealdb.com/docs/reference/query-language/statements/define/api](https://surrealdb.com/docs/reference/query-language/statements/define/api?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



## 11. Extending SurrealQL with Rust and other execution mechanisms

There are three materially different forms of extending SurrealQL behavior.

| Mechanism                     | Implementation               | Where it executes                | Typical use                                                     |
| ----------------------------- | ---------------------------- | -------------------------------- | --------------------------------------------------------------- |
| `DEFINE FUNCTION`             | SurrealQL                    | Native query engine              | Reusable database expressions and workflows                     |
| Embedded `function() { ... }` | JavaScript                   | Database scripting runtime       | Logic not readily expressible with native functions             |
| Surrealism extension          | Rust compiled to WebAssembly | Sandboxed WASM component runtime | Custom algorithms and reusable Rust-authored database functions |

### 11.1 Embedded JavaScript

SurrealQL supports embedded JavaScript functions, including ES2020 facilities, when the server permits scripting.

The documented server option is `--allow-scripting`.

The JavaScript runtime converts supported SurrealDB values into JavaScript representations and back. That conversion boundary can matter for identity, numerical precision, and specialized database types.

Scripting is also subject to the server's capability and security configuration. It should not be assumed to grant arbitrary operating-system access.

Reference: [https://surrealdb.com/docs/reference/query-language/scripting/overview](https://surrealdb.com/docs/reference/query-language/scripting/overview?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 11.2 Surrealism: Rust-authored WebAssembly extensions

Surrealism allows Rust functions to be made callable from SurrealQL without modifying and recompiling the database engine.

The Rust code is built for the `wasm32-wasip2` target using the WebAssembly component model and packaged into a `.surli` archive.

A simple exported Rust function looks like:

```
use surrealism::surrealism;

#[surrealism]
fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

Once packaged, installed, and defined as an extension, it becomes callable through an `ext::` namespace, such as:

```
RETURN ext::calculator::add(20, 22);
```

This provides a direct route to using selected Rust algorithms as database-native functions from the caller's perspective.

The important qualification is that the Rust is not running as native machine code linked into the database process. It runs inside a WASM sandbox.

The extension interacts with the database through exposed host interfaces and a transaction handle rather than direct access to SurrealDB's internal Rust objects.

### 11.3 Extension lifecycle, state, and permissions

SurrealDB 4.0 significantly changes the extension model:

- `DEFINE MODULE` and `mod::` are replaced by `DEFINE EXTENSION` and `ext::`; the old syntax remains as a deprecated alias.
- Archives must declare a signer or explicitly be marked unsigned.
- Definitions can pin an archive digest.
- `CAPABILITIES` restrict host access, permitted operations, execution time, and memory.
- Each function call creates a fresh WASM instance. Ordinary in-memory state does not survive between calls.
- A build-time initialization snapshot can be reused to avoid repeating certain setup work.
- An extension may use the current query's transaction handle, so its database writes participate in the enclosing transaction.

This architecture can remove client-to-server round trips for custom computation that must interact tightly with database state.

It also creates nontrivial costs: WASM instantiation, ABI conversion, capability checks, memory limits, and isolation overhead. There is no general guarantee that invoking a Rust extension is faster than running comparable code in a native Rust application.

Importantly, the Surrealism capability remains experimental in the 4.0 documentation and must be enabled explicitly, for example using `--allow-experimental surrealism`.

Extensions are also barred from schema clauses requiring proven absence of side effects, including `COMPUTED`, `VALUE`, `DEFAULT`, `ASSERT`, and permission clauses.

References:

- [https://surrealdb.com/docs/reference/query-language/statements/define/extension](https://surrealdb.com/docs/reference/query-language/statements/define/extension?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/learn/extensions/guides/extension-architecture](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture?utm_source=chatgpt.com)&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

  Database+2



### 11.4 Native data and file capabilities

SurrealDB's 4.0 file and bucket subsystem is stable rather than experimental.

Through bucket definitions and `file::` functions, SurrealQL can reference files, perform supported storage operations, and generate presigned access URLs.

Buckets can use configured storage backends and have explicit root, namespace, or database scope.

This provides an additional integration point for documents and large binary content, but file storage and relational/document records have different data models and lifecycle obligations. A file object is not interchangeable with a transactional record field merely because both are reachable through SurrealQL.

Reference: [https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



## 12. Rust SDK: executing and integrating SurrealQL

The primary supported Rust interface is the `surrealdb` crate, currently version 4.0.1. The SDK requires Rust 1.95 or later and supports both remote and embedded database connections.

It provides two overlapping programming interfaces:

1. SurrealQL execution: submit arbitrary statements using `.query()`, with bound parameters and typed results.
2. Rust-native database operations: use methods such as `.select()`, `.create()`, `.insert()`, `.update()`, `.upsert()`, `.delete()`, and `.begin()` without manually assembling a SurrealQL statement for every operation.

The Rust-native methods are not a separate query engine. They use SurrealDB's database semantics and underlying execution infrastructure.

### 12.1 Embedded and remote execution

| Connection mode | Rust integration            | Operational distinction                                        |
| --------------- | --------------------------- | -------------------------------------------------------------- |
| In-memory       | Embedded `Mem` engine       | No separate database service; data is transient                |
| SurrealKV       | Embedded `SurrealKv` engine | Persistent, local on-disk storage                              |
| WebSocket       | Remote engine               | Persistent connection, subscriptions, request/response queries |
| HTTP            | Remote engine               | Request-based remote access                                    |
| gRPC            | Remote engine               | Remote queries with additional streaming capabilities          |

Relevant Cargo feature selections include `kv-mem` and `kv-surrealkv`.

```
cargo add surrealdb --features kv-mem
```

Or:

```
cargo add surrealdb --features kv-surrealkv
```

SurrealDB 4.0 removed the previously supported embedded RocksDB and IndexedDB feature selections. SurrealKV is the principal persistent embedded engine.

The same SurrealQL language is available across these modes, but their concurrency, durability, streaming, and transaction capabilities are not identical.

Reference: [https://surrealdb.com/docs/reference/rust/embedding](https://surrealdb.com/docs/reference/rust/embedding?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 12.2 Parameterized queries and typed results

An illustrative Rust 4.0 SDK example:

```
use surrealdb::engine::any::connect;
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, SurrealValue)]
struct Person {
    id: RecordId,
    name: String,
}

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    let db = connect("memory").await?;
    db.use_ns("example").use_db("example").await?;

    let mut result = db
        .query("SELECT id, name FROM person WHERE age >= $minimum")
        .bind(("minimum", 18))
        .await?;

    result.check()?;

    let people: Vec<Person> = result.take(0)?;

    println!("{people:?}");
    Ok(())
}
```

The important boundary is `SurrealValue`, which converts values between Rust and SurrealDB's type system.

Parameterized values preserve richer database types, including record IDs, rather than requiring everything to pass through JSON strings.

A query can also accept multiple bindings using the `vars!` macro or a Rust struct implementing the appropriate conversion traits.

Parameter binding has a significant failure mode: an unbound parameter resolves to `NONE` rather than necessarily causing an error. Extra unused bindings are ignored, and later bindings with the same name take precedence.

As a result, parameter binding protects value construction from string interpolation, but it does not provide compile-time verification of parameter names.

Reference: [https://surrealdb.com/docs/reference/rust/methods/query](https://surrealdb.com/docs/reference/rust/methods/query?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 12.3 `SurrealValue`, record identity, and typed conversions

The shared `surrealdb-types` crate exposes the public value model and is re-exported through `surrealdb::types`.

The `SurrealValue` trait provides conversion between Rust types and SurrealDB values. It supports derived and manually implemented conversions.

Advanced features of the derive include:

- Field renaming and naming conventions.
- Flattening nested structures.
- Tagged and untagged enum representations.
- Handling of missing fields through defaults.
- Wrapping selected Serde-compatible types.
- Representation choices for tuple structures and enum payloads.

The attribute namespace is `#[surreal(...)]`, not `#[serde(...)]`. Deriving `SurrealValue` does not automatically honor Serde's field annotations.

This matters when converting an existing Rust domain model: serialization rules intended for JSON may not produce the same shape in SurrealDB.

`RecordId` should be retained as a typed value where identity semantics matter. Converting a database record ID to a plain JSON string and later reinterpreting it can introduce unnecessary parsing and ambiguity.

The SDK does not automatically generate Rust structs from live table schemas. That remains a distinct concern from serialization.

Reference: [https://surrealdb.com/docs/reference/rust/concepts/working-with-types](https://surrealdb.com/docs/reference/rust/concepts/working-with-types?utm_source=chatgpt.com)

Reference: [https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes](https://surrealdb.com/docs/reference/rust/concepts/surrealvalue-attributes?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 12.4 Multi-statement results and structured errors

`.query()` can submit multiple statements together. Its result is an `IndexedResults` structure, with each statement occupying its own result position.

A particularly important distinction is that a successful database request does not imply that every statement succeeded.

```
let mut response = db.query(
    "CREATE person:one SET age = 30;
     SELECT * FROM person;"
).await?;

response.check()?;

let people: Vec<Person> = response.take(1)?;
```

The call may return `Ok(...)` even when individual statements failed during execution. The errors must be inspected using `.check()`, `.take_errors()`, or the relevant result extraction operation.

The SDK's structured error kinds and codes are a better foundation for programmatic error handling than matching free-form error-message strings.

Reference: [https://surrealdb.com/docs/reference/rust/concepts/error-handling](https://surrealdb.com/docs/reference/rust/concepts/error-handling?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 12.5 Result streaming: a particularly valuable advanced feature

SurrealDB's Rust SDK distinguishes materialized query results from two kinds of streaming.

| Mechanism            | Behavior                                                     |
| -------------------- | ------------------------------------------------------------ |
| `.query(...).await?` | Waits for completion and materializes results                |
| `.stream()`          | Subscribes to notifications from a live query                |
| `.stream_items()`    | Streams ordinary query rows and statement-completion records |

`stream_items()` is especially interesting for large result sets.

It yields `StreamItem::Row` values followed by `StreamItem::StatementEnd`, which reports completion, statistics, and any error.

However, its incremental behavior depends on the engine:

- Embedded memory and SurrealKV engines support incremental delivery.
- The gRPC remote engine supports incremental delivery.
- WebSocket and HTTP clients currently complete the query before replaying the stream items.

There is also an important transactional caveat: a streamed row is provisional until the statement successfully completes. A later failure may invalidate rows already received. Consumers that perform external actions as each row arrives must account for this possibility.

Dropping an active stream stops its underlying query.

This is not merely a memory optimization. It changes when consumers can act on a result and what guarantees they have before completion.

Reference: [https://surrealdb.com/docs/reference/rust/methods/query](https://surrealdb.com/docs/reference/rust/methods/query?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 12.6 Rust-managed transactions

The SDK's `.begin()` API produces a transaction-scoped handle with familiar query and CRUD operations.

The transaction handle takes ownership of the client and returns it on `.commit()` or `.cancel()`.

```
let tx = db.begin().await?;

tx.query("CREATE audit:example SET action = 'created'")
    .await?
    .check()?;

let db = tx.commit().await?;
```

Client-managed transactions are supported for embedded and suitable persistent remote connections, including WebSocket. They are not equivalent to maintaining a transaction across independent ordinary HTTP requests.

In 4.0, transaction configuration also includes isolation selection through the SDK where supported by the engine.

Reference: [https://surrealdb.com/docs/reference/rust/methods/begin](https://surrealdb.com/docs/reference/rust/methods/begin?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



## 13. Rust parser, AST, and language-tooling ecosystem

For Rust developers who want to analyze, transform, lint, or generate SurrealQL, the implementation ecosystem is distinct from the execution SDK.

### 13.1 Relevant Rust crates

| Component                   | Responsibility                                                       | Stability consideration                                                       |
| --------------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `surrealdb`                 | Supported Rust SDK and embedded engine access                        | Public supported integration surface                                          |
| `surrealdb-types`           | Public value types and Rust conversions                              | Public types; individual APIs may have their own stability notices            |
| `surrealdb-parser-next`     | Lexer, parser, flat syntax tree, diagnostics, streaming parsing      | Engine-oriented parser; no general long-term public AST guarantee established |
| `surrealdb-expr`            | Logical SurrealQL expressions, statements, and value representations | Explicitly internal and unstable                                              |
| `surrealdb-syn`             | Earlier lexer/parser implementation                                  | Explicitly internal, unstable, and marked for replacement                     |
| `surrealdb-sql`             | Older SurrealQL AST definitions                                      | Historical API; not a safe assumption for SurrealDB 4                         |
| `surql-parser`              | Third-party parsing, linting, formatting, and schema analysis        | Independent compatibility and release lifecycle                               |
| `surrealql-language-server` | Rust language server and semantic tooling                            | Tooling interface, not an execution engine                                    |

### 13.2 `surrealdb-parser-next`

The published 4.0 parser-next implementation is particularly relevant for programmatic tooling.

Its documented design emphasizes:

- An allocation-free lexer.
- A parser whose stack usage does not increase with expression nesting.
- A flat, arena-style syntax tree.
- Statement-at-a-time streaming.
- Explicit parsing limits and scratch storage.

Publicly visible types and entry points include `Ast`, `Node`, `NodeId`, `Tag`, `Diag`, `Limits`, `Scratch`, `parse`, `parse_script`, `parse_statement`, and `StreamParser`.

These facilities suggest opportunities for building query analyzers or source-processing tools without loading the entire database engine.

However, the parser's internal node model and the database's executed logical expression model are different abstractions. Parsing a statement does not establish that it is semantically valid for a particular database schema, allowed under a particular capability configuration, or executable under a selected storage engine.

The parser documentation also does not establish a stable cross-release AST contract for downstream consumers.

Reference: [https://docs.rs/surrealdb-parser-next/latest/surrealdb_parser_next/](https://docs.rs/surrealdb-parser-next/latest/surrealdb_parser_next/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 13.3 The logical-expression layer

`surrealdb-expr` holds the logical form of the language, including expression and statement types.

This is closer to the database's interpretation of a query than a lossless source syntax tree.

The distinction matters for tooling:

- A syntax tree is appropriate for text editing, formatting, and source diagnostics.
- A logical expression representation is more appropriate for normalized query reasoning.
- The executable query plan is a later, database-engine-specific representation.

`surrealdb-expr` explicitly warns that it is an internal implementation detail with unstable APIs. Direct dependencies therefore create maintenance exposure across database upgrades.

The older `surrealdb-syn` crate similarly identifies itself as internal and scheduled for removal once the replacement parser reaches parity.

References:

- [https://docs.rs/crate/surrealdb-expr/latest](https://docs.rs/crate/surrealdb-expr/latest?utm_source=chatgpt.com)
- [https://docs.rs/surrealdb-syn/latest/surrealdb_syn/](https://docs.rs/surrealdb-syn/latest/surrealdb_syn/?utm_source=chatgpt.com)&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Docs.rs+1



### 13.4 SurrealQL language server

The SurrealDB organization also maintains a Rust-based SurrealQL language server.

Its capabilities extend beyond syntax highlighting:

- Schema inference from DDL and query flow.
- Diagnostics for unknown fields, types, and permissions.
- Hover information and function signatures.
- Contextual completions.
- Definition and reference navigation.
- Function call hierarchy.
- Semantic document symbols.
- Headless checks suitable for CI and automated code generation.

Its built-in function catalog is generated from the SurrealDB implementation, making it a potentially useful data source for language intelligence.

The language server uses a separately maintained tree-sitter grammar. The repository documents known grammar gaps, so its parser and the database engine's parser must not automatically be treated as identical.

For machine-oriented analysis, the headless checker can provide structured JSON diagnostics with stable diagnostic codes.

Reference: [https://github.com/surrealdb/surrealql-language-server](https://github.com/surrealdb/surrealql-language-server?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 13.5 Validation, formatting, and schema management

Additional tools complete the authoring ecosystem.

`surreal validate` validates `.surql` files without requiring a running database connection.

`surreal format` provides formatting through the SurrealDB CLI. Its documentation warns that formatting can accept input that is not a valid complete query, so formatting should not substitute for prior validation.

`surqlfmt`, distributed as `@surrealdb/surql-fmt`, provides formatting suitable for whole-directory processing and CI checks.

SurrealKit is a separate schema-management tool built around `.surql` definitions. Its capabilities include schema synchronization, staged rollouts, seeding, schema testing, and type generation.

SurrealKit's 1.0 release-candidate documentation targets SurrealDB 4.0. It should not be confused with the core SurrealQL language, and its release-candidate features should not be treated as a stable Rust SDK contract.

References:

- [https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/validate](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/validate?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/format](https://surrealdb.com/docs/reference/cli/surrealdb-cli/commands/format?utm_source=chatgpt.com)
- [https://surrealdb.com/docs/manage/schema-migration](https://surrealdb.com/docs/manage/schema-migration?utm_source=chatgpt.com)&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

  Database+2



## 14. Consequential limitations and integration boundaries

Several distinctions determine whether SurrealQL's advanced functionality can replace external Rust code effectively.

| Boundary                               | Important implication                                                                               |
| -------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Language versus Rust                   | SurrealQL cannot directly call arbitrary native Rust functions                                      |
| Native query functions versus WASM     | Surrealism adds an ABI and sandbox boundary, not native linking                                     |
| Transaction isolation                  | Serializable isolation is unavailable on SurrealKV in 4.0                                           |
| Database scripting                     | JavaScript execution requires explicit server permission                                            |
| Parser versus executor                 | Successfully parsing SurrealQL does not validate schema, permissions, or execution capability       |
| Schema versus Rust types               | The SDK does not automatically generate Rust domain structs from schemas                            |
| Live updates versus CDC                | Live queries are session-bound; changefeeds are retained and replayable                             |
| In-memory versus disk execution        | Embedded storage choices affect durability and isolation                                            |
| Approximate versus exact retrieval     | HNSW and DiskANN do not guarantee exact nearest neighbors                                           |
| Materialized versus computed values    | Derived table views and write-time field values have different costs from read-time computed fields |
| Graph traversal versus graph analytics | Native traversal is extensive, but specialized analytical algorithms remain separate                |
| SQL compatibility                      | PostgreSQL/MySQL SQL listeners, GraphQL, ISO GQL, and SurrealQL are different language interfaces   |

### 14.1 SQL dialect compatibility is not SurrealQL compatibility

SurrealDB 4.0 supports PostgreSQL and MySQL SQL dialects over their respective wire-protocol listeners.

These are separate language front ends to the database.

In particular, SurrealQL should not be submitted to the PostgreSQL or MySQL listeners on the assumption that those listeners simply forward it unchanged. The 4.0 compatibility documentation explicitly distinguishes their accepted dialects.

GraphQL and ISO GQL likewise have different syntax and capability boundaries.

Reference: [https://surrealdb.com/docs/reference/rest-api/sql-compatibility](https://surrealdb.com/docs/reference/rest-api/sql-compatibility?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 14.2 Version-dependent behavior can be substantial

SurrealDB 4.0 is a major architectural release, not merely a new set of query functions.

Relevant changes include the unified executor, explicit isolation levels, native schema migrations, spatial indexes, additional types, new extension semantics, and removal of older embedded engines and SurrealML.

Datastore migration from SurrealDB 3.x is explicitly gated and can become irreversible after adopting the 4.0 record format.

The 4.0.1 release is confirmed through the published Rust crate and release artifacts, although some official release-index pages were still displaying 4.0.0 as their latest patch when researched. I therefore use the published 4.0.1 SDK/version as the current target, but attribute the major feature changes to the documented 4.0.0 release rather than claiming that they were introduced in 4.0.1.

Reference: [https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 14.3 Performance: where the architectural value comes from

SurrealQL's performance opportunities can be understood in four categories.

Reduced application-to-database coordination. A statement that combines filtering, navigation, transformation, and mutation can avoid repeated network requests and application-side joins.

Storage-aware execution. Indexes, structured record-key ranges, graph adjacency representations, and vector search can avoid loading large intermediate datasets into Rust.

Database-maintained derived data. Materialized views, stored field transformations, reverse references, and event-maintained structures move work from readers to writers.

Specialized execution mechanisms. The 4.0 compiler, plan caching, parallel scanning, memory spilling, and approximate search indexes provide capabilities that would otherwise require considerable custom machinery.

None of these implies that arbitrary computation becomes faster merely by moving it into SurrealQL. Workloads with expensive numerical kernels, tightly controlled data locality, SIMD-heavy application loops, specialized graph algorithms, or GPU execution may still benefit from native Rust libraries and external computation.

The appropriate boundary depends on where data resides, how often it crosses interfaces, what guarantees are required, and whether the relevant computation is already natively implemented in SurrealDB.

## 15. Primary-source navigation

The following references provide the most useful entry points for further investigation.

| Subject                          | Direct source                                                                                                                                                                                       |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SurrealQL language               | [https://surrealdb.com/docs/reference/query-language](https://surrealdb.com/docs/reference/query-language?utm_source=chatgpt.com)                                                                   |
| SurrealDB 4.0 changes            | [https://surrealdb.com/releases/4.0](https://surrealdb.com/releases/4.0?utm_source=chatgpt.com)                                                                                                     |
| Rust SDK                         | [https://surrealdb.com/docs/reference/rust](https://surrealdb.com/docs/reference/rust?utm_source=chatgpt.com)                                                                                       |
| Rust crate                       | [https://docs.rs/crate/surrealdb/latest](https://docs.rs/crate/surrealdb/latest?utm_source=chatgpt.com)                                                                                             |
| Rust value types                 | [https://surrealdb.com/docs/reference/rust/concepts/working-with-types](https://surrealdb.com/docs/reference/rust/concepts/working-with-types?utm_source=chatgpt.com)                               |
| Parser-next                      | [https://docs.rs/surrealdb-parser-next/latest/surrealdb_parser_next/](https://docs.rs/surrealdb-parser-next/latest/surrealdb_parser_next/?utm_source=chatgpt.com)                                   |
| Logical expression crate         | [https://docs.rs/crate/surrealdb-expr/latest](https://docs.rs/crate/surrealdb-expr/latest?utm_source=chatgpt.com)                                                                                   |
| Index definitions                | [https://surrealdb.com/docs/reference/query-language/statements/define/indexes](https://surrealdb.com/docs/reference/query-language/statements/define/indexes?utm_source=chatgpt.com)               |
| Field definitions and migrations | [https://surrealdb.com/docs/reference/query-language/statements/define/field](https://surrealdb.com/docs/reference/query-language/statements/define/field?utm_source=chatgpt.com)                   |
| Transactions                     | [https://surrealdb.com/docs/reference/query-language/language-primitives/transactions](https://surrealdb.com/docs/reference/query-language/language-primitives/transactions?utm_source=chatgpt.com) |
| Events                           | [https://surrealdb.com/docs/reference/query-language/statements/define/event](https://surrealdb.com/docs/reference/query-language/statements/define/event?utm_source=chatgpt.com)                   |
| Custom APIs                      | [https://surrealdb.com/docs/reference/query-language/statements/define/api](https://surrealdb.com/docs/reference/query-language/statements/define/api?utm_source=chatgpt.com)                       |
| Rust/WASM extensions             | [https://surrealdb.com/docs/learn/extensions/guides/extension-architecture](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture?utm_source=chatgpt.com)                       |
| Language server                  | [https://github.com/surrealdb/surrealql-language-server](https://github.com/surrealdb/surrealql-language-server?utm_source=chatgpt.com)                                                             |
| Schema tooling                   | [https://surrealdb.com/docs/manage/schema-migration](https://surrealdb.com/docs/manage/schema-migration?utm_source=chatgpt.com)                                                                     |
| Upstream implementation          | [https://github.com/surrealdb/surrealdb](https://github.com/surrealdb/surrealdb?utm_source=chatgpt.com)                                                                                             |

## 16. Overall architectural assessment

SurrealQL is best understood as an integrated data-querying and database-programming language implemented by a Rust-native database engine, rather than as a conventional standalone Rust library.

Its distinctive strength is not any single feature such as SQL querying or graph traversal. It is the ability to combine several traditionally separate layers:

- Typed persistent records and complex values.
- Graph relationships and recursive navigation.
- Indexed lexical, semantic-vector, and spatial retrieval.
- Declarative integrity and derived-data maintenance.
- Procedural database functions and transactional operations.
- Authentication, authorization, and database-hosted APIs.
- Rust SDK integration and optional Rust-authored WASM extensions.

These combinations can make it possible to implement data-intensive operations with substantially less custom orchestration.

The most consequential capabilities to investigate in depth are structured record IDs, lightweight and inlined graph relationships, materialized views, computed and migrated fields, higher-order expressions, query-plan introspection, transactional events, result streaming, and Rust/WASM extensions.

The principal tradeoff is the distinction between database-native execution and arbitrary Rust execution. SurrealQL provides considerable expressive power inside the database, but the supported mechanisms, their performance, and their guarantees remain constrained by SurrealDB's type system, query planner, storage engine, authorization model, and extension runtime.

For advanced Rust engineering, the strongest architectural opportunity is therefore not to force all computation into SurrealQL, but to recognize which operations the database can already perform efficiently and transactionally—and reserve separate Rust execution for functionality that genuinely requires a different computational model.

# SurrealQL for Rust — Technical Capability Addendum

Research date: November 10, 2026\
Target versions: SurrealDB 4.0.1, Rust SDK 4.0.1, and the corresponding SurrealQL implementation.

This addendum supplements the original SurrealQL technical reference. It develops the previously omitted or insufficiently explained capabilities without repeating the original coverage of graph relationships, ordinary CRUD operations, schema definitions, vector indexes, transactions, or the basic Rust SDK.

The additional capabilities are especially significant in four areas:

- Database-native programmability: dynamically evaluated queries, persistent parameters, sequences, expression composition, and data transformations.
- Operational control: index lifecycle management, metadata introspection, database configuration, and credential administration.
- Search and numerical computation: configurable text-processing pipelines, vector aggregation, and specialized numerical and temporal operations.
- Rust integration: independent client sessions, direct API and function invocation, type reflection, execution statistics, and lossless value interchange.

The discussion distinguishes language features from Rust SDK capabilities, server configuration, protocol behavior, and storage-engine guarantees.

All code examples are illustrative and have not been executed locally.

## 1. Dynamic SurrealQL execution and runtime query composition

### 1.1 `eval::surql()`: executing queries represented as values

Available since SurrealDB 3.2

SurrealQL supports dynamically parsed and evaluated queries through `eval::surql()`.

Its signature is:

```
eval::surql(
    query: string,
    bindings: option<object>
) -> any
```

The query is supplied as a string, parsed at runtime, and executed in the caller's session and transaction context.

```
RETURN eval::surql(
    "RETURN $quantity * $unit_price",
    {
        quantity: 12,
        unit_price: 7.5
    }
);
```

This differs fundamentally from a conventional subquery. A normal subquery is part of the surrounding query's parsed expression structure. With `eval::surql()`, the text of the query can itself be retrieved from a record, selected dynamically, or constructed during execution.

For multiple statements, the supplied query must be enclosed in a block:

```
RETURN eval::surql(
    "{
        LET $x = 10;
        LET $y = 20;
        RETURN $x + $y;
    }"
);
```

The distinction opens several architectural possibilities:

Stored query definitions. Query text can be maintained as database data and selected according to runtime conditions. This can support database-driven analytical procedures or configurable processing rules.

Dynamic query composition. A function can determine which query to run based on data, rather than requiring an external Rust application to orchestrate each alternative.

Cross-language query execution. The related `eval::gql()` function evaluates ISO GQL query text, allowing supported GQL operations to be invoked from a SurrealQL execution context.

Transactionally integrated execution. Nested data modifications operate within the caller's existing transaction, avoiding a separate transaction boundary for the dynamically evaluated operation.

These facilities are especially valuable where the structure of an operation is data-dependent rather than merely its parameter values.

### 1.2 Scope isolation and transaction semantics

Dynamically evaluated queries do not automatically inherit the caller's local variables.

```
LET $secret = 42;

RETURN eval::surql(
    "RETURN $secret ?? 'not provided'"
);
```

The nested query does not see `$secret` unless it is explicitly passed.

```
RETURN eval::surql(
    "RETURN $secret",
    { secret: $secret }
);
```

This provides an explicit variable-passing boundary, but not a security sandbox.

The nested query still executes with the caller's database permissions and inside the caller's transaction. Its writes participate in that transaction, and transaction-control statements such as `BEGIN`, `COMMIT`, and `CANCEL` are prohibited within the evaluated query.

Recursive evaluation is bounded by the database's computation limits.

### 1.3 Authorization and capability restrictions

Dynamic evaluation is considered a sensitive operation.

It is denied for every subject by default, including configurations using `--allow-all`.

The server must explicitly enable the capability with `--allow-eval-query`. The caller must also satisfy the arbitrary-query permission gate. A deployment that prohibits arbitrary queries for record users cannot use `eval::surql()` to bypass that restriction.

This means dynamic evaluation is best suited to controlled query definitions and trusted procedural logic, rather than unrestricted execution of user-provided text.

Architectural distinction: Where the query shape is known ahead of time, `DEFINE FUNCTION`, `DEFINE API`, or ordinary parameterized SDK queries generally provide a narrower and easier-to-audit interface. Dynamic evaluation becomes valuable when the query structure itself must be determined at runtime.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/functions/database-functions/eval](https://surrealdb.com/docs/reference/query-language/functions/database-functions/eval?utm_source=chatgpt.com)

[https://surrealdb.com/docs/learn/security/authorization/capabilities](https://surrealdb.com/docs/learn/security/authorization/capabilities?utm_source=chatgpt.com)

## 2. Persistent parameters, session state, and distributed sequences

SurrealQL provides several mechanisms for retaining values across operations. They have different lifetimes, concurrency properties, and appropriate uses.

### 2.1 Database-wide parameters: `DEFINE PARAM`

`DEFINE PARAM` creates a named parameter definition at database scope.

```
DEFINE PARAM $configuration VALUE {
    search_limit: 100,
    confidence_threshold: 0.85,
    enable_reranking: true
};
```

The value can then be referenced by subsequent queries:

```
RETURN $configuration.confidence_threshold;
```

A defined parameter is accessible across clients using that database, subject to its permissions.

It supports:

- `IF NOT EXISTS` for conditional definition.
- `OVERWRITE` for replacing an existing definition.
- `COMMENT` for documenting its purpose.
- `PERMISSIONS` for controlling access to the parameter.

Unlike a `LET` binding, a defined parameter forms part of the database's persistent configuration metadata.

An important limitation is that its value is not updated through ordinary record mutation. Updating it requires changing the parameter definition. For frequently changing state, ordinary records are usually the more appropriate abstraction.

One useful consequence is that schema expressions, reusable functions, and application queries can share centrally defined constants without every caller rebinding them.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/statements/define/param](https://surrealdb.com/docs/reference/query-language/statements/define/param?utm_source=chatgpt.com)

### 2.2 Distinguishing parameter lifetimes

SurrealQL has at least four relevant parameter sources:

| Mechanism             | Lifetime and scope                            | Principal purpose                        |
| --------------------- | --------------------------------------------- | ---------------------------------------- |
| `LET $value`          | Query/session context, subject to scope rules | Intermediate state and query composition |
| SDK `.set()`          | Client session                                | Reusable client-specific variables       |
| SDK `.bind()`         | Individual submitted query                    | Parameterized execution                  |
| `DEFINE PARAM $value` | Persistent database definition                | Shared database-wide constants           |

`LET` is not universally block-local. Top-level parameters may persist across subsequent queries on the same connection or session, while nested query blocks introduce their own scope behavior.

That distinction is particularly important when designing reusable sessions or executing nested queries.

Parameter precedence, authorization, and missing-value behavior should be considered alongside the Rust SDK's binding semantics.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/language-primitives/parameters](https://surrealdb.com/docs/reference/query-language/language-primitives/parameters?utm_source=chatgpt.com)

### 2.3 Distributed numeric sequences: `DEFINE SEQUENCE`

Available since SurrealDB 3.0

A sequence is a native stateful generator for integer identifiers.

```
DEFINE SEQUENCE entity_number
    BATCH 1000
    START 100
    TIMEOUT 5s;

RETURN sequence::nextval('entity_number');
```

Sequences provide database-managed allocation without requiring applications to implement a counter record, lock protocol, or independent distributed ID service.

The implementation uses several mechanisms:

| Mechanism                 | Operational role                                                            |
| ------------------------- | --------------------------------------------------------------------------- |
| Batch reservation         | Reserve multiple identifiers with one coordinated allocation                |
| Per-node ownership        | Associate reserved ranges with node identifiers                             |
| Durable metadata          | Recover safe allocation state across restarts                               |
| Local sequence cache      | Serve values from reserved ranges without repeated distributed coordination |
| Exponential retry backoff | Reduce contention during failed allocations                                 |
| Configurable timeout      | Bound time spent acquiring another range                                    |

Larger batches reduce coordination overhead but can leave more unused values when a node stops before consuming its allocation.

The distinction between uniqueness and transaction commitment is critical.

Sequence advancement is not rolled back when a transaction fails or is cancelled. Therefore, values can be skipped, and sequence identifiers must not be interpreted as a complete record of committed transactions.

In distributed operation, allocated ranges also should not be treated as a strict global clock for the real-time order in which concurrent operations complete.

Sequences are particularly appropriate for surrogate numeric identifiers, monotonic allocation within the documented sequence model, and other use cases that permit gaps.

They are not suitable as a direct substitute for gapless accounting numbers, strict transaction ordering, or globally synchronized execution barriers.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/statements/define/sequence](https://surrealdb.com/docs/reference/query-language/statements/define/sequence?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/functions/database-functions/sequence](https://surrealdb.com/docs/reference/query-language/functions/database-functions/sequence?utm_source=chatgpt.com)

## 3. Expression semantics, operators, and index-aware query construction

SurrealQL's expression system has several semantics that materially differ from both Rust and conventional SQL. They affect correctness, query planning, and the behavior of dynamically assembled queries.

### 3.1 Equality, truthiness, and nullish evaluation

SurrealQL distinguishes ordinary value equality from exact equality:

```
RETURN [
    10 = 10.0,
    10 == 10.0,
    10 = '10'
];
```

The important distinction is that `=` permits equality between compatible numeric representations, whereas `==` also requires type identity.

Logical operators can return operands rather than strictly boolean values:

```
RETURN [
    10 AND 20 AND 30,
    0 OR false OR 10,
    NULL ?? 0 ?? 10,
    NULL ?: 0 ?: 10
];
```

These expressions illustrate four different behaviors:

| Operator | Behavior                                                    | Example result     |
| -------- | ----------------------------------------------------------- | ------------------ |
| `AND`    | Returns according to operand truthiness                     | `10 AND 20` → `20` |
| `OR`     | Returns the first truthy operand, or the last operand       | `0 OR 10` → `10`   |
| `??`     | Selects the first operand that is neither `NONE` nor `NULL` | `NULL ?? 0` → `0`  |
| `?:`     | Selects the first truthy operand                            | `0 ?: 10` → `10`   |

This makes SurrealQL expressions useful for compact defaulting and conditional computation.

It also means that replacing an expression with what appears to be a logically equivalent boolean expression can change its returned value.

`??` and `?:` have particularly low precedence. Parentheses should be used when combining them with complex logical conditions.

### 3.2 Collection and containment semantics

SurrealQL distinguishes equality across collections from existential or universal predicates:

- `?=` determines whether any collection element equals a value.
- `*=` determines whether all elements equal a value.
- `CONTAINSALL` requires all specified values to be contained.
- `CONTAINSANY` requires at least one specified value.
- `CONTAINSNONE` requires none of the specified values.
- `ALLINSIDE`, `ANYINSIDE`, and `NONEINSIDE` express the inverse direction of containment.

For example:

```
RETURN [
    [10, 20, 30] ?= 20,
    [10, 10, 10] *= 10,
    [10, 20, 30] CONTAINSALL [10, 30],
    [10, 20, 30] CONTAINSANY [5, 20]
];
```

The same general operator vocabulary also applies to strings, geometries, and certain record structures. The precise behavior depends on the operand types.

### 3.3 Semantic equivalence does not guarantee the same query plan

An especially important optimization distinction is that the planner recognizes specific expression forms.

For example, current documentation confirms that an ordinary index can support both comparison predicates and range membership:

```
DEFINE INDEX by_age ON person FIELDS age;

SELECT * FROM person WHERE age >= 18 AND age < 30;

SELECT * FROM person WHERE age IN 18..30;
```

Correction to the preceding gap analysis: That analysis stated that `age IN 18..=65` necessarily requires a table scan. This was outdated. Since SurrealDB 3.3.1, the planner can use suitable standard indexes for range membership with `IN`.

A more useful example of remaining query-shape sensitivity is string-prefix matching:

```
SELECT * FROM person
WHERE string::starts_with(name, 'Ann');
```

With an appropriate index, the planner recognizes this function-call form as a prefix scan.

However:

```
SELECT * FROM person
WHERE name.starts_with('Ann');
```

is documented as producing the same results while using a table scan.

This is consequential because function and method syntax are often interchangeable semantically, but they are not always interchangeable for optimization.

Index eligibility also differs for array-element containment, scalar string equality, substring matching, and full-text operators.

An index on an array's elements can support certain containment operators, while an ordinary index on a string field is primarily suited to equality, membership, ranges, and supported prefix forms.

For performance-sensitive query generation, `EXPLAIN` remains the authoritative way to establish which access path the optimizer actually selected.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/language-primitives/operators](https://surrealdb.com/docs/reference/query-language/language-primitives/operators?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/statements/define/indexes](https://surrealdb.com/docs/reference/query-language/statements/define/indexes?utm_source=chatgpt.com)

### 3.4 Full-text operator composition

SurrealQL also supports more specialized match operators than the basic `@@` form.

`@AND@` and `@OR@` express conjunction and disjunction over terms within an indexed full-text field:

```
SELECT *
FROM document
WHERE content @AND@ 'graph database';

SELECT *
FROM document
WHERE content @OR@ 'graph database';
```

This is different from composing separate full-text expressions using ordinary boolean operators.

The documentation identifies a query-planner correctness issue with multiple `MATCHES` conditions in versions before 3.3.1. The issue could cause one condition to be ignored under some plans.

It was corrected in 3.3.1. This version distinction matters for systems where full-text predicates also restrict access to records.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/language-primitives/operators](https://surrealdb.com/docs/reference/query-language/language-primitives/operators?utm_source=chatgpt.com)

## 4. Full-text analysis as a configurable processing pipeline

The previous overview described full-text indexes but did not adequately develop the analyzer model that produces the indexed tokens.

### 4.1 Analyzer architecture

`DEFINE ANALYZER` can combine three processing stages:

Input text

Preprocessing function

Optional SurrealQL function to normalize or transform the complete input string

Tokenizers

Split text using character classes, whitespace, punctuation, or camel-case boundaries

Token filters

Normalize case and characters, stem terms, produce n-grams, or apply dictionary mappings

Tokens supplied to full-text indexing and retrieval

A representative source-code analyzer can be defined as:

```
DEFINE ANALYZER code
    TOKENIZERS class, camel
    FILTERS lowercase, ascii;
```

The `camel` tokenizer is useful for identifiers whose meaningful terms are embedded in mixed-case names.

For autocomplete, edge n-gram generation provides another option:

```
DEFINE ANALYZER autocomplete
    FILTERS lowercase, edgengram(2, 10);
```

Edge n-grams generate token prefixes, making indexed prefix retrieval possible for suitable text workloads.

### 4.2 Custom preprocessing functions

The `FUNCTION` clause is more powerful than a fixed tokenizer configuration.

It allows an analyzer to execute a named function before tokenization.

This can support domain-specific normalization such as canonicalizing known identifiers, standardizing formatting, or converting specialized terminology into a consistent textual representation.

For some workloads, it avoids the need to preprocess strings independently in every Rust ingestion path.

The value is not limited to reducing application code. Centralizing text normalization within an analyzer can make indexing and query-time analysis more consistent.

However, changes to analyzer definitions can affect which tokens exist in already constructed indexes. Index lifecycle and rebuilding must therefore be considered when changing normalization behavior.

### 4.3 Dictionary mapping and stemming

The analyzer subsystem includes `mapper(path)`, which loads a dictionary of token mappings.

A dictionary might normalize distinct expressions to one canonical token:

```
file_not_found    NOT_FOUND
Datei_nicht_gefunden    NOT_FOUND
Fichier_introuvable    NOT_FOUND
```

This supports domain terminology normalization, cross-language term equivalence, and vocabulary harmonization without requiring a separate neural text-processing service.

It is not unrestricted lemmatization. The mappings are explicitly supplied and applied according to the analyzer's token-processing semantics.

Dictionary loading also has an operational restriction: file paths must be permitted by `SURREAL_FILE_ALLOWLIST`.

That makes dictionary-backed analyzers deployment-sensitive. The dictionary must be accessible to the relevant database process, and changes to its contents must be coordinated with the indexing lifecycle.

### 4.4 Analyzer introspection

`search::analyze()` exposes the token stream generated by a named analyzer:

```
RETURN search::analyze(
    'code',
    'calculateTotalValue'
);
```

This provides an important diagnostic facility.

Instead of inferring how a tokenizer or stemmer behaves, a query can inspect the normalized tokens that will participate in retrieval.

For advanced text-search systems, this can reduce the need for duplicated external tokenization code.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/statements/define/analyzer](https://surrealdb.com/docs/reference/query-language/statements/define/analyzer?utm_source=chatgpt.com)

## 5. Index lifecycle, asynchronous construction, and storage maintenance

Index creation should be understood as an operational lifecycle, not simply a schema declaration.

### 5.1 Concurrent index construction

`DEFINE INDEX ... CONCURRENTLY` starts a background build rather than waiting for every existing record to be indexed.

```
DEFINE INDEX observation_time
ON observation FIELDS observed_at
CONCURRENTLY;
```

The index passes through several stages, including initial indexing and processing updates received while the initial build was underway.

The state is available through:

```
INFO FOR INDEX observation_time ON observation;
```

The index is usable for normal query planning once its build reaches `ready`.

An index definition that exists but has not yet finished building is therefore not equivalent to a fully usable index.

### 5.2 Uniqueness enforcement during index construction

Background construction introduces an important correctness issue for unique indexes.

Suppose an existing table contains duplicate values, and a unique index is created concurrently.

If the build encounters those duplicates, it can enter an error state.

In SurrealDB 4.0, the database also refuses writes that would need enforcement by the failed index. This prevents new writes from silently violating a constraint that cannot currently be verified.

Recovery requires correcting the conflicting records and rebuilding the index, or removing the index.

The database's response differs according to whether a write actually changes an indexed field. Not every write to the table is necessarily prohibited.

### 5.3 Index construction inside transactions

SurrealDB 4.0 also documents specific behavior for non-concurrent index definitions executed inside an existing transaction.

The initial index build may execute through its own storage transactions. Consequently, the enclosing transaction cannot always read entries created by that build.

The engine prevents certain combinations of writes and index definitions that would otherwise produce inconsistent results.

In particular:

- A transaction that already modified records of a table cannot subsequently define a non-concurrent index over those records.
- After a non-concurrent index definition, some modifications to existing indexed records are prohibited until the enclosing transaction completes.
- Other sessions may need to wait for the transaction defining the index.
- Concurrent index construction begins after the defining transaction commits, avoiding several of these restrictions.

This matters for schema-management tools that apply DDL and data migrations together.

It also illustrates why the transaction semantics of DDL should not be inferred solely from the transaction semantics of ordinary records.

### 5.4 Rebuilding and decommissioning indexes

`REBUILD INDEX` supports both blocking and concurrent operation:

```
REBUILD INDEX observation_time
ON observation
CONCURRENTLY;
```

Rebuilding is useful after index-definition changes, failed construction, or degradation of certain vector index structures through repeated modifications.

`ALTER INDEX ... PREPARE REMOVE` provides a different capability:

```
ALTER INDEX observation_time
ON observation
PREPARE REMOVE;
```

The index is retained but removed from ordinary query planning.

This makes it possible to examine the consequences of decommissioning an index before permanently deleting its data.

A prepared index can be made usable again through a rebuild.

### 5.5 Compaction and index-maintenance state

SurrealDB 4.0 exposes additional state for count, full-text, HNSW, and DiskANN indexes.

Relevant `INFO FOR INDEX` fields include:

| Field          | Meaning                                                     |
| -------------- | ----------------------------------------------------------- |
| `status`       | Build state, including readiness or failure                 |
| `pending`      | Updates awaiting processing during a concurrent build       |
| `queued`       | Whether index compaction has been requested                 |
| `compacting`   | Whether compaction remains in progress                      |
| `active`       | Whether this node is actively running a compaction pass     |
| `armed_shards` | Pending-write partitions for supported vector-index layouts |

A vector index may report `status = 'ready'` while still having pending compaction work.

That matters because queries may need to inspect both the compacted index and recent unmerged writes.

The documented vector-index layout uses 32 pending-write partitions. Monitoring those partitions can help distinguish a completed initial build from an index that has also finished consolidating recent updates.

`pending` measures a different kind of outstanding work and must not be interpreted as a universal compaction-progress counter.

### 5.6 The distinction between `CONCURRENTLY` and legacy `DEFER`

An additional version-sensitive clarification is necessary.

The legacy `DEFER` clause was supported in SurrealDB 2.x for deferring ongoing index updates.

It was removed in 3.0.

`CONCURRENTLY` is not a drop-in replacement for that mechanism. It primarily changes how an index is initially constructed or rebuilt, rather than simply making all future index maintenance asynchronous.

This distinction matters when comparing historical ingestion-performance guidance with SurrealDB 4.0 behavior.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+3



[https://surrealdb.com/docs/reference/query-language/statements/define/indexes](https://surrealdb.com/docs/reference/query-language/statements/define/indexes?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/statements/rebuild](https://surrealdb.com/docs/reference/query-language/statements/rebuild?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/statements/alter/indexes](https://surrealdb.com/docs/reference/query-language/statements/alter/indexes?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/statements/info](https://surrealdb.com/docs/reference/query-language/statements/info?utm_source=chatgpt.com)

## 6. Schema introspection and metadata-driven database operations

### 6.1 `INFO` as a queryable metadata interface

Since SurrealDB 3.0, `INFO` statements can participate in other queries and expressions.

This is a significant distinction from database systems where schema inspection is primarily a separate administrative interface.

For example:

```
LET $metadata = INFO FOR DB;

RETURN {
    tables: $metadata.tables.keys(),
    functions: $metadata.functions.keys(),
    sequences: $metadata.sequences.keys()
};
```

The available scopes include root, namespace, database, table, user, and index.

At database scope, SurrealDB 4.0 exposes definitions for tables, functions, parameters, analyzers, sequences, extensions, API endpoints, bucket definitions, named types, users, and other configuration objects.

The resulting values can be inspected and transformed using ordinary SurrealQL operations.

This supports a number of native capabilities:

Schema inventory. Enumerate defined database resources without maintaining an external schema catalog.

Schema comparison. Capture metadata at different points and compare representations using `value::diff()`.

Dynamic database programming. Retrieve resource definitions and incorporate them into subsequent logic.

Declarative configuration inspection. Determine whether an API, function, sequence, or index is defined before relying on it.

`INFO FOR TABLE` similarly exposes table-specific fields, events, indexes, and related definitions.

### 6.2 Structured metadata and stability

`INFO ... STRUCTURE` provides a different representation of schema metadata.

Instead of returning mainly textual definitions, it produces structured objects and arrays describing fields and their properties.

This is potentially convenient for schema-analysis tooling.

However, the documentation explicitly identifies `STRUCTURE` as an internal interface whose representation may change without notice.

It should therefore not be treated as a stable machine-readable schema protocol across SurrealDB versions.

`INFO` also has limits: it does not provide a general mechanism for measuring the storage size of a table, namespace, or database. Those measurements remain storage-engine or monitoring concerns.

### 6.3 Dynamic resource names and field projections

SurrealQL 3.0 expanded the locations where expressions and parameters can be used inside statements.

For example:

```
FOR $language IN ['en', 'fr', 'de'] {
    DEFINE TABLE 'content_' + $language SCHEMAFULL;
};
```

Other expressions can similarly construct record targets and resource names.

For dynamic retrieval, `type::table()`, `type::record()`, `type::field()`, and `type::fields()` are especially useful.

```
LET $table = 'person';
LET $fields = ['name', 'age'];

SELECT type::fields($fields)
FROM type::table($table);
```

These functions make it possible to parameterize structural query elements that ordinary value bindings cannot directly replace.

For example, `SELECT $fields` does not expand an array of field names into a projection. `type::fields($fields)` does.

This can substantially reduce custom query-string construction in Rust.

Nevertheless, parameters selecting tables or fields are still dynamic input to database operations. They should be constrained to acceptable resources and interpreted within the database's authorization rules.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



[https://surrealdb.com/docs/reference/query-language/statements/info](https://surrealdb.com/docs/reference/query-language/statements/info?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/functions/database-functions/type](https://surrealdb.com/docs/reference/query-language/functions/database-functions/type?utm_source=chatgpt.com)

## 7. Database configuration, credential administration, and API middleware

### 7.1 Database-wide API configuration

`DEFINE CONFIG API` provides configuration that applies to database-defined API endpoints.

For example:

```
DEFINE CONFIG API
    MIDDLEWARE
        api::timeout(10s),
        api::res::headers({
            'Access-Control-Allow-Origin': '*'
        });
```

This introduces a database-level middleware configuration layer in addition to individual `DEFINE API` endpoint declarations.

The built-in middleware families support:

| Function               | Capability                                     |
| ---------------------- | ---------------------------------------------- |
| `api::timeout()`       | Bound endpoint execution time                  |
| `api::req::max_body()` | Reject oversized request bodies before parsing |
| `api::req::body()`     | Select request-body interpretation             |
| `api::res::body()`     | Set response-body serialization                |
| `api::res::header()`   | Add individual response headers                |
| `api::res::headers()`  | Configure multiple response headers            |
| `api::res::status()`   | Set the response status                        |

Custom middleware is also supported through user-defined SurrealQL functions.

Such functions receive a request object and a continuation closure. The middleware may inspect or modify the request, invoke the continuation, and transform the resulting response.

The sequence of middleware calls therefore forms a processing chain rather than merely an unordered list of configuration options.

Architectural implication: An API that primarily exposes database operations can sometimes be implemented with endpoint logic and middleware inside SurrealDB, avoiding an additional request-handling layer for supported transformations.

The tradeoff is that the API now depends on the database's authorization, capability, observability, and execution environment.

### 7.2 Request-size and resource controls

Middleware order can affect both safety and efficiency.

For example:

```
DEFINE API '/upload'
    FOR post
    MIDDLEWARE
        api::req::max_body('1mb'),
        api::req::body('json')
    THEN {
        RETURN {
            status: 201,
            body: {
                received: $request.body
            }
        };
    };
```

The size limit is checked before attempting JSON decoding.

An oversized request returns status `413` without entering the handler.

This illustrates how the middleware layer can enforce resource limits before expensive or potentially untrusted processing begins.

Custom API functions can use the same middleware execution model for authentication checks, request validation, or other supported policies.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



[https://surrealdb.com/docs/reference/query-language/statements/define/config](https://surrealdb.com/docs/reference/query-language/statements/define/config?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/functions/database-functions/api](https://surrealdb.com/docs/reference/query-language/functions/database-functions/api?utm_source=chatgpt.com)

### 7.3 GraphQL schema configuration

`DEFINE CONFIG GRAPHQL` controls the database's GraphQL exposure.

It supports:

- Automatic inclusion of eligible tables and functions.
- Explicit inclusion or exclusion lists.
- Query depth limits.
- Query complexity limits.
- Disabling schema introspection.

A configuration must be defined before the GraphQL endpoint can be used. Simply enabling GraphQL at server startup is not sufficient.

For example:

```
DEFINE CONFIG GRAPHQL AUTO;
```

This selects automatic schema exposure. More restrictive configurations can expose only designated tables and functions.

It is important to distinguish GraphQL schema exposure from underlying database authorization. GraphQL configuration determines which schema elements are accessible through that interface, while database access and permission rules determine what a particular caller may read or modify.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/statements/define/config](https://surrealdb.com/docs/reference/query-language/statements/define/config?utm_source=chatgpt.com)

### 7.4 Access-grant lifecycle management

The `ACCESS` statement provides native management of grants associated with defined access methods.

Its major operations are:

| Statement operation | Function                                          |
| ------------------- | ------------------------------------------------- |
| `GRANT`             | Generate a grant for a system user or record user |
| `SHOW`              | Inspect active and historical grant metadata      |
| `REVOKE`            | Revoke individual or selected grants              |
| `PURGE`             | Remove expired or revoked grant records           |

The database can create bearer credentials with an associated identity and lifetime.

An important security property is that the credential secret is returned on initial issuance, while later inspections display it as redacted.

Grant issuance, revocation, and purging can also produce server audit logs when the configured log level permits them.

This is an additional native state-management mechanism: the database maintains a lifecycle for bearer credentials rather than requiring an application to implement all the grant bookkeeping itself.

It does not replace general secrets management for the initial credential or eliminate the need to define appropriate access policies.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/statements/access](https://surrealdb.com/docs/reference/query-language/statements/access?utm_source=chatgpt.com)

### 7.5 System configuration and compaction

SurrealQL also exposes several database-administration operations.

`ALTER SYSTEM` can set a datastore-wide query timeout, request compaction, and select the default compatibility level for newly created databases.

```
ALTER SYSTEM QUERY_TIMEOUT 30s;
ALTER SYSTEM COMPACT;
ALTER SYSTEM COMPATIBILITY 4;
```

The compatibility setting affects databases created subsequently; existing databases require their own compatibility changes.

Compaction is storage-engine-dependent.

In SurrealDB 4.0, SurrealKV supports explicit compaction requests. The storage engine flushes and merges applicable data structures and reclaims versions or deletions that are no longer required by active readers.

The in-memory engine and TiKV do not support the same compaction requests.

Compaction can also be requested at narrower namespace, database, or table scopes through the corresponding `ALTER` statements.

These facilities are valuable for operational control, but they do not imply that SurrealQL exposes arbitrary low-level storage-engine configuration.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/statements/alter/system](https://surrealdb.com/docs/reference/query-language/statements/alter/system?utm_source=chatgpt.com)

## 8. Document transformations, JSON Patch, and contextual expressions

### 8.1 JSON Patch as a native document update mechanism

SurrealQL supports JSON Patch-style operations through the `PATCH` clause and `value::patch()`.

The supported operations include:

| Operation | Effect                                     |
| --------- | ------------------------------------------ |
| `add`     | Add or insert a value                      |
| `remove`  | Delete a value                             |
| `replace` | Replace an existing value                  |
| `copy`    | Copy a value from another path             |
| `move`    | Move a value between paths                 |
| `test`    | Require a value to match before proceeding |
| `change`  | Apply a text diff to a string value        |

For example:

```
UPDATE document:report PATCH [
    {
        op: 'test',
        path: '/status',
        value: 'draft'
    },
    {
        op: 'replace',
        path: '/status',
        value: 'published'
    },
    {
        op: 'add',
        path: '/published_at',
        value: time::now()
    }
];
```

The `test` operation is particularly important because it can enforce a precondition within the patch operation sequence.

If the test fails, the patch fails.

This supports conditional document transformations without separately retrieving and resubmitting every nested field.

However, the semantics are not identical to a general multi-record transaction predicate. The test is evaluated against the targeted document as the patch is processed.

### 8.2 Array paths, ordering, and permission enforcement

JSON Patch paths use JSON Pointer semantics.

Within arrays, operations may insert, remove, or replace elements at specific positions. The special `-` path segment can identify an append position.

For `move`, the source is removed before the destination is interpreted. Consequently, moving an element within the same array may shift the positions used by subsequent operations.

An important security distinction applies to `copy`, `move`, and `test`.

These operations read values at the specified paths. SurrealDB resolves those reads against the caller's permitted view of the record.

If a pointer attempts to read a field that the caller lacks permission to select, the operation fails rather than silently substituting an absent value.

This is different from some ordinary expression contexts, where restricted fields may appear as `NONE`.

The database therefore treats document-patch operations as potentially involving both reads and writes, enforcing the appropriate permissions for each.

### 8.3 `value::diff()` and `value::patch()`

The value functions provide transformation primitives that are independent of a particular record update statement.

```
LET $before = {
    title: 'Report',
    status: 'draft'
};

LET $after = {
    title: 'Report',
    status: 'published'
};

LET $patch = value::diff($before, $after);

RETURN value::patch($before, $patch);
```

This can replace external JSON-diff machinery for supported SurrealQL values.

The functions also interact naturally with `LIVE SELECT DIFF`, which exposes changes using patch-style representations.

For strings, the nonstandard `change` operation uses a text-diff format. Unlike a simple replacement, it attempts to apply changes using surrounding textual context.

A text-diff operation can leave a string unchanged when the expected context is not found, rather than necessarily producing an error. This should not be interpreted as a strong compare-and-swap guarantee.

### 8.4 Assertions inside expression chains

`value::expect()` provides another advanced transformation primitive.

It accepts a value and a predicate closure, returns the value when the condition succeeds, and raises an error otherwise.

```
RETURN {
    confidence: 0.9,
    status: 'valid'
}.expect(
    |$v| $v.confidence >= 0 AND $v.confidence <= 1,
    'Confidence must be between zero and one'
);
```

This allows a transformation pipeline to assert intermediate properties without defining a database field assertion.

The documentation notes that the implementation clones the original value to preserve its return semantics, so repeated use on large values can introduce memory and processing costs.

It is consequently more appropriate for targeted validation than for indiscriminate insertion after every transformation.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



[https://surrealdb.com/docs/reference/query-language/statements/update](https://surrealdb.com/docs/reference/query-language/statements/update?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/functions/database-functions/value](https://surrealdb.com/docs/reference/query-language/functions/database-functions/value?utm_source=chatgpt.com)

### 8.5 Contextual variables and protected execution state

SurrealQL provides built-in contextual parameters that can be used inside field definitions, mutations, events, subqueries, and API handlers.

The important variables include:

| Parameter                     | Context and significance                                   |
| ----------------------------- | ---------------------------------------------------------- |
| `$this`                       | Current record in an expression or subquery                |
| `$parent`                     | Parent record in a nested query                            |
| `$value`                      | Current value in a field evaluation                        |
| `$input`                      | Original submitted value before applicable transformations |
| `$before`                     | Value before a write                                       |
| `$after`                      | Value after a write                                        |
| `$event`                      | `CREATE`, `UPDATE`, or `DELETE` event                      |
| `$reference`                  | Reference involved in a tracked-reference deletion action  |
| `$request`                    | Request object supplied to a custom API handler            |
| `$auth`, `$token`, `$session` | Authentication and session context                         |

These parameters allow database expressions to distinguish incoming data, stored data, transformed data, and execution context.

That distinction is important for maintaining historical values, applying normalization, validating updates, or determining whether a particular record transition occurred.

SurrealDB 4.0 adds protection for system-bound variable names inside procedural blocks and function bodies. Attempts to redefine protected contextual variables with `LET` in those contexts are rejected.

This prevents accidental shadowing of execution-context values in places where their meaning is defined by the database.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/language-primitives/parameters](https://surrealdb.com/docs/reference/query-language/language-primitives/parameters?utm_source=chatgpt.com)

## 9. Serialization, type fidelity, and optimized result representations

### 9.1 JSON, CBOR, and Base64 are distinct abstractions

SurrealQL provides native encoding functions:

```
encoding::json::encode($value);
encoding::json::decode($text);

encoding::cbor::encode($value);
encoding::cbor::decode($bytes);

encoding::base64::encode($bytes);
encoding::base64::decode($text);
```

The important distinction is between representing a value and preserving its full SurrealQL type information.

JSON has fewer native types than SurrealQL.

For example:

```
RETURN encoding::json::decode(
    encoding::json::encode(NONE)
);
```

The result is `NULL`, not `NONE`.

The two values are not semantically interchangeable. A conversion that changes an absent value into a present null value can alter field-update behavior, application validation, and persisted representations.

CBOR provides a richer binary representation capable of preserving SurrealDB-specific type distinctions.

The practical consequence is that JSON should not be assumed to be a lossless intermediate representation for arbitrary SurrealQL values.

When data passes between Rust and SurrealDB, the native `Value` types and SDK conversion mechanisms may preserve distinctions that would otherwise be lost through JSON.

### 9.2 Compact shaped results

Protocol feature introduced in SurrealDB 4.0

SurrealDB 4.0 introduces an optional wire-protocol representation called shaped results.

The motivation is straightforward: ordinary record serialization repeatedly includes field names and type information for each record.

For a large collection of structurally similar records, that repetition creates unnecessary transfer overhead.

Shaped results transmit the field names and types once, followed by row values.

Conceptually:

```
Ordinary representation

[
  { id: ..., name: ..., score: ... },
  { id: ..., name: ..., score: ... },
  { id: ..., name: ..., score: ... }
]

Shaped representation

Schema:
  id: record
  name: string
  score: float

Rows:
  [ ..., ..., ... ]
  [ ..., ..., ... ]
  [ ..., ..., ... ]
```

The actual encoding uses a specialized CBOR representation rather than this illustrative text format.

The protocol uses CBOR tag `57` for shaped row blocks.

It can be requested through the supported WebSocket subprotocol or appropriate HTTP content-negotiation headers.

The format can appear in ordinary query results, streamed result frames, and live-query notifications.

Important boundary: Shaped results are a protocol capability, not a separate SurrealQL function or a general guarantee that every Rust SDK query automatically uses this representation.

A client must negotiate the format and support decoding it.

The expected benefit is reduced serialization redundancy and network transfer for homogeneous result sets. The actual benefit depends on record shape, result size, and client implementation.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/functions/database-functions/encoding](https://surrealdb.com/docs/reference/query-language/functions/database-functions/encoding?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/rest-api/rpc-protocol](https://surrealdb.com/docs/reference/rest-api/rpc-protocol?utm_source=chatgpt.com)

## 10. Database-native vector aggregation and numerical analysis

The original overview concentrated on vector retrieval. SurrealQL also supports calculations over vectors, including aggregate operations across stored records.

### 10.1 Elementwise and geometric operations

The `vector::` family provides:

| Functional family        | Available operations                                |
| ------------------------ | --------------------------------------------------- |
| Elementwise arithmetic   | Add, subtract, multiply, divide                     |
| Scalar/vector operations | Scale, magnitude, normalization                     |
| Linear geometry          | Dot product, cross product, angle, projection       |
| Distance                 | Euclidean, Manhattan, Minkowski, Chebyshev, Hamming |
| Statistical distance     | Mahalanobis                                         |
| Similarity               | Cosine, Jaccard, Pearson, Spearman                  |
| Aggregation              | `vector::sum()`                                     |

These functions are useful when numerical calculations are closely coupled to stored data and do not require an external numerical solver.

They do not constitute a complete linear algebra, differentiation, or mathematical optimization system.

### 10.2 Streaming vector aggregation

Available since SurrealDB 3.3

`vector::sum()` has two forms.

The scalar form sums an existing collection of vectors:

```
RETURN vector::sum([
    [1, 2, 3],
    [4, 5, 6]
]);
```

The aggregate form operates over records:

```
SELECT
    category,
    vector::sum(embedding) AS summed_embedding
FROM document
GROUP BY category;
```

The aggregate implementation maintains a running vector for each group.

For a fixed-dimensional vector, the per-group accumulation state is proportional to the vector dimension rather than the number of records.

This can be much more efficient than loading all embeddings into Rust solely to perform an elementwise sum.

There are important semantic differences between the scalar and aggregate forms:

- An empty scalar collection returns `NONE`.
- The aggregate skips rows whose vector is `NONE` or `NULL`.
- A group containing no valid vectors returns `NONE`.
- Incompatible vector dimensions cause an error in scalar form but produce `NULL` for the affected aggregate group.

### 10.3 Weighted vector centroids

`vector::sum()`, `vector::scale()`, and `math::sum()` can be composed to calculate weighted centroids:

```
SELECT
    category,
    vector::scale(
        vector::sum(
            vector::scale(embedding, weight)
        ),
        1.0 / math::sum(weight)
    ) AS centroid
FROM document
GROUP BY category;
```

Mathematically:

\\[ \boldsymbol{\mu} = \frac{\sum_i w_i\mathbf{x}\_i}{\sum_i w_i} \\]

This is useful for summarizing groups of embeddings, computing aggregate representations, and implementing basic vector-statistical operations close to the stored data.

The source documentation highlights a numerical pitfall: the reciprocal should use a floating-point numerator such as `1.0`. Integer division may truncate the result to zero.

Other edge cases also remain relevant, including zero total weight, inconsistent dimensionality, and the cost of maintaining many simultaneous groups.

### 10.4 Statistical similarity and covariance-aware distance

SurrealQL's Mahalanobis distance function takes two vectors and a covariance matrix:

```
RETURN vector::distance::mahalanobis(
    [1, 2],
    [3, 4],
    [[2, 1], [1, 2]]
);
```

The function requires a symmetric positive-definite covariance matrix, validated using Cholesky decomposition.

That makes it useful for covariance-aware distance measurements, but it does not eliminate the need to estimate or provide the covariance matrix.

Spearman similarity provides rank-based correlation, including average-rank handling for ties.

Pearson similarity provides linear correlation.

These offer alternatives to cosine similarity or Euclidean distance when the relevant vector components have different statistical interpretations.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/functions/database-functions/vector](https://surrealdb.com/docs/reference/query-language/functions/database-functions/vector?utm_source=chatgpt.com)

## 11. Temporal and geospatial semantics

### 11.1 Calendar-relative time is distinct from elapsed duration

SurrealDB 4.0 introduces the `date`, `time`, `timestamp`, and `interval` types.

The most consequential distinction is between fixed elapsed durations and calendar-relative intervals.

For example:

```
RETURN [
    d'2024-01-01T09:00:00Z' + 1y,
    d'2024-01-01T09:00:00Z' + interval'1 year'
];
```

A duration of `1y` represents 365 days.

A calendar interval of one year advances the year in the calendar.

In leap years, the resulting dates can differ.

An interval stores months, days, and a sub-day time component separately. This allows calendar arithmetic to preserve the semantics of months and years rather than reducing every operation to a fixed number of seconds.

Other important behavior includes:

- `timestamp` represents a local date and time without a timezone.
- `datetime` represents an instant in UTC.
- Dates and timestamps can represent positive and negative infinity.
- Intervals can contain signed components.
- Interval ordering uses a normalized duration comparison in which a month counts as 30 days.
- Types are not automatically interchangeable in comparisons.

The normalized comparison rule creates a subtle distinction: two intervals may compare as equal while retaining different calendar arithmetic behavior.

For example, a month and 30 days can compare as equal, but adding them to January 31 can produce different results.

This matters for scheduling, retention rules, subscriptions, and temporal analytics.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/dates-and-times](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/dates-and-times?utm_source=chatgpt.com)

### 11.2 GeoJSON conversion and spatial representation

SurrealQL supports GeoJSON-based geometry values.

The implementation can recognize suitable GeoJSON-shaped objects as geometry values, avoiding explicit conversion in some expressions.

However, the representation has strict requirements.

A geometry point follows GeoJSON coordinate ordering:

```
{
    type: 'Point',
    coordinates: [-0.118092, 51.509865]
}
```

Coordinates are longitude first, latitude second.

This distinction is easy to overlook when integrating geographic sources that display latitude first.

The geometry conversion logic also distinguishes valid GeoJSON object shapes from ordinary objects that merely contain coordinate-like fields.

For example, an object containing extra unrelated keys is not automatically treated as the corresponding geometry.

This is relevant when importing heterogeneous geographic data: storing an arbitrary JSON object is not equivalent to producing a typed geometry suitable for native spatial functions and indexes.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/geometries](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/geometries?utm_source=chatgpt.com)

## 12. Additional Rust SDK capabilities

### 12.1 Client cloning creates independent sessions

Behavior introduced in Rust SDK 3.0 and retained in 4.0

The most important Rust-specific omission is the SDK's session cloning behavior.

Calling:

```
let another_session = db.clone();
```

creates a separate logical database session.

The new session shares the underlying database connection but maintains independent session state.

That includes namespace selection, database selection, authentication, session variables, and transaction state.

The session begins with the original session's configuration, but subsequent changes are independent.

This allows a single connection to support multiple logical execution contexts.

However, for remote connections, creating the session can involve additional network work to reproduce the necessary connection state.

For workloads that merely need multiple handles to one existing session, the appropriate ownership pattern is different:

```
use std::sync::Arc;

let shared_db = Arc::new(db);

let other_handle = Arc::clone(&shared_db);
```

Here, the Rust wrapper is cloned without creating a separate SurrealDB session.

The distinction is especially consequential for asynchronous task orchestration and web frameworks that frequently clone application state.

It is also directly relevant to transaction isolation: two Rust handles that share one session are not equivalent to two independent sessions that share one connection.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/rust/concepts/multi-tenancy](https://surrealdb.com/docs/reference/rust/concepts/multi-tenancy?utm_source=chatgpt.com)

### 12.2 Direct invocation of database-defined APIs

Rust SDK 4.0

The SDK's `.api()` interface invokes endpoints created using `DEFINE API`.

Unlike calling a remote HTTP endpoint through a separate HTTP client, this interface translates the invocation into the database's `api::invoke()` mechanism.

It works over embedded, WebSocket, HTTP, and gRPC connections.

```
let response = db.api().get("/entities").await?;

println!("{}", response.status);
```

Typed result extraction is also supported:

```
let entities: Vec<Entity> = db
    .api()
    .get("/entities")
    .value()
    .await?;
```

The builder exposes standard HTTP-like operations, request headers, query parameters, request bodies, and reusable path prefixes.

A key distinction is that awaiting an API operation returns an `ApiResponse`, including for HTTP-style error statuses.

Its fields include:

- `status`, represented as `u16`.
- `headers`, represented as a string map.
- `body`, represented as a SurrealDB value.

`.value::<R>()` imposes stricter behavior, requiring a successful `2xx` status before decoding the response body.

The same mechanism is exposed through a transaction handle using `Transaction::api()`.

This lets Rust code treat database-defined endpoints as reusable functions while retaining the semantics of their API response format.

A further SurrealDB 4.0 behavior is worth distinguishing: when a database-defined API handler fails, its writes are rolled back, but the surrounding transaction can continue. This is not equivalent to assuming that every failed nested API call automatically aborts the entire caller transaction.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/rust/methods/api](https://surrealdb.com/docs/reference/rust/methods/api?utm_source=chatgpt.com)

### 12.3 Direct SurrealQL function invocation

The SDK's `.run()` method provides a typed interface for invoking database functions.

```
let value: f32 = db.run("rand::float").await?;
```

It also supports functions defined using `DEFINE FUNCTION`.

This avoids assembling a query string such as `RETURN fn::example()` for every invocation.

The function's return value passes through the SDK's ordinary typed conversion machinery.

The advantage is primarily interface simplicity and type handling, rather than a guarantee of faster execution.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/rust/methods/run](https://surrealdb.com/docs/reference/rust/methods/run?utm_source=chatgpt.com)

### 12.4 `SurrealValue::kind_of()` and Rust-driven type information

The `SurrealValue` trait provides a `kind_of()` associated method.

It returns a SurrealQL `Kind` describing the corresponding Rust type.

For example:

```
use surrealdb::types::{SurrealValue, ToSql};

fn main() {
    println!("{}", Option::<String>::kind_of().to_sql());
    println!("{}", Vec::<String>::kind_of().to_sql());
}
```

The first produces a type allowing `NONE` or `string`.

The second produces `array<string>`.

For a derived structure:

```
use surrealdb::types::{RecordId, SurrealValue};

#[derive(SurrealValue)]
struct Person {
    id: RecordId,
    name: String,
    nickname: Option<String>,
    tags: Vec<String>,
}
```

The corresponding SurrealQL type description can be obtained from `Person::kind_of()`.

The `kind!()` macro provides another mechanism for expressing more elaborate SurrealQL type structures from Rust, including literal unions and structured alternatives.

These capabilities support type introspection and can reduce duplicated descriptions of Rust and database-side value types.

However, they do not automatically provide full schema generation, live schema reconciliation, or Rust struct generation from an existing database.

### 12.5 A subtle numeric serialization distinction

SurrealDB 4.0 supports sized number types, but Rust SDK numeric values do not necessarily preserve their Rust bit width when serialized.

For example:

- Rust `i8`, `i16`, `i32`, and related integer values are generally sent as SurrealQL `int`.
- Large unsigned values may require a wider decimal representation.
- Rust `f32` is sent as SurrealQL `float`, rather than automatically as the sized SurrealQL `f32` type.

This preserves compatibility with older supported database versions.

To enforce a sized stored type, the database field must declare that type explicitly.

For example:

```
DEFINE FIELD level ON reading TYPE u8;
```

The database then applies the declared type conversion and range checking during writes.

This is particularly important when precise integer ranges, storage representations, or numerical assumptions matter.

### 12.6 Typed record IDs and identity preservation

The Rust SDK's `RecordId` represents both table identity and the record key.

A record ID should not be treated as an ordinary string merely because its SurrealQL representation is printable.

A subtle example is `RecordId::parse_simple()`.

It parses a string such as `person:1`, but interprets the key as a string rather than inferring the numeric key type from its text.

Thus, a record ID with string key `"1"` is not the same identity as one whose key is numeric `1`.

For complex record IDs, `RecordId::new()` or appropriate SurrealQL conversion facilities provide more precise control.

This distinction matters for composite keys, typed domain identities, and systems exchanging record IDs through textual interfaces.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/rust/concepts/working-with-types](https://surrealdb.com/docs/reference/rust/concepts/working-with-types?utm_source=chatgpt.com)

### 12.7 Query execution statistics

The Rust query builder supports `.with_stats()`.

It requests per-statement execution statistics alongside query results.

```
let mut response = db
    .query("SELECT * FROM person")
    .with_stats()
    .await?;

if let Some((stats, result)) = response.take(0) {
    let people: Vec<Person> = result?;
    println!("Execution time: {:?}", stats.execution_time);
}
```

The returned type is `WithStats<IndexedResults>`, which retains the result-indexing behavior of multi-statement queries while associating execution statistics with each statement.

This supports programmatic query instrumentation without requiring every query to be rewritten as an explicit `EXPLAIN` statement.

It complements, rather than replaces, query-plan inspection: execution time alone does not explain which operators were selected.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/rust/methods/query](https://surrealdb.com/docs/reference/rust/methods/query?utm_source=chatgpt.com)

## 13. Additional correctness and compatibility qualifications

### 13.1 Live-query notifications are not a strictly ordered event log

SurrealDB documents that live-query notifications are emitted only for committed transactions.

SurrealDB 4.0 extends this to transactions explicitly opened and committed by SDK clients.

However, ordering across concurrent clients is best-effort rather than strictly guaranteed.

Notifications generated by transactions committed through the same client retain their order.

There are also several important edge cases:

- A `LIVE SELECT` on a node table does not automatically notify clients when only its connected edge records change.
- A live query whose filter depends on related records is not automatically reevaluated when unrelated tables change.
- Live-query filters and projections cannot perform database mutations.
- Authentication changes that alter the session's principal terminate existing subscriptions.
- Authorization is checked when a notification is published and may reflect changes since the triggering transaction committed.

Therefore, a live subscription should not be treated as a complete, strictly ordered, replayable event history.

For state reconstruction requiring durable offsets, changefeeds offer a different abstraction.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/statements/live-select](https://surrealdb.com/docs/reference/query-language/statements/live-select?utm_source=chatgpt.com)

### 13.2 Native SurrealQL is not equivalent to PostgreSQL or MySQL SQL

SurrealDB 4.0 implements PostgreSQL and MySQL compatibility interfaces, but their SQL features should not be attributed to SurrealQL.

For example, the native SurrealQL language does not provide:

- A conventional `UNION` or `UNION ALL` statement.
- Native SQL window functions such as `RANK()`.

Similar outcomes can sometimes be expressed through arrays, multiple `FROM` targets, subqueries, and aggregations, but those alternatives have different execution characteristics.

The PostgreSQL and MySQL compatibility interfaces implement documented subsets of their respective dialects and may support constructs absent from SurrealQL.

They also have separate connection and language-selection rules. In 4.0, the PostgreSQL listener executes PostgreSQL SQL by default rather than transparently accepting SurrealQL.

The dialect interface, native query language, and underlying Rust execution engine must therefore be treated as distinct layers.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/statements/select](https://surrealdb.com/docs/reference/query-language/statements/select?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/rest-api/sql-compatibility](https://surrealdb.com/docs/reference/rest-api/sql-compatibility?utm_source=chatgpt.com)

### 13.3 Index-planning documentation conflict

The current official documentation contains conflicting descriptions of range-membership index eligibility.

The `DEFINE INDEX` page states that standard indexes support `IN` with ranges starting in SurrealDB 3.3.1.

However, the current `SELECT` page still presents a numeric-range example in which the comparison form uses `IndexScan` while `IN` uses `TableScan`.

Both are official sources, and neither page resolves the contradiction.

Consequently, the stronger conclusion stated earlier in this addendum—that indexed `IN` ranges are unconditionally supported in the target release—should be treated as unverified, not as an established guarantee.

The reliable architectural point is that index selection depends on exact query shape, optimizer version, and available index definitions. In a specific 4.0.1 deployment, the actual plan should be established using `EXPLAIN`.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



[https://surrealdb.com/docs/reference/query-language/statements/define/indexes](https://surrealdb.com/docs/reference/query-language/statements/define/indexes?utm_source=chatgpt.com)

[https://surrealdb.com/docs/reference/query-language/statements/select](https://surrealdb.com/docs/reference/query-language/statements/select?utm_source=chatgpt.com)

### 13.4 Legacy futures are not part of the current type system

One correction to the original overview is unambiguous.

The legacy `future` value type was removed in SurrealDB 3.0.

The supported replacement for dynamically computed field values is the `COMPUTED` field mechanism.

References to `<future>` syntax should therefore be identified as historical rather than current SurrealQL capabilities.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



[https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/futures](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/futures?utm_source=chatgpt.com)

## 14. Architectural implications of the additional capabilities

The most valuable interpretation of these additions is how they expand the kinds of work that SurrealDB can perform natively.

| General problem                        | Native capabilities worth evaluating                             | What may become unnecessary                                       |
| -------------------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------- |
| Runtime-selectable database procedures | `eval::surql`, `eval::gql`, parameterized functions              | External multi-query dispatch logic                               |
| Shared constants and identifiers       | `DEFINE PARAM`, `DEFINE SEQUENCE`                                | Custom configuration and counter infrastructure                   |
| Schema-driven processing               | `INFO`, dynamic DDL, `type::field(s)`, `type::table`             | Some custom schema-discovery and query-construction machinery     |
| Text normalization                     | Analyzer preprocessing, tokenizers, filters, dictionary mappings | Duplicated preprocessing across ingestion paths                   |
| Large index management                 | Concurrent builds, rebuilds, compaction status, decommissioning  | Bespoke index lifecycle tracking                                  |
| Database-oriented APIs                 | `DEFINE CONFIG API`, middleware, SDK `.api()`                    | Portions of external HTTP middleware and API orchestration        |
| Nested document transformations        | `PATCH`, `value::diff`, `value::patch`, contextual parameters    | Custom JSON diffing and read-modify-write logic                   |
| Typed Rust/database interchange        | `SurrealValue`, `Kind`, `kind!`, native values                   | Duplicate type representations and unnecessary JSON serialization |
| Grouped numerical calculations         | `vector::sum`, vector arithmetic, statistical functions          | Some external vector-reduction passes                             |
| Calendar-aware calculations            | `date`, `time`, `timestamp`, `interval`                          | Custom calendar arithmetic for supported operations               |
| Independent logical sessions           | `Surreal::clone()`                                               | Separate physical database connections for every logical session  |
| Large result transfer                  | Shaped protocol results                                          | Some serialization overhead from repeated record metadata         |

These possibilities are conditional.

Native capabilities can reduce application complexity and data movement when the required operations fit SurrealQL's semantics. They do not eliminate the need for external Rust execution when the workload requires capabilities outside the database's programming or execution model.

The essential distinctions remain the cost of data movement, storage-engine guarantees, transaction boundaries, authorization, compatibility, and the runtime behavior of the particular operation.

## 15. Final synthesis

With these additions, the SurrealQL reference captures a substantially broader range of database-native functionality.

The previously underrepresented capabilities reveal that SurrealQL is not only a language for querying and mutating records. It also supplies mechanisms for dynamic query execution, shared state definitions, distributed identifier generation, schema introspection, data transformation, text processing, administrative control, and application-facing API execution.

The Rust integration is correspondingly richer than a simple query client: independent session management, typed value reflection, direct function invocation, transaction-integrated API calls, and execution statistics all provide useful abstractions above raw query submission.

The highest-value architectural distinction is which operations can be expressed, validated, and executed within one database context without requiring custom Rust coordination across multiple interfaces.

That is where the combination of SurrealQL and SurrealDB's native Rust engine offers the greatest potential simplification—while retaining the need to assess storage-specific guarantees, index behavior, numerical limitations, and the precise boundary between database-native computation and external execution.