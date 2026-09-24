# `tracing_subscriber::fmt::writer::WithFilter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.WithFilter.json).

<a id="op-82740b61d324165daa059d96"></a>
## WithFilter

`struct` · `tracing_subscriber::fmt::writer::WithFilter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct WithFilter<M, F>
```

Source: `src/fmt/writer.rs:616`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) combinator that wraps a [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) with a predicate for
span and event [`Metadata`], so that the [`MakeWriter::make_writer_for`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-f80df00cea31d415c78108e6)
method returns [`OptionalWriter::some`][ows] when the predicate returns `true`,
and [`OptionalWriter::none`][own] when the predicate returns `false`.

This is returned by the [`MakeWriterExt::with_filter`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-25389b1a363c79682d8c131a) method. See the
method documentation for details.

[`Metadata`]: tracing_core::Metadata
[ows]: EitherWriter::some
[own]: EitherWriter::none

<a id="op-a3f8c19131a838203b369e40"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::WithFilter::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "M"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1014, 1], "end": [1034, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1019`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d983c4f052b8fd3f2f027c45"></a>
## clone

`function` · `tracing_subscriber::fmt::writer::WithFilter::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WithFilter<M, F>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [615, 16], "end": [615, 21], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/writer.rs:615`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7c9d391df413cf345568cb95"></a>
## eq

`function` · `tracing_subscriber::fmt::writer::WithFilter::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &WithFilter<M, F>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [615, 34], "end": [615, 43], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/writer.rs:615`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6b038198361d531da513e7a0"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::WithFilter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [615, 23], "end": [615, 28], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:615`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7922beae980e73a8ea643cc1"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::WithFilter::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "M"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1014, 1], "end": [1034, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1022`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-421674abb1f9e3848e457d37"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::WithFilter::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "M"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [1014, 1], "end": [1034, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1027`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0000ef79db8369deb91f853a"></a>
## new

`function` · `tracing_subscriber::fmt::writer::WithFilter::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(make: M, filter: F) -> Self where F: Fn(&Metadata<'_>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::WithFilter", "path": "WithFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [997, 1], "end": [1012, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:1006`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `make` with the provided `filter`, returning a [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that
will call `make.make_writer_for()` when `filter` returns `true` for a
span or event's [`Metadata`], and returns a [`sink`] otherwise.

See [`MakeWriterExt::with_filter`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-25389b1a363c79682d8c131a) for details.

[`Metadata`]: tracing_core::Metadata
[`sink`]: std::io::sink

Unresolved upstream links (retained, not inferred): `std::io::sink`.
