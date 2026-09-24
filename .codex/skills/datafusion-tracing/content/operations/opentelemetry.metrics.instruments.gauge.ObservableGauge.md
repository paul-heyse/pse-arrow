# `opentelemetry::metrics::instruments::gauge::ObservableGauge`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.gauge.ObservableGauge.json).

<a id="op-6bc1f97a963a42f9d0fd5794"></a>
## ObservableGauge

`struct` · `opentelemetry::metrics::instruments::gauge::ObservableGauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ObservableGauge<T>
```

Source: `src/metrics/instruments/gauge.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An async instrument that records independent readings.

<a id="op-25d96443c457182173cc3e16"></a>
## clone

`function` · `opentelemetry::metrics::instruments::gauge::ObservableGauge::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ObservableGauge<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::ObservableGauge", "path": "ObservableGauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [38, 10], "end": [38, 15], "filename": "src/metrics/instruments/gauge.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instruments/gauge.rs:38`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c9f861a356f94ec82035506f"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::gauge::ObservableGauge::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::ObservableGauge", "path": "ObservableGauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [44, 1], "end": [54, 2], "filename": "src/metrics/instruments/gauge.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/gauge.rs:48`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6766a35938a5c2c1d873daf2"></a>
## new

`function` · `opentelemetry::metrics::instruments::gauge::ObservableGauge::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::ObservableGauge", "path": "ObservableGauge"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [56, 1], "end": [64, 2], "filename": "src/metrics/instruments/gauge.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/gauge.rs:59`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new gauge
