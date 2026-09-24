# `tracing_subscriber::fmt::format::json::JsonFields`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.json.JsonFields.json).

<a id="op-6f888db73af63bd5c419003f"></a>
## JsonFields

`struct` · `tracing_subscriber::fmt::format::json::JsonFields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct JsonFields
```

Source: `src/fmt/format/json.rs:347`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The JSON [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation.


<a id="op-fee77778a6f6454720af8f37"></a>
## add_fields

`function` · `tracing_subscriber::fmt::format::json::JsonFields::add_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn add_fields(&self, current: &'a mut FormattedFields<Self>, fields: &Record<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "JsonFields"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [367, 1], "end": [420, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}, "trait_path": "tracing_subscriber::fmt::format::FormatFields"}`

Source: `src/fmt/format/json.rs:380`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Record additional field(s) on an existing span.

By default, this appends a space to the current set of fields if it is
non-empty, and then calls `self.format_fields`. If different behavior is
required, the default implementation of this method can be overridden.

<a id="op-1838854144a0b8f1858121b9"></a>
## default

`function` · `tracing_subscriber::fmt::format::json::JsonFields::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "JsonFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [365, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/json.rs:362`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5c68d374bff5734ff06f412"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::json::JsonFields::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "JsonFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [346, 10], "end": [346, 15], "filename": "src/fmt/format/json.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/json.rs:346`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f738927522810635aa5277e2"></a>
## format_fields

`function` · `tracing_subscriber::fmt::format::json::JsonFields::format_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_fields<R: RecordFields>(&self, writer: Writer<'_>, fields: R) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "JsonFields"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [367, 1], "end": [420, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}, "trait_path": "tracing_subscriber::fmt::format::FormatFields"}`

Source: `src/fmt/format/json.rs:369`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Format the provided `fields` to the provided `writer`, returning a result.

<a id="op-fa85e0c8143b450196f41b90"></a>
## new

`function` · `tracing_subscriber::fmt::format::json::JsonFields::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "JsonFields"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [353, 1], "end": [359, 2], "filename": "src/fmt/format/json.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/json.rs:356`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new JSON [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation.

