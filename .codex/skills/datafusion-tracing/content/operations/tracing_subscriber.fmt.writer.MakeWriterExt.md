# `tracing_subscriber::fmt::writer::MakeWriterExt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.MakeWriterExt.json).

<a id="op-4f9599f0ed408c253b20e964"></a>
## MakeWriterExt

`trait` · `tracing_subscriber::fmt::writer::MakeWriterExt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait MakeWriterExt<'a>: MakeWriter<'a>
```

Source: `src/fmt/writer.rs:220`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait adding combinators for working with types implementing
[`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a).

This is not intended to be implemented directly for user-defined
[`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a)s; instead, it should be imported when the desired methods are
used.

<a id="op-661cbfa269f4ebce10d749a7"></a>
## and

`function` · `tracing_subscriber::fmt::writer::MakeWriterExt::and` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn and<B>(self, other: B) -> Tee<Self, B> where Self: Sized, B: MakeWriter<'a> + Sized
```

Source: `src/fmt/writer.rs:455`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines `self` with another type implementing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a), returning
a new [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that produces [writers] that write to *both*
outputs.

If writing to either writer returns an error, the returned writer will
return that error. However, both writers will still be written to before
the error is returned, so it is possible for one writer to fail while
the other is written to successfully.

# Examples

```
use tracing_subscriber::fmt::writer::MakeWriterExt;

// Construct a writer that outputs events to `stdout` *and* `stderr`.
let mk_writer = std::io::stdout.and(std::io::stderr);

tracing_subscriber::fmt().with_writer(mk_writer).init();
```

`and` can be used in conjunction with filtering combinators. For
example, if we want to write to a number of outputs depending on the
level of an event, we could write:

```
use tracing::Level;
# use tracing_subscriber::fmt::writer::MakeWriterExt;
use std::{sync::Arc, fs::File};
# // don't actually create the file when running the tests.
# fn docs() -> std::io::Result<()> {
let debug_log = Arc::new(File::create("debug.log")?);

// Write everything to the debug log.
let mk_writer = debug_log
    // Write the `ERROR` and `WARN` levels to stderr.
    .and(std::io::stderr.with_max_level(Level::WARN))
    // Write `INFO` to `stdout`.
    .and(std::io::stdout
        .with_max_level(Level::INFO)
        .with_min_level(Level::INFO)
    );

tracing_subscriber::fmt().with_writer(mk_writer).init();
# Ok(()) }
```

[writers]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-321b77fb903fda6ba83d5fdd"></a>
## or_else

`function` · `tracing_subscriber::fmt::writer::MakeWriterExt::or_else` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn or_else<W, B>(self, other: B) -> OrElse<Self, B> where Self: MakeWriter<'a, Writer = OptionalWriter<W>> + Sized, B: MakeWriter<'a> + Sized, W: Write
```

Source: `src/fmt/writer.rs:486`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines `self` with another type implementing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a), returning
a new [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that calls `other`'s [`make_writer`] if `self`'s
`make_writer` returns [`OptionalWriter::none`][own].

# Examples

```
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

// Produces a writer that writes to `stderr` if the level is <= WARN,
// or returns `OptionalWriter::none()` otherwise.
let stderr = std::io::stderr.with_max_level(Level::WARN);

// If the `stderr` `MakeWriter` is disabled by the max level filter,
// write to stdout instead:
let mk_writer = stderr.or_else(std::io::stdout);

tracing_subscriber::fmt().with_writer(mk_writer).init();
```

[`make_writer`]: MakeWriter::make_writer
[own]: EitherWriter::none

<a id="op-25389b1a363c79682d8c131a"></a>
## with_filter

`function` · `tracing_subscriber::fmt::writer::MakeWriterExt::with_filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_filter<F>(self, filter: F) -> WithFilter<Self, F> where Self: Sized, F: Fn(&Metadata<'_>) -> bool
```

Source: `src/fmt/writer.rs:400`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` with a predicate that takes a span or event's [`Metadata`]
and returns a `bool`. The returned [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a)'s
[`MakeWriter::make_writer_for`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-f80df00cea31d415c78108e6) method will check the predicate to
determine if  a writer should be produced for a given span or event.

If the predicate returns `false`, the wrapped [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a)'s
[`make_writer_for`][mwf] will return [`OptionalWriter::none`][own].
Otherwise, it calls the wrapped [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a)'s
[`make_writer_for`][mwf] method, and returns the produced writer.

This can be used to filter an output based on arbitrary [`Metadata`]
parameters.

# Examples

Writing events with a specific target to an HTTP access log, and other
events to stdout:

```
use tracing_subscriber::fmt::writer::MakeWriterExt;
use std::{sync::Arc, fs::File};
# // don't actually create the file when running the tests.
# fn docs() -> std::io::Result<()> {
let access_log = Arc::new(File::create("access.log")?);

let mk_writer = access_log
    // Only write events with the target "http::access_log" to the
    // access log file.
    .with_filter(|meta| meta.target() == "http::access_log")
    // Write events with all other targets to stdout.
    .or_else(std::io::stdout);

tracing_subscriber::fmt().with_writer(mk_writer).init();
# Ok(())
# }
```

Conditionally enabling or disabling a log file:
```
use tracing_subscriber::fmt::writer::MakeWriterExt;
use std::{
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    fs::File,
};

static DEBUG_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

# // don't actually create the file when running the tests.
# fn docs() -> std::io::Result<()> {
// Create the debug log file
let debug_file = Arc::new(File::create("debug.log")?)
    // Enable the debug log only if the flag is enabled.
    .with_filter(|_| DEBUG_LOG_ENABLED.load(Ordering::Acquire));

// Always write to stdout
let mk_writer = std::io::stdout
    // Write to the debug file if it's enabled
    .and(debug_file);

tracing_subscriber::fmt().with_writer(mk_writer).init();

// ...

// Later, we can toggle on or off the debug log file.
DEBUG_LOG_ENABLED.store(true, Ordering::Release);
# Ok(())
# }
```

[`Metadata`]: tracing_core::Metadata
[mwf]: MakeWriter::make_writer_for
[own]: EitherWriter::none

<a id="op-7991c17e378056a2039494a3"></a>
## with_max_level

`function` · `tracing_subscriber::fmt::writer::MakeWriterExt::with_max_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_level(self, level: tracing_core::Level) -> WithMaxLevel<Self> where Self: Sized
```

Source: `src/fmt/writer.rs:279`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` and returns a [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that will only write output
for events at or below the provided verbosity [`Level`]. For instance,
`Level::TRACE` is considered to be _more verbose` than `Level::INFO`.

Events whose level is more verbose than `level` will be ignored, and no
output will be written.

# Examples

```
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

// Construct a writer that outputs events to `stderr` only if the span or
// event's level is <= WARN (WARN and ERROR).
let mk_writer = std::io::stderr.with_max_level(Level::WARN);

tracing_subscriber::fmt().with_writer(mk_writer).init();
```

Writing the `ERROR` and `WARN` levels to `stderr`, and everything else
to `stdout`:

```
# use tracing::Level;
# use tracing_subscriber::fmt::writer::MakeWriterExt;

let mk_writer = std::io::stderr
    .with_max_level(Level::WARN)
    .or_else(std::io::stdout);

tracing_subscriber::fmt().with_writer(mk_writer).init();
```

Writing the `ERROR` level to `stderr`, the `INFO` and `WARN` levels to
`stdout`, and the `INFO` and DEBUG` levels to a file:

```
# use tracing::Level;
# use tracing_subscriber::fmt::writer::MakeWriterExt;
use std::{sync::Arc, fs::File};
# // don't actually create the file when running the tests.
# fn docs() -> std::io::Result<()> {
let debug_log = Arc::new(File::create("debug.log")?);

let mk_writer = std::io::stderr
    .with_max_level(Level::ERROR)
    .or_else(std::io::stdout
        .with_max_level(Level::INFO)
        .and(debug_log.with_max_level(Level::DEBUG))
    );

tracing_subscriber::fmt().with_writer(mk_writer).init();
# Ok(()) }
```

[`Level`]: tracing_core::Level
[`io::Write`]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-bd0cba079fd2dbd015c758ab"></a>
## with_min_level

`function` · `tracing_subscriber::fmt::writer::MakeWriterExt::with_min_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_min_level(self, level: tracing_core::Level) -> WithMinLevel<Self> where Self: Sized
```

Source: `src/fmt/writer.rs:321`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` and returns a [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that will only write output
for events at or above the provided verbosity [`Level`].

Events whose level is less verbose than `level` will be ignored, and no
output will be written.

# Examples

```
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;

// Construct a writer that outputs events to `stdout` only if the span or
// event's level is >= DEBUG (DEBUG and TRACE).
let mk_writer = std::io::stdout.with_min_level(Level::DEBUG);

tracing_subscriber::fmt().with_writer(mk_writer).init();
```
This can be combined with [`MakeWriterExt::with_max_level`](../operations/tracing_subscriber.fmt.writer.MakeWriterExt.md#op-7991c17e378056a2039494a3) to write
only within a range of levels:

```
# use tracing::Level;
# use tracing_subscriber::fmt::writer::MakeWriterExt;
// Only write the `DEBUG` and `INFO` levels to stdout.
let mk_writer = std::io::stdout
    .with_max_level(Level::DEBUG)
    .with_min_level(Level::INFO)
    // Write the `WARN` and `ERROR` levels to stderr.
    .and(std::io::stderr.with_min_level(Level::WARN));

tracing_subscriber::fmt().with_writer(mk_writer).init();
```
[`Level`]: tracing_core::Level
[`io::Write`]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.
