# `opentelemetry::global::propagation::get_text_map_propagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.propagation.get_text_map_propagator.json).

<a id="op-f4ebe756cafe0ba432e3fe4f"></a>
## get_text_map_propagator

`function` · `opentelemetry::global::propagation::get_text_map_propagator` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_text_map_propagator<T, F>(f: F) -> T where F: FnMut(&dyn TextMapPropagator) -> T
```

Source: `src/global/propagation.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Executes a closure with a reference to the current global [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3) propagator.
