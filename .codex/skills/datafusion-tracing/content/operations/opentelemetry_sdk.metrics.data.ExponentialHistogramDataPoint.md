# `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.ExponentialHistogramDataPoint.json).

<a id="op-106c6fd274f2846fd6ed3db1"></a>
## ExponentialHistogramDataPoint

`struct` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ExponentialHistogramDataPoint<T>
```

Source: `src/metrics/data/mod.rs:444`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A single exponential histogram data point in a time series.

<a id="op-4f9b39647607948676071d51"></a>
## attributes

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn attributes(&self) -> impl Iterator<Item = &KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:489`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the attributes in [ExponentialHistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.ExponentialHistogramDataPoint.md#op-106c6fd274f2846fd6ed3db1).

<a id="op-c15baf42de09671a1dca152c"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ExponentialHistogramDataPoint<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [443, 17], "end": [443, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:443`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f0abe129251a5a8b8afc021d"></a>
## count

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::count` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn count(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:499`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the number of updates this histogram has been calculated with.

<a id="op-52b1aef0bf0fbdc55606cd87"></a>
## eq

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &ExponentialHistogramDataPoint<T>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [443, 24], "end": [443, 33], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/data/mod.rs:443`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1791a5252ca61e884c200cf8"></a>
## exemplars

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::exemplars` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:494`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the exemplars in [ExponentialHistogramDataPoint](../operations/opentelemetry_sdk.metrics.data.ExponentialHistogramDataPoint.md#op-106c6fd274f2846fd6ed3db1).

<a id="op-a93b39ffb96c47c77f4a517f"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [443, 10], "end": [443, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:443`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4a1abc42aaf0f3044b5d8811"></a>
## max

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::max` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn max(&self) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [529, 1], "end": [544, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:536`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the maximum value recorded.

<a id="op-35294d8eb4d7b79d810bc882"></a>
## min

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::min` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn min(&self) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [529, 1], "end": [544, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:531`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the minimum value recorded.

<a id="op-a61e92e115ca7ed8188b21aa"></a>
## negative_bucket

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::negative_bucket` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn negative_bucket(&self) -> &ExponentialBucket
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:519`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the range of negative value bucket counts.

<a id="op-4333b6acda854e9025a54f72"></a>
## positive_bucket

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::positive_bucket` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn positive_bucket(&self) -> &ExponentialBucket
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:514`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the range of positive value bucket counts.

<a id="op-510093d28abe0f754646dd45"></a>
## scale

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::scale` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn scale(&self) -> i8
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:504`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the resolution of the histogram.

<a id="op-5b30ba8ba62935d5e365758a"></a>
## sum

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::sum` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn sum(&self) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [529, 1], "end": [544, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:541`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the sum of the values recorded.

<a id="op-939e1286fda334f16049562f"></a>
## zero_count

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::zero_count` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn zero_count(&self) -> u64
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:509`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the number of values whose absolute value is less than or equal to zero_threshold.

<a id="op-74c386152a03cdfbac7ed890"></a>
## zero_threshold

`function` · `opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint::zero_threshold` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn zero_threshold(&self) -> f64
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::ExponentialHistogramDataPoint", "path": "ExponentialHistogramDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [487, 1], "end": [527, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:524`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the width of the zero region.
