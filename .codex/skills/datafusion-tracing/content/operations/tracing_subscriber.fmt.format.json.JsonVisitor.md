# `tracing_subscriber::fmt::format::json::JsonVisitor`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.json.JsonVisitor.json).

<a id="op-b9abaf7d22af065c198279c9"></a>
## JsonVisitor

`struct` · `tracing_subscriber::fmt::format::json::JsonVisitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct JsonVisitor<'a>
```

Source: `src/fmt/format/json.rs:426`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The [visitor] produced by [`JsonFields`](../operations/tracing_subscriber.fmt.format.json.JsonFields.md#op-6f888db73af63bd5c419003f)'s [`MakeVisitor`] implementation.

[visitor]: crate::field::Visit
[`MakeVisitor`]: crate::field::MakeVisitor

<a id="op-941bae6659f5735f35a364ba"></a>
## finish

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [458, 1], "end": [477, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": []}}, {"type": {"resolved_path": {"args": null, "id": "core::fmt::Error", "path": "Error"}}}], "constraints": []}}, "id": "core::result::Result", "path": "Result"}}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/fmt/format/json.rs:459`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-35153390d31f5b5e11bf5799"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [431, 1], "end": [435, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/json.rs:432`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8dc6ec64482df2052323b67b"></a>
## new

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(writer: &'a mut dyn Write) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [437, 1], "end": [450, 2], "filename": "src/fmt/format/json.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/json.rs:444`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new default visitor that formats to the provided `writer`.

# Arguments
- `writer`: the writer to format to.
- `is_empty`: whether or not any fields have been previously written to
  that writer.

<a id="op-307fa646f3ccdf2dfc4b872c"></a>
## record_bool

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_bool` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bool(&mut self, field: &Field, value: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:519`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a boolean value.

<a id="op-0c548757a9f1012d6dec5fbf"></a>
## record_bytes

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_bytes` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bytes(&mut self, field: &Field, value: &[u8])
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:530`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2d3b50422f925699ffe8d47d"></a>
## record_debug

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:535`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-636e2421f1ae202fec3ca4a4"></a>
## record_f64

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_f64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_f64(&mut self, field: &Field, value: f64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:501`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a double precision floating point value.

<a id="op-4e4cc300b3ab3fe8366654f7"></a>
## record_i64

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_i64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_i64(&mut self, field: &Field, value: i64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:507`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a signed 64-bit integer value.

<a id="op-c9bf3ed2184283bb94b450f8"></a>
## record_str

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:525`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a string value.

<a id="op-3a0e27ec64ecb792a0f921cd"></a>
## record_u64

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::record_u64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_u64(&mut self, field: &Field, value: u64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [479, 1], "end": [550, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/json.rs:513`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit an unsigned 64-bit integer value.

<a id="op-6177435e2a2a2ff4e9099850"></a>
## writer

`function` · `tracing_subscriber::fmt::format::json::JsonVisitor::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::json::JsonVisitor", "path": "JsonVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [452, 1], "end": [456, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/fmt/format/json.rs:453`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
