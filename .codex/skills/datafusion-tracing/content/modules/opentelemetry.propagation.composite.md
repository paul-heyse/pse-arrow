# `opentelemetry::propagation::composite`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.composite.json).

<a id="op-091e622b1290349db710abb4"></a>
## composite

`module` · `opentelemetry::propagation::composite` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod composite
```

Source: `src/propagation/composite.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

# Composite Propagator

A utility over multiple `Propagator`s to group multiple Propagators from different cross-cutting
concerns in order to leverage them as a single entity.

Each composite Propagator will implement a specific Propagator type, such as TextMapPropagator,
as different Propagator types will likely operate on different data types.
