# `opentelemetry_sdk::metrics::data::ExponentialHistogram`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.ExponentialHistogram.json).

<a id="op-18e96c357a99104505003e9b"></a>
## ExponentialHistogram

`struct` · `opentelemetry_sdk::metrics::data::ExponentialHistogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ExponentialHistogram<T>
```

Source: `src/metrics/data/mod.rs:407`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The histogram of all measurements of values from an instrument.

<a id="op-ccf8aaa4dd9278be26f1d3ad"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogram::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ExponentialHistogram<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [406, 17], "end": [406, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:406`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-51dc97169f120f73eeb4ac15"></a>
## data_points

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogram::data_points` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn data_points(&self) -> impl Iterator<Item = &ExponentialHistogramDataPoint<T>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [419, 1], "end": [440, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:421`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the [ExponentialHistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.ExponentialHistogramDataPoint.md#op-106c6fd274f2846fd6ed3db1)s in [ExponentialHistogram](../operations/opentelemetry_sdk.metrics.data.ExponentialHistogram.md#op-18e96c357a99104505003e9b).

<a id="op-80da758317a02bff0073aea2"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogram::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [406, 10], "end": [406, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:406`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ceb7db9c001f890d390302cb"></a>
## start_time

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogram::start_time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn start_time(&self) -> SystemTime
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [419, 1], "end": [440, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:426`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the time when the time series was started.

<a id="op-fd9193361a3dcd60af364a47"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogram::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [419, 1], "end": [440, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:437`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the temporality describing if the aggregation is reported as the change
from the last report time, or the cumulative changes since a fixed start time.

<a id="op-45f0ef7163f309bec5ed068f"></a>
## time

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogram::time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn time(&self) -> SystemTime
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [419, 1], "end": [440, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:431`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the time when the time series was recorded.
