# `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.manual_reader.ManualReaderBuilder.json).

<a id="op-1b8c0287afffa50251e104db"></a>
## ManualReaderBuilder

`struct` · `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ManualReaderBuilder
```

Source: `src/metrics/manual_reader.rs:137`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration for a [ManualReader](../operations/opentelemetry_sdk.metrics.manual_reader.ManualReader.md#op-ae28e473e5d34d7697a88a62)

<a id="op-776aa4571a10cdf73b2e6605"></a>
## build

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ManualReader
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder", "path": "ManualReaderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [163, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/manual_reader.rs:160`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new [ManualReader](../operations/opentelemetry_sdk.metrics.manual_reader.ManualReader.md#op-ae28e473e5d34d7697a88a62) from this configuration.

<a id="op-b76f314fcadbc13b00504924"></a>
## default

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> ManualReaderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder", "path": "ManualReaderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [136, 10], "end": [136, 17], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/manual_reader.rs:136`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2aa28af67970b1b3b8a2a8ff"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder", "path": "ManualReaderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 1], "end": [145, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/manual_reader.rs:142`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f2b97d191127d7a49d508042"></a>
## new

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder", "path": "ManualReaderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [163, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/manual_reader.rs:149`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

New manual builder configuration

<a id="op-dace85d27005f7f517152f45"></a>
## with_temporality

`function` · `opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder::with_temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_temporality(self, temporality: Temporality) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::manual_reader::ManualReaderBuilder", "path": "ManualReaderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [163, 2], "filename": "src/metrics/manual_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/manual_reader.rs:154`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the [Temporality](../operations/opentelemetry_sdk.metrics.Temporality.md#op-dc2884d798040a5373238d65) of the exporter.
