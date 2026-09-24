# `tracing_subscriber::fmt::writer::EitherWriter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.EitherWriter.json).

<a id="op-4e2e1a917fe2fdba5b634207"></a>
## EitherWriter

`enum` · `tracing_subscriber::fmt::writer::EitherWriter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
enum EitherWriter<A, B>
```

Source: `src/fmt/writer.rs:560`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [writer] that is one of two types implementing [`io::Write`].

This may be used by [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) implementations that may conditionally
return one of two writers.

[writer]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`, ``io::Write``.

<a id="op-2ee8c70eed68c33b77a9a00e"></a>
## A

`variant` · `tracing_subscriber::fmt::writer::EitherWriter::A` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
A
```

Source: `src/fmt/writer.rs:562`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A writer of type `A`.

<a id="op-34602d3cc3af8e757f5a75f3"></a>
## B

`variant` · `tracing_subscriber::fmt::writer::EitherWriter::B` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
B
```

Source: `src/fmt/writer.rs:564`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A writer of type `B`.

<a id="op-b60dde74982c4266e3b6340f"></a>
## clone

`function` · `tracing_subscriber::fmt::writer::EitherWriter::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> EitherWriter<A, B>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [559, 16], "end": [559, 21], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/writer.rs:559`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c918d7f7439e839f21f5e620"></a>
## eq

`function` · `tracing_subscriber::fmt::writer::EitherWriter::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &EitherWriter<A, B>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [559, 34], "end": [559, 43], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/writer.rs:559`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0be5dc05c98d9820b26a148f"></a>
## flush

`function` · `tracing_subscriber::fmt::writer::EitherWriter::flush` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flush(&mut self) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [851, 1], "end": [895, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:865`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f80b0a75624ff0fc8ec90d4d"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::EitherWriter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [559, 23], "end": [559, 28], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:559`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f6e00051b2c8a83963973ec7"></a>
## none

`function` · `tracing_subscriber::fmt::writer::EitherWriter::none` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn none() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}, {"type": {"resolved_path": {"args": null, "id": "core::io::util::Sink", "path": "Sink"}}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [897, 1], "end": [917, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:906`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a [disabled writer].

Any bytes written to the returned writer are discarded.

This is equivalent to returning [`Option::None`].

[disabled writer]: std::io::sink

Unresolved upstream links (retained, not inferred): ``Option::None``, `std::io::sink`.

<a id="op-50c42a02ad55d7e674b58a59"></a>
## some

`function` · `tracing_subscriber::fmt::writer::EitherWriter::some` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn some(t: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}, {"type": {"resolved_path": {"args": null, "id": "core::io::util::Sink", "path": "Sink"}}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [897, 1], "end": [917, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:914`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an enabled writer of type `T`.

This is equivalent to returning [`Option::Some`].

Unresolved upstream links (retained, not inferred): ``Option::Some``.

<a id="op-181482e21943e3dc10cf93d1"></a>
## write

`function` · `tracing_subscriber::fmt::writer::EitherWriter::write` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write(&mut self, buf: &[u8]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [851, 1], "end": [895, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:857`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ed9a10578023d36bc5418fde"></a>
## write_all

`function` · `tracing_subscriber::fmt::writer::EitherWriter::write_all` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_all(&mut self, buf: &[u8]) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [851, 1], "end": [895, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:881`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f98e96af9fc763919ee47425"></a>
## write_fmt

`function` · `tracing_subscriber::fmt::writer::EitherWriter::write_fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [851, 1], "end": [895, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:889`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c1d03e5336d7972f491102c1"></a>
## write_vectored

`function` · `tracing_subscriber::fmt::writer::EitherWriter::write_vectored` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::EitherWriter", "path": "EitherWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [851, 1], "end": [895, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:873`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
