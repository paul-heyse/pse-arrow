# `opentelemetry::context::future_ext::FutureExt`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.context.future_ext.FutureExt.json).

<a id="op-0b23d57a653ef8543662473d"></a>
## FutureExt

`trait` · `opentelemetry::context::future_ext::FutureExt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait FutureExt: Sized
```

Source: `src/context/future_ext.rs:82`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Extension trait allowing futures, streams, and sinks to be traced with a span.

<a id="op-af6a5cf2dd68236aab2f9f16"></a>
## with_context

`function` · `opentelemetry::context::future_ext::FutureExt::with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_context(self, otel_cx: Context) -> WithContext<Self>
```

Source: `src/context/future_ext.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Attaches the provided [`Context`] to this type, returning a `WithContext`
wrapper.

When the wrapped type is a future, stream, or sink, the attached context
will be set as current while it is being polled.

[`Context`]: Context

<a id="op-c53e011b5936b59d0a29fc21"></a>
## with_current_context

`function` · `opentelemetry::context::future_ext::FutureExt::with_current_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_context(self) -> WithContext<Self>
```

Source: `src/context/future_ext.rs:104`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Attaches the current [`Context`] to this type, returning a `WithContext`
wrapper.

When the wrapped type is a future, stream, or sink, the attached context
will be set as the default while it is being polled.

[`Context`]: Context
