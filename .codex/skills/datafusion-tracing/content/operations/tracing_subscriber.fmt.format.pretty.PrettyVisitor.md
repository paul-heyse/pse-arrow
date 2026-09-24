# `tracing_subscriber::fmt::format::pretty::PrettyVisitor`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.pretty.PrettyVisitor.json).

<a id="op-e47500c85ef8cca53e8eb752"></a>
## PrettyVisitor

`struct` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct PrettyVisitor<'a>
```

Source: `src/fmt/format/pretty.rs:108`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The [visitor] produced by [`Pretty`](../operations/tracing_subscriber.fmt.format.pretty.Pretty.md#op-9ae7afec101d61be119e29f4)'s [`MakeVisitor`] implementation.

[visitor]: field::Visit
[`MakeVisitor`]: crate::field::MakeVisitor

<a id="op-3e77b9985f0829844b57e792"></a>
## finish

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [511, 1], "end": [516, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": []}}, {"type": {"resolved_path": {"args": null, "id": "core::fmt::Error", "path": "Error"}}}], "constraints": []}}, "id": "core::result::Result", "path": "Result"}}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/fmt/format/pretty.rs:512`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0cad3ed535cfd04da6bf6a99"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 10], "end": [107, 15], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/pretty.rs:107`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8d68fc2c6feb04a9916a6ad9"></a>
## new

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(writer: Writer<'a>, is_empty: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [401, 1], "end": [438, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/pretty.rs:408`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new default visitor that formats to the provided `writer`.

# Arguments
- `writer`: the writer to format to.
- `is_empty`: whether or not any fields have been previously written to
  that writer.

<a id="op-f0507b93f44ff4d40201eba7"></a>
## record_debug

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 1], "end": [509, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/pretty.rs:476`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-428bbc37345233dbbba35764"></a>
## record_error

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::record_error` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_error(&mut self, field: &Field, value: &dyn std::error::Error + 'static)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 1], "end": [509, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/pretty.rs:453`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a4b29502556934808df313df"></a>
## record_str

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::record_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 1], "end": [509, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/fmt/format/pretty.rs:441`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8375b958d4826ed2fe57752a"></a>
## writer

`function` · `tracing_subscriber::fmt::format::pretty::PrettyVisitor::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::pretty::PrettyVisitor", "path": "PrettyVisitor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [518, 1], "end": [522, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/fmt/format/pretty.rs:519`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
