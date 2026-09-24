# `tracing_subscriber::field::delimited::Delimited`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.delimited.Delimited.json).

<a id="op-634950aa3198fa3bbdda3f26"></a>
## Delimited

`struct` · `tracing_subscriber::field::delimited::Delimited` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Delimited<D, V>
```

Source: `src/field/delimited.rs:10`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A `MakeVisitor` wrapper that wraps a visitor that writes formatted output so
that a delimiter is inserted between writing formatted field values.

<a id="op-8fc1453f3a24be74c94cb6a6"></a>
## Visitor

`assoc_type` · `tracing_subscriber::field::delimited::Delimited::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::Delimited", "path": "Delimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "D"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Visitor", "self_type": {"generic": "V"}, "trait": {"args": null, "id": "tracing_subscriber::field::MakeVisitor", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [27, 1], "end": [38, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/field/delimited.rs:33`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eb6bbe2573d077bec13c817a"></a>
## clone

`function` · `tracing_subscriber::field::delimited::Delimited::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Delimited<D, V>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::Delimited", "path": "Delimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 17], "end": [9, 22], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/field/delimited.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eb81017f6367901dbb7298d0"></a>
## fmt

`function` · `tracing_subscriber::field::delimited::Delimited::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::Delimited", "path": "Delimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 10], "end": [9, 15], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field/delimited.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f56d96e4c9fea058f70d735c"></a>
## make_visitor

`function` · `tracing_subscriber::field::delimited::Delimited::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, target: T) -> Self::Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::Delimited", "path": "Delimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "D"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Visitor", "self_type": {"generic": "V"}, "trait": {"args": null, "id": "tracing_subscriber::field::MakeVisitor", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [27, 1], "end": [38, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/field/delimited.rs:34`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cb291162abc822835c98b3f7"></a>
## new

`function` · `tracing_subscriber::field::delimited::Delimited::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(delimiter: D, inner: V) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::Delimited", "path": "Delimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [40, 1], "end": [48, 2], "filename": "src/field/delimited.rs"}, "trait": null, "trait_path": null}`

Source: `src/field/delimited.rs:45`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`MakeVisitor`] implementation that wraps `inner` so that
it will format each visited field separated by the provided `delimiter`.

[`MakeVisitor`]: super::MakeVisitor
