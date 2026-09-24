# `opentelemetry::global::trace::set_tracer_provider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.set_tracer_provider.json).

<a id="op-1838068eed0349336444df04"></a>
## set_tracer_provider

`function` · `opentelemetry::global::trace::set_tracer_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_tracer_provider<P, T, S>(new_provider: P) where S: trace::Span + Send + Sync + 'static, T: trace::Tracer<Span = S> + Send + Sync + 'static, P: trace::TracerProvider<Tracer = T> + Send + Sync + 'static
```

Source: `src/global/trace.rs:427`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the given [`TracerProvider`](../operations/opentelemetry.trace.tracer_provider.TracerProvider.md#op-3d14c743f48c20c76241bcf3) instance as the current global provider.

Libraries should NOT call this function. It is intended for applications/executables.
[`TracerProvider`](../operations/opentelemetry.trace.tracer_provider.TracerProvider.md#op-3d14c743f48c20c76241bcf3): crate::trace::TracerProvider
