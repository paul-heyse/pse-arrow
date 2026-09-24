# `opentelemetry::baggage::Iter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.baggage.Iter.json).

<a id="op-5d476740bc0d38d40dfd0362"></a>
## Iter

`struct` · `opentelemetry::baggage::Iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Iter<'a>
```

Source: `src/baggage.rs:244`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An iterator over the entries of a [`Baggage`](../operations/opentelemetry.baggage.Baggage.md#op-2c88c4516fa3cd4da1db039e).

<a id="op-24e35fc23d124c9f8a7c254a"></a>
## Item

`assoc_type` · `opentelemetry::baggage::Iter::Item` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::baggage::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [246, 1], "end": [252, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/baggage.rs:247`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8dff3ac87a980a609dae78d9"></a>
## fmt

`function` · `opentelemetry::baggage::Iter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::baggage::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 10], "end": [243, 15], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/baggage.rs:243`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b6ec41c14b46c0efdf55042"></a>
## next

`function` · `opentelemetry::baggage::Iter::next` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::baggage::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [246, 1], "end": [252, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/baggage.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
