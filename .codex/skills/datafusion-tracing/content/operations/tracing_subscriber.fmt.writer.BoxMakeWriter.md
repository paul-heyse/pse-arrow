# `tracing_subscriber::fmt::writer::BoxMakeWriter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.BoxMakeWriter.json).

<a id="op-5360e67ef72fa1f760a6246b"></a>
## BoxMakeWriter

`struct` · `tracing_subscriber::fmt::writer::BoxMakeWriter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct BoxMakeWriter
```

Source: `src/fmt/writer.rs:548`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A writer that erases the specific [`io::Write`] and [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) types being used.

This is useful in cases where the concrete type of the writer cannot be known
until runtime.

# Examples

A function that returns a [`Subscriber`] that will write to either stdout or stderr:

```rust
# use tracing::Subscriber;
# use tracing_subscriber::fmt::writer::BoxMakeWriter;

fn dynamic_writer(use_stderr: bool) -> impl Subscriber {
    let writer = if use_stderr {
        BoxMakeWriter::new(std::io::stderr)
    } else {
        BoxMakeWriter::new(std::io::stdout)
    };

    tracing_subscriber::fmt().with_writer(writer).finish()
}
```

[`Subscriber`]: tracing::Subscriber
[`io::Write`]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-f20ef7c94ce075bb12ed7374"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::BoxMakeWriter::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::BoxMakeWriter", "path": "BoxMakeWriter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [773, 1], "end": [785, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:774`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bbf00821636fd7bd1173910c"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::BoxMakeWriter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::BoxMakeWriter", "path": "BoxMakeWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [765, 1], "end": [771, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:766`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-43fe4f3ae996501d83fb4225"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::BoxMakeWriter::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::BoxMakeWriter", "path": "BoxMakeWriter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [773, 1], "end": [785, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:777`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-13d350f1822da00118b97ef3"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::BoxMakeWriter::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::BoxMakeWriter", "path": "BoxMakeWriter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [773, 1], "end": [785, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:782`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7022d8b95503dcec829e43ac"></a>
## new

`function` · `tracing_subscriber::fmt::writer::BoxMakeWriter::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new<M>(make_writer: M) -> Self where M: for<'a> MakeWriter<'a> + Send + Sync + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::BoxMakeWriter", "path": "BoxMakeWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [751, 1], "end": [763, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:754`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Constructs a `BoxMakeWriter` wrapping a type implementing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a).

