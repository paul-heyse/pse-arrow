# `tracing_subscriber::fmt::format::FormatFields`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.FormatFields.json).

<a id="op-9bc846bbdc7bd7bc3e4045c0"></a>
## FormatFields

`trait` · `tracing_subscriber::fmt::format::FormatFields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait FormatFields<'writer>
```

Source: `src/fmt/format/mod.rs:235`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A type that can format a [set of fields] to a [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021).

[`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) is primarily used in the context of [`FmtSubscriber`]. Each
time a span or event with fields is recorded, the subscriber will format
those fields with its associated [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation.

[set of fields]: crate::field::RecordFields
[`FmtSubscriber`]: super::Subscriber

<a id="op-946b14ff5f93926a8d29b2e4"></a>
## add_fields

`function` · `tracing_subscriber::fmt::format::FormatFields::add_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn add_fields(&self, current: &'writer mut FormattedFields<Self>, fields: &span::Record<'_>) -> fmt::Result
```

Source: `src/fmt/format/mod.rs:244`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Record additional field(s) on an existing span.

By default, this appends a space to the current set of fields if it is
non-empty, and then calls `self.format_fields`. If different behavior is
required, the default implementation of this method can be overridden.

<a id="op-01424b9dbd594952e76f0f8d"></a>
## format_fields

`function` · `tracing_subscriber::fmt::format::FormatFields::format_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> fmt::Result
```

Source: `src/fmt/format/mod.rs:237`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Format the provided `fields` to the provided [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021), returning a result.
