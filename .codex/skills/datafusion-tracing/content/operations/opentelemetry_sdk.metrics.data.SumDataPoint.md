# `opentelemetry_sdk::metrics::data::SumDataPoint`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.SumDataPoint.json).

<a id="op-7afab8f11fcbb0467bb350da"></a>
## SumDataPoint

`struct` · `opentelemetry_sdk::metrics::data::SumDataPoint` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SumDataPoint<T>
```

Source: `src/metrics/data/mod.rs:226`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

DataPoint is a single data point in a time series.

<a id="op-606d74fd2b33271d9055ef4d"></a>
## attributes

`function` · `opentelemetry_sdk::metrics::data::SumDataPoint::attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn attributes(&self) -> impl Iterator<Item = &KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::SumDataPoint", "path": "SumDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [236, 1], "end": [246, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:238`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the attributes in [SumDataPoint](../operations/opentelemetry_sdk.metrics.data.SumDataPoint.md#op-7afab8f11fcbb0467bb350da).

<a id="op-090d4a46fcc42c4b595b22b4"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::SumDataPoint::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SumDataPoint<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::SumDataPoint", "path": "SumDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [225, 17], "end": [225, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8ad03eed713ba3a0e855588a"></a>
## eq

`function` · `opentelemetry_sdk::metrics::data::SumDataPoint::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SumDataPoint<T>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::SumDataPoint", "path": "SumDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [225, 24], "end": [225, 33], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/data/mod.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-52945a5c270b387dd2afc9b6"></a>
## exemplars

`function` · `opentelemetry_sdk::metrics::data::SumDataPoint::exemplars` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn exemplars(&self) -> impl Iterator<Item = &Exemplar<T>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::SumDataPoint", "path": "SumDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [236, 1], "end": [246, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:243`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the [Exemplar](../operations/opentelemetry_sdk.metrics.data.Exemplar.md#op-95720da4f16acadf482dd981)s in [SumDataPoint](../operations/opentelemetry_sdk.metrics.data.SumDataPoint.md#op-7afab8f11fcbb0467bb350da).

<a id="op-10c82fb63b476a55a4a2d587"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::SumDataPoint::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::SumDataPoint", "path": "SumDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [225, 10], "end": [225, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fe243608efa758111dd36dfa"></a>
## value

`function` · `opentelemetry_sdk::metrics::data::SumDataPoint::value` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn value(&self) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::SumDataPoint", "path": "SumDataPoint"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Copy", "path": "Copy"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [248, 1], "end": [253, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:250`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the value of this data point.
