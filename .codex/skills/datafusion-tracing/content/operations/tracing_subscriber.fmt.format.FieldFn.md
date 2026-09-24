# `tracing_subscriber::fmt::format::FieldFn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.FieldFn.json).

<a id="op-fa83f8da1881e1fa3a738761"></a>
## FieldFn

`struct` · `tracing_subscriber::fmt::format::FieldFn` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FieldFn<F>
```

Source: `src/fmt/format/mod.rs:320`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation that formats fields by calling a function
or closure.


<a id="op-2b05e1a61c903ef1d3275d78"></a>
## Visitor

`assoc_type` · `tracing_subscriber::fmt::format::FieldFn::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFn", "path": "FieldFn"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": true, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "tracing_core::field::Field"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}]}}}}], "output": {"resolved_path": {"args": null, "id": "core::fmt::Result", "path": "fmt::Result"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1582, 1], "end": [1595, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/fmt/format/mod.rs:1586`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5244b380e58e7be657e1507"></a>
## clone

`function` · `tracing_subscriber::fmt::format::FieldFn::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> FieldFn<F>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFn", "path": "FieldFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [319, 17], "end": [319, 22], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/mod.rs:319`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5c469ea52c6997725152686a"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::FieldFn::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFn", "path": "FieldFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [319, 10], "end": [319, 15], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:319`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fd0b0ceed4a58a0075bfa8af"></a>
## make_visitor

`function` · `tracing_subscriber::fmt::format::FieldFn::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, writer: Writer<'a>) -> Self::Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFn", "path": "FieldFn"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": true, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "tracing_core::field::Field"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}]}}}}], "output": {"resolved_path": {"args": null, "id": "core::fmt::Result", "path": "fmt::Result"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1582, 1], "end": [1595, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/fmt/format/mod.rs:1588`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
