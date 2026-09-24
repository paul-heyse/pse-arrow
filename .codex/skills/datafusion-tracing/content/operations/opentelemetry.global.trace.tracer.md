# `opentelemetry::global::trace::tracer`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.tracer.json).

<a id="op-853638065d514e466d11aed4"></a>
## tracer

`function` · `opentelemetry::global::trace::tracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer(name: impl Into<std::borrow::Cow<'static, str>>) -> BoxedTracer
```

Source: `src/global/trace.rs:392`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a named instance of [`Tracer`] via the configured [`GlobalTracerProvider`](../operations/opentelemetry.global.trace.GlobalTracerProvider.md#op-41d9d444ae131d3115b9c6a7).

If the name is an empty string, the provider will use a default name.

This is a more convenient way of expressing `global::tracer_provider().tracer(name)`.

[`Tracer`]: crate::trace::Tracer
