# `opentelemetry_sdk::trace::tracer::SdkTracer`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.tracer.SdkTracer.json).

<a id="op-994f3cb8ad5bc1ae17502b0e"></a>
## SdkTracer

`struct` · `opentelemetry_sdk::trace::tracer::SdkTracer` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkTracer
```

Source: `src/trace/tracer.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`Tracer` implementation to create and manage spans

<a id="op-7eadd1adbd53624654692de0"></a>
## Span

`assoc_type` · `opentelemetry_sdk::trace::tracer::SdkTracer::Span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::tracer::SdkTracer", "path": "SdkTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [297, 2], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "Tracer"}, "trait_path": "opentelemetry::trace::tracer::Tracer"}`

Source: `src/trace/tracer.rs:175`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

This implementation of `Tracer` produces `sdk::Span` instances.

<a id="op-c46dcb43e170a07b985fb2e2"></a>
## build_with_context

`function` · `opentelemetry_sdk::trace::tracer::SdkTracer::build_with_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_with_context(&self, builder: SpanBuilder, parent_cx: &Context) -> Self::Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::tracer::SdkTracer", "path": "SdkTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [297, 2], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "Tracer"}, "trait_path": "opentelemetry::trace::tracer::Tracer"}`

Source: `src/trace/tracer.rs:184`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Starts a span from a `SpanBuilder`.

Each span has zero or one parent spans and zero or more child spans, which
represent causally related operations. A tree of related spans comprises a
trace. A span is said to be a _root span_ if it does not have a parent. Each
trace includes a single root span, which is the shared ancestor of all other
spans in the trace.

<a id="op-ead258c44c8256b1b85cbb90"></a>
## clone

`function` · `opentelemetry_sdk::trace::tracer::SdkTracer::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SdkTracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::tracer::SdkTracer", "path": "SdkTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 10], "end": [25, 15], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/tracer.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7bf342d0a33e0c816cb9cc2"></a>
## fmt

`function` · `opentelemetry_sdk::trace::tracer::SdkTracer::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::tracer::SdkTracer", "path": "SdkTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [31, 1], "end": [40, 2], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/tracer.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Formats the `Tracer` using the given formatter.
Omitting `provider` here is necessary to avoid cycles.
