# `opentelemetry_sdk::logs::export::LogBatch`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.export.LogBatch.json).

<a id="op-8624f83b029674bcb04544cc"></a>
## LogBatch

`struct` · `opentelemetry_sdk::logs::export::LogBatch` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct LogBatch<'a>
```

Source: `src/logs/export.rs:21`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A batch of log records to be exported by a `LogExporter`.

The `LogBatch` struct holds a collection of log records along with their associated
instrumentation scopes. This structure is used to group log records together for efficient
export operations.

# Type Parameters
- `'a`: The lifetime of the references to the log records and instrumentation scopes.


<a id="op-73183fb585572c72b2332c03"></a>
## fmt

`function` · `opentelemetry_sdk::logs::export::LogBatch::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry_sdk::logs::export::LogBatch", "path": "LogBatch"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [20, 10], "end": [20, 15], "filename": "src/logs/export.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/export.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8954fcd9ac729438696bafde"></a>
## iter

`function` · `opentelemetry_sdk::logs::export::LogBatch::iter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn iter(&self) -> impl Iterator<Item = (&SdkLogRecord, &InstrumentationScope)>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry_sdk::logs::export::LogBatch", "path": "LogBatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [65, 1], "end": [81, 2], "filename": "src/logs/export.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/export.rs:75`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the log records and instrumentation scopes in the batch.

Each item yielded by the iterator is a tuple containing references to a `LogRecord`
and an `InstrumentationScope`.

# Returns

An iterator that yields references to the `LogRecord` and `InstrumentationScope` in the batch.


<a id="op-c85a4f4c4dd5b2b9a082a5b2"></a>
## new

`function` · `opentelemetry_sdk::logs::export::LogBatch::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(data: &'a [(&'a SdkLogRecord, &'a InstrumentationScope)]) -> LogBatch<'a>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry_sdk::logs::export::LogBatch", "path": "LogBatch"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [35, 1], "end": [63, 2], "filename": "src/logs/export.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/export.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of `LogBatch`.

# Arguments

* `data` - A slice of tuples, where each tuple consists of a reference to a `LogRecord`
  and a reference to an `InstrumentationScope`. These tuples represent the log records
  and their associated instrumentation scopes to be exported.

# Returns

A `LogBatch` instance containing the provided log records and instrumentation scopes.

Note - this is not a public function, and should not be used directly. This would be
made private in the future.
