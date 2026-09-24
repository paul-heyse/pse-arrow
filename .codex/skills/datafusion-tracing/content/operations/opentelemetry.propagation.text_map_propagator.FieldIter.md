# `opentelemetry::propagation::text_map_propagator::FieldIter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.text_map_propagator.FieldIter.json).

<a id="op-ca07ce1552110226ee53c01b"></a>
## FieldIter

`struct` · `opentelemetry::propagation::text_map_propagator::FieldIter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct FieldIter<'a>
```

Source: `src/propagation/text_map_propagator.rs:63`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An iterator over fields of a [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3)


<a id="op-5c35e6985b644edf1cb699f9"></a>
## Item

`assoc_type` · `opentelemetry::propagation::text_map_propagator::FieldIter::Item` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::propagation::text_map_propagator::FieldIter", "path": "FieldIter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [77, 2], "filename": "src/propagation/text_map_propagator.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/propagation/text_map_propagator.rs:73`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c6a9964bbf2318d1539e3489"></a>
## fmt

`function` · `opentelemetry::propagation::text_map_propagator::FieldIter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::propagation::text_map_propagator::FieldIter", "path": "FieldIter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [62, 10], "end": [62, 15], "filename": "src/propagation/text_map_propagator.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/propagation/text_map_propagator.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a6d353049b3feacecbd4017b"></a>
## new

`function` · `opentelemetry::propagation::text_map_propagator::FieldIter::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(fields: &'a [String]) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::propagation::text_map_propagator::FieldIter", "path": "FieldIter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [65, 1], "end": [70, 2], "filename": "src/propagation/text_map_propagator.rs"}, "trait": null, "trait_path": null}`

Source: `src/propagation/text_map_propagator.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new `FieldIter` from a slice of propagator fields

<a id="op-54a6985731f44cb5ceffd048"></a>
## next

`function` · `opentelemetry::propagation::text_map_propagator::FieldIter::next` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::propagation::text_map_propagator::FieldIter", "path": "FieldIter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [77, 2], "filename": "src/propagation/text_map_propagator.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/propagation/text_map_propagator.rs:74`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
