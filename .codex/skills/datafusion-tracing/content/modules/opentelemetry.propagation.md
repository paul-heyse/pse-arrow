# `opentelemetry::propagation`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.json).

<a id="op-ffc5d8b8e3baa059d19ad9a6"></a>
## propagation

`module` · `opentelemetry::propagation` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod propagation
```

Source: `src/propagation/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

# OpenTelemetry Propagator interface
Cross-cutting concerns send their state to the next process using Propagators, which are defined
as objects used to read and write context data to and from messages exchanged by the applications.

`Propagator`s leverage the [`Context`] to inject and extract data for each cross-cutting concern,
such as `TraceContext` and [`Baggage`].

The Propagators API is expected to be leveraged by users writing instrumentation libraries.

Currently, the following `Propagator` types are supported:
-  [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3), inject values into and extracts values from carriers as string key/value pairs

A binary Propagator type will be added in
the future, See [tracking issues](https://github.com/open-telemetry/opentelemetry-specification/issues/437)).

`Propagator`s uses [`Injector`](../operations/opentelemetry.propagation.Injector.md#op-6df55495bf9714237f4c3690) and [`Extractor`](../operations/opentelemetry.propagation.Extractor.md#op-295c86db9e9eeabe7894a847) to read and write context data to and from messages.
Each specific Propagator type defines its expected carrier type, such as a string map or a byte array.

[`Baggage`]: crate::baggage::Baggage
[`Context`]: crate::Context
