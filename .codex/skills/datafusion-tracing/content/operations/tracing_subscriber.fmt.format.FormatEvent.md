# `tracing_subscriber::fmt::format::FormatEvent`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.FormatEvent.json).

<a id="op-f71c6d57702e5d6dfe4a8abb"></a>
## FormatEvent

`trait` · `tracing_subscriber::fmt::format::FormatEvent` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait FormatEvent<S, N> where S: Subscriber + for<'a> LookupSpan<'a>, N: for<'a> FormatFields<'a> + 'static
```

Source: `src/fmt/format/mod.rs:198`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A type that can format a tracing [`Event`] to a [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021).

[`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) is primarily used in the context of [`fmt::Subscriber`] or
[`fmt::Layer`]. Each time an event is dispatched to [`fmt::Subscriber`] or
[`fmt::Layer`], the subscriber or layer
forwards it to its associated [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) to emit a log message.

This trait is already implemented for function pointers with the same
signature as `format_event`.

# Arguments

The following arguments are passed to [`FormatEvent::format_event`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f2311ad8285c947f806e24ac):

* A [`FmtContext`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-edf8024d45f27b1a256d34ab). This is an extension of the [`layer::Context`] type,
  which can be used for accessing stored information such as the current
  span context an event occurred in.

  In addition, [`FmtContext`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-edf8024d45f27b1a256d34ab) exposes access to the [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0)
  implementation that the subscriber was configured to use via the
  [`FmtContext::field_format`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-071d5b13e98cb914d4209740) method. This can be used when the
  [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) implementation needs to format the event's fields.

  For convenience, [`FmtContext`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-edf8024d45f27b1a256d34ab) also implements [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0),
  forwarding to the configured [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) type.

* A [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) to which the formatted representation of the event is
  written. This type implements the [`std::fmt::Write`] trait, and therefore
  can be used with the [`std::write!`] and [`std::writeln!`] macros, as well
  as calling [`std::fmt::Write`] methods directly.

  The [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) type also implements additional methods that provide
  information about how the event should be formatted. The
  [`Writer::has_ansi_escapes`] method indicates whether [ANSI terminal
  escape codes] are supported by the underlying I/O writer that the event
  will be written to. If this returns `true`, the formatter is permitted to
  use ANSI escape codes to add colors and other text formatting to its
  output. If it returns `false`, the event will be written to an output that
  does not support ANSI escape codes (such as a log file), and they should
  not be emitted.

  Crates like [`nu_ansi_term`] and [`owo-colors`] can be used to add ANSI
  escape codes to formatted output.

* The actual [`Event`] to be formatted.

# Examples

This example re-implements a simplified version of this crate's [default
formatter]:

```rust
use std::fmt;
use tracing_core::{Subscriber, Event};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext,
    FormattedFields,
};
use tracing_subscriber::registry::LookupSpan;

struct MyFormatter;

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

let _subscriber = tracing_subscriber::fmt()
    .event_format(MyFormatter)
    .init();

let _span = tracing::info_span!("my_span", answer = 42).entered();
tracing::info!(question = "life, the universe, and everything", "hello world");
```

This formatter will print events like this:

```text
DEBUG yak_shaving::shaver: some-span{field-on-span=foo}: started shaving yak
```

[`layer::Context`]: crate::layer::Context
[`fmt::Layer`]: super::Layer
[`fmt::Subscriber`]: super::Subscriber
[`Event`]: tracing::Event
[implements `FormatFields`]: super::FmtContext#impl-FormatFields<'writer>
[ANSI terminal escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code
[`Writer::has_ansi_escapes`]: Writer::has_ansi_escapes
[`nu_ansi_term`]: https://crates.io/crates/nu_ansi_term
[`owo-colors`]: https://crates.io/crates/owo-colors
[default formatter]: Full

Unresolved upstream links (retained, not inferred): ``std::fmt::Write``, ``std::writeln!``, ``std::write!``.

<a id="op-f2311ad8285c947f806e24ac"></a>
## format_event

`function` · `tracing_subscriber::fmt::format::FormatEvent::format_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_event(&self, ctx: &FmtContext<'_, S, N>, writer: Writer<'_>, event: &Event<'_>) -> fmt::Result
```

Source: `src/fmt/format/mod.rs:204`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Write a log message for [`Event`](../operations/tracing_core.event.Event.md#op-7ee85389e31294d1a098f039) in [`FmtContext`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-edf8024d45f27b1a256d34ab) to the given [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021).
