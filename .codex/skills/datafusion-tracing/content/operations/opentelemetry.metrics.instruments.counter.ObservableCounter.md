# `opentelemetry::metrics::instruments::counter::ObservableCounter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.counter.ObservableCounter.json).

<a id="op-3a7ca43ee61854d86479ff85"></a>
## ObservableCounter

`struct` · `opentelemetry::metrics::instruments::counter::ObservableCounter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ObservableCounter<T>
```

Source: `src/metrics/instruments/counter.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An async instrument that records increasing values.

<a id="op-194b30dbe749351c5f7d7daf"></a>
## clone

`function` · `opentelemetry::metrics::instruments::counter::ObservableCounter::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ObservableCounter<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::ObservableCounter", "path": "ObservableCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [38, 10], "end": [38, 15], "filename": "src/metrics/instruments/counter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instruments/counter.rs:38`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2c4e2efd83e3812bc3f7a661"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::counter::ObservableCounter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::ObservableCounter", "path": "ObservableCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [61, 2], "filename": "src/metrics/instruments/counter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/counter.rs:55`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4d2290d18ba62d32dcc8326f"></a>
## new

`function` · `opentelemetry::metrics::instruments::counter::ObservableCounter::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::ObservableCounter", "path": "ObservableCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [44, 1], "end": [52, 2], "filename": "src/metrics/instruments/counter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/counter.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new observable counter.
