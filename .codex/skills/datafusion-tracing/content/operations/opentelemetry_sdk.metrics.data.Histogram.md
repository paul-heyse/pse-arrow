# `opentelemetry_sdk::metrics::data::Histogram`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.Histogram.json).

<a id="op-803003ee4b88e360f1ff0491"></a>
## Histogram

`struct` · `opentelemetry_sdk::metrics::data::Histogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Histogram<T>
```

Source: `src/metrics/data/mod.rs:301`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Represents the histogram of all measurements of values from an instrument.

<a id="op-4b2ff872d6afc2a759bc87aa"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::Histogram::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Histogram<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 17], "end": [300, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:300`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4b31aa38817921fce7482507"></a>
## data_points

`function` · `opentelemetry_sdk::metrics::data::Histogram::data_points` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn data_points(&self) -> impl Iterator<Item = &HistogramDataPoint<T>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [313, 1], "end": [334, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:315`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the [HistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.HistogramDataPoint.md#op-b6b4ea2436f236d73ddf99ed)s in [Histogram](../operations/opentelemetry_sdk.metrics.data.Histogram.md#op-803003ee4b88e360f1ff0491).

<a id="op-dc04d0842159678d3a0d098d"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::Histogram::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 10], "end": [300, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:300`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2456598cc1026c52d59a285a"></a>
## start_time

`function` · `opentelemetry_sdk::metrics::data::Histogram::start_time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn start_time(&self) -> SystemTime
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [313, 1], "end": [334, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:320`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the time when the time series was started.

<a id="op-ebbc646fc7db204a1d41237e"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::data::Histogram::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [313, 1], "end": [334, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:331`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the temporality describing if the aggregation is reported as the change
from the last report time, or the cumulative changes since a fixed start time.

<a id="op-56f496363762da02ca373edc"></a>
## time

`function` · `opentelemetry_sdk::metrics::data::Histogram::time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn time(&self) -> SystemTime
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [313, 1], "end": [334, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:325`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the time when the time series was recorded.
