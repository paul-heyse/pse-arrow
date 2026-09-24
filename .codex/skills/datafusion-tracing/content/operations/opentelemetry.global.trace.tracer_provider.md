# `opentelemetry::global::trace::tracer_provider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.tracer_provider.json).

<a id="op-9485a7842a9adee25f670ed8"></a>
## tracer_provider

`function` · `opentelemetry::global::trace::tracer_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer_provider() -> GlobalTracerProvider
```

Source: `src/global/trace.rs:374`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns an instance of the currently configured global [`TracerProvider`] through
[`GlobalTracerProvider`].

[`TracerProvider`]: crate::trace::TracerProvider
[`GlobalTracerProvider`]: crate::global::GlobalTracerProvider
