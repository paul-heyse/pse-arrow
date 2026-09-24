# `tracing_subscriber::field::VisitFmt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.VisitFmt.json).

<a id="op-1bd1c68868a8de75cd448ec5"></a>
## VisitFmt

`trait` · `tracing_subscriber::field::VisitFmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait VisitFmt: VisitOutput<fmt::Result>
```

Source: `src/field/mod.rs:124`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait implemented by visitors to indicate that they write to a
`fmt::Write` instance, and allow access to that writer.

<a id="op-412c76487f3008990053ebc5"></a>
## writer

`function` · `tracing_subscriber::field::VisitFmt::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Source: `src/field/mod.rs:126`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the formatter that this visitor writes to.
