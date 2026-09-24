# `tracing_subscriber::fmt::fmt_layer::FormattedFields`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.fmt_layer.FormattedFields.json).

<a id="op-497c6687a48692d81dd34ffa"></a>
## FormattedFields

`struct` · `tracing_subscriber::fmt::fmt_layer::FormattedFields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FormattedFields<E: ?Sized>
```

Source: `src/fmt/fmt_layer.rs:784`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A formatted representation of a span's fields stored in its [extensions].

Because `FormattedFields` is generic over the type of the formatter that
produced it, multiple versions of a span's formatted fields can be stored in
the [`Extensions`][extensions] type-map. This means that when multiple
formatters are in use, each can store its own formatted representation
without conflicting.

[extensions]: crate::registry::Extensions

<a id="op-15badb4ac6b70204a1eeb4dd"></a>
## Target

`assoc_type` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::Target` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Target
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [842, 1], "end": [847, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::ops::deref::Deref", "path": "Deref"}, "trait_path": "core::ops::deref::Deref"}`

Source: `src/fmt/fmt_layer.rs:843`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2265bf0519c44e03fe1ceb07"></a>
## as_writer

`function` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::as_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn as_writer(&mut self) -> format::Writer<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [803, 1], "end": [823, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:818`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`format::Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) for writing to this `FormattedFields`.

The returned [`format::Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) can be used with the
[`FormatFields::format_fields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-01424b9dbd594952e76f0f8d) method.

<a id="op-021de46b600edbfdcefb4762"></a>
## default

`function` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [792, 1], "end": [801, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/fmt_layer.rs:793`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a8785c7d2161a05800b8bf7f"></a>
## deref

`function` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::deref` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn deref(&self) -> &Self::Target
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [842, 1], "end": [847, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::ops::deref::Deref", "path": "Deref"}, "trait_path": "core::ops::deref::Deref"}`

Source: `src/fmt/fmt_layer.rs:844`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-78948bc938fa11857c639c13"></a>
## fields

`struct_field` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fields: alloc::string::String
```

Source: `src/fmt/fmt_layer.rs:789`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The formatted fields of a span.

<a id="op-0c05b28af3a212fbee7f3ae4"></a>
## fmt

`function` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [825, 1], "end": [834, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/fmt_layer.rs:826`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8288e77de3a2ed34d316930a"></a>
## fmt

`function` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [836, 1], "end": [840, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/fmt/fmt_layer.rs:837`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-898fdf736bfe1377b5780e30"></a>
## new

`function` · `tracing_subscriber::fmt::fmt_layer::FormattedFields::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(fields: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FormattedFields", "path": "FormattedFields"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "maybe", "trait": {"args": null, "id": "core::marker::Sized", "path": "Sized"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [803, 1], "end": [823, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:805`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `FormattedFields`.
