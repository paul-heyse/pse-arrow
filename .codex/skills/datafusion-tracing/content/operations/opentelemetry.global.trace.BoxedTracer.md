# `opentelemetry::global::trace::BoxedTracer`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.BoxedTracer.json).

<a id="op-716a226937c40d7602b7e516"></a>
## BoxedTracer

`struct` · `opentelemetry::global::trace::BoxedTracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BoxedTracer
```

Source: `src/global/trace.rs:242`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Wraps the [`GlobalTracerProvider`]'s [`Tracer`] so it can be used generically by
applications without knowing the underlying type.

[`Tracer`]: crate::trace::Tracer
[`GlobalTracerProvider`]: crate::global::GlobalTracerProvider

<a id="op-f326f26f3c4941176e045171"></a>
## Span

`assoc_type` · `opentelemetry::global::trace::BoxedTracer::Span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedTracer", "path": "BoxedTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [257, 1], "end": [266, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "Tracer"}, "trait_path": "opentelemetry::trace::tracer::Tracer"}`

Source: `src/global/trace.rs:260`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Global tracer uses `BoxedSpan`s so that it can be a global singleton,
which is not possible if it takes generic type parameters.

<a id="op-23680056db185a434aaba38f"></a>
## build_with_context

`function` · `opentelemetry::global::trace::BoxedTracer::build_with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_with_context(&self, builder: trace::SpanBuilder, parent_cx: &Context) -> Self::Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedTracer", "path": "BoxedTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [257, 1], "end": [266, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "Tracer"}, "trait_path": "opentelemetry::trace::tracer::Tracer"}`

Source: `src/global/trace.rs:263`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a span from a `SpanBuilder`

<a id="op-b35f6cbf7faf4cd856a11010"></a>
## fmt

`function` · `opentelemetry::global::trace::BoxedTracer::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedTracer", "path": "BoxedTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [255, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/global/trace.rs:252`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6179c5d3e2fc0c9826e1d764"></a>
## new

`function` · `opentelemetry::global::trace::BoxedTracer::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(tracer: Box<dyn ObjectSafeTracer + Send + Sync>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedTracer", "path": "BoxedTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [244, 1], "end": [249, 2], "filename": "src/global/trace.rs"}, "trait": null, "trait_path": null}`

Source: `src/global/trace.rs:246`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a `BoxedTracer` from an object-safe tracer.
