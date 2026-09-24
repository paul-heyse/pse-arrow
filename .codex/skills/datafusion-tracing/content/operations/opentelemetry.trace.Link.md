# `opentelemetry::trace::Link`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.Link.json).

<a id="op-412ecb298a74ff9ab354c6ff"></a>
## Link

`struct` · `opentelemetry::trace::Link` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Link
```

Source: `src/trace/mod.rs:240`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Link is the relationship between two Spans.

The relationship can be within the same trace or across different traces.

<a id="op-2f3571e12c8997903ab8eb43"></a>
## attributes

`struct_field` · `opentelemetry::trace::Link::attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
attributes: Vec<KeyValue>
```

Source: `src/trace/mod.rs:245`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Attributes that describe this link.

<a id="op-dc149e1239a75d8021692d87"></a>
## clone

`function` · `opentelemetry::trace::Link::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Link
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Link", "path": "Link"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 10], "end": [239, 15], "filename": "src/trace/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/mod.rs:239`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bcbf8b14fdb9ebfdfd271884"></a>
## dropped_attributes_count

`struct_field` · `opentelemetry::trace::Link::dropped_attributes_count` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
dropped_attributes_count: u32
```

Source: `src/trace/mod.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The number of attributes that were above the configured limit, and thus
dropped.

<a id="op-6cad367eead719b652c0e2e8"></a>
## eq

`function` · `opentelemetry::trace::Link::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Link) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Link", "path": "Link"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 24], "end": [239, 33], "filename": "src/trace/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/mod.rs:239`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9d7c0e516aedd7a261b5e3a6"></a>
## fmt

`function` · `opentelemetry::trace::Link::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Link", "path": "Link"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 17], "end": [239, 22], "filename": "src/trace/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/mod.rs:239`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d36d595d2cc96acb15b3b96"></a>
## new

`function` · `opentelemetry::trace::Link::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(span_context: SpanContext, attributes: Vec<KeyValue>, dropped_attributes_count: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Link", "path": "Link"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [252, 1], "end": [274, 2], "filename": "src/trace/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/mod.rs:254`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create new `Link`

<a id="op-cf9ec965a36b6b45ba48b203"></a>
## span_context

`struct_field` · `opentelemetry::trace::Link::span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_context: SpanContext
```

Source: `src/trace/mod.rs:242`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The span context of the linked span.

<a id="op-e1abaf71ac4f7b92695655c8"></a>
## with_context

`function` · `opentelemetry::trace::Link::with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_context(span_context: SpanContext) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Link", "path": "Link"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [252, 1], "end": [274, 2], "filename": "src/trace/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/mod.rs:267`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create new `Link` with given context
