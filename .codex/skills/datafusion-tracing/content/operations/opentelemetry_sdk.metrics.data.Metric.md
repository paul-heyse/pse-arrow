# `opentelemetry_sdk::metrics::data::Metric`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.Metric.json).

<a id="op-f7695837d73ebe6f39a46c66"></a>
## Metric

`struct` · `opentelemetry_sdk::metrics::data::Metric` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Metric
```

Source: `src/metrics/data/mod.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A collection of one or more aggregated time series from an [Instrument].

[Instrument]: crate::metrics::Instrument

<a id="op-6f1120f491a6a2c936dd4fca"></a>
## data

`function` · `opentelemetry_sdk::metrics::data::Metric::data` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn data(&self) -> &AggregatedMetrics
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::Metric", "path": "Metric"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [97, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:94`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the aggregated data from the instrument.

<a id="op-2877d1aad4ab605b223f4c7b"></a>
## description

`function` · `opentelemetry_sdk::metrics::data::Metric::description` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn description(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::Metric", "path": "Metric"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [97, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the description of the instrument.

<a id="op-78b1f49afe8edd073101f4a1"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::Metric::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::Metric", "path": "Metric"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [65, 10], "end": [65, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:65`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-435515d442af97a39e700975"></a>
## name

`function` · `opentelemetry_sdk::metrics::data::Metric::name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::Metric", "path": "Metric"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [97, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the name of the instrument that created this data.

<a id="op-9c92a378713ff5ce4d260442"></a>
## unit

`function` · `opentelemetry_sdk::metrics::data::Metric::unit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn unit(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::Metric", "path": "Metric"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [97, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:89`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the unit in which the instrument reports.
