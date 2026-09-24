# `tracing_subscriber::fmt::format::DefaultVisitor`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.DefaultVisitor.json).

<a id="op-86dc315fc25bdd551b27ca6a"></a>
## DefaultVisitor

`struct` · `tracing_subscriber::fmt::format::DefaultVisitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct DefaultVisitor<'a>
```

Source: `src/fmt/format/mod.rs:1209`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The [visitor] produced by [`DefaultFields`](../operations/tracing_subscriber.fmt.format.DefaultFields.md#op-2ac3921259826db0e8bdb02e)'s [`MakeVisitor`] implementation.

[visitor]: super::super::field::Visit
[`MakeVisitor`]: super::super::field::MakeVisitor

<a id="op-d9a86b0f9e4a016497ef7d72"></a>
## finish

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1343, 1], "end": [1347, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": []}}, {"type": {"resolved_path": {"args": null, "id": "core::fmt::Error", "path": "Error"}}}], "constraints": []}}, "id": "core::result::Result", "path": "Result"}}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/fmt/format/mod.rs:1344`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8b264d2b3f124d1ba011ce10"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1208, 10], "end": [1208, 15], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:1208`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ed386d4c1fec3398bbab533f"></a>
## new

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(writer: Writer<'a>, is_empty: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1239, 1], "end": [1261, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1246`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new default visitor that formats to the provided `writer`.

# Arguments
- `writer`: the writer to format to.
- `is_empty`: whether or not any fields have been previously written to
  that writer.

<a id="op-b6ba790ea879bf2de5e87b6c"></a>
## record_debug

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1263, 1], "end": [1341, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/mod.rs:1299`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4dd4ea1b6c34311e45b25793"></a>
## record_error

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::record_error` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_error(&mut self, field: &Field, value: &dyn std::error::Error + 'static)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1263, 1], "end": [1341, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/mod.rs:1276`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-75ecc9084b923945336e7853"></a>
## record_str

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::record_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1263, 1], "end": [1341, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/mod.rs:1264`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-99e94c7b079e2a2c45e4a252"></a>
## writer

`function` · `tracing_subscriber::fmt::format::DefaultVisitor::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::DefaultVisitor", "path": "DefaultVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1349, 1], "end": [1353, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/fmt/format/mod.rs:1350`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
