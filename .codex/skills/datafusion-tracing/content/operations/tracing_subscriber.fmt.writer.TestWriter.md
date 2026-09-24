# `tracing_subscriber::fmt::writer::TestWriter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.TestWriter.json).

<a id="op-b8e9c667384dce2df0d37142"></a>
## TestWriter

`struct` · `tracing_subscriber::fmt::writer::TestWriter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct TestWriter
```

Source: `src/fmt/writer.rs:516`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A writer intended to support [`libtest`'s output capturing][capturing] for use in unit tests.

`TestWriter` is used by [`fmt::Subscriber`] or [`fmt::Layer`] to enable capturing support.

`cargo test` can only capture output from the standard library's [`print!`] and [`eprint!`]
macros. See [`libtest`'s output capturing][capturing] and
[rust-lang/rust#90785](https://github.com/rust-lang/rust/issues/90785) for more details about
output capturing.

Writing to [`io::stdout`] and [`io::stderr`] produces the same results as using
[`libtest`'s `--nocapture` option][nocapture] which may make the results look unreadable.

[`fmt::Subscriber`]: super::Subscriber
[`fmt::Layer`]: super::Layer
[capturing]: https://doc.rust-lang.org/book/ch11-02-running-tests.html#showing-function-output
[nocapture]: https://doc.rust-lang.org/cargo/commands/cargo-test.html
[`io::stdout`]: std::io::stdout
[`io::stderr`]: std::io::stderr
[`print!`]: std::print!

Unresolved upstream links (retained, not inferred): ``eprint!``, `std::print!`, `std::io::stderr`, `std::io::stdout`.

<a id="op-2772cc0fec23c34439e56728"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::TestWriter::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [741, 1], "end": [747, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:742`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-059eead30a905c51461bc184"></a>
## default

`function` · `tracing_subscriber::fmt::writer::TestWriter::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> TestWriter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [515, 10], "end": [515, 17], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/writer.rs:515`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7c38d89de5d6cae7698da253"></a>
## flush

`function` · `tracing_subscriber::fmt::writer::TestWriter::flush` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flush(&mut self) -> io::Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [725, 1], "end": [739, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:736`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aa2ca38a1611b74947142a5d"></a>
## fmt

`function` · `tracing_subscriber::fmt::writer::TestWriter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [515, 19], "end": [515, 24], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/writer.rs:515`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a99969bd1079aa453d24ff66"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::TestWriter::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [741, 1], "end": [747, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}, "trait_path": "tracing_subscriber::fmt::writer::MakeWriter"}`

Source: `src/fmt/writer.rs:744`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d82e8c0d56856f1f57ed883f"></a>
## new

`function` · `tracing_subscriber::fmt::writer::TestWriter::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [713, 1], "end": [723, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:715`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `TestWriter` with the default configuration.

<a id="op-324adf0b72d74be8bef314af"></a>
## with_stderr

`function` · `tracing_subscriber::fmt::writer::TestWriter::with_stderr` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_stderr() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [713, 1], "end": [723, 2], "filename": "src/fmt/writer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/writer.rs:720`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `TestWriter` that writes to `stderr` instead of `stdout`.

<a id="op-ca566752e3a0ee3143337c0e"></a>
## write

`function` · `tracing_subscriber::fmt::writer::TestWriter::write` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write(&mut self, buf: &[u8]) -> io::Result<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::writer::TestWriter", "path": "TestWriter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [725, 1], "end": [739, 2], "filename": "src/fmt/writer.rs"}, "trait": {"args": null, "id": "std::io::Write", "path": "Write"}, "trait_path": "std::io::Write"}`

Source: `src/fmt/writer.rs:726`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
