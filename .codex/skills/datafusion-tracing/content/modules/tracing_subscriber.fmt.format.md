# `tracing_subscriber::fmt::format`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.json).

<a id="op-a04fc10e64efcca800d0eaec"></a>
## format

`module` · `tracing_subscriber::fmt::format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
mod format
```

Source: `src/fmt/format/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Formatters for logging [`tracing`](../modules/tracing.md#op-9c1f7d23f358e3a0a8455190) events.

This module provides several formatter implementations, as well as utilities
for implementing custom formatters.

# Formatters
This module provides a number of formatter implementations:

* [`Full`](../operations/tracing_subscriber.fmt.format.Full.md#op-b6feb895c3a5317d8f8397fd): The default formatter. This emits human-readable,
  single-line logs for each event that occurs, with the current span context
  displayed before the formatted representation of the event. See
  [here](Full#example-output) for sample output.

* [`Compact`](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea): A variant of the default formatter, optimized for
  short line lengths. Fields from the current span context are appended to
  the fields of the formatted event, and span names are not shown; the
  verbosity level is abbreviated to a single character. See
  [here](Compact#example-output) for sample output.

* [`Pretty`](../operations/tracing_subscriber.fmt.format.pretty.Pretty.md#op-9ae7afec101d61be119e29f4): Emits excessively pretty, multi-line logs, optimized
  for human readability. This is primarily intended to be used in local
  development and debugging, or for command-line applications, where
  automated analysis and compact storage of logs is less of a priority than
  readability and visual appeal. See [here](Pretty#example-output)
  for sample output.

* [`Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c): Outputs newline-delimited JSON logs. This is intended
  for production use with systems where structured logs are consumed as JSON
  by analysis and viewing tools. The JSON output is not optimized for human
  readability. See [here](Json#example-output) for sample output.
