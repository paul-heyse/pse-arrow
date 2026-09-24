# `tracing_subscriber::field::VisitOutput`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.VisitOutput.json).

<a id="op-a9438d4636538f1184679957"></a>
## VisitOutput

`trait` · `tracing_subscriber::field::VisitOutput` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait VisitOutput<Out>: Visit
```

Source: `src/field/mod.rs:37`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [visitor] that produces output once it has visited a set of fields.

[visitor]: tracing_core::field::Visit

<a id="op-a47bc81aea630fb6114e059a"></a>
## finish

`function` · `tracing_subscriber::field::VisitOutput::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> Out
```

Source: `src/field/mod.rs:41`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Completes the visitor, returning any output.

This is called once a full set of fields has been visited.

<a id="op-02bcc7ca888e900d331639e6"></a>
## visit

`function` · `tracing_subscriber::field::VisitOutput::visit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn visit<R>(self, fields: &R) -> Out where R: RecordFields, Self: Sized
```

Source: `src/field/mod.rs:45`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a set of fields, and return the output of finishing the visitor
once the fields have been visited.
