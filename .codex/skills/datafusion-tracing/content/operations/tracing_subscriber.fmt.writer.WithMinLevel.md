# `tracing_subscriber::fmt::writer::WithMinLevel`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.WithMinLevel.json).

<a id="op-323c2a156b0c08228a6a3518"></a>
## WithMinLevel

`struct` · `tracing_subscriber::fmt::writer::WithMinLevel` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct WithMinLevel<M>
```

Source: `src/fmt/writer.rs:599`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) combinator that only returns an enabled [writer] for spans
and events with metadata at or above a specified verbosity [`Level`].

This is returned by the [`MakeWriterExt::with_min_level`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-bd0cba079fd2dbd015c758ab) method. See the
method documentation for details.

[writer]: std::io::Write
[`Level`]: tracing_core::Level

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-b23678e17274b971c4f6ca11"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::WithMinLevel::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [977, 1], "end": [993, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:978`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-114668007a086149f91ed0a8"></a>
## clone

`function` · `tracing_subscriber::fmt::writer::WithMinLevel::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WithMinLevel<M>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 16], "end": [598, 21], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/writer.rs:598`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ebeddbff39fdfd69a66502e3"></a>
## eq

`function` · `tracing_subscriber::fmt::writer::WithMinLevel::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &WithMinLevel<M>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 34], "end": [598, 43], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/writer.rs:598`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-047374630f3a4ef65388d9f5"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::WithMinLevel::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 23], "end": [598, 28], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:598`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8e525dc995a1007ab4428e4f"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::WithMinLevel::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [977, 1], "end": [993, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:981`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-09a4656a83973d6af4939cd1"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::WithMinLevel::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [977, 1], "end": [993, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:987`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-623c653f40c94da725cd051a"></a>
## new

`function` · `tracing_subscriber::fmt::writer::WithMinLevel::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(make: M, level: tracing_core::Level) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMinLevel", "path": "WithMinLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [964, 1], "end": [975, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:972`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps the provided [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) with a minimum [`Level`], so that it
returns [`OptionalWriter::none`](../operations/tracing_subscriber.fmt.writer.EitherWriter.md#op-f6e00051b2c8a83963973ec7) for spans and events whose level is
less verbose than the maximum level.

See [`MakeWriterExt::with_min_level`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-bd0cba079fd2dbd015c758ab) for details.

[`Level`]: tracing_core::Level
