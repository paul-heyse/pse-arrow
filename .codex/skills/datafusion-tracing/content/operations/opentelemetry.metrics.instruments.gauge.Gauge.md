# `opentelemetry::metrics::instruments::gauge::Gauge`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.gauge.Gauge.json).

<a id="op-01a034e74bec61ee471db7bf"></a>
## Gauge

`struct` · `opentelemetry::metrics::instruments::gauge::Gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Gauge<T>
```

Source: `src/metrics/instruments/gauge.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An instrument that records independent values

[`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) can be cloned to create multiple handles to the same instrument. If a [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) needs to be shared,
users are recommended to clone the [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) instead of creating duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric. Creating
duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric could lower SDK performance.

<a id="op-f8418952be15f3874d0c033d"></a>
## clone

`function` · `opentelemetry::metrics::instruments::gauge::Gauge::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Gauge<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "Gauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 10], "end": [12, 15], "filename": "src/metrics/instruments/gauge.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instruments/gauge.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-790409c705415e2188e1eafb"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::gauge::Gauge::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "Gauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [16, 1], "end": [23, 2], "filename": "src/metrics/instruments/gauge.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/gauge.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-19bc654b14c7b0432e4ec7d1"></a>
## new

`function` · `opentelemetry::metrics::instruments::gauge::Gauge::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(inner: Arc<dyn SyncInstrument<T> + Send + Sync>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "Gauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 1], "end": [35, 2], "filename": "src/metrics/instruments/gauge.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/gauge.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new gauge.

<a id="op-444c20528348cbdd428cda50"></a>
## record

`function` · `opentelemetry::metrics::instruments::gauge::Gauge::record` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, value: T, attributes: &[KeyValue])
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "Gauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 1], "end": [35, 2], "filename": "src/metrics/instruments/gauge.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/gauge.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Records an independent value.
