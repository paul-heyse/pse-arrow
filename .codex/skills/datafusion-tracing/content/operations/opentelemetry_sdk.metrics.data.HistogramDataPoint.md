# `opentelemetry_sdk::metrics::data::HistogramDataPoint`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.HistogramDataPoint.json).

<a id="op-b6b4ea2436f236d73ddf99ed"></a>
## HistogramDataPoint

`struct` · `opentelemetry_sdk::metrics::data::HistogramDataPoint` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct HistogramDataPoint<T>
```

Source: `src/metrics/data/mod.rs:338`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A single histogram data point in a time series.

<a id="op-a87943e78dfb75d5cfd8c4f2"></a>
## attributes

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn attributes(&self) -> impl Iterator<Item = &KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [386, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:363`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the attributes in [HistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.HistogramDataPoint.md#op-b6b4ea2436f236d73ddf99ed).

<a id="op-05783979a8c4342c2c631588"></a>
## bounds

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::bounds` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn bounds(&self) -> impl Iterator<Item = f64> + '_
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [386, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:373`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the bucket boundaries in [HistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.HistogramDataPoint.md#op-b6b4ea2436f236d73ddf99ed).

<a id="op-a51a9c29995762c3c07d941c"></a>
## bucket_counts

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::bucket_counts` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn bucket_counts(&self) -> impl Iterator<Item = u64> + '_
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [386, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:378`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the bucket counts in [HistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.HistogramDataPoint.md#op-b6b4ea2436f236d73ddf99ed).

<a id="op-8ed57afe186d4dbd5ecb4af1"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> HistogramDataPoint<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [337, 17], "end": [337, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:337`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3b6bf3d7780805b821c6c51f"></a>
## count

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::count` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn count(&self) -> u64
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [386, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:383`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the number of updates this histogram has been calculated with.

<a id="op-152d16b583634d5117d0e84a"></a>
## eq

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &HistogramDataPoint<T>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [337, 24], "end": [337, 33], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/data/mod.rs:337`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4e44a955864f78da58a3f9ae"></a>
## exemplars

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::exemplars` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [386, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:368`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the exemplars in [HistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.HistogramDataPoint.md#op-b6b4ea2436f236d73ddf99ed).

<a id="op-8a1df6b90d119e7493c51759"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [337, 10], "end": [337, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:337`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5a87b7beeedab5dcef9e53c"></a>
## max

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::max` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn max(&self) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [388, 1], "end": [403, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:395`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the maximum value recorded.

<a id="op-a463e9ea85f9348e90e81f33"></a>
## min

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::min` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn min(&self) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [388, 1], "end": [403, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:390`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the minimum value recorded.

<a id="op-da0234eadbfba1170a0ef41c"></a>
## sum

`function` · `opentelemetry_sdk::metrics::data::HistogramDataPoint::sum` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn sum(&self) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::HistogramDataPoint", "path": "HistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [388, 1], "end": [403, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:400`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the sum of the values recorded.
