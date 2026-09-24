# `tracing_subscriber::fmt::format::pretty::PrettyFields`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.pretty.PrettyFields.json).

<a id="op-19cd2f5d02f60844d4df2e26"></a>
## PrettyFields

`struct` · `tracing_subscriber::fmt::format::pretty::PrettyFields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct PrettyFields
```

Source: `src/fmt/format/pretty.rs:119`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An excessively pretty, human-readable [`MakeVisitor`] implementation.

[`MakeVisitor`]: crate::field::MakeVisitor

<a id="op-587d1e2552c2fbf251e65849"></a>
## Visitor

`assoc_type` · `tracing_subscriber::fmt::format::pretty::PrettyFields::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::PrettyFields", "path": "PrettyFields"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [387, 1], "end": [397, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/fmt/format/pretty.rs:388`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cd2b4a3f56d613974292c9d0"></a>
## default

`function` · `tracing_subscriber::fmt::format::pretty::PrettyFields::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::PrettyFields", "path": "PrettyFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [359, 1], "end": [363, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/pretty.rs:360`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-01e250e98f8fc4a3b51e8877"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::pretty::PrettyFields::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::PrettyFields", "path": "PrettyFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [118, 10], "end": [118, 15], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/pretty.rs:118`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8e53cf2d7c2cf03c1127c3f0"></a>
## make_visitor

`function` · `tracing_subscriber::fmt::format::pretty::PrettyFields::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::PrettyFields", "path": "PrettyFields"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [387, 1], "end": [397, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/fmt/format/pretty.rs:391`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9e8a0af3f19fe1047c558495"></a>
## new

`function` · `tracing_subscriber::fmt::format::pretty::PrettyFields::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::PrettyFields", "path": "PrettyFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [365, 1], "end": [385, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/pretty.rs:367`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new default [`PrettyFields`](../operations/tracing_subscriber.fmt.format.pretty.PrettyFields.md#op-19cd2f5d02f60844d4df2e26) implementation.

<a id="op-639e0b673f3588853e92e1e6"></a>
## with_ansi

`function` · `tracing_subscriber::fmt::format::pretty::PrettyFields::with_ansi` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_ansi(self, ansi: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::PrettyFields", "path": "PrettyFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [365, 1], "end": [385, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/pretty.rs:379`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Enable ANSI encoding for formatted fields.
