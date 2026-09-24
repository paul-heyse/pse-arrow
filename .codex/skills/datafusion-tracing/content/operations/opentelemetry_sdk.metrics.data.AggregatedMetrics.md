# `opentelemetry_sdk::metrics::data::AggregatedMetrics`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.AggregatedMetrics.json).

<a id="op-cd6c30b8b499d6236b40d4b6"></a>
## AggregatedMetrics

`enum` · `opentelemetry_sdk::metrics::data::AggregatedMetrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum AggregatedMetrics
```

Source: `src/metrics/data/mod.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Aggregated metrics data from an instrument

<a id="op-4115fa7953b83a366a437325"></a>
## F64

`variant` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::F64` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
F64
```

Source: `src/metrics/data/mod.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

All metric data with `f64` value type

<a id="op-3e00fe08ff6554d75b5acd7f"></a>
## I64

`variant` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::I64` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
I64
```

Source: `src/metrics/data/mod.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

All metric data with `i64` value type

<a id="op-d5cf3ad0a6bc33f65d3d65cc"></a>
## U64

`variant` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::U64` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
U64
```

Source: `src/metrics/data/mod.rs:105`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

All metric data with `u64` value type

<a id="op-419f482a276ff1242e88456c"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::AggregatedMetrics", "path": "AggregatedMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [100, 10], "end": [100, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:100`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3c9d27ae05dc4561b746ed05"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: MetricData<u64>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::AggregatedMetrics", "path": "AggregatedMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [135, 1], "end": [139, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:136`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3ee0759b0a269709eb13e139"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: MetricData<i64>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::AggregatedMetrics", "path": "AggregatedMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [129, 1], "end": [133, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:130`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e5ed356489c840ed24709256"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::AggregatedMetrics::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: MetricData<f64>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::AggregatedMetrics", "path": "AggregatedMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [123, 1], "end": [127, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:124`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
