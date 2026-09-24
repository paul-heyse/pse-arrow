# `tracing_subscriber::fmt::writer::OrElse`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.OrElse.json).

<a id="op-bb3a38515ca89059d6d01ad8"></a>
## OrElse

`struct` · `tracing_subscriber::fmt::writer::OrElse` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct OrElse<A, B>
```

Source: `src/fmt/writer.rs:630`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines a [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that returns an [`OptionalWriter`](../operations/tracing_subscriber.fmt.writer.OptionalWriter.md#op-aca5124df0ea38cd4409ff35) with another
[`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a), so that the second [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) is used when the first
[`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) returns [`OptionalWriter::none`][own].

This is returned by the [`MakeWriterExt::or_else](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-321b77fb903fda6ba83d5fdd) method. See the
method documentation for details.

[own]: EitherWriter::none

<a id="op-1a4214f9ad39af7b1339347b"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::OrElse::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": [{"args": null, "binding": {"equality": {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OptionalWriter", "path": "OptionalWriter"}}}}, "name": "Writer"}]}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [1129, 1], "end": [1152, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1135`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-887a3a1011be039365c441d1"></a>
## clone

`function` · `tracing_subscriber::fmt::writer::OrElse::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> OrElse<A, B>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [629, 16], "end": [629, 21], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/writer.rs:629`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2e1377d64573dfac037748b5"></a>
## eq

`function` · `tracing_subscriber::fmt::writer::OrElse::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &OrElse<A, B>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [629, 34], "end": [629, 43], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/writer.rs:629`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-10345d5c77007b9a9d775394"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::OrElse::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [629, 23], "end": [629, 28], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:629`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-db64cd5b8dc575233c0bc5f2"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::OrElse::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": [{"args": null, "binding": {"equality": {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OptionalWriter", "path": "OptionalWriter"}}}}, "name": "Writer"}]}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [1129, 1], "end": [1152, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1138`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eb57e30d2981874e7a1b63b5"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::OrElse::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": [{"args": null, "binding": {"equality": {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OptionalWriter", "path": "OptionalWriter"}}}}, "name": "Writer"}]}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [1129, 1], "end": [1152, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1146`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a6c1352b664742342fef37bb"></a>
## new

`function` · `tracing_subscriber::fmt::writer::OrElse::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new<'a, W>(inner: A, or_else: B) -> Self where A: MakeWriter<'a, Writer = OptionalWriter<W>>, B: MakeWriter<'a>, W: Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::OrElse", "path": "OrElse"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1117, 1], "end": [1127, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:1119`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines
