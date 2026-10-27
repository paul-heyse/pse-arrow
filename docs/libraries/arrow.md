# Apache Arrow for Rust — Comprehensive Capability Reference

Research date: October 26, 2026 | Target: Apache Arrow Rust 60.0.0 (`arrow-rs`) and its associated libraries

## 1. Executive orientation

Apache Arrow's Rust ecosystem is a collection of interoperable libraries for representing, transforming, storing, and transporting typed columnar data. Its primary architectural value is that a common memory representation can serve as the foundation for data ingestion, numerical computation, relational processing, file persistence, and interprocess communication without requiring each subsystem to maintain a separate data model.

The ecosystem encompasses considerably more than Arrow arrays:

- Memory and data modeling: Typed arrays, shared buffers, nested structures, schema metadata, dictionary encoding, run-end encoding, and zero-copy views.
- Computation: Vectorized arithmetic, comparisons, casting, sorting, filtering, indexed gathering, string processing, row encoding, and aggregation primitives.
- Serialization and persistence: Arrow IPC, Parquet, CSV, JSON, and Avro, including advanced encoding, compression, projection, and selective decoding.
- Communication and interoperability: Arrow Flight, Flight SQL, C Data and C Stream interfaces, and Python interoperability.
- Storage and execution integration: Asynchronous object storage, columnar query engines, streaming execution, and integration with independently developed systems.

The key distinction is that Arrow provides the data structures and low-level execution primitives, while systems such as Apache DataFusion compose those primitives into relational query execution, optimization, and SQL processing.

### Version scope

The latest stable mainline release I could verify is arrow-rs 60.0.0, released September 15, 2026 and officially announced September 29. This version introduced the `arrow-cmp` crate, Parquet ALP encoding, improved Parquet page indexes, additional statistics, substantial kernel optimizations, and several correctness and safety improvements.

The release raised the minimum supported Rust version to 1.88.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Note that the separately released multi-language Apache Arrow project is at version 26.0.0 as of October 2026. That is not the Rust crate version; the Arrow format specification is shared, but implementation release numbers are independent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow+1



Primary entry points:

- Rust implementation: [https://github.com/apache/arrow-rs](https://github.com/apache/arrow-rs?utm_source=chatgpt.com)
- Versioned Rust API: [https://docs.rs/arrow/60.0.0/arrow/](https://docs.rs/arrow/60.0.0/arrow/?utm_source=chatgpt.com)
- Release details: [https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/](https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/?utm_source=chatgpt.com)
- Arrow format specification: [https://arrow.apache.org/docs/format/](https://arrow.apache.org/docs/format/?utm_source=chatgpt.com)

## 2. Architecture and crate topology

Arrow is best understood as several layers of capability rather than one monolithic library.

Relational and application execution

DataFusion, custom execution operators, streaming systems, database adapters

Compute

arith · cmp · ord · select · cast · string · row

Persistence

Parquet · IPC · Avro · CSV · JSON

Transport

Flight · Flight SQL · FFI · PyArrow

Arrow arrays, schemas, RecordBatch

arrow-array · arrow-schema · arrow-data

Shared memory and physical buffers

arrow-buffer · contiguous values · offsets · validity bitmaps

Conceptual dependencies and roles, not a complete Cargo dependency graph. The layers can be used independently or combined.

### Core crate families

| Crate          | Responsibility                                                              |
| -------------- | --------------------------------------------------------------------------- |
| `arrow`        | Convenience umbrella crate, with core modules, kernels, IO and re-exports   |
| `arrow-buffer` | Allocation ownership, immutable/mutable buffers, typed buffers, bitmaps     |
| `arrow-schema` | Data types, schemas, fields, metadata, schema compatibility                 |
| `arrow-data`   | Low-level `ArrayData` representation, validation and construction           |
| `arrow-array`  | Typed arrays, builders, iterators, `RecordBatch`, FFI abstractions          |
| `arrow-arith`  | Arithmetic, numeric and temporal kernels                                    |
| `arrow-cmp`    | Comparison kernels, separated from `arrow-ord` in v60                       |
| `arrow-ord`    | Ordering, sort indices, ranking, partitioning                               |
| `arrow-select` | Filtering, take, concatenation, interleaving and other selection primitives |
| `arrow-cast`   | Type conversion and casting                                                 |
| `arrow-string` | String and regular-expression kernels                                       |
| `arrow-row`    | Lexicographically comparable, encoded row representation                    |

### Format, transport, and specialized libraries

| Crate                    | Responsibility                                                                 |
| ------------------------ | ------------------------------------------------------------------------------ |
| `arrow-ipc`              | Native Arrow stream and file serialization                                     |
| `arrow-json`             | JSON and newline-delimited JSON reading/writing; Serde integration             |
| `arrow-csv`              | CSV parsing, inference and writing                                             |
| `arrow-avro`             | Apache Avro to/from Arrow interoperability                                     |
| `parquet`                | Full Parquet reader/writer implementation, metadata, encodings and advanced IO |
| `parquet_derive`         | Derive macros for direct Parquet record reading/writing                        |
| `arrow-flight`           | Flight RPC, Flight SQL protocol types, clients and servers                     |
| `arrow-pyarrow`          | Rust/Python interoperability through PyArrow                                   |
| `parquet-variant` family | Specialized support for Parquet's semi-structured Variant encoding             |
| `parquet-geospatial`     | Geospatial support associated with the Parquet ecosystem                       |

The umbrella `arrow` crate is convenient, but the subcrate decomposition allows libraries to depend only on the abstractions they require. For example, a kernel library may use `arrow-array`, `arrow-schema` and `arrow-buffer` without bringing in CSV, JSON, IPC or RPC support.

The independently maintained `object_store` library is an important companion for cloud and local storage. It is not an Arrow data-format crate. DataFusion is likewise a separate query engine rather than part of `arrow-rs`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Reference: [https://github.com/apache/arrow-rs/blob/main/README.md](https://github.com/apache/arrow-rs/blob/main/README.md?utm_source=chatgpt.com)

## 3. The Arrow memory model

The memory model is the foundation of the Rust implementation. Understanding its separation of logical types, physical layouts, and ownership explains many of the library's performance and integration capabilities.

### 3.1 Buffers, ownership, and zero-copy operations

At the physical layer, Arrow data is stored in shared, contiguous memory buffers.

`arrow-buffer` provides several distinct abstractions:

| Type                               | Purpose                 | Important characteristic                           |
| ---------------------------------- | ----------------------- | -------------------------------------------------- |
| `Buffer`                           | Immutable byte storage  | Shared ownership; cheap clone and slice            |
| `MutableBuffer`                    | Growable byte storage   | Mutable construction before immutable sharing      |
| `ScalarBuffer<T>`                  | Primitive typed values  | Typed access to underlying contiguous storage      |
| `BooleanBuffer`                    | Packed Boolean values   | One bit per Boolean                                |
| `NullBuffer`                       | Validity bitmap         | Set bit = valid; unset bit = null                  |
| `OffsetBuffer<O>`                  | Variable-length offsets | Defines ranges into child/value buffers            |
| `RunEndBuffer<E>`                  | Run-end boundaries      | Supports compact representation of repeated values |
| `MemoryPool` / `MemoryReservation` | Allocation accounting   | Enables tracking and budgeting of allocations      |

For example, a nullable string array generally consists of an offsets buffer, a contiguous UTF-8 data buffer, and an optional validity bitmap. It does not require individual heap allocations for every string.

There are important differences between operations that can reuse existing buffers and those that must create new ones:

- Cloning an immutable array reference or taking an Arrow slice can reuse the existing allocation.
- Constructing primitive arrays from suitable Rust `Vec<T>` values can transfer allocation ownership without copying their contents.
- Filtering, indexed gathering, casting to another physical representation, and many computations generally create new output buffers.
- A zero-copy slice can retain the complete underlying allocation, including bytes outside the visible slice.

Thus, zero-copy is an operation-specific property, not a blanket guarantee of Arrow processing.

The immutable representation makes buffers natural for concurrent read-only use. Builders and mutable buffers provide the corresponding construction path.

References:

[https://docs.rs/arrow-buffer/60.0.0/arrow_buffer/](https://docs.rs/arrow-buffer/60.0.0/arrow_buffer/?utm_source=chatgpt.com)

[https://docs.rs/arrow-array/60.0.0/arrow_array/](https://docs.rs/arrow-array/60.0.0/arrow_array/?utm_source=chatgpt.com)

[https://docs.rs/arrow-buffer/60.0.0/arrow_buffer/trait.MemoryPool.html](https://docs.rs/arrow-buffer/60.0.0/arrow_buffer/trait.MemoryPool.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 3.2 Logical and physical types

`arrow-schema` describes logical data types and field relationships. `arrow-array` supplies their concrete memory representations.

The type system includes:

| Type family        | Major representations                     | Architectural significance                                 |
| ------------------ | ----------------------------------------- | ---------------------------------------------------------- |
| Primitive numerics | Signed/unsigned integers, Float16/32/64   | Contiguous, typed numeric buffers                          |
| Decimals           | Decimal32/64/128/256                      | Exact fixed-point values with precision and scale          |
| Temporal           | Date, Time, Timestamp, Duration, Interval | Explicit units and temporal semantics                      |
| Strings/binary     | Utf8, LargeUtf8, Binary, LargeBinary      | Offset-based variable-width storage                        |
| Views              | Utf8View, BinaryView                      | Short inline values or references into shared data buffers |
| Lists              | List, LargeList, FixedSizeList            | Repeated values of a specified child type                  |
| List views         | ListView, LargeListView                   | Independent starts and lengths into shared child data      |
| Structures         | Struct                                    | Named, typed child arrays                                  |
| Maps               | Map                                       | Key/value entries represented as a list of structs         |
| Unions             | DenseUnion, SparseUnion                   | Values with different possible physical types              |
| Dictionaries       | Dictionary                                | Integer indices into a dictionary of values                |
| Run-end encoded    | RunEndEncoded                             | Encodes consecutive repeated values as runs                |
| Null               | Null                                      | A logical type containing only nulls                       |

Several of these deserve special attention.

Dictionary encoding represents categorical or frequently repeated values with compact integer keys. It can reduce memory consumption and enable efficient comparisons, but dictionaries may need reconciliation across batches, files, or partitions.

Run-end encoding represents contiguous repetitions without materializing a value for every logical row. It is distinct from dictionary encoding: it exploits repeated sequences rather than repeated values anywhere in a column. Operations that preserve runs can be especially efficient; others may require specialized kernels or materialization.

StringView/BinaryView use a view-oriented representation, permitting short values to be stored inline and longer values to reference larger shared buffers. This can reduce copying in suitable string-intensive workloads, but the view representation changes locality and ownership considerations.

ListView separates list offsets and sizes instead of requiring the monotonic cumulative boundaries used by ordinary `List`. This offers alternative sharing and construction patterns, particularly when logical lists reference existing child data.

Struct, Map, and Union provide native representations of complex data. Their presence does not imply that every operation treats all nested types identically; kernel support and conversion behavior must be checked by type.

References:

[https://docs.rs/arrow-array/60.0.0/arrow_array/array/](https://docs.rs/arrow-array/60.0.0/arrow_array/array/?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/format/Columnar.html](https://arrow.apache.org/docs/format/Columnar.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 3.3 Typed arrays versus generic arrays

Rust Arrow deliberately supports both statically typed and dynamically typed programming.

Statically typed processing

`PrimitiveArray<T>`, `GenericByteArray<T>`, and other concrete implementations expose representations whose types are known to the compiler. Generic constraints such as `ArrowPrimitiveType`, `ArrayAccessor` and `ListLikeArray` enable specialized processing without a type-erased operation for every value.

Dynamically typed processing

The `Array` trait and `ArrayRef = Arc<dyn Array>` allow generic data containers and operators to handle runtime-determined schemas.

`AsArray` provides convenient downcasting after type checking.

These modes compose naturally: general-purpose systems can transport and inspect dynamic arrays, then dispatch their performance-critical operations into typed implementations.

The lower-level `ArrayData` representation provides access to common physical metadata, buffers, child arrays, and validation. It is particularly relevant for interoperability, custom array construction, and kernels that manipulate physical storage directly.

The API distinguishes structural validation from comprehensive recursive value validation. `validate_full()` checks the array and descendants, although not every conceivable domain invariant is enforced by Arrow's physical validity checks.

References:

[https://docs.rs/arrow-array/60.0.0/arrow_array/](https://docs.rs/arrow-array/60.0.0/arrow_array/?utm_source=chatgpt.com)

[https://docs.rs/arrow-data/60.0.0/arrow_data/](https://docs.rs/arrow-data/60.0.0/arrow_data/?utm_source=chatgpt.com)

[https://docs.rs/arrow-data/60.0.0/arrow_data/struct.ArrayData.html](https://docs.rs/arrow-data/60.0.0/arrow_data/struct.ArrayData.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 3.4 Schema metadata and extension types

`Schema`, `Field`, and `DataType` provide a common typed contract for the data representation.

A field includes a name, logical type, nullability, and metadata. Schemas also contain metadata and support field-level operations and schema construction.

This enables data producers and consumers to carry information beyond primitive storage types, such as logical identity, semantic annotations, interoperability hints, and domain-specific type names.

`arrow-schema::extension` defines an extension-type mechanism and supports canonical extension types behind the `canonical_extension_types` Cargo feature.

The canonical types include:

- UUID, JSON, Bool8, TimestampWithOffset, and Opaque.
- FixedShapeTensor and VariableShapeTensor.

These extension types are particularly interesting because they allow a domain concept to retain its logical identity while continuing to use an Arrow-compatible physical storage representation.

For example, a tensor-valued field can be distinguished from an arbitrary nested list even if the underlying storage uses nested Arrow arrays.

Important boundary: Extension metadata describes the logical interpretation. It does not automatically equip all compute kernels, serialization backends, or query engines with specialized behavior for that type. Each consumer must preserve or understand the extension where relevant.

References:

[https://docs.rs/arrow-schema/60.0.0/arrow_schema/](https://docs.rs/arrow-schema/60.0.0/arrow_schema/?utm_source=chatgpt.com)

[https://docs.rs/arrow-schema/60.0.0/arrow_schema/extension/](https://docs.rs/arrow-schema/60.0.0/arrow_schema/extension/?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/format/CanonicalExtensions.html](https://arrow.apache.org/docs/format/CanonicalExtensions.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 3.5 RecordBatch and streaming representation

`RecordBatch` is the central tabular interchange unit.

It contains:

- A shared `SchemaRef`.
- An ordered collection of `ArrayRef` columns.
- A common logical row count across its columns.

A `RecordBatch` is not a SQL table, a persistent database object, or a complete multi-batch dataset. It is an in-memory partition of tabular values.

Unlike the Arrow C++ and PyArrow implementations, Rust Arrow deliberately does not implement the same `ChunkedArray` abstraction. The documented alternatives are collections, iterators, and asynchronous streams of arrays or record batches.

This is consequential: an operation can consume a sequence of batches without consolidating them into one large array. Execution engines can handle bounded batches and stream results incrementally.

The abstraction also makes Arrow useful independently of relational databases: a data transformation, numerical routine, file reader, or network consumer can share the same `RecordBatch` contract.

References:

[https://docs.rs/arrow-array/60.0.0/arrow_array/struct.RecordBatch.html](https://docs.rs/arrow-array/60.0.0/arrow_array/struct.RecordBatch.html?utm_source=chatgpt.com)

[https://docs.rs/arrow-array/60.0.0/arrow_array/trait.RecordBatchReader.html](https://docs.rs/arrow-array/60.0.0/arrow_array/trait.RecordBatchReader.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

## 4. Native compute kernels and data transformation

Arrow Rust provides an extensive collection of columnar kernels without requiring a SQL engine.

The important architectural distinction is between elementwise operations, data movement, ordering, and reductions. Their computational costs and composition behavior differ substantially.

### 4.1 Kernel families

| Crate          | Major capabilities                                                                                                |
| -------------- | ----------------------------------------------------------------------------------------------------------------- |
| `arrow-arith`  | Numeric arithmetic, wrapping operations, checked aggregation, temporal extraction, Boolean and bitwise operations |
| `arrow-cmp`    | Typed and dynamic comparisons, including array-to-scalar comparisons                                              |
| `arrow-ord`    | Sorting, sorting indices, ranking, partition detection and ordering                                               |
| `arrow-select` | Filter, take, concat, interleave, merge, zip, nullif, array shifting and batch coalescing                         |
| `arrow-cast`   | Numeric, temporal, decimal, string and nested-type conversions                                                    |
| `arrow-string` | String comparisons and transformations, substring and regular-expression-related operations                       |
| `arrow-row`    | Order-preserving row encoding and decoding                                                                        |

These libraries offer reusable, optimized implementations of common columnar operations. Their boundaries make it possible to use individual kernels without introducing a complete relational execution layer.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



### 4.2 Arithmetic, Boolean operations, and reductions

`arrow-arith` includes numeric binary operations, unary operations, bitwise manipulation, Boolean logic, temporal kernels, and aggregations.

Relevant APIs include:

`numeric::add`, `sub`, `mul`, `div`, `rem`, `neg`, `aggregate::sum`, `sum_checked`, `arity::unary`, `try_unary`, `binary`, and `try_binary`.

The `arity` family is especially useful for library authors. It provides generic primitives for writing custom elementwise kernels, including fallible variants, rather than requiring manual handling of every array buffer and validity bitmap.

Boolean processing includes null-aware variants such as `and_kleene` and `or_kleene`, which distinguish three-valued logical semantics from simple bitwise operations.

This distinction matters in any system where null represents unknown rather than a Boolean false value.

Other important considerations are numeric overflow, division errors, decimal scale, floating-point special values, and whether a kernel preserves or propagates nulls. A selected arithmetic operation's documented behavior should determine how it is composed.

Reference: [https://docs.rs/arrow-arith/60.0.0/arrow_arith/](https://docs.rs/arrow-arith/60.0.0/arrow_arith/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

docs.rs+1

### 4.3 Selection, rearrangement, and batch compaction

`arrow-select` supplies operations that are foundational to data pipelines:

- `filter` retains rows selected by a Boolean predicate.
- `take` gathers values at specified indices.
- `concat` combines compatible arrays.
- `interleave` draws elements from multiple arrays according to source/index pairs.
- `zip` selects between arrays using a Boolean mask.
- `merge` and `merge_n` combine sorted inputs.
- `nullif` conditionally produces nulls.
- `window` supplies operations such as shifting.
- `BatchCoalescer` combines smaller record batches into larger batches.

These primitives permit data transformations to be expressed using existing kernels rather than bespoke row-by-row loops.

A useful conceptual composition is:

Typed input columns

Comparison kernel → Boolean predicate

Filter / take → Selected rows

Cast / transform → Output columns

RecordBatch / batch stream

This is only one possible composition; no engine is required to perform it.

One tradeoff is the creation of intermediate arrays and repeated passes over data. Query engines and custom fused kernels may improve on straightforward kernel chaining by combining operations, controlling materialization, or eliminating redundant computations.

References:

[https://docs.rs/arrow-select/60.0.0/arrow_select/](https://docs.rs/arrow-select/60.0.0/arrow_select/?utm_source=chatgpt.com)

[https://docs.rs/arrow-select/60.0.0/arrow_select/coalesce/struct.BatchCoalescer.html](https://docs.rs/arrow-select/60.0.0/arrow_select/coalesce/struct.BatchCoalescer.html?utm_source=chatgpt.com)

### 4.4 Casting and typed conversion

`arrow-cast` is an important semantic integration layer, not merely a convenience for changing numeric types.

It provides:

- Conversion between supported Arrow logical and physical types.
- String parsing and formatting.
- Decimal scaling and precision-sensitive conversions.
- Temporal conversion and formatting options.
- Type compatibility checks such as `can_cast_types`.
- Conversion utilities that can avoid manually rebuilding arrays.

`CastOptions` makes an important choice explicit: its `safe` setting controls whether certain failed element conversions produce nulls or errors.

In the documented API, `safe = true` permits conversion failures to become null values; `safe = false` requests errors for such failures.

This means "safe casting" does not universally mean lossless casting. Whether information can be lost, rounded, or converted to null depends on the conversion.

Reference: [https://docs.rs/arrow-cast/60.0.0/arrow_cast/cast/](https://docs.rs/arrow-cast/60.0.0/arrow_cast/cast/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 4.5 Sorting, ranking, and the row encoding engine

`arrow-row` is one of the more architecturally significant libraries in the collection.

Normal Arrow storage is columnar, which is efficient for column-oriented operations but less convenient when repeatedly comparing composite keys across many columns.

`arrow-row` provides `RowConverter`, `SortField`, and the `Rows` representation. These encode values from multiple arrays into byte sequences whose lexicographical comparison follows the specified Arrow sort semantics.

The representation enables efficient:

- Multicolumn sorting.
- Group key comparison.
- Ordered grouping and aggregation.
- Window ordering.
- Merge operations and other comparison-intensive algorithms.

The critical benefit is that repeated comparisons can operate on normalized row-encoded keys rather than repeatedly dispatching through several dynamic column types.

It is a complement to columnar storage, not a replacement.

Two limitations are consequential:

First, encoded rows must be constructed using compatible settings from the same `RowConverter` for their comparisons to be meaningful.

Second, dictionary arrays are expanded to their underlying values during conversion, so decoding the row representation does not necessarily reproduce the original dictionary-encoded physical type.

`arrow-ord` provides more conventional sorting, ranking and partitioning kernels. In v60, comparisons were also factored into the new `arrow-cmp` crate for more focused dependency selection.

References:

[https://docs.rs/arrow-row/60.0.0/arrow_row/](https://docs.rs/arrow-row/60.0.0/arrow_row/?utm_source=chatgpt.com)

[https://docs.rs/arrow-ord/60.0.0/arrow_ord/](https://docs.rs/arrow-ord/60.0.0/arrow_ord/?utm_source=chatgpt.com)

[https://docs.rs/arrow-cmp/60.0.0/arrow_cmp/](https://docs.rs/arrow-cmp/60.0.0/arrow_cmp/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2

## 5. Serialization, interchange, and ingestion

Arrow Rust supports several serialization families with different design goals. It is important to distinguish an Arrow-compatible in-memory representation from a format designed for long-term storage, interchange, or application-facing messages.

| Format            | Principal Rust crate | Optimized for                                               | Important distinction                                        |
| ----------------- | -------------------- | ----------------------------------------------------------- | ------------------------------------------------------------ |
| Arrow IPC stream  | `arrow-ipc`          | Sequential transfer of Arrow record batches                 | Preserves Arrow's native representation and stream semantics |
| Arrow IPC file    | `arrow-ipc`          | File interchange and batch-level random access              | Footer provides record-batch locations                       |
| Parquet           | `parquet`            | Compressed persistent columnar datasets and selective scans | Encodings, statistics, row groups, pages and indexes         |
| JSON / JSON Lines | `arrow-json`         | Text-based ingestion and interchange                        | Requires explicit type interpretation                        |
| CSV               | `arrow-csv`          | Delimited text ingestion and export                         | Limited inherent schema information                          |
| Avro              | `arrow-avro`         | Schema-based record interchange and streaming messages      | Writer/reader schemas and registry-oriented encodings        |

### 5.1 Arrow IPC: native serialization

`arrow-ipc` implements both variants of Arrow's IPC format:

- `StreamReader` / `StreamWriter` for sequential records and potentially unbounded streams.
- `FileReader` / `FileWriter` for bounded files with a footer supporting random access to serialized batches.

Both support Arrow schemas and record batches; dictionaries are transmitted through their own message types and managed as part of the decoding state. IPC compression is available through an optional Cargo feature.

IPC is distinctive because its serialized representation closely follows Arrow's in-memory buffers. Under appropriate alignment, memory mapping, ownership, and compression conditions, readers can reuse serialized buffer memory rather than copying it. This is not guaranteed for every file or ordinary reader configuration.

Arrow Rust includes an example demonstrating a memory-mapped IPC reading path. It is particularly relevant for local, read-mostly datasets and process boundaries where persistent Arrow-format interchange is desirable.

The IPC file format should not be confused with Parquet. IPC prioritizes faithful interchange of Arrow values, while Parquet provides more extensive data-oriented encoding, compression and scan optimization.

The Rust implementation also should not be assumed to implement every operation described by Arrow's broader IPC specification. For example, the published implementation status distinguishes conventional record-batch IPC from tensor and sparse-tensor IPC support.

References:

[https://docs.rs/arrow-ipc/60.0.0/arrow_ipc/](https://docs.rs/arrow-ipc/60.0.0/arrow_ipc/?utm_source=chatgpt.com)

[https://github.com/apache/arrow-rs/blob/main/arrow/examples/zero_copy_ipc.rs](https://github.com/apache/arrow-rs/blob/main/arrow/examples/zero_copy_ipc.rs?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/format/Columnar.html](https://arrow.apache.org/docs/format/Columnar.html?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/status.html](https://arrow.apache.org/docs/status.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

### 5.2 JSON: structured and semistructured data conversion

`arrow-json` supports schema-aware conversion between JSON records and Arrow arrays.

Its facilities include:

- Line-delimited JSON readers and writers.
- JSON schema inference.
- Batched decoding and configurable decoder behavior.
- `Decoder` interfaces for integration with externally managed input buffers.
- Configurable encoders and `EncoderFactory`.
- Serde-oriented ingestion and serialization.
- Control over representation of nested structs through `StructMode`.

One subtle capability is that structs can be represented as JSON objects or positional arrays, allowing adaptation to producers with different structural conventions.

Another is binary encoding: the default JSON representation of Arrow binary data is hexadecimal (`Base16`), not Base64. The crate documents custom encoders or explicit binary-to-Base64 transformations when another representation is required.

JSON conversion is not a transparent representation-preserving round trip. Missing properties, nulls, numeric interpretation, nested structure, and schema inference can introduce semantic distinctions absent from a fully typed Arrow stream.

Reference: [https://docs.rs/arrow-json/60.0.0/arrow_json/](https://docs.rs/arrow-json/60.0.0/arrow_json/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.3 CSV: typed, configurable tabular ingestion

`arrow-csv` provides batch-oriented CSV reading and writing.

`ReaderBuilder` and associated format utilities support schema inference, headers, delimiter and quoting configuration, projections, configurable batch sizes, and null-handling policies.

Its decoder can also be driven incrementally, separating byte acquisition from record decoding. This lets asynchronous file or network consumers retain control of input acquisition while using the same CSV parser.

An interesting capability is configurable handling of truncated records, which can populate missing nullable fields rather than rejecting every short row.

The limitation is inherent to CSV: it does not carry a complete Arrow schema or faithfully represent arbitrary nested types. Schema inference is necessarily an interpretation of text, and consistency across files is a caller-level concern.

Reference: [https://docs.rs/arrow-csv/60.0.0/arrow_csv/](https://docs.rs/arrow-csv/60.0.0/arrow_csv/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 5.4 Avro: schema evolution and message-oriented interoperability

`arrow-avro` deserves attention beyond conventional file ingestion.

Version 60 supports conversion between Arrow batches and several Avro representations:

| Representation                        | Supported role                                            |
| ------------------------------------- | --------------------------------------------------------- |
| Avro Object Container File (OCF)      | Self-describing block-oriented file reading and writing   |
| Unframed Avro binary datums           | Decode records when the writer schema is known externally |
| Avro Single-Object Encoding           | Schema-fingerprint-based record interchange               |
| Confluent Schema Registry wire format | Schema-ID-based message decoding and encoding             |
| Apicurio framing                      | Registry-ID-based interoperability                        |

The crate includes schema parsing, schema stores, fingerprint algorithms, reader-schema handling, and compression facilities for OCF. An optional asynchronous reader supports incremental processing.

A particularly significant distinction is that Avro schemas can vary across successive messages. The decoder's schema-switching behavior maintains a single schema per emitted `RecordBatch`; a batch boundary is required when the active schema changes.

The crate supports `Rabin`, schema-ID, and optionally MD5/SHA-256 fingerprint representations. This can avoid bespoke framing and schema lookup machinery in message-oriented systems.

Avro schema resolution and Arrow type mapping remain separate concerns. Writer-schema evolution must be translated to the requested reader schema, and not every Avro logical structure has a perfect Arrow equivalent.

References:

[https://docs.rs/arrow-avro/60.0.0/arrow_avro/](https://docs.rs/arrow-avro/60.0.0/arrow_avro/?utm_source=chatgpt.com)

[https://docs.rs/arrow-avro/60.0.0/arrow_avro/reader/](https://docs.rs/arrow-avro/60.0.0/arrow_avro/reader/?utm_source=chatgpt.com)

[https://docs.rs/arrow-avro/60.0.0/arrow_avro/schema/](https://docs.rs/arrow-avro/60.0.0/arrow_avro/schema/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

## 6. Parquet: persistent columnar storage and selective decoding

The Rust `parquet` crate is one of the most extensive components of the Arrow ecosystem. It is not just a file reader/writer wrapper around Arrow.

It implements Parquet's physical storage structures, metadata, encoding and compression mechanisms, low-level column and row APIs, and efficient conversion to and from Arrow record batches.

It also exposes primitives on which query engines can construct sophisticated selective scans without implementing Parquet decoding themselves.

### 6.1 Parquet's physical organization

A Parquet file is organized conceptually as:

File metadata and footer

Physical schema · Row groups · Column locations · Statistics · Optional indexes

Row group 1

Column chunk A

Page · Page · Page

Column chunk B

Page · Page · Page

Row group 2

Column chunk A

Page · Page · Page

Column chunk B

Page · Page · Page

Pages contain encoded and optionally compressed values; definition and repetition levels represent nested data.

Row groups divide files into independently accessible groups of rows. Within a row group, each leaf column is stored separately in a column chunk; the chunks contain encoded pages.

This organization enables a reader to skip irrelevant columns, row groups, and sometimes data pages. It also means that file and row-group sizing decisions affect compression, memory usage, random-access granularity, and the number of requests needed for remote reads.

Parquet stores nested data using repetition/definition semantics that differ from Arrow's offsets and validity buffers. Accordingly, converting between Parquet and Arrow generally involves decoding and reconstruction, even when the logical values are identical.

Reference: [https://docs.rs/parquet/60.0.0/parquet/](https://docs.rs/parquet/60.0.0/parquet/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 6.2 Multiple reader architectures

The crate offers three important Arrow-oriented reading styles.

| Reader                            | Execution model                    | Best distinguished by                                  |
| --------------------------------- | ---------------------------------- | ------------------------------------------------------ |
| `ParquetRecordBatchReaderBuilder` | Synchronous, pull-based iteration  | Straightforward local or synchronous IO                |
| `ParquetRecordBatchStreamBuilder` | Asynchronous stream                | Ranged IO, async sources, streaming consumption        |
| `ParquetPushDecoderBuilder`       | Caller-driven incremental decoding | Explicit separation of input acquisition from decoding |

The resulting reader or decoder emits Arrow `RecordBatch` values, while lower-level Parquet readers can work directly with columnar or record-oriented Parquet structures.

`ParquetPushDecoder` is particularly interesting for advanced execution systems. A surrounding runtime can coordinate fetching, scheduling, buffering, backpressure, and decoding rather than requiring those concerns to be coupled in one blocking reader.

`ArrowReaderMetadata` allows metadata to be loaded once and reused when constructing readers.

Reference: [https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 6.3 Projection, statistics, and multistage filtering

Efficient Parquet scanning normally involves several different forms of pruning.

| Stage                                | Available mechanism           | Work avoided                                       |
| ------------------------------------ | ----------------------------- | -------------------------------------------------- |
| Column projection                    | `ProjectionMask`              | Reading and decoding unneeded columns              |
| Row-group pruning                    | `with_row_groups`             | Accessing excluded row groups                      |
| Page/row-range pruning               | `RowSelection`, page indexes  | Fetching or decoding excluded pages/row ranges     |
| Predicate evaluation during decoding | `RowFilter`, `ArrowPredicate` | Materializing rows excluded by earlier predicates  |
| Output windowing                     | `with_offset`, `with_limit`   | Producing rows outside the requested result window |

These mechanisms are related but not interchangeable.

`RowSelection` can be constructed from page-index information, external indexes, or other precomputed knowledge. It represents rows that can be skipped before their column values are read.

`RowFilter` evaluates predicates as part of decoding. Its predicates can use distinct column projections, allowing later predicates and final output columns to be decoded only for rows surviving earlier predicates. This is a form of late materialization.

The tradeoff is that overlapping predicate projections may cause a column to be decoded more than once. Predicate ordering and projection selection therefore matter.

Newer reader interfaces also support row-group-local selections (`RowGroupSelection`) and policies governing selection representation.

The critical architectural boundary is explicit in the official documentation:

The Parquet crate supplies the primitives for predicate pushdown, but it does not itself provide a complete query predicate optimizer or planner.

A surrounding engine must determine which files, row groups, and pages can be excluded using statistics, constraints, indexes, or application predicates. The decoder executes the resulting scan configuration.

References:

[https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/struct.ArrowReaderBuilder.html](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/struct.ArrowReaderBuilder.html?utm_source=chatgpt.com)

[https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/struct.RowFilter.html](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/struct.RowFilter.html?utm_source=chatgpt.com)

[https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/struct.RowSelection.html](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/struct.RowSelection.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 6.4 Statistics, indexes, and metadata as execution inputs

Parquet metadata has substantial value independently of the data it describes.

The Rust implementation exposes:

- Row-group and column-level min/max statistics.
- Null counts and distinct-value statistics where stored.
- Column and offset page indexes.
- Bloom filters.
- Physical and logical schema information.
- Row counts, file and row-group metadata.
- Typed statistics conversion into Arrow arrays through `StatisticsConverter`.

`StatisticsConverter` is important because it gives higher-level systems a way to work with Parquet statistics using the same typed Arrow representation as ordinary data, including type-sensitive conversions.

The 60.0.0 release expanded this area with:

- `PageIndex`, `PageIndexBuilder`, and `PageIndexProvider`.
- Support for supplying cached or externally fetched page indexes.
- Writing number-of-distinct-values (NDV) statistics.
- Row-group distinct-count access through `StatisticsConverter`.
- Corrected IEEE 754 total-order handling for floating-point statistics.

The ability to provide externally cached page indexes is especially useful for repeated scans and wide schemas: it can avoid repeated metadata IO without requiring the Parquet decoder to own the caching system.

Statistics are summaries, not necessarily exact indexes for arbitrary predicates. Their usability depends on how they were written, null semantics, truncation, sort order, and whether the requested comparison is compatible with the stored statistics.

Bloom filters are similarly probabilistic: they can disprove membership subject to the filter's guarantees but may return false positives.

References:

[https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/statistics/struct.StatisticsConverter.html](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_reader/statistics/struct.StatisticsConverter.html?utm_source=chatgpt.com)

[https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/](https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 6.5 Writer configuration and data layout

`ArrowWriter` and `AsyncArrowWriter` accept Arrow record batches and create Parquet files.

The main configuration surface is `WriterPropertiesBuilder`, supplemented by `ArrowWriterOptions`.

Important writer controls include:

| Dimension           | Configurable mechanisms                                      | Consequences                                              |
| ------------------- | ------------------------------------------------------------ | --------------------------------------------------------- |
| Compression         | Zstd, Snappy, Gzip, Brotli, LZ4 and supported alternatives   | Storage size versus compression/decompression CPU         |
| Encoding            | Plain, dictionary, delta and supported specialized encodings | Compression ratio and decoder cost                        |
| Row-group sizing    | Maximum rows and related limits                              | Scan granularity, writer memory and metadata count        |
| Page sizing         | Page-size and batch-size settings                            | Compression granularity and selective reads               |
| Statistics          | Column/row-group statistics, optional NDV                    | Pruning and planning capability                           |
| Bloom filters       | Global and per-column configuration                          | Membership-based pruning                                  |
| Page/offset indexes | Writer options                                               | Fine-grained selective access                             |
| Sorting metadata    | Column sorting declarations                                  | Enables consumers to exploit known order where applicable |
| Encryption          | Optional Parquet encryption feature and properties           | File confidentiality and reader compatibility             |
| Schema/metadata     | Arrow schema hints, Parquet schema and key-value metadata    | Interoperability and precise type recovery                |
| Page storage        | `PageStoreFactory`                                           | Control of memory versus external temporary page storage  |

The writer accumulates input batches into row groups; a batch does not necessarily correspond to a separate Parquet row group.

Page storage is a particularly useful advanced extension point. `ArrowWriterOptions::with_page_store_factory` permits an alternative to the default in-memory page store. Completed encoded pages can be spilled to temporary storage rather than all retained in writer memory, subject to the selected implementation.

This can improve memory predictability for large row groups or resource-constrained writers, while introducing spill IO and lifecycle management.

References:

[https://docs.rs/parquet/60.0.0/parquet/file/properties/struct.WriterPropertiesBuilder.html](https://docs.rs/parquet/60.0.0/parquet/file/properties/struct.WriterPropertiesBuilder.html?utm_source=chatgpt.com)

[https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_writer/struct.ArrowWriterOptions.html](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_writer/struct.ArrowWriterOptions.html?utm_source=chatgpt.com)

[https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_writer/struct.ArrowWriter.html](https://docs.rs/parquet/60.0.0/parquet/arrow/arrow_writer/struct.ArrowWriter.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

### 6.6 Advanced encoding and specialized data

The Parquet implementation supports conventional encodings as well as newer format capabilities.

Of particular note, v60.0.0 added Adaptive Lossless Floating-Point (ALP) encoding and decoding. This targets compact representation of floating-point columns with efficient decoding and random value access. Its actual benefit depends on the numerical distribution and file-writing settings.

There are also several distinct specialized capabilities.

Parquet Variant. The `parquet-variant`, `parquet-variant-json`, and `parquet-variant-compute` crates address Parquet's semi-structured Variant representation. They expose low-level Variant manipulation, JSON conversion, and specialized operations, including shredding-related functionality. The Arrow/Parquet integration is feature-gated and remains experimental; the project's own documentation describes parts of the implementation as work in progress.

Geospatial. `parquet-geospatial` provides support for the geospatial representation associated with the Parquet ecosystem, including geometry/geography-oriented types and metadata. This is distinct from a general spatial query engine; geometry predicates, indexing, and spatial joins require appropriate higher-level capabilities.

Direct Rust record derivation. `parquet_derive` offers derive macros for reading and writing supported Rust record structures with Parquet's lower-level record APIs. This can be useful when a direct struct-oriented interface is more natural than record-batch processing. It does not replace the general Arrow array model.

These are separate capability areas: the existence of a specialized storage type does not automatically provide an optimized computation engine for that type.

References:

[https://docs.rs/parquet-variant/60.0.0/parquet_variant/](https://docs.rs/parquet-variant/60.0.0/parquet_variant/?utm_source=chatgpt.com)

[https://docs.rs/parquet-variant-compute/60.0.0/parquet_variant_compute/](https://docs.rs/parquet-variant-compute/60.0.0/parquet_variant_compute/?utm_source=chatgpt.com)

[https://docs.rs/parquet-geospatial/60.0.0/parquet_geospatial/](https://docs.rs/parquet-geospatial/60.0.0/parquet_geospatial/?utm_source=chatgpt.com)

[https://docs.rs/parquet_derive/60.0.0/parquet_derive/](https://docs.rs/parquet_derive/60.0.0/parquet_derive/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow+4

### 6.7 Schema fidelity: Arrow versus Parquet

Arrow and Parquet are both typed and columnar, but their type systems are not identical.

For instance, the same Parquet binary representation may be materialized as different Arrow binary array forms. Nested structure, time units, dictionary encoding, and extension metadata can also introduce compatibility considerations.

To improve round-trip fidelity, the Arrow-oriented Parquet writer stores a serialized Arrow schema hint in `ARROW_SCHEMA_META_KEY`. The reader can use this to recover the original Arrow types more precisely.

This is a useful interoperability mechanism, but not a universal guarantee for files created by unrelated writers or subsequently transformed by other systems.

The metadata can be intentionally skipped, for example for consumers that require a particular Parquet-oriented interpretation.

Reference: [https://docs.rs/parquet/60.0.0/parquet/arrow/](https://docs.rs/parquet/60.0.0/parquet/arrow/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 7. Apache Arrow Flight and Flight SQL

`arrow-flight` is the network-transport component of the Rust ecosystem. It builds on Arrow IPC and gRPC, using Tonic and Prost to implement Arrow's remote data exchange protocols.

It should be distinguished from both raw Arrow IPC and database connectivity APIs: Flight provides network operations and stream-oriented transport, not an embedded database or query engine.

### 7.1 The Flight protocol

Flight defines a collection of RPC operations with different purposes.

| Operation        | Role                                                         |
| ---------------- | ------------------------------------------------------------ |
| `ListFlights`    | Discover advertised data streams                             |
| `GetFlightInfo`  | Obtain descriptors, endpoints and tickets for data retrieval |
| `GetSchema`      | Retrieve schema information                                  |
| `DoGet`          | Download a stream of Arrow data                              |
| `DoPut`          | Upload a stream of Arrow data                                |
| `DoExchange`     | Bidirectional streaming and exchange                         |
| `DoAction`       | Execute application-defined operations                       |
| `Handshake`      | Protocol-level handshake, often used during authentication   |
| `PollFlightInfo` | Poll for progressively available information/results         |

Flight separates identifying or discovering a dataset from retrieving its contents.

For example, `GetFlightInfo` may return multiple endpoints with tickets. A client can retrieve the corresponding streams using `DoGet`, potentially from distinct servers.

`DoExchange` is especially general: it enables duplex streams, supporting transformations and protocols in which requests and results are exchanged incrementally.

References:

[https://arrow.apache.org/docs/format/Flight.html](https://arrow.apache.org/docs/format/Flight.html?utm_source=chatgpt.com)

[https://docs.rs/arrow-flight/60.0.0/arrow_flight/client/struct.FlightClient.html](https://docs.rs/arrow-flight/60.0.0/arrow_flight/client/struct.FlightClient.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow v26.0.0+1

### 7.2 Rust implementation layers

The crate exposes multiple abstraction levels.

Raw protocol bindings. Prost-generated Flight messages and Tonic-generated `FlightServiceClient` / `FlightServiceServer` provide control over protocol behavior.

Convenience client. `FlightClient` wraps common operations such as `handshake`, `do_get`, `do_put`, `do_exchange`, and flight discovery. Lower-level client access remains available when detailed RPC headers, metadata, or protocol behavior matter.

Stream codecs. `FlightDataEncoderBuilder`, `FlightRecordBatchStream`, and associated decoders convert between Arrow record-batch streams and Flight's `FlightData` messages.

The codec layer is especially useful because callers can compose Rust asynchronous streams rather than manually constructing schema, dictionary, and batch IPC messages.

`FlightDataEncoderBuilder` supports control over message size and dictionary handling. Its documented default target is approximately 2 MB per `FlightData` message, in part because gRPC deployments commonly impose message-size limits. Large input batches can be split into smaller transport messages.

Dictionary handling is an important interoperability consideration. A transport encoder may hydrate dictionary values rather than transmit dictionary-encoded arrays unchanged, depending on configuration. This trades representation fidelity and message size against simpler decoding compatibility.

References:

[https://docs.rs/arrow-flight/60.0.0/arrow_flight/](https://docs.rs/arrow-flight/60.0.0/arrow_flight/?utm_source=chatgpt.com)

[https://docs.rs/arrow-flight/60.0.0/arrow_flight/encode/struct.FlightDataEncoderBuilder.html](https://docs.rs/arrow-flight/60.0.0/arrow_flight/encode/struct.FlightDataEncoderBuilder.html?utm_source=chatgpt.com)

[https://docs.rs/arrow-flight/60.0.0/arrow_flight/decode/](https://docs.rs/arrow-flight/60.0.0/arrow_flight/decode/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 7.3 Flight SQL

Flight SQL defines a standardized database interaction protocol on top of Flight.

Its capabilities include:

- Executing SQL queries and updates.
- Prepared statements and parameter binding.
- Bulk ingestion of record batches.
- Retrieving catalog, schema, table, column, primary-key and foreign-key metadata.
- SQL feature and server-capability discovery.
- Session and transaction-related protocol operations.

The Rust crate exposes Flight SQL protobuf messages, command encoding/decoding, a client, and server-side service abstractions behind the `flight-sql` Cargo feature.

The crucial distinction is between what the Flight SQL specification defines and what a particular server or client actually implements.

The v60 source documentation explicitly labels `FlightSqlServiceClient` experimental and says complete specification conformance is not guaranteed. Likewise, a server advertising Flight SQL does not necessarily support every optional protocol capability.

The `arrow-flight` distribution also documents a `flight_sql_client` command-line tool, providing a convenient way to query Flight SQL endpoints and inspect their Arrow results. This is a separate client utility, not an embedded SQL execution engine.

References:

[https://arrow.apache.org/docs/format/FlightSql.html](https://arrow.apache.org/docs/format/FlightSql.html?utm_source=chatgpt.com)

[https://docs.rs/arrow-flight/60.0.0/arrow_flight/sql/](https://docs.rs/arrow-flight/60.0.0/arrow_flight/sql/?utm_source=chatgpt.com)

[https://docs.rs/arrow-flight/60.0.0/arrow_flight/sql/client/](https://docs.rs/arrow-flight/60.0.0/arrow_flight/sql/client/?utm_source=chatgpt.com)

[https://github.com/apache/arrow-rs/blob/main/arrow-flight/README.md](https://github.com/apache/arrow-rs/blob/main/arrow-flight/README.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow v26.0.0+3

### 7.4 Network, security, and lifecycle boundaries

Flight uses gRPC transport, not shared-memory IPC between processes.

Consequently, it still incurs serialization, network transmission, transport buffering, and deserialization costs, even though Arrow is used as its data representation.

Tonic provides the transport mechanisms; the actual service is responsible for authorization policy, ticket validity, authentication interpretation, error handling, and resource lifecycle.

The `arrow-flight` package exposes optional TLS-related features through Tonic. They must be enabled and configured for deployments requiring those facilities.

For streaming systems, the main architectural opportunity is to preserve the record-batch abstraction across the network boundary, while allowing the application to manage streaming execution and backpressure on either side.

Reference: [https://docs.rs/crate/arrow-flight/60.0.0](https://docs.rs/crate/arrow-flight/60.0.0?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1

## 8. FFI, PyArrow, and cross-language interoperability

Arrow provides multiple mechanisms for moving typed data between programming environments. They differ in how much ownership, transport, and serialization machinery they require.

### 8.1 Arrow C Data Interface

The Arrow C Data Interface is a small, ABI-stable interoperability specification for exchanging Arrow arrays between components in the same process.

The core structures describe:

- An array's physical buffers and child arrays.
- Its schema and type information.
- Dictionary values when applicable.
- Ownership and release callbacks.

The defining advantage is that compatible producers and consumers can share array buffers without serializing them into a second representation.

However, this is a low-level memory interface, not a general-purpose serialization format. Its non-goals explicitly include exchange between processes and persistent storage.

The Rust implementation exposes FFI facilities through `arrow-array`, `arrow-data`, `arrow-schema`, and the umbrella `arrow` crate's optional `ffi` feature.

The interface is deliberately designed for implementations with different internal data structures. The receiver does not have to use the same Arrow library as the producer.

Memory safety depends on correctly implementing the release callbacks, ownership transfer, physical layout, and lifetimes. The Rust boundary necessarily involves carefully encapsulated `unsafe` operations.

References:

[https://arrow.apache.org/docs/format/CDataInterface.html](https://arrow.apache.org/docs/format/CDataInterface.html?utm_source=chatgpt.com)

[https://docs.rs/arrow/60.0.0/arrow/ffi/](https://docs.rs/arrow/60.0.0/arrow/ffi/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow v26.0.0+1

### 8.2 Arrow C Stream Interface

The C Stream Interface extends the same model to sequences of record batches.

`FFI_ArrowArrayStream` exposes the stream ABI, while `ArrowArrayStreamReader` adapts an imported stream into Rust's `RecordBatchReader` abstraction.

This is a consequential architectural capability: a consumer can process batches incrementally across a language boundary rather than requiring a complete dataset to be materialized first.

The difference between C Stream and Flight is fundamental:

| Mechanism          | Boundary                        | Principal value                                 |
| ------------------ | ------------------------------- | ----------------------------------------------- |
| C Data Interface   | Same process, individual arrays | ABI-stable array sharing                        |
| C Stream Interface | Same process, batch stream      | Incremental cross-language access               |
| Arrow IPC          | Serialized file or byte stream  | Persistence and process-independent interchange |
| Flight             | Network service                 | Remote, structured RPC and streaming            |

A C Stream implementation can preserve a streaming model, but it does not itself introduce asynchronous scheduling, network communication, or distributed execution.

References:

[https://arrow.apache.org/docs/format/CStreamInterface.html](https://arrow.apache.org/docs/format/CStreamInterface.html?utm_source=chatgpt.com)

[https://docs.rs/arrow/60.0.0/arrow/ffi_stream/](https://docs.rs/arrow/60.0.0/arrow/ffi_stream/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 8.3 `arrow-pyarrow`: Rust and Python integration

`arrow-pyarrow` provides PyO3-based conversion between Rust Arrow objects and PyArrow objects through Arrow's interoperability interfaces.

The main abstraction is `PyArrowType<T>`, supported by conversion traits such as `FromPyArrow`, `IntoPyArrow`, and `ToPyArrow`.

Its mappings include:

| PyArrow object              | Rust representation                            |
| --------------------------- | ---------------------------------------------- |
| `pyarrow.DataType`          | `DataType`                                     |
| `pyarrow.Field`             | `Field`                                        |
| `pyarrow.Schema`            | `Schema`                                       |
| `pyarrow.Array`             | `ArrayData`                                    |
| `pyarrow.RecordBatch`       | `RecordBatch`                                  |
| `pyarrow.RecordBatchReader` | `ArrowArrayStreamReader` or boxed batch reader |
| `pyarrow.Table`             | Convenience `Table` wrapper                    |

An important nuance is that the Rust `Table` wrapper internally contains `Vec<RecordBatch>` and is primarily a Python interoperability convenience. It is not evidence that `arrow-rs` has adopted PyArrow's chunked-table model throughout its core.

The documentation explicitly recommends streaming interfaces for general bulk data access. A PyArrow table can be imported through the Arrow Array Stream/PyCapsule interface without forcing eager conversion of the complete table into a Rust vector of batches.

This boundary is valuable for Python extension modules: computation can occur in native Rust while Arrow handles the typed memory interchange, avoiding the need for Python-object conversion per element.

It does not eliminate the costs of constructing new output arrays, running Python-facing code, or synchronizing ownership across the Python/Rust boundary.

References:

[https://docs.rs/arrow-pyarrow/60.0.0/arrow_pyarrow/](https://docs.rs/arrow-pyarrow/60.0.0/arrow_pyarrow/?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/format/PyCapsuleInterface.html](https://arrow.apache.org/docs/format/PyCapsuleInterface.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

## 9. Object storage and asynchronous IO

The `object_store` crate is one of the most important companions to Arrow and Parquet, particularly for large or remotely stored datasets.

It was previously housed in `arrow-rs` but now has its own repository and release lifecycle.

As of October 26, 2026, the latest verified release is `object_store` 0.14.2. Arrow Rust 60.0.0 updated its dependency to 0.14.1; the two projects do not share release numbers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 9.1 Unified storage semantics

`object_store::ObjectStore` provides a uniform asynchronous abstraction for multiple storage systems:

- Amazon S3 and compatible services.
- Azure Blob Storage.
- Google Cloud Storage.
- HTTP/WebDAV.
- Local filesystem storage.
- In-memory storage.

Its API deliberately follows object-store semantics rather than emulating a traditional seekable file.

Capabilities include ranged and vectored reads, streaming fetches, object listing, metadata queries, conditional reads/writes, multipart uploads, bulk deletion, and configurable storage backends.

The distinction matters because object stores provide atomic object-level operations rather than arbitrary modification of an existing file's byte ranges.

For example, conditional writes and ETag-based preconditions can support safe publication of immutable files or coordination around object versions without implementing separate filesystem-style locking.

The crate also provides `BufReader` and `BufWriter` adapters, request throttling, concurrent-request limiting, and URL-based storage configuration.

References:

[https://docs.rs/object_store/0.14.2/object_store/](https://docs.rs/object_store/0.14.2/object_store/?utm_source=chatgpt.com)

[https://github.com/apache/arrow-rs-object-store](https://github.com/apache/arrow-rs-object-store?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 9.2 Parquet integration through `AsyncFileReader`

The relationship between `parquet` and `object_store` is especially important.

`ParquetRecordBatchStreamBuilder` consumes an implementation of `AsyncFileReader`, an interface requiring:

- `get_bytes` for ranged reads.
- `get_byte_ranges` for multiple ranges, with a default implementation.
- `get_metadata` for Parquet metadata acquisition.

The interface enables file readers to exploit the storage backend's ability to retrieve selected regions without downloading complete files.

It also allows metadata retrieval to be customized. An implementation can supply cached or prefetched Parquet metadata, potentially avoiding repeated footer requests.

`SpawnedReader` can move IO operations onto a dedicated runtime when that separation is useful.

Version-sensitive distinction: `ParquetObjectReader` is deprecated since 59.2.0. The documentation recommends implementing `AsyncFileReader` directly, using its official `object_store` example. `ParquetObjectWriter` has also been deprecated in favor of more general writing interfaces.

This means the supported architectural extension point is the IO trait, not an obligatory direct dependency on one storage backend.

References:

[https://docs.rs/parquet/60.0.0/parquet/arrow/async_reader/trait.AsyncFileReader.html](https://docs.rs/parquet/60.0.0/parquet/arrow/async_reader/trait.AsyncFileReader.html?utm_source=chatgpt.com)

[https://docs.rs/parquet/60.0.0/parquet/arrow/async_reader/struct.SpawnedReader.html](https://docs.rs/parquet/60.0.0/parquet/arrow/async_reader/struct.SpawnedReader.html?utm_source=chatgpt.com)

[https://github.com/apache/arrow-rs/blob/main/parquet/examples/object_store.rs](https://github.com/apache/arrow-rs/blob/main/parquet/examples/object_store.rs?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 9.3 Combining storage, metadata, and selective decoding

Several native capabilities can combine into a sophisticated scanning architecture:

1. Object metadata and listings identify candidate files.
2. Parquet metadata and external indexes identify relevant row groups, pages, and columns.
3. `AsyncFileReader` retrieves the required byte ranges.
4. `ParquetPushDecoder` or an asynchronous stream reader decodes selected data into record batches.
5. Arrow kernels or a query engine perform further computation.

The important benefit is the ability to avoid unnecessary work at multiple levels: opening files, transferring bytes, decoding pages, materializing columns, and evaluating predicates.

The coordination and planning logic remain outside the core Arrow libraries. This is where a query engine or a specialized execution system can add substantial value.

## 10. Integration with broader Rust data systems

Arrow's most powerful ecosystem property is that its typed memory representation is shared by libraries with otherwise different responsibilities.

### 10.1 Apache DataFusion: relational execution over Arrow

Apache DataFusion is a separately maintained query engine that uses Arrow as its in-memory representation.

It adds capabilities that the core Arrow crates do not attempt to provide as an integrated system:

| Capability                                   | Arrow Rust                 | DataFusion                                       |
| -------------------------------------------- | -------------------------- | ------------------------------------------------ |
| Typed, immutable columnar arrays             | Native foundation          | Uses Arrow                                       |
| Vectorized compute kernels                   | Native foundation          | Uses and extends kernels                         |
| Record-batch streams                         | Native interfaces          | Core execution representation                    |
| SQL parsing and relational planning          | Not a core capability      | Integrated                                       |
| Logical and physical optimization            | Not a core capability      | Integrated                                       |
| Hash joins, relational aggregations, windows | Selected primitives only   | Full operators                                   |
| Partitioned and parallel execution           | Building blocks            | Integrated engine                                |
| Catalogs and table abstractions              | Schemas, not table catalog | `CatalogProvider`, `TableProvider`, related APIs |
| Extensible execution operators               | Kernel-level extension     | `ExecutionPlan` and optimizer extensions         |
| File-based relational scans                  | Parquet/CSV/JSON readers   | Planned, optimized scans                         |

DataFusion's `ExecutionPlan` abstraction operates on streams of Arrow record batches. Its planner and execution engine orchestrate partitioning, operators, expressions, data sources, and resource management around that representation.

That relationship is more significant than simply sharing some utility functions. A custom Arrow producer can become a DataFusion data source; a custom execution operator can consume and emit Arrow batches; an independently implemented compute kernel can participate in relational execution.

The main tradeoff is scope. For a bounded column transformation or specialized numerical operation, Arrow's direct kernels may be sufficient and give greater control over execution. For joins, complex aggregations, relational transformations, SQL, and optimizer-driven execution, DataFusion can replace substantial custom machinery.

DataFusion Ballista is a further, independently released layer for distributed execution. It is not part of `arrow-rs`.

References:

[https://datafusion.apache.org/](https://datafusion.apache.org/?utm_source=chatgpt.com)

[https://datafusion.apache.org/user-guide/arrow-introduction.html](https://datafusion.apache.org/user-guide/arrow-introduction.html?utm_source=chatgpt.com)

[https://github.com/apache/datafusion](https://github.com/apache/datafusion?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://datafusion.apache.org\&sz=32)

Apache DataFusion documentation+2

### 10.2 ADBC: Arrow-native database connectivity

Apache Arrow Database Connectivity (ADBC) is another important but distinct project.

ADBC standardizes database access using Arrow streams for result sets and Arrow-compatible data for parameter binding and ingestion.

Its Rust ecosystem includes:

- `adbc_core`: API abstractions, including driver, database, connection and statement interfaces.
- `adbc_driver_manager`: Dynamic loading and management of compatible drivers.
- `adbc_ffi`: FFI support for interoperability with drivers using the ADBC C ABI.

The ADBC Libraries 25 release, announced in September 2026, identifies Rust components at version 0.25.0, separate from `arrow-rs` 60.0.0 and from the ADBC 1.1.0 API specification.

The distinction between ADBC and Flight SQL is worth emphasizing:

| Aspect                          | ADBC                             | Flight SQL                        |
| ------------------------------- | -------------------------------- | --------------------------------- |
| Definition                      | Database connectivity API        | Network protocol                  |
| Core abstraction                | Drivers, connections, statements | RPC commands, endpoints, streams  |
| Transport                       | Depends on driver                | Flight/gRPC                       |
| Driver management               | Part of ecosystem                | Not the protocol's responsibility |
| Database discovery and metadata | Standard API methods             | Standard protocol commands        |
| Arrow results                   | Native streaming                 | Native streaming                  |

An ADBC driver may internally use Flight SQL, but that is an implementation choice. ADBC can also connect through other database-specific transports.

A significant integration consideration is Cargo version compatibility: two independently released libraries may pull in incompatible major versions of `arrow-array` or `arrow-schema`. Arrow values from different Rust crate major versions are not necessarily interchangeable as Rust types even if both conform to the same format specification. The official ADBC Rust documentation explicitly highlights this issue.

References:

[https://arrow.apache.org/adbc/](https://arrow.apache.org/adbc/?utm_source=chatgpt.com)

[https://arrow.apache.org/adbc/main/rust/](https://arrow.apache.org/adbc/main/rust/?utm_source=chatgpt.com)

[https://arrow.apache.org/adbc/main/rust/quickstart.html](https://arrow.apache.org/adbc/main/rust/quickstart.html?utm_source=chatgpt.com)

[https://github.com/apache/arrow-adbc/releases](https://github.com/apache/arrow-adbc/releases?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

### 10.3 Other integration boundaries

Other relevant ecosystem components include:

| Component                | Relationship to Arrow                                                               |
| ------------------------ | ----------------------------------------------------------------------------------- |
| `parquet_opendal`        | Adapts Parquet IO to OpenDAL-managed storage backends                               |
| Polars                   | Independent Rust DataFrame engine with Arrow-oriented columnar interoperability     |
| DuckDB                   | Independent analytical SQL engine that can interchange Arrow data                   |
| PyArrow                  | Python-facing Arrow implementation and extensive file/data tooling                  |
| Ballista                 | Distributed execution extension within the DataFusion ecosystem                     |
| Arrow C Device Interface | Specification for describing data associated with devices, including non-CPU memory |

The general distinction is between sharing a representation and sharing the same implementation. Compatible Arrow data can move across systems, but different engines may have different internal layouts, supported types, memory policies, ownership rules, and execution semantics.

The Rust `arrow-rs` crate itself should not be treated as a comprehensive GPU, distributed SQL, or tensor-computation engine simply because the broader Arrow ecosystem defines integration formats that support those areas.

References:

[https://github.com/apache/arrow-rs](https://github.com/apache/arrow-rs?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/format/CDeviceDataInterface.html](https://arrow.apache.org/docs/format/CDeviceDataInterface.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://apache.googlesource.com\&sz=32)

apache.googlesource.com+1

## 11. Resource management, correctness, and performance considerations

### 11.1 Memory accounting and ownership

Arrow's shared memory representation makes conventional memory accounting more subtle than summing the visible lengths of arrays.

For example, two arrays may share a buffer, a sliced array may retain a much larger parent allocation, and nested arrays may reference buffers belonging to multiple children.

The optional `pool` feature exposes `MemoryPool`, `MemoryReservation`, and `TrackingMemoryPool`.

`Array::claim` can recursively account for the buffers used by an array and its children while avoiding double counting of already-claimed shared memory. A slice claiming memory already attributed to its parent therefore does not necessarily increase the accounted usage.

This is valuable for understanding retained memory and implementing resource budgets. It should not be confused with an automatic global allocator that captures every allocation made by a process.

References:

[https://docs.rs/arrow-buffer/60.0.0/arrow_buffer/](https://docs.rs/arrow-buffer/60.0.0/arrow_buffer/?utm_source=chatgpt.com)

[https://docs.rs/arrow-array/60.0.0/arrow_array/array/trait.Array.html](https://docs.rs/arrow-array/60.0.0/arrow_array/array/trait.Array.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 11.2 Nulls, logical nulls, and missing values

Arrow stores nullability explicitly, but physical validity and logical nullness are not always identical.

For example, a dictionary array may have valid dictionary indices that reference null dictionary values. The parent validity bitmap alone does not fully describe the logical nulls of the array.

The `Array` trait therefore distinguishes APIs such as:

- `null_count()`: Physical null count associated with an array representation.
- `logical_null_count()`: Logical nullness, including relevant child or referenced-value semantics.
- `logical_nulls()`: Logical validity information where applicable.
- `may_have_logical_nulls()`: A conservative check when exact determination could be expensive.

This distinction matters for filtering, comparisons, aggregation, serialization, and validation.

Similarly, null is not interchangeable with floating-point `NaN`, an empty string, an empty list, a missing JSON property, or an application-level sentinel.

References:

[https://docs.rs/arrow-array/60.0.0/arrow_array/array/trait.Array.html](https://docs.rs/arrow-array/60.0.0/arrow_array/array/trait.Array.html?utm_source=chatgpt.com)

[https://arrow.apache.org/docs/format/Columnar.html](https://arrow.apache.org/docs/format/Columnar.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 11.3 Array invariants and error handling

Correct Arrow arrays must satisfy physical invariants concerning buffer lengths, offsets, child arrays, null counts, and type-specific representations.

The typed constructors and builders provide validated construction pathways. Lower-level `ArrayData` and FFI facilities offer more flexibility but require particular care.

The v60 release continued converting potential panics into fallible errors and fixed multiple correctness problems, including decimal conversions, array slicing, dictionary overflow handling, IPC parsing, and FFI lifetimes.

One consequential example is that dictionary-key overflow in certain selection operations now returns an error instead of panicking.

The distinction between `try_*` and infallible APIs remains useful, but not every malformed or application-invalid value can be detected merely by validating Arrow's physical structures.

Reference: [https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/](https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow

### 11.4 Performance is governed by representation and work avoided

There are several distinct sources of potential efficiency.

| Mechanism                | Work potentially avoided                 | Principal tradeoff                       |
| ------------------------ | ---------------------------------------- | ---------------------------------------- |
| Shared immutable buffers | Repeated copying and synchronization     | Retention of larger allocations          |
| Dictionary arrays        | Duplicated values and memory bandwidth   | Dictionary management and kernel support |
| Run-end encoding         | Materializing long repeated sequences    | Specialized processing requirements      |
| View arrays              | Copying variable-length values           | Indirection and retained data buffers    |
| Vectorized kernels       | Repeated per-row dispatch                | Intermediate arrays and materialization  |
| Row encoding             | Repeated multicolumn comparison dispatch | Encoding cost and temporary row buffers  |
| Projection pushdown      | Decoding unnecessary columns             | Requires accurate dependency knowledge   |
| Page/row-group pruning   | Fetching and decoding irrelevant data    | Metadata and index dependencies          |
| Asynchronous ranged IO   | Blocking on individual remote requests   | Scheduling and buffering complexity      |
| Batch streaming          | Full-dataset materialization             | Batch-size and backpressure decisions    |

These mechanisms have different applicability. For example, a run-end encoding can be highly advantageous for long repeated runs but unhelpful for randomly distributed values. A row encoding can accelerate repeated composite-key comparison but add unnecessary overhead to operations that only inspect one column.

The 60.0.0 release reports improvements in numerous microbenchmarks, including selection, rank, decimal parsing, and Parquet decoding. Those measurements establish improvement in the tested kernels, not a general performance multiplier for arbitrary workloads.

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow

### 11.5 Format boundaries and type preservation

Several seemingly straightforward transformations can change physical or logical semantics:

| Boundary                                | Important considerations                                                         |
| --------------------------------------- | -------------------------------------------------------------------------------- |
| Arrow → Parquet → Arrow                 | Physical type conversion, dictionary treatment, schema hints, temporal precision |
| Arrow → JSON → Arrow                    | Inferred numeric types, binary encoding, missing properties, nulls               |
| Arrow → Avro → Arrow                    | Schema resolution, logical-type mapping, schema changes                          |
| Arrow → Flight → Arrow                  | Dictionary handling, message boundaries, transport limits                        |
| Arrow Rust → PyArrow                    | Lifetime management, chunking, stream versus eager conversion                    |
| Arrow arrays → row encoding             | Comparison semantics, dictionary expansion, encoder compatibility                |
| Arrow extension type → generic consumer | Whether semantic metadata is recognized and preserved                            |

A shared Arrow representation significantly reduces translation requirements, but does not make all transformations identity operations.

The highest-value integration designs preserve schema information, type interpretation, ordering assumptions, and ownership explicitly at the boundaries where those guarantees matter.

### 11.6 Feature gates and release stability

Cargo feature selection materially changes the available capabilities.

| Package                  | Notable features                                                    | Status                     |
| ------------------------ | ------------------------------------------------------------------- | -------------------------- |
| `arrow`                  | `csv`, `ipc`, `json`                                                | Default-enabled in v60     |
| `arrow`                  | `ffi`, `pyarrow`, `ipc_compression`, `pool`, `async`, `prettyprint` | Opt-in                     |
| `arrow` / `arrow-schema` | `canonical_extension_types`                                         | Opt-in                     |
| `parquet`                | `arrow`, common compression codecs                                  | Default-enabled            |
| `parquet`                | `async`, `crc`, `encryption`, `cli`                                 | Opt-in                     |
| `parquet`                | `variant_experimental`, `geospatial`, `experimental`                | Experimental functionality |
| `arrow-flight`           | `flight-sql`                                                        | Opt-in                     |
| `arrow-flight`           | TLS-related features                                                | Opt-in                     |
| `arrow-avro`             | `async`, fingerprint options, specialized type features             | Opt-in                     |

The Parquet `crc` feature enables verification of page checksums when available. The `encryption` feature enables Parquet encryption support. Those are separate capabilities with different purposes.

Both Parquet Variant and geospatial functionality are explicitly marked experimental in the v60 package documentation. APIs enabled by the broader `experimental` feature are not promised the same stability as ordinary released interfaces.

Apache Arrow Rust follows Cargo-style semantic versioning, with major releases permitted to introduce breaking changes. Its upstream project also states that little-endian platforms are officially supported and tested, while big-endian compatibility is best-effort.

References:

[https://docs.rs/crate/arrow/60.0.0/features](https://docs.rs/crate/arrow/60.0.0/features?utm_source=chatgpt.com)

[https://docs.rs/crate/parquet/60.0.0/features](https://docs.rs/crate/parquet/60.0.0/features?utm_source=chatgpt.com)

[https://docs.rs/crate/arrow-flight/60.0.0](https://docs.rs/crate/arrow-flight/60.0.0?utm_source=chatgpt.com)

[https://github.com/apache/arrow-rs](https://github.com/apache/arrow-rs?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+3

## 12. Diagnostic and command-line tooling

Although Arrow's Rust libraries are primarily intended for programmatic use, the repository also includes useful command-line binaries, especially within the `parquet` crate.

With the `cli` feature enabled, the published Parquet package includes tools such as:

| Tool                        | Purpose                                                           |
| --------------------------- | ----------------------------------------------------------------- |
| `parquet-read`              | Inspect and read Parquet records                                  |
| `parquet-schema`            | Display Parquet schema information                                |
| `parquet-rowcount`          | Inspect row counts                                                |
| `parquet-layout`            | Inspect physical layout, column chunks and page-related structure |
| `parquet-index`             | Inspect Parquet index information                                 |
| `parquet-show-bloom-filter` | Inspect Bloom filter information                                  |
| `parquet-rewrite`           | Rewrite supported Parquet content                                 |
| `parquet-concat`            | Concatenate compatible Parquet files                              |
| `parquet-fromcsv`           | Convert CSV content to Parquet                                    |

These provide direct visibility into Parquet structures without requiring a SQL engine.

`parquet-layout` is particularly useful because its focus is the physical organization of the file rather than only its logical schema and values.

The published package documents installation through:

```
cargo install parquet --version 60.0.0 --features cli --locked
```

The `arrow-flight` package separately documents a Flight SQL command-line client.

References:

[https://github.com/apache/arrow-rs/tree/main/parquet/src/bin](https://github.com/apache/arrow-rs/tree/main/parquet/src/bin?utm_source=chatgpt.com)

[https://docs.rs/parquet/60.0.0/parquet/](https://docs.rs/parquet/60.0.0/parquet/?utm_source=chatgpt.com)

[https://arrow.apache.org/rust/parquet_layout/index.html](https://arrow.apache.org/rust/parquet_layout/index.html?utm_source=chatgpt.com)

[https://arrow.apache.org/rust/parquet_read/index.html](https://arrow.apache.org/rust/parquet_read/index.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

## 13. Architectural distinctions that matter most

The Rust Arrow ecosystem supports several levels of abstraction. They should be viewed as complementary rather than competing ways to perform every operation.

| Requirement or problem shape              | Native capability to investigate                            | Boundary to recognize                                               |
| ----------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------- |
| Typed memory shared across components     | `arrow-array`, `arrow-buffer`, `arrow-schema`               | Common representation does not imply common application semantics   |
| Compact complex data representation       | Nested arrays, dictionaries, run ends, extension types      | Kernel support and semantic interpretation vary                     |
| Custom high-performance column operations | `arrow-arith`, `arrow-cmp`, `arrow-select`, `arrow-ord`     | Operations may materialize new buffers                              |
| Composite-key ordering and grouping       | `arrow-row`                                                 | Preparation overhead and compatible converters                      |
| Direct columnar transformation pipelines  | Kernel composition and `RecordBatchReader`                  | Relational optimization remains outside Arrow                       |
| Persistent analytical datasets            | `parquet`                                                   | File planning and table management are separate                     |
| Low-latency selective file scans          | Parquet metadata, page indexes, `RowSelection`, `RowFilter` | Predicate evaluation and pruning decisions belong to a higher layer |
| Cloud-neutral file IO                     | `object_store` and `AsyncFileReader`                        | Storage requests and execution scheduling are separate              |
| Same-process cross-language data          | C Data / C Stream interfaces                                | Lifetimes, ownership, and ABI obligations                           |
| Rust/Python batch exchange                | `arrow-pyarrow`                                             | Streaming generally differs from eager table conversion             |
| Remote streaming                          | `arrow-flight`                                              | RPC transport and application service design                        |
| SQL database RPC                          | Flight SQL                                                  | Server capabilities and client conformance                          |
| General database drivers                  | ADBC                                                        | Separate API specification and driver ecosystem                     |
| Relational query planning and execution   | DataFusion                                                  | Separate engine built on Arrow                                      |
| Distributed relational computation        | Ballista and other execution systems                        | Not a native property of Arrow arrays                               |

The unifying architectural principle is that Arrow separates typed data representation from the systems responsible for organizing computation, storage, and transport.

That separation lets specialized libraries compose around shared arrays and record-batch streams, while preserving flexibility over execution engines, storage backends, languages, and application-level data models.

It also explains the ecosystem's most important limitation: possessing Arrow-compatible data does not, by itself, provide SQL semantics, a database catalog, automatic predicate optimization, durable table transactions, distributed scheduling, or domain-specific validation. Those capabilities are supplied by other layers or explicitly constructed around Arrow's primitives.

### Authoritative source routes for further investigation

The most useful durable entry points, beyond the topic-specific API pages above, are:

| Resource                                                                                                                                                      | Coverage                                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| [https://github.com/apache/arrow-rs](https://github.com/apache/arrow-rs?utm_source=chatgpt.com)                                                               | Source, crate topology, tagged releases, issues, tests     |
| [https://docs.rs/arrow/60.0.0/arrow/](https://docs.rs/arrow/60.0.0/arrow/?utm_source=chatgpt.com)                                                             | Umbrella Arrow API and re-exports                          |
| [https://docs.rs/arrow-array/60.0.0/arrow_array/](https://docs.rs/arrow-array/60.0.0/arrow_array/?utm_source=chatgpt.com)                                     | Arrays, builders, record batches, FFI and typed access     |
| [https://docs.rs/arrow-schema/60.0.0/arrow_schema/](https://docs.rs/arrow-schema/60.0.0/arrow_schema/?utm_source=chatgpt.com)                                 | Types, schemas, fields and extension metadata              |
| [https://docs.rs/arrow-row/60.0.0/arrow_row/](https://docs.rs/arrow-row/60.0.0/arrow_row/?utm_source=chatgpt.com)                                             | Comparable row encoding                                    |
| [https://docs.rs/parquet/60.0.0/parquet/](https://docs.rs/parquet/60.0.0/parquet/?utm_source=chatgpt.com)                                                     | Parquet IO, schemas, statistics, encodings and metadata    |
| [https://docs.rs/arrow-flight/60.0.0/arrow_flight/](https://docs.rs/arrow-flight/60.0.0/arrow_flight/?utm_source=chatgpt.com)                                 | Flight and Flight SQL                                      |
| [https://docs.rs/object_store/0.14.2/object_store/](https://docs.rs/object_store/0.14.2/object_store/?utm_source=chatgpt.com)                                 | Storage abstractions, remote IO and conditional operations |
| [https://arrow.apache.org/docs/format/](https://arrow.apache.org/docs/format/?utm_source=chatgpt.com)                                                         | Normative Arrow interchange specifications                 |
| [https://parquet.apache.org/docs/file-format/](https://parquet.apache.org/docs/file-format/?utm_source=chatgpt.com)                                           | Normative Parquet format specification                     |
| [https://arrow.apache.org/docs/status.html](https://arrow.apache.org/docs/status.html?utm_source=chatgpt.com)                                                 | Arrow implementation-support matrix                        |
| [https://parquet.apache.org/docs/file-format/implementationstatus/](https://parquet.apache.org/docs/file-format/implementationstatus/?utm_source=chatgpt.com) | Parquet implementation-support status                      |
| [https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/](https://arrow.apache.org/blog/2026/09/29/arrow-rs-60.0.0/?utm_source=chatgpt.com)                 | Version 60 release enhancements and breaking changes       |
| [https://datafusion.apache.org/](https://datafusion.apache.org/?utm_source=chatgpt.com)                                                                       | Independent relational execution engine                    |
| [https://arrow.apache.org/adbc/](https://arrow.apache.org/adbc/?utm_source=chatgpt.com)                                                                       | Arrow-native database connectivity API and libraries       |

The official release, crate documentation, and format specifications should be read together for version-sensitive implementation work. The format specification defines what an interoperable representation can express; the Rust crate documentation establishes what its current APIs implement; the release notes identify newly introduced functionality, compatibility changes, and limitations.