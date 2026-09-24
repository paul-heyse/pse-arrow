# `opentelemetry::trace::noop::NoopTextMapPropagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.noop.NoopTextMapPropagator.json).

<a id="op-0812981bfaacffb18c746507"></a>
## NoopTextMapPropagator

`struct` · `opentelemetry::trace::noop::NoopTextMapPropagator` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct NoopTextMapPropagator
```

Source: `src/trace/noop.rs:138`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A no-op instance of an [`TextMapPropagator`].

[`TextMapPropagator`]: crate::propagation::TextMapPropagator

<a id="op-50c668c5aa251a925c1731c6"></a>
## default

`function` · `opentelemetry::trace::noop::NoopTextMapPropagator::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> NoopTextMapPropagator
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTextMapPropagator", "path": "NoopTextMapPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [137, 17], "end": [137, 24], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/noop.rs:137`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4767599b95a1f491ac295f31"></a>
## extract_with_context

`function` · `opentelemetry::trace::noop::NoopTextMapPropagator::extract_with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn extract_with_context(&self, _cx: &Context, _extractor: &dyn Extractor) -> Context
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTextMapPropagator", "path": "NoopTextMapPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [149, 1], "end": [161, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/trace/noop.rs:154`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2e42a4dde12d084156e7af45"></a>
## fields

`function` · `opentelemetry::trace::noop::NoopTextMapPropagator::fields` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> FieldIter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTextMapPropagator", "path": "NoopTextMapPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [149, 1], "end": [161, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/trace/noop.rs:158`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8ee0e6b6e83c6970995e39fa"></a>
## fmt

`function` · `opentelemetry::trace::noop::NoopTextMapPropagator::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTextMapPropagator", "path": "NoopTextMapPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [137, 10], "end": [137, 15], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/noop.rs:137`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-08b95903e1d4eecdbc8b105b"></a>
## inject_context

`function` · `opentelemetry::trace::noop::NoopTextMapPropagator::inject_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn inject_context(&self, _cx: &Context, _injector: &mut dyn Injector)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTextMapPropagator", "path": "NoopTextMapPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [149, 1], "end": [161, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/trace/noop.rs:150`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ba4708308b8bb96212b1a640"></a>
## new

`function` · `opentelemetry::trace::noop::NoopTextMapPropagator::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTextMapPropagator", "path": "NoopTextMapPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [142, 1], "end": [147, 2], "filename": "src/trace/noop.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/noop.rs:144`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new noop text map propagator
