# `opentelemetry_sdk::metrics::data::ResourceMetrics`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.ResourceMetrics.json).

<a id="op-779be1786eb4f37fa53a44d6"></a>
## ResourceMetrics

`struct` · `opentelemetry_sdk::metrics::data::ResourceMetrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ResourceMetrics
```

Source: `src/metrics/data/mod.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A collection of [ScopeMetrics](../operations/opentelemetry_sdk.metrics.data.ScopeMetrics.md#op-582c94a4752d1d6acbc34515) and the associated [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) that created them.

<a id="op-ee817529955a3b223451e702"></a>
## default

`function` · `opentelemetry_sdk::metrics::data::ResourceMetrics::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ResourceMetrics", "path": "ResourceMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [20, 1], "end": [27, 2], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/data/mod.rs:21`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-daf345c1686fa554999d9c86"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::ResourceMetrics::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ResourceMetrics", "path": "ResourceMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 10], "end": [12, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-04eceea385702b10676634e2"></a>
## resource

`function` · `opentelemetry_sdk::metrics::data::ResourceMetrics::resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn resource(&self) -> &Resource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ResourceMetrics", "path": "ResourceMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [39, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns a reference to the [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) in [ResourceMetrics](../operations/opentelemetry_sdk.metrics.data.ResourceMetrics.md#op-779be1786eb4f37fa53a44d6).

<a id="op-a244216d434f509defb8f720"></a>
## scope_metrics

`function` · `opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn scope_metrics(&self) -> impl Iterator<Item = &ScopeMetrics>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ResourceMetrics", "path": "ResourceMetrics"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [39, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the [ScopeMetrics](../operations/opentelemetry_sdk.metrics.data.ScopeMetrics.md#op-582c94a4752d1d6acbc34515) in [ResourceMetrics](../operations/opentelemetry_sdk.metrics.data.ResourceMetrics.md#op-779be1786eb4f37fa53a44d6).
