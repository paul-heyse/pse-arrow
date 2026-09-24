# `tracing_subscriber::fmt::writer::MakeWriter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.MakeWriter.json).

<a id="op-49cad405bcbd7170e0ab390a"></a>
## MakeWriter

`trait` · `tracing_subscriber::fmt::writer::MakeWriter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait MakeWriter<'a>
```

Source: `src/fmt/writer.rs:98`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A type that can create [`io::Write`] instances.

`MakeWriter` is used by [`fmt::Layer`] or [`fmt::Subscriber`] to print
formatted text representations of [`Event`]s.

This trait is already implemented for function pointers and
immutably-borrowing closures that return an instance of [`io::Write`], such
as [`io::stdout`] and [`io::stderr`]. Additionally, it is implemented for
[`std::sync::Mutex`] when the type inside the mutex implements
[`io::Write`].

# Examples

The simplest usage is to pass in a named function that returns a writer. For
example, to log all events to stderr, we could write:
```
let subscriber = tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .finish();
# drop(subscriber);
```

Any function that returns a writer can be used:

```
fn make_my_great_writer() -> impl std::io::Write {
    // ...
    # std::io::stdout()
}

let subscriber = tracing_subscriber::fmt()
    .with_writer(make_my_great_writer)
    .finish();
# drop(subscriber);
```

A closure can be used to introduce arbitrary logic into how the writer is
created. Consider the (admittedly rather silly) example of sending every 5th
event to stderr, and all other events to stdout:

```
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

let n = AtomicUsize::new(0);
let subscriber = tracing_subscriber::fmt()
    .with_writer(move || -> Box<dyn io::Write> {
        if n.fetch_add(1, Relaxed) % 5 == 0 {
            Box::new(io::stderr())
        } else {
            Box::new(io::stdout())
       }
    })
    .finish();
# drop(subscriber);
```

A single instance of a type implementing [`io::Write`] may be used as a
`MakeWriter` by wrapping it in a [`Mutex`]. For example, we could
write to a file like so:

```
use std::{fs::File, sync::Mutex};

# fn docs() -> Result<(), Box<dyn std::error::Error>> {
let log_file = File::create("my_cool_trace.log")?;
let subscriber = tracing_subscriber::fmt()
    .with_writer(Mutex::new(log_file))
    .finish();
# drop(subscriber);
# Ok(())
# }
```

[`io::Write`]: std::io::Write
[`fmt::Layer`]: crate::fmt::Layer
[`fmt::Subscriber`]: crate::fmt::Subscriber
[`Event`]: tracing_core::event::Event
[`io::stdout`]: std::io::stdout()
[`io::stderr`]: std::io::stderr()
[`MakeWriter::make_writer_for`]: MakeWriter::make_writer_for
[`Metadata`]: tracing_core::Metadata
[levels]: tracing_core::Level
[targets]: tracing_core::Metadata::target

Unresolved upstream links (retained, not inferred): ``Mutex``, `std::io::Write`, `std::io::stdout()`, `tracing_core::Metadata::target`, `std::io::stderr()`, ``std::sync::Mutex``.

<a id="op-0b71b1694d10f2872b678698"></a>
## Writer

`assoc_type` · `tracing_subscriber::fmt::writer::MakeWriter::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Writer
```

Source: `src/fmt/writer.rs:103`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The concrete [`io::Write`] implementation returned by [`make_writer`].

[`io::Write`]: std::io::Write
[`make_writer`]: MakeWriter::make_writer

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-04947f8cd797aae6fbf28878"></a>
## make_writer

`function` · `tracing_subscriber::fmt::writer::MakeWriter::make_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer(&'a self) -> Self::Writer
```

Source: `src/fmt/writer.rs:118`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an instance of [`Writer`].

# Implementer notes

[`fmt::Layer`] or [`fmt::Subscriber`] will call this method each time an event is recorded. Ensure any state
that must be saved across writes is not lost when the [`Writer`] instance is dropped. If
creating a [`io::Write`] instance is expensive, be sure to cache it when implementing
[`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) to improve performance.

[`Writer`]: MakeWriter::Writer
[`fmt::Layer`]: crate::fmt::Layer
[`fmt::Subscriber`]: crate::fmt::Subscriber
[`io::Write`]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.

<a id="op-f80df00cea31d415c78108e6"></a>
## make_writer_for

`function` · `tracing_subscriber::fmt::writer::MakeWriter::make_writer_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer
```

Source: `src/fmt/writer.rs:208`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a [`Writer`] for writing data from the span or event described
by the provided [`Metadata`].

By default, this calls [`self.make_writer()`][make_writer], ignoring
the provided metadata, but implementations can override this to provide
metadata-specific behaviors.

This method allows `MakeWriter` implementations to implement different
behaviors based on the span or event being written. The `MakeWriter`
type might return different writers based on the provided metadata, or
might write some values to the writer before or after providing it to
the caller.

For example, we might want to write data from spans and events at the
[`ERROR`] and [`WARN`] levels to `stderr`, and data from spans or events
at lower levels to stdout:

```
use std::io::{self, Stdout, Stderr, StdoutLock, StderrLock};
use tracing_subscriber::fmt::writer::MakeWriter;
use tracing_core::{Metadata, Level};

pub struct MyMakeWriter {
    stdout: Stdout,
    stderr: Stderr,
}

/// A lock on either stdout or stderr, depending on the verbosity level
/// of the event being written.
pub enum StdioLock<'a> {
    Stdout(StdoutLock<'a>),
    Stderr(StderrLock<'a>),
}

impl<'a> io::Write for StdioLock<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            StdioLock::Stdout(lock) => lock.write(buf),
            StdioLock::Stderr(lock) => lock.write(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        // ...
        # match self {
        #     StdioLock::Stdout(lock) => lock.write_all(buf),
        #     StdioLock::Stderr(lock) => lock.write_all(buf),
        # }
    }

    fn flush(&mut self) -> io::Result<()> {
        // ...
        # match self {
        #     StdioLock::Stdout(lock) => lock.flush(),
        #     StdioLock::Stderr(lock) => lock.flush(),
        # }
    }
}

impl<'a> MakeWriter<'a> for MyMakeWriter {
    type Writer = StdioLock<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        // We must have an implementation of `make_writer` that makes
        // a "default" writer without any configuring metadata. Let's
        // just return stdout in that case.
        StdioLock::Stdout(self.stdout.lock())
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if meta.level() <= &Level::WARN {
            return StdioLock::Stderr(self.stderr.lock());
        }

        // Otherwise, we'll return stdout.
        StdioLock::Stdout(self.stdout.lock())
    }
}
```

[`Writer`]: MakeWriter::Writer
[`Metadata`]: tracing_core::Metadata
[make_writer]: MakeWriter::make_writer
[`WARN`]: tracing_core::Level::WARN
[`ERROR`]: tracing_core::Level::ERROR

Unresolved upstream links (retained, not inferred): `tracing_core::Level::WARN`, `tracing_core::Level::ERROR`.
