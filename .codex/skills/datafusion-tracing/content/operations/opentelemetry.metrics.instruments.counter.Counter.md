# `opentelemetry::metrics::instruments::counter::Counter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.counter.Counter.json).

<a id="op-c72b21c1e56e7cb9d5a026e5"></a>
## Counter

`struct` · `opentelemetry::metrics::instruments::counter::Counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Counter<T>
```

Source: `src/metrics/instruments/counter.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An instrument that records increasing values.

[`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) can be cloned to create multiple handles to the same instrument. If a [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) needs to be shared,
users are recommended to clone the [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) instead of creating duplicate [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5)s for the same metric. Creating
duplicate [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5)s for the same metric could lower SDK performance.

<a id="op-c19ab6e831c1ec283eeaaa45"></a>
## add

`function` · `opentelemetry::metrics::instruments::counter::Counter::add` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add(&self, value: T, attributes: &[KeyValue])
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::Counter", "path": "Counter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 1], "end": [35, 2], "filename": "src/metrics/instruments/counter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/counter.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Records an increment to the counter.

<a id="op-d4d1c14297cf4ec040456b3e"></a>
## clone

`function` · `opentelemetry::metrics::instruments::counter::Counter::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Counter<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::Counter", "path": "Counter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 10], "end": [12, 15], "filename": "src/metrics/instruments/counter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instruments/counter.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9fd43f70fb7d92ec1549e7e3"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::counter::Counter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::Counter", "path": "Counter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [16, 1], "end": [23, 2], "filename": "src/metrics/instruments/counter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/counter.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4baed76dfbded91f06605ac1"></a>
## new

`function` · `opentelemetry::metrics::instruments::counter::Counter::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::Counter", "path": "Counter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 1], "end": [35, 2], "filename": "src/metrics/instruments/counter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/counter.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new counter.
