# `tracing_subscriber::fmt::writer::OptionalWriter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.writer.OptionalWriter.json).

<a id="op-aca5124df0ea38cd4409ff35"></a>
## OptionalWriter

`type_alias` · `tracing_subscriber::fmt::writer::OptionalWriter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
type OptionalWriter<T> = EitherWriter<T, std::io::Sink>
```

Source: `src/fmt/writer.rs:574`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [writer] which may or may not be enabled.

This may be used by [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) implementations that wish to
conditionally enable or disable the returned writer based on a span or
event's [`Metadata`](../operations/tracing_core.metadata.Metadata.md#op-3c5a7a9d81c273e2173bb24c).

[writer]: std::io::Write

Unresolved upstream links (retained, not inferred): `std::io::Write`.
