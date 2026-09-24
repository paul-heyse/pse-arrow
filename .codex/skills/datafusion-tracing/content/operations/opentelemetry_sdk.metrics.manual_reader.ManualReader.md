# `opentelemetry_sdk::metrics::manual_reader::ManualReader`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.manual_reader.ManualReader.json).

<a id="op-ae28e473e5d34d7697a88a62"></a>
## ManualReader

`struct` · `opentelemetry_sdk::metrics::manual_reader::ManualReader` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ManualReader
```

Source: `src/metrics/manual_reader.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A simple [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) that allows an application to read metrics on demand.

See [ManualReaderBuilder](../operations/opentelemetry_sdk.metrics.manual_reader.ManualReaderBuilder.md#op-1b8c0287afffa50251e104db) for configuration options.

# Example

```
use opentelemetry_sdk::metrics::ManualReader;

// can specify additional reader configuration
let reader = ManualReader::builder().build();
# drop(reader)
```

<a id="op-81246f7279425a3cf65f1967"></a>
## builder

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> ManualReaderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [71, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/manual_reader.rs:57`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration for this reader

<a id="op-6a5021d0c8e77c2ce7584ba4"></a>
## collect

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::collect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [73, 1], "end": [133, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/manual_reader.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Gathers all metrics from the SDK, calling any
callbacks necessary and returning the results.

Returns an error if called after shutdown.

<a id="op-dec90a71ebfd7ec9b8740865"></a>
## default

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [37, 1], "end": [41, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/manual_reader.rs:38`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-be64c78fa62f4c6f9f754c81"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [43, 1], "end": [47, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/manual_reader.rs:44`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e02d5c3b81b7c68dc462db10"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [73, 1], "end": [133, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/manual_reader.rs:112`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

ForceFlush is a no-op, it always returns nil.

<a id="op-0f14ade8458f4441b2599cf0"></a>
## register_pipeline

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::register_pipeline` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn register_pipeline(&self, pipeline: Weak<Pipeline>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [73, 1], "end": [133, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/manual_reader.rs:76`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Register a pipeline which enables the caller to read metrics from the SDK
on demand.

<a id="op-b12ba785d16db8aceb20d241"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [73, 1], "end": [133, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/manual_reader.rs:117`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Closes any connections and frees any resources used by the reader.

<a id="op-748633a9f40b7b9b047e54b4"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReader::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self, kind: super::InstrumentKind) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReader", "path": "ManualReader"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [73, 1], "end": [133, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/manual_reader.rs:130`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
