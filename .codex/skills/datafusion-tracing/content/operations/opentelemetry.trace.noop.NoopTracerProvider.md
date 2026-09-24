# `opentelemetry::trace::noop::NoopTracerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.noop.NoopTracerProvider.json).

<a id="op-50eb0fc2bc83f7363e3a7743"></a>
## NoopTracerProvider

`struct` · `opentelemetry::trace::noop::NoopTracerProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct NoopTracerProvider
```

Source: `src/trace/noop.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A no-op instance of a `TracerProvider`.

<a id="op-9239c286b6d9f0afa92f9a90"></a>
## Tracer

`assoc_type` · `opentelemetry::trace::noop::NoopTracerProvider::Tracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracerProvider", "path": "NoopTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [33, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/trace/noop.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2f1b42940c636ad8d1ffafd7"></a>
## clone

`function` · `opentelemetry::trace::noop::NoopTracerProvider::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> NoopTracerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracerProvider", "path": "NoopTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 10], "end": [14, 15], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/noop.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b068960f37a7b8f25d88e27f"></a>
## default

`function` · `opentelemetry::trace::noop::NoopTracerProvider::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> NoopTracerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracerProvider", "path": "NoopTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 24], "end": [14, 31], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/noop.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9cd019582a5094353c7ac31d"></a>
## fmt

`function` · `opentelemetry::trace::noop::NoopTracerProvider::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracerProvider", "path": "NoopTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 17], "end": [14, 22], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/noop.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c2e72f49c1979bf374e877ec"></a>
## new

`function` · `opentelemetry::trace::noop::NoopTracerProvider::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracerProvider", "path": "NoopTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [19, 1], "end": [24, 2], "filename": "src/trace/noop.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/noop.rs:21`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new no-op tracer provider

<a id="op-3b230a43414e2e7d2a41abed"></a>
## tracer_with_scope

`function` · `opentelemetry::trace::noop::NoopTracerProvider::tracer_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer_with_scope(&self, _scope: InstrumentationScope) -> Self::Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracerProvider", "path": "NoopTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [33, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/trace/noop.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new `NoopTracer` instance.
