# `tracing_subscriber::fmt::writer::MutexGuardWriter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.MutexGuardWriter.json).

<a id="op-32b28f7c7b09500ebeb990ce"></a>
## MutexGuardWriter

`struct` · `tracing_subscriber::fmt::writer::MutexGuardWriter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct MutexGuardWriter<'a, W>
```

Source: `src/fmt/writer.rs:659`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A type implementing [`io::Write`] for a [`MutexGuard`] where the type
inside the [`Mutex`] implements [`io::Write`].

This is used by the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) implementation for [`Mutex`], because
[`MutexGuard`] itself will not implement [`io::Write`] — instead, it
_dereferences_ to a type implementing [`io::Write`]. Because [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a)
requires the `Writer` type to implement [`io::Write`], it's necessary to add
a newtype that forwards the trait implementation.

[`io::Write`]: std::io::Write
[`MutexGuard`]: std::sync::MutexGuard
[`Mutex`]: std::sync::Mutex

Unresolved upstream links (retained, not inferred): `std::io::Write`, `std::sync::Mutex`, `std::sync::MutexGuard`.

<a id="op-e41ea474af88a7163f23deb0"></a>
## flush

`function` · `tracing_subscriber::fmt::writer::MutexGuardWriter::flush` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flush(&mut self) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MutexGuardWriter", "path": "MutexGuardWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [819, 1], "end": [847, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:829`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d7d9797e70bc8b77665f578e"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::MutexGuardWriter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MutexGuardWriter", "path": "MutexGuardWriter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [658, 10], "end": [658, 15], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:658`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b96bd45a011e2eb7599b809"></a>
## write

`function` · `tracing_subscriber::fmt::writer::MutexGuardWriter::write` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write(&mut self, buf: &[u8]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MutexGuardWriter", "path": "MutexGuardWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [819, 1], "end": [847, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:824`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d27ee84235a46c5d682f08e8"></a>
## write_all

`function` · `tracing_subscriber::fmt::writer::MutexGuardWriter::write_all` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_all(&mut self, buf: &[u8]) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MutexGuardWriter", "path": "MutexGuardWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [819, 1], "end": [847, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:839`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-df962fd1755731271fc26ac3"></a>
## write_fmt

`function` · `tracing_subscriber::fmt::writer::MutexGuardWriter::write_fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MutexGuardWriter", "path": "MutexGuardWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [819, 1], "end": [847, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:844`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9b57b02677915fe59b168df3"></a>
## write_vectored

`function` · `tracing_subscriber::fmt::writer::MutexGuardWriter::write_vectored` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MutexGuardWriter", "path": "MutexGuardWriter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "std::io::Write", "path": "io::Write"}}}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [819, 1], "end": [847, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:834`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
