# `opentelemetry::global::trace::ObjectSafeTracer`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.ObjectSafeTracer.json).

<a id="op-ebd532b1879e2214fa3aa6af"></a>
## ObjectSafeTracer

`trait` · `opentelemetry::global::trace::ObjectSafeTracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait ObjectSafeTracer
```

Source: `src/global/trace.rs:272`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Allows a specific [`Tracer`] to be used generically by [`BoxedTracer`](../operations/opentelemetry.global.trace.BoxedTracer.md#op-716a226937c40d7602b7e516)
instances by mirroring the interface and boxing the return types.

[`Tracer`]: crate::trace::Tracer

<a id="op-405ec0f989ce9f2d0083cb77"></a>
## build_with_context_boxed

`function` · `opentelemetry::global::trace::ObjectSafeTracer::build_with_context_boxed` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_with_context_boxed(&self, builder: trace::SpanBuilder, parent_cx: &Context) -> Box<dyn ObjectSafeSpan + Send + Sync>
```

Source: `src/global/trace.rs:275`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a trait object so the underlying implementation can be swapped
out at runtime.
