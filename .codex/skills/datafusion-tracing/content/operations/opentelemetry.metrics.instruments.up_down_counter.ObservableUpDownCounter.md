# `opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.up_down_counter.ObservableUpDownCounter.json).

<a id="op-7383c55f2c5c31581d4ac017"></a>
## ObservableUpDownCounter

`struct` · `opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ObservableUpDownCounter<T>
```

Source: `src/metrics/instruments/up_down_counter.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An async instrument that records increasing or decreasing values.

<a id="op-89824ac9cb7e7261cd36d236"></a>
## clone

`function` · `opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ObservableUpDownCounter<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter", "path": "ObservableUpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 10], "end": [41, 15], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instruments/up_down_counter.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-74b60f7dee2536bc3577b919"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter", "path": "ObservableUpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [47, 1], "end": [57, 2], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/up_down_counter.rs:51`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3ae18f28b151f2a0b819ecbc"></a>
## new

`function` · `opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter", "path": "ObservableUpDownCounter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [59, 1], "end": [67, 2], "filename": "src/metrics/instruments/up_down_counter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/up_down_counter.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new observable up down counter.
