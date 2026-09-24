# `opentelemetry::global::trace::ObjectSafeTracerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.ObjectSafeTracerProvider.json).

<a id="op-c911618885e7a9d7e731865f"></a>
## ObjectSafeTracerProvider

`trait` · `opentelemetry::global::trace::ObjectSafeTracerProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait ObjectSafeTracerProvider
```

Source: `src/global/trace.rs:303`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Allows a specific [`TracerProvider`] to be used generically by the
[`GlobalTracerProvider`] by mirroring the interface and boxing the return types.

[`TracerProvider`]: crate::trace::TracerProvider
[`GlobalTracerProvider`]: crate::global::GlobalTracerProvider

<a id="op-d1bd3a1623d32f6df4fd65ea"></a>
## boxed_tracer

`function` · `opentelemetry::global::trace::ObjectSafeTracerProvider::boxed_tracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn boxed_tracer(&self, scope: InstrumentationScope) -> Box<dyn ObjectSafeTracer + Send + Sync>
```

Source: `src/global/trace.rs:306`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a versioned named tracer instance that is a trait object through the underlying
`TracerProvider`.
