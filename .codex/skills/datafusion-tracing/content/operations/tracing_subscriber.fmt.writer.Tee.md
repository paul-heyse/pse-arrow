# `tracing_subscriber::fmt::writer::Tee`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.Tee.json).

<a id="op-fa75bf129c566031f96a23eb"></a>
## Tee

`struct` · `tracing_subscriber::fmt::writer::Tee` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Tee<A, B>
```

Source: `src/fmt/writer.rs:641`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines two types implementing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) (or [`std::io::Write`]) to
produce a writer that writes to both [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a)'s returned writers.

This is returned by the [`MakeWriterExt::and`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-661cbfa269f4ebce10d749a7) method. See the method
documentation for details.

Unresolved upstream links (retained, not inferred): ``std::io::Write``.

<a id="op-fc029eecf78b1dfcb346dae0"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::Tee::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1051, 1], "end": [1067, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1056`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fe7b3d83f99136d492321b87"></a>
## clone

`function` · `tracing_subscriber::fmt::writer::Tee::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Tee<A, B>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [640, 16], "end": [640, 21], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/writer.rs:640`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bb374be05fdf84c7b435815e"></a>
## eq

`function` · `tracing_subscriber::fmt::writer::Tee::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Tee<A, B>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [640, 34], "end": [640, 43], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/writer.rs:640`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d1be443d73374b170b510210"></a>
## flush

`function` · `tracing_subscriber::fmt::writer::Tee::flush` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flush(&mut self) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1079, 1], "end": [1113, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:1091`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ffc5645d3af4f6f1bef73a1d"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::Tee::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [640, 23], "end": [640, 28], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:640`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d046197cf677ff68053464f2"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::Tee::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1051, 1], "end": [1067, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1059`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9d93d5a6d9fcd7b216fdd517"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::Tee::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1051, 1], "end": [1067, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:1064`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-70e8aaa35647fafad6df12b0"></a>
## new

`function` · `tracing_subscriber::fmt::writer::Tee::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(a: A, b: B) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1038, 1], "end": [1049, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:1046`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines two types implementing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a), returning
a new [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that produces [writers] that write to *both*
outputs.

See the documentation for [`MakeWriterExt::and`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-661cbfa269f4ebce10d749a7) for details.

[writers]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-d4f5f896e9297f9a4b0dbf29"></a>
## write

`function` · `tracing_subscriber::fmt::writer::Tee::write` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write(&mut self, buf: &[u8]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1079, 1], "end": [1113, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:1085`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-75fa001cb42c4f2a7bface62"></a>
## write_all

`function` · `tracing_subscriber::fmt::writer::Tee::write_all` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_all(&mut self, buf: &[u8]) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1079, 1], "end": [1113, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:1103`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a9b297984e21710a37ece9c4"></a>
## write_fmt

`function` · `tracing_subscriber::fmt::writer::Tee::write_fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1079, 1], "end": [1113, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:1109`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ad8971fe1e3f25a908baf90b"></a>
## write_vectored

`function` · `tracing_subscriber::fmt::writer::Tee::write_vectored` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::Tee", "path": "Tee"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [1079, 1], "end": [1113, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:1097`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
