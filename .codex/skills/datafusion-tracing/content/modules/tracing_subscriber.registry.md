# `tracing_subscriber::registry`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.json).

<a id="op-40e91aae80c79f31a37c4ab2"></a>
## registry

`module` · `tracing_subscriber::registry` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
mod registry
```

Source: `src/registry/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Storage for span data shared by multiple [`Layer`]s.

## Using the Span Registry

This module provides the [`Registry`](../operations/tracing_subscriber.registry.sharded.Registry.md#op-4736e765871f1c85d718ade6) type, a [`Subscriber`] implementation
which tracks per-span data and exposes it to [`Layer`]s. When a `Registry`
is used as the base `Subscriber` of a `Layer` stack, the
[`layer::Context`][ctx] type will provide methods allowing `Layer`s to
[look up span data][lookup] stored in the registry. While [`Registry`](../operations/tracing_subscriber.registry.sharded.Registry.md#op-4736e765871f1c85d718ade6) is a
reasonable default for storing spans and events, other stores that implement
[`LookupSpan`](../operations/tracing_subscriber.registry.LookupSpan.md#op-d0bf19fd7852e1b6f08511d9) and [`Subscriber`] themselves (with [`SpanData`](../operations/tracing_subscriber.registry.SpanData.md#op-7713a4015fe2313bb70953e1) implemented
by the per-span data they store) can be used as a drop-in replacement.

For example, we might create a `Registry` and add multiple `Layer`s like so:
```rust
use tracing_subscriber::{registry::Registry, Layer, prelude::*};
# use tracing_core::Subscriber;
# pub struct FooLayer {}
# pub struct BarLayer {}
# impl<S: Subscriber> Layer<S> for FooLayer {}
# impl<S: Subscriber> Layer<S> for BarLayer {}
# impl FooLayer {
# fn new() -> Self { Self {} }
# }
# impl BarLayer {
# fn new() -> Self { Self {} }
# }

let subscriber = Registry::default()
    .with(FooLayer::new())
    .with(BarLayer::new());
```

If a type implementing `Layer` depends on the functionality of a `Registry`
implementation, it should bound its `Subscriber` type parameter with the
[`LookupSpan`](../operations/tracing_subscriber.registry.LookupSpan.md#op-d0bf19fd7852e1b6f08511d9) trait, like so:

```rust
use tracing_subscriber::{registry, Layer};
use tracing_core::Subscriber;

pub struct MyLayer {
    // ...
}

impl<S> Layer<S> for MyLayer
where
    S: Subscriber + for<'a> registry::LookupSpan<'a>,
{
    // ...
}
```
When this bound is added, the `Layer` implementation will be guaranteed
access to the [`Context`][ctx] methods, such as [`Context::span`][lookup], that
require the root subscriber to be a registry.

[`Layer`]: crate::layer::Layer
[`Subscriber`]: tracing_core::Subscriber
[ctx]: crate::layer::Context
[lookup]: crate::layer::Context::span()
