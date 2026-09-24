# `opentelemetry_sdk::metrics::data::ScopeMetrics`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.ScopeMetrics.json).

<a id="op-582c94a4752d1d6acbc34515"></a>
## ScopeMetrics

`struct` · `opentelemetry_sdk::metrics::data::ScopeMetrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ScopeMetrics
```

Source: `src/metrics/data/mod.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A collection of metrics produced by a meter.

<a id="op-2f91febf5be544f18bf7c5d9"></a>
## default

`function` · `opentelemetry_sdk::metrics::data::ScopeMetrics::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> ScopeMetrics
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ScopeMetrics", "path": "ScopeMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [42, 10], "end": [42, 17], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/data/mod.rs:42`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5721b865dc49b89fd8490f7"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::ScopeMetrics::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ScopeMetrics", "path": "ScopeMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [42, 19], "end": [42, 24], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:42`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-03eff279b0647fece4b36204"></a>
## metrics

`function` · `opentelemetry_sdk::metrics::data::ScopeMetrics::metrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn metrics(&self) -> impl Iterator<Item = &Metric>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ScopeMetrics", "path": "ScopeMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [50, 1], "end": [60, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:57`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the [Metric](../operations/opentelemetry_sdk.metrics.data.Metric.md#op-f7695837d73ebe6f39a46c66)s in [ScopeMetrics](../operations/opentelemetry_sdk.metrics.data.ScopeMetrics.md#op-582c94a4752d1d6acbc34515).

<a id="op-322e332e14961ee422012f9f"></a>
## scope

`function` · `opentelemetry_sdk::metrics::data::ScopeMetrics::scope` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn scope(&self) -> &InstrumentationScope
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ScopeMetrics", "path": "ScopeMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [50, 1], "end": [60, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:52`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns a reference to the [InstrumentationScope](../operations/opentelemetry.common.InstrumentationScope.md#op-5a674011980d528952015efb) in [ScopeMetrics](../operations/opentelemetry_sdk.metrics.data.ScopeMetrics.md#op-582c94a4752d1d6acbc34515).
