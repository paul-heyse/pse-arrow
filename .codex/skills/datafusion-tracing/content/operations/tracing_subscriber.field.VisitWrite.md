# `tracing_subscriber::field::VisitWrite`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.VisitWrite.json).

<a id="op-74314928fc09151080b884fb"></a>
## VisitWrite

`trait` · `tracing_subscriber::field::VisitWrite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait VisitWrite: VisitOutput<Result<(), io::Error>>
```

Source: `src/field/mod.rs:116`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait implemented by visitors to indicate that they write to an
`io::Write` instance, and allow access to that writer.

<a id="op-c69ed5f1c812b88cbd4a3672"></a>
## writer

`function` · `tracing_subscriber::field::VisitWrite::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn io::Write
```

Source: `src/field/mod.rs:118`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the writer that this visitor writes to.
