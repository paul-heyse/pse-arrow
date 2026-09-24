# `tracing_subscriber::fmt::writer::WithMaxLevel`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.WithMaxLevel.json).

<a id="op-9aad0d2e98029c8c0dcbf020"></a>
## WithMaxLevel

`struct` · `tracing_subscriber::fmt::writer::WithMaxLevel` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct WithMaxLevel<M>
```

Source: `src/fmt/writer.rs:585`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) combinator that only returns an enabled [writer] for spans
and events with metadata at or below a specified verbosity [`Level`].

This is returned by the [`MakeWriterExt::with_max_level`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-7991c17e378056a2039494a3) method. See the
method documentation for details.

[writer]: std::io::Write
[`Level`]: tracing_core::Level

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-c6ac39a2ad8729fbb282e8ba"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::WithMaxLevel::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [944, 1], "end": [960, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:945`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-19d9165f8686aed57e88f77e"></a>
## clone

`function` · `tracing_subscriber::fmt::writer::WithMaxLevel::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WithMaxLevel<M>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [584, 16], "end": [584, 21], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/writer.rs:584`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-926c353fe6ed16123d8199ef"></a>
## eq

`function` · `tracing_subscriber::fmt::writer::WithMaxLevel::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &WithMaxLevel<M>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [584, 34], "end": [584, 43], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/writer.rs:584`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-db12142862692206ad2b2206"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::WithMaxLevel::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [584, 23], "end": [584, 28], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:584`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-956f28f3822d4ef0fa1cce61"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::WithMaxLevel::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [944, 1], "end": [960, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:948`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8ab44bf964c98a1a3feaea7c"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::WithMaxLevel::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [944, 1], "end": [960, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:954`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-48f8dc8aef79ca2f8786b2e1"></a>
## new

`function` · `tracing_subscriber::fmt::writer::WithMaxLevel::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(make: M, level: tracing_core::Level) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithMaxLevel", "path": "WithMaxLevel"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [931, 1], "end": [942, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:939`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps the provided [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) with a maximum [`Level`], so that it
returns [`OptionalWriter::none`](../operations/tracing_subscriber.fmt.writer.EitherWriter.md#op-f6e00051b2c8a83963973ec7) for spans and events whose level is
more verbose than the maximum level.

See [`MakeWriterExt::with_max_level`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-7991c17e378056a2039494a3) for details.

[`Level`]: tracing_core::Level
