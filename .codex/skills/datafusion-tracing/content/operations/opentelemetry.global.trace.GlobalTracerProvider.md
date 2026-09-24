# `opentelemetry::global::trace::GlobalTracerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.GlobalTracerProvider.json).

<a id="op-41d9d444ae131d3115b9c6a7"></a>
## GlobalTracerProvider

`struct` · `opentelemetry::global::trace::GlobalTracerProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct GlobalTracerProvider
```

Source: `src/global/trace.rs:327`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Represents the globally configured [`TracerProvider`] instance for this
application. This allows generic tracing through the returned
[`BoxedTracer`](../operations/opentelemetry.global.trace.BoxedTracer.md#op-716a226937c40d7602b7e516) instances.

[`TracerProvider`]: crate::trace::TracerProvider

<a id="op-4d0643839766f54b06b41716"></a>
## Tracer

`assoc_type` · `opentelemetry::global::trace::GlobalTracerProvider::Tracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::GlobalTracerProvider", "path": "GlobalTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [351, 1], "end": [358, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/global/trace.rs:352`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-089296d4aaced3551fdb8adc"></a>
## clone

`function` · `opentelemetry::global::trace::GlobalTracerProvider::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> GlobalTracerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::GlobalTracerProvider", "path": "GlobalTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [326, 10], "end": [326, 15], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/global/trace.rs:326`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c163a3afb5afaa969cf2b5c8"></a>
## fmt

`function` · `opentelemetry::global::trace::GlobalTracerProvider::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::GlobalTracerProvider", "path": "GlobalTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [331, 1], "end": [335, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/global/trace.rs:332`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2c7a555142ffcc96ea90ab01"></a>
## tracer_with_scope

`function` · `opentelemetry::global::trace::GlobalTracerProvider::tracer_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer_with_scope(&self, scope: InstrumentationScope) -> Self::Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::GlobalTracerProvider", "path": "GlobalTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [351, 1], "end": [358, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/global/trace.rs:355`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a tracer using the global provider.
