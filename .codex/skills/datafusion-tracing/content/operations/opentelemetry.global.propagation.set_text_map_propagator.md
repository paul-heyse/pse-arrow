# `opentelemetry::global::propagation::set_text_map_propagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.propagation.set_text_map_propagator.json).

<a id="op-51d1784681b81283416b31c4"></a>
## set_text_map_propagator

`function` · `opentelemetry::global::propagation::set_text_map_propagator` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_text_map_propagator<P: TextMapPropagator + Send + Sync + 'static>(propagator: P)
```

Source: `src/global/propagation.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the given [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3) propagator as the current global propagator.
