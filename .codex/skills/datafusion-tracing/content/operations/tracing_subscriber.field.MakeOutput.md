# `tracing_subscriber::field::MakeOutput`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.MakeOutput.json).

<a id="op-e736deed0d4e3d1a031b37ab"></a>
## MakeOutput

`trait` · `tracing_subscriber::field::MakeOutput` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait MakeOutput<T, Out> where Self: MakeVisitor<T> + sealed::Sealed<(T, Out)>, Self::Visitor: VisitOutput<Out>
```

Source: `src/field/mod.rs:95`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait implemented for all `MakeVisitor` implementations that
produce a visitor implementing `VisitOutput`.

<a id="op-d5afef9240589ceba0446398"></a>
## visit_with

`function` · `tracing_subscriber::field::MakeOutput::visit_with` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn visit_with<F>(&self, target: T, fields: &F) -> Out where F: RecordFields
```

Source: `src/field/mod.rs:102`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visits all fields in `fields` with a new visitor constructed from
`target`.
