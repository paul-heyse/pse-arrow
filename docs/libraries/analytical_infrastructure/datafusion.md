# Apache DataFusion: Comprehensive Rust Library and Ecosystem Reference

Research date: October 26, 2026 | Target: Apache DataFusion 55.2.0 | Language: Rust

## 1. Executive orientation

Apache DataFusion is an extensible, Arrow-native relational query engine whose components can be embedded independently or combined to construct sophisticated data-processing systems.

Its principal value extends well beyond accelerating SQL queries. DataFusion provides a reusable infrastructure for relational computation: typed expressions, logical plans, optimization, physical execution, data-source abstraction, resource management, runtime statistics, and integration with external engines.

The current stable core release is 55.2.0, published October 7, 2026. The Apache website also contains documentation for the forthcoming 56.0.0 release; those development-version changes should not be assumed available in 55.2.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://release-catalog.apache.org\&sz=32)

release-catalog.apache.org+2



Official sources:

- Core repository: `https://github.com/apache/datafusion`
- Documentation: `https://datafusion.apache.org/`
- Rust API reference: `https://docs.rs/datafusion/55.2.0/datafusion/`
- Version 55 release notes: `https://datafusion.apache.org/blog/output/2026/08/25/datafusion-55.0.0/`
- Version 55 migration guide: `https://datafusion.apache.org/library-user-guide/upgrading/55.0.0.html`

### 1.1 The fundamental architecture

DataFusion separates five responsibilities that many applications otherwise implement together:

Query representation and entry points

SQL · DataFrame API · Expr · LogicalPlanBuilder · custom frontends

Semantic analysis and logical optimization

Names · schemas · types · relational transformations · rewrite rules

Physical planning and optimization

Operator selection · partitioning · ordering · statistics · distribution

Execution engine

Vectorized Arrow kernels · asynchronous RecordBatch streams · joins · aggregations · windows

Results and downstream consumers

Arrow batches · streams · files · external applications · other engines

Cross-cutting: CatalogProvider · SchemaProvider · TableProvider · ObjectStore · RuntimeEnv · functions · metrics

These stages expose distinct Rust interfaces. An application can use DataFusion as a complete query engine, reuse only its expression and planning machinery, introduce specialized operators, or place it beneath another query language and storage system.

DataFusion itself is not a complete persistent, transactional database or distributed database service. Its catalog and table interfaces can connect those responsibilities to independently managed storage, metadata, and transaction systems. Distributed execution is supplied by companion systems such as Ballista.

### 1.2 Main capability families

| Family           | Principal capabilities                                                                              |
| ---------------- | --------------------------------------------------------------------------------------------------- |
| Relational model | Typed expression trees, logical plans, schemas, relational operators, SQL semantics                 |
| Query interfaces | SQL, lazy DataFrames, programmatic plan construction, alternative SQL frontends                     |
| Optimization     | Expression simplification, predicate/projection pushdown, joins, ordering, partitioning, statistics |
| Execution        | Parallel vectorized operators, Arrow streams, aggregation, windows, top-K, joins                    |
| Data access      | Arrow, Parquet, CSV, JSON, Avro, object stores, external table providers                            |
| Extensibility    | Custom functions, SQL planning, logical nodes, physical operators, optimizer rules                  |
| Resource control | Memory pools, spilling, runtime configuration, parallelism, scheduling                              |
| Introspection    | Logical and physical plan inspection, execution metrics, `EXPLAIN ANALYZE`, tracing                 |
| Interoperability | Arrow, Substrait, Protobuf, FFI, Spark compatibility, external engines                              |
| Ecosystem        | Ballista, Comet, DataFusion Python, SQLParser, community table providers and federation             |

The crucial distinction is between DataFusion as an execution engine and DataFusion as a composable relational programming infrastructure. The second role makes much of its functionality useful even when SQL query execution is only one component of a larger system.

## 2. The Rust crate ecosystem

DataFusion is a Cargo workspace with deliberately separated responsibilities. The top-level `datafusion` crate is the normal integration point, while the component crates expose lower-level functionality and permit dependency selection without adopting every part of the engine.

### 2.1 Principal crates

| Crate                                         | Responsibility and distinctive value                                                                |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `datafusion`                                  | Umbrella API: `SessionContext`, DataFrames, SQL, query execution, and reexports of major components |
| `datafusion-common`                           | `DataFusionError`, `ScalarValue`, `DFSchema`, configuration, statistics, shared utilities           |
| `datafusion-expr`                             | Logical relational operators, `LogicalPlan`, `Expr`, function abstractions, extension nodes         |
| `datafusion-expr-common`                      | Shared logical-expression primitives and supporting types                                           |
| `datafusion-sql`                              | `DFParser`, `SqlToRel`, SQL dialect/planning support, and SQL unparsing                             |
| `datafusion-optimizer`                        | Logical optimization and analyzer rules                                                             |
| `datafusion-physical-expr`                    | Executable expressions over Arrow arrays, expression properties, equivalence and ordering analysis  |
| `datafusion-physical-expr-adapter`            | Adapting physical expressions across schema differences                                             |
| `datafusion-physical-plan`                    | Physical execution operators and `ExecutionPlan`                                                    |
| `datafusion-physical-optimizer`               | Physical optimization, distribution, ordering, joins, and runtime filter placement                  |
| `datafusion-execution`                        | Runtime state, execution contexts, memory pools, disk management, object store registration         |
| `datafusion-session`                          | Session interfaces and configuration-related abstractions                                           |
| `datafusion-catalog`                          | Catalogs, schemas, tables, function and metadata interfaces                                         |
| `datafusion-catalog-listing`                  | File-based catalog and schema-discovery functionality                                               |
| `datafusion-datasource`                       | Data-source and data-sink abstractions, file scanning infrastructure                                |
| `datafusion-datasource-arrow`                 | Arrow-oriented file/data-source support                                                             |
| `datafusion-datasource-parquet`               | Specialized Parquet reading, pruning, metadata, and scan integration                                |
| `datafusion-datasource-csv`, `-json`, `-avro` | Format-specific readers and integration                                                             |
| `datafusion-pruning`                          | Reusable predicate and statistics-based pruning infrastructure                                      |
| `datafusion-functions`                        | General scalar functions: string, datetime, numeric, encoding, and related operations               |
| `datafusion-functions-aggregate`              | Aggregation implementations and registration                                                        |
| `datafusion-functions-window`                 | Ranking, windowing, and analytical functions                                                        |
| `datafusion-functions-nested`                 | Array, list, map, and struct functions                                                              |
| `datafusion-functions-table`                  | Table-valued functions                                                                              |
| `datafusion-spark`                            | Spark-compatible expression semantics and function implementations                                  |

The same workspace also contains smaller shared crates, including runtime, function-common, physical-expression-common, macro, documentation, and protocol-model components. Most are internal building blocks rather than alternative entry points.

Separation does not imply independent version compatibility. These crates share evolving Rust interfaces and generally follow the same release series. Mixing arbitrary versions of `datafusion-expr`, `datafusion-physical-plan`, and the umbrella crate can create duplicate or incompatible types.

The 55.2.0 umbrella crate depends on Arrow 59.2.0, the Rust `object_store` 0.13.2 series, and `sqlparser` 0.62.0. In particular, the separately released `datafusion-sqlparser-rs` 0.63.0 is not automatically the parser version integrated into DataFusion 55.2.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



Reference: `https://docs.rs/datafusion/55.2.0/datafusion/`

### 2.2 Companion crates with distinct integration purposes

These are especially important because they address functionality not provided by the core reexports.

| Crate                     | Purpose                                                                                                  | Boundary                                                                           |
| ------------------------- | -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `datafusion-proto`        | Serialize and deserialize logical and physical plans with Protobuf                                       | DataFusion-specific encoding; cross-version compatibility is not guaranteed        |
| `datafusion-substrait`    | Convert logical and physical plans to/from Substrait                                                     | Cross-engine plan representation, but incomplete coverage of DataFusion constructs |
| `datafusion-ffi`          | Foreign-function interfaces for catalog, table, planning, execution, and other Rust component boundaries | Explicit ABI/lifetime interoperability; not a universal plugin loader              |
| `datafusion-sqllogictest` | SQL result and regression testing infrastructure                                                         | Test infrastructure rather than a query-serving API                                |
| `datafusion-cli`          | Standalone interactive SQL client                                                                        | Useful for querying and inspecting plans, not required for embedding the engine    |

`datafusion-proto` is primarily suited to moving DataFusion-native plans between cooperating components; `datafusion-substrait` is suited to interoperability with other engines that understand Substrait. Neither establishes automatic compatibility with arbitrary versions, functions, or data-source implementations.

`datafusion-ffi` serves a different purpose: dynamically interacting with Rust code across binary boundaries without depending on the unstable native Rust ABI. Its independently published version should be verified separately from the core release.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



`https://docs.rs/datafusion-proto/latest/datafusion_proto/`

`https://docs.rs/datafusion-substrait/latest/datafusion_substrait/`

`https://docs.rs/datafusion-ffi/latest/datafusion_ffi/`

### 2.3 Dependency and feature selection

The umbrella crate enables many capabilities by default, including SQL, Parquet, compression, nested expressions, regex, Unicode, string, datetime, cryptographic, and encoding functions.

Not everything is always compiled in. The `avro`, `parquet_encryption`, `serde`, and `math_expressions` features, for example, are separately selectable in 55.2.0. A feature or subcrate being available in the ecosystem does not guarantee that it is enabled in a particular build.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Three distinct integration styles are possible:

| Integration style                | Characteristics                                                                                                              |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Umbrella `datafusion`            | Simplest access to a complete query engine; broad dependency graph                                                           |
| Umbrella with selective features | Retains the integrated experience while controlling optional functionality                                                   |
| Individual component crates      | Reuses expressions, optimization, providers, or execution infrastructure without necessarily including SQL or a full session |

The component approach is valuable for specialized engines, constrained binaries, and systems that already supply their own frontend or metadata model. Its tradeoff is greater responsibility for assembling compatible components and managing version-sensitive APIs.

Source: `https://docs.rs/crate/datafusion/55.2.0/features`

## 3. Relational representation, planning, and SQL

One of DataFusion's most consequential capabilities is that its relational model is a first-class, programmatically manipulable Rust representation, not merely an internal consequence of executing SQL.

### 3.1 The three fundamental representations

| Representation  | Meaning                                                                                                    | Typical use                                                           |
| --------------- | ---------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| `Expr`          | Typed logical expression: column references, literals, operators, functions, casts, predicates, subqueries | Construct, inspect, simplify, and rewrite calculations                |
| `LogicalPlan`   | Relational operation tree: scans, projections, filters, joins, aggregates, windows, sorts, unions          | Express data-processing semantics independently of execution strategy |
| `ExecutionPlan` | Executable operator tree, with partitioning, ordering, physical expressions, and batch streams             | Efficient physical realization of a query                             |

For example:

```
LogicalPlan
  Projection(region, total)
    Aggregate(GROUP BY region, SUM(amount) AS total)
      Filter(status = 'completed')
        TableScan(transactions)
```

The logical plan captures the relational calculation. It does not specify whether aggregation uses a hash table, whether input is partitioned, whether the filter is evaluated by a storage engine, or how many worker tasks execute it.

The physical planner and optimizer can determine those mechanisms based on source capabilities, expression properties, execution requirements, and statistics.

This separation permits independently manipulating business or analytical logic without encoding low-level execution decisions into that logic.

### 3.2 `Expr`, `DFSchema`, and semantic analysis

`datafusion-expr` and `datafusion-common` provide more than an AST of arithmetic expressions.

The logical-expression system represents SQL-aware calculation, including type information, casts, null semantics, aggregate and window expressions, function calls, and column identity.

`DFSchema` extends Arrow's physical schema model with information needed for relational name resolution, including qualified column references. DataFusion additionally carries information such as functional dependencies and constraints that can inform transformations.

The distinction is important:

- Arrow `Schema` describes the fields and types of a `RecordBatch`.
- `DFSchema` provides the richer naming and relational context necessary to interpret expressions inside a logical plan.
- `PhysicalExpr` ultimately evaluates the expression against Arrow data using resolved field positions and concrete kernels.

The logical layer also supplies tree traversal and transformation utilities, including `TreeNode`, making it possible to implement systematic rewrites instead of duplicating expression parsing or inventing a separate relational intermediate representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+1



Source: `https://datafusion.apache.org/library-user-guide/working-with-exprs.html`

### 3.3 Multiple frontends, one relational engine

SQL frontend. `DFParser` and the associated `sqlparser-rs` parse SQL into syntax trees; `SqlToRel` resolves these into DataFusion logical plans. The resulting plans pass through the same analyzer, optimizer, and execution stages as programmatically constructed queries.

DataFrame frontend. DataFrames are lazy query builders. Operations such as selection, filtering, joining, aggregation, sorting, and limiting construct a logical plan; execution occurs at terminal operations such as collection or streaming.

Direct plan construction. `LogicalPlanBuilder` permits building relational operations explicitly, including constructing plans independently from SQL. `SessionState::create_physical_plan` can then turn a resolved logical plan into executable form.

Custom frontends. An application-specific query language, parser, graphical builder, or programmatic rule system can produce `Expr` and `LogicalPlan` without necessarily involving SQL parsing.

This is a significant source of architectural leverage: multiple ways of expressing computation can share one optimizer, execution engine, and connector model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+2



Sources:

`https://datafusion.apache.org/library-user-guide/building-logical-plans.html`

`https://datafusion.apache.org/user-guide/dataframe.html`

`https://docs.rs/datafusion-sql/55.2.0/datafusion_sql/`

### 3.4 SQL coverage and relational capabilities

DataFusion supports a substantial analytical SQL surface, including joins, aggregation, set operations, common table expressions, subqueries, window functions, nested data types, analytical functions, and prepared statements.

Relevant capabilities span several categories.

| Category              | Capabilities and distinctions                                                                               |
| --------------------- | ----------------------------------------------------------------------------------------------------------- |
| Relational operations | Filtering, projection, joins, `GROUP BY`, `HAVING`, sorting, limits, distinct, set operations               |
| Advanced aggregation  | Grouping sets, rollups, cubes, ordered and distinct aggregations, approximate aggregates                    |
| Subqueries            | Scalar, `IN`, `EXISTS`, and correlated forms where supported; eligible forms may be decorrelated into joins |
| Windows               | Ranking, navigation, aggregate windows, partitions, ordering, window frames                                 |
| Nested data           | Arrow lists, structs, maps, nested expressions and accessors                                                |
| SQL metadata          | `SHOW`, `information_schema`, function introspection, configuration inspection                              |
| Parameterization      | Prepared statements and parameter substitution                                                              |
| Writing and mutation  | `COPY`, `INSERT` and provider-dependent mutation interfaces                                                 |
| SQL generation        | `datafusion-sql` unparser can convert supported logical plans and expressions back to SQL                   |

The unparser is particularly interesting for federated systems, SQL rewriting, and diagnostic tools. It makes bidirectional movement between logical relational representations and SQL possible, subject to the constructs supported by the unparser and target dialect. It is not a guarantee that every arbitrary optimized or extended plan can be reconstructed as equivalent SQL.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+3



Source: `https://docs.rs/datafusion/55.2.0/datafusion/sql/unparser/`

### 3.5 DDL, DML, and the limits of database semantics

DataFusion parses and plans database-like statements, but persistence and mutation depend on the registered providers.

For example, `INSERT INTO` can be expressed at the relational level while the table implementation determines how rows are actually stored.

Version 55 introduced planning support for `MERGE INTO`, with a `TableProvider` extension mechanism for its execution. That does not mean the built-in providers acquired fully implemented transactional merges.

Similarly, table constraints are represented in the catalog model, including primary-key, unique, foreign-key, and check constraints, but they are not generally enforced at runtime. They cannot be treated as equivalent to a transactional database's integrity guarantees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog+1



Source: `https://datafusion.apache.org/library-user-guide/table-constraints.html`

### 3.6 The importance of query-plan reuse

The availability of a typed relational IR opens several general possibilities: expressing recurring transformations as plans rather than strings, composing transformations without repeatedly materializing intermediates, analyzing dependencies and output schemas before execution, and applying optimizer rewrites across otherwise separate operations.

There are qualifications. Plan reuse must account for catalog evolution, resolved schema references, parameter values, function registry changes, execution-time state, and version compatibility. A logical plan is not automatically a durable, immutable, environment-independent specification.

## 4. Optimization as a reusable library capability

DataFusion's optimization machinery is an independently extensible subsystem spanning semantic normalization, algebraic rewrites, physical operator choice, execution properties, and runtime information.

### 4.1 Analyzer, logical optimizer, and physical optimizer

| Stage              | Representative capabilities                                                                                                      | Principal crate                         |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| Analyzer           | Type coercion, grouping-function resolution, semantic preparation                                                                | `datafusion-optimizer`                  |
| Logical optimizer  | Expression simplification, predicate/projection/limit pushdown, subquery decorrelation, join elimination, CSE, distinct rewrites | `datafusion-optimizer`                  |
| Physical planner   | Choose executable operators, physical expressions, scan implementations, aggregation and join strategies                         | Planning interfaces and physical crates |
| Physical optimizer | Enforce partitioning and ordering, improve joins, push runtime filters, optimize sorts and limits, validate requirements         | `datafusion-physical-optimizer`         |

These passes are not merely cosmetic reductions in plan size. They change how much input must be read, how much data moves between partitions, when operators can start producing output, and how much state must be retained.

The 55.2.0 API reference enumerates the default rule pipeline, including `common_sub_expression_eliminate`, `push_down_filter`, `optimize_projections`, `join_selection`, `WindowTopN`, `EnsureRequirements`, `TopKRepartition`, `PushdownSort`, and `SanityCheckPlan`. The exact sequence is version-dependent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: `https://docs.rs/datafusion/55.2.0/datafusion/#built-in-optimizer-rules`

### 4.2 Predicate, projection, and limit pushdown

DataFusion can move relational operations closer to source scans when semantics permit.

Projection pushdown reduces the columns read or decoded. Predicate pushdown can reduce the number of rows, files, row groups, or pages accessed. Limit pushdown may permit early termination.

These mechanisms interact rather than operating independently. A pushed predicate that requires a column not present in the final projection must still make that column available for evaluation; a source able to exploit predicate constraints can be substantially cheaper than applying the same expression after a full scan.

The distinction between logical pushdown and physical source pushdown matters. The logical optimizer may move a filter next to a scan without that scan being capable of executing the filter internally. The provider determines how much of the source-level operation it can actually perform.

### 4.3 Join planning

DataFusion supports multiple physical join implementations and join optimizations, including hash and sort-merge strategies, semi and anti joins, cross joins, join elimination, and optimizations involving null-rejecting predicates.

Join planning depends on row estimates, data partitioning, ordering, boundedness, and available memory. Moving or eliminating a join can avoid substantial computation, while a poor estimate can produce an expensive build-side selection or unnecessary repartition.

Version 55 adds additional semi-join transformations and outer-join elimination improvements. Its physical optimizer also includes `HashJoinBuffering`, intended to improve the execution pipeline around build/probe coordination.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog+1



### 4.4 Equivalence properties, ordering, and partitioning

One of DataFusion's more advanced features is its explicit representation of properties of the data produced by physical operators.

These include equivalences between expressions, ordering guarantees, partitioning, and requirements imposed by downstream operators.

Consider a dataset already ordered by `(customer_id, timestamp)`. A downstream window operation may be able to exploit that order without a fresh sort. Similarly, if input is already partitioned by the correct join keys, a repartitioning exchange may be unnecessary.

The `EquivalenceProperties`, `Partitioning`, and `PlanProperties` machinery makes those facts available to the physical optimizer.

DataFusion 55 adds native `Partitioning::Range` support, describing partitions by key ranges rather than only hash-based partition membership. When range boundaries are known and valid, an engine can exploit them to preserve useful distribution or reduce work during a query.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog+1



This property-based approach is especially valuable for custom operators. A specialized operator can participate in broad optimizations without requiring every other optimizer rule to recognize its implementation details—provided its declared properties are correct.

Incorrect ordering, equivalence, or partitioning metadata can be worse than missing metadata: it can cause the optimizer to remove operations actually required for correct results.

### 4.5 Sort optimization and Top-K

DataFusion distinguishes full sorting from selecting a small ordered prefix.

An `ORDER BY ... LIMIT` can often be implemented with bounded candidate state rather than materializing and sorting the entire input. Its optimizer also tries to exploit existing order, push sorting requirements toward providers, and apply limit reductions earlier.

An advanced distinction is between exact and inexact sort pushdown. If a source proves that it supplies the required global ordering, a redundant sort may be eliminated. If the source only supplies a useful approximate or locally favorable order, the final sort remains necessary, but less processing may be required.

DataFusion also propagates runtime Top-K thresholds toward Parquet scans. As a better candidate threshold is found, row groups that cannot improve the answer can be skipped. Version 55 extends this with runtime row-group pruning and compound ordering support.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog+2



Sources:

`https://datafusion.apache.org/blog/2026/07/20/sort-pushdown/`

`https://datafusion.apache.org/blog/2025/09/10/dynamic-filters/`

### 4.6 Dynamic filters and runtime optimization

Most query optimization occurs before execution. Dynamic filtering introduces information discovered while execution is progressing.

For example, a hash join's build side can reveal which key values or ranges are relevant to its probe side. The resulting dynamic predicate can be propagated toward a scan, reducing reads that would ultimately produce no matches.

Similarly, a Top-K operator can discover progressively tighter bounds and feed them back to an upstream scan.

This changes the traditional model from an entirely static execution graph to one in which certain operators can communicate selective runtime information without rebuilding the entire plan.

The opportunity is especially significant for large, selectively accessed columnar datasets. The limitation is equally important: runtime predicates must preserve query semantics, operate within the source's supported pruning capabilities, and account for the timing of information availability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog



### 4.7 Statistics-driven computation

Statistics have several roles beyond join cardinality estimation.

They can inform scan pruning, physical join selection, partitioning, and in certain cases allow DataFusion to answer an aggregate from exact metadata without scanning the underlying rows. The physical `aggregate_statistics` rule explicitly targets such opportunities.

DataFusion also provides reusable statistics and pruning APIs for sources that possess their own metadata.

An important semantic distinction is exact versus approximate information. Metadata that safely proves a value range or row count can justify skipping data or answering a calculation. Estimated statistics can inform cost decisions, but cannot justify transformations requiring exact truth.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.8 Extensible optimization

Applications can add `OptimizerRule`, `AnalyzerRule`, or `PhysicalOptimizerRule` implementations and customize the planning pipeline.

This allows specialized rewrites, including replacing relational operators with precomputed summaries, exploiting domain indexes, or applying source-aware transformations.

For example, the official extension guide describes an optimizer that recognizes temporal aggregation patterns, obtains an equivalent result from a specialized index, and substitutes a `MemTable`-backed scan.

The more general opportunity is to make a specialized computation available through the same logical model as ordinary SQL, rather than introducing an entirely separate execution path. The risk is semantic correctness: a rewrite must preserve results, types, null behavior, and other observable properties.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+1



Sources:

`https://datafusion.apache.org/library-user-guide/query-optimizer.html`

`https://datafusion.apache.org/library-user-guide/extending-operators.html`

## 5. Data access, catalogs, and storage integration

DataFusion's data access architecture separates what constitutes a queryable relation from how the underlying bytes are stored and retrieved.

### 5.1 Catalog hierarchy

DataFusion has a hierarchical metadata model:

```
CatalogProviderList
  └── CatalogProvider
       └── SchemaProvider
            └── TableProvider
```

A catalog can contain multiple schemas; each schema can expose multiple relations. Table names resolve through this hierarchy, permitting an application to present independently managed datasets under a unified namespace.

The implementation need not use DataFusion's in-memory catalog. Custom catalog and schema providers can expose metadata from files, object stores, remote databases, or other services. The table-resolution interface supports asynchronous lookup.

This provides a useful distinction between relational discovery and physical data access. A catalog supplies identities and definitions; a table provider supplies data access and execution planning.

DataFusion can additionally expose `information_schema` views for inspection of tables, columns, functions, and configuration. Such views are descriptions of the registered catalog, not a guarantee of centralized durable metadata storage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+1



Sources:

`https://datafusion.apache.org/library-user-guide/catalogs.html`

`https://datafusion.apache.org/user-guide/sql/information_schema.html`

### 5.2 `TableProvider`: the core integration contract

`TableProvider` is one of the most consequential interfaces in the ecosystem.

It exposes a relation's schema and type, and supplies a physical execution plan when the engine scans it. The scan request can carry a projection, predicates, and a limit.

Conceptually:

```
async fn scan(
    &self,
    state: &dyn Session,
    projection: Option<&[usize]>,
    filters: &[Expr],
    limit: Option<usize>,
) -> Result<Arc<dyn ExecutionPlan>>;
```

This is an illustrative signature matching the documented extension pattern; exact associated methods and generic bounds should be taken from the version-pinned trait documentation.

The provider is therefore not just a collection of records. It participates in query planning.

For example, a relational engine backed by a remote SQL database might translate a DataFusion predicate into source SQL, request only selected columns, and delegate supported aggregations or operations to the remote system. Conversely, a simulation or computed-data provider could generate Arrow batches directly without any file or database storage.

The central architectural opportunity is to expose multiple heterogeneous sources through a common relational interface without forcing callers to understand their native access mechanisms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://docs.rs/datafusion/55.2.0/datafusion/catalog/trait.TableProvider.html`

### 5.3 Pushdown guarantees: `Exact`, `Inexact`, `Unsupported`

The provider explicitly advertises whether it can evaluate proposed predicates.

| Response      | Contract                                                         | Engine consequence                        |
| ------------- | ---------------------------------------------------------------- | ----------------------------------------- |
| `Exact`       | Source guarantees every emitted row satisfies the predicate      | A separate residual filter is unnecessary |
| `Inexact`     | Source can reduce candidate data but may return nonmatching rows | DataFusion retains a residual filter      |
| `Unsupported` | Source cannot use the predicate                                  | DataFusion evaluates the predicate itself |

This is not merely a performance hint; it has direct correctness implications.

A source that skips entire Parquet row groups using min/max statistics but does not filter individual rows generally offers an inexact predicate pushdown unless it can prove every surviving row satisfies the condition. Advertising exact pushdown incorrectly can cause silent wrong results because DataFusion is permitted to remove the residual filter.

Projection and limit pushdown likewise require accurate preservation of the schema and requested semantics. Physical limit pushdown is sensitive to global-versus-partitioned execution; a local limit does not automatically implement an ordered global top-N.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



### 5.4 Built-in table implementations

| Starting point                             | Best-described role                                                               |
| ------------------------------------------ | --------------------------------------------------------------------------------- |
| `MemTable`                                 | Existing Arrow `RecordBatch` data; simple in-memory relations                     |
| Streaming table providers                  | Batches supplied on demand by a stream or stream factory                          |
| `ViewTable`                                | A relation defined by another logical plan                                        |
| `ListingTable`                             | Collections of files, potentially across directories or object stores             |
| Custom `TableProvider`                     | Database, generated data, external API, specialized index, or nonstandard storage |
| Table-valued function returning a provider | Parameterized relation discovery or generation                                    |

The important difference between `ViewTable` and a materialized result is that a view represents computation. It does not inherently imply storing or caching its output.

Likewise, a streaming provider does not by itself establish the event-time, watermark, checkpoint, or exactly-once guarantees associated with a dedicated stream-processing platform.

For file-backed sources, `ListingTable` already supplies substantial infrastructure: discovering files, interpreting partition directories, integrating with object stores, inferring schemas, and constructing scans. A custom file format can often reuse that machinery by implementing `FileFormat`, `FileSource`, and `FileOpener` rather than creating an entire provider and execution engine from scratch.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+1



### 5.5 Parquet: a particularly advanced subsystem

Parquet integration is much deeper than decoding a columnar file into Arrow batches.

`datafusion-datasource-parquet` exposes a specialized reader, `ParquetSource`, metadata infrastructure, file-reader factories, schema adaptation, and mechanisms for pruning the portions of files that cannot contribute to the result.

| Mechanism                            | What it avoids or enables                                                          |
| ------------------------------------ | ---------------------------------------------------------------------------------- |
| File and partition pruning           | Avoids reading entire irrelevant files or partition directories                    |
| Row-group statistics pruning         | Skips row groups using min/max and related metadata                                |
| Bloom-filter pruning                 | Can prove that searched values are absent                                          |
| Page-index pruning                   | Avoids pages within otherwise relevant row groups                                  |
| Row filtering / late materialization | Filters during decoding to avoid decoding unnecessary values                       |
| Projection pushdown                  | Restricts reading and decoding to required columns                                 |
| Nested leaf projection               | Avoids unused fields nested within structs and lists                               |
| Schema reconciliation                | Reads compatible Parquet files with different physical schemas                     |
| Parallel file and row-group reads    | Exploits concurrency across files and within sufficiently large files              |
| Custom reader factories              | Supports alternative metadata caching and I/O behavior                             |
| Explicit `ParquetAccessPlan`         | Accepts external information specifying which row groups or row selections to read |

The final item deserves emphasis. `ParquetAccessPlan` can bridge DataFusion's native Parquet reader with external indexes or other independent metadata systems.

An external index can determine that only particular portions of a Parquet file are relevant. It can provide that selection to DataFusion, which can then further narrow the scan using its own statistics and predicates.

This may avoid writing a bespoke low-level Parquet reader while still exploiting specialized indexing structures.

Version 55 also added nested leaf pruning and improved runtime pruning behavior, which matters for wide nested datasets and selective Top-K queries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

`https://docs.rs/datafusion/55.2.0/datafusion/datasource/physical_plan/struct.ParquetSource.html`

`https://docs.rs/datafusion-datasource-parquet/55.2.0/datafusion_datasource_parquet/`

### 5.6 Object stores and remote I/O

DataFusion uses the Rust `object_store` abstraction, with registration through its execution environment, to access data without embedding storage-provider-specific logic into every operator.

This supports a consistent scan architecture across local and remote storage backends supported by the registered `ObjectStore` implementation.

The separation allows I/O behavior, authentication, caching, and object-store connection lifecycles to remain outside relational query semantics.

For remote storage, performance is often constrained as much by metadata requests, object listing, range reads, and network latency as by Arrow computation. DataFusion's parallel scan infrastructure, metadata caches, pruning, and I/O configuration provide mechanisms to reduce those costs.

Object-store interoperability does not automatically provide transaction coordination, atomic multi-file commits, or lakehouse snapshot semantics. Those require an additional table-format or storage-management layer.

### 5.7 Data writing and sinks

DataFusion can write query results to Arrow, Parquet, CSV, and JSON formats, including partitioned output through `COPY` and supported DataFrame writer facilities.

The underlying sink abstractions include `DataSink` and format-specific implementations.

Parquet writing exposes controls for compression, encoding, statistics, bloom filters, row-group sizing, buffering, and parallelism. These choices influence both write throughput and the performance of subsequent queries.

There is an important interaction here: files written with useful row-group boundaries, statistics, and appropriate physical clustering may be much more effectively pruned than files optimized only for compression or sequential writing.

A further advanced option is Parquet content-defined chunking, documented as experimental and disabled by default. Its purpose is to improve chunk-level deduplication opportunities, at a cost in write-path complexity and performance characteristics; it is not a general requirement for efficient Parquet writing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+1



Sources:

`https://datafusion.apache.org/user-guide/sql/dml.html`

`https://datafusion.apache.org/user-guide/configs.html`

## 6. Execution engine, parallelism, and physical operators

### 6.1 Arrow-native, batch-oriented execution

DataFusion operates on Arrow arrays and `RecordBatch` streams, using columnar kernels for much of its computation.

The fundamental execution interface is:

```
fn execute(
    &self,
    partition: usize,
    context: Arc<TaskContext>,
) -> Result<SendableRecordBatchStream>;
```

An `ExecutionPlan` can produce multiple partitions. Each partition supplies an asynchronous stream of batches, and its downstream consumers pull from that stream.

This architecture enables pipelining: scanning, filtering, projection, and other operators can process batches incrementally rather than waiting for an entire relation to be materialized.

It also provides a reusable building block for applications that already consume Arrow, avoiding a separate row-oriented intermediate representation in many integration paths.

However, Arrow-native does not imply universal zero-copy execution. Filtering, casts, joins, aggregation, sorting, dictionary transformations, and conversion between encodings may allocate or copy. Zero-copy is possible at suitable boundaries, not a blanket guarantee of every operator.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: `https://docs.rs/datafusion/55.2.0/datafusion/physical_plan/trait.ExecutionPlan.html`

### 6.2 Execution operators and mechanisms

| Operator family               | Important mechanisms                                                                 |
| ----------------------------- | ------------------------------------------------------------------------------------ |
| Scan                          | File scans, in-memory scans, streams, custom source plans                            |
| Selection and projection      | Vectorized boolean filtering, column mapping, computed expressions                   |
| Aggregation                   | Hash/grouped aggregation, partial and final aggregation, specialized aggregate state |
| Joins                         | Hash joins, sort-merge joins, nested-loop and other applicable strategies            |
| Ordering                      | Sort, partial sort, sort-preserving merge, Top-K                                     |
| Windows                       | Partitioned and ordered window processing, ranking, frames                           |
| Distribution                  | Repartition, coalescing, partition-preserving operations                             |
| Set and relational operations | Union, distinct, limit, intersect/except planning and corresponding execution        |
| Sinks                         | Write batches to files or custom destinations                                        |

Physical operators have different memory and streaming characteristics. Projection and filtering can often begin emitting immediately. Sorting may need to accumulate substantial input before producing output. Hash joins require build-side state; some window operations require buffering.

These differences are represented in physical-plan metadata, including boundedness and emission behavior. Consequently, execution-planning decisions concern not only CPU efficiency but also memory growth, time-to-first-result, and whether a plan is valid for unbounded input.

### 6.3 Parallel execution and repartitioning

DataFusion exploits parallelism through execution partitions rather than requiring each logical relation to be processed by one task.

`RepartitionExec` redistributes data across partitions; `CoalescePartitionsExec` combines partitioned streams when downstream execution needs fewer streams.

The default target partition count generally follows the available CPU cores, but the useful degree of parallelism depends on the scan, operators, file layout, and workload.

Too few partitions can underutilize hardware. Too many can increase scheduling overhead, buffering, coordination, and repartitioning costs.

A particularly valuable possibility is to retain a source's existing partitioning throughout a query, rather than repeatedly redistributing equivalent keys. This depends on accurate physical properties and the compatibility of each downstream operator's requirements.

### 6.4 Tokio and asynchronous scheduling

DataFusion uses Tokio to schedule query execution. Its streams combine asynchronous I/O with CPU-intensive Arrow processing and rely on cooperative scheduling.

This is a productive combination for analytical workloads involving both remote data access and computation, but there is a material caveat.

CPU-bound work can delay latency-sensitive I/O tasks if both compete on the same runtime. The official architecture documentation discusses the use of separate Tokio runtimes for network and CPU-intensive work under concurrent load.

The distinction is not simply asynchronous versus synchronous execution. It is about how effectively computation yields and whether the scheduling environment preserves network progress and fair access to CPU capacity.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.5 Custom `ExecutionPlan` implementations

Custom physical plans expose much more than an `execute` method.

They can declare required input ordering and distribution, whether input order is preserved, whether repartitioning would be beneficial, the effects on cardinality, available statistics, limit pushdown opportunities, dynamic filter behavior, and metrics.

Version 55 includes additional hooks such as `apply_expressions` and `dynamic_expressions_produced`, improving optimizer visibility into specialized operators and supporting more sophisticated distributed-engine integrations.

The most important principle is that these declarations are optimization contracts, not annotations for documentation. They determine whether DataFusion inserts or removes physical operators.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 6.6 Streaming and unbounded relations

DataFusion can execute incremental pipelines and accepts potentially unbounded input through streaming-oriented providers.

However, the capability must be distinguished from a comprehensive streaming data-processing system.

An operator may accept unbounded input while another cannot produce useful output without seeing the end of a partition. For example, global sorting of an infinite relation cannot complete using normal finite-input semantics.

Specialized join or window plans may also require pruning or bounded state to remain viable.

DataFusion's plan properties and optimizer checks help reason about such compatibility, but watermark advancement, event-time completeness, durable checkpoints, recovery semantics, and exactly-once processing are not automatically supplied by the core library.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 7. Function systems and computational extensibility

Functions are a separate major extension surface, not just a way to call Rust code from SQL.

DataFusion exposes type signatures, logical function identities, vectorized execution, aggregate state, window evaluation, and function-specific optimizer metadata.

### 7.1 Function families

| Function family       | Principal interface                   | Computational model                                                     |
| --------------------- | ------------------------------------- | ----------------------------------------------------------------------- |
| Scalar                | `ScalarUDFImpl`                       | One output value per row, typically evaluated over Arrow arrays         |
| Aggregate             | `AggregateUDFImpl`, `Accumulator`     | Aggregate state across rows in a group                                  |
| Multi-group aggregate | `GroupsAccumulator`                   | Efficiently manages state for many grouping keys together               |
| Window                | `WindowUDFImpl`, `PartitionEvaluator` | Evaluate values relative to a window partition and frame                |
| Table-valued          | `TableFunctionImpl`                   | Return a queryable `TableProvider` from function arguments              |
| Async scalar          | `AsyncScalarUDFImpl`                  | Perform asynchronous operations within a scalar function execution path |

Functions can generally be invoked through SQL and, where the corresponding expression constructors are available, programmatic logical expressions.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+1



`https://datafusion.apache.org/library-user-guide/functions/adding-udfs.html`

### 7.2 Scalar functions as optimizer-aware expressions

`ScalarUDFImpl` supplies the name, type signature, return type or field information, and evaluation behavior of a function.

Input and output types alone are not the entire contract. Functions can communicate semantic properties that affect optimization.

| Property                         | Why it matters                                                                        |
| -------------------------------- | ------------------------------------------------------------------------------------- |
| Volatility                       | Determines when expressions may be treated as repeatable or safely simplified         |
| Null propagation / strictness    | Allows reasoning about predicates that reject null-extended rows                      |
| Output nullability               | Supports type and schema correctness                                                  |
| Ordering behavior                | May allow an order-preserving transformation to avoid subsequent sorting              |
| Function-specific simplification | Permits a call to be rewritten into a simpler or more canonical expression            |
| Argument coercion and signatures | Centralizes type compatibility rather than requiring handwritten checks at call sites |

Version 55 explicitly improved APIs for declaring null-strict and strictly order-preserving scalar functions.

For example, when an order-preserving transformation of a sorted column is accurately declared, the optimizer may avoid an otherwise redundant sort.

These are significant capabilities for advanced computation embedded in relational pipelines. Conversely, incorrect claims about volatility or ordering can change results or produce invalid physical plans.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: `https://docs.rs/datafusion/55.2.0/datafusion/logical_expr/trait.ScalarUDFImpl.html`

### 7.3 Aggregates: `Accumulator` versus `GroupsAccumulator`

Custom aggregates maintain state across multiple rows.

A basic `Accumulator` handles one group and exposes operations for incorporating input, merging partial state, and producing results.

That mergeable state representation allows aggregation to be distributed across partitions:

```
Input partitions
      ↓
Partial aggregate state
      ↓
State exchange / merge
      ↓
Final aggregate
```

An advanced alternative is `GroupsAccumulator`, which manages state for many groups together, often using densely indexed vectors rather than a separately allocated object per group.

This can substantially reduce overhead for high-cardinality groupings, depending on the aggregate's implementation and types.

The underlying architectural distinction is between a per-group abstraction optimized for simplicity and a multi-group abstraction optimized for vectorization, locality, and reduced allocation overhead.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/datafusion-expr/55.2.0/datafusion_expr/trait.GroupsAccumulator.html`

### 7.4 Stateful and structured aggregate outputs

Aggregate output need not be a simple numeric scalar.

A function can maintain state with multiple measures and emit an Arrow `Struct`, allowing downstream relational operations to access individual fields.

This makes it possible to represent composite aggregate results—such as statistics with count, bounds, mean, variance, or diagnostic measures—inside the same typed relational system.

Where partial aggregation is needed, the internal merge state should be distinguished from the externally visible finalized result. A suitable output type does not by itself make the aggregation mergeable, associative, or numerically stable.

The broader opportunity is to express rich reductions through DataFusion's native aggregation machinery instead of implementing separate group-by loops and per-group result assembly.

### 7.5 Window UDFs

Custom window functions use `PartitionEvaluator` and related interfaces to evaluate results over partitions, ordering, and window frames.

The interface supports different evaluation methods, including more efficient whole-partition or rank-oriented paths when the calculation permits.

The critical tradeoff is algorithmic complexity. Recomputing a moving statistic across the full frame for every row can be much more expensive than maintaining incremental state or using a specialized evaluation method.

Window functions therefore benefit from understanding both the evaluation interface and the physical requirements for sorted and partitioned inputs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



### 7.6 Table-valued functions as parameterized relations

A table-valued function returns a `TableProvider`, not simply an array.

That makes a table function a particularly powerful extension mechanism. It can expose a generated series, parameterized dataset, external service query, or computed domain relation while retaining participation in relational joins, filters, projections, and aggregation.

The distinction from a scalar UDF is architectural: the result is a relation with a schema, not one value per input row.

This can remove the need for bespoke APIs that retrieve and reshape intermediate records before handing them to the relational engine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



### 7.7 Async functions and external operations

Asynchronous scalar UDF support allows operations involving network access or other asynchronous work to participate in a query.

The interface includes `AsyncScalarUDFImpl` and batch-oriented invocation, with options for declaring an ideal batch size.

This is useful when computation genuinely requires remote inference, enrichment, or lookup. It is not necessarily the most efficient path for high-volume data access. A `TableProvider` with source pushdown or a batched relational join may provide better control over concurrency, caching, retries, and query planning.

The distinction matters because introducing I/O into scalar expressions changes the latency and failure characteristics of a query pipeline.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



### 7.8 Extending SQL itself

Custom functions are only part of SQL extensibility.

DataFusion provides three planner extension families:

| Planner           | Scope                                                                                 |
| ----------------- | ------------------------------------------------------------------------------------- |
| `ExprPlanner`     | SQL expressions, operators, literals, field access, and specialized function handling |
| `TypePlanner`     | Mapping SQL type declarations to Arrow fields and types                               |
| `RelationPlanner` | Extending the interpretation of relations in `FROM` clauses                           |

This allows custom syntactic constructs to map onto existing DataFusion relational operations, potentially without introducing any new physical operator.

It also permits domain-specific SQL types, including using Arrow extension metadata or alternative physical representations where appropriate.

A distinction should be maintained between parsing a syntax form, assigning it logical meaning, and providing an executable implementation. Supporting one layer does not automatically provide the others.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://datafusion.apache.org/library-user-guide/extending-sql.html`

## 8. Runtime state, resource management, and execution control

DataFusion separates per-session configuration from the lower-level runtime resources used to execute queries. This permits multiple sessions to share infrastructure while retaining different catalogs, functions, and query settings.

### 8.1 Session and runtime architecture

| Component        | Responsibility                                                                                          |
| ---------------- | ------------------------------------------------------------------------------------------------------- |
| `SessionContext` | High-level entry point for SQL, DataFrames, catalog registration, and execution                         |
| `SessionState`   | Resolved catalog/function environment, planner, optimizer, and session configuration                    |
| `SessionConfig`  | Query-planning and execution settings, including partition counts and optimizer controls                |
| `RuntimeEnv`     | Shared runtime resources, including memory management, temporary storage, and object-store registration |
| `TaskContext`    | Context passed into execution tasks and partition streams                                               |
| `MemoryPool`     | Tracks memory reservations and supports bounded execution                                               |
| `DiskManager`    | Manages temporary storage used by operators, including spilling                                         |

One useful consequence is that query sessions do not need to own separate physical infrastructure. A host can supply a shared `RuntimeEnv` while giving sessions distinct query configuration or catalog visibility.

Conversely, a session is not automatically an isolation boundary. Shared resources, shared providers, and externally managed connections may require their own concurrency limits and authorization rules.

### 8.2 Memory reservations and spilling

DataFusion tracks operator memory reservations through its memory-pool abstractions. Memory-intensive operators, such as sorting, hash joins, and aggregations, can use those reservations to control how much state they retain.

When supported operators cannot reserve additional memory, they can spill intermediate data to temporary storage.

This provides two different performance regimes:

| Regime    | Execution characteristics                                                                                                      |
| --------- | ------------------------------------------------------------------------------------------------------------------------------ |
| In-memory | Avoids serialization and temporary I/O; performance depends on available memory                                                |
| Spilling  | Permits some workloads larger than their memory budget, but introduces additional I/O, encoding, merging, and scheduling costs |

Spilling should not be interpreted as a guarantee that every query can execute within any chosen memory limit. Operators differ in their ability to spill, and certain buffers, allocations, or states may not be managed by the reservation pool.

Version 55 made spill behavior more resilient, particularly for aggregates and external sorting under pressure. It also introduced pluggable spill backends through `SpillFile` and `TempFileFactory`.

An embedding application can therefore integrate temporary execution storage with an existing storage layer instead of always using ordinary operating-system temporary files. The upstream example demonstrates an object-store-backed implementation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog



Source: `https://datafusion.apache.org/blog/2026/08/25/datafusion-55.0.0/`

### 8.3 Important runtime controls

The principal controls include:

| Setting or mechanism                         | Purpose                                                               |
| -------------------------------------------- | --------------------------------------------------------------------- |
| `datafusion.execution.target_partitions`     | Requested execution parallelism                                       |
| `datafusion.execution.batch_size`            | Preferred Arrow batch granularity                                     |
| `datafusion.runtime.memory_limit`            | Bounds managed query-execution memory                                 |
| `datafusion.runtime.temp_directory`          | Controls spill-file location                                          |
| `datafusion.runtime.max_temp_directory_size` | Bounds configured temporary storage usage                             |
| `datafusion.runtime.max_spill_merge_fan_in`  | Limits merge fan-in for external sorting                              |
| Metadata and file statistics caches          | Reduce repeated metadata reads and planning work                      |
| Object-store configuration                   | Influences parallel I/O, buffering, connections, and request behavior |

The relationships are not monotonic. Larger batches can reduce per-batch overhead but increase memory pressure. Greater partition counts may increase parallelism but also increase the number of buffers and tasks. Caching metadata can reduce remote requests at the expense of memory and potentially freshness.

Resource configuration should therefore be understood alongside the physical plan, not as a separate performance-tuning problem.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://datafusion.apache.org/user-guide/configs.html`

### 8.4 Security and untrusted SQL

DataFusion does not claim to be a security sandbox for arbitrary SQL.

The application is responsible for deciding what catalogs, providers, functions, files, and operations a query may access.

`SQLOptions` and `SessionContext::sql_with_options` can reject categories of statements, including DDL and DML. SQL access to files by URL or local path is opt-in through `enable_url_table()`.

Importantly, the memory limit controls the DataFusion execution pool, not total process memory. Hard process or tenant isolation requires surrounding application and operating-system controls.

Likewise, custom UDFs and providers can execute ordinary Rust code with the permissions of the host process. A SQL allowlist does not confine their behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://datafusion.apache.org/library-user-guide/securing-datafusion.html`

## 9. Observability, introspection, and testing

DataFusion exposes information at several levels: the relational plan, physical operator graph, runtime metrics, and source-specific scan diagnostics.

### 9.1 Plan introspection

`EXPLAIN` exposes the planned computation without running the query, while `EXPLAIN ANALYZE` executes it and attaches runtime measurements.

`EXPLAIN VERBOSE` can reveal intermediate transformations, helping distinguish logical rewrites from physical planning decisions.

DataFusion supports multiple plan display formats, including `tree`, `indent`, and PostgreSQL-inspired `pgjson`. The latter permits integration with visualization tooling that understands PostgreSQL-style JSON explain structures.

For programmatic applications, the underlying plan trees and display utilities can be accessed through Rust rather than parsing human-readable explain output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://datafusion.apache.org/user-guide/sql/explain.html`

### 9.2 Execution metrics

DataFusion's metric system supplies common metrics for many physical operators and additional measurements for specialized operators.

| Metric family                   | Diagnostic value                                             |
| ------------------------------- | ------------------------------------------------------------ |
| `elapsed_compute`               | Time spent actively computing inside an operator             |
| `output_rows`, `output_batches` | Cardinality and batch-flow analysis                          |
| `output_bytes`                  | Estimated memory volume of output batches                    |
| Filter selectivity              | Distinguishes effective predicates from low-value filters    |
| Hash join metrics               | Build/probe costs, hash table memory, fanout characteristics |
| Aggregate metrics               | Aggregation state and execution behavior                     |
| Parquet pruning metrics         | Files, row groups, pages, or rows avoided                    |
| Spill and memory metrics        | Pressure from sorting, joining, and aggregation              |

An important qualification is that `output_bytes` can count shared Arrow buffers multiple times across different output batches. It should not be equated with unique allocated memory or total process RSS.

Likewise, summing operator compute times does not necessarily yield wall-clock query duration because operators may execute concurrently.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://datafusion.apache.org/user-guide/metrics.html`

### 9.3 Diagnosing the entire pipeline

Different forms of introspection answer different questions.

| Question                              | Most relevant evidence                                                      |
| ------------------------------------- | --------------------------------------------------------------------------- |
| Why was a filter not pushed down?     | Logical plan, provider pushdown contract, optimized physical plan           |
| Why is a query reading too much data? | Parquet pruning metrics, file metadata, residual filters                    |
| Why was an exchange inserted?         | Partitioning properties and physical requirements                           |
| Why is sorting expensive?             | Ordering properties, sort input cardinality, spill metrics                  |
| Why is a join slow?                   | Join strategy, build-side size, selectivity, repartitioning                 |
| Why does planning take too long?      | Logical optimizer transformations and planning instrumentation              |
| Why does memory grow?                 | Operator reservations, spill activity, runtime allocations outside the pool |

The particularly valuable interaction is between plan structure and execution evidence. A slow operator is not necessarily the cause of poor performance. An unnecessary upstream scan or repartition may produce most of the work observed downstream.

### 9.4 Testing and correctness infrastructure

The workspace includes `datafusion-sqllogictest`, along with unit, integration, SQL regression, and optimizer testing infrastructure.

This matters for extension development because testing only the output of one example query is insufficient to validate an optimizer or physical operator.

Custom operators must remain correct under different batch sizes, partitions, null distributions, empty inputs, and optimizer transformations. Custom aggregate implementations additionally need to preserve semantics when partial state is merged in different orders.

Floating-point aggregation deserves particular care: different partitioning or merge orders can legitimately change the last bits of numerical results because floating-point addition is not associative.

Source: `https://datafusion.apache.org/contributor-guide/testing.html`

## 10. Serialization, FFI, and interoperability with other engines

The DataFusion ecosystem supports three meaningfully different integration mechanisms: Arrow data interchange, query-plan serialization, and binary component interoperability.

### 10.1 Native DataFusion plans versus Substrait

| Mechanism                 | Intended role                                                    | Important constraint                                                     |
| ------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `datafusion-proto`        | Exchange or persist DataFusion logical and physical plans        | Not guaranteed compatible between DataFusion versions                    |
| `datafusion-substrait`    | Exchange relational plans with other Substrait-aware systems     | Does not cover every DataFusion expression and plan construct            |
| Arrow IPC                 | Exchange Arrow arrays and batches                                | Transfers data, not the logical semantics of a query                     |
| Arrow Flight / Flight SQL | Transport Arrow data and SQL interactions over network protocols | Requires a server/client implementation, not merely a DataFusion session |

`datafusion-proto` serializes `LogicalPlan` and `ExecutionPlan`, including their expression representations, into Protobuf-based binary formats.

It can be useful for shipping a plan to another component of a system, but compatibility is tied to the version and extension environment. A stored plan is not an independently standardized long-term schema.

`datafusion-substrait` instead implements the Substrait relational-plan representation. This is suited to communication with systems such as Apache Calcite and other engines capable of consuming Substrait.

The distinction is consequential: a portable protocol can preserve the structure of a relational operation without guaranteeing that two engines interpret every function, cast, time zone, collation, or custom extension identically.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

`https://docs.rs/datafusion-proto/latest/datafusion_proto/`

`https://docs.rs/datafusion-substrait/latest/datafusion_substrait/`

### 10.2 DataFusion FFI

`datafusion-ffi` exposes FFI-safe wrappers for DataFusion abstractions including catalogs, schemas, table providers, physical expressions, execution plans, query planners, streams, and function-related interfaces.

This makes it possible for separately compiled Rust components to cooperate without directly relying on the unstable native Rust ABI.

Its role differs from Protobuf serialization: it concerns live interoperability between components, rather than encoding an execution plan into a transferable message.

It is still necessary to respect compatible FFI contracts, allocation and ownership rules, shared library versions, and supported boundaries. An FFI wrapper is not automatic binary compatibility between all independently compiled DataFusion versions.

Version distinction: The published `datafusion-ffi` documentation resolves to 55.1.0 at this research date, while the core umbrella crate is 55.2.0. Companion-package publication should be checked individually rather than assuming every package is available at the core version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/datafusion-ffi/latest/datafusion_ffi/`

### 10.3 Spark-compatible functions versus Spark integration

The Rust crate `datafusion-spark` provides function implementations and planning support intended to reproduce Spark semantics where supported.

This is distinct from DataFusion Comet.

`datafusion-spark` is a Rust library component that can be used when an embedded DataFusion engine needs Spark-compatible calculations. Comet is a Spark accelerator that embeds native execution underneath Spark's established planning and distributed runtime.

These distinctions help avoid incorrectly assuming that installing Spark-compatible functions creates a Spark cluster integration, or that Comet's comprehensive Spark compatibility work is entirely contained in the Rust function crate.

Source: `https://docs.rs/datafusion-spark/latest/datafusion_spark/`

## 11. Distributed execution and major DataFusion subprojects

DataFusion's surrounding Apache projects address substantially different use cases.

| Component            | Verified release family | Architectural role                                      |
| -------------------- | ----------------------- | ------------------------------------------------------- |
| DataFusion core      | 55.2.0                  | Embedded relational engine and Rust component libraries |
| DataFusion Ballista  | 55.0.0                  | Distributed execution across multiple nodes             |
| DataFusion Comet     | 1.1.0                   | Native acceleration of Apache Spark                     |
| DataFusion Python    | 55.0 series             | Python SQL/DataFrame interface to the engine            |
| DataFusion Java      | Independently versioned | Java access to DataFusion functionality                 |
| DataFusion SQLParser | Independent 0.63 series | SQL parsing without requiring the full engine           |

### 11.1 Apache DataFusion Ballista

Ballista is the distributed execution system built around DataFusion's planning and physical execution architecture.

It adds schedulers, executors, distributed stages, shuffle transport, task coordination, and cluster-level query execution.

The Ballista 55.0.0 release builds on DataFusion 55.1.0—not the separately released 55.2.0 core patch—and introduced several substantial distributed capabilities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog



| Capability                    | Mechanism and significance                                                             |
| ----------------------------- | -------------------------------------------------------------------------------------- |
| Adaptive Query Execution      | Reoptimizes between stages using statistics from already completed execution           |
| Distributed stage scheduling  | Divides execution into independently schedulable work                                  |
| Sort-based shuffle            | Exchanges partitioned data between executors                                           |
| Multi-partition tasks         | Reduces scheduler overhead by assigning multiple input partitions per task             |
| Range repartitioning          | Supports ordered or unordered redistribution based on key ranges                       |
| Broadcast join optimization   | Distributes small build-side relations to avoid expensive two-sided shuffles           |
| Bounded executor memory       | Default `FairSpillPool` with spill support                                             |
| Flight SQL frontend           | Exposes SQL connectivity through Arrow Flight SQL, optionally enabled                  |
| Event logs and history server | Retains completed-job execution information independently of scheduler memory          |
| Distributed windows           | Experimental range-based parallelization for a restricted class of window aggregations |

Two developments are especially significant.

First, adaptive planning is enabled by default in Ballista 55.0.0. The scheduler can use observed stage statistics rather than only pre-execution estimates to decide subsequent join strategies and exchanges.

This permits runtime information to influence distributed execution at a coarser granularity than DataFusion's in-process dynamic filters. Dynamic filtering reduces work within an executing plan; Ballista's adaptive planner can change the plans for later stages.

Second, ordered range shuffle enables distributed algorithms that cannot be expressed efficiently using only arbitrary hash partitioning.

For example, Ballista can experimentally parallelize certain global window aggregates with `UNBOUNDED PRECEDING` by using ordered ranges and communicating prefix state. In 55.0.0, this requires the opt-in setting `ballista.planner.parallel_window.enabled` and is limited to a single ascending fixed-width ordering key without `PARTITION BY`.

Ballista also offers an optional Flight SQL frontend requiring the `flight-sql` build feature and `--flight-sql` runtime flag. This makes it accessible to SQL clients via standard Arrow database connectivity mechanisms.

The event-log infrastructure and `ballista-history-server` provide durable query-execution history for examination after jobs finish or the scheduler restarts.

Ballista is therefore much more than a simple RPC wrapper around DataFusion. It adds its own planning, scheduling, data movement, fault-handling, and operational infrastructure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog



Sources:

`https://datafusion.apache.org/ballista/`

`https://github.com/apache/datafusion-ballista`

`https://datafusion.apache.org/blog/output/2026/10/03/datafusion-ballista-55.0.0/`

### 11.2 Apache DataFusion Comet

Comet accelerates Apache Spark by translating supported Spark physical operators into native DataFusion-based execution.

Spark remains responsible for distributed orchestration and retains its higher-level SQL and workload semantics. Comet selectively replaces portions of physical execution with native operators, falling back when a construct or configuration is unsupported.

This is fundamentally different from Ballista.

| Dimension                        | Ballista                                                  | Comet                                     |
| -------------------------------- | --------------------------------------------------------- | ----------------------------------------- |
| Distributed coordinator          | Ballista scheduler                                        | Apache Spark                              |
| Execution foundation             | DataFusion                                                | Spark with native DataFusion acceleration |
| Compatibility objective          | DataFusion semantics                                      | Spark semantics                           |
| JVM dependency                   | Not required                                              | Spark/JVM required                        |
| Primary integration              | Distributed SQL/DataFusion applications                   | Existing Spark workloads                  |
| Unsupported native functionality | Requires DataFusion/Ballista capability or implementation | Can often fall back to Spark              |

Comet 1.1.0 adds several important capabilities beyond native query execution.

Its experimental Iceberg writer can use `iceberg-rust` to write data files while preserving Spark/Iceberg's existing metadata and commit process. The JVM remains responsible for committing the table update. This separation avoids replacing the entire transaction protocol merely to accelerate the file-writing portion.

Comet also improves native memory accounting, Iceberg scan support, Spark function compatibility, and Parquet integration.

The release includes work on native shuffle through Apache Celeborn, but native integration is not usable with the released Celeborn 0.6.x and 0.7.x clients; those continue to use ordinary Spark/Celeborn shuffle. This is a useful example of functionality present in a codebase that has not yet become broadly usable in the associated deployed ecosystem.

Experimental native Parquet writing on Spark 4.0+ similarly requires specific configuration and remains constrained by the supported write paths.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion Blog



Sources:

`https://datafusion.apache.org/comet/`

`https://github.com/apache/datafusion-comet`

`https://datafusion.apache.org/blog/output/2026/10/01/datafusion-comet-1.1.0/`

### 11.3 Python and Java

DataFusion Python and DataFusion Java expose the native engine to their respective language ecosystems.

For Rust architecture, their importance is primarily interoperability: one execution engine can support different language-level interfaces and allow Arrow data to cross suitable language boundaries without being converted into row-oriented objects at every stage.

These bindings should not be confused with independently implementing the Rust core engine, nor should their version numbers be treated as synchronized with every underlying Rust crate.

Sources:

`https://github.com/apache/datafusion-python`

`https://github.com/apache/datafusion-java`

## 12. Community connectors, federation, and lakehouse integration

A particularly important part of the ecosystem sits outside Apache's core DataFusion release.

The project's official extensions directory explicitly distinguishes these community-managed components from ASF-governed core projects.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



### 12.1 External table providers

The `datafusion-table-providers` project supplies implementations that expose other databases and services as DataFusion relations.

Its ecosystem includes PostgreSQL, MySQL, SQLite, DuckDB, and Flight SQL providers, with additional provider implementations depending on their individual packages and versions.

The important architectural benefit is not merely that DataFusion can read external rows. These providers permit external data to participate in the same DataFusion relational planning and execution environment as Arrow or Parquet data.

However, registering an external `TableProvider` does not necessarily push an entire query to the backend. Source-side filter and projection pushdown is different from pushing joins, aggregates, sorts, or whole subplans.

Source: `https://github.com/datafusion-contrib/datafusion-table-providers`

### 12.2 DataFusion Federation

`datafusion-federation` addresses the more general challenge of executing eligible portions of a DataFusion logical query through an external query engine.

This opens the possibility of moving an entire compatible relational subplan to the source rather than fetching all relevant rows into DataFusion first.

For example, an external database may be able to execute several joins and aggregations before returning a much smaller result.

The benefit depends on the backend's available operators, SQL dialect, cost of remote execution, data transfer, consistency semantics, and whether the subplan can be translated without loss of meaning.

The architectural distinction is:

| Path                      | Execution responsibility                                        |
| ------------------------- | --------------------------------------------------------------- |
| Ordinary provider scan    | Backend supplies rows; DataFusion performs remaining operations |
| Provider with pushdown    | Backend also performs specific eligible scan-local operations   |
| Federated query execution | Backend may execute a substantial relational subplan            |

Federation therefore complements rather than replaces the catalog and table-provider contracts.

Source: `https://github.com/datafusion-contrib/datafusion-federation`

### 12.3 Other important extensions

| Extension                 | Additional capability                                                                      |
| ------------------------- | ------------------------------------------------------------------------------------------ |
| DataFusion ORC            | ORC file access through a separate provider                                                |
| DataFusion JSON Functions | Functions for querying JSON stored as strings                                              |
| DataFusion Bloom          | Adaptive predicate transfer using bloom-filter-style mechanisms                            |
| Delta Lake / delta-rs     | Transaction-log and table-format integration over Parquet/object storage                   |
| Apache Iceberg Rust       | Table-format metadata, snapshot, and data-file integration through accompanying components |

JSON file reading should not be confused with rich JSON expression support. Similarly, native Parquet reading is not equivalent to understanding an Iceberg or Delta table's snapshot, delete semantics, or transactional metadata.

A table-format integration typically supplies the current set of files, logical schema, partition metadata, and relevant delete or snapshot semantics. DataFusion then executes suitable scans and relational operators against that interpretation of the table.

This separation is significant because it lets specialized table-format code concentrate on table identity, metadata, and consistency, while DataFusion supplies much of the actual relational computation.

Sources:

`https://github.com/datafusion-contrib/datafusion-orc`

`https://github.com/datafusion-contrib/datafusion-functions-json`

`https://github.com/YimingQiao/datafusion-bloom`

`https://github.com/delta-io/delta-rs`

`https://github.com/apache/iceberg-rust`

## 13. Consequential architectural combinations

The strongest capabilities often emerge from combining several DataFusion interfaces.

The following are general architectural patterns, not recommendations for any particular codebase.

| Problem shape                          | Native capabilities that compose                                          | Main benefit and qualification                                                                               |
| -------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Domain-specific relational engine      | Custom SQL planner + `Expr` + logical optimizer + UDFs                    | Specialized query semantics without building a separate optimizer or expression evaluator                    |
| Virtual relational data model          | Catalog providers + views + table functions + logical plans               | Relations can represent calculations, external entities, or generated data without mandatory materialization |
| External indexing over Parquet         | Custom metadata/index + `TableProvider` + `ParquetAccessPlan` + pruning   | Specialized indexing can reuse DataFusion's mature Parquet decoding and batch execution                      |
| Heterogeneous analytics                | Multiple `TableProvider`s + federation + relational optimizer             | Unified query interface across several sources, subject to source semantics and pushdown quality             |
| High-performance analytical transforms | `LogicalPlan` + Arrow + native aggregation/windows + physical optimizer   | Removes bespoke grouping, joins, sorting, buffering, and expression evaluation                               |
| Specialized computational kernels      | Scalar/aggregate/window UDFs + physical expressions + function properties | Native computation integrated into relational planning and vectorized execution                              |
| Memory-bounded execution               | `MemoryPool` + spillable operators + configurable spill backend           | Reuses memory-pressure and external-processing infrastructure                                                |
| Multi-process or distributed engine    | Protobuf/Substrait + FFI + custom physical operators or Ballista          | Separates query semantics from process and cluster topology, with explicit compatibility obligations         |
| SQL-compatible API or tooling          | SQL parser + logical IR + unparser + catalogs + `information_schema`      | Enables query inspection, transformation, and re-expression without text-based rewriting                     |

### 13.1 The distinction between semantic reuse and mechanical reuse

DataFusion permits both.

Semantic reuse comes from sharing the relational model. A custom table provider, optimizer rewrite, and DataFrame transformation can all exchange typed logical plans and expressions.

Mechanical reuse comes from sharing the physical infrastructure: Arrow arrays, vectorized kernels, aggregation machinery, sorts, joins, I/O readers, memory management, and task scheduling.

The most substantial reduction in bespoke infrastructure occurs when both kinds of reuse are possible. An isolated SQL parser may remove custom parsing code, but it does not automatically eliminate custom execution logic. Similarly, using Arrow arrays without DataFusion's relational operators does not provide query planning or optimization.

### 13.2 Correctness-sensitive distinctions

Several boundaries deserve particular attention when combining advanced features.

| Concern                             | Consequence                                                                              |
| ----------------------------------- | ---------------------------------------------------------------------------------------- |
| Nulls and three-valued logic        | Predicate and join rewrites must preserve SQL null semantics                             |
| Expression evaluation order         | Optimizers may reorder predicates; side-effect or error assumptions must be explicit     |
| Ordering                            | Relations are unordered unless a specific ordering guarantee exists                      |
| Exact versus approximate statistics | Approximation can guide planning but cannot justify exact-result rewrites                |
| Function volatility                 | Controls whether expressions can be duplicated, eliminated, or evaluated early           |
| Partitioning and equivalences       | Incorrect metadata can permit invalid removal of exchanges or sorts                      |
| Integer/decimal/float semantics     | Casts, overflow, precision, and aggregation order can affect results                     |
| Source snapshot consistency         | Reading multiple files or systems does not automatically give one transactional snapshot |
| UDF and provider side effects       | Execution plans may be retried, canceled, or reorganized by a surrounding engine         |
| Plan serialization                  | Serialized operator identity does not guarantee version or extension compatibility       |

Version 55's migration guidance is worth reviewing for expression-evaluation changes: relying on one independent predicate to prevent evaluation of another can be unsafe after predicate reordering. A conditional expression such as `CASE` is often the appropriate means to encode an actual evaluation dependency.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation



Source: `https://datafusion.apache.org/library-user-guide/upgrading/55.0.0.html`

### 13.3 Cost-model and optimizer limitations

DataFusion has sophisticated rule-based optimization, physical-property analysis, and statistics-informed planning. This should not be interpreted as a guarantee of a globally optimal physical plan.

Its estimates can be incomplete, particularly for custom providers, correlated filters, skewed distributions, and specialized expressions whose costs are unknown.

A provider can improve physical choices by supplying truthful statistics and properties, but the optimizer cannot infer arbitrary domain knowledge or external execution costs on its own.

Similarly, a custom rule may be useful for a class of operations that general-purpose optimization does not recognize; introducing such a rule also creates a semantic-maintenance obligation as the engine evolves.

## 14. Primary-source navigation map

The following resources provide a compact, durable set of entry points for deeper investigation. URLs are shown in full for direct use in documentation or research workflows.

| Topic                            | Primary source                                                                 |
| -------------------------------- | ------------------------------------------------------------------------------ |
| Core Rust API                    | `https://docs.rs/datafusion/55.2.0/datafusion/`                                |
| Crate dependencies and features  | `https://docs.rs/crate/datafusion/55.2.0/features`                             |
| Architecture and system design   | `https://datafusion.apache.org/contributor-guide/architecture.html`            |
| Logical-plan construction        | `https://datafusion.apache.org/library-user-guide/building-logical-plans.html` |
| Expression model                 | `https://datafusion.apache.org/library-user-guide/working-with-exprs.html`     |
| Optimizer customization          | `https://datafusion.apache.org/library-user-guide/query-optimizer.html`        |
| Operator extension               | `https://datafusion.apache.org/library-user-guide/extending-operators.html`    |
| Catalog and table abstractions   | `https://datafusion.apache.org/library-user-guide/catalogs.html`               |
| Custom table providers           | `https://datafusion.apache.org/library-user-guide/custom-table-providers.html` |
| Custom functions                 | `https://datafusion.apache.org/library-user-guide/functions/adding-udfs.html`  |
| SQL syntax and planner extension | `https://datafusion.apache.org/library-user-guide/extending-sql.html`          |
| Configuration                    | `https://datafusion.apache.org/user-guide/configs.html`                        |
| Execution metrics                | `https://datafusion.apache.org/user-guide/metrics.html`                        |
| Plan introspection               | `https://datafusion.apache.org/user-guide/sql/explain.html`                    |
| Profiling                        | `https://datafusion.apache.org/library-user-guide/profiling.html`              |
| Security                         | `https://datafusion.apache.org/library-user-guide/securing-datafusion.html`    |
| DataFusion-native serialization  | `https://docs.rs/datafusion-proto/latest/datafusion_proto/`                    |
| Substrait interoperability       | `https://docs.rs/datafusion-substrait/latest/datafusion_substrait/`            |
| FFI                              | `https://docs.rs/datafusion-ffi/latest/datafusion_ffi/`                        |
| Community extensions             | `https://datafusion.apache.org/library-user-guide/extensions.html`             |
| Version 55 release capabilities  | `https://datafusion.apache.org/blog/2026/08/25/datafusion-55.0.0/`             |
| Version 55 upgrade changes       | `https://datafusion.apache.org/library-user-guide/upgrading/55.0.0.html`       |
| Ballista distributed engine      | `https://github.com/apache/datafusion-ballista`                                |
| Comet Spark accelerator          | `https://github.com/apache/datafusion-comet`                                   |
| Ecosystem source repository      | `https://github.com/apache/datafusion`                                         |

## 15. Overall capability perspective

The defining strength of Apache DataFusion is not any single SQL operator, optimizer rule, or Parquet optimization. It is the existence of a coherent, extensible relational engine whose semantic, planning, execution, and integration layers can be used together or independently.

Three capabilities are particularly important to recognize.

First, DataFusion's `Expr`, `LogicalPlan`, and catalog interfaces provide a mature foundation for typed, composable relational computation. This allows systems to express complex processing through reusable operations rather than accumulating custom procedural transformations.

Second, its physical optimizer, Arrow execution infrastructure, source-pruning capabilities, and resource-management interfaces provide substantial execution machinery that would otherwise require specialized implementations.

Third, its extension contracts permit external storage engines, domain-specific computation, alternative SQL semantics, and distributed services to participate in a shared planning environment.

The principal tradeoff is that extensibility carries semantic responsibilities. Custom providers must accurately represent their filtering guarantees and statistics; custom operators must accurately represent physical properties; functions must declare truthful behavior; and integrations must manage their own persistence, authorization, concurrency, and compatibility boundaries.

Under those conditions, DataFusion can serve not only as an analytical query engine, but as a general-purpose relational computation kernel for data-intensive Rust systems.