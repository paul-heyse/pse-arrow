# `tracing_subscriber::fmt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.json).

<a id="op-7f8793caaa642a6fdcf5c97d"></a>
## fmt

`module` · `tracing_subscriber::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
mod fmt
```

Source: `src/fmt/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A `Subscriber` for formatting and logging `tracing` data.

# Overview

[`tracing`] is a framework for instrumenting Rust programs with context-aware,
structured, event-based diagnostic information. This crate provides an
implementation of the [`Subscriber`] trait that records `tracing`'s `Event`s
and `Span`s by formatting them as text and logging them to stdout.

# Usage

First, add this to your `Cargo.toml` file:

```toml
[dependencies]
tracing-subscriber = "0.3"
```

*Compiler support: [requires `rustc` 1.65+][msrv]*

[msrv]: super#supported-rust-versions

Add the following to your executable to initialize the default subscriber:
```rust
use tracing_subscriber;

tracing_subscriber::fmt::init();
```

## Filtering Events with Environment Variables

The default subscriber installed by `init` enables you to filter events
at runtime using environment variables (using the [`EnvFilter`]).

The filter syntax is a superset of the [`env_logger`] syntax.

For example:
- Setting `RUST_LOG=debug` enables all `Span`s and `Event`s
  set to the log level `DEBUG` or higher
- Setting `RUST_LOG=my_crate=trace` enables `Span`s and `Event`s
  in `my_crate` at all log levels

**Note**: This should **not** be called by libraries. Libraries should use
[`tracing`] to publish `tracing` `Event`s.

# Configuration

You can configure a subscriber instead of using the defaults with
the following functions:

### Subscriber

The [`FmtSubscriber`] formats and records `tracing` events as line-oriented logs.
You can create one by calling:

```rust
let subscriber = tracing_subscriber::fmt()
    // ... add configuration
    .finish();
```

You can find the configuration methods for [`FmtSubscriber`] in
[`SubscriberBuilder`](../operations/tracing_subscriber.fmt.SubscriberBuilder.md#op-38fcd8fc52ca33acc34d54e9).

## Formatters

The output format used by the layer and subscriber in this module is
represented by implementing the [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) trait, and can be
customized. This module provides a number of formatter implementations:

* [`format::Full`](../operations/tracing_subscriber.fmt.format.Full.md#op-b6feb895c3a5317d8f8397fd): The default formatter. This emits human-readable,
  single-line logs for each event that occurs, with the current span context
  displayed before the formatted representation of the event. See
  [here](format::Full#example-output) for sample output.

* [`format::Compact`](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea): A variant of the default formatter, optimized for
  short line lengths. Fields from the current span context are appended to
  the fields of the formatted event. See
  [here](format::Compact#example-output) for sample output.

* [`format::Pretty`](../operations/tracing_subscriber.fmt.format.pretty.Pretty.md#op-9ae7afec101d61be119e29f4): Emits excessively pretty, multi-line logs, optimized
  for human readability. This is primarily intended to be used in local
  development and debugging, or for command-line applications, where
  automated analysis and compact storage of logs is less of a priority than
  readability and visual appeal. See [here](format::Pretty#example-output)
  for sample output.

* [`format::Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c): Outputs newline-delimited JSON logs. This is intended
  for production use with systems where structured logs are consumed as JSON
  by analysis and viewing tools. The JSON output is not optimized for human
  readability. See [here](format::Json#example-output) for sample output.

### Customizing Formatters

The formatting of log lines for spans and events is controlled by two
traits, [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) and [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0). The [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) trait
determines the overall formatting of the log line, such as what information
from the event's metadata and span context is included and in what order.
The [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) trait determines how fields &mdash; both the event's
fields and fields on spans &mdash; are formatted.

The [`fmt::format`] module provides several types which implement these traits,
many of which expose additional configuration options to customize their
output. The [`format::Format`](../operations/tracing_subscriber.fmt.format.Format.md#op-758341dc152c0b1f07c0c605) type implements common configuration used by
all the formatters provided in this crate, and can be used as a builder to
set specific formatting settings. For example:

```
use tracing_subscriber::fmt;

// Configure a custom event formatter
let format = fmt::format()
   .with_level(false) // don't include levels in formatted output
   .with_target(false) // don't include targets
   .with_thread_ids(true) // include the thread ID of the current thread
   .with_thread_names(true) // include the name of the current thread
   .compact(); // use the `Compact` formatting style.

// Create a `fmt` subscriber that uses our custom event format, and set it
// as the default.
tracing_subscriber::fmt()
    .event_format(format)
    .init();
```

However, if a specific output format is needed, other crates can
also implement [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) and [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0). See those traits'
documentation for details on how to implement them.

## Filters

If you want to filter the `tracing` `Events` based on environment
variables, you can use the [`EnvFilter`] as follows:

```rust
use tracing_subscriber::EnvFilter;

let filter = EnvFilter::from_default_env();
```

As mentioned above, the [`EnvFilter`] allows `Span`s and `Event`s to
be filtered at runtime by setting the `RUST_LOG` environment variable.

You can find the other available [`filter`]s in the documentation.

### Using Your Subscriber

Finally, once you have configured your `Subscriber`, you need to
configure your executable to use it.

A subscriber can be installed globally using:
```rust
use tracing;
use tracing_subscriber::FmtSubscriber;

let subscriber = FmtSubscriber::new();

tracing::subscriber::set_global_default(subscriber)
    .map_err(|_err| eprintln!("Unable to set global default subscriber"));
// Note this will only fail if you try to set the global default
// subscriber multiple times
```

### Composing Layers

Composing an [`EnvFilter`] `Layer` and a [format `Layer`][super::fmt::Layer](../operations/tracing_subscriber.fmt.fmt_layer.Layer.md#op-fa1a59e4f05bcd8ea89adaa4):

```rust
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

let fmt_layer = fmt::layer()
    .with_target(false);
let filter_layer = EnvFilter::try_from_default_env()
    .or_else(|_| EnvFilter::try_new("info"))
    .unwrap();

tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();
```

[`EnvFilter`]: super::filter::EnvFilter
[`env_logger`]: https://docs.rs/env_logger/
[`filter`]: super::filter
[`FmtSubscriber`]: Subscriber
[`Subscriber`]:
    https://docs.rs/tracing/latest/tracing/trait.Subscriber.html
[`tracing`]: https://crates.io/crates/tracing
[`fmt::format`]: mod@crate::fmt::format
