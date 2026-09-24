# `opentelemetry::trace::noop::NoopTracer`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.noop.NoopTracer.json).

<a id="op-6a243c79a425778b9db10ee4"></a>
## NoopTracer

`struct` · `opentelemetry::trace::noop::NoopTracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct NoopTracer
```

Source: `src/trace/noop.rs:109`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A no-op instance of a `Tracer`.

<a id="op-1f2324b26b88b587a2259f08"></a>
## Span

`assoc_type` · `opentelemetry::trace::noop::NoopTracer::Span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "NoopTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [120, 1], "end": [132, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "Tracer"}, "trait_path": "opentelemetry::trace::tracer::Tracer"}`

Source: `src/trace/noop.rs:121`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3a0a9eac40c3265aa5c36094"></a>
## build_with_context

`function` · `opentelemetry::trace::noop::NoopTracer::build_with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_with_context(&self, _builder: trace::SpanBuilder, parent_cx: &Context) -> Self::Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "NoopTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [120, 1], "end": [132, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "Tracer"}, "trait_path": "opentelemetry::trace::tracer::Tracer"}`

Source: `src/trace/noop.rs:127`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Builds a `NoopSpan` from a `SpanBuilder`.

If the span builder or the context's current span contains a valid span context, it is
propagated.

<a id="op-cde52ca7b9ed6fd329bde27e"></a>
## clone

`function` · `opentelemetry::trace::noop::NoopTracer::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> NoopTracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "NoopTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [108, 10], "end": [108, 15], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/noop.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3c992697cad3f85595271d86"></a>
## default

`function` · `opentelemetry::trace::noop::NoopTracer::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> NoopTracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "NoopTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [108, 24], "end": [108, 31], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/noop.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b4cbe6b2754109c2ca6f3109"></a>
## fmt

`function` · `opentelemetry::trace::noop::NoopTracer::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "NoopTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [108, 17], "end": [108, 22], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/noop.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cafda8e5da5b31e8ef510ea3"></a>
## new

`function` · `opentelemetry::trace::noop::NoopTracer::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "NoopTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [113, 1], "end": [118, 2], "filename": "src/trace/noop.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/noop.rs:115`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new no-op tracer
