# `tracing_subscriber::fmt::format::DefaultFields`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.DefaultFields.json).

<a id="op-2ac3921259826db0e8bdb02e"></a>
## DefaultFields

`struct` · `tracing_subscriber::fmt::format::DefaultFields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct DefaultFields
```

Source: `src/fmt/format/mod.rs:1198`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The default [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation.


<a id="op-bad59b80f301199e65e14bbc"></a>
## Visitor

`assoc_type` · `tracing_subscriber::fmt::format::DefaultFields::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::DefaultFields", "path": "DefaultFields"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1228, 1], "end": [1235, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/fmt/format/mod.rs:1229`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dbd24afd007711be1614be0e"></a>
## default

`function` · `tracing_subscriber::fmt::format::DefaultFields::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::DefaultFields", "path": "DefaultFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1222, 1], "end": [1226, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/mod.rs:1223`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-13d65973a6109ba44ef3f63d"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::DefaultFields::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::DefaultFields", "path": "DefaultFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1197, 10], "end": [1197, 15], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:1197`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b3e248b4dc3407d400ab4af0"></a>
## make_visitor

`function` · `tracing_subscriber::fmt::format::DefaultFields::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::DefaultFields", "path": "DefaultFields"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1228, 1], "end": [1235, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/fmt/format/mod.rs:1232`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5a522ec180296ff0c5fb675d"></a>
## new

`function` · `tracing_subscriber::fmt::format::DefaultFields::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::DefaultFields", "path": "DefaultFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1215, 1], "end": [1220, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1217`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new default [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation.
