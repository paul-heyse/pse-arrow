# `opentelemetry_sdk::metrics::data::MetricData`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.MetricData.json).

<a id="op-cb1d7404cbfdfe73367a20c9"></a>
## MetricData

`enum` · `opentelemetry_sdk::metrics::data::MetricData` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum MetricData<T>
```

Source: `src/metrics/data/mod.rs:112`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Metric data for all types

<a id="op-d351bb0e30010cec8623cb1e"></a>
## ExponentialHistogram

`variant` · `opentelemetry_sdk::metrics::data::MetricData::ExponentialHistogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ExponentialHistogram
```

Source: `src/metrics/data/mod.rs:120`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Metric data for ExponentialHistogram

<a id="op-7ad276636c89368dc98c52c1"></a>
## Gauge

`variant` · `opentelemetry_sdk::metrics::data::MetricData::Gauge` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Gauge
```

Source: `src/metrics/data/mod.rs:114`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Metric data for Gauge

<a id="op-782e2849aecbbabb74840b87"></a>
## Histogram

`variant` · `opentelemetry_sdk::metrics::data::MetricData::Histogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Histogram
```

Source: `src/metrics/data/mod.rs:118`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Metric data for Histogram

<a id="op-c73b909d51e7c985b97f25f9"></a>
## Sum

`variant` · `opentelemetry_sdk::metrics::data::MetricData::Sum` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Sum
```

Source: `src/metrics/data/mod.rs:116`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Metric data for Sum

<a id="op-fc7a57a6a3e4090626950cb7"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::MetricData::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [111, 10], "end": [111, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:111`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-55d4f1822b55abbc6b2e0d9f"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::MetricData::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: Histogram<T>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [153, 1], "end": [157, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Histogram", "path": "Histogram"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:154`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-84e2999272bcf6508d555c41"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::MetricData::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: ExponentialHistogram<T>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [159, 1], "end": [163, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogram", "path": "ExponentialHistogram"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:160`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-905a37adc36426c5c4f0afe7"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::MetricData::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: Gauge<T>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 1], "end": [145, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Gauge", "path": "Gauge"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:142`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ce164064e380ab75f4831cb6"></a>
## from

`function` · `opentelemetry_sdk::metrics::data::MetricData::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: Sum<T>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::MetricData", "path": "MetricData"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [151, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Sum", "path": "Sum"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metrics/data/mod.rs:148`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
