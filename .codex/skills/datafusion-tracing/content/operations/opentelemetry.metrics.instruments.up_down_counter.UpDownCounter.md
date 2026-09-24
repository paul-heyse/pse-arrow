# `opentelemetry::metrics::instruments::up_down_counter::UpDownCounter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.json).

<a id="op-b0254753eff4c3e7cb7010cc"></a>
## UpDownCounter

`struct` · `opentelemetry::metrics::instruments::up_down_counter::UpDownCounter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct UpDownCounter<T>
```

Source: `src/metrics/instruments/up_down_counter.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An instrument that records increasing or decreasing values.

[`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) can be cloned to create multiple handles to the same instrument. If a [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) needs to be shared,
users are recommended to clone the [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) instead of creating duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric. Creating
duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric could lower SDK performance.

<a id="op-54c05b175aa97bdc67983a7c"></a>
## add

`function` · `opentelemetry::metrics::instruments::up_down_counter::UpDownCounter::add` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add(&self, value: T, attributes: &[KeyValue])
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::UpDownCounter", "path": "UpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [38, 2], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/up_down_counter.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Records an increment or decrement to the counter.

<a id="op-8ca9fee2913caa4ceb634fb5"></a>
## clone

`function` · `opentelemetry::metrics::instruments::up_down_counter::UpDownCounter::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> UpDownCounter<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::UpDownCounter", "path": "UpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 10], "end": [12, 15], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instruments/up_down_counter.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c43bf3c96d3294f044afefc9"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::up_down_counter::UpDownCounter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::UpDownCounter", "path": "UpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [16, 1], "end": [26, 2], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/up_down_counter.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4fd7a7e1f86c2436188af7d1"></a>
## new

`function` · `opentelemetry::metrics::instruments::up_down_counter::UpDownCounter::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::UpDownCounter", "path": "UpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [38, 2], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/up_down_counter.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new up down counter.
