# `tracing_subscriber::fmt::format::FieldFnVisitor`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.FieldFnVisitor.json).

<a id="op-40092ff9b473f89ceaf123dd"></a>
## FieldFnVisitor

`struct` · `tracing_subscriber::fmt::format::FieldFnVisitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FieldFnVisitor<'a, F>
```

Source: `src/fmt/format/mod.rs:325`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The [visitor] produced by [`FieldFn`](../operations/tracing_subscriber.fmt.format.FieldFn.md#op-fa83f8da1881e1fa3a738761)'s [`MakeVisitor`] implementation.

[visitor]: super::super::field::Visit
[`MakeVisitor`]: super::super::field::MakeVisitor

<a id="op-d3993a29a3d8d8b7afacf3ab"></a>
## finish

`function` · `tracing_subscriber::fmt::format::FieldFnVisitor::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFnVisitor", "path": "FieldFnVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": true, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "tracing_core::field::Field"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}]}}}}], "output": {"resolved_path": {"args": null, "id": "core::fmt::Result", "path": "fmt::Result"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1608, 1], "end": [1615, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": []}}, {"type": {"resolved_path": {"args": null, "id": "core::fmt::Error", "path": "Error"}}}], "constraints": []}}, "id": "core::result::Result", "path": "Result"}}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/fmt/format/mod.rs:1612`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-452db4c9c9dee355fb4366f3"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::FieldFnVisitor::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFnVisitor", "path": "FieldFnVisitor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1626, 1], "end": [1634, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:1627`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-51208e4f36b95c437135a106"></a>
## record_debug

`function` · `tracing_subscriber::fmt::format::FieldFnVisitor::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFnVisitor", "path": "FieldFnVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": true, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "tracing_core::field::Field"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}]}}}}], "output": {"resolved_path": {"args": null, "id": "core::fmt::Result", "path": "fmt::Result"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1597, 1], "end": [1606, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/mod.rs:1601`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f08567c96553079faa2719b2"></a>
## writer

`function` · `tracing_subscriber::fmt::format::FieldFnVisitor::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FieldFnVisitor", "path": "FieldFnVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": true, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "tracing_core::field::Field"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}]}}}}], "output": {"resolved_path": {"args": null, "id": "core::fmt::Result", "path": "fmt::Result"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1617, 1], "end": [1624, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/fmt/format/mod.rs:1621`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
